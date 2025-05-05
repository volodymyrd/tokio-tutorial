use std::marker::PhantomData;

/// An owned permission to join on a task (await its termination).
///
/// We are using PhantomData, which is a special marker type.
/// PhantomData consumes no space, but simulates a field of the given type for the purpose
/// of static analysis.
pub struct JoinHandle<T> {
    _p: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub fn new() -> JoinHandle<T> {
        JoinHandle { _p: PhantomData }
    }
}
