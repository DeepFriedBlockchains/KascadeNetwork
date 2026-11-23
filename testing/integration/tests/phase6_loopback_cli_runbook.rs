#![cfg(target_os = "linux")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::{config::genesis::SIMNET_GENESIS, header::Header, network::NetworkType};
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_rpc_core::{GetBlockDagInfoResponse, api::rpc::RpcApi, notify::mode::NotificationMode};
use std::{
    fs::File,
    path::Path,
    process::{Child, Command},
    time::Duration,
};
use tokio::time::{sleep, timeout};

struct Node(Child);

impl Node {
    fn start(binary: &Path, data_root: &Path, logs: &Path, index: u16, label: &str, test_mining: bool) -> Self {
        let name = ["a", "b", "c"][usize::from(index - 1)];
        let base_port = 19_000 + index * 100;
        let mut command = Command::new(binary);
        command
            .arg("--simnet")
            .arg(format!("--appdir={}", data_root.join(format!("phase6-node-{name}")).display()))
            .arg(format!("--listen=127.0.0.1:{}", base_port + 11))
            .arg(format!("--rpclisten=127.0.0.1:{}", base_port + 10))
            .args(["--nodnsseed", "--disable-upnp"]);
        if index == 1 {
            command.arg("--outpeers=0");
        } else {
            command.arg("--connect=127.0.0.1:19111");
        }
        if test_mining {
            command.arg("--enable-unsynced-mining");
        }

        let log = File::create(logs.join(format!("{label}.log"))).unwrap();
        command.stdout(log.try_clone().unwrap()).stderr(log);
        eprintln!("starting {label}: {command:?}");
        Self(command.spawn().unwrap())
    }

    async fn stop(&mut self) {
        let pid = self.0.id().to_string();
        assert!(Command::new("kill").args(["-INT", &pid]).status().unwrap().success());
        let status = timeout(Duration::from_secs(30), async {
            loop {
                if let Some(status) = self.0.try_wait().unwrap() {
                    return status;
                }
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .expect("kaspad did not exit after SIGINT");
        assert!(status.success(), "kaspad exited with {status}");
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

async fn connect(rpc_port: u16) -> GrpcClient {
    timeout(Duration::from_secs(60), async {
        loop {
            if let Ok(client) = GrpcClient::connect_with_args(
                NotificationMode::Direct,
                format!("grpc://127.0.0.1:{rpc_port}"),
                None,
                false,
                None,
                false,
                Some(500_000),
                Default::default(),
            )
            .await
            {
                return client;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("loopback gRPC did not start")
}

async fn expect_peers(client: &GrpcClient, count: usize) {
    timeout(Duration::from_secs(60), async {
        loop {
            if client.get_connected_peer_info().await.unwrap().peer_info.len() == count {
                return;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("loopback peer topology did not converge");
}

async fn expect_tip(client: &GrpcClient, tip: Hash) -> GetBlockDagInfoResponse {
    timeout(Duration::from_secs(60), async {
        loop {
            let info = client.get_block_dag_info().await.unwrap();
            if info.sink == tip {
                return info;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("selected DAG tip did not converge")
}

async fn submit_block(client: &GrpcClient) -> Hash {
    let address = Address::new(NetworkType::Simnet.into(), Version::PubKey, &[0; 32]);
    let block = client.get_block_template(address, vec![]).await.unwrap().block;
    let hash = Header::try_from(&block.header).unwrap().hash;
    let response = client.submit_block(block, false).await.unwrap();
    assert!(response.report.is_success(), "Simnet block rejected: {}", response.reject_detail);
    hash
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires a built kaspad binary and isolated Linux runner"]
async fn phase6_loopback_cli_runbook() {
    let binary = std::env::var_os("KASCADE_CLI_KASPAD").unwrap();
    let binary = Path::new(&binary);
    let log_dir = std::env::var_os("KASCADE_CLI_LOGS").unwrap();
    let log_dir = Path::new(&log_dir);
    let data = tempfile::tempdir().unwrap();

    let mut a = Node::start(binary, data.path(), log_dir, 1, "a-start", false);
    let a_rpc = connect(19110).await;
    let mut b = Node::start(binary, data.path(), log_dir, 2, "b-start", false);
    let b_rpc = connect(19210).await;
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-start", false);
    let c_rpc = connect(19310).await;

    expect_peers(&a_rpc, 2).await;
    for client in [&a_rpc, &b_rpc, &c_rpc] {
        assert_eq!(client.get_current_network().await.unwrap(), NetworkType::Simnet);
        let dag = expect_tip(client, SIMNET_GENESIS.hash).await;
        assert_eq!(dag.pruning_point_hash, SIMNET_GENESIS.hash);
    }
    eprintln!("documented startup commands selected matching Simnet genesis and formed a two-follower topology");

    a_rpc.disconnect().await.unwrap();
    a.stop().await;
    let mut a = Node::start(binary, data.path(), log_dir, 1, "a-test-mining", true);
    let a_rpc = connect(19110).await;
    expect_peers(&a_rpc, 2).await;
    let first = submit_block(&a_rpc).await;
    let before = expect_tip(&c_rpc, first).await;
    expect_tip(&b_rpc, first).await;
    c_rpc.disconnect().await.unwrap();
    c.stop().await;
    expect_peers(&a_rpc, 1).await;

    let second = submit_block(&a_rpc).await;
    assert_ne!(first, second);
    expect_tip(&b_rpc, second).await;
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-reopen", false);
    let c_rpc = connect(19310).await;
    expect_peers(&a_rpc, 2).await;
    let recovered = expect_tip(&c_rpc, second).await;
    let authority = expect_tip(&a_rpc, second).await;
    assert!(recovered.block_count > before.block_count);
    assert_eq!(recovered.pruning_point_hash, authority.pruning_point_hash);
    assert_eq!(recovered.virtual_daa_score, authority.virtual_daa_score);
    eprintln!("reopened follower caught up to tip {second} and matched the authority's pruning point and virtual DAA");

    for client in [&a_rpc, &b_rpc, &c_rpc] {
        client.disconnect().await.unwrap();
    }
    c.stop().await;
    b.stop().await;
    a.stop().await;
    eprintln!("all three daemon processes exited successfully after SIGINT");
}
