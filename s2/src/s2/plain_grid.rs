use super::grid::*;
use strum::EnumCount;

#[derive(Debug, Default)]
pub struct PlainGrid([Option<GridValue>; IIdx::COUNT * JIdx::COUNT]);

impl Grid for PlainGrid {
    fn index(&self, idx: Idx) -> &Self::Output {
        grid.0[Idx.to_row_major(idx)]
    }

    fn index_mut(&mut self, idx: Idx) -> &mut Self::Output {
        &mut grid.0[Idx.to_row_major(idx)]
    }
}
