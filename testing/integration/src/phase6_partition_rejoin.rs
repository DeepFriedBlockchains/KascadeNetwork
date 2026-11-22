use crate::common::daemon::Daemon;
use kaspa_addresses::{Address, Version};
use kaspa_alloc::init_allocator_with_default_settings;
use kaspa_consensus::params::{SIMNET_GENESIS, SIMNET_PARAMS};
use kaspa_consensus_core::{
    blockstatus::BlockStatus::StatusUTXOValid,
    config::params::OverrideParams,
    tx::{TransactionOutpoint, UtxoEntry},
};
use kaspa_consensusmanager::ConsensusManager;
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_rpc_core::api::rpc::RpcApi;
use kaspad_lib::args::Args;
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

const FD_LIMIT: i32 = 10;

async fn mine(client: &GrpcClient, miner: &Address, count: usize) -> Hash {
    for _ in 0..count {
        let template = client.get_block_template(miner.clone(), vec![]).await.unwrap();
        client.submit_block(template.block, false).await.unwrap();
    }
    client.get_block_dag_info().await.unwrap().sink
}

async fn wait_for_sink(client: &GrpcClient, expected: Hash) {
    tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            let info = client.get_block_dag_info().await.unwrap();
            if info.sink == expected {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("node did not reach expected sink {expected}"));
}

async fn wait_for_peer_count(client: &GrpcClient, expected: usize) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if client.get_connected_peer_info().await.unwrap().peer_info.len() == expected {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("node did not reach expected peer count {expected}"));
}

async fn connect(client: &GrpcClient, peer: &Daemon, permanent: bool) {
    client.add_peer(format!("127.0.0.1:{}", peer.p2p_port).try_into().unwrap(), permanent).await.unwrap();
    wait_for_peer_count(client, 1).await;
}

fn manager(daemon: &Daemon) -> Arc<ConsensusManager> {
    let service = daemon.core.find(ConsensusManager::IDENT).unwrap();
    Arc::downcast::<ConsensusManager>(service.into_any_arc()).unwrap()
}

#[derive(Debug, PartialEq, Eq)]
struct Checkpoint {
    sink: Hash,
    pruning_point: Hash,
    finality_point: Hash,
    utxo_commitment: Hash,
    shielded_commitment: [u8; 32],
    utxos: HashMap<TransactionOutpoint, UtxoEntry>,
}

async fn checkpoint(daemon: &Daemon, requires_promotion: bool) -> Checkpoint {
    let manager = manager(daemon);
    tokio::task::spawn_blocking(move || {
        let active = manager.consensus();
        let session = active.unguarded_session_blocking();
        let sink = session.get_sink();
        assert_eq!(session.get_block_status(sink), Some(StatusUTXOValid));
        assert!(session.is_pruning_shielded_stable());
        assert!(!session.supports_verified_staging_import());
        assert!(!manager.is_staging_import_pending());
        if requires_promotion {
            assert!(session.has_verified_staging_promotion().unwrap());
        }
        let utxos = session.get_virtual_utxos(None, 512, false);
        assert!(utxos.len() < 512, "the fixture exceeded its full UTXO comparison limit");
        Checkpoint {
            sink,
            pruning_point: session.pruning_point(),
            finality_point: session.finality_point(),
            utxo_commitment: session.get_header(sink).unwrap().utxo_commitment,
            shielded_commitment: session.get_shielded_state_root(sink).unwrap(),
            utxos: utxos.into_iter().collect(),
        }
    })
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn authority_integration_preview_three_node_partition_rejoin() {
    init_allocator_with_default_settings();
    kaspa_core::log::try_init_logger("INFO");
    let params_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/params/shielded_ibd_sync_test_params.json");
    let overrides: OverrideParams = serde_json::from_str(&std::fs::read_to_string(&params_file).unwrap()).unwrap();
    let params = SIMNET_PARAMS.override_params(overrides);
    let args = Args {
        simnet: true,
        unsafe_rpc: true,
        enable_unsynced_mining: true,
        disable_upnp: true,
        disable_dns_seeding: true,
        outbound_target: 0,
        utxoindex: true,
        override_params_file: Some(params_file.to_string_lossy().into_owned()),
        ..Default::default()
    };
    let mut first = Daemon::new_random_with_args(args.clone(), FD_LIMIT);
    let first_client = first.start().await;
    let first_recipient = kaspa_shielded_core::wallet::address_bytes_from_seed([7; 32]).unwrap();
    let first_miner = Address::new(first.network.into(), Version::ShieldedOrchard, &first_recipient);

    mine(&first_client, &first_miner, (params.coinbase_maturity() + params.pruning_depth()) as usize + 80).await;
    for _ in 0..params.finality_depth() + 200 {
        let info = first_client.get_block_dag_info().await.unwrap();
        if first_client.get_block(info.pruning_point_hash, false).await.unwrap().header.blue_score > params.finality_depth() {
            break;
        }
        mine(&first_client, &first_miner, 1).await;
    }
    let baseline = checkpoint(&first, false).await;
    assert_ne!(baseline.pruning_point, SIMNET_GENESIS.hash);
    assert!(
        first_client.get_block(baseline.pruning_point, false).await.unwrap().header.blue_score > params.finality_depth(),
        "sender never reached the pruning-proof fixture depth"
    );

    let mut second = Daemon::new_random_with_args(args.clone(), FD_LIMIT);
    let second_client = second.start().await;
    connect(&second_client, &first, true).await;
    wait_for_sink(&second_client, baseline.sink).await;
    assert_eq!(checkpoint(&second, true).await, baseline);

    let mut third = Daemon::new_random_with_args(args, FD_LIMIT);
    let third_client = third.start().await;
    let third_recipient = kaspa_shielded_core::wallet::address_bytes_from_seed([8; 32]).unwrap();
    let third_miner = Address::new(third.network.into(), Version::ShieldedOrchard, &third_recipient);
    connect(&third_client, &first, false).await;
    wait_for_sink(&third_client, baseline.sink).await;
    assert_eq!(checkpoint(&third, true).await, baseline);

    let isolated_ip = "127.0.0.1".parse().unwrap();
    third_client.ban(isolated_ip).await.unwrap();
    wait_for_peer_count(&third_client, 0).await;
    wait_for_peer_count(&first_client, 1).await;
    wait_for_peer_count(&second_client, 1).await;

    let minority_tip = mine(&third_client, &third_miner, 3).await;
    let majority_tip = mine(&first_client, &first_miner, 20).await;
    assert_ne!(minority_tip, baseline.sink);
    assert_ne!(majority_tip, baseline.sink);
    assert_ne!(minority_tip, majority_tip);
    wait_for_sink(&second_client, majority_tip).await;
    assert_eq!(third_client.get_block_dag_info().await.unwrap().sink, minority_tip);

    third_client.unban(isolated_ip).await.unwrap();
    connect(&third_client, &first, true).await;
    let announced = mine(&first_client, &first_miner, 1).await;
    tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            let sink = third_client.get_block_dag_info().await.unwrap().sink;
            if sink == majority_tip || sink == announced {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("third node did not validate the pre-rejoin branch");

    let third_manager = manager(&third);
    tokio::task::spawn_blocking(move || {
        let active = third_manager.consensus();
        let session = active.unguarded_session_blocking();
        assert_eq!(session.get_block_status(majority_tip), Some(StatusUTXOValid));
        let change = session.get_virtual_chain_from_block(minority_tip, None).unwrap();
        assert!(change.removed.contains(&minority_tip), "the competing selected tip was never rolled back");
        assert!(change.added.contains(&majority_tip), "the winning branch was never applied");
    })
    .await
    .unwrap();

    // Let the rejoined node relay a checkpoint from its reconciled DAG.
    let converged_tip = mine(&third_client, &third_miner, 2).await;
    wait_for_sink(&first_client, converged_tip).await;
    wait_for_sink(&second_client, converged_tip).await;
    let converged = checkpoint(&first, false).await;
    assert_eq!(checkpoint(&second, true).await, converged);
    assert_eq!(checkpoint(&third, true).await, converged);
    assert_ne!(converged.shielded_commitment, baseline.shielded_commitment);

    third_client.disconnect().await.unwrap();
    drop(third_client);
    third = third.restart(FD_LIMIT);
    let third_client = third.start().await;
    wait_for_peer_count(&third_client, 0).await;
    assert_eq!(checkpoint(&third, true).await, converged);
    connect(&third_client, &first, true).await;
    let post_restart_tip = mine(&first_client, &first_miner, 1).await;
    wait_for_sink(&third_client, post_restart_tip).await;
    wait_for_sink(&second_client, post_restart_tip).await;
    let post_restart = checkpoint(&first, false).await;
    assert_eq!(checkpoint(&second, true).await, post_restart);
    assert_eq!(checkpoint(&third, true).await, post_restart);

    first_client.disconnect().await.unwrap();
    second_client.disconnect().await.unwrap();
    third_client.disconnect().await.unwrap();
    first.shutdown();
    second.shutdown();
    third.shutdown();
}
