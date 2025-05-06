use crate::runtime::context;
use crate::runtime::scheduler::{self};
use crate::runtime::task::{self, JoinHandle};
use crate::util::RngSeedGenerator;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::thread::ThreadId;

/// Executes tasks on the current thread
pub(crate) struct CurrentThread {}

/// Handle to the current thread scheduler
pub(crate) struct Handle {
    /// Current random number generator seed
    pub(crate) seed_generator: RngSeedGenerator,

    #[allow(dead_code)]
    /// If this is a `LocalRuntime`, flags the owning thread ID.
    pub(crate) local_tid: Option<ThreadId>,
}

impl CurrentThread {
    pub(crate) fn new(
        seed_generator: RngSeedGenerator,
        local_tid: Option<ThreadId>,
    ) -> (CurrentThread, Arc<Handle>) {
        let handle = Arc::new(Handle {
            seed_generator,
            local_tid,
        });
        let scheduler = CurrentThread {};

        (scheduler, handle)
    }

    pub(crate) fn block_on<F: Future>(&self, handle: &scheduler::Handle, future: F) -> F::Output {
        // pin!(future);
        // Pinning ensures that the memory address of the future doesn't change after it's been
        // polled.
        // Rust requires you to pin the future before polling it to ensure its memory doesn't move.
        let mut future = future;
        unsafe { Pin::new_unchecked(&mut future) };

        context::enter_runtime(handle, false, |_blocking| {
            let _handle = handle.as_current_thread();

            // Attempt to steal the scheduler core and block_on the future if we can
            // there, otherwise, lets select on a notification that the core is
            // available or the future is complete.
            loop {
                println!("starting...");
            }
        })
    }
}

impl fmt::Debug for CurrentThread {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CurrentThread").finish()
    }
}

// ===== impl Handle =====

impl Handle {
    /// Spawns a future onto the `CurrentThread` scheduler
    pub(crate) fn spawn<F>(me: &Arc<Self>, _future: F, id: task::Id) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        println!(
            "Spawns a future onto the `CurrentThread` scheduler {:?} {id}",
            me
        );
        JoinHandle::new()
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("current_thread::Handle { ... }").finish()
    }
}
