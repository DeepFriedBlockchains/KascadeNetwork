#![cfg(feature = "phase6-staging-preview")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus_core::{
    blockstatus::BlockStatus::StatusUTXOValid,
    config::params::BlockrateParams,
    constants::TX_VERSION_TOCCATA,
    header::Header,
    sign::sign,
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{MutableTransaction, Transaction, TransactionInput, TransactionOutput},
};
use kaspa_consensusmanager::ConsensusManager;
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_rpc_core::api::rpc::RpcApi;
use kaspa_testing_integration::common::{
    daemon::{ClientManager, Daemon},
    fee,
    utils::fetch_spendable_utxos,
};
use kaspa_txscript::pay_to_address_script;
use kaspa_utils::networking::ContextualNetAddress;
use kaspa_wrpc_server::address::WrpcNetAddress;
use kaspad_lib::args::Args;
use secp256k1::{Keypair, SECP256K1};
use std::{collections::BTreeSet, sync::Arc, time::Duration};
use tokio::time::{Instant, MissedTickBehavior, interval, sleep, timeout};

const BASELINE_SAMPLE_BLOCKS: usize = 50;
const SUSTAINED_SAMPLE_BLOCKS: usize = 1_500;
const SIGNED_TRAFFIC_SAMPLE_BLOCKS: usize = 8;
const SIGNED_TRAFFIC_PER_BLOCK: usize = 2;
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
    run_five_bps_loopback(BASELINE_SAMPLE_BLOCKS, 0).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "opt-in five-minute 5-BPS measurement on an isolated runner"]
async fn phase6_five_bps_loopback_five_minutes() {
    run_five_bps_loopback(SUSTAINED_SAMPLE_BLOCKS, 0).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "opt-in signed transparent transaction smoke measurement on three isolated Simnet peers"]
async fn phase6_five_bps_loopback_signed_traffic() {
    run_five_bps_loopback(SIGNED_TRAFFIC_SAMPLE_BLOCKS, SIGNED_TRAFFIC_PER_BLOCK).await;
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

async fn prepare_signed_traffic(
    client: &GrpcClient,
    miner: &Address,
    key: Keypair,
    coinbase_maturity: u64,
    transaction_count: usize,
) -> (Hash, Vec<Transaction>) {
    let mut tip = None;
    // Continue past normal maturity so each measured spend has its own mature input.
    for _ in 0..coinbase_maturity as usize + transaction_count + 2 {
        let block = client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
        tip = Some(Header::try_from(&block.header).unwrap().hash);
        let accepted = client.submit_block(block, false).await.unwrap();
        assert!(accepted.report.is_success(), "warmup block rejected: {}", accepted.reject_detail);
    }

    let utxos = timeout(Duration::from_secs(10), async {
        loop {
            let utxos = fetch_spendable_utxos(client, miner.clone(), coinbase_maturity).await;
            if utxos.len() >= transaction_count {
                break utxos;
            }
            sleep(COLLECTION_INTERVAL).await;
        }
    })
    .await
    .expect("not enough mature coinbase outputs for the workload");
    let script_public_key = pay_to_address_script(miner);
    let transactions = utxos
        .into_iter()
        .take(transaction_count)
        .map(|(outpoint, entry)| {
            assert!(entry.is_coinbase);
            let value = entry.amount.checked_sub(fee::calc_for_plain_standard_tx(1, 1)).expect("test coinbase must cover the fee");
            let tx = Transaction::new(
                TX_VERSION_TOCCATA,
                vec![TransactionInput::new(outpoint, vec![], 0, 1)],
                vec![TransactionOutput { value, script_public_key: script_public_key.clone(), covenant: None }],
                0,
                SUBNETWORK_ID_NATIVE,
                0,
                vec![],
            );
            let mut signed = sign(MutableTransaction::with_entries(tx, vec![entry]), key).tx;
            signed.finalize();
            signed
        })
        .collect();
    (tip.expect("signed traffic requires a nonempty warmup"), transactions)
}

async fn await_tip(client: &GrpcClient, tip: Hash) {
    timeout(Duration::from_secs(10), async {
        while client.get_block_dag_info().await.unwrap().sink != tip {
            sleep(COLLECTION_INTERVAL).await;
        }
    })
    .await
    .expect("sample node did not select the expected tip");
}

fn signed_traffic_daemon(mut args: Args) -> Daemon {
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

async fn run_five_bps_loopback(sample_blocks: usize, transactions_per_block: usize) {
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
        utxoindex: transactions_per_block > 0,
        override_params_file: Some(params.to_string_lossy().into_owned()),
        ..Default::default()
    };
    let start_daemon = |args| {
        if transactions_per_block > 0 { signed_traffic_daemon(args) } else { Daemon::new_random_with_args(args, 10) }
    };
    let mut sender = start_daemon(args.clone());
    let mut first = start_daemon(args.clone());
    let mut second = start_daemon(args);
    let submit = sender.start().await;
    let first_client = first.start().await;
    let second_client = second.start().await;
    connect(&first_client, &sender).await;
    connect(&second_client, &sender).await;

    let first_manager = manager(&first);
    let second_manager = manager(&second);
    let miner_key = Keypair::from_seckey_slice(SECP256K1, &[9; 32]).unwrap();
    let miner_public_key = miner_key.x_only_public_key().0.serialize();
    let miner =
        Address::new(sender.network.into(), Version::PubKey, if transactions_per_block > 0 { &miner_public_key } else { &[0; 32] });
    let traffic = if transactions_per_block > 0 {
        let (tip, transactions) =
            prepare_signed_traffic(&submit, &miner, miner_key, profile.coinbase_maturity, sample_blocks * transactions_per_block)
                .await;
        tokio::join!(await_tip(&first_client, tip), await_tip(&second_client, tip));
        transactions
    } else {
        Vec::new()
    };
    let mut clock = interval(block_period);
    clock.set_missed_tick_behavior(MissedTickBehavior::Skip);
    #[cfg(target_os = "linux")]
    let before = process_resources();
    let start = Instant::now();
    let mut samples = Vec::with_capacity(sample_blocks);
    let mut final_tip = None;

    for index in 0..sample_blocks {
        clock.tick().await;
        let offset = index * transactions_per_block;
        let batch = &traffic[offset..offset + transactions_per_block];
        for tx in batch {
            let id = submit.submit_transaction(tx.into(), false).await.unwrap();
            assert_eq!(id, tx.id(), "RPC did not accept the submitted transaction ID");
        }
        let block = submit.get_block_template(miner.clone(), vec![]).await.unwrap().block;
        if !batch.is_empty() {
            let template_ids =
                block.transactions.iter().skip(1).cloned().map(|tx| Transaction::try_from(tx).unwrap().id()).collect::<BTreeSet<_>>();
            assert_eq!(template_ids, batch.iter().map(|tx| tx.id()).collect(), "template did not contain the exact submitted batch");
        }
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
        await_tip(client, tip).await;
        if transactions_per_block > 0 {
            let expected = traffic.iter().map(|tx| (tx.id(), 0, tx.outputs[0].value)).collect::<BTreeSet<_>>();
            timeout(Duration::from_secs(10), async {
                loop {
                    let outputs = client.get_utxos_by_addresses(vec![miner.clone()]).await.unwrap();
                    let settled = outputs
                        .iter()
                        .filter(|output| !output.utxo_entry.is_coinbase)
                        .map(|output| (output.outpoint.transaction_id, output.outpoint.index, output.utxo_entry.amount))
                        .collect::<BTreeSet<_>>();
                    if settled == expected {
                        break;
                    }
                    sleep(COLLECTION_INTERVAL).await;
                }
            })
            .await
            .expect("sample peer did not settle the exact signed transaction outputs");
        }
    }
    if transactions_per_block > 0 {
        let evidence = serde_json::json!({
            "blocks": sample_blocks,
            "transactions": traffic.len(),
            "transactions_per_block": transactions_per_block,
            "peers_with_matching_outputs": 3,
            "target_period_ms": profile.target_time_per_block,
            "final_tip": tip.to_string(),
        });
        println!("phase6_signed_traffic,{evidence}");
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
