use std::ptr;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering::AcqRel;

/// A thread-safe mutable memory location.
///
/// `AtomicCell` provides a way to have a mutable value that can be safely
/// read and written to by multiple threads concurrently without requiring
/// a traditional `Mutex` for every access. It is similar in concept to
/// `std::cell::Cell` but is thread-safe.
///
/// The `AtomicCell` achieves thread-safety by storing the data inside a
/// `Box<T>` on the heap and atomically managing the pointer to this data
/// using an `AtomicPtr<T>`.
///
/// Note that operations on `AtomicCell` replace the *entire* contained value
/// atomically. This makes it suitable for scenarios where you need to
/// atomically swap or replace the data, but less so for fine-grained
/// in-place mutation of the contained value (unless `T` itself provides
/// atomic operations).
pub(crate) struct AtomicCell<T> {
    data: AtomicPtr<T>,
}

// Safety:
// AtomicCell is Send if T is Send because the data is managed
// via an AtomicPtr and operations are atomic. The memory is
// properly managed via Box::into_raw and Box::from_raw and
// the Drop implementation.
unsafe impl<T: Send> Send for AtomicCell<T> {}

// Safety:
// AtomicCell is Sync if T is Send because access to the data
// is controlled by atomic operations on the AtomicPtr.
unsafe impl<T: Send> Sync for AtomicCell<T> {}

impl<T> AtomicCell<T> {
    /// Creates a new `AtomicCell` containing the optional boxed data.
    ///
    /// If `data` is `Some(boxed_value)`, the cell will contain the value.
    /// If `data` is `None`, the cell will be empty.
    pub(crate) fn new(data: Option<Box<T>>) -> AtomicCell<T> {
        AtomicCell {
            // Convert the Option<Box<T>> to a raw pointer. If None, it's null.
            // Box::into_raw consumes the Box without deallocating the memory,
            // giving us the raw pointer.
            data: AtomicPtr::new(to_raw(data)),
        }
    }

    /// Atomically swaps the current contained value with the new value.
    ///
    /// This operation is atomic with respect to other operations on the same
    /// `AtomicCell`. The ordering used is `AcqRel`, which ensures that
    /// operations before the swap in the current thread are visible to a
    /// thread that subsequently loads the new value, and operations after
    /// the swap are not affected by prior operations in other threads.
    ///
    /// Returns the value that was previously contained in the cell.
    pub(crate) fn swap(&self, val: Option<Box<T>>) -> Option<Box<T>> {
        // Atomically swap the raw pointer.
        let old = self.data.swap(to_raw(val), AcqRel);
        // Convert the old raw pointer back into an Option<Box<T>>, taking
        // ownership of the memory if the pointer was not null.
        from_raw(old)
    }

    /// Sets the contained value, dropping the previous one if it existed.
    ///
    /// This is a convenience method that uses `swap` internally.
    pub(crate) fn set(&self, val: Box<T>) {
        // Swap with the new value, ignoring the old value (which is dropped
        // when the Option<Box<T>> returned by swap goes out of scope).
        let _ = self.swap(Some(val));
    }

    /// Takes the contained value, leaving the cell empty.
    ///
    /// This is a convenience method that uses `swap` internally to replace
    /// the current value with `None`.
    ///
    /// Returns the value that was previously contained in the cell, or `None`
    /// if the cell was already empty.
    pub(crate) fn take(&self) -> Option<Box<T>> {
        // Swap with None, taking the old value.
        self.swap(None)
    }
}

/// Converts an `Option<Box<T>>` into a raw mutable pointer `*mut T`.
///
/// If the `Option` is `Some`, the `Box` is consumed and the raw pointer
/// to the allocated memory is returned. The caller is responsible for
/// managing the lifetime and deallocation of this pointer.
/// If the `Option` is `None`, a null pointer is returned.
fn to_raw<T>(data: Option<Box<T>>) -> *mut T {
    data.map_or(ptr::null_mut(), Box::into_raw)
}

/// Converts a raw mutable pointer `*mut T` into an `Option<Box<T>>`.
///
/// If the pointer is not null, it is converted back into a `Box<T>`,
/// taking ownership of the allocated memory. This operation is `unsafe`
/// because the caller must guarantee that the pointer is valid,
/// was originally created from a `Box<T>`, and that no other `Box` or
/// reference to the same memory exists.
/// If the pointer is null, `None` is returned.
fn from_raw<T>(val: *mut T) -> Option<Box<T>> {
    if val.is_null() {
        None
    } else {
        // Safety: The caller must ensure the pointer is valid and
        // originated from a Box that is no longer owned elsewhere.
        Some(unsafe { Box::from_raw(val) })
    }
}

impl<T> Drop for AtomicCell<T> {
    /// Frees the data still held by the cell when it is dropped.
    ///
    /// This is done by calling `take()`, which performs an atomic swap
    /// with `None` and returns the contained `Option<Box<T>>`. When this
    /// `Option<Box<T>>` goes out of scope, the `Box` is dropped, and
    /// the memory is deallocated.
    fn drop(&mut self) {
        // Free any data still held by the cell by taking it.
        let _ = self.take();
    }
}
