pub(crate) mod current_thread;

pub(crate) use current_thread::CurrentThread;
use std::sync::Arc;

use crate::runtime::task::Id;
use crate::task::JoinHandle;
use crate::util::RngSeedGenerator;

macro_rules! match_flavor {
    ($self:expr, $ty:ident($h:ident) => $e:expr) => {
        match $self {
            $ty::CurrentThread($h) => $e,
        }
    };
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum Handle {
    CurrentThread(Arc<current_thread::Handle>),
}

impl Handle {
    pub(crate) fn spawn<F>(&self, future: F, id: Id) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        println!("Try to start spawn in handle...");
        match self {
            Handle::CurrentThread(h) => current_thread::Handle::spawn(h, future, id),
        }
    }

    pub(crate) fn seed_generator(&self) -> &RngSeedGenerator {
        match_flavor!(self, Handle(h) => &h.seed_generator)
    }

    pub(crate) fn as_current_thread(&self) -> &Arc<current_thread::Handle> {
        match self {
            Handle::CurrentThread(handle) => handle,
        }
    }
}
