use indexmap::IndexMap;
use itertools::Itertools;
use kaspa_consensus_core::tx::TransactionId;
use kaspa_core::debug;
use kaspa_p2p_lib::{
    Hub, make_message,
    pb::{InvTransactionsMessage, KaspadMessage, kaspad_message::Payload},
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::RwLock, time::Instant as BroadcastInstant};

/// Interval between mempool scanning tasks (in seconds)
const SCANNING_TASK_INTERVAL: u64 = 10;
const REBROADCAST_FREQUENCY: u64 = 3;
const BROADCAST_INTERVAL: Duration = Duration::from_millis(500);
pub(crate) const MAX_INV_PER_TX_INV_MSG: usize = 131_072;

pub struct TransactionsSpread {
    hub: Hub,
    last_scanning_time: Instant,
    scanning_task_running: bool,
    scanning_job_count: u64,
    transaction_ids: IndexMap<TransactionId, bool>,
    last_broadcast_time: Option<BroadcastInstant>,
    flush_scheduled: bool,
}

impl TransactionsSpread {
    pub fn new(hub: Hub) -> Self {
        Self {
            hub,
            last_scanning_time: Instant::now(),
            scanning_task_running: false,
            scanning_job_count: 0,
            transaction_ids: IndexMap::new(),
            last_broadcast_time: None,
            flush_scheduled: false,
        }
    }

    /// Returns true if the time has come for running the task of scanning mempool transactions
    /// and if so, mark the task as running.
    pub fn should_run_mempool_scanning_task(&mut self) -> bool {
        let now = Instant::now();
        if self.scanning_task_running || now < self.last_scanning_time + Duration::from_secs(SCANNING_TASK_INTERVAL) {
            return false;
        }
        let delta = now.checked_duration_since(self.last_scanning_time).expect("verified above");
        // Keep the launching times aligned to exact intervals. Note that `delta=10.1` seconds will result in
        // adding 10 seconds to last scan time, while `delta=11` will result in adding 20 (assuming scanning
        // interval is 10 seconds).
        self.last_scanning_time += Duration::from_secs(delta.as_secs().div_ceil(SCANNING_TASK_INTERVAL) * SCANNING_TASK_INTERVAL);

        self.scanning_job_count += 1;
        self.scanning_task_running = true;
        true
    }

    /// Returns true if the time for a rebroadcast of the mempool high priority transactions has come.
    pub fn should_rebroadcast(&self) -> bool {
        self.scanning_job_count.is_multiple_of(REBROADCAST_FREQUENCY)
    }

    pub fn mempool_scanning_job_count(&self) -> u64 {
        self.scanning_job_count
    }

    pub fn mempool_scanning_is_done(&mut self) {
        assert!(self.scanning_task_running, "no stop without a matching start");
        self.scanning_task_running = false;
    }

    /// Add the given transactions IDs to a set of IDs to broadcast. The IDs will be broadcasted to all peers
    /// within transaction Inv messages.
    ///
    /// Coalesce transactions until the broadcast interval elapses or the queue
    /// fills one inventory message. A single timer also flushes idle queues.
    pub async fn broadcast_transactions<I: IntoIterator<Item = TransactionId>>(
        spread: &Arc<RwLock<Self>>,
        transaction_ids: I,
        should_throttle: bool,
    ) {
        let mut incoming = transaction_ids.into_iter().peekable();
        if incoming.peek().is_none() {
            return;
        }
        let mut pending = spread.write().await;
        for id in incoming {
            // Retain full fanout for an ID if any caller requested it.
            *pending.transaction_ids.entry(id).or_insert(should_throttle) &= should_throttle;
        }
        let Some(deadline) = pending.flush_if_due().await else { return };
        if pending.flush_scheduled {
            return;
        }
        pending.flush_scheduled = true;
        let spread = Arc::downgrade(spread);
        tokio::spawn(async move {
            let mut deadline = deadline;
            loop {
                tokio::time::sleep_until(deadline).await;
                let Some(spread) = spread.upgrade() else { return };
                let mut pending = spread.write().await;
                // A full queue can flush before the timer fires, shifting the
                // deadline for the next batch. Keep using this one timer.
                if let Some(next) = pending.flush_if_due().await {
                    deadline = next;
                } else {
                    pending.flush_scheduled = false;
                    return;
                }
            }
        });
    }

    async fn flush_if_due(&mut self) -> Option<BroadcastInstant> {
        if self.transaction_ids.is_empty() {
            return None;
        }
        if let Some(last) = self.last_broadcast_time
            && self.transaction_ids.len() < MAX_INV_PER_TX_INV_MSG
            && BroadcastInstant::now() < last + BROADCAST_INTERVAL
        {
            return Some(last + BROADCAST_INTERVAL);
        }

        let (throttled, unrestricted): (Vec<_>, Vec<_>) = self.transaction_ids.drain(..).partition(|(_, throttle)| *throttle);
        for (batch, should_throttle) in [(unrestricted, false), (throttled, true)] {
            for chunk in batch.chunks(MAX_INV_PER_TX_INV_MSG) {
                let ids = chunk.iter().map(|(id, _)| (*id).into()).collect_vec();
                debug!("Transaction propagation: broadcasting {} transactions", ids.len());
                let msg = make_message!(Payload::InvTransactions, InvTransactionsMessage { ids });
                self.broadcast(msg, should_throttle).await;
            }
        }

        self.last_broadcast_time = Some(BroadcastInstant::now());
        None
    }

    async fn broadcast(&self, msg: KaspadMessage, should_throttle: bool) {
        if should_throttle {
            // TODO: Figure out a better number
            self.hub.broadcast_to_some_peers(msg, 8).await
        } else {
            self.hub.broadcast(msg, None).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn idle_transaction_relay_flushes_without_more_inventory() {
        let spread = Arc::new(RwLock::new(TransactionsSpread::new(Hub::new())));
        TransactionsSpread::broadcast_transactions(&spread, [TransactionId::from(1u64)], false).await;
        let start = spread.read().await.last_broadcast_time.expect("the first inventory is sent immediately");

        TransactionsSpread::broadcast_transactions(&spread, [TransactionId::from(2u64)], false).await;
        TransactionsSpread::broadcast_transactions(&spread, [TransactionId::from(3u64), TransactionId::from(2u64)], true).await;
        {
            let pending = spread.read().await;
            assert_eq!(pending.transaction_ids.len(), 2);
            assert!(pending.flush_scheduled);
            assert!(!pending.transaction_ids[&TransactionId::from(2u64)]);
            assert!(pending.transaction_ids[&TransactionId::from(3u64)]);
        }

        tokio::time::sleep(BROADCAST_INTERVAL - Duration::from_millis(1)).await;
        assert_eq!(spread.read().await.last_broadcast_time, Some(start));
        tokio::time::sleep(Duration::from_millis(2)).await;
        let pending = spread.read().await;
        assert!(pending.transaction_ids.is_empty());
        assert!(!pending.flush_scheduled);
        assert!(pending.last_broadcast_time.unwrap() >= start + BROADCAST_INTERVAL);
        let last = pending.last_broadcast_time;
        drop(pending);

        tokio::time::sleep(BROADCAST_INTERVAL).await;
        TransactionsSpread::broadcast_transactions(&spread, [], false).await;
        let pending = spread.read().await;
        assert!(!pending.flush_scheduled);
        assert_eq!(pending.last_broadcast_time, last);
    }
}
