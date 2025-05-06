use super::{CONTEXT, Context};
use crate::runtime::{TryCurrentError, scheduler};
use crate::util::markers::SyncNotSend;
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;

#[derive(Debug)]
#[must_use]
pub(crate) struct SetCurrentGuard {
    #[allow(dead_code)]
    // The previous handle
    prev: Option<scheduler::Handle>,

    #[allow(dead_code)]
    // The depth for this guard
    depth: usize,

    // Don't let the type move across threads.
    _p: PhantomData<SyncNotSend>,
}

pub(super) struct HandleCell {
    /// Current handle
    handle: RefCell<Option<scheduler::Handle>>,

    /// Tracks the number of nested calls to `try_set_current`.
    depth: Cell<usize>,
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

impl Context {
    pub(super) fn set_current(&self, handle: &scheduler::Handle) -> SetCurrentGuard {
        let old_handle = self.current.handle.borrow_mut().replace(handle.clone());
        let depth = self.current.depth.get();

        assert!(depth != usize::MAX, "reached max `enter` depth");

        let depth = depth + 1;
        self.current.depth.set(depth);

        SetCurrentGuard {
            prev: old_handle,
            depth,
            _p: PhantomData,
        }
    }
}

impl HandleCell {
    pub(super) const fn new() -> HandleCell {
        HandleCell {
            handle: RefCell::new(None),
            depth: Cell::new(0),
        }
    }
}
