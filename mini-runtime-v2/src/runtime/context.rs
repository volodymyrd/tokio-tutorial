mod current;

pub(crate) use current::{SetCurrentGuard, with_current};
use std::cell::Cell;

mod runtime;
pub(crate) use runtime::{EnterRuntime, enter_runtime};

mod blocking;
pub(crate) use blocking::BlockingRegionGuard;

use crate::util::rand::FastRand;

struct Context {
    /// Handle to the runtime scheduler running on the current thread.
    current: current::HandleCell,

    /// Tracks if the current thread is currently driving a runtime.
    /// Note, that if this is set to "entered", the current scheduler
    /// handle may not reference the runtime currently executing. This
    /// is because other runtime handles may be set to current from
    /// within a runtime.
    runtime: Cell<EnterRuntime>,

    /// Used for fair scheduling, random selection, jitter, etc.
    ///
    /// Uses Lock-free & lightweight FastRand (compare to Global RNG (thread_rng)),
    /// can control seed,
    rng: Cell<Option<FastRand>>,
}

mini_runtime_thread_local! {
    static CONTEXT: Context = const {
        Context {
             current: current::HandleCell::new(),

            // Tracks if the current thread is currently driving a runtime.
            // Note, that if this is set to "entered", the current scheduler
            // handle may not reference the runtime currently executing. This
            // is because other runtime handles may be set to current from
            // within a runtime.
            runtime: Cell::new(EnterRuntime::NotEntered),

            rng: Cell::new(None),
        }
    }
}
