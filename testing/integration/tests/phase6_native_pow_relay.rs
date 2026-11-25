#![cfg(feature = "phase6-staging-preview")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::{config::params::SIMNET_PARAMS, header::Header};
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_math::{Uint256, Uint320};
use kaspa_pow::State;
use kaspa_rpc_core::{RpcRawBlock, SubmitBlockRejectReason, SubmitBlockReport, api::rpc::RpcApi};
use kaspa_testing_integration::common::daemon::{ClientManager, Daemon};
use kaspa_utils::networking::ContextualNetAddress;
use kaspa_wrpc_server::address::WrpcNetAddress;
use kaspad_lib::args::Args;
use std::{path::Path, sync::Arc, time::Duration};

fn native_simnet_args(params: &Path) -> Args {
    Args {
        simnet: true,
        enable_unsynced_mining: true,
        unsafe_rpc: true,
        disable_upnp: true,
        disable_dns_seeding: true,
        outbound_target: 0,
        override_params_file: Some(params.to_string_lossy().into_owned()),
        ..Default::default()
    }
}

async fn wait_for_sink(client: &GrpcClient, expected: Hash) {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if client.get_block_dag_info().await.unwrap().sink == expected {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("node did not validate native proof-of-work block {expected}"));
}

fn solve_native_block(block: &mut RpcRawBlock) -> Hash {
    let candidate = Header::try_from(&block.header).unwrap();
    assert!(candidate.aux_pow.is_none());
    let state = State::new(&candidate);
    block.header.nonce = (0..16_384).find(|nonce| state.check_pow(*nonce).0).expect("fixture should mine an easy native nonce");
    Header::try_from(&block.header).unwrap().hash
}

async fn connect(first: &Daemon, second_client: &GrpcClient) {
    second_client.add_peer(format!("127.0.0.1:{}", first.p2p_port).try_into().unwrap(), true).await.unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        while second_client.get_connected_peer_info().await.unwrap().peer_info.is_empty() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Simnet nodes did not connect");
}

async fn submit_solved_block(client: &GrpcClient, block: RpcRawBlock, expected: Hash) {
    let response = client.submit_block(block, false).await.unwrap();
    assert!(response.report.is_success(), "valid native proof of work rejected: {}", response.reject_detail);
    wait_for_sink(client, expected).await;
}

async fn submit_native_block(client: &GrpcClient, miner: &Address, reject_invalid_nonce: bool) -> Hash {
    let mut block = client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
    if reject_invalid_nonce {
        let candidate = Header::try_from(&block.header).unwrap();
        let state = State::new(&candidate);
        let mut invalid = block.clone();
        invalid.header.nonce =
            (0..16_384).find(|nonce| !state.check_pow(*nonce).0).expect("fixture should have a nonce above its target");
        let response = client.submit_block(invalid, false).await.unwrap();
        assert!(matches!(response.report, SubmitBlockReport::Reject(SubmitBlockRejectReason::BlockInvalid)));
    }

    let expected = solve_native_block(&mut block);
    submit_solved_block(client, block, expected).await;
    expected
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn authority_integration_native_pow_rejects_invalid_nonce_and_relays_valid_blocks() {
    kaspa_alloc::init_allocator_with_default_settings();
    let temp = tempfile::tempdir().unwrap();
    let params = temp.path().join("native-pow.json");
    std::fs::write(&params, r#"{"skip_proof_of_work": false, "shielded_coinbase": false}"#).unwrap();
    let args = native_simnet_args(&params);
    let mut first = Daemon::new_random_with_args(args.clone(), 10);
    let mut second = Daemon::new_random_with_args(args, 10);
    let first_client = first.start().await;
    let second_client = second.start().await;
    let miner = Address::new(first.network.into(), Version::PubKey, &[0; 32]);
    connect(&first, &second_client).await;

    let first_hash = submit_native_block(&first_client, &miner, true).await;
    wait_for_sink(&second_client, first_hash).await;
    let second_hash = submit_native_block(&second_client, &miner, false).await;
    assert_ne!(first_hash, second_hash);
    wait_for_sink(&first_client, second_hash).await;

    first_client.disconnect().await.unwrap();
    second_client.disconnect().await.unwrap();
    first.shutdown();
    second.shutdown();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn authority_integration_native_pow_reopens_after_difficulty_retarget() {
    kaspa_alloc::init_allocator_with_default_settings();
    const WINDOW: u64 = 8;
    let temp = tempfile::tempdir().unwrap();
    let params = temp.path().join("native-pow-retarget.json");
    let mut blockrate = SIMNET_PARAMS.blockrate.clone();
    blockrate.difficulty_sample_rate = 1;
    let target_interval = blockrate.target_time_per_block;
    let fast_interval = target_interval * 3 / 4;
    let profile = serde_json::json!({
        "skip_proof_of_work": false,
        "shielded_coinbase": false,
        "difficulty_window_size": WINDOW,
        "min_difficulty_window_size": WINDOW,
        "blockrate": blockrate
    });
    std::fs::write(&params, serde_json::to_vec(&profile).unwrap()).unwrap();

    let args = native_simnet_args(&params);
    let mut first = Daemon::new_random_with_args(args.clone(), 10);
    let mut second = Daemon::new_random_with_args(args, 10);
    let first_client = first.start().await;
    let second_client = second.start().await;
    let miner = Address::new(first.network.into(), Version::PubKey, &[0; 32]);
    connect(&first, &second_client).await;

    let genesis = &SIMNET_PARAMS.genesis;
    assert_eq!(first_client.get_block_dag_info().await.unwrap().sink, genesis.hash);
    assert_eq!(second_client.get_block_dag_info().await.unwrap().sink, genesis.hash);
    let first_template = first_client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
    // Keep the whole deterministic sequence behind wall-clock time and above genesis.
    let start = first_template.header.timestamp.checked_sub(10_000).unwrap();
    assert!(start > genesis.timestamp);
    for index in 0..WINDOW {
        let mut block = first_client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
        assert_eq!(block.header.bits, genesis.bits, "retargeted before the window filled");
        block.header.timestamp = start + index * fast_interval;
        let hash = solve_native_block(&mut block);
        submit_solved_block(&first_client, block, hash).await;
        wait_for_sink(&second_client, hash).await;
    }

    let previous = first_client.get_block_dag_info().await.unwrap().sink;
    let mut retargeted = first_client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
    let peer_template = second_client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
    let old_target = Uint256::from_compact_target_bits(genesis.bits);
    // The first full sampled window has identical old targets and evenly spaced timestamps.
    let expected_target = Uint256::try_from(Uint320::from(old_target) * fast_interval / target_interval).unwrap();
    let expected_bits = expected_target.compact_target_bits();
    assert_ne!(expected_bits, genesis.bits);
    assert_eq!(retargeted.header.bits, expected_bits);
    assert_eq!(peer_template.header.bits, expected_bits);
    assert_eq!(peer_template.header.daa_score, retargeted.header.daa_score);

    retargeted.header.timestamp = start + WINDOW * fast_interval;
    let mut stale = retargeted.clone();
    stale.header.bits = genesis.bits;
    let stale_hash = solve_native_block(&mut stale);
    let rejected = first_client.submit_block(stale, false).await.unwrap();
    assert_eq!(rejected.report, SubmitBlockReport::Reject(SubmitBlockRejectReason::BlockInvalid));
    assert!(
        rejected
            .reject_detail
            .contains(&format!("block {stale_hash} difficulty of {} is not the expected value of {expected_bits}", genesis.bits)),
        "{}",
        rejected.reject_detail
    );
    assert_eq!(first_client.get_block_dag_info().await.unwrap().sink, previous);

    let valid_hash = solve_native_block(&mut retargeted);
    submit_solved_block(&first_client, retargeted, valid_hash).await;
    wait_for_sink(&second_client, valid_hash).await;
    assert_eq!(second_client.get_block(valid_hash, false).await.unwrap().header.bits, expected_bits);

    // The next rolling window drops its oldest block, then excludes the
    // oldest timestamp from the average. Six old targets and the decoded
    // first retarget remain.
    let first_retarget = Uint256::from_compact_target_bits(expected_bits);
    let average_target = (Uint320::from(old_target) * (WINDOW - 2) + Uint320::from(first_retarget)) / (WINDOW - 1);
    let next_bits = Uint256::try_from(average_target * fast_interval / target_interval).unwrap().compact_target_bits();
    assert_ne!(next_bits, expected_bits);
    let observed = first_client.get_block_template(miner.clone(), vec![]).await.unwrap().block.header;
    assert_eq!(observed.bits, next_bits);

    second_client.disconnect().await.unwrap();
    drop(second_client);
    second = second.restart(10);
    let second_client = second.start().await;
    assert!(second_client.get_connected_peer_info().await.unwrap().peer_info.is_empty());
    assert_eq!(second_client.get_block_dag_info().await.unwrap().sink, valid_hash);
    assert_eq!(second_client.get_block(valid_hash, false).await.unwrap().header.bits, expected_bits);

    let mut next = second_client.get_block_template(miner, vec![]).await.unwrap().block;
    assert_eq!(next.header.parents_by_level[0], [valid_hash]);
    assert_eq!(next.header.bits, next_bits);
    assert_eq!(next.header.daa_score, observed.daa_score);
    next.header.timestamp = start + (WINDOW + 1) * fast_interval;
    let returned = solve_native_block(&mut next);

    connect(&first, &second_client).await;
    submit_solved_block(&second_client, next, returned).await;
    wait_for_sink(&first_client, returned).await;
    assert_eq!(first_client.get_block(returned, false).await.unwrap().header.bits, next_bits);

    first_client.disconnect().await.unwrap();
    second_client.disconnect().await.unwrap();
    first.shutdown();
    second.shutdown();
}

fn loopback_daemon(mut args: Args) -> Daemon {
    Daemon::fill_args_with_random_ports(&mut args);
    let loopback = |address: ContextualNetAddress| ContextualNetAddress::loopback().with_port(address.normalize(0).port);
    args.listen = args.listen.map(loopback);
    args.rpclisten = args.rpclisten.map(loopback);
    let Some(WrpcNetAddress::Custom(borsh)) = args.rpclisten_borsh else {
        panic!("the daemon fixture must allocate a Borsh RPC port");
    };
    args.rpclisten_borsh = Some(WrpcNetAddress::Custom(loopback(borsh)));
    args.rpclisten_json = None;
    Daemon::with_manager(Arc::new(ClientManager::new(args)), 10)
}

async fn wait_for_peer_count(client: &GrpcClient, expected: usize) {
    tokio::time::timeout(Duration::from_secs(15), async {
        while client.get_connected_peer_info().await.unwrap().peer_info.len() != expected {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("node did not reach {expected} connected peers"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn authority_integration_native_pow_competing_branches_rejoin() {
    kaspa_alloc::init_allocator_with_default_settings();
    let temp = tempfile::tempdir().unwrap();
    let params = temp.path().join("native-pow-partition.json");
    std::fs::write(&params, r#"{"skip_proof_of_work": false, "shielded_coinbase": false}"#).unwrap();
    let args = native_simnet_args(&params);
    let mut first = loopback_daemon(args.clone());
    let mut second = loopback_daemon(args.clone());
    let mut isolated = loopback_daemon(args);
    let first_client = first.start().await;
    let second_client = second.start().await;
    let isolated_client = isolated.start().await;
    let miner = Address::new(first.network.into(), Version::PubKey, &[0; 32]);
    let isolated_miner = Address::new(isolated.network.into(), Version::PubKey, &[1; 32]);
    let first_address = format!("127.0.0.1:{}", first.p2p_port).try_into().unwrap();

    connect(&first, &second_client).await;
    // Only the isolated node can request this connection. A temporary request allows it to close the link.
    isolated_client.add_peer(first_address, false).await.unwrap();
    wait_for_peer_count(&first_client, 2).await;
    wait_for_peer_count(&isolated_client, 1).await;
    let common = submit_native_block(&first_client, &miner, false).await;
    wait_for_sink(&second_client, common).await;
    wait_for_sink(&isolated_client, common).await;

    let loopback_ip = "127.0.0.1".parse().unwrap();
    isolated_client.ban(loopback_ip).await.unwrap();
    wait_for_peer_count(&isolated_client, 0).await;
    wait_for_peer_count(&first_client, 1).await;
    let isolated_tip = submit_native_block(&isolated_client, &isolated_miner, false).await;
    let mut connected_branch = Vec::new();
    for _ in 0..4 {
        let block = submit_native_block(&first_client, &miner, false).await;
        wait_for_sink(&second_client, block).await;
        connected_branch.push(block);
    }
    let connected_tip = *connected_branch.last().unwrap();
    assert_eq!(isolated_client.get_block_dag_info().await.unwrap().sink, isolated_tip);
    assert!(isolated_client.get_block(connected_tip, false).await.is_err());
    assert!(first_client.get_block(isolated_tip, false).await.is_err());

    isolated_client.unban(loopback_ip).await.unwrap();
    isolated_client.add_peer(first_address, false).await.unwrap();
    wait_for_peer_count(&first_client, 2).await;
    wait_for_sink(&isolated_client, connected_tip).await;
    let rollback = isolated_client.get_virtual_chain_from_block(isolated_tip, false, None).await.unwrap();
    assert_eq!(rollback.removed_chain_block_hashes, [isolated_tip]);
    assert_eq!(rollback.added_chain_block_hashes, connected_branch);

    let recovered_tip = submit_native_block(&isolated_client, &isolated_miner, false).await;
    let recovered = isolated_client.get_block_dag_info().await.unwrap();
    for client in [&first_client, &second_client] {
        wait_for_sink(client, recovered_tip).await;
        let dag = client.get_block_dag_info().await.unwrap();
        assert_eq!(dag.virtual_parent_hashes, recovered.virtual_parent_hashes);
        assert_eq!(dag.virtual_daa_score, recovered.virtual_daa_score);
        assert_eq!(dag.pruning_point_hash, recovered.pruning_point_hash);
    }

    first_client.disconnect().await.unwrap();
    second_client.disconnect().await.unwrap();
    isolated_client.disconnect().await.unwrap();
}
