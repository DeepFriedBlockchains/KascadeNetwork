//! Opt-in single-write batches with cache invalidation only after persistence.
use crate::prelude::{BatchDbWriter, DB, StoreError, StoreResult};
use rocksdb::{WriteBatch, WriteBatchIterator, WriteOptions};
use std::{collections::BTreeSet, sync::Arc};

/// Does not publish staged values. Dropping the owner abandons all effects.
/// Callers must serialize their state transitions and must not mix eager cache
/// writers into this batch. This is not a generation/CAS or pruning authority.
#[must_use = "commit the batch or deliberately abandon it"]
pub struct DeferredDbBatch<'a> {
    db: Arc<DB>,
    batch: WriteBatch,
    invalidations: Vec<Box<dyn FnOnce() + 'a>>,
    rejected: bool,
    reserved: BTreeSet<Vec<u8>>,
}

impl<'a> DeferredDbBatch<'a> {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db, batch: WriteBatch::default(), invalidations: Vec::new(), rejected: false, reserved: BTreeSet::new() }
    }

    /// Stage bytes for an uncached row after checking exact database identity.
    /// Cached stores must use their typed deferred staging methods.
    pub fn put_raw(&mut self, db: &Arc<DB>, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> StoreResult<()> {
        self.ensure_database(db)?;
        self.batch.put(key, value);
        Ok(())
    }

    pub(crate) fn writer(&mut self) -> BatchDbWriter<'_> {
        BatchDbWriter::new(&mut self.batch)
    }

    /// Reject the entire batch if the owner or a sibling uses another database handle.
    pub fn ensure_database(&mut self, db: &Arc<DB>) -> StoreResult<()> {
        if !Arc::ptr_eq(&self.db, db) {
            self.rejected = true;
            return Err(StoreError::DataInconsistency("deferred batch belongs to another database".into()));
        }
        Ok(())
    }

    /// Check the actual staged point writes, including typed-store operations.
    /// Operations the iterator cannot inspect, such as range deletes, are denied.
    pub fn ensure_key_prefix(&self, prefix: u8) -> StoreResult<()> {
        struct Scope {
            prefix: u8,
            visited: usize,
            allowed: bool,
        }

        impl WriteBatchIterator for Scope {
            fn put(&mut self, key: &[u8], _: &[u8]) {
                self.delete(key);
            }

            fn delete(&mut self, key: &[u8]) {
                self.visited += 1;
                self.allowed &= key.first() == Some(&self.prefix);
            }
        }

        let mut scope = Scope { prefix, visited: 0, allowed: true };
        self.batch.iterate(&mut scope);
        if !scope.allowed || scope.visited != self.batch.len() {
            return Err(StoreError::DataInconsistency("deferred batch exceeds its authorized write scope".into()));
        }
        Ok(())
    }

    /// A failed required staging operation must make the entire batch unusable.
    pub fn reject(&mut self) {
        self.rejected = true;
    }

    /// Immutable store inserts must reserve their physical key because staged
    /// rows are intentionally invisible to normal DB/cache existence checks.
    pub fn reserve_unique_key(&mut self, key: &[u8]) -> StoreResult<()> {
        if !self.reserved.insert(key.to_vec()) {
            self.rejected = true;
            return Err(StoreError::DataInconsistency("duplicate deferred immutable key".into()));
        }
        Ok(())
    }

    pub(crate) fn invalidate_after_commit(&mut self, invalidate: impl FnOnce() + 'a) {
        self.invalidations.push(Box::new(invalidate));
    }

    /// Exactly one WAL-backed synchronous write; no callback runs on failure.
    /// Invalidation, rather than cache insertion, is safe when later commits
    /// finish first. Cache-fill generation checks fence in-flight old reads.
    pub fn commit(self) -> StoreResult<()> {
        if self.rejected {
            return Err(StoreError::DataInconsistency("deferred batch was rejected".into()));
        }
        let mut options = WriteOptions::default();
        options.disable_wal(false);
        options.set_sync(true);
        self.db.write_opt(self.batch, &options)?;
        for invalidate in self.invalidations {
            invalidate();
        }
        Ok(())
    }
}
