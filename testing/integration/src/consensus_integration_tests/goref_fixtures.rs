//! Current-format replacements for archived Go-reference DAGs.
//! Generation uses the real template builder and validates every generated block.
use super::*;
use flate2::{Compression, GzBuilder};
use kaspa_consensus::params::Params;
use kaspa_consensus_core::hashing::sighash_type::SIG_HASH_ALL;
use kaspa_rpc_core::RpcBlock;
use std::io::{BufWriter, Write};

fn base_params() -> Params {
    let mut params = DEVNET_PARAMS;
    // Explicit transparent test network, as used by other UTXO consensus tests.
    // Retain current payload/transaction validation; no historical codec exists.
    params.shielded_coinbase = false;
    params.skip_proof_of_work = true;
    params.dev_fee_permille = 0;
    params.dev_fee_recipient = None;
    params.low_difficulty_start_blocks = 0;
    params.difficulty_ramp_blocks = 0;
    params
}

const RECIPES: [(&str, usize, usize); 3] =
    [("goref_custom_pruning_depth", 5000, 4790), ("goref-notx-5000-blocks", 5000, 0), ("goref-1060-tx-265-blocks", 265, 224)];

#[tokio::test]
#[ignore = "explicit fixture generation; normal tests replay checked-in output"]
async fn generate_goref_fixtures() {
    init_allocator_with_default_settings();
    let output_root = std::env::var_os("GOREF_FIXTURE_OUTPUT").expect("set GOREF_FIXTURE_OUTPUT to a staging directory");
    for (name, block_count, transaction_count) in RECIPES {
        let archive = Path::new("testdata/dags_for_json_tests/upstream-kaspa-originals").join(name).join("blocks.json.gz");
        let header = gzip_file_lines(&archive).next().unwrap();
        let mut archived_params: serde_json::Value = serde_json::from_str(&header).unwrap();
        let blockrate = archived_params["blockrate"].as_object_mut().expect("archived blockrate");
        // The archived upstream files predate shielded anchor fields.
        blockrate.entry("shielded_anchor_depth").or_insert(20.into());
        blockrate.entry("max_shielded_anchor_age").or_insert(50.into());
        let mut overrides: OverrideParams = serde_json::from_value(archived_params).unwrap();
        let blockrate = overrides.blockrate.as_ref().unwrap();
        assert!(blockrate.shielded_anchor_depth < blockrate.max_shielded_anchor_age);
        assert!(blockrate.max_shielded_anchor_age < blockrate.pruning_depth - blockrate.finality_depth);
        overrides.shielded_coinbase = Some(false);
        overrides.skip_proof_of_work = Some(true);
        let params = base_params().override_params(overrides);
        assert!(!params.shielded_coinbase);
        let config = Config::new(params.clone());
        let tc = TestConsensus::new(&config);
        let handles = tc.init();
        let output = Path::new(&output_root).join(name);
        std::fs::create_dir_all(&output).unwrap();
        let file = BufWriter::new(File::create(output.join("blocks.json.gz")).unwrap());
        let mut writer = GzBuilder::new().mtime(0).write(file, Compression::best());
        serde_json::to_writer(&mut writer, &OverrideParams::from(params.clone())).unwrap();
        writeln!(writer).unwrap();
        let genesis =
            Block::from_arcs(tc.get_header(params.genesis.hash).unwrap(), tc.get_block(params.genesis.hash).unwrap().transactions);
        serde_json::to_writer(&mut writer, &RpcBlock::from(&genesis)).unwrap();
        writeln!(writer).unwrap();
        // Public test-only key for repeatable fixture signatures.
        let secp = secp256k1::Secp256k1::new();
        let signing_key = secp256k1::Keypair::from_seckey_slice(&secp, &[42; 32]).unwrap();
        let (public_key, _) = signing_key.x_only_public_key();
        let script = ScriptPublicKey::from_vec(0, std::iter::once(0x20).chain(public_key.serialize()).chain([0xac]).collect());
        let mut hashes = vec![params.genesis.hash];
        let mut remaining = transaction_count;
        for i in 1..=block_count {
            // Two sibling blocks followed by a merger in every fixture. The
            // small fixture still has enough post-maturity rows for 224 spends.
            let sibling = i % 40 == 39;
            let merge = i.is_multiple_of(40);
            let parents = if sibling {
                vec![hashes[i - 2]]
            } else if merge {
                vec![hashes[i - 1], hashes[i - 2]]
            } else {
                vec![hashes[i - 1]]
            };
            let mut txs = Vec::new();
            if remaining > 0 && i > 205 && !(i % 40 == 38 || sibling) {
                let daa = tc.get_virtual_daa_score();
                let mut utxos = tc.get_virtual_utxos(None, usize::MAX, false);
                utxos.retain(|(_, entry)| {
                    entry.script_public_key == script
                        && entry.amount > 1000
                        && (!entry.is_coinbase || entry.block_daa_score + params.coinbase_maturity() <= daa)
                });
                utxos.sort_by_key(|(outpoint, _)| (outpoint.transaction_id, outpoint.index));
                let count = remaining.min(if block_count < 1000 { 4 } else { 2 });
                assert!(utxos.len() >= count, "{name}: insufficient mature outputs at block {i}");
                for (outpoint, entry) in utxos.into_iter().take(count) {
                    let tx = Transaction::new(
                        0,
                        vec![TransactionInput::new(outpoint, vec![], 0, 1)],
                        vec![TransactionOutput::new(entry.amount - 1000, script.clone())],
                        0,
                        SUBNETWORK_ID_NATIVE,
                        0,
                        vec![],
                    );
                    let mut signable = MutableTransaction::with_entries(tx, vec![entry]);
                    let reused_values = SigHashReusedValuesUnsync::new();
                    let sighash = calc_schnorr_signature_hash(&signable.as_verifiable(), 0, SIG_HASH_ALL, &reused_values);
                    let message = secp256k1::Message::from_digest(sighash.as_bytes());
                    let signature = secp.sign_schnorr_no_aux_rand(&message, &signing_key);
                    signable.tx.inputs[0].signature_script =
                        std::iter::once(65).chain(signature.as_ref().iter().copied()).chain([SIG_HASH_ALL.to_u8()]).collect();
                    txs.push(signable.tx);
                }
                remaining -= count;
            }
            let miner = MinerData::new(script.clone(), (i as u64).to_le_bytes().to_vec());
            let mut block = tc.build_utxo_valid_block_with_parents(blockhash::NONE, parents, miner, txs);
            block.header.timestamp = params.genesis.timestamp + i as u64 * params.target_time_per_block();
            block.header.finalize();
            let block = block.to_immutable();
            assert_eq!(hashing::header::hash(&block.header), block.hash());
            let status = tc.validate_and_insert_block(block.clone()).virtual_state_task.await.unwrap();
            assert!(status.is_utxo_valid_or_pending());
            hashes.push(block.hash());
            serde_json::to_writer(&mut writer, &RpcBlock::from(&block)).unwrap();
            writeln!(writer).unwrap();
            if i.is_multiple_of(250) {
                println!("{name}: {i}/{block_count} blocks, {} transactions", transaction_count - remaining);
            }
        }
        assert_eq!(remaining, 0, "all requested transactions must be generated");
        assert!(tc.body_tips().iter().any(|h| tc.block_status(*h) == BlockStatus::StatusUTXOValid));
        if name.contains("pruning") {
            assert_ne!(tc.pruning_point(), params.genesis.hash);
        }
        writer.finish().unwrap().flush().unwrap();
        tc.shutdown(handles);
    }
}
