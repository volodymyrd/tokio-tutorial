/// A thread-safe mechanism to generate unique, non-zero 64-bit unsigned integer IDs for threads.
/// It uses an atomic counter (NEXT_ID) and a compare-and-swap operation (compare_exchange_weak)
/// within a loop to ensure that each call to ThreadId::next() returns a distinct ID,
/// even when called concurrently from multiple threads.
use std::num::NonZeroU64;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;

/// NonZeroU64 represents a 64-bit unsigned integer that is guaranteed to be non-zero.
/// This is useful for thread IDs because a zero ID might be used to represent an invalid
/// or uninitialized state
#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub(crate) struct ThreadId(NonZeroU64);

impl ThreadId {
    pub(crate) fn next() -> Self {
        // AtomicU64, which is a 64-bit unsigned integer that can be safely accessed and modified
        // concurrently by multiple threads. This is crucial for generating unique IDs without race
        // conditions.
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        // Use Relaxed memory ordering. Memory ordering specifies how memory accesses by one
        // thread are observed by other threads. Relaxed ordering is the weakest form,
        // offering the most performance but with fewer guarantees about the visibility of
        // operations across threads. In this specific case, for simple ID generation,
        // it's often sufficient.
        // This line loads the current value of NEXT_ID using the load operation with Relaxed
        // ordering and stores it in a mutable local variable last.
        let mut last = NEXT_ID.load(Relaxed);

        // This starts an infinite loop that continues until a unique ID is successfully generated
        // and returned. This loop handles potential race conditions.
        loop {
            // This attempts to add 1 to the last ID. checked_add returns an Option<u64>.
            // If the addition would cause an overflow (reaching the maximum value of u64),
            // it returns None.
            let id = match last.checked_add(1) {
                Some(id) => id,
                // Error message indicating that all possible 64-bit unsigned integers have been
                // used as thread IDs. This is a very unlikely scenario in most practical
                // applications.
                None => panic!("failed to generate unique thread ID: bitspace exhausted"),
            };

            // compare_exchange_weak - atomic operation attempts to compare the current value of
            // NEXT_ID with last.
            // If they are equal, it attempts to set the value of NEXT_ID to id and returns Ok(_).
            // This means a unique ID has been successfully acquired.
            // If they are not equal (meaning another thread has updated NEXT_ID in the meantime),
            // it returns Err(current_value), where current_value is the current value of NEXT_ID.
            match NEXT_ID.compare_exchange_weak(last, id, Relaxed, Relaxed) {
                // The .unwrap() is safe here because id is guaranteed to be at least 1
                // (since we started from 0 and incremented), so NonZeroU64::new()
                // will always return Some.
                Ok(_) => return ThreadId(NonZeroU64::new(id).unwrap()),
                // the loop continues to try again with the updated value.
                // This is a "spin loop" that retries until a unique ID can be atomically acquired.
                Err(id) => last = id,
            }
        }
    }
}
