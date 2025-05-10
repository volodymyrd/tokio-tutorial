//! Provides a mechanism for scoped, thread-local storage.
//!
//! The `Scoped<T>` struct allows a value of type `T` to be temporarily associated
//! with the current thread for the duration of a closure's execution. When the
//! closure finishes (either normally or by panicking), the previously active
//! value (or lack thereof) is restored.
//!
//! This is useful for situations where a piece of context needs to be made
//! available temporarily without passing it explicitly through function arguments,
//! and where nesting of such contexts is possible. For example, in a runtime,
//! the current scheduler context might be set using this mechanism while a task
//! is being polled.
//!
//! # Safety
//!
//! The primary safety consideration when using `Scoped<T>` is the lifetime of the
//! value `t: &T` passed to the `set` method. This value `t` **must** live for the
//! entire duration of the closure `f` executed by `set`. `Scoped<T>` does not
//! take ownership of `T` but merely borrows it temporarily. The `unsafe` block
//! in the `with` method relies on this guarantee from the caller of `set`.

use std::cell::Cell;
use std::ptr;

/// Manages a scoped, thread-local value of type `T`.
///
/// It uses a raw pointer internally, allowing it to represent an unset state
/// (null pointer) and to be set with a temporary borrow of `T`.
/// The `pub(super)` visibility restricts its use to the parent module (`context`)
/// and its submodules.
pub(super) struct Scoped<T> {
    /// Stores a raw pointer to the current value of `T`.
    ///
    /// - `Cell`: Used for interior mutability, as thread-local storage typically
    ///   requires modification without an exclusive reference (`&mut self`) to the
    ///   thread-local static.
    /// - `*const T`: A raw pointer is used instead of `Option<&'a T>` to:
    ///   1. Represent an empty or unset state with `ptr::null()`.
    ///   2. Avoid complex lifetime annotations that would be needed if `Scoped`
    ///      itself tried to manage the lifetime of the borrowed `T` across `set`
    ///      and `with` calls directly in its type signature. The lifetime
    ///      management is effectively handled by the `set` method's RAII guard
    ///      and the caller's responsibility to ensure the borrowed `T` is valid.
   pub inner: Cell<*const T>,
}

impl<T> Scoped<T> {
    /// Creates a new `Scoped<T>` instance, initially without a value set.
    ///
    /// The internal pointer is initialized to `ptr::null()`, signifying that no
    /// value is currently scoped. This is a `const fn`, allowing it to be used
    /// in static initializers like `thread_local! { static MY_VAR: Scoped<i32> = Scoped::new(); }`.
   pub const fn new() -> Scoped<T> {
        Scoped {
            inner: Cell::new(ptr::null()),
        }
    }

    /// Sets a value `t` for the `Scoped` cell for the duration of the closure `f`.
    ///
    /// This method temporarily makes `t` the current value associated with this
    /// `Scoped` instance on the current thread. It first saves the currently
    /// stored pointer, then updates the `inner` cell with the pointer to `t`.
    /// A `Reset` guard is created, which, upon dropping, will restore the
    /// previously saved pointer value. This ensures that the state is correctly
    /// reset even if the closure `f` panics.
    ///
    /// # Parameters
    /// - `t`: A reference to the value of type `T` to be set. **Crucially, this
    ///   value `t` must live for the entire duration of the closure `f`'s execution.**
    ///   `Scoped` only borrows `t`.
    /// - `f`: A closure that will be executed. During its execution, `t` will be
    ///   the value accessible via the `with` method on this `Scoped` instance.
    ///
    /// # Returns
    /// The value returned by the closure `f`.
   pub fn set<F, R>(&self, t: &T, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        /// A RAII guard to ensure the previous value is restored when dropped.
        ///
        /// Created by `Scoped::set` to ensure that the previously held
        /// thread-local value is restored to the `cell` when this struct
        /// goes out of scope, even if the enclosed closure panics.
        struct Reset<'a, T> {
            cell: &'a Cell<*const T>, // Reference to the Scoped's inner Cell.
            prev: *const T,           // The pointer value to restore.
        }

        impl<T> Drop for Reset<'_, T> {
            /// Restores the previous pointer value to the `Cell`.
            fn drop(&mut self) {
                self.cell.set(self.prev);
            }
        }

        // Get the current pointer value to be restored later.
        let prev_ptr = self.inner.get();
        // Set the new value `t` by storing its pointer.
        // `t` is a `&T`, so `t as *const _` casts it to `*const T`.
        self.inner.set(t as *const _);

        // Create the RAII guard. The `_` prefix for `_reset` indicates that
        // its binding is primarily for its side effect (the Drop implementation).
        // This guard will be dropped when the `set` function exits.
        let _reset = Reset {
            cell: &self.inner,
            prev: prev_ptr,
        };

        // Execute the provided closure. The value set above is available
        // within this closure and any functions it calls, until `_reset`
        // is dropped.
        f()
    }

    /// Executes a closure `f` with access to the current scoped value, if any.
    ///
    /// This method retrieves the current pointer from the `inner` cell. If the
    /// pointer is null, it means no value is currently set in the scope, and
    /// the closure `f` is called with `None`. If the pointer is not null, it
    /// is dereferenced to provide a `Some(&T)` to the closure `f`.
    ///
    /// The closure `f` receives an `Option<&T>`:
    /// - `Some(&T)`: If a value is currently set via a surrounding `set` call.
    ///   The lifetime of this `&T` is tied to the duration of the `with` call
    ///   itself, and more broadly, to the lifetime of the `t` provided to the
    ///   enclosing `set` call.
    /// - `None`: If no value is currently set (i.e., the internal pointer is null).
    ///
    /// # Parameters
    /// - `f`: A closure that takes an `Option<&T>` and returns a value of type `R`.
    ///
    /// # Returns
    /// The value returned by the closure `f`.
    ///
    /// # Safety
    /// This method contains an `unsafe` block because it dereferences a raw pointer
    /// (`self.inner.get()`). This operation is considered safe under the following
    /// conditions, which are upheld by the design of `Scoped<T>` and the contract
    /// of the `set` method:
    /// 1. If the pointer `val_ptr` is non-null, it means it was previously set by
    ///    a call to the `set` method.
    /// 2. The `set` method requires that the `t: &T` it borrows is valid for the
    ///    entire duration that `t`'s pointer is stored in `self.inner`.
    /// 3. The `Reset` guard in `set` ensures the pointer is either valid or restored
    ///    to its previous state (which could also be a valid pointer from an outer
    ///    `set` call, or null).
    /// Therefore, a non-null `val_ptr` is assumed to point to a valid `T`.
   pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&T>) -> R,
    {
        // Get the current pointer value from the cell
        let val_ptr = self.inner.get();

        if val_ptr.is_null() {
            // If null, no value is set, call closure with None
            f(None)
        } else {
            // If not null, it points to a valid T reference set by `set`.
            // Dereference the raw pointer to get a reference to T.
            // This is safe because `set` ensures the pointer validity within its scope.
            unsafe { f(Some(&*val_ptr)) }
        }
    }
}
