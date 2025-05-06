use crate::runtime::Runtime;
use crate::runtime::handle::Handle;
use crate::runtime::scheduler::CurrentThread;
use crate::util::rand::{RngSeed, RngSeedGenerator};
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

    /// Specify a random number generator seed to provide deterministic results
    pub(super) seed_generator: RngSeedGenerator,
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
        Builder {
            kind,
            seed_generator: RngSeedGenerator::new(RngSeed::new()),
        }
    }

    pub fn build(&mut self) -> io::Result<Runtime> {
        match &self.kind {
            Kind::CurrentThread => self.build_current_thread_runtime(),
        }
    }

    fn build_current_thread_runtime(&mut self) -> io::Result<Runtime> {
        use crate::runtime::runtime::Scheduler;

        let (scheduler, handle) = self.build_current_thread_runtime_components(None)?;

        Ok(Runtime::from_parts(
            Scheduler::CurrentThread(scheduler),
            handle,
        ))
    }

    fn build_current_thread_runtime_components(
        &mut self,
        local_tid: Option<ThreadId>,
    ) -> io::Result<(CurrentThread, Handle)> {
        use crate::runtime::scheduler;

        // And now put a single-threaded scheduler on top of the timer. When
        // there are no futures ready to do something, it'll let the timer or
        // the reactor to generate some new stimuli for the futures to continue
        // in their life.
        let (scheduler, handle) =
            CurrentThread::new(self.seed_generator.next_generator(), local_tid);

        let handle = Handle {
            inner: scheduler::Handle::CurrentThread(handle),
        };

        Ok((scheduler, handle))
    }
}
