use crate::cancellation_token::CancellationToken;
use crate::grid::{Grid, GridDiff};
use std::iter::FromIterator;
use thiserror::Error;

mod greedy_solver;
pub use greedy_solver::GreedySolver;

#[derive(Debug, Error, Eq, PartialEq)]
#[error("Sudoku is either infeasible or constraints are already violated")]
pub enum SolverError {
    #[error("cancelled")]
    Cancelled,
    #[error("infeasible")]
    Infeasible,
    #[error("constraints are violated")]
    ConstraintsViolated,
}

pub trait Solver {
    fn solve<C, T, U>(&self, cancellation_token: &C, grid: &T) -> Result<U, SolverError>
    where
        C: CancellationToken,
        T: Grid + ?Sized,
        U: FromIterator<GridDiff>;
}
