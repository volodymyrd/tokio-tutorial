/// These types are often used as "markers" within other, more complex structs or enums.
/// By including a field of type SyncNotSend or NotSendOrSync (often within a PhantomData to
/// avoid increasing the size of the parent type), a type can explicitly opt out of being Send or
/// Sync even if its other fields would normally allow it.
///
///
/// Understanding Send and Sync in Rust
///
/// Send: A marker trait that indicates that a type is safe to be transferred to another thread.
/// If a type T is Send, you can move a value of T from one thread to another.
/// Sync: A marker trait that indicates that a type is safe to be shared between threads.
/// If a type T is Sync, then &T (a shared reference to T) is Send, meaning you can send
/// a shared reference to T to another thread and access it concurrently.
///
/// Most primitive types and types composed entirely of Send and Sync types automatically
/// implement these traits. However, types that provide interior mutability without
/// synchronization (like RefCell or Cell) or raw pointers (*const T, *mut T) are generally
/// not Send or Sync by default because the compiler cannot guarantee thread safety.
///
/// By default, the presence of a raw pointer (*mut ()) within SyncNotSend prevents
/// the automatic implementation of both Send and Sync for SyncNotSend.
/// The compiler is conservative and assumes that types containing raw pointers are not thread-safe
/// unless explicitly told otherwise.
#[allow(dead_code)]
pub(crate) struct SyncNotSend(#[allow(dead_code)] *mut ());

/// By using unsafe impl Sync, the programmer is making a guarantee to the compiler that it is safe
/// to share &SyncNotSend across multiple threads. This means that even though SyncNotSend contains
/// a raw pointer, the way it's used (or intended to be used) in the broader code ensures
/// that shared, concurrent access is safe.
///
/// This is an unsafe operation because the compiler does not verify this guarantee.
/// If the programmer's assertion is incorrect and concurrent access to a SyncNotSend leads to
/// data races or other memory safety issues, it results in undefined behavior.
///
/// Crucially, this unsafe impl Sync only implements Sync. It does not automatically implement Send.
/// Because the default Send implementation was blocked by the raw pointer, and we have only
/// manually implemented Sync, SyncNotSend becomes a type that is Sync but not Send.
unsafe impl Sync for SyncNotSend {}

/// Unlike SyncNotSend, there is no unsafe impl for either Send or Sync for NotSendOrSync.
/// Because it contains a raw pointer, and neither Send nor Sync are automatically implemented
/// or manually implemented, NotSendOrSync remains a type that is neither Send nor Sync by default.
pub(crate) struct NotSendOrSync(#[allow(dead_code)] *mut ());
