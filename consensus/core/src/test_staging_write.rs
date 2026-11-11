//! Test-only rendezvous after a real SMT RocksDB batch has committed.
//! No storage or authority lock is acquired here.
use std::sync::{Arc, Condvar, Mutex, OnceLock};

struct State {
    paused: bool,
    released: bool,
}

struct Case {
    state: Mutex<State>,
    changed: Condvar,
}

fn active() -> &'static Mutex<Option<Arc<Case>>> {
    static ACTIVE: OnceLock<Mutex<Option<Arc<Case>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

pub struct WritePause {
    case: Arc<Case>,
}

pub fn pause() -> WritePause {
    let case = Arc::new(Case { state: Mutex::new(State { paused: false, released: false }), changed: Condvar::new() });
    let mut current = active().lock().unwrap();
    assert!(current.is_none(), "an SMT write pause is already armed");
    *current = Some(case.clone());
    WritePause { case }
}

pub fn is_armed() -> bool {
    active().lock().unwrap().is_some()
}

impl WritePause {
    pub fn is_paused(&self) -> bool {
        self.case.state.lock().unwrap().paused
    }

    pub fn release(&self) {
        self.case.state.lock().unwrap().released = true;
        self.case.changed.notify_all();
    }
}

impl Drop for WritePause {
    fn drop(&mut self) {
        self.release();
        let mut current = active().lock().unwrap();
        if current.as_ref().is_some_and(|case| Arc::ptr_eq(case, &self.case)) {
            current.take();
        }
    }
}

/// Called only after DB::write returns, while the importer still owns its
/// worker reservation and before metadata/readiness publication.
pub fn after_smt_write() {
    let case = active().lock().unwrap().clone();
    if let Some(case) = case {
        let mut state = case.state.lock().unwrap();
        state.paused = true;
        while !state.released {
            state = case.changed.wait(state).unwrap();
        }
    }
}
