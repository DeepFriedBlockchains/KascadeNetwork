//! Evidence from the existing pruning proof validator for inactive import completion.

use super::branch_readiness::ReadinessPoint;
#[cfg(test)]
use crate::consensus::Consensus;
use crate::model::stores::{
    headers::{DbHeadersStore, HeaderStoreReader},
    pruning::{DbPruningStore, PruningProofDescriptor, PruningStoreReader},
};
use kaspa_consensus_core::{BlockHashSet, api::ValidatedStagingPruningProof, hashing, pruning::PruningPointProof};
#[cfg(test)]
use kaspa_consensus_core::{api::ConsensusApi, pruning::PruningProofMetadata};
use kaspa_database::prelude::{CachePolicy, DB, StoreError, StoreResult};
use std::sync::Arc;

/// Created by validating against the defending consensus before acquiring the
/// receiver's pruning guards. An imported descriptor alone cannot mint it.
#[derive(Clone)]
pub(super) struct ValidatedImportProof {
    descriptor: PruningProofDescriptor,
    headers: Arc<BlockHashSet>,
}

impl ValidatedImportProof {
    #[cfg(test)]
    pub(super) fn authenticate(defender: &Consensus, proof: &PruningPointProof, metadata: &PruningProofMetadata) -> StoreResult<Self> {
        let evidence = Self::capture_validated(proof)?;
        defender.validate_pruning_proof(proof, metadata).map_err(|err| StoreError::DataInconsistency(err.to_string()))?;
        Ok(evidence)
    }

    pub(super) fn from_receipt(receipt: &ValidatedStagingPruningProof) -> StoreResult<Self> {
        Self::capture_validated(receipt.proof())
    }

    pub(super) fn pruning_point(&self) -> kaspa_hashes::Hash {
        self.descriptor.pruning_point
    }

    pub(super) fn capture_validated(proof: &PruningPointProof) -> StoreResult<Self> {
        let point = proof.first().and_then(|level| level.last()).ok_or_else(|| inconsistent("empty pruning proof"))?.hash;
        if proof.iter().any(Vec::is_empty) {
            return Err(inconsistent("pruning proof has an empty level"));
        }
        let mut headers = BlockHashSet::default();
        for header in proof.iter().flatten() {
            if hashing::header::hash(header) != header.hash {
                return Err(inconsistent("pruning proof header does not match its hash"));
            }
            headers.insert(header.hash);
        }
        Ok(Self { descriptor: PruningProofDescriptor::from_proof(proof, point, true), headers: Arc::new(headers) })
    }

    /// Corroborate the authenticated proof against fresh receiver DB reads.
    /// The receiver must retain the pruning guards through READY publication.
    pub(super) fn verify_committed(&self, db: &Arc<DB>, point: ReadinessPoint) -> StoreResult<()> {
        let actual = DbPruningStore::new(db.clone()).pruning_proof_descriptor()?;
        if point.hash != self.descriptor.pruning_point
            || actual.pruning_point != self.descriptor.pruning_point
            || actual.external != self.descriptor.external
            || actual.tips != self.descriptor.tips
            || actual.roots != self.descriptor.roots
            || actual.counts != self.descriptor.counts
        {
            return Err(inconsistent("committed pruning proof descriptor differs from the authenticated proof"));
        }

        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        for hash in self.headers.iter() {
            let header = headers.get_header(*hash)?;
            if header.hash != *hash || hashing::header::hash(&header) != *hash {
                return Err(inconsistent("committed proof header does not match its authenticated hash"));
            }
        }
        Ok(())
    }
}

fn inconsistent(reason: &str) -> StoreError {
    StoreError::DataInconsistency(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        consensus::test_consensus::TestConsensus,
        model::stores::{headers::HeaderStore, pruning::PruningStore},
    };
    use kaspa_consensus_core::{
        block::Block,
        config::{ConfigBuilder, genesis::DEVNET_GENESIS, params::DEVNET_PARAMS},
        header::Header,
        issuance_checkpoint::{DeveloperPayoutRelease, IssuanceClaim},
    };
    use kaspa_database::{create_temp_db, prelude::ConnBuilder};
    use kaspa_hashes::Hash;

    #[tokio::test]
    async fn live_pruning_candidate_authenticates_selected_child_after_reorg() {
        use super::super::{
            branch_readiness::DbBranchReadiness,
            import_shielded_readiness::authenticated_selected_child_coinbase,
        };
        use crate::model::services::reachability::ReachabilityService;
        use kaspa_consensus_core::{
            api::ConsensusApi,
            blockhash,
            coinbase::MinerData,
            config::params::SIMNET_PARAMS,
            tx::{ScriptPublicKey, ScriptVec},
        };

        async fn append(tc: &TestConsensus, parent: Hash, tag: u8) -> Block {
            let miner = MinerData::new(ScriptPublicKey::new(0, ScriptVec::from_slice(&[0x51, tag])), vec![]);
            let mut block = tc.build_utxo_valid_block_with_parents(blockhash::NONE, vec![parent], miner, vec![]);
            let params = tc.params();
            block.header.timestamp = params.genesis.timestamp + u64::from(tag) * params.target_time_per_block();
            block.header.nonce = u64::from(tag);
            block.header.finalize();
            let block = block.to_immutable();
            tc.validate_and_insert_block(block.clone()).virtual_state_task.await.unwrap();
            block
        }

        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let config = ConfigBuilder::new(SIMNET_PARAMS)
            .edit_consensus_params(|params| {
                params.blockrate.finality_depth = 3;
                params.blockrate.pruning_depth = 12;
                params.skip_proof_of_work = true;
            })
            .set_archival()
            .build();
        let (sender, _receiver) = async_channel::unbounded();
        let tc = TestConsensus::with_db(db.clone(), &config, sender);
        let workers = tc.init();
        let consensus = tc.consensus_clone();
        let processor = tc.virtual_processor();
        let first = append(&tc, config.genesis.hash, 1).await;
        let parent = append(&tc, first.hash(), 2).await;
        let candidate = append(&tc, parent.hash(), 3).await;
        let point = candidate.hash();
        assert!(processor.pruning_point_manager.is_pruning_sample(candidate.header.blue_score, parent.header.blue_score, 3));

        let old_child = append(&tc, point, 4).await;
        let old_tip = append(&tc, old_child.hash(), 5).await;
        assert_eq!(consensus.get_sink(), old_tip.hash());
        assert!(!processor.pruning_point_manager.is_valid_pruning_point(point, old_tip.hash()));
        assert_eq!(
            authenticated_selected_child_coinbase(&db, point, &old_child, old_child.hash()).unwrap().id(),
            old_child.transactions[0].id()
        );

        let child = append(&tc, point, 6).await;
        let mut tip = child.hash();
        for tag in 7..=19 {
            tip = append(&tc, tip, tag).await.hash();
        }
        assert_eq!(consensus.get_sink(), tip);
        assert!(processor.pruning_point_manager.is_valid_pruning_point(point, tip));
        assert!(processor.reachability_service.is_chain_ancestor_of(child.hash(), tip));
        assert!(!processor.reachability_service.is_chain_ancestor_of(old_child.hash(), tip));
        assert_eq!(
            authenticated_selected_child_coinbase(&db, point, &child, child.hash()).unwrap().id(),
            child.transactions[0].id()
        );
        assert!(authenticated_selected_child_coinbase(&db, parent.hash(), &child, child.hash()).is_err());
        let missing_body = Block::from_header_arc(child.header.clone());
        assert!(authenticated_selected_child_coinbase(&db, point, &missing_body, child.hash()).is_err());
        let wrong_body = Block::from_arcs(child.header.clone(), old_child.transactions.clone());
        assert!(authenticated_selected_child_coinbase(&db, point, &wrong_body, child.hash()).is_err());
        // Ordinary consensus advancement cannot stand in for completing an
        // isolated import with independently verified UTXO and SMT evidence.
        let readiness = DbBranchReadiness::new(db.clone());
        assert!(!readiness.is_ready(ReadinessPoint { hash: point, index: 1 }).unwrap());
        assert!(processor.authority_lease().is_err());
        tc.shutdown(workers);
    }

    #[test]
    fn authenticated_completion_rejects_missing_committed_import_metadata() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &ConfigBuilder::new(DEVNET_PARAMS).build(), sender);
        let processor = consensus.virtual_processor();
        let mut header = Header::from(&DEVNET_GENESIS);
        header.finalize();
        let point = ReadinessPoint { hash: header.hash, index: 3 };
        processor.pruning_point_store.write().set(point.hash, point.index).unwrap();

        let proof = vec![vec![Arc::new(header.clone())]];
        let proof = ValidatedImportProof::capture_validated(&proof).unwrap();
        let child = Block::new(header, vec![]);
        let token = processor.begin_authority_import().unwrap();
        assert!(processor.finish_authenticated_authority_import(token, point, &proof, &child).is_err());
        assert!(processor.authority_lease().is_err());
    }

    #[test]
    fn matching_descriptor_cannot_substitute_for_committed_proof_headers() {
        let (_temp, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let headers = DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty);
        let mut pruning = DbPruningStore::new(db.clone());
        let mut root = Header::from(&DEVNET_GENESIS);
        root.finalize();
        let mut middle = root.clone();
        middle.nonce += 1;
        middle.finalize();
        let mut tip = middle.clone();
        tip.nonce += 1;
        tip.finalize();

        let proof = vec![vec![Arc::new(root), Arc::new(middle), Arc::new(tip)]];
        let validated = ValidatedImportProof::capture_validated(&proof).unwrap();
        let point = ReadinessPoint { hash: proof[0][2].hash, index: 3 };

        headers.insert(proof[0][0].hash, proof[0][0].clone(), 0).unwrap();
        headers.insert(point.hash, proof[0][2].clone(), 0).unwrap();
        pruning.set_pruning_proof_descriptor(validated.descriptor.clone()).unwrap();
        assert!(validated.verify_committed(&db, point).is_err());

        headers.insert(proof[0][1].hash, proof[0][1].clone(), 0).unwrap();
        validated.verify_committed(&db, point).unwrap();
        let mut substituted = validated.descriptor.clone();
        substituted.counts[0] += 1;
        pruning.set_pruning_proof_descriptor(substituted).unwrap();
        assert!(validated.verify_committed(&db, point).is_err());
    }

    fn install_shielded_child(db: &Arc<DB>, parent: Hash, nonce: u64) -> Block {
        use crate::{
            model::stores::{
                ghostdag::{DbGhostdagStore, GhostdagData, GhostdagStore},
                reachability::{DbReachabilityStore, ReachabilityStore},
            },
            processes::{reachability::interval::Interval, shielded::PruningPointShieldedMetadata},
        };
        use kaspa_consensus_core::{
            merkle::calc_hash_merkle_root, subnets::SUBNETWORK_ID_COINBASE, tx::Transaction,
            zkas_state_binding::SHIELDED_COMMITMENT_OFFSET,
        };

        let mut payload = vec![0; SHIELDED_COMMITMENT_OFFSET];
        payload.extend_from_slice(&PruningPointShieldedMetadata::empty_state_root());
        payload.extend_from_slice(&[0; 3]);
        let coinbase = Transaction::new(0, vec![], vec![], 0, SUBNETWORK_ID_COINBASE, 0, payload);
        let mut header = Header::from(&DEVNET_GENESIS);
        header.nonce = nonce;
        header.hash_merkle_root = calc_hash_merkle_root(std::iter::once(&coinbase));
        header.finalize();
        DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty)
            .insert(header.hash, Arc::new(header.clone()), 0)
            .unwrap();
        DbGhostdagStore::new(db.clone(), 0, CachePolicy::Empty, CachePolicy::Empty)
            .insert(header.hash, Arc::new(GhostdagData::new_with_selected_parent(parent, 5)))
            .unwrap();
        DbReachabilityStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty)
            .insert(header.hash, parent, Interval::new(1, 10), nonce)
            .unwrap();
        Block::new(header, vec![coinbase])
    }

    // Keep fees out of subsidy accounting. Each builder invocation settles
    // two eligible sources and recycles the third source's fees.
    fn nonzero_settlements(
        manager: &crate::processes::coinbase::CoinbaseManager,
        intermediate: Hash,
        point: Hash,
    ) -> (Vec<IssuanceClaim>, Vec<DeveloperPayoutRelease>) {
        use crate::{model::stores::ghostdag::GhostdagData, processes::coinbase::CoinbaseClaimContext};
        use kaspa_consensus_core::{
            BlockHashMap,
            coinbase::{BlockRewardData, MinerData},
            tx::ScriptPublicKey,
        };
        let script = ScriptPublicKey::from_vec(0, vec![0x51]);
        let settle = |block, index, daa, deferred_before, first_source| {
            let [blue, red, fee_only] = [first_source, first_source + 1, first_source + 2].map(Hash::from_u64_word);
            let ghostdag = GhostdagData {
                mergeset_blues: Arc::new(vec![blue]),
                mergeset_reds: Arc::new(vec![red, fee_only]),
                ..Default::default()
            };
            let mut rewards = BlockHashMap::default();
            for (source, subsidy, fees) in [(blue, 1_000, 9), (red, 2_000, 11), (fee_only, 5_000, 13)] {
                rewards.insert(source, BlockRewardData::new(subsidy, fees, script.clone()));
            }
            let mut non_daa = BlockHashSet::default();
            non_daa.insert(fee_only);
            let template = manager
                .expected_coinbase_transaction(
                    daa,
                    MinerData::new(script.clone(), vec![]),
                    &ghostdag,
                    &rewards,
                    &non_daa,
                    [0; 32],
                    daa - 1,
                    deferred_before,
                )
                .unwrap();
            let context = CoinbaseClaimContext { settlement_block: block, selected_index: index, daa_score: daa, deferred_before };
            let result = manager.attribute_settlement_claims(&template, &ghostdag, &rewards, &non_daa, context).unwrap();
            assert_eq!(result.settlement.recycled_fees, 33);
            assert_eq!(result.claims.iter().map(|claim| claim.reward_source).collect::<Vec<_>>(), [blue, red]);
            result
        };

        let first = settle(intermediate, 1, 101, 0, 21);
        assert_eq!((first.settlement.paid_subsidy, first.settlement.deferred_after), (2_850, 150));
        assert!(first.deferred_payout.is_none());
        let second = settle(point, 2, 110, first.settlement.deferred_after, 31);
        assert_eq!((second.settlement.paid_subsidy, second.settlement.deferred_after), (3_150, 0));
        let release = second.deferred_payout.unwrap();
        assert_eq!(release.amount, 300);
        (first.claims.into_iter().chain(second.claims).collect(), vec![release])
    }

    #[test]
    fn authority_integration_reopen_before_promotion_rejects_incomplete_evidence() {
        exercise_guarded_issuance_publication(false);
    }

    #[test]
    fn authority_integration_publishes_nonzero_settlement_and_readiness() {
        exercise_guarded_issuance_publication(true);
    }

    fn exercise_guarded_issuance_publication(nonzero: bool) {
        use super::super::{
            branch_readiness::DbBranchReadiness,
            import_readiness::tests::{install_point, stage_explicit_evidence},
            staging_import::PROMOTION_KEY,
        };
        use crate::model::stores::issuance_checkpoint::{DbIssuanceCheckpoint, LocalIssuanceReplay};
        use crate::model::stores::{
            ghostdag::GhostdagData,
            headers_selected_tip::{DbHeadersSelectedTipStore, HeadersSelectedTipStore},
            selected_chain::{DbSelectedChainStore, SelectedChainStore},
            statuses::{DbStatusesStore, StatusesStore},
            virtual_state::{VirtualState, VirtualStateStore},
        };
        use crate::processes::ghostdag::ordering::SortableBlock;
        use kaspa_consensus_core::{
            ChainPath,
            blockstatus::BlockStatus::StatusUTXOValid,
            config::params::{ForkActivation, ZKAS_DEV_FEE_RECIPIENT},
            issuance_checkpoint::IssuanceCheckpoint,
        };
        use rocksdb::WriteBatch;

        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(10).with_db_path(temp.path().to_owned()).build().unwrap();
        let db = connect();
        let config = ConfigBuilder::new(DEVNET_PARAMS)
            .edit_consensus_params(|params| {
                params.toccata_activation = ForkActivation::never();
                params.dev_fee_accrual_activation = ForkActivation::new(100);
                params.dev_fee_payout_interval = 10;
                params.dev_fee_permille = 50;
                params.dev_fee_recipient = Some(ZKAS_DEV_FEE_RECIPIENT);
            })
            .skip_adding_genesis()
            .build();
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender.clone());
        let processor = consensus.virtual_processor();
        let mut header = Header::from(&DEVNET_GENESIS);
        header.finalize();
        let genesis = header.hash;
        let intermediate = Hash::from_u64_word(700);
        if nonzero {
            header.nonce += 1;
            header.finalize();
        }
        let point = ReadinessPoint { hash: header.hash, index: 3 };
        let proof = ValidatedImportProof::capture_validated(&vec![vec![Arc::new(header.clone())]]).unwrap();
        DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty).insert(point.hash, Arc::new(header), 0).unwrap();
        install_point(&db, point, ReadinessPoint { hash: 7.into(), index: 1 });
        DbPruningStore::new(db.clone()).set_pruning_proof_descriptor(proof.descriptor.clone()).unwrap();
        let child = install_shielded_child(&db, point.hash, 201);
        DbHeadersSelectedTipStore::new(db.clone()).set(SortableBlock::new(child.hash(), child.header.blue_work)).unwrap();
        DbStatusesStore::new(db.clone(), CachePolicy::Empty).set(point.hash, StatusUTXOValid).unwrap();
        let mut batch = WriteBatch::default();
        let mut pruning = DbPruningStore::new(db.clone());
        pruning.set_retention_checkpoint(&mut batch, point.hash).unwrap();
        pruning.set_retention_period_root(&mut batch, point.hash).unwrap();
        let mut selected = DbSelectedChainStore::new(db.clone(), CachePolicy::Empty);
        selected.init_with_pruning_point(&mut batch, genesis).unwrap();
        db.write(batch).unwrap();
        let mut batch = WriteBatch::default();
        if nonzero {
            selected.apply_changes(&mut batch, &ChainPath { removed: vec![], added: vec![intermediate, point.hash] }).unwrap();
        }
        stage_explicit_evidence(&mut processor.pruning_meta_stores.write(), &mut batch, point);
        db.write(batch).unwrap();
        drop(pruning);
        drop(selected);
        processor
            .virtual_stores
            .write()
            .state
            .set(Arc::new(VirtualState {
                parents: vec![point.hash],
                ghostdag_data: GhostdagData::new_with_selected_parent(point.hash, 5),
                ..Default::default()
            }))
            .unwrap();

        let token = processor.begin_authority_import().unwrap();
        let unspent = processor.finish_verified_ibd_import(token, &proof, &child).unwrap();
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        drop(unspent);
        let mut batch = WriteBatch::default();
        processor.pruning_meta_stores.write().set_pruning_smt_stable_flag(&mut batch, false).unwrap();
        db.write(batch).unwrap();
        drop(consensus);
        drop(db);

        let db = connect();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        assert!(processor.authority_lease().is_err());
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        assert!(processor.recover_authenticated_staging(point, &proof, &child, &child).is_err());
        assert!(!DbBranchReadiness::new(db.clone()).is_ready(point).unwrap());

        let mut batch = WriteBatch::default();
        processor.pruning_meta_stores.write().set_pruning_smt_stable_flag(&mut batch, true).unwrap();
        db.write(batch).unwrap();
        assert!(processor.authority_lease().is_err());
        let recovered = processor.recover_authenticated_staging(point, &proof, &child, &child).unwrap();
        drop(recovered);
        const CAP: u64 = 2_900_000_000_000_000_000;
        assert_eq!(genesis, DEVNET_GENESIS.hash);
        let (path, claims, releases) = if nonzero {
            let path = vec![genesis, intermediate, point.hash];
            let (claims, releases) = nonzero_settlements(&processor.coinbase_manager, intermediate, point.hash);
            (path, claims, releases)
        } else {
            (vec![genesis], vec![], vec![])
        };
        let replay = || LocalIssuanceReplay {
            network_genesis: genesis,
            supply_cap: CAP,
            selected_path: &path,
            claims: &claims,
            releases: &releases,
        };
        let checkpoint = IssuanceCheckpoint::derive_from_settlements(genesis, point.hash, CAP, &path, &claims, &releases).unwrap();
        if nonzero {
            assert_eq!((checkpoint.claim_count, checkpoint.minted, checkpoint.paid, checkpoint.deferred), (4, 6_000, 6_000, 0));
        }
        {
            let readiness = DbBranchReadiness::new(db.clone());
            let store = DbIssuanceCheckpoint::new(db.clone());
            let unavailable = || {
                let selected = processor.selected_chain_store.read();
                assert!(store.load_retained_replay_checkpoint(&selected, genesis, point.hash, CAP).is_err());
                assert!(!readiness.is_ready(point).unwrap());
                assert!(db.get(PROMOTION_KEY).unwrap().is_none());
            };
            let token = processor.begin_authority_import().unwrap();
            drop(token);
            unavailable();

            let token = processor.begin_authority_import().unwrap();
            assert!(processor.finish_verified_ibd_import_with_issuance(token, &proof, &child, Some((None, &checkpoint))).is_err());
            unavailable();

            let token = processor.begin_authority_import().unwrap();
            let substituted = IssuanceCheckpoint { ordered_claims_root: child.hash(), ..checkpoint };
            assert!(
                processor
                    .finish_verified_ibd_import_with_issuance(token, &proof, &child, Some((Some(replay()), &substituted)))
                    .is_err()
            );
            unavailable();

            if nonzero {
                let mut repeated_claims = claims.clone();
                repeated_claims.push(claims[0]);
                let mut repeated_releases = releases.clone();
                repeated_releases.push(releases[0]);
                let excessive_release = [DeveloperPayoutRelease { amount: releases[0].amount + 1, ..releases[0] }];
                let stale_path = [genesis, child.hash(), point.hash];
                for invalid in [
                    LocalIssuanceReplay { claims: &repeated_claims, ..replay() },
                    LocalIssuanceReplay { releases: &repeated_releases, ..replay() },
                    LocalIssuanceReplay { releases: &excessive_release, ..replay() },
                    LocalIssuanceReplay { selected_path: &stale_path, ..replay() },
                    LocalIssuanceReplay { network_genesis: child.hash(), ..replay() },
                ] {
                    let token = processor.begin_authority_import().unwrap();
                    assert!(
                        processor
                            .finish_verified_ibd_import_with_issuance(token, &proof, &child, Some((Some(invalid), &checkpoint)))
                            .is_err()
                    );
                    unavailable();
                }
            }

            let token = processor.begin_authority_import().unwrap();
            let recovered = processor
                .finish_verified_ibd_import_with_issuance(token, &proof, &child, Some((Some(replay()), &checkpoint)))
                .unwrap();
            assert!(readiness.is_ready(point).unwrap());
            {
                let selected = processor.selected_chain_store.read();
                assert_eq!(store.load_retained_replay_checkpoint(&selected, genesis, point.hash, CAP).unwrap(), checkpoint);
            }
            processor.commit_staging_promotion_checked(recovered, &proof, &child, true).unwrap();
            assert_eq!(db.get(PROMOTION_KEY).unwrap().unwrap().as_slice(), b"promotion-ready");
        }
        drop(consensus);
        drop(db);

        let db = connect();
        assert!(DbBranchReadiness::new(db.clone()).is_ready(point).unwrap());
        let selected = parking_lot::RwLock::new(DbSelectedChainStore::new(db.clone(), CachePolicy::Empty));
        let guard = selected.read();
        assert_eq!(
            DbIssuanceCheckpoint::new(db.clone()).load_retained_replay_checkpoint(&guard, genesis, point.hash, CAP).unwrap(),
            checkpoint
        );
        assert_eq!(db.get(PROMOTION_KEY).unwrap().unwrap().as_slice(), b"promotion-ready");
        drop(guard);
        if nonzero {
            let mut batch = WriteBatch::default();
            selected
                .write()
                .apply_changes(&mut batch, &ChainPath { removed: vec![point.hash], added: vec![Hash::from_u64_word(701)] })
                .unwrap();
            db.write(batch).unwrap();
            assert!(
                DbIssuanceCheckpoint::new(db).load_retained_replay_checkpoint(&selected.read(), genesis, point.hash, CAP).is_err()
            );
        }
    }

    #[test]
    fn authority_integration_promotes_and_recovers_a_committed_descendant_chain() {
        use super::super::{
            branch_readiness::DbBranchReadiness,
            import_readiness::tests::install_point,
            staging_import::{PROMOTION_KEY, StagingComponent, StagingImport},
        };
        use crate::{
            model::stores::{
                block_transactions::{BlockTransactionsStore, DbBlockTransactionsStore},
                ghostdag::GhostdagData,
                headers_selected_tip::{DbHeadersSelectedTipStore, HeadersSelectedTipStore},
                selected_chain::{DbSelectedChainStore, SelectedChainStore},
                shielded::{DbNullifierDiffStore, DbNullifierSetStore, NullifierSetStore},
                statuses::{DbStatusesStore, StatusesStore},
                virtual_state::{VirtualState, VirtualStateStore},
            },
            processes::ghostdag::ordering::SortableBlock,
        };
        use kaspa_consensus_core::{
            api::{StagingPromotion, StagingPromotionError},
            blockstatus::BlockStatus::StatusUTXOValid,
            config::params::ForkActivation,
            muhash::MuHashExtensions,
            tx::{ScriptPublicKey, TransactionOutpoint, UtxoEntry},
        };
        use kaspa_database::{prelude::DeferredDbBatch, registry::DatabaseStorePrefixes as Prefix};
        use kaspa_muhash::MuHash;
        use rocksdb::WriteBatch;

        let temp = kaspa_database::utils::get_kaspa_tempdir();
        let connect = || ConnBuilder::default().with_files_limit(10).with_db_path(temp.path().to_owned()).build().unwrap();
        let db = connect();
        let config = ConfigBuilder::new(DEVNET_PARAMS)
            .edit_consensus_params(|params| params.toccata_activation = ForkActivation::never())
            .skip_adding_genesis()
            .build();
        let (sender, _receiver) = async_channel::unbounded();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender.clone());
        let processor = consensus.virtual_processor();
        let mut point_header = Header::from(&DEVNET_GENESIS);
        point_header.nonce = 100;
        let outpoint = TransactionOutpoint::new(12.into(), 0);
        let entry = UtxoEntry::new(10, ScriptPublicKey::from_vec(0, vec![0x51]), 1, false, None);
        point_header.utxo_commitment = MuHash::from_utxo(&outpoint, &entry).finalize();
        point_header.finalize();
        let point = ReadinessPoint { hash: point_header.hash, index: 3 };
        let retention = ReadinessPoint { hash: 7.into(), index: 1 };
        let proof = ValidatedImportProof::capture_validated(&vec![vec![Arc::new(point_header.clone())]]).unwrap();
        DbHeadersStore::new(db.clone(), CachePolicy::Empty, CachePolicy::Empty).insert(point.hash, Arc::new(point_header), 0).unwrap();
        install_point(&db, point, retention);
        let child = install_shielded_child(&db, point.hash, 101);
        let mut header_tip = DbHeadersSelectedTipStore::new(db.clone());
        header_tip.set(SortableBlock::new(child.hash(), child.header.blue_work)).unwrap();

        let mut staging = StagingImport::begin_fixture(processor.clone(), proof.clone()).unwrap();
        assert!(staging.finish(retention, &child).is_err());
        assert!(StagingImport::begin_fixture(processor.clone(), proof.clone()).is_err());
        let (_foreign_temp, foreign_db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let foreign = TestConsensus::with_db(foreign_db, &config, sender.clone());
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let components = [
            (
                StagingComponent::Proof,
                vec![
                    (Prefix::PruningProofDescriptor, bincode::serialize(&proof.descriptor).unwrap()),
                    (Prefix::BodyMissingAnticone, bincode::serialize(&Vec::<Hash>::new()).unwrap()),
                ],
            ),
            (StagingComponent::Smt, vec![(Prefix::SmtSyncFlag, bincode::serialize(&true).unwrap())]),
            (StagingComponent::Shielded, vec![(Prefix::ShieldedSyncFlag, bincode::serialize(&true).unwrap())]),
        ];
        for (component, rows) in components {
            let worker_db = db.clone();
            runtime.block_on(async {
                staging
                    .submit(component, true, move || {
                        let mut batch = DeferredDbBatch::new(worker_db.clone());
                        for (prefix, data) in rows {
                            batch.put_raw(&worker_db, prefix, data)?;
                        }
                        Ok(batch)
                    })
                    .unwrap()
                    .await
                    .unwrap()
                    .unwrap();
            });
            assert!(processor.authority_lease().is_err());
        }
        let chunk = vec![(outpoint, entry.clone())];
        assert!(foreign.stage_phase6_pruning_utxo_chunk(&mut staging, chunk.clone()).is_err());
        assert!(foreign.complete_phase6_pruning_utxos(&mut staging, point.hash).is_err());
        assert!(staging.submit(StagingComponent::Utxo, true, || panic!("generic UTXO batch started")).is_err());
        runtime.block_on(async { consensus.stage_phase6_pruning_utxo_chunk(&mut staging, chunk).unwrap().await }).unwrap().unwrap();
        assert!(staging.finish(retention, &child).is_err());
        runtime.block_on(async { consensus.complete_phase6_pruning_utxos(&mut staging, point.hash).unwrap().await }).unwrap().unwrap();
        let verified = staging.finish(retention, &child).unwrap();
        let sibling = processor.authority_lease().unwrap();

        assert!(matches!(
            consensus.authorize_staging_promotion(StagingPromotion::new(())),
            Err(StagingPromotionError::InvalidReceipt)
        ));
        assert!(db.get(PROMOTION_KEY).unwrap().is_none());
        consensus.authorize_staging_promotion(verified.into_promotion()).unwrap();
        assert!(processor.commit_staging_promotion(sibling, &proof, &child).is_err());
        assert_eq!(db.get(PROMOTION_KEY).unwrap().unwrap().as_slice(), b"promotion-ready");
        drop(staging);

        let token = processor.begin_authority_import().unwrap();
        let stale = processor.finish_authenticated_authority_import(token, retention, &proof, &child).unwrap();
        let mut metadata = processor.pruning_meta_stores.write();
        metadata.utxo_set.clear().unwrap();
        drop(metadata);
        assert!(processor.commit_staging_promotion(stale, &proof, &child).is_err());
        assert!(DbBranchReadiness::new(db.clone()).closed_revision().unwrap().is_some());
        let mut metadata = processor.pruning_meta_stores.write();
        metadata.utxo_set.write_from_iterator_without_cache([(outpoint, Arc::new(entry.clone()))]).unwrap();
        drop(metadata);
        assert!(processor.authority_lease().is_err());
        let token = processor.begin_authority_import().unwrap();
        let reverified = processor.finish_authenticated_authority_import(token, retention, &proof, &child).unwrap();
        processor.commit_staging_promotion(reverified, &proof, &child).unwrap();

        // The IBD receipt also requires the finalized virtual UTXO set.
        let write_retention = |hash| {
            let mut pruning = DbPruningStore::new(db.clone());
            let mut batch = WriteBatch::default();
            pruning.set_retention_checkpoint(&mut batch, hash).unwrap();
            pruning.set_retention_period_root(&mut batch, hash).unwrap();
            db.write(batch).unwrap();
        };
        write_retention(point.hash);
        DbStatusesStore::new(db.clone(), CachePolicy::Empty).set(point.hash, StatusUTXOValid).unwrap();
        let token = processor.begin_authority_import().unwrap();
        assert!(processor.finish_verified_ibd_import(token, &proof, &child).is_err());
        assert!(processor.authority_lease().is_err());

        let state = VirtualState {
            parents: vec![point.hash],
            ghostdag_data: GhostdagData::new_with_selected_parent(point.hash, 5),
            multiset: MuHash::from_utxo(&outpoint, &entry),
            ..Default::default()
        };
        let mut virtual_write = processor.virtual_stores.write();
        virtual_write.state.set(Arc::new(state)).unwrap();
        virtual_write.utxo_set.write_from_iterator_without_cache([(outpoint, Arc::new(entry.clone()))]).unwrap();
        drop(virtual_write);
        let token = processor.begin_authority_import().unwrap();
        let receipt = processor.finish_verified_ibd_import(token, &proof, &child).unwrap();
        processor.virtual_stores.write().utxo_set.clear().unwrap();
        assert!(processor.commit_staging_promotion_checked(receipt, &proof, &child, true).is_err());
        assert!(processor.authority_lease().is_err());

        processor.virtual_stores.write().utxo_set.write_from_iterator_without_cache([(outpoint, Arc::new(entry))]).unwrap();
        let token = processor.begin_authority_import().unwrap();
        let receipt = processor.finish_verified_ibd_import(token, &proof, &child).unwrap();
        processor.commit_staging_promotion_checked(receipt, &proof, &child, true).unwrap();

        write_retention(retention.hash);
        let tip_child = install_shielded_child(&db, child.hash(), 102);
        header_tip.set(SortableBlock::new(tip_child.hash(), tip_child.header.blue_work)).unwrap();
        let mut chain = DbSelectedChainStore::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        chain.init_with_pruning_point(&mut batch, point.hash).unwrap();
        chain.write_entry(&mut batch, 1, child.hash()).unwrap();
        chain.set_highest_index(&mut batch, 1).unwrap();
        db.write(batch).unwrap();
        let state = VirtualState { ghostdag_data: GhostdagData::new_with_selected_parent(child.hash(), 5), ..Default::default() };
        processor.virtual_stores.write().state.set(Arc::new(state)).unwrap();
        drop(chain);
        drop(header_tip);
        drop(consensus);
        drop(db);

        let db = connect();
        let consensus = TestConsensus::with_db(db.clone(), &config, sender);
        let processor = consensus.virtual_processor();
        let readiness = DbBranchReadiness::new(db.clone());
        assert!(readiness.is_ready(point).unwrap());
        assert!(processor.authority_lease().is_err());
        let diffs = DbNullifierDiffStore::new(db.clone(), CachePolicy::Empty);
        let mut batch = WriteBatch::default();
        diffs.set_batch(&mut batch, child.hash(), vec![[9; 32]]).unwrap();
        db.write(batch).unwrap();
        assert!(processor.recover_authenticated_staging(retention, &proof, &child, &tip_child).is_err());
        assert!(!readiness.is_ready(point).unwrap());

        let mark_unready = || {
            let metadata = processor.pruning_meta_stores.write();
            let mut batch = DeferredDbBatch::new(db.clone());
            metadata.stage_pruning_shielded_stable(&mut batch, false).unwrap();
            batch.commit().unwrap();
        };
        mark_unready();
        let before = db.get([u8::from(Prefix::ShieldedSyncFlag)]).unwrap();
        assert!(processor.reconcile_descendant_shielded_readiness(point.hash, child.hash(), &child, &tip_child).is_err());
        assert_eq!(db.get([u8::from(Prefix::ShieldedSyncFlag)]).unwrap(), before);
        assert_eq!(db.get(PROMOTION_KEY).unwrap().unwrap().as_slice(), b"promotion-ready");
        assert!(!readiness.is_ready(point).unwrap());

        let mut batch = WriteBatch::default();
        diffs.delete_batch(&mut batch, child.hash()).unwrap();
        db.write(batch).unwrap();
        assert!(processor.reconcile_descendant_shielded_readiness(point.hash, point.hash, &child, &tip_child).is_err());
        assert!(processor.reconcile_descendant_shielded_readiness(point.hash, child.hash(), &child, &child).is_err());
        assert_eq!(db.get([u8::from(Prefix::ShieldedSyncFlag)]).unwrap(), before);
        processor.reconcile_descendant_shielded_readiness(point.hash, child.hash(), &child, &tip_child).unwrap();
        assert!(processor.pruning_meta_stores.read().pruning_shielded_stable_flag());
        assert!(!readiness.is_ready(point).unwrap());
        assert!(processor.recover_authenticated_staging(retention, &proof, &child, &tip_child).is_ok());
        assert!(processor.authority_lease().is_ok());

        assert!(processor.capture_shielded_export_snapshot(point.hash).is_err());
        DbBlockTransactionsStore::new(db.clone(), CachePolicy::Empty).insert(child.hash(), child.transactions.clone()).unwrap();
        assert!(processor.capture_shielded_export_snapshot(retention.hash).is_err());
        let captured = processor.capture_shielded_export_snapshot(point.hash).unwrap();
        assert!(captured.metadata.is_none());
        assert!(captured.nullifiers.is_empty());
        mark_unready();
        assert!(processor.capture_shielded_export_snapshot(point.hash).is_err());
        processor.reconcile_descendant_shielded_readiness(point.hash, child.hash(), &child, &tip_child).unwrap();

        let mut batch = WriteBatch::default();
        DbNullifierSetStore::new(db.clone(), CachePolicy::Empty).insert_batch(&mut batch, [9; 32]).unwrap();
        db.write(batch).unwrap();
        assert!(processor.capture_shielded_export_snapshot(point.hash).is_err());
        assert!(captured.nullifiers.is_empty());
        assert_eq!(db.get(PROMOTION_KEY).unwrap().unwrap().as_slice(), b"promotion-ready");
    }
}
