use super::test_consensus::TestConsensus;
use crate::{
    model::stores::issuance_checkpoint::{DbIssuanceCheckpoint, IssuanceCheckpointAnchor, LocalIssuanceReplay},
    processes::coinbase::{CoinbaseClaimBatch, CoinbaseClaimContext},
};
use kaspa_addresses::{Address, Prefix, Version};
use kaspa_consensus_core::{
    api::{ConsensusApi, args::TransactionValidationArgs},
    block::Block,
    blockhash,
    blockstatus::BlockStatus::StatusUTXOValid,
    coinbase::{CoinbaseTransactionTemplate, MinerData},
    config::{ConfigBuilder, params::SIMNET_PARAMS},
    constants::TX_VERSION_TOCCATA,
    issuance_checkpoint::{IssuanceCheckpoint, IssuanceClaim},
    sign::sign,
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{MutableTransaction, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput},
};
use kaspa_database::prelude::{ConnBuilder, DeferredDbBatch};
use kaspa_hashes::Hash;
use kaspa_txscript::pay_to_address_script;
use std::sync::Arc;

const CAP: u64 = 2_900_000_000_000_000_000;

fn validated_spend(
    consensus: &TestConsensus,
    outpoint: TransactionOutpoint,
    key: secp256k1::Keypair,
    script: kaspa_consensus_core::tx::ScriptPublicKey,
    fee: u64,
) -> MutableTransaction<Arc<Transaction>> {
    let (_, entry) = consensus.get_virtual_utxos(None, 32, false).into_iter().find(|(key, _)| *key == outpoint).unwrap();
    assert!(entry.is_coinbase);
    let unsigned = Transaction::new(
        TX_VERSION_TOCCATA,
        vec![TransactionInput::new_with_compute_budget(outpoint, vec![], 0, 0)],
        vec![TransactionOutput::new(entry.amount.checked_sub(fee).unwrap(), script)],
        0,
        SUBNETWORK_ID_NATIVE,
        0,
        vec![],
    );
    let signed = sign(MutableTransaction::with_entries(unsigned, vec![entry]), key).tx;
    let mut spend = MutableTransaction::from_tx(signed);
    consensus.validate_mempool_transaction(&mut spend, &TransactionValidationArgs::default()).unwrap();
    assert_eq!(spend.calculated_fee, Some(fee));
    spend
}

async fn append_selected_block(
    consensus: &TestConsensus,
    miner: &MinerData,
    path: &mut Vec<Hash>,
    transactions: Vec<Transaction>,
) -> (Block, CoinbaseClaimBatch) {
    let parent = *path.last().unwrap();
    let pending = consensus.lkg_virtual_state.load_full();
    assert_eq!(pending.ghostdag_data.selected_parent, parent);
    let mut block = consensus.build_utxo_valid_block_with_parents(blockhash::NONE, vec![parent], miner.clone(), transactions);
    let index = path.len() as u64;
    block.header.timestamp = consensus.params().genesis.timestamp + index * consensus.params().target_time_per_block();
    block.header.nonce = index;
    block.header.finalize();
    let block = block.to_immutable();
    assert_eq!(consensus.validate_and_insert_block(block.clone()).virtual_state_task.await.unwrap(), StatusUTXOValid);
    assert_eq!(consensus.get_sink(), block.hash());

    let coinbase = CoinbaseTransactionTemplate { tx: block.transactions[0].clone(), has_red_reward: false, dev_accrued: 0 };
    let context = CoinbaseClaimContext {
        settlement_block: block.hash(),
        selected_index: index,
        daa_score: block.header.daa_score,
        deferred_before: 0,
    };
    let attributed = consensus
        .services
        .coinbase_manager
        .attribute_settlement_claims(&coinbase, &pending.ghostdag_data, &pending.mergeset_rewards, &pending.mergeset_non_daa, context)
        .unwrap();
    assert!(attributed.deferred_payout.is_none());
    path.push(block.hash());
    (block, attributed)
}

#[tokio::test]
async fn signed_transparent_fee_is_recycled_without_increasing_issuance() {
    const FEE: u64 = 100_000;
    let config =
        ConfigBuilder::new(SIMNET_PARAMS.clone()).edit_consensus_params(|params| params.blockrate.coinbase_maturity = 2).build();
    let genesis = config.genesis.hash;
    let directory = kaspa_database::utils::get_kaspa_tempdir();
    let open_db = || ConnBuilder::default().with_files_limit(10).with_db_path(directory.path().to_owned()).build().unwrap();
    let db = open_db();
    let (sender, _receiver) = async_channel::unbounded();
    let consensus = TestConsensus::with_db(db.clone(), &config, sender);
    let workers = consensus.init();

    let key = secp256k1::Keypair::from_seckey_slice(secp256k1::SECP256K1, &[9; 32]).unwrap();
    let address = Address::new(Prefix::Simnet, Version::PubKey, &key.x_only_public_key().0.serialize());
    let script = pay_to_address_script(&address);
    let miner = MinerData::new(script.clone(), vec![]);
    let mut path = vec![genesis];
    let mut claims = Vec::<IssuanceClaim>::new();
    let (_, first) = append_selected_block(&consensus, &miner, &mut path, vec![]).await;
    claims.extend(first.claims);
    let (funding, second) = append_selected_block(&consensus, &miner, &mut path, vec![]).await;
    claims.extend(second.claims);
    for _ in 0..config.coinbase_maturity() {
        let (_, attributed) = append_selected_block(&consensus, &miner, &mut path, vec![]).await;
        claims.extend(attributed.claims);
    }

    let outpoint = TransactionOutpoint::new(funding.transactions[0].id(), 0);
    let mempool_spend = validated_spend(&consensus, outpoint, key, script, FEE);
    let output_value = mempool_spend.tx.outputs[0].value;
    let spend_id = mempool_spend.id();
    let (source, before_fee) =
        append_selected_block(&consensus, &miner, &mut path, vec![Arc::unwrap_or_clone(mempool_spend.tx)]).await;
    claims.extend(before_fee.claims);
    let previous = IssuanceCheckpoint::derive_from_claims(genesis, source.hash(), CAP, &path, &claims).unwrap();

    let pending = consensus.lkg_virtual_state.load_full();
    let reward = pending.mergeset_rewards.get(&source.hash()).unwrap();
    assert_eq!(reward.total_fees, FEE);
    let subsidy = consensus.services.coinbase_manager.deserialize_coinbase_payload(&source.transactions[0].payload).unwrap().subsidy;
    assert!(subsidy > 0);

    let (child, attributed) = append_selected_block(&consensus, &miner, &mut path, vec![]).await;
    let acceptance = consensus.get_block_acceptance_data(child.hash()).unwrap();
    assert!(acceptance.iter().any(|entry| {
        entry.block_hash == source.hash()
            && entry.accepted_transactions.iter().any(|tx| tx.transaction_id == spend_id && tx.index_within_block == 1)
    }));
    let replacement = TransactionOutpoint::new(spend_id, 0);
    let current_utxos = consensus.get_virtual_utxos(None, 32, false);
    assert!(!current_utxos.iter().any(|(key, _)| *key == outpoint));
    assert!(current_utxos.iter().any(|(key, entry)| *key == replacement && entry.amount == output_value && !entry.is_coinbase));
    assert_eq!(child.transactions[0].outputs.iter().map(|output| output.value).sum::<u64>(), subsidy + FEE);
    assert_eq!(attributed.settlement.recycled_fees, FEE);
    assert_eq!(attributed.claims.len(), 1);
    assert_eq!(
        (attributed.claims[0].reward_source, attributed.claims[0].minted, attributed.claims[0].paid),
        (source.hash(), subsidy, subsidy)
    );
    claims.extend(attributed.claims);
    let next = IssuanceCheckpoint::derive_from_claims(genesis, child.hash(), CAP, &path, &claims).unwrap();
    assert_eq!(
        (next.claim_count, next.minted, next.paid, next.deferred),
        (previous.claim_count + 1, previous.minted + subsidy, previous.paid + subsidy, 0)
    );

    let anchor = |checkpoint: IssuanceCheckpoint, selected_index| IssuanceCheckpointAnchor {
        network_id: genesis,
        supply_cap: CAP,
        pruning_point: checkpoint.pruning_point,
        selected_index,
        expected_commitment: checkpoint.commitment(),
    };
    let next_index = path.len() as u64 - 1;
    let previous_anchor = anchor(previous, next_index - 1);
    let next_anchor = anchor(next, next_index);
    {
        let store = DbIssuanceCheckpoint::new(db.clone());
        let selected = consensus.storage.selected_chain_store.read();
        let mut batch = DeferredDbBatch::new(db.clone());
        store.stage(&mut batch, &selected, previous_anchor, &previous).unwrap();
        batch.commit().unwrap();
        let mut batch = DeferredDbBatch::new(db.clone());
        store.stage_reconciled_transition(&mut batch, &selected, previous_anchor, next_anchor, &next, &attributed.settlement).unwrap();
        batch.commit().unwrap();
    }
    consensus.shutdown(workers);
    drop(consensus);
    drop(db);

    let db = open_db();
    let (sender, _receiver) = async_channel::unbounded();
    let config = config.to_builder().skip_adding_genesis().build();
    let reopened = TestConsensus::with_db(db.clone(), &config, sender);
    let selected = reopened.storage.selected_chain_store.read();
    assert_eq!(DbIssuanceCheckpoint::new(db).load(&selected, next_anchor).unwrap(), Some(next));
}

#[tokio::test]
async fn locally_attributed_issuance_replay_replaces_a_selected_spend_fork() {
    const FEE_A: u64 = 100_000;
    const FEE_B: u64 = 200_000;
    let config =
        ConfigBuilder::new(SIMNET_PARAMS.clone()).edit_consensus_params(|params| params.blockrate.coinbase_maturity = 2).build();
    let genesis = config.genesis.hash;
    let directory = kaspa_database::utils::get_kaspa_tempdir();
    let open_db = || ConnBuilder::default().with_files_limit(10).with_db_path(directory.path().to_owned()).build().unwrap();
    let db = open_db();
    let (sender, _receiver) = async_channel::unbounded();
    let main = TestConsensus::with_db(db.clone(), &config, sender);
    let main_workers = main.init();
    let fork = TestConsensus::new(&config);
    let fork_workers = fork.init();

    let key = secp256k1::Keypair::from_seckey_slice(secp256k1::SECP256K1, &[9; 32]).unwrap();
    let address = Address::new(Prefix::Simnet, Version::PubKey, &key.x_only_public_key().0.serialize());
    let script = pay_to_address_script(&address);
    let miner = MinerData::new(script.clone(), vec![]);
    let mut path = vec![genesis];
    let mut claims = Vec::new();
    let mut funding = None;
    for index in 0..2 + config.coinbase_maturity() {
        let (block, attributed) = append_selected_block(&main, &miner, &mut path, vec![]).await;
        assert_eq!(fork.validate_and_insert_block(block.clone()).virtual_state_task.await.unwrap(), StatusUTXOValid);
        assert_eq!(fork.get_sink(), block.hash());
        if index == 1 {
            funding = Some(TransactionOutpoint::new(block.transactions[0].id(), 0));
        }
        claims.extend(attributed.claims);
    }
    let funding = funding.unwrap();
    let mut path_b = path.clone();
    let mut claims_b = claims.clone();
    let spend_a = validated_spend(&main, funding, key, script.clone(), FEE_A);
    let spend_b = validated_spend(&fork, funding, key, script, FEE_B);
    let out_a = TransactionOutpoint::new(spend_a.id(), 0);
    let out_b = TransactionOutpoint::new(spend_b.id(), 0);
    let value_b = spend_b.tx.outputs[0].value;
    assert_ne!(out_a, out_b);
    let (_, settlement_a) = append_selected_block(&main, &miner, &mut path, vec![Arc::unwrap_or_clone(spend_a.tx)]).await;
    claims.extend(settlement_a.claims);
    let (tip_a, settled_a) = append_selected_block(&main, &miner, &mut path, vec![]).await;
    assert_eq!(settled_a.settlement.recycled_fees, FEE_A);
    claims.extend(settled_a.claims);
    let checkpoint_a = IssuanceCheckpoint::derive_from_claims(genesis, tip_a.hash(), CAP, &path, &claims).unwrap();
    let stage = |path: &[Hash], claims: &[IssuanceClaim], checkpoint: &IssuanceCheckpoint| {
        let selected = main.storage.selected_chain_store.read();
        let replay = LocalIssuanceReplay { network_genesis: genesis, supply_cap: CAP, selected_path: path, claims, releases: &[] };
        let store = DbIssuanceCheckpoint::new(db.clone());
        let mut batch = DeferredDbBatch::new(db.clone());
        store.stage_retained_replay_checkpoint(&mut batch, &selected, replay, checkpoint).unwrap();
        batch.commit().unwrap();
    };
    stage(&path, &claims, &checkpoint_a);
    assert!(main.get_virtual_utxos(None, 32, false).iter().any(|(outpoint, _)| *outpoint == out_a));

    // Attribute both branch B settlements while they are selected in an independent local replay.
    let (source_b, settlement_b) = append_selected_block(&fork, &miner, &mut path_b, vec![Arc::unwrap_or_clone(spend_b.tx)]).await;
    claims_b.extend(settlement_b.claims);
    let (child_b, settled_b) = append_selected_block(&fork, &miner, &mut path_b, vec![]).await;
    assert_eq!(settled_b.settlement.recycled_fees, FEE_B);
    claims_b.extend(settled_b.claims);
    let equal_height_b = IssuanceCheckpoint::derive_from_claims(genesis, child_b.hash(), CAP, &path_b, &claims_b).unwrap();
    assert_eq!(
        (equal_height_b.claim_count, equal_height_b.minted, equal_height_b.paid, equal_height_b.deferred),
        (checkpoint_a.claim_count, checkpoint_a.minted, checkpoint_a.paid, checkpoint_a.deferred)
    );
    assert_ne!(equal_height_b.ordered_claims_root, checkpoint_a.ordered_claims_root);
    let (tip_b, final_b) = append_selected_block(&fork, &miner, &mut path_b, vec![]).await;
    claims_b.extend(final_b.claims);
    let checkpoint_b = IssuanceCheckpoint::derive_from_claims(genesis, tip_b.hash(), CAP, &path_b, &claims_b).unwrap();
    for block in [source_b, child_b, tip_b.clone()] {
        main.validate_and_insert_block(block).virtual_state_task.await.unwrap();
    }
    assert_eq!(main.get_sink(), tip_b.hash());
    {
        let selected = main.storage.selected_chain_store.read();
        let store = DbIssuanceCheckpoint::new(db.clone());
        assert!(store.load_retained_replay_checkpoint(&selected, genesis, tip_a.hash(), CAP).is_err());
    }
    stage(&path_b, &claims_b, &checkpoint_b);
    main.shutdown(main_workers);
    fork.shutdown(fork_workers);
    drop(main);
    drop(db);

    let db = open_db();
    let (sender, _receiver) = async_channel::unbounded();
    let reopened = TestConsensus::with_db(db.clone(), &config.to_builder().skip_adding_genesis().build(), sender);
    let selected = reopened.storage.selected_chain_store.read();
    let store = DbIssuanceCheckpoint::new(db);
    assert_eq!(store.load_retained_replay_checkpoint(&selected, genesis, tip_b.hash(), CAP).unwrap(), checkpoint_b);
    assert!(store.load_retained_replay_checkpoint(&selected, genesis, tip_a.hash(), CAP).is_err());
    let utxos = reopened.get_virtual_utxos(None, 32, false);
    assert!(!utxos.iter().any(|(outpoint, _)| *outpoint == out_a));
    assert!(utxos.iter().any(|(outpoint, entry)| *outpoint == out_b && entry.amount == value_b && !entry.is_coinbase));
}
