use super::{GridIdx, GridValue};
use crate::format;
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy)]
pub struct PlainGrid([Option<GridValue>; GridIdx::COUNT]);

impl Default for PlainGrid {
    fn default() -> Self {
        Self([None; GridIdx::COUNT])
    }
}

impl Index<GridIdx> for PlainGrid {
    type Output = Option<GridValue>;

    fn index(&self, idx: GridIdx) -> &Self::Output {
        &self.0[idx.row_major()]
    }
}

impl IndexMut<GridIdx> for PlainGrid {
    fn index_mut(&mut self, idx: GridIdx) -> &mut Self::Output {
        &mut self.0[idx.row_major()]
    }
}

impl super::Grid for PlainGrid {}

impl super::GridMut for PlainGrid {}

impl super::GridMutWithDefault for PlainGrid {}

impl<T> PartialEq<T> for PlainGrid
where
    T: super::Grid,
{
    fn eq(&self, other: &T) -> bool {
        super::eq(self, other)
    }
}

impl<T> PartialOrd<T> for PlainGrid
where
    T: super::Grid,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        Some(super::cmp(self, other))
    }
}

impl std::fmt::Debug for PlainGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format::write_string(&format::RowMajorAscii::default(), self);
        f.write_str(&s)
    }
}
