use super::grid::{render, to_row_major, GridIdx, GridValue, IIdx, JIdx};
use std::ops::{Index, IndexMut};
use strum::EnumCount;

#[derive(Debug, Clone, Copy)]
pub struct PlainGrid([Option<GridValue>; IIdx::COUNT * JIdx::COUNT]);

impl Index<GridIdx> for PlainGrid {
    type Output = Option<GridValue>;

    fn index(&self, idx: GridIdx) -> &Self::Output {
        &self.0[to_row_major(idx)]
    }
}

impl IndexMut<GridIdx> for PlainGrid {
    fn index_mut(&mut self, idx: GridIdx) -> &mut Self::Output {
        &mut self.0[to_row_major(idx)]
    }
}

impl std::fmt::Display for PlainGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        render(self, f)
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
