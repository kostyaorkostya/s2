use super::Bits9;

#[derive(Debug, Default, Clone, Copy)]
pub struct BoolMatrix9x9(u128);

impl BoolMatrix9x9 {
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    pub fn set(&mut self, idx: (u8, u8)) {
        self.0 |= 1u128 << (idx.0 * 9 + idx.1)
    }

    pub fn unset(&mut self, idx: (u8, u8)) {
        self.0 &= !(1u128 << (idx.0 * 9 + idx.1))
    }

    pub fn row(&self, row: u8) -> Bits9 {
        ((self.0 >> (row * 9)) as u16).into()
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = Bits9> {
        let this = *self;
        (0u8..9u8).map(move |row| this.row(row))
    }
}
