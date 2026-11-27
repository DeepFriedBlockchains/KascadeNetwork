#![cfg(feature = "phase6-staging-preview")]

use kaspa_addresses::{Address, Version};
use kaspa_consensus::processes::coinbase::CoinbaseManager;
use kaspa_consensus_core::{
    config::params::{SIMNET_GENESIS, SIMNET_PARAMS},
    constants::TX_VERSION_TOCCATA,
    header::Header,
    sign::sign,
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{MutableTransaction, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry},
};
use kaspa_grpc_client::GrpcClient;
use kaspa_hashes::Hash;
use kaspa_pow::State;
use kaspa_rpc_core::{RpcRawBlock, RpcUtxoEntry, SubmitBlockRejectReason, SubmitBlockReport, api::rpc::RpcApi};
use kaspa_testing_integration::common::{
    daemon::{ClientManager, Daemon},
    fee, utils,
};
use kaspa_txscript::pay_to_address_script;
use kaspa_utils::networking::ContextualNetAddress;
use kaspa_wrpc_server::address::WrpcNetAddress;
use kaspad_lib::args::Args;
use std::{sync::Arc, time::Duration};

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
    let block = client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
    if reject_invalid_nonce {
        let mut invalid = block.clone();
        let state = State::new(&Header::try_from(&invalid.header).unwrap());
        invalid.header.nonce =
            (0..16_384).find(|nonce| !state.check_pow(*nonce).0).expect("fixture should have a nonce above its target");
        let response = client.submit_block(invalid, false).await.unwrap();
        assert!(matches!(response.report, SubmitBlockReport::Reject(SubmitBlockRejectReason::BlockInvalid)));
    }

    submit_native_template(client, block).await
}

async fn submit_native_template(client: &GrpcClient, mut block: RpcRawBlock) -> Hash {
    let candidate = Header::try_from(&block.header).unwrap();
    let state = State::new(&candidate);
    assert!(candidate.aux_pow.is_none(), "this fixture must exercise native proof of work");

    block.header.nonce = (0..16_384).find(|nonce| state.check_pow(*nonce).0).expect("fixture should mine an easy native nonce");
    let expected = Header::try_from(&block.header).unwrap().hash;
    let response = client.submit_block(block, false).await.unwrap();
    assert!(response.report.is_success(), "valid native proof of work rejected: {}", response.reject_detail);
    wait_for_sink(client, expected).await;
    expected
}

fn loopback_daemon(mut args: Args) -> Daemon {
    Daemon::fill_args_with_random_ports(&mut args);
    let loopback = |address: ContextualNetAddress| ContextualNetAddress::loopback().with_port(address.normalize(0).port);
    args.listen = args.listen.map(loopback);
    args.rpclisten = args.rpclisten.map(loopback);
    let Some(WrpcNetAddress::Custom(borsh)) = args.rpclisten_borsh else {
        panic!("fixture must allocate a Borsh RPC port");
    };
    args.rpclisten_borsh = Some(WrpcNetAddress::Custom(loopback(borsh)));
    args.rpclisten_json = None;
    Daemon::with_manager(Arc::new(ClientManager::new(args)), 10)
}

fn signed_spend(
    input: TransactionOutpoint,
    entry: UtxoEntry,
    destination: &Address,
    key: secp256k1::Keypair,
    fee: u64,
) -> Transaction {
    assert!(entry.is_coinbase);
    let amount = entry.amount.checked_sub(fee).unwrap();
    let tx = Transaction::new(
        TX_VERSION_TOCCATA,
        vec![TransactionInput::new_with_compute_budget(input, vec![], 0, 0)],
        vec![TransactionOutput::new(amount, pay_to_address_script(destination))],
        0,
        SUBNETWORK_ID_NATIVE,
        0,
        vec![],
    );
    let mut signed = sign(MutableTransaction::with_entries(tx, vec![entry]), key).tx;
    signed.finalize();
    signed
}

async fn settled_output(
    client: &GrpcClient,
    miner: &Address,
    destination: &Address,
    input: TransactionOutpoint,
    output: TransactionOutpoint,
) -> RpcUtxoEntry {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let utxos = client.get_utxos_by_addresses(vec![miner.clone(), destination.clone()]).await.unwrap();
            if let Some(settled) = utxos.iter().find(|item| item.outpoint == output.into()) {
                assert_eq!(settled.address.as_ref(), Some(destination));
                assert!(!utxos.iter().any(|item| item.outpoint == input.into()));
                return settled.utxo_entry.clone();
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("peer did not settle the signed spend")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn phase6_native_signed_fee_settles_and_reopens_across_peers() {
    kaspa_alloc::init_allocator_with_default_settings();
    const MATURITY: u64 = 2;
    const WARMUP_INTERVAL: Duration = Duration::from_secs(1);
    let temp = tempfile::tempdir().unwrap();
    let params = temp.path().join("native-signed-fee.json");
    let mut blockrate = SIMNET_PARAMS.blockrate.clone();
    blockrate.coinbase_maturity = MATURITY;
    std::fs::write(
        &params,
        serde_json::json!({"skip_proof_of_work": false, "shielded_coinbase": false, "blockrate": blockrate}).to_string(),
    )
    .unwrap();
    let args = Args {
        simnet: true,
        enable_unsynced_mining: true,
        unsafe_rpc: true,
        disable_upnp: true,
        disable_dns_seeding: true,
        outbound_target: 0,
        override_params_file: Some(params.to_string_lossy().into_owned()),
        utxoindex: true,
        ..Default::default()
    };
    let mut first = loopback_daemon(args.clone());
    let mut second = loopback_daemon(args.clone());
    let mut third = loopback_daemon(args);
    let first_client = first.start().await;
    let second_client = second.start().await;
    let third_client = third.start().await;
    let clients = [&first_client, &second_client, &third_client];
    let seed: ContextualNetAddress = format!("127.0.0.1:{}", first.p2p_port).parse().unwrap();
    second_client.add_peer(seed, true).await.unwrap();
    third_client.add_peer(seed, true).await.unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        while first_client.get_connected_peer_info().await.unwrap().peer_info.len() != 2 {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("signed-fee peers did not connect");
    for client in clients {
        wait_for_sink(client, SIMNET_GENESIS.hash).await;
    }

    let key = secp256k1::Keypair::from_seckey_slice(secp256k1::SECP256K1, &[9; 32]).unwrap();
    let receiver = secp256k1::Keypair::from_seckey_slice(secp256k1::SECP256K1, &[12; 32]).unwrap();
    let miner = Address::new(first.network.into(), Version::PubKey, &key.x_only_public_key().0.serialize());
    let destination = Address::new(first.network.into(), Version::PubKey, &receiver.x_only_public_key().0.serialize());
    for _ in 0..MATURITY + 3 {
        let tip = submit_native_block(&first_client, &miner, false).await;
        for client in clients {
            wait_for_sink(client, tip).await;
        }
        // Keep the fixture's native warmup on the same one-second cadence as the
        // sustained relay rehearsal. After the last tip, let the preceding block's
        // mempool bookkeeping finish and its 500 ms announcement interval elapse
        // before submitting the only transaction in this otherwise idle fixture.
        tokio::time::sleep(WARMUP_INTERVAL).await;
    }
    for client in clients {
        assert!(client.get_server_info().await.unwrap().is_synced, "transaction relay requires a recent sink");
    }

    let (input, entry) = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(coinbase) = utils::fetch_spendable_utxos(&second_client, miner.clone(), MATURITY).await.into_iter().next() {
                return coinbase;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("fixture did not produce a mature native coinbase");
    let fee_amount = fee::calc_for_plain_standard_tx(1, 1);
    let output_amount = entry.amount.checked_sub(fee_amount).unwrap();
    let signed = signed_spend(input, entry.clone(), &destination, key, fee_amount);
    let conflict = signed_spend(input, entry, &destination, key, fee_amount + 1);
    assert_ne!(signed.id(), conflict.id());
    assert_eq!(second_client.submit_transaction((&signed).into(), false).await.unwrap(), signed.id());
    for (node, client) in ["miner", "submitter", "follower"].into_iter().zip(clients) {
        let mempool = tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                if let Ok(tx) = client.get_mempool_entry(signed.id(), false, false).await {
                    return tx;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("{node} did not receive signed spend {}", signed.id()));
        assert!(!mempool.is_orphan);
        assert_eq!(mempool.fee, fee_amount);
        assert_eq!(Transaction::try_from(mempool.transaction).unwrap().id(), signed.id());
    }

    let template = first_client.get_block_template(miner.clone(), vec![]).await.unwrap().block;
    assert_eq!(template.transactions.len(), 2);
    let included = Transaction::try_from(template.transactions[1].clone()).unwrap();
    assert_eq!(included.id(), signed.id());
    assert_eq!(included.inputs[0].signature_script, signed.inputs[0].signature_script);
    let source = submit_native_template(&first_client, template).await;
    for client in clients {
        wait_for_sink(client, source).await;
    }

    // The child pays the selected parent's subsidy and validated fees.
    let params = &SIMNET_PARAMS;
    let coinbase = CoinbaseManager::new(
        params.coinbase_payload_script_public_key_max_len,
        params.max_coinbase_payload_len,
        params.deflationary_phase_daa_score,
        params.pre_deflationary_phase_base_subsidy,
        params.bps_history(),
        params.toccata_activation,
        params.dev_fee_permille,
        params.dev_fee_recipient,
        params.dev_fee_accrual_activation,
        params.dev_fee_payout_interval,
    );
    let source = first_client.get_block(source, true).await.unwrap();
    let subsidy = coinbase.deserialize_coinbase_payload(&source.transactions[0].payload).unwrap().subsidy;
    assert!(subsidy > 0);
    let tip = submit_native_block(&first_client, &miner, false).await;
    let output = TransactionOutpoint::new(signed.id(), 0);
    let expected = settled_output(&first_client, &miner, &destination, input, output).await;
    assert_eq!(expected.amount, output_amount);
    assert!(!expected.is_coinbase);
    for client in clients {
        wait_for_sink(client, tip).await;
        let child = client.get_block(tip, true).await.unwrap();
        assert_eq!(child.transactions[0].outputs.iter().map(|output| output.value).sum::<u64>(), subsidy + fee_amount);
        assert_eq!(settled_output(client, &miner, &destination, input, output).await, expected);
    }
    assert!(third_client.submit_transaction((&conflict).into(), false).await.is_err());
    assert!(third_client.get_mempool_entry(conflict.id(), true, false).await.is_err());

    third_client.disconnect().await.unwrap();
    drop(third_client);
    let mut third = third.restart(10);
    let third_client = third.start().await;
    assert!(third_client.get_connected_peer_info().await.unwrap().peer_info.is_empty());
    wait_for_sink(&third_client, tip).await;
    assert_eq!(settled_output(&third_client, &miner, &destination, input, output).await, expected);
    first_client.disconnect().await.unwrap();
    second_client.disconnect().await.unwrap();
    third_client.disconnect().await.unwrap();
    third.shutdown();
    second.shutdown();
    first.shutdown();
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
