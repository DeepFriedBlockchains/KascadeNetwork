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
    after_payload: bool,
    sender: Mutex<Option<SenderPause>>,
    finish: Mutex<Option<oneshot::Receiver<()>>>,
    receiver: Mutex<Option<oneshot::Sender<()>>>,
}

fn active() -> &'static Mutex<Option<Arc<Interruption>>> {
    static ACTIVE: OnceLock<Mutex<Option<Arc<Interruption>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

/// Dropping the control releases the peer and disarms the next connection.
pub struct TransferPause {
    case: Arc<Interruption>,
    write_pause: Option<kaspa_consensus_core::test_staging_write::WritePause>,
    sender_paused: Option<oneshot::Receiver<()>>,
    receiver_waiting: Option<oneshot::Receiver<()>>,
    release: Option<oneshot::Sender<()>>,
    finish: Option<oneshot::Sender<()>>,
}

pub fn pause_transfer(component: StagingImportComponent) -> TransferPause {
    arm(component, false)
}

/// Pause only after a nonempty payload has been durably written by the receiver.
pub fn pause_after_payload(component: StagingImportComponent) -> TransferPause {
    arm(component, true)
}

fn arm(component: StagingImportComponent, after_payload: bool) -> TransferPause {
    let (sender_ready, sender_paused) = oneshot::channel();
    let (receiver_ready, receiver_waiting) = oneshot::channel();
    let (release, wait_for_release) = oneshot::channel();
    let (finish, wait_for_finish) = oneshot::channel();
    let case = Arc::new(Interruption {
        component,
        after_payload,
        sender: Mutex::new(Some(SenderPause { ready: sender_ready, release: wait_for_release })),
        finish: Mutex::new(Some(wait_for_finish)),
        receiver: Mutex::new(Some(receiver_ready)),
    });
    let mut current = active().lock();
    assert!(current.is_none(), "an interruption case is already armed");
    *current = Some(case.clone());
    drop(current);
    let write_pause = if after_payload {
        match component {
            StagingImportComponent::Smt => Some(kaspa_consensus_core::test_staging_write::pause()),
            StagingImportComponent::Shielded => Some(kaspa_consensus_core::test_staging_write::pause_shielded()),
            _ => None,
        }
    } else {
        None
    };
    TransferPause {
        case,
        write_pause,
        sender_paused: Some(sender_paused),
        receiver_waiting: Some(receiver_waiting),
        release: Some(release),
        finish: Some(finish),
    }
}

impl TransferPause {
    pub async fn wait_until_paused(&mut self) {
        if let Some(write) = &self.write_pause {
            while !write.is_paused() {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        } else {
            self.sender_paused.take().expect("sender pause was already awaited").await.expect("peer sender exited before pause");
        }
        self.receiver_waiting.take().expect("receiver was already awaited").await.expect("receiver exited before import");
    }

    pub fn release(&mut self) {
        if let Some(finish) = self.finish.take() {
            let _ = finish.send(());
        }
        if let Some(write) = &self.write_pause {
            write.release();
        }
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
    if case.after_payload {
        // The UTXO sender reaches this hook before its explicit done frame.
        // Hold it back until disconnect so buffered payload cannot finish IBD.
        if component == StagingImportComponent::Utxo {
            let finish = case.finish.lock().take();
            if let Some(finish) = finish {
                let _ = finish.await;
            }
        }
        return;
    }
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

pub(crate) async fn receiver_payload_written(component: StagingImportComponent) {
    let case = active().lock().clone();
    let Some(case) = case.filter(|case| case.component == component && case.after_payload) else { return };
    let pending = case.sender.lock().take();
    if let Some(SenderPause { ready, release }) = pending {
        let _ = ready.send(());
        let _ = release.await;
    }
}

struct TamperCase {
    ready: Mutex<Option<oneshot::Sender<()>>>,
    release: Mutex<Option<oneshot::Receiver<()>>>,
    mutated: std::sync::atomic::AtomicBool,
    rejection: Mutex<Option<String>>,
}

fn tamper_case() -> &'static Mutex<Option<Arc<TamperCase>>> {
    static ACTIVE: OnceLock<Mutex<Option<Arc<TamperCase>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

/// One-shot wire corruption. No consensus data or authenticated metadata is changed.
pub struct TamperedTransfer {
    case: Arc<TamperCase>,
    ready: Option<oneshot::Receiver<()>>,
    release: Option<oneshot::Sender<()>>,
}

pub fn tamper_next_nullifier_payload() -> TamperedTransfer {
    let (ready, wait_ready) = oneshot::channel();
    let (release, wait_release) = oneshot::channel();
    let case = Arc::new(TamperCase {
        ready: Mutex::new(Some(ready)),
        release: Mutex::new(Some(wait_release)),
        mutated: std::sync::atomic::AtomicBool::new(false),
        rejection: Mutex::new(None),
    });
    let mut current = tamper_case().lock();
    assert!(current.is_none(), "a nullifier tamper case is already armed");
    *current = Some(case.clone());
    drop(current);
    TamperedTransfer { case, ready: Some(wait_ready), release: Some(release) }
}

impl TamperedTransfer {
    pub async fn wait_until_ready(&mut self) {
        self.ready.take().expect("tamper readiness already awaited").await.expect("sender exited before the nonempty payload");
    }

    pub fn release(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }

    pub fn rejection(&self) -> Option<String> {
        self.case.rejection.lock().clone()
    }
}

impl Drop for TamperedTransfer {
    fn drop(&mut self) {
        self.release();
        let mut current = tamper_case().lock();
        if current.as_ref().is_some_and(|case| Arc::ptr_eq(case, &self.case)) {
            current.take();
        }
    }
}

pub(crate) async fn tamper_nullifier_chunk(nullifiers: &mut [Vec<u8>]) {
    use std::sync::atomic::Ordering;
    let case = tamper_case().lock().clone();
    let Some(case) = case else { return };
    let Some(first) = nullifiers.first_mut().filter(|nf| !nf.is_empty()) else { return };
    if case.mutated.swap(true, Ordering::AcqRel) {
        return;
    }
    first[0] ^= 1;
    let ready = case.ready.lock().take();
    if let Some(ready) = ready {
        let _ = ready.send(());
    }
    let release = case.release.lock().take();
    if let Some(release) = release {
        let _ = release.await;
    }
}

/// Called only after the receiver has joined the actual shielded importer.
pub(crate) fn record_shielded_rejection(message: String) {
    use std::sync::atomic::Ordering;
    let case = tamper_case().lock().clone();
    if let Some(case) = case.filter(|case| case.mutated.load(Ordering::Acquire)) {
        *case.rejection.lock() = Some(message);
    }
}
