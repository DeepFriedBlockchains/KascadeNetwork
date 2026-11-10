use itertools::Itertools;
use kaspa_consensus_core::api::{ConsensusApi, DynConsensus, StagingPromotion, StagingPromotionError};
use kaspa_core::{core::Core, debug, service::Service};
use parking_lot::RwLock;
use std::{
    collections::VecDeque,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard, oneshot};

mod batch;
mod session;

pub use batch::BlockProcessingBatch;
pub use session::{
    ConsensusInstance, ConsensusProxy, ConsensusSessionBlocking, ConsensusSessionOwned, SessionLock, SessionReadGuard,
    SessionWriteGuard, spawn_blocking,
};

/// Consensus controller trait. Includes methods required to start/stop/control consensus, but which should not
/// be exposed to ordinary users
pub trait ConsensusCtl: Sync + Send {
    /// Initialize and start workers etc    
    fn start(&self) -> Vec<JoinHandle<()>>;

    /// Shutdown all workers and clear runtime resources
    fn stop(&self);

    /// Set as current active consensus
    fn make_active(&self);
}

pub type DynConsensusCtl = Arc<dyn ConsensusCtl>;

pub trait ConsensusFactory: Sync + Send {
    /// Load an instance of current active consensus or create one if no such exists
    fn new_active_consensus(&self) -> (ConsensusInstance, DynConsensusCtl);

    /// Create a new empty staging consensus
    fn new_staging_consensus(&self) -> (ConsensusInstance, DynConsensusCtl);

    /// Close the factory and cleanup any shared resources used by it
    fn close(&self);

    /// If the node is not configured as archival -- delete inactive consensus entries and their databases  
    fn delete_inactive_consensus_entries(&self);

    /// Delete the staging consensus entry and its database (this is done even if the node is archival
    /// since staging reflects non-final data)
    fn delete_staging_entry(&self);
}

/// Test-only mock factory
struct MockFactory;

impl ConsensusFactory for MockFactory {
    fn new_active_consensus(&self) -> (ConsensusInstance, DynConsensusCtl) {
        unimplemented!()
    }

    fn new_staging_consensus(&self) -> (ConsensusInstance, DynConsensusCtl) {
        unimplemented!()
    }

    fn close(&self) {
        unimplemented!()
    }

    fn delete_inactive_consensus_entries(&self) {
        unimplemented!()
    }

    fn delete_staging_entry(&self) {
        unimplemented!()
    }
}

/// Defines a trait which handles consensus resets for external parts of the system. We avoid using
/// the generic notification system since the reset needs to be handled synchronously in order to
/// retain state consistency
pub trait ConsensusResetHandler: Send + Sync {
    fn handle_consensus_reset(&self);
}

/// Wraps all needed structures required for interacting and controlling a consensus instance
struct ConsensusInner {
    consensus: ConsensusInstance,
    ctl: DynConsensusCtl,
}

impl ConsensusInner {
    fn new(consensus: ConsensusInstance, ctl: DynConsensusCtl) -> Self {
        Self { consensus, ctl }
    }
}

struct ManagerInner {
    /// Current consensus
    current: ConsensusInner,

    /// Service join handles
    handles: VecDeque<JoinHandle<()>>,

    /// Handlers called when the consensus is reset to a staging consensus
    consensus_reset_handlers: Vec<Arc<dyn ConsensusResetHandler>>,
}

impl ManagerInner {
    fn new(consensus: ConsensusInstance, ctl: DynConsensusCtl) -> Self {
        Self {
            current: ConsensusInner::new(consensus, ctl),
            handles: Default::default(),
            consensus_reset_handlers: Default::default(),
        }
    }
}

pub struct ConsensusManager {
    factory: Arc<dyn ConsensusFactory>,
    inner: RwLock<ManagerInner>,
    staging_gate: Arc<AsyncMutex<()>>,
}

impl ConsensusManager {
    pub const IDENT: &'static str = "consensus manager";

    pub fn new(factory: Arc<dyn ConsensusFactory>) -> Self {
        let (consensus, ctl) = factory.new_active_consensus();
        Self { factory, inner: RwLock::new(ManagerInner::new(consensus, ctl)), staging_gate: Default::default() }
    }

    /// Creates a consensus manager with a fixed consensus. Will panic if staging API is used. To be
    /// used for test purposes only.
    pub fn from_consensus<T: ConsensusApi + ConsensusCtl + 'static>(consensus: Arc<T>) -> Self {
        let (consensus, ctl) = (consensus.clone() as DynConsensus, consensus as DynConsensusCtl);
        Self {
            factory: Arc::new(MockFactory),
            inner: RwLock::new(ManagerInner::new(ConsensusInstance::new(SessionLock::new(), consensus), ctl)),
            staging_gate: Default::default(),
        }
    }

    pub fn consensus(&self) -> ConsensusInstance {
        self.inner.read().current.consensus.clone()
    }

    pub fn new_staging_consensus(self: &Arc<Self>) -> StagingConsensus {
        let (consensus, ctl) = self.factory.new_staging_consensus();
        StagingConsensus::new(self.clone(), ConsensusInner::new(consensus, ctl))
    }

    pub async fn new_managed_staging_consensus(self: &Arc<Self>) -> ManagedStagingConsensus {
        let gate = self.staging_gate.clone().lock_owned().await;
        let (done, cancelled) = oneshot::channel();
        let stage = self.new_staging_consensus();
        ManagedStagingConsensus {
            owner: StagingOwner(Some(Arc::new(StagingLifetime { stage: Some(stage), gate: Some(gate), done: Some(done), stopped: AtomicBool::new(false) }))),
            cancelled,
        }
    }

    pub fn register_consensus_reset_handler(&self, handler: Arc<dyn ConsensusResetHandler>) {
        self.inner.write().consensus_reset_handlers.push(handler);
    }

    fn worker(&self) {
        let handles = self.inner.read().current.ctl.clone().start();
        self.inner.write().handles.extend(handles);
        // If current consensus is switched, this loop will join the replaced handles, and will switch to waiting for the new ones
        let mut g = self.inner.write();
        while let Some(handle) = g.handles.pop_front() {
            drop(g);
            handle.join().unwrap();
            g = self.inner.write();
        }

        // All consensus instances have been shutdown and we are exiting, so close the factory. Internally this closes
        // the notification root sender channel, leading to a graceful shutdown of the notification sub-system.
        debug!("[Consensus manager] all consensus threads exited");
        self.factory.close();
    }

    pub fn delete_inactive_consensus_entries(&self) {
        self.factory.delete_inactive_consensus_entries();
    }

    pub fn delete_staging_entry(&self) {
        self.factory.delete_staging_entry();
    }

    /// Synchronously invokes all registered consensus-reset handlers.
    ///
    /// This is used to notify external system components that the consensus state
    /// has been explicitly reset. Handlers are invoked synchronously (rather than
    /// via the generic notification system) to ensure state consistency across
    /// components before normal operation resumes.
    pub fn invoke_consensus_reset_handlers(&self) {
        let handlers = self.inner.read().consensus_reset_handlers.iter().cloned().collect_vec();
        for handler in handlers {
            handler.handle_consensus_reset();
        }
    }
}

impl Service for ConsensusManager {
    fn ident(self: Arc<Self>) -> &'static str {
        Self::IDENT
    }

    fn start(self: Arc<Self>, _core: Arc<Core>) -> Vec<JoinHandle<()>> {
        vec![std::thread::spawn(move || self.worker())]
    }

    fn stop(self: Arc<Self>) {
        self.inner.read().current.ctl.clone().stop();
    }
}

/// The staging DB and its factory slot outlive every worker proxy.
pub struct ManagedStagingConsensus {
    owner: StagingOwner,
    cancelled: oneshot::Receiver<()>,
}

struct StagingOwner(Option<Arc<StagingLifetime>>);

impl Drop for StagingOwner {
    fn drop(&mut self) {
        if let Some(lifetime) = self.0.as_ref() {
            lifetime.stop();
        }
    }
}

struct StagingLifetime {
    stage: Option<StagingConsensus>,
    gate: Option<OwnedMutexGuard<()>>,
    done: Option<oneshot::Sender<()>>,
    stopped: AtomicBool,
}

impl StagingLifetime {
    fn stop(&self) {
        if !self.stopped.swap(true, Ordering::AcqRel)
            && let Some(stage) = &self.stage
        {
            stage.staging.ctl.stop();
        }
    }
}

impl Drop for StagingLifetime {
    fn drop(&mut self) {
        self.stop();
        if let Some(stage) = self.stage.take() {
            let gate = self.gate.take();
            let done = self.done.take();
            std::thread::spawn(move || {
                stage.finish_cancel();
                drop(gate);
                if let Some(done) = done {
                    let _ = done.send(());
                }
            });
        }
    }
}

impl ManagedStagingConsensus {
    fn lifetime(&self) -> &Arc<StagingLifetime> {
        self.owner.0.as_ref().expect("managed staging receiver is available")
    }

    fn stage(&self) -> &StagingConsensus {
        self.lifetime().stage.as_ref().expect("managed staging receiver is available")
    }

    pub async fn session(&self) -> ConsensusProxy {
        self.stage().session().await.with_staging_lifetime(self.lifetime().clone())
    }

    pub fn unguarded_session(&self) -> ConsensusProxy {
        self.stage().unguarded_session().with_staging_lifetime(self.lifetime().clone())
    }

    pub async fn cancel(self) {
        let Self { owner, cancelled } = self;
        drop(owner);
        let _ = cancelled.await;
    }

    pub async fn commit(mut self, promotion: Option<StagingPromotion>) -> Result<(), StagingPromotionError> {
        let lifetime = self.owner.0.take().expect("managed staging receiver is available");
        let mut lifetime = match Arc::try_unwrap(lifetime) {
            Ok(lifetime) => lifetime,
            Err(lifetime) => {
                lifetime.stop();
                drop(lifetime);
                let _ = self.cancelled.await;
                return Err(StagingPromotionError::Rejected("a staging worker still holds the receiver".into()));
            }
        };
        spawn_blocking(move || {
            let stage = lifetime.stage.take().expect("managed staging receiver is available");
            if let Some(promotion) = promotion {
                if let Err((stage, error)) = stage.commit_verified(promotion) {
                    stage.cancel();
                    return Err(error);
                }
            } else if stage.unguarded_session_blocking().supports_verified_staging_import() {
                stage.cancel();
                return Err(StagingPromotionError::InvalidReceipt);
            } else {
                stage.commit();
            }
            drop(lifetime);
            Ok(())
        })
        .await
        .map_err(|error| StagingPromotionError::Rejected(error.to_string()))?
    }
}

pub struct StagingConsensus {
    manager: Arc<ConsensusManager>,
    staging: ConsensusInner,
    handles: VecDeque<JoinHandle<()>>,
}

impl StagingConsensus {
    fn new(manager: Arc<ConsensusManager>, staging: ConsensusInner) -> Self {
        let handles = VecDeque::from_iter(staging.ctl.start());
        Self { manager, staging, handles }
    }

    /// Verify the receipt on the staging receiver before making it active.
    /// Return the receiver on rejection so the caller can drain workers before cancellation.
    pub fn commit_verified(self, promotion: StagingPromotion) -> Result<(), (Box<Self>, StagingPromotionError)> {
        let result = self.staging.consensus.unguarded_session_blocking().authorize_staging_promotion(promotion);
        if let Err(error) = result {
            return Err((Box::new(self), error));
        }
        self.commit();
        Ok(())
    }

    pub fn commit(self) {
        let mut g = self.manager.inner.write();
        let prev = std::mem::replace(&mut g.current, self.staging);
        g.handles.extend(self.handles);
        prev.ctl.stop();
        g.current.ctl.make_active();
        drop(g);
        self.manager.invoke_consensus_reset_handlers();
        // Drop `prev` so that deletion below succeeds
        drop(prev);
        // Staging was committed and is now the active consensus so we can delete
        // any previous, now inactive, consensus entries
        self.manager.delete_inactive_consensus_entries();
    }

    pub fn cancel(self) {
        self.staging.ctl.stop();
        self.finish_cancel();
    }

    fn finish_cancel(self) {
        for handle in self.handles {
            handle.join().unwrap();
        }
        // Drop staging (and DB refs therein) so that the delete operation below succeeds
        drop(self.staging);
        // Delete the canceled staging consensus
        self.manager.delete_staging_entry();
    }
}

impl Deref for StagingConsensus {
    type Target = ConsensusInstance;

    fn deref(&self) -> &Self::Target {
        &self.staging.consensus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubConsensus;

    impl ConsensusApi for StubConsensus {}

    impl ConsensusCtl for StubConsensus {
        fn start(&self) -> Vec<JoinHandle<()>> {
            vec![]
        }

        fn stop(&self) {}

        fn make_active(&self) {
            panic!("an unauthorized staging receiver became active");
        }
    }

    struct StagingTestFactory {
        deleted: Arc<std::sync::atomic::AtomicBool>,
    }

    impl StagingTestFactory {
        fn receiver() -> (ConsensusInstance, DynConsensusCtl) {
            let receiver = Arc::new(StubConsensus);
            (ConsensusInstance::new(SessionLock::new(), receiver.clone()), receiver)
        }
    }

    impl ConsensusFactory for StagingTestFactory {
        fn new_active_consensus(&self) -> (ConsensusInstance, DynConsensusCtl) {
            Self::receiver()
        }

        fn new_staging_consensus(&self) -> (ConsensusInstance, DynConsensusCtl) {
            Self::receiver()
        }

        fn close(&self) {}

        fn delete_inactive_consensus_entries(&self) {}

        fn delete_staging_entry(&self) {
            self.deleted.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[test]
    fn authority_integration_managed_staging_waits_for_detached_receiver() {
        use std::sync::atomic::Ordering;

        let deleted = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let manager = Arc::new(ConsensusManager::new(Arc::new(StagingTestFactory { deleted: deleted.clone() })));
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        runtime.block_on(async {
            let stage = manager.new_managed_staging_consensus().await;
            let session = stage.session().await;
            let mut cancel = Box::pin(stage.cancel());
            assert!(futures_util::poll!(cancel.as_mut()).is_pending());
            assert!(!deleted.load(Ordering::SeqCst));
            assert!(manager.staging_gate.try_lock().is_err());
            drop(session);
            cancel.await;
            assert!(deleted.load(Ordering::SeqCst));
            assert!(manager.staging_gate.try_lock().is_ok());
        });
    }

    #[test]
    fn authority_integration_manager_rejects_unsupported_promotion() {
        let manager = Arc::new(ConsensusManager::from_consensus(Arc::new(StubConsensus)));
        let receiver = Arc::new(StubConsensus);
        let staging = StagingConsensus::new(
            manager.clone(),
            ConsensusInner::new(ConsensusInstance::new(SessionLock::new(), receiver.clone()), receiver),
        );

        let Err((staging, StagingPromotionError::Unsupported)) = staging.commit_verified(StagingPromotion::new(())) else {
            panic!("unsupported receiver did not reject staging promotion");
        };
        assert!(Arc::ptr_eq(&staging.manager, &manager));
    }
}
