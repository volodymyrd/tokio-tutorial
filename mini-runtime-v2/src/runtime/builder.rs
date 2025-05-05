use crate::runtime::Runtime;
use crate::runtime::scheduler::CurrentThread;
use std::io;
use std::thread::ThreadId;

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    CurrentThread,
}

/// Builds Runtime with custom configuration values.
pub struct Builder {
    /// Runtime type
    kind: Kind,
}

impl Builder {
    pub fn new_current_thread() -> Builder {
        Builder::new(Kind::CurrentThread)
    }

    /// Returns a new runtime builder initialized with default configuration
    /// values.
    ///
    /// Configuration methods can be chained on the return value.
    pub(crate) fn new(kind: Kind) -> Builder {
        Builder { kind }
    }

    pub fn build(&mut self) -> io::Result<Runtime> {
        match &self.kind {
            Kind::CurrentThread => self.build_current_thread_runtime(),
        }
    }

    fn build_current_thread_runtime(&mut self) -> io::Result<Runtime> {
        use crate::runtime::runtime::Scheduler;

        let scheduler = self.build_current_thread_runtime_components(None)?;

        Ok(Runtime::from_parts(Scheduler::CurrentThread(scheduler)))
    }

    fn build_current_thread_runtime_components(
        &mut self,
        _local_tid: Option<ThreadId>,
    ) -> io::Result<CurrentThread> {
        Ok(CurrentThread::new())
    }
}
