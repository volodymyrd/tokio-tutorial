use std::fmt;
use std::num::NonZeroU64;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;

/// An opaque ID that uniquely identifies a task relative to all other currently running tasks.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Id(pub(crate) NonZeroU64);

impl Id {
    pub(crate) fn next() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        loop {
            let id = NEXT_ID.fetch_add(1, Relaxed);
            if let Some(id) = NonZeroU64::new(id) {
                return Self(id);
            }
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
