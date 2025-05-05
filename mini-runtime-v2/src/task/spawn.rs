use crate::task::JoinHandle;

/// Spawns a new asynchronous task, returning a
/// [`JoinHandle`](JoinHandle) for it.
///
/// The provided future will start running in the background immediately
/// when `spawn` is called, even if you don't await the returned
/// `JoinHandle`.
///
/// The function takes a single argument named future, of type F.
///
/// Returns JoinHandle<F::Output>. F::Output refers to the associated type of the Future trait.
/// It’s the type the future returns when it's done.
///
/// where -This clause gives extra constraints on the generic types:
/// F: Future - F must implement the Future trait — meaning it’s a future that can be awaited;
/// + Send  - The future must be safe to move to another thread (sendable across threads);
/// + 'static - The future owns all the data it references and doesn’t borrow non-static references.
/// In other words, it must live for the entire duration of the program
/// (or be completely self-contained).
///
/// F::Output: Send + 'static - The result the future produces must also be sendable across
/// threads and live for 'static.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    use crate::runtime::{context, task};
    let id = task::Id::next();
    match context::with_current(|handle| handle.spawn(future, id)) {
        Ok(join_handle) => join_handle,
        Err(e) => panic!("{}", e),
    }
}
