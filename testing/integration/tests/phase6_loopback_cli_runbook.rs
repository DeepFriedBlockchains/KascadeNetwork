#![cfg(target_os = "linux")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::{config::genesis::SIMNET_GENESIS, header::Header, network::NetworkType};
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_pow::State;
use kaspa_rpc_core::{
    GetBlockDagInfoResponse, SubmitBlockRejectReason, SubmitBlockReport, api::rpc::RpcApi, notify::mode::NotificationMode,
};
use std::{
    fs::File,
    path::Path,
    process::{Child, Command},
    time::Duration,
};
use tokio::time::{sleep, timeout};

struct Node(Child);

impl Node {
    fn start(
        binary: &Path,
        data_root: &Path,
        logs: &Path,
        index: u16,
        label: &str,
        test_mining: bool,
        params: Option<&Path>,
    ) -> Self {
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

        if let Some(params) = params {
            command.arg(format!("--override-params-file={}", params.display()));
            command.args(["--perf-metrics", "--perf-metrics-interval-sec=1"]);
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

    let mut a = Node::start(binary, data.path(), log_dir, 1, "a-start", false, None);
    let a_rpc = connect(19110).await;
    let mut b = Node::start(binary, data.path(), log_dir, 2, "b-start", false, None);
    let b_rpc = connect(19210).await;
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-start", false, None);
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
    let mut a = Node::start(binary, data.path(), log_dir, 1, "a-test-mining", true, None);
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
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-reopen", false, None);
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

async fn record_node_metrics(client: &GrpcClient, node: &str, stage: &str) {
    let report = client.get_metrics(true, true, true, false, true, false).await.unwrap();
    let process = report.process_metrics.expect("node must return requested process metrics");
    let bandwidth = report.bandwidth_metrics.expect("node must return requested bandwidth metrics");
    let storage = report.storage_metrics.expect("node must return requested storage metrics");
    let row = serde_json::json!({
        "node": node,
        "stage": stage,
        "server_time": report.server_time,
        "cpu_usage": process.cpu_usage,
        "rss_bytes": process.resident_set_size,
        "disk_io_read_bytes": process.disk_io_read_bytes,
        "disk_io_write_bytes": process.disk_io_write_bytes,
        "storage_size_bytes": storage.storage_size_bytes,
        "p2p_bytes_sent": bandwidth.p2p_bytes_tx,
        "p2p_bytes_received": bandwidth.p2p_bytes_rx,
    });
    eprintln!("phase6_node_metrics,{row}");
}

async fn submit_real_pow_block(client: &GrpcClient, prove_pow_enabled: bool) -> Hash {
    let address = Address::new(NetworkType::Simnet.into(), Version::PubKey, &[0; 32]);
    let mut block = client.get_block_template(address, vec![]).await.unwrap().block;
    let candidate = Header::try_from(&block.header).unwrap();
    let state = State::new(&candidate);
    let native_limit = 16_384;
    assert!(candidate.aux_pow.is_none());

    if prove_pow_enabled {
        let mut invalid = block.clone();
        invalid.header.nonce = (0..native_limit).find(|nonce| !state.check_pow(*nonce).0).expect("need an above-target nonce");
        let rejected = client.submit_block(invalid, false).await.unwrap();
        assert!(matches!(rejected.report, SubmitBlockReport::Reject(SubmitBlockRejectReason::BlockInvalid)));
    }

    block.header.nonce = (0..native_limit).find(|nonce| state.check_pow(*nonce).0).expect("need an easy valid nonce");
    let hash = Header::try_from(&block.header).unwrap().hash;
    let duplicate = prove_pow_enabled.then(|| block.clone());
    let accepted = client.submit_block(block, false).await.unwrap();
    assert!(accepted.report.is_success(), "real Simnet proof rejected: {}", accepted.reject_detail);
    if let Some(duplicate) = duplicate {
        let before = expect_tip(client, hash).await;
        let replay = client.submit_block(duplicate, false).await.unwrap();
        let after = expect_tip(client, hash).await;
        assert_eq!(after.block_count, before.block_count);
        assert_eq!(after.virtual_daa_score, before.virtual_daa_score);
        assert_eq!(after.pruning_point_hash, before.pruning_point_hash);
        eprintln!("duplicate native-PoW submission left DAG state unchanged: {:?}", replay.report);
    }
    hash
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires a built kaspad binary and isolated Linux runner"]
async fn phase6_loopback_cli_real_pow_recovery() {
    let binary = std::env::var_os("KASCADE_CLI_KASPAD").unwrap();
    let binary = Path::new(&binary);
    let log_dir = std::env::var_os("KASCADE_CLI_LOGS").unwrap();
    let log_dir = Path::new(&log_dir);
    let data = tempfile::tempdir().unwrap();
    let params_file = data.path().join("real-pow.json");
    std::fs::write(&params_file, r#"{"skip_proof_of_work":false,"shielded_coinbase":false}"#).unwrap();
    let params = Some(params_file.as_path());

    let mut a = Node::start(binary, data.path(), log_dir, 1, "a-real-pow", true, params);
    let a_rpc = connect(19110).await;
    let mut b = Node::start(binary, data.path(), log_dir, 2, "b-real-pow", false, params);
    let b_rpc = connect(19210).await;
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-real-pow", false, params);
    let c_rpc = connect(19310).await;
    expect_peers(&a_rpc, 2).await;

    let genesis = expect_tip(&a_rpc, SIMNET_GENESIS.hash).await;
    for client in [&a_rpc, &b_rpc, &c_rpc] {
        assert_eq!(client.get_current_network().await.unwrap(), NetworkType::Simnet);
        let dag = expect_tip(client, SIMNET_GENESIS.hash).await;
        assert_eq!(dag.block_count, genesis.block_count);
        assert_eq!(dag.pruning_point_hash, SIMNET_GENESIS.hash);
        assert_eq!(dag.virtual_daa_score, genesis.virtual_daa_score);
    }

    for (name, client) in [("a", &a_rpc), ("b", &b_rpc), ("c", &c_rpc)] {
        record_node_metrics(client, name, "genesis").await;
    }

    let mut tip = SIMNET_GENESIS.hash;
    for i in 0..6 {
        let next = submit_real_pow_block(&a_rpc, i == 0).await;
        assert_ne!(next, tip);
        tip = next;
        for client in [&a_rpc, &b_rpc, &c_rpc] {
            expect_tip(client, tip).await;
        }
    }

    let before = expect_tip(&c_rpc, tip).await;
    record_node_metrics(&c_rpc, "c", "before_stop").await;
    c_rpc.disconnect().await.unwrap();
    c.stop().await;
    let offline_started = std::time::Instant::now();
    expect_peers(&a_rpc, 1).await;
    for _ in 0..6 {
        let next = submit_real_pow_block(&a_rpc, false).await;
        assert_ne!(next, tip);
        tip = next;
        for client in [&a_rpc, &b_rpc] {
            expect_tip(client, tip).await;
        }
    }

    let offline_ms = offline_started.elapsed().as_millis();
    let reopen_started = std::time::Instant::now();
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-real-pow-reopen", false, params);
    let c_rpc = connect(19310).await;
    let rpc_ready_ms = reopen_started.elapsed().as_millis();
    expect_peers(&a_rpc, 2).await;
    let recovered = expect_tip(&c_rpc, tip).await;
    let converged_ms = reopen_started.elapsed().as_millis();
    let authority = expect_tip(&a_rpc, tip).await;
    let peer = expect_tip(&b_rpc, tip).await;
    assert_eq!(before.block_count, genesis.block_count + 6);
    assert_eq!(authority.block_count, genesis.block_count + 12);
    assert_eq!(recovered.block_count, before.block_count + 6);
    for dag in [&peer, &recovered] {
        assert_eq!(dag.block_count, authority.block_count);
        assert_eq!(dag.pruning_point_hash, authority.pruning_point_hash);
        assert_eq!(dag.virtual_daa_score, authority.virtual_daa_score);
    }
    let recovery = serde_json::json!({
        "follower": "c",
        "accepted_blocks": authority.block_count - genesis.block_count,
        "missed_blocks": recovered.block_count - before.block_count,
        "final_block_count": authority.block_count,
        "final_pruning_point": authority.pruning_point_hash.to_string(),
        "final_virtual_daa_score": authority.virtual_daa_score,
        "offline_ms": offline_ms,
        "reopen_to_rpc_ready_ms": rpc_ready_ms,
        "reopen_to_tip_convergence_ms": converged_ms,
        "final_tip": tip.to_string(),
    });
    eprintln!("phase6_recovery,{recovery}");

    for (name, client) in [("a", &a_rpc), ("b", &b_rpc), ("c", &c_rpc)] {
        record_node_metrics(client, name, "converged").await;
        client.disconnect().await.unwrap();
    }
    c.stop().await;
    b.stop().await;
    a.stop().await;
}
