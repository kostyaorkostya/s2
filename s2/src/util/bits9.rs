use bit_iter::BitIter;
use std::ops::BitOr;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bits9(u16);

impl Bits9 {
    pub fn count_zeros(&self) -> u8 {
        (self.0 | !((1u16 << 9) - 1)).count_zeros() as u8
    }

    pub fn count_ones(&self) -> u8 {
        self.0.count_ones() as u8
    }

    pub fn iter_zeros(&self) -> impl Iterator<Item = u8> + use<> {
        BitIter::from(!self.0 & ((1u16 << 9) - 1)).map(|x| x as u8)
    }
}

impl From<&u16> for Bits9 {
    fn from(v: &u16) -> Self {
        Self(*v)
    }
}

impl From<u16> for Bits9 {
    fn from(v: u16) -> Self {
        Self::from(&v)
    }
}

impl From<&Bits9> for u16 {
    fn from(v: &Bits9) -> Self {
        v.0 & ((1u16 << 9) - 1)
    }
}

impl From<Bits9> for u16 {
    fn from(v: Bits9) -> Self {
        (&v).into()
    }
}

impl BitOr for Bits9 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self.0 | rhs.0).into()
    }
}
