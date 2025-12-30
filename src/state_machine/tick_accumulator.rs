use std::sync::atomic::{AtomicU64, Ordering};

/// Invariant: Will never decrease
// (unless wrap 😛)
#[derive(Debug, Default)]
pub struct TickAccumulator(AtomicU64);

impl TickAccumulator {
    /// returns old value
    pub fn increment(&self, increment_by: u64) -> u64 {
        self.0.fetch_add(increment_by, Ordering::AcqRel)
    }

    pub fn load(&self) -> u64 {
        self.0.load(Ordering::Acquire)
    }
}
