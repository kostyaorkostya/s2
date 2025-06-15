use super::Bits9;

#[derive(Debug, Default)]
pub struct BoolMatrix9x9(u128);

impl BoolMatrix9x9 {
    pub fn set(&mut self, idx: (u8, u8)) {
        self.0 |= 1u128 << (idx.0 * 9 + idx.1)
    }

    pub fn unset(&mut self, idx: (u8, u8)) {
        self.0 &= !(1u128 << (idx.0 * 9 + idx.1))
    }

    pub fn row(&self, idx: u8) -> Bits9 {
        ((self.0 >> (idx * 9)) as u16).into()
    }
}
