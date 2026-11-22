#![cfg(feature = "phase6-staging-preview")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::header::Header;
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

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn authority_integration_native_pow_rejects_invalid_nonce_and_relays_valid_blocks() {
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
    let second_hash = submit_native_block(&second_client, &miner, false).await;
    assert_ne!(first_hash, second_hash);
    wait_for_sink(&first_client, second_hash).await;

    first_client.disconnect().await.unwrap();
    second_client.disconnect().await.unwrap();
    first.shutdown();
    second.shutdown();
}
