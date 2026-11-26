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
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
    task::JoinSet,
    time::{Instant, sleep, sleep_until, timeout},
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

const SUSTAINED_INTERVAL: Duration = Duration::from_secs(1);
const SUSTAINED_CONNECTED_BLOCKS: u32 = 61;
const SUSTAINED_OFFLINE_BLOCKS: u32 = 10;
const SUSTAINED_REJOINED_BLOCKS: u32 = 5;

async fn mine_paced_native_blocks(
    leader: &GrpcClient,
    followers: &[&GrpcClient],
    phase: &str,
    count: u32,
    mut tip: Hash,
) -> (Hash, Duration, Duration) {
    let mut next_start = Instant::now();
    let mut accepted = next_start;
    let mut first_accepted = accepted;
    let mut max_tip_wait = Duration::ZERO;
    for sequence in 1..=count {
        sleep_until(next_start).await;
        let block = timeout(Duration::from_secs(15), submit_real_pow_block(leader, false))
            .await
            .unwrap_or_else(|_| panic!("{phase} block {sequence} could not be mined and submitted within 15 seconds"));
        assert_ne!(tip, block);
        tip = block;
        accepted = Instant::now();
        next_start = accepted + SUSTAINED_INTERVAL;
        if sequence == 1 {
            first_accepted = accepted;
        }
        for client in std::iter::once(leader).chain(followers.iter().copied()) {
            expect_tip(client, tip).await;
        }
        let tip_wait = accepted.elapsed();
        max_tip_wait = max_tip_wait.max(tip_wait);
        let observation = serde_json::json!({
            "phase": phase,
            "sequence": sequence,
            "tip": tip.to_string(),
            "accepted_since_phase_start_ms": accepted.duration_since(first_accepted).as_millis(),
            "all_expected_nodes_selected_ms": tip_wait.as_millis(),
        });
        eprintln!("phase6_native_paced_block,{observation}");
    }
    (tip, accepted.duration_since(first_accepted), max_tip_wait)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires a built kaspad binary and isolated Linux runner"]
async fn phase6_loopback_cli_sustained_native_pow() {
    let binary = std::env::var_os("KASCADE_CLI_KASPAD").unwrap();
    let binary = Path::new(&binary);
    let logs = std::env::var_os("KASCADE_CLI_LOGS").unwrap();
    let logs = Path::new(&logs);
    let data = tempfile::tempdir().unwrap();
    let params = logs.join("native-pow-params.json");
    std::fs::write(&params, r#"{"skip_proof_of_work":false,"shielded_coinbase":false}"#).unwrap();
    let options = NodeOptions { params: Some(&params), ..Default::default() };

    let mut a = Node::start(binary, data.path(), logs, 1, "a-sustained", true, options);
    let a_rpc = connect(19110).await;
    let mut b = Node::start(binary, data.path(), logs, 2, "b-sustained", false, options);
    let b_rpc = connect(19210).await;
    let mut c = Node::start(binary, data.path(), logs, 3, "c-sustained", false, options);
    let c_rpc = connect(19310).await;
    expect_peers(&a_rpc, 2).await;
    let genesis = expect_tip(&a_rpc, SIMNET_GENESIS.hash).await;
    for (name, client) in [("a", &a_rpc), ("b", &b_rpc), ("c", &c_rpc)] {
        assert_eq!(client.get_current_network().await.unwrap(), NetworkType::Simnet);
        let dag = expect_tip(client, SIMNET_GENESIS.hash).await;
        assert_eq!(dag.block_count, genesis.block_count);
        assert_eq!(dag.pruning_point_hash, SIMNET_GENESIS.hash);
        assert_eq!(dag.virtual_daa_score, genesis.virtual_daa_score);
        record_node_metrics(client, name, "genesis").await;
    }

    let (connected_tip, connected_span, connected_max_wait) =
        mine_paced_native_blocks(&a_rpc, &[&b_rpc, &c_rpc], "connected", SUSTAINED_CONNECTED_BLOCKS, SIMNET_GENESIS.hash).await;
    let before = expect_tip(&c_rpc, connected_tip).await;
    assert_eq!(before.block_count, genesis.block_count + u64::from(SUSTAINED_CONNECTED_BLOCKS));
    record_node_metrics(&c_rpc, "c", "before_stop").await;
    c_rpc.disconnect().await.unwrap();
    c.stop().await;
    expect_peers(&a_rpc, 1).await;

    let offline_started = Instant::now();
    let (offline_tip, offline_span, _) =
        mine_paced_native_blocks(&a_rpc, &[&b_rpc], "offline", SUSTAINED_OFFLINE_BLOCKS, connected_tip).await;
    let offline_duration = offline_started.elapsed();
    let reopen_started = Instant::now();
    let mut c = Node::start(binary, data.path(), logs, 3, "c-sustained-reopen", false, options);
    let c_rpc = connect(19310).await;
    let reopen_rpc_ready = reopen_started.elapsed();
    expect_peers(&a_rpc, 2).await;
    expect_peers(&c_rpc, 1).await;
    let recovered = expect_tip(&c_rpc, offline_tip).await;
    let reopen_converged = reopen_started.elapsed();
    assert_eq!(recovered.block_count - before.block_count, u64::from(SUSTAINED_OFFLINE_BLOCKS));

    let (tip, rejoined_span, _) =
        mine_paced_native_blocks(&a_rpc, &[&b_rpc, &c_rpc], "rejoined", SUSTAINED_REJOINED_BLOCKS, offline_tip).await;
    let authority = expect_tip(&a_rpc, tip).await;
    let accepted_total = u64::from(SUSTAINED_CONNECTED_BLOCKS + SUSTAINED_OFFLINE_BLOCKS + SUSTAINED_REJOINED_BLOCKS);
    assert_eq!(authority.block_count - genesis.block_count, accepted_total);
    for (name, client) in [("a", &a_rpc), ("b", &b_rpc), ("c", &c_rpc)] {
        let dag = expect_tip(client, tip).await;
        assert_eq!(dag.block_count, authority.block_count);
        assert_eq!(dag.pruning_point_hash, authority.pruning_point_hash);
        assert_eq!(dag.virtual_daa_score, authority.virtual_daa_score);
        record_node_metrics(client, name, "converged").await;
        client.disconnect().await.unwrap();
    }
    c.stop().await;
    b.stop().await;
    a.stop().await;

    let result = serde_json::json!({
        "target_submission_interval_ms": SUSTAINED_INTERVAL.as_millis(),
        "accepted_blocks": accepted_total,
        "connected_blocks": SUSTAINED_CONNECTED_BLOCKS,
        "connected_first_to_last_accept_ms": connected_span.as_millis(),
        "connected_observed_bps": f64::from(SUSTAINED_CONNECTED_BLOCKS - 1) / connected_span.as_secs_f64(),
        "connected_max_all_expected_nodes_selected_ms": connected_max_wait.as_millis(),
        "missed_blocks": recovered.block_count - before.block_count,
        "offline_first_to_last_accept_ms": offline_span.as_millis(),
        "offline_workload_ms": offline_duration.as_millis(),
        "reopen_to_rpc_ready_ms": reopen_rpc_ready.as_millis(),
        "reopen_to_tip_convergence_ms": reopen_converged.as_millis(),
        "rejoined_blocks": SUSTAINED_REJOINED_BLOCKS,
        "rejoined_first_to_last_accept_ms": rejoined_span.as_millis(),
        "final_tip": tip.to_string(),
        "final_block_count": authority.block_count,
        "final_pruning_point": authority.pruning_point_hash.to_string(),
        "final_virtual_daa_score": authority.virtual_daa_score,
    });
    std::fs::write(logs.join("sustained.json"), serde_json::to_string_pretty(&result).unwrap()).unwrap();
    eprintln!("phase6_native_sustained,{result}");
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
