//! Bounded accounting for an untrusted nullifier stream. No payload is retained.
use kaspa_consensus_core::api::{MAX_SHIELDED_NULLIFIER_IMPORT_COUNT, SHIELDED_NULLIFIER_IMPORT_BATCH_LIMIT};
use kaspa_muhash::MuHash;

pub(crate) struct ShieldedImportAccumulator {
    declared: u64,
    count: u64,
    accumulator: MuHash,
}

impl ShieldedImportAccumulator {
    pub(crate) fn new(declared: u64) -> Result<Self, String> {
        if declared > MAX_SHIELDED_NULLIFIER_IMPORT_COUNT {
            return Err(format!("declared nullifier count {declared} exceeds the sanity cap {MAX_SHIELDED_NULLIFIER_IMPORT_COUNT}"));
        }
        Ok(Self { declared, count: 0, accumulator: MuHash::new() })
    }

    pub(crate) fn append(&mut self, batch: &[[u8; 32]]) -> Result<(), String> {
        if batch.is_empty() || batch.len() > SHIELDED_NULLIFIER_IMPORT_BATCH_LIMIT {
            return Err("shielded import batch is empty or exceeds the batch limit".into());
        }
        let count = self.count + batch.len() as u64; // Both operands are bounded by the limits above.
        if count > self.declared {
            return Err(format!("streamed nullifiers exceed the declared count {}", self.declared));
        }
        for nullifier in batch {
            self.accumulator.add_element(nullifier);
        }
        self.count = count;
        Ok(())
    }

    pub(crate) fn finish(mut self, expected: &MuHash) -> Result<u64, String> {
        if self.count != self.declared {
            return Err(format!("nullifier count mismatch: metadata says {}, streamed {}", self.declared, self.count));
        }
        if self.accumulator.finalize() != expected.clone().finalize() {
            return Err(format!("streamed nullifier set ({}) does not reproduce the committed accumulator", self.count));
        }
        Ok(self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_shielded_accounting_rejects_oversize_incomplete_and_corrupt_streams() {
        let mut import = ShieldedImportAccumulator::new(1).unwrap();
        assert!(import.append(&vec![[1; 32]; SHIELDED_NULLIFIER_IMPORT_BATCH_LIMIT + 1]).unwrap_err().contains("batch limit"));
        assert!(import.append(&[]).is_err());
        assert!(ShieldedImportAccumulator::new(1).unwrap().finish(&MuHash::new()).unwrap_err().contains("count mismatch"));
        import.append(&[[1; 32]]).unwrap();
        assert!(import.finish(&MuHash::new()).unwrap_err().contains("committed accumulator"));
    }
}
