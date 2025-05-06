use super::{BlockingRegionGuard, CONTEXT, SetCurrentGuard};
use crate::runtime::scheduler;
use crate::util::rand::{FastRand, RngSeed};

#[derive(Debug, Clone, Copy)]
#[must_use]
pub(crate) enum EnterRuntime {
    /// Currently in a runtime context.
    Entered {
        #[allow(dead_code)]
        allow_block_in_place: bool,
    },

    /// Not in a runtime context **or** a blocking region.
    NotEntered,
}

/// Guard tracking that a caller has entered a runtime context.
#[must_use]
pub(crate) struct EnterRuntimeGuard {
    /// Tracks that the current thread has entered a blocking function call.
    pub(crate) blocking: BlockingRegionGuard,

    #[allow(dead_code)] // Only tracking the guard.
    pub(crate) handle: SetCurrentGuard,

    #[allow(dead_code)]
    // Tracks the previous random number generator seed
    old_seed: RngSeed,
}

/// Marks the current thread as being within the dynamic extent of an
/// executor.
/// - Mark the current thread as "inside the runtime."
/// - Prevent nested runtime entry, like calling block_on from within another runtime.
/// -
#[track_caller]
pub(crate) fn enter_runtime<F, R>(handle: &scheduler::Handle, allow_block_in_place: bool, f: F) -> R
where
    F: FnOnce(&mut BlockingRegionGuard) -> R,
{
    // CONTEXT.runtime tells each thread whether it has already "entered" a runtime.
    // Think of CONTEXT.runtime as a "thread-specific notepad".
    // Every thread has its own notepad where it writes.
    let maybe_guard = CONTEXT.with(|c| {
        if c.runtime.get().is_entered() {
            None
        } else {
            // Set the entered flag
            c.runtime.set(EnterRuntime::Entered {
                allow_block_in_place,
            });

            // Generate a new seed, ensures each thread running on the runtime has its
            // own RNG stream, which avoids collisions or bias.
            let rng_seed = handle.seed_generator().next_seed();

            // Swap the RNG seed.
            // We need RNG swap logic to avoid leaking or corrupting global thread-local state
            // when entering and exiting a runtime multiple times or using shared threads.
            // - Thread state isolation: Prevents RNG from polluting outer thread state;
            // - Restore previous RNG: If a thread already had a random generator, it’s put back
            // when runtime exits;
            // - Support re-entrancy: Threads may enter and exit the runtime many times;
            // - Multiple runtimes: Allows using different runtimes without them interfering with
            // each other’s RNG;
            // - Predictability: Enables deterministic random behavior per-runtime, which is useful
            // for debugging and test.
            let mut rng = c.rng.get().unwrap_or_else(FastRand::new);
            let old_seed = rng.replace_seed(rng_seed);
            c.rng.set(Some(rng));

            Some(EnterRuntimeGuard {
                blocking: BlockingRegionGuard::new(),
                handle: c.set_current(handle),
                old_seed,
            })
        }
    });

    if let Some(mut guard) = maybe_guard {
        return f(&mut guard.blocking);
    }

    panic!(
        "Cannot start a runtime from within a runtime. This happens \
            because a function (like `block_on`) attempted to block the \
            current thread while the thread is being used to drive \
            asynchronous tasks."
    );
}

impl EnterRuntime {
    pub(crate) fn is_entered(self) -> bool {
        matches!(self, EnterRuntime::Entered { .. })
    }
}
