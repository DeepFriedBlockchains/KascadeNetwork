use kaspa_rpc_core::{RpcError, api::rpc::RpcApi};
use kaspa_testing_integration::common::daemon::Daemon;
use kaspad_lib::args::Args;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn safe_mode_rejects_grpc_admin_operations() {
    kaspa_alloc::init_allocator_with_default_settings();
    let args = Args { simnet: true, disable_upnp: true, disable_dns_seeding: true, outbound_target: 0, ..Default::default() };
    assert!(!args.unsafe_rpc, "the fixture must exercise the default RPC permission gate");
    let mut daemon = Daemon::new_random_with_args(args, 10);
    let client = daemon.start().await;

    let safe_mode = RpcError::UnavailableInSafeMode.to_string();
    let result = tokio::time::timeout(Duration::from_secs(15), async {
        let peer_error = client.add_peer("127.0.0.1:1".try_into().unwrap(), false).await.unwrap_err();
        assert_eq!(peer_error.to_string(), safe_mode);
        let shutdown_error = client.shutdown().await.unwrap_err();
        assert_eq!(shutdown_error.to_string(), safe_mode);
        client.get_block_dag_info().await.expect("public RPC must work after rejecting admin calls");
    })
    .await;

    client.disconnect().await.unwrap();
    daemon.shutdown();
    result.expect("the daemon must promptly reject privileged RPC requests");
}
