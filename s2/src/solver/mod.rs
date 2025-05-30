use crate::grid::{GridIdx, GridValue};
use std::iter::FromIterator;
use std::ops::Index;
use thiserror::Error;

mod greedy_solver;
pub use greedy_solver::GreedySolver;

#[derive(Debug, Error, Eq, PartialEq)]
#[error("Sudoku is either infeasible or constraints are already violated")]
pub struct SolverError;

pub trait Solver {
    fn solve<Grid, Placement>(&self, grid: &Grid) -> Result<Placement, SolverError>
    where
        Grid: Index<GridIdx, Output = Option<GridValue>>,
        Placement: FromIterator<(GridIdx, GridValue)>;
}
