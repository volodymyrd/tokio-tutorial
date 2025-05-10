//! This code is needed to provide a safe, standard way to create Waker instances from types
//! shared via Arc, which is a fundamental requirement for building asynchronous runtimes and
//! libraries in Rust where tasks often need to signal readiness based on shared state.
//! It handles the complex and unsafe interaction with the low-level RawWaker interface on behalf
//! of the user, allowing them to simply implement the Wake trait on their shared data structure.
//!
//! This code exists to bridge a gap between how asynchronous tasks manage shared state
//! (often using Arc) and the low-level interface required by the Rust standard library's Waker.
//!
//! Here's the breakdown of the "why":
//!
//! 1. Asynchronous Tasks and Wakers: In Rust's async/await ecosystem, when a Future cannot complete
//! immediately (e.g., waiting for I/O), it returns Poll::Pending and provides a Waker
//! to the runtime (or executor). The executor saves this Waker. When the event the Future
//! is waiting on occurs, the Waker is used to notify the executor that the task associated with
//! that Future is now ready to make progress and should be polled again.
//!
//! 2. Shared State: Asynchronous tasks often need to share state. For example, multiple tasks
//! might wait on the same network connection or a shared queue. Arc (Atomically Reference Counted)
//! is the standard Rust type for sharing data safely across threads and tasks.
//! The state that needs to trigger a wake-up (like a completion signal on a network stream)
//! is typically part of this shared state, wrapped inside an Arc.
//!
//! 3. The Waker Interface: The standard library's Waker is designed to be efficient and flexible,
//! working with various underlying mechanisms. However, the core mechanism for creating a Waker
//! from raw components is Waker::from_raw, which requires a RawWaker and a RawWakerVTable.
//! - RawWaker: This struct simply holds a *const () pointer (the "data") and a reference to a
//! RawWakerVTable.
//! - RawWakerVTable: This struct holds function pointers for the four essential low-level
//! operations: clone, wake, wake_by_ref, and drop. These functions receive the *const () data
//! pointer and must know how to perform the respective operation using that pointer.
//!
//! 4. The Problem: The Waker infrastructure doesn't inherently know how to handle an Arc<T>.
//! If your task state is in an Arc<MyTaskState>, and MyTaskState knows how to perform the wake
//! operation, you need a way to create a Waker whose internal data pointer is the pointer to the
//! Arc<MyTaskState>'s contents, and whose RawWakerVTable functions correctly manipulate that
//! specific Arc.
//!
//! 5. This Code's Solution:
//! - The Wake Trait: Defines a standard way for a type W within an Arc to expose its wake
//! functionality (wake and wake_by_ref).
//! - waker_vtable: This function creates the crucial bridge. It generates a RawWakerVTable
//! specifically designed to work with pointers originating from Arc<W>. The functions in this
//! vtable (clone_arc_raw, etc.) use unsafe code to convert the raw *const () pointer back into
//! an Arc<W> (or manipulate its reference count directly) to perform the required operations.
//! - waker_ref: This function provides a safe, convenient entry point. Given a borrow of an Arc<W>,
//! it uses the Arc::as_ptr method to get the raw data pointer and pairs it with the waker_vtable
//! to create a Waker wrapped in WakerRef. WakerRef adds a lifetime constraint to ensure
//! the resulting Waker doesn't outlive the borrowed Arc.
//! - The unsafe Helpers (clone_arc_raw, etc.): These are the core implementations for the vtable.
//! They use unsafe because they directly manipulate raw pointers and the Arc's internal state
//! (increment_strong_count, from_raw). This is necessary because the RawWaker interface operates
//! at a very low level, requiring manual memory management details for the specific data type
//! it wraps (in this case, Arc). The safety relies on the invariant that the *const () passed
//! to these functions is indeed a valid pointer to the data inside an Arc<T> that was created
//! by Arc::as_ptr or a similar mechanism, and that the reference counts are managed correctly
//! by these functions.

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::Arc;
use std::task::{RawWaker, RawWakerVTable, Waker};

/// A trait defining the necessary operations for a type that can be woken
/// up when shared via an `Arc`.
///
/// Types implementing `Wake` are expected to represent the state associated
/// with a task or resource that can notify an executor when it's ready
/// to be polled again.
///
/// The trait bounds `Send + Sync + Sized + 'static` are required because
/// instances of types implementing `Wake` are shared across threads (`Send`, `Sync`),
/// must have a known size at compile time (`Sized`), and must not contain
/// borrowed references tied to a specific, shorter lifetime than the program's (`'static`).
pub(crate) trait Wake: Send + Sync + Sized + 'static {
    /// Wakes the task associated with this `Arc<Self>`.
    ///
    /// This method consumes the `Arc`, effectively dropping one strong reference
    /// count when called. This is suitable for "by value" wake operations.
    fn wake(arc_self: Arc<Self>);

    /// Wakes the task associated with this `Arc<Self>` by reference.
    ///
    /// This method borrows the `Arc` and does not consume it. It is suitable
    /// for "by reference" wake operations where the source `Arc` needs to
    /// remain valid.
    fn wake_by_ref(arc_self: &Arc<Self>);
}

/// A `Waker` that is only valid for a given lifetime `'a`.
///
/// This struct wraps a standard `Waker` but uses `PhantomData` to tie
/// its lifetime to a borrow, typically a borrow of an `Arc<impl Wake>`.
/// This ensures the `Waker` does not outlive the underlying data it refers to.
///
/// It uses `ManuallyDrop` because the actual dropping logic for the
/// wrapped `Waker` is handled by the `RawWakerVTable` functions, not
/// by the `WakerRef`'s own drop implementation.
#[derive(Debug)]
pub(crate) struct WakerRef<'a> {
    // We use ManuallyDrop because the drop glue for the Waker is
    // provided by the RawWakerVTable, specifically the `drop_arc_raw` function.
    waker: ManuallyDrop<Waker>,
    // This PhantomData ensures that WakerRef doesn't outlive the data
    // pointed to by the Waker, which is borrowed for lifetime 'a.
    _p: PhantomData<&'a ()>,
}

impl Deref for WakerRef<'_> {
    type Target = Waker;

    /// Dereferences the `WakerRef` to a `&Waker`.
    ///
    /// This allows calling `Waker` methods directly on a `WakerRef` instance.
    fn deref(&self) -> &Waker {
        // Safety: The Waker is valid for the lifetime of WakerRef as enforced by PhantomData.
        // It is wrapped in ManuallyDrop, so we access the inner value directly.
        &self.waker
    }
}

/// Creates a reference to a `Waker` (`WakerRef`) from a reference to `Arc<impl Wake>`.
///
/// This function is the primary way to create a temporary `Waker` from an `Arc`
/// that manages wakeable state. The resulting `WakerRef` is valid only for
/// the lifetime of the input `Arc` reference.
///
/// The created `Waker`'s internal representation (`RawWaker`) will hold a pointer
/// to the data within the `Arc`, and its operations (clone, wake, drop) will
/// be handled by the `waker_vtable` which knows how to correctly manipulate
/// the `Arc`'s reference count and call the `Wake` trait methods.
pub(crate) fn waker_ref<W: Wake>(wake: &Arc<W>) -> WakerRef<'_> {
    // Get a raw pointer to the data managed by the Arc.
    let ptr = Arc::as_ptr(wake).cast::<()>();

    // Create a RawWaker using the data pointer and the vtable specific to Arc<W>.
    // The vtable provides the low-level functions needed by the Waker.
    // This is unsafe because we are manually creating the RawWaker; the caller
    // relies on the correctness of the vtable functions for memory safety.
    let waker = unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<W>())) };

    // Wrap the created Waker in ManuallyDrop (since drop is handled by vtable)
    // and pair it with PhantomData for lifetime tracking.
    WakerRef {
        waker: ManuallyDrop::new(waker),
        _p: PhantomData,
    }
}

/// Generates a `RawWakerVTable` tailored for `Arc<W>` where `W` implements `Wake`.
///
/// This vtable provides the necessary function pointers (`clone`, `wake`, `wake_by_ref`, `drop`)
/// for a `RawWaker` whose data pointer points to the contents of an `Arc<W>`.
/// These functions correctly interact with the `Arc`'s reference count and call
/// the `Wake` trait methods.
fn waker_vtable<W: Wake>() -> &'static RawWakerVTable {
    // Define the low-level operations for the RawWaker.
    &RawWakerVTable::new(
        clone_arc_raw::<W>,       // How to clone the Waker (increments Arc count)
        wake_arc_raw::<W>,        // How to wake by value (consumes one Arc count)
        wake_by_ref_arc_raw::<W>, // How to wake by reference (borrows Arc)
        drop_arc_raw::<W>,        // How to drop the Waker (decrements Arc count)
    )
}

// --- Raw VTable Helper Functions (unsafe) ---
// These functions operate on a raw pointer that is expected to be
// the data pointer from an Arc<T> where T: Wake.

/// Implements the `clone` operation for `RawWaker` backed by `Arc<T>`.
///
/// This function is called when a `Waker` created from `waker_ref` is cloned.
/// It increments the strong count of the underlying `Arc<T>` using `Arc::increment_strong_count`.
///
/// # Safety
/// This function is unsafe because it assumes `data` is a valid pointer to the data
/// within an `Arc<T>`.
unsafe fn clone_arc_raw<T: Wake>(data: *const ()) -> RawWaker {
    // Increment the strong count of the Arc pointed to by `data`.
    // This is the core of cloning an Arc-based Waker.
    Arc::<T>::increment_strong_count(data as *const T);
    // Return a new RawWaker with the same data pointer and vtable.
    RawWaker::new(data, waker_vtable::<T>())
}

/// Implements the `wake` operation for `RawWaker` backed by `Arc<T>`.
///
/// This function is called when `Waker::wake()` is called on a `Waker` created
/// from `waker_ref`. It reconstructs the `Arc<T>` from the raw pointer using
/// `Arc::from_raw` (which takes ownership of one reference count) and then
/// calls the `Wake::wake` method, consuming the `Arc`.
///
/// # Safety
/// This function is unsafe because it assumes `data` is a valid pointer to the data
/// within an `Arc<T>` and that the `RawWaker` held a valid reference count
/// that can now be consumed via `Arc::from_raw`.
unsafe fn wake_arc_raw<T: Wake>(data: *const ()) {
    // Reconstruct the Arc from the raw pointer. This takes ownership
    // of the reference count held by the RawWaker.
    let arc: Arc<T> = Arc::from_raw(data as *const T);
    // Call the wake method on the Arc. This consumes the Arc.
    Wake::wake(arc);
}

/// Implements the `wake_by_ref` operation for `RawWaker` backed by `Arc<T>`.
///
/// This function is called when `Waker::wake_by_ref()` is called. It reconstructs
/// the `Arc<T>` from the raw pointer temporarily to call `Wake::wake_by_ref`.
/// It uses `ManuallyDrop` to prevent the reconstructed `Arc` from decrementing
/// the reference count when it goes out of scope, as `wake_by_ref` does not
/// consume the ownership.
///
/// # Safety
/// This function is unsafe because it assumes `data` is a valid pointer to the data
/// within an `Arc<T>`. It relies on the correct use of `ManuallyDrop` to avoid
/// incorrect reference count manipulation.
unsafe fn wake_by_ref_arc_raw<T: Wake>(data: *const ()) {
    // Reconstruct the Arc from the raw pointer and wrap it in ManuallyDrop.
    // This gives us a temporary Arc value to borrow from, but prevents
    // the Arc's drop implementation (which would decrement the count) from running.
    let arc = ManuallyDrop::new(Arc::<T>::from_raw(data.cast()));
    // Call the wake_by_ref method using a reference to the Arc.
    Wake::wake_by_ref(&arc);
    // ManuallyDrop ensures the Arc isn't dropped here.
}

/// Implements the `drop` operation for `RawWaker` backed by `Arc<T>`.
///
/// This function is called when a `Waker` created from `waker_ref` is dropped.
/// It reconstructs the `Arc<T>` from the raw pointer using `Arc::from_raw`
/// (taking ownership of one reference count) and then drops it, which decrements
/// the strong count.
///
/// # Safety
/// This function is unsafe because it assumes `data` is a valid pointer to the data
/// within an `Arc<T>` and that the `RawWaker` held a valid reference count
/// that can now be consumed via `Arc::from_raw` and dropped.
unsafe fn drop_arc_raw<T: Wake>(data: *const ()) {
    // Reconstruct the Arc from the raw pointer. This takes ownership
    // of the reference count held by the RawWaker.
    let arc: Arc<T> = Arc::from_raw(data.cast());
    // Drop the Arc, decrementing its strong count.
    drop(arc);
}
