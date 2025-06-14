use super::CancellationToken;

#[derive(Debug, Default)]
pub struct Const<const CANCELLED: bool>;

impl<const CANCELLED: bool> Const<CANCELLED> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<const CANCELLED: bool> CancellationToken for Const<CANCELLED> {
    fn cancelled(&self) -> bool {
        CANCELLED
    }
}
