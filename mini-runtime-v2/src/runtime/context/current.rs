use super::CONTEXT;
use crate::runtime::{TryCurrentError, scheduler};
use std::cell::RefCell;

pub(super) struct HandleCell {
    /// Current handle
    handle: RefCell<Option<scheduler::Handle>>,
}

impl HandleCell {
    pub(super) const fn new() -> HandleCell {
        HandleCell {
            handle: RefCell::new(None),
        }
    }
}

pub(crate) fn with_current<F, R>(f: F) -> Result<R, TryCurrentError>
where
    F: FnOnce(&scheduler::Handle) -> R,
{
    match CONTEXT.try_with(|ctx| ctx.current.handle.borrow().as_ref().map(f)) {
        Ok(Some(ret)) => Ok(ret),
        Ok(None) => Err(TryCurrentError::new_no_context()),
        Err(_access_error) => Err(TryCurrentError::new_thread_local_destroyed()),
    }
}
