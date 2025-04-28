use super::grid::{GridIdx, GridValue};
use std::iter::FromIterator;
use std::ops::Index;

pub trait Solver {
    fn solve<Grid, Placement>(&self, grid: &Grid) -> Placement
    where
        Grid: Index<GridIdx, Output = Option<GridValue>>,
        Placement: FromIterator<(GridIdx, GridValue)>;
}
