use super::CancellationFlag;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Default)]
pub struct Atomic(AtomicBool);

impl Atomic {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release)
    }
}

impl CancellationFlag for Atomic {
    fn cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
