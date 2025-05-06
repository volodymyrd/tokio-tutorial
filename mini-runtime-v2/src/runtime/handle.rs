use crate::runtime::scheduler;
use crate::util::error::{CONTEXT_MISSING_ERROR, THREAD_LOCAL_DESTROYED_ERROR};
use std::{error, fmt};

/// Handle to the runtime.
///
/// The handle is internally reference-counted and can be freely cloned. A handle can be
/// obtained using the [`Runtime::handle`] method.
///
/// [`Runtime::handle`]: crate::runtime::Runtime::handle()
#[derive(Debug, Clone)]
pub struct Handle {
    pub(crate) inner: scheduler::Handle,
}

enum TryCurrentErrorKind {
    NoContext,
    ThreadLocalDestroyed,
}

/// Error returned by `try_current` when no Runtime has been started
#[derive(Debug)]
pub struct TryCurrentError {
    kind: TryCurrentErrorKind,
}

impl TryCurrentError {
    pub(crate) fn new_no_context() -> Self {
        Self {
            kind: TryCurrentErrorKind::NoContext,
        }
    }

    pub(crate) fn new_thread_local_destroyed() -> Self {
        Self {
            kind: TryCurrentErrorKind::ThreadLocalDestroyed,
        }
    }
}

impl fmt::Debug for TryCurrentErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryCurrentErrorKind::NoContext => f.write_str("NoContext"),
            TryCurrentErrorKind::ThreadLocalDestroyed => f.write_str("ThreadLocalDestroyed"),
        }
    }
}

impl fmt::Display for TryCurrentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TryCurrentErrorKind as E;
        match self.kind {
            E::NoContext => f.write_str(CONTEXT_MISSING_ERROR),
            E::ThreadLocalDestroyed => f.write_str(THREAD_LOCAL_DESTROYED_ERROR),
        }
    }
}

impl error::Error for TryCurrentError {}
