#![cfg(target_os = "linux")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::{config::genesis::SIMNET_GENESIS, header::Header, network::NetworkType};
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_pow::State;
use kaspa_rpc_core::{
    GetBlockDagInfoResponse, RpcPeerAddress, SubmitBlockRejectReason, SubmitBlockReport, api::rpc::RpcApi,
    notify::mode::NotificationMode,
};
use std::{
    fs::File,
    net::SocketAddr,
    path::Path,
    process::{Child, Command},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
    task::JoinSet,
    time::{Instant, sleep, timeout},
};

#[derive(Clone, Copy, Default)]
struct NodeOptions<'a> {
    params: Option<&'a Path>,
    upstream_port: Option<u16>,
}

struct Node(Child);

impl Node {
    fn start(
        binary: &Path,
        data_root: &Path,
        logs: &Path,
        index: u16,
        label: &str,
        test_mining: bool,
        options: NodeOptions<'_>,
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
            command.arg(format!("--connect=127.0.0.1:{}", options.upstream_port.unwrap_or(19111)));
        }
        if test_mining {
            command.arg("--enable-unsynced-mining");
        }

        if let Some(params) = options.params {
            command.arg(format!("--override-params-file={}", params.display()));
            command.args(["--perf-metrics", "--perf-metrics-interval-sec=1"]);
        }
        Self::spawn(command, logs, label)
    }

    fn start_discovery(
        binary: &Path,
        data_root: &Path,
        logs: &Path,
        index: u16,
        label: &str,
        bootstrap: Option<SocketAddr>,
        outbound_target: usize,
    ) -> Self {
        let base_port = 19_000 + index * 100;
        let mut command = Command::new(binary);
        command
            .args(["--simnet", "--nodnsseed", "--disable-upnp"])
            .arg(format!("--appdir={}", data_root.join(format!("node-{index}")).display()))
            .arg(format!("--listen=10.203.0.{}:{}", 10 + index, base_port + 11))
            .arg(format!("--rpclisten=127.0.0.1:{}", base_port + 10))
            .arg(format!("--outpeers={outbound_target}"));
        if let Some(bootstrap) = bootstrap {
            command.arg(format!("--addpeer={bootstrap}"));
        }
        Self::spawn(command, logs, label)
    }

    fn spawn(mut command: Command, logs: &Path, label: &str) -> Self {
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
    let mut last_count = None;
    timeout(Duration::from_secs(60), async {
        loop {
            let connected = client.get_connected_peer_info().await.unwrap().peer_info.len();
            if connected == count {
                return;
            }
            last_count = Some(connected);
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("loopback peer topology did not converge: expected {count}, last observed {last_count:?}"));
}

async fn expect_tip(client: &GrpcClient, tip: Hash) -> GetBlockDagInfoResponse {
    let mut last_seen = None;
    timeout(Duration::from_secs(60), async {
        loop {
            let info = client.get_block_dag_info().await.unwrap();
            if info.sink == tip {
                return info;
            }
            last_seen = Some((info.sink, info.block_count));
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("selected DAG tip did not converge: expected {tip}, last observed (tip, block count) {last_seen:?}"))
}

async fn expect_known_peer(client: &GrpcClient, address: RpcPeerAddress) {
    timeout(Duration::from_secs(15), async {
        loop {
            if client.get_peer_addresses().await.unwrap().known_addresses.contains(&address) {
                return;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("address book did not learn {address}"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the isolated private-peer Linux network namespace and built kaspad"]
async fn phase6_private_peer_discovery() {
    let namespace = std::env::var_os("KASCADE_CLI_DISCOVERY_NAMESPACE").expect("start via the private discovery namespace script");
    assert_eq!(std::fs::read_link("/proc/self/ns/net").unwrap().as_os_str(), namespace);
    let binary = std::env::var_os("KASCADE_CLI_KASPAD").unwrap();
    let log_dir = std::env::var_os("KASCADE_CLI_LOGS").unwrap();
    let (binary, logs) = (Path::new(&binary), Path::new(&log_dir));
    let data = tempfile::tempdir().unwrap();
    let (hub, advertised) = ("10.203.0.11:19111".parse::<SocketAddr>().unwrap(), "10.203.0.12:19211".parse::<SocketAddr>().unwrap());

    let mut b = Node::start_discovery(binary, data.path(), logs, 2, "b", None, 0);
    let b_rpc = connect(19210).await;
    let mut a = Node::start_discovery(binary, data.path(), logs, 1, "a", Some(advertised), 0);
    let a_rpc = connect(19110).await;
    expect_known_peer(&a_rpc, advertised.into()).await;

    // A is the only bootstrap input to C and C starts with no saved address book.
    assert!(!data.path().join("node-3").exists());
    let mut c = Node::start_discovery(binary, data.path(), logs, 3, "c-discovery", Some(hub), 2);
    let c_rpc = connect(19310).await;
    expect_known_peer(&c_rpc, advertised.into()).await;
    timeout(Duration::from_secs(75), async {
        loop {
            let peers = c_rpc.get_connected_peer_info().await.unwrap().peer_info;
            if [hub, advertised].into_iter().all(|address| peers.iter().any(|peer| peer.is_outbound && peer.address == address.into()))
            {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("C did not connect outbound to both its sole bootstrap and the discovered peer");

    for client in [&a_rpc, &b_rpc, &c_rpc] {
        assert_eq!(client.get_current_network().await.unwrap(), NetworkType::Simnet);
        assert_eq!(expect_tip(client, SIMNET_GENESIS.hash).await.pruning_point_hash, SIMNET_GENESIS.hash);
        client.disconnect().await.unwrap();
    }
    c.stop().await;
    a.stop().await;
    b.stop().await;
    eprintln!("phase6_private_discovery: C learned {advertised} through {hub} and handshook outbound with both in Simnet");
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

    let mut a = Node::start(binary, data.path(), log_dir, 1, "a-start", false, NodeOptions::default());
    let a_rpc = connect(19110).await;
    let mut b = Node::start(binary, data.path(), log_dir, 2, "b-start", false, NodeOptions::default());
    let b_rpc = connect(19210).await;
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-start", false, NodeOptions::default());
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
    let mut a = Node::start(binary, data.path(), log_dir, 1, "a-test-mining", true, NodeOptions::default());
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
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-reopen", false, NodeOptions::default());
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
    let report = timeout(Duration::from_secs(10), async {
        loop {
            let report = client.get_metrics(true, true, true, false, true, false).await.unwrap();
            if report.process_metrics.as_ref().expect("node must return requested process metrics").resident_set_size > 0 {
                return report;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("node {node} did not publish a process sample at {stage}"));
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
    let params_file = log_dir.join("native-pow-params.json");
    std::fs::write(&params_file, r#"{"skip_proof_of_work":false,"shielded_coinbase":false}"#).unwrap();
    let options = NodeOptions { params: Some(params_file.as_path()), ..Default::default() };

    let mut a = Node::start(binary, data.path(), log_dir, 1, "a-real-pow", true, options);
    let a_rpc = connect(19110).await;
    let mut b = Node::start(binary, data.path(), log_dir, 2, "b-real-pow", false, options);
    let b_rpc = connect(19210).await;
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-real-pow", false, options);
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
    let mut c = Node::start(binary, data.path(), log_dir, 3, "c-real-pow-reopen", false, options);
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

async fn forward_peer(listener: Arc<TcpListener>) -> std::io::Result<()> {
    loop {
        let (mut downstream, _) = listener.accept().await?;
        if let Ok(mut upstream) = TcpStream::connect("127.0.0.1:19111").await {
            // At most one forwarded session, with no queue of forwarding tasks.
            let _ = copy_bidirectional(&mut downstream, &mut upstream).await;
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires a built kaspad binary and isolated Linux runner"]
async fn phase6_loopback_cli_live_peer_reconnect() {
    let binary = std::env::var_os("KASCADE_CLI_KASPAD").unwrap();
    let binary = Path::new(&binary);
    let logs = std::env::var_os("KASCADE_CLI_LOGS").unwrap();
    let logs = Path::new(&logs);
    let data = tempfile::tempdir().unwrap();
    let params = logs.join("native-pow-params.json");
    std::fs::write(&params, r#"{"skip_proof_of_work":false,"shielded_coinbase":false}"#).unwrap();
    let options = NodeOptions { params: Some(&params), ..Default::default() };
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
    let follower_options = NodeOptions { upstream_port: Some(listener.local_addr().unwrap().port()), ..options };
    let mut bridge = JoinSet::new();
    bridge.spawn(forward_peer(listener.clone()));

    let mut a = Node::start(binary, data.path(), logs, 1, "a-reconnect", true, options);
    let a_rpc = connect(19110).await;
    let mut b = Node::start(binary, data.path(), logs, 2, "b-reconnect", false, options);
    let b_rpc = connect(19210).await;
    let mut c = Node::start(binary, data.path(), logs, 3, "c-reconnect", false, follower_options);
    let c_rpc = connect(19310).await;
    expect_peers(&a_rpc, 2).await;
    expect_peers(&c_rpc, 1).await;
    for client in [&a_rpc, &b_rpc, &c_rpc] {
        expect_tip(client, SIMNET_GENESIS.hash).await;
    }

    let first = submit_real_pow_block(&a_rpc, true).await;
    expect_tip(&a_rpc, first).await;
    expect_tip(&b_rpc, first).await;
    let before = expect_tip(&c_rpc, first).await;
    record_node_metrics(&c_rpc, "c", "before_partition").await;

    let partition_started = Instant::now();
    bridge.shutdown().await;
    expect_peers(&a_rpc, 1).await;
    expect_peers(&c_rpc, 0).await;
    let mut tip = first;
    for _ in 0..3 {
        tip = submit_real_pow_block(&a_rpc, false).await;
        expect_tip(&a_rpc, tip).await;
        expect_tip(&b_rpc, tip).await;
    }
    let isolated = c_rpc.get_block_dag_info().await.unwrap();
    assert_eq!(isolated.sink, first);
    assert_eq!(isolated.block_count, before.block_count);
    assert!(c.0.try_wait().unwrap().is_none(), "follower must stay alive during link interruption");

    let forwarding_resumed = Instant::now();
    bridge.spawn(forward_peer(listener.clone()));
    expect_peers(&a_rpc, 2).await;
    expect_peers(&c_rpc, 1).await;
    let connected_after = forwarding_resumed.elapsed();
    let caught_up = expect_tip(&c_rpc, tip).await;
    let caught_up_after = forwarding_resumed.elapsed();
    assert!(caught_up_after >= connected_after);
    assert_eq!(caught_up.block_count, before.block_count + 3);
    for peer in [&a_rpc, &b_rpc] {
        let dag = expect_tip(peer, tip).await;
        assert_eq!(dag.block_count, caught_up.block_count);
        assert_eq!(dag.pruning_point_hash, caught_up.pruning_point_hash);
        assert_eq!(dag.virtual_daa_score, caught_up.virtual_daa_score);
    }
    for (name, client) in [("a", &a_rpc), ("b", &b_rpc), ("c", &c_rpc)] {
        record_node_metrics(client, name, "reconnected").await;
    }

    c_rpc.disconnect().await.unwrap();
    c.stop().await;
    bridge.shutdown().await;
    expect_peers(&a_rpc, 1).await;
    let reopen_started = Instant::now();
    let mut c = Node::start(binary, data.path(), logs, 3, "c-reconnect-reopen", false, follower_options);
    let c_rpc = connect(19310).await;
    expect_peers(&c_rpc, 0).await;
    let reopened = expect_tip(&c_rpc, tip).await;
    expect_peers(&c_rpc, 0).await;
    assert_eq!(reopened.block_count, caught_up.block_count);
    assert_eq!(reopened.pruning_point_hash, caught_up.pruning_point_hash);
    assert_eq!(reopened.virtual_daa_score, caught_up.virtual_daa_score);
    let recovery = serde_json::json!({
        "interruption_ms": forwarding_resumed.duration_since(partition_started).as_millis(),
        "reconnect_ms": connected_after.as_millis(),
        "catch_up_ms": caught_up_after.as_millis(),
        "offline_reopen_ms": reopen_started.elapsed().as_millis(),
        "synced_blocks": caught_up.block_count - before.block_count,
        "final_tip": tip.to_string(),
    });
    eprintln!("phase6_peer_recovery,{recovery}");

    for client in [&a_rpc, &b_rpc, &c_rpc] {
        client.disconnect().await.unwrap();
    }
    c.stop().await;
    b.stop().await;
    a.stop().await;
}
