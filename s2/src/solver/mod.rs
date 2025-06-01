use crate::grid::{Grid, GridDiff};
use std::iter::FromIterator;
use thiserror::Error;

mod greedy_solver;
pub use greedy_solver::GreedySolver;

#[derive(Debug, Error, Eq, PartialEq)]
#[error("Sudoku is either infeasible or constraints are already violated")]
pub struct SolverError;

pub trait Solver {
    fn solve<T, U>(&self, grid: &T) -> Result<U, SolverError>
    where
        T: Grid + ?Sized,
        U: FromIterator<GridDiff>;
}
