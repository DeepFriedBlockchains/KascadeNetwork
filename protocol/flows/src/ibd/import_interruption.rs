//! Deterministic peer interruption controls for the preview integration tests.
use kaspa_consensus_core::api::StagingImportComponent;
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};
use tokio::sync::oneshot;

struct SenderPause {
    ready: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}

struct Interruption {
    component: StagingImportComponent,
    sender: Mutex<Option<SenderPause>>,
    receiver: Mutex<Option<oneshot::Sender<()>>>,
}

fn active() -> &'static Mutex<Option<Arc<Interruption>>> {
    static ACTIVE: OnceLock<Mutex<Option<Arc<Interruption>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

/// Dropping the control releases the peer and disarms the next connection.
pub struct TransferPause {
    case: Arc<Interruption>,
    sender_paused: Option<oneshot::Receiver<()>>,
    receiver_waiting: Option<oneshot::Receiver<()>>,
    release: Option<oneshot::Sender<()>>,
}

pub fn pause_transfer(component: StagingImportComponent) -> TransferPause {
    let (sender_ready, sender_paused) = oneshot::channel();
    let (receiver_ready, receiver_waiting) = oneshot::channel();
    let (release, wait_for_release) = oneshot::channel();
    let case = Arc::new(Interruption {
        component,
        sender: Mutex::new(Some(SenderPause { ready: sender_ready, release: wait_for_release })),
        receiver: Mutex::new(Some(receiver_ready)),
    });
    let mut current = active().lock();
    assert!(current.is_none(), "an interruption case is already armed");
    *current = Some(case.clone());
    drop(current);
    TransferPause { case, sender_paused: Some(sender_paused), receiver_waiting: Some(receiver_waiting), release: Some(release) }
}

impl TransferPause {
    pub async fn wait_until_paused(&mut self) {
        self.sender_paused.take().expect("sender pause was already awaited").await.expect("peer sender exited before pause");
        self.receiver_waiting.take().expect("receiver was already awaited").await.expect("receiver exited before import");
    }

    pub fn release(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }
}

impl Drop for TransferPause {
    fn drop(&mut self) {
        self.release();
        let mut current = active().lock();
        if current.as_ref().is_some_and(|case| Arc::ptr_eq(case, &self.case)) {
            current.take();
        }
    }
}

pub(crate) async fn pause_sender(component: StagingImportComponent) {
    let case = active().lock().clone();
    let Some(case) = case.filter(|case| case.component == component) else { return };
    let pending = case.sender.lock().take();
    if let Some(SenderPause { ready, release }) = pending {
        let _ = ready.send(());
        let _ = release.await;
    }
}

pub(crate) fn receiver_waiting(component: StagingImportComponent) {
    let case = active().lock().clone();
    let Some(case) = case.filter(|case| case.component == component) else { return };
    let ready = case.receiver.lock().take();
    if let Some(ready) = ready {
        let _ = ready.send(());
    }
}
