use crate::runtime::Handle;
use crate::runtime::scheduler::CurrentThread;

/// The runtime scheduler is either a multi-thread or a current-thread executor.
#[derive(Debug)]
pub(super) enum Scheduler {
    /// Execute all tasks on the current-thread.
    CurrentThread(CurrentThread),
}

#[derive(Debug)]
pub struct Runtime {
    /// Task scheduler
    scheduler: Scheduler,
    /// Handle to runtime, also contains driver handles
    handle: Handle,
}

impl Runtime {
    pub(super) fn from_parts(scheduler: Scheduler, handle: Handle) -> Runtime {
        Runtime { scheduler, handle }
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.block_on_inner(future)
    }

    fn block_on_inner<F: Future>(&self, future: F) -> F::Output {
        match &self.scheduler {
            Scheduler::CurrentThread(exec) => exec.block_on(&self.handle.inner, future),
        }
    }
}
