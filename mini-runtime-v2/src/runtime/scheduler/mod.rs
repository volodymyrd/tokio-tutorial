pub(crate) mod current_thread;
pub(crate) use current_thread::CurrentThread;

use crate::runtime::task::Id;
use crate::task::JoinHandle;
use loom::sync::Arc;

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
}
