/// Error string explaining that the Tokio context hasn't been instantiated.
pub(crate) const CONTEXT_MISSING_ERROR: &str =
    "there is no reactor running, must be called from the context of a Mini runtime";

/// Error string explaining that the Tokio context is not available because the
/// thread-local storing it has been destroyed. This usually only happens during
/// destructors of other thread-locals.
pub(crate) const THREAD_LOCAL_DESTROYED_ERROR: &str =
    "The Tokio context thread-local variable has been destroyed.";
