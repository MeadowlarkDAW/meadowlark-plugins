use std::sync::atomic::{AtomicU64, Ordering};

/// Simple atomic floating point variable with relaxed ordering.
///
/// Designed for the common case of sharing VST parameters between
/// multiple threads when no synchronization or change notification
/// is needed.
pub struct AtomicF64 {
    atomic: AtomicU64,
}

impl AtomicF64 {
    /// New atomic float with initial value `value`.
    pub fn new(value: f64) -> AtomicF64 {
        AtomicF64 {
            atomic: AtomicU64::new(value.to_bits()),
        }
    }

    /// Get the current value of the atomic float.
    pub fn get(&self) -> f64 {
        f64::from_bits(self.atomic.load(Ordering::Relaxed))
    }

    /// Set the value of the atomic float to `value`.
    pub fn set(&self, value: f64) {
        self.atomic.store(value.to_bits(), Ordering::Relaxed)
    }
}

impl Default for AtomicF64 {
    fn default() -> Self {
        AtomicF64::new(0.0)
    }
}
