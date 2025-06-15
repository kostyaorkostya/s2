use super::CancellationFlag;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Default)]
pub struct Atomic(AtomicBool);

impl Atomic {
    pub fn new(cancelled: bool) -> Self {
        Self(AtomicBool::new(cancelled))
    }

    pub fn cancel(&mut self) {
        self.0.store(true, Ordering::Release)
    }
}

impl CancellationFlag for Atomic {
    fn cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
