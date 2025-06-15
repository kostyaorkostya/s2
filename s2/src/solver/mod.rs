use crate::cancellation_flag::CancellationFlag;
use crate::grid::{Grid, GridDiff};
use std::iter::FromIterator;
use thiserror::Error;

mod greedy_solver;
pub use greedy_solver::GreedySolver;

#[derive(Debug, Default, Error, Eq, PartialEq)]
#[error("Sudoku is either infeasible or constraints are already violated")]
pub enum SolverError {
    #[error("infeasible")]
    #[default]
    Infeasible,
    #[error("cancelled")]
    Cancelled,
    #[error("constraints are violated")]
    ConstraintsViolated,
}

impl SolverError {
    pub fn ok_or_cancelled<T>(res: Result<T, Self>) -> Option<Result<T, Self>> {
        match res {
            ok @ Ok(_) => Some(ok),
            err @ Err(SolverError::Cancelled) => Some(err),
            Err(_) => None,
        }
    }
}

pub trait Solver {
    fn solve<C, T, U>(&self, cancellation_flag: &C, grid: &T) -> Result<U, SolverError>
    where
        C: CancellationFlag,
        T: Grid + ?Sized,
        U: FromIterator<GridDiff>;
}
