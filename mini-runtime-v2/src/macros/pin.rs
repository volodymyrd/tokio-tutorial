/// Pins a value on the stack.
///
/// Calls to `async fn` return anonymous [`Future`] values that are `!Unpin`.
/// These values must be pinned before they can be polled. Calling `.await` will
/// handle this, but consumes the future. If it is required to call `.await` on
/// a `&mut _` reference, the caller is responsible for pinning the future.
///
/// Pinning may be done by allocating with [`Box::pin`] or by using the stack
/// with the `pin!` macro.
#[macro_export]
macro_rules! pin {
    ($($x:ident),*) => { $(
        // Move the value to ensure that it is owned
        let mut $x = $x;
        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $x = unsafe {
            std::pin::Pin::new_unchecked(&mut $x)
        };
    )* };
    ($(
            let $x:ident = $init:expr;
    )*) => {
        $(
            let $x = $init;
            $crate::pin!($x);
        )*
    };
}
