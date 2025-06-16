use super::Bits9;
use crate::grid::GridValue;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Domain(Bits9);

impl From<&Bits9> for Domain {
    fn from(v: &Bits9) -> Self {
        Self(*v)
    }
}

impl From<Bits9> for Domain {
    fn from(v: Bits9) -> Self {
        Self::from(&v)
    }
}

impl Domain {
    pub fn size(&self) -> u8 {
        self.0.count_zeros()
    }

    pub fn iter(&self) -> impl Iterator<Item = GridValue> + use<> {
        self.0.iter_zeros().map(move |x| x.try_into().unwrap())
    }
}
