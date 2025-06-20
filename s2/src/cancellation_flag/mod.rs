pub trait CancellationFlag: Send + Sync {
    fn cancelled(&self) -> bool;
}

mod r#const;
pub type AlreadyCancelled = r#const::Const<true>;
pub type NeverCancelled = r#const::Const<false>;

mod atomic;
pub use atomic::Atomic;
