use super::grid::{to_row_major, GridIdx, GridValue, IIdx, JIdx};
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};
use strum::EnumCount;

#[derive(Clone, Copy)]
pub struct PlainGrid([Option<GridValue>; IIdx::COUNT * JIdx::COUNT]);

impl Index<GridIdx> for PlainGrid {
    type Output = Option<GridValue>;

    fn index(&self, idx: GridIdx) -> &Self::Output {
        &self.0[to_row_major(idx)]
    }
}

impl<T> PartialEq<T> for PlainGrid
where
    T: Index<GridIdx, Output = Option<GridValue>>,
{
    fn eq(&self, other: &T) -> bool {
        super::grid::eq(self, other)
    }
}

impl<T> PartialOrd<T> for PlainGrid
where
    T: Index<GridIdx, Output = Option<GridValue>>,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        super::grid::partial_cmp(self, other)
    }
}

impl IndexMut<GridIdx> for PlainGrid {
    fn index_mut(&mut self, idx: GridIdx) -> &mut Self::Output {
        &mut self.0[to_row_major(idx)]
    }
}

impl std::fmt::Debug for PlainGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = super::format::write_string(&super::format::RowMajorAscii::default(), self);
        f.write_str(&s)
    }
}

impl Default for PlainGrid {
    fn default() -> Self {
        PlainGrid([None; IIdx::COUNT * JIdx::COUNT])
    }
}

impl PlainGrid {
    pub fn new() -> Self {
        Default::default()
    }
}
