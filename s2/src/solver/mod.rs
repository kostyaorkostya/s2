use crate::grid::{GridIdx, GridValue};
use std::iter::FromIterator;
use std::ops::Index;

mod greedy_solver;
pub use greedy_solver::GreedySolver;

pub trait Solver {
    fn solve<Grid, Placement>(&self, grid: &Grid) -> Result<Placement, ()>
    where
        Grid: Index<GridIdx, Output = Option<GridValue>>,
        Placement: FromIterator<(GridIdx, GridValue)>;
}
