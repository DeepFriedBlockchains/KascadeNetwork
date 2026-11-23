#![cfg(feature = "phase6-staging-preview")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::{blockstatus::BlockStatus::StatusUTXOValid, config::params::BlockrateParams, header::Header};
use kaspa_consensusmanager::ConsensusManager;
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_rpc_core::api::rpc::RpcApi;
use kaspa_testing_integration::common::daemon::Daemon;
use kaspad_lib::args::Args;
use std::{sync::Arc, time::Duration};
use tokio::time::{Instant, MissedTickBehavior, interval, sleep, timeout};

const BASELINE_SAMPLE_BLOCKS: usize = 50;
const SUSTAINED_SAMPLE_BLOCKS: usize = 1_500;
const COLLECTION_INTERVAL: Duration = Duration::from_millis(5);

fn manager(node: &Daemon) -> Arc<ConsensusManager> {
    let service = node.core.find(ConsensusManager::IDENT).unwrap();
    Arc::downcast::<ConsensusManager>(service.into_any_arc()).unwrap()
}

async fn await_block(manager: &ConsensusManager, hash: Hash) {
    timeout(Duration::from_secs(30), async {
        loop {
            let active = manager.consensus();
            if active.unguarded_session_blocking().get_block_status(hash) == Some(StatusUTXOValid) {
                return;
            }
            sleep(COLLECTION_INTERVAL).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("peer did not validate {hash} within the sample deadline"));
}

async fn connect(client: &GrpcClient, sender: &Daemon) {
    client.add_peer(format!("127.0.0.1:{}", sender.p2p_port).try_into().unwrap(), true).await.unwrap();
    timeout(Duration::from_secs(15), async {
        while client.get_connected_peer_info().await.unwrap().peer_info.is_empty() {
            sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("sample nodes did not connect");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "opt-in 5-BPS measurement on an isolated runner"]
async fn phase6_five_bps_loopback_baseline() {
    run_five_bps_loopback(BASELINE_SAMPLE_BLOCKS).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "opt-in five-minute 5-BPS measurement on an isolated runner"]
async fn phase6_five_bps_loopback_five_minutes() {
    run_five_bps_loopback(SUSTAINED_SAMPLE_BLOCKS).await;
}

#[cfg(target_os = "linux")]
fn process_resources() -> serde_json::Value {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap();
    // The command field can contain spaces and parentheses. Fields after
    // the final parenthesis start with field 3 (state).
    let fields = stat.rsplit_once(')').unwrap().1.split_whitespace().collect::<Vec<_>>();
    let user_ticks = fields[11].parse::<u64>().unwrap();
    let system_ticks = fields[12].parse::<u64>().unwrap();
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let memory = |key: &str| {
        let field = status.lines().find_map(|line| line.strip_prefix(key)).unwrap();
        field.split_whitespace().next().unwrap().parse::<u64>().unwrap()
    };
    serde_json::json!({
        "user_ticks": user_ticks,
        "system_ticks": system_ticks,
        "rss_kib": memory("VmRSS:"),
        "process_peak_rss_kib": memory("VmHWM:"),
    })
}

async fn run_five_bps_loopback(sample_blocks: usize) {
    kaspa_alloc::init_allocator_with_default_settings();
    let profile = BlockrateParams::new::<5>();
    let block_period = Duration::from_millis(profile.target_time_per_block);
    let temp = tempfile::tempdir().unwrap();
    let params = temp.path().join("five-bps.json");
    let config = serde_json::json!({"blockrate": profile, "skip_proof_of_work": true, "shielded_coinbase": false});
    std::fs::write(&params, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
    let args = Args {
        simnet: true,
        unsafe_rpc: true,
        enable_unsynced_mining: true,
        disable_upnp: true,
        disable_dns_seeding: true,
        outbound_target: 0,
        override_params_file: Some(params.to_string_lossy().into_owned()),
        ..Default::default()
    };
    let mut sender = Daemon::new_random_with_args(args.clone(), 10);
    let mut first = Daemon::new_random_with_args(args.clone(), 10);
    let mut second = Daemon::new_random_with_args(args, 10);
    let submit = sender.start().await;
    let first_client = first.start().await;
    let second_client = second.start().await;
    connect(&first_client, &sender).await;
    connect(&second_client, &sender).await;

    let first_manager = manager(&first);
    let second_manager = manager(&second);
    let miner = Address::new(sender.network.into(), Version::PubKey, &[0; 32]);
    let mut clock = interval(block_period);
    clock.set_missed_tick_behavior(MissedTickBehavior::Skip);
    #[cfg(target_os = "linux")]
    let before = process_resources();
    let start = Instant::now();
    let mut samples = Vec::with_capacity(sample_blocks);
    let mut final_tip = None;

    for index in 0..sample_blocks {
        clock.tick().await;
        let block = submit.get_block_template(miner.clone(), vec![]).await.unwrap().block;
        let hash = Header::try_from(&block.header).unwrap().hash;
        let submitted = Instant::now();
        let first_manager = first_manager.clone();
        let second_manager = second_manager.clone();
        samples.push(tokio::spawn(async move {
            let observe = |manager: Arc<ConsensusManager>| async move {
                await_block(&manager, hash).await;
                submitted.elapsed().as_micros()
            };
            let (first_us, second_us) = tokio::join!(observe(first_manager), observe(second_manager));
            (index, hash, submitted.duration_since(start).as_micros(), first_us, second_us)
        }));
        let accepted = submit.submit_block(block, false).await.unwrap();
        assert!(accepted.report.is_success(), "sample block rejected: {}", accepted.reject_detail);
        final_tip = Some(hash);
    }

    println!("phase6_config,{config}");
    println!("index,block_hash,submit_offset_us,first_valid_us,second_valid_us");
    for sample in samples {
        let (index, hash, submit_us, first_us, second_us) = sample.await.unwrap();
        println!("{index},{hash},{submit_us},{first_us},{second_us}");
    }
    let tip = final_tip.expect("measurement should submit blocks");
    for client in [&submit, &first_client, &second_client] {
        timeout(Duration::from_secs(10), async {
            while client.get_block_dag_info().await.unwrap().sink != tip {
                sleep(COLLECTION_INTERVAL).await;
            }
        })
        .await
        .expect("sample node did not select the final tip");
    }
    #[cfg(target_os = "linux")]
    {
        let elapsed_us = start.elapsed().as_micros();
        let after = process_resources();
        let resources = serde_json::json!({"elapsed_us": elapsed_us, "before": before, "after": after});
        println!("phase6_resources,{resources}");
    }
    for client in [&submit, &first_client, &second_client] {
        client.disconnect().await.unwrap();
    }
    println!("phase6_tip,{tip}");
    drop(first_manager);
    drop(second_manager);
    second.shutdown();
    first.shutdown();
    sender.shutdown();
}
