use std::sync::{Condvar, Mutex, OnceLock};
use std::thread;

struct ReentrantState {
    owner: Option<thread::ThreadId>,
    depth: usize,
}

struct ReentrantLock {
    state: Mutex<ReentrantState>,
    cv: Condvar,
}

static FFI_LOCK: OnceLock<ReentrantLock> = OnceLock::new();

struct FfiGuard {
    tid: thread::ThreadId,
}

impl Drop for FfiGuard {
    fn drop(&mut self) {
        let lock = match FFI_LOCK.get() {
            Some(lock) => lock,
            None => return,
        };

        // Avoid panicking on poisoned locks; a poison here only means a previous caller
        // panicked while holding the lock. We still want to be able to continue.
        let mut state = lock.state.lock().unwrap_or_else(|e| e.into_inner());

        if state.owner != Some(self.tid) {
            // Should be impossible for normal use. Don't panic in Drop; instead, unlock to avoid
            // wedging the whole process if invariants were already broken (e.g. after a panic).
            state.owner = None;
            state.depth = 0;
            lock.cv.notify_all();
            return;
        }

        state.depth = state.depth.saturating_sub(1);
        if state.depth == 0 {
            state.owner = None;
            lock.cv.notify_one();
        }
    }
}

fn lock_ffi() -> FfiGuard {
    let lock = FFI_LOCK.get_or_init(|| ReentrantLock {
        state: Mutex::new(ReentrantState {
            owner: None,
            depth: 0,
        }),
        cv: Condvar::new(),
    });

    let tid = thread::current().id();

    // Avoid panicking on poisoned locks; a poison here only means a previous caller
    // panicked while holding the lock. We still want to be able to continue.
    let mut state = lock.state.lock().unwrap_or_else(|e| e.into_inner());

    loop {
        match state.owner {
            None => {
                state.owner = Some(tid);
                state.depth = 1;
                break;
            }
            Some(owner) if owner == tid => {
                state.depth = state.depth.saturating_add(1);
                break;
            }
            Some(_) => {
                state = lock.cv.wait(state).unwrap_or_else(|e| e.into_inner());
            }
        }
    }

    drop(state);
    FfiGuard { tid }
}

pub(crate) fn with_ffi_lock<R>(f: impl FnOnce() -> R) -> R {
    let _guard = lock_ffi();
    f()
}
