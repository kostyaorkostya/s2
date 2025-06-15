use super::CancellationFlag;

#[derive(Debug, Default)]
pub struct Const<const CANCELLED: bool>;

impl<const CANCELLED: bool> Const<CANCELLED> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<const CANCELLED: bool> CancellationFlag for Const<CANCELLED> {
    fn cancelled(&self) -> bool {
        CANCELLED
    }
}
