//! Asynchronous green-threads.

pub use crate::runtime::task::JoinHandle;

mod spawn;
pub use spawn::spawn;
