#![cfg(feature = "phase6-staging-preview")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::{
    auxpow::AuxPow, header::Header, merkle::calc_hash_merkle_root, subnets::SUBNETWORK_ID_COINBASE, tx::Transaction,
};
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_pow::State;
use kaspa_rpc_core::{SubmitBlockRejectReason, SubmitBlockReport, api::rpc::RpcApi};
use kaspa_testing_integration::common::daemon::Daemon;
use kaspad_lib::args::Args;
use std::time::Duration;

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

async fn submit_native_block(client: &GrpcClient, miner: &Address, reject_invalid_nonce: bool) -> Hash {
    let mut block = client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
    let candidate = Header::try_from(&block.header).unwrap();
    let state = State::new(&candidate);
    assert!(candidate.aux_pow.is_none(), "this fixture must exercise native proof of work");

    if reject_invalid_nonce {
        let mut invalid = block.clone();
        invalid.header.nonce =
            (0..16_384).find(|nonce| !state.check_pow(*nonce).0).expect("fixture should have a nonce above its target");
        let response = client.submit_block(invalid, false).await.unwrap();
        assert!(matches!(response.report, SubmitBlockReport::Reject(SubmitBlockRejectReason::BlockInvalid)));
    }

    block.header.nonce = (0..16_384).find(|nonce| state.check_pow(*nonce).0).expect("fixture should mine an easy native nonce");
    let expected = Header::try_from(&block.header).unwrap().hash;
    let response = client.submit_block(block, false).await.unwrap();
    assert!(response.report.is_success(), "valid native proof of work rejected: {}", response.reject_detail);
    wait_for_sink(client, expected).await;
    expected
}

async fn submit_auxpow_block(client: &GrpcClient, miner: &Address) -> Hash {
    let mut block = client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
    let candidate = Header::try_from(&block.header).unwrap();
    let native = State::new(&candidate);
    block.header.nonce = (0..16_384).find(|nonce| !native.check_pow(*nonce).0).expect("child must fail native proof of work");
    let child = Header::try_from(&block.header).unwrap();

    let commitment = AuxPow::embed_commitment(&[], child.hash, &[]);
    let coinbase = Transaction::new(0, vec![], vec![], 0, SUBNETWORK_ID_COINBASE, 0, commitment);
    let mut parent = Header::from_precomputed_hash(Hash::from_bytes([0; 32]), vec![Hash::from_bytes([1; 32])]);
    parent.hash_merkle_root = calc_hash_merkle_root(std::iter::once(&coinbase));
    parent.bits = child.bits;
    parent.timestamp = child.timestamp;
    let work = State::new(&parent);
    parent.nonce = (0..16_384).find(|nonce| work.check_pow(*nonce).0).expect("fixture should mine an easy parent nonce");
    parent.finalize();

    let expected = child.hash;
    block.header = child
        .with_aux_pow(AuxPow { parent_header: parent, parent_coinbase: coinbase, coinbase_merkle_branch: vec![] })
        .into();
    assert_eq!(Header::try_from(&block.header).unwrap().hash, expected);
    let result = client.submit_block(block, false).await.unwrap();
    assert!(result.report.is_success(), "valid auxiliary proof of work rejected: {}", result.reject_detail);
    wait_for_sink(client, expected).await;
    expected
}

async fn run_pow_relay(auxpow: bool) {
    kaspa_alloc::init_allocator_with_default_settings();
    let temp = tempfile::tempdir().unwrap();
    let params = temp.path().join("native-pow.json");
    std::fs::write(&params, r#"{"skip_proof_of_work": false, "shielded_coinbase": false}"#).unwrap();
    let args = Args {
        simnet: true,
        enable_unsynced_mining: true,
        unsafe_rpc: true,
        disable_upnp: true,
        disable_dns_seeding: true,
        outbound_target: 0,
        override_params_file: Some(params.to_string_lossy().into_owned()),
        ..Default::default()
    };
    let mut first = Daemon::new_random_with_args(args.clone(), 10);
    let mut second = Daemon::new_random_with_args(args, 10);
    let first_client = first.start().await;
    let second_client = second.start().await;
    let miner = Address::new(first.network.into(), Version::PubKey, &[0; 32]);
    second_client.add_peer(format!("127.0.0.1:{}", first.p2p_port).try_into().unwrap(), true).await.unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        while second_client.get_connected_peer_info().await.unwrap().peer_info.is_empty() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Simnet nodes did not connect");

    let first_hash = submit_native_block(&first_client, &miner, true).await;
    wait_for_sink(&second_client, first_hash).await;
    let second_hash = if auxpow {
        submit_auxpow_block(&second_client, &miner).await
    } else {
        submit_native_block(&second_client, &miner, false).await
    };
    assert_ne!(first_hash, second_hash);
    wait_for_sink(&first_client, second_hash).await;

    first_client.disconnect().await.unwrap();
    second_client.disconnect().await.unwrap();
    first.shutdown();
    second.shutdown();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn authority_integration_native_pow_rejects_invalid_nonce_and_relays_valid_blocks() {
    run_pow_relay(false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn authority_integration_auxpow_relays_block_without_native_proof() {
    run_pow_relay(true).await;
}
