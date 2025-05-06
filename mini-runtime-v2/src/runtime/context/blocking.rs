use crate::util::markers::NotSendOrSync;
use std::marker::PhantomData;

/// Guard tracking that a caller has entered a blocking region.
#[must_use]
pub(crate) struct BlockingRegionGuard {
    _p: PhantomData<NotSendOrSync>,
}

impl BlockingRegionGuard {
    pub(super) fn new() -> BlockingRegionGuard {
        BlockingRegionGuard { _p: PhantomData }
    }
}
