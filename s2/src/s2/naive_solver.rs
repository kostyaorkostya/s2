use super::grid::{copy, GridIdx, GridValue, IIdx, JIdx};
use super::plain_grid::PlainGrid;
use super::solver::Solver;
use super::status::{eval_status, SudokuStatus};
use itertools::Itertools;
use std::ops::Index;
use strum::IntoEnumIterator;

#[derive(Debug, Default)]
pub struct NaiveSolver {}

fn solve_rec<I>(cur: &mut PlainGrid, mut iter: I, none_values: usize) -> Result<SudokuStatus, ()>
where
    I: Iterator<Item = GridIdx> + Clone,
{
    let none_values = if none_values == 0 {
        return Err(());
    } else {
        none_values - 1
    };

    let idx = match iter.find(|idx| cur[*idx].is_none()) {
        None => {
            unreachable!("Assuming sudoku grid was valid and incomplete we'll never hit this case")
        }
        Some(idx) => idx,
    };
    for value in GridValue::iter() {
        cur[idx] = Some(value);
        match eval_status(cur) {
            Err(()) => continue,
            x @ Ok(SudokuStatus::Complete) => return x,
            Ok(SudokuStatus::Incomplete) => match solve_rec(cur, iter.clone(), none_values) {
                Ok(SudokuStatus::Incomplete) => {
                    unreachable!("If [Ok], can only be [Complete]")
                }
                Err(()) => continue,
                x @ Ok(SudokuStatus::Complete) => return x,
            },
        }
    }

    cur[idx] = None;
    Err(())
}

impl NaiveSolver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Solver for NaiveSolver {
    fn solve<Grid, Placement>(&self, grid: &Grid) -> Placement
    where
        Grid: Index<GridIdx, Output = Option<GridValue>>,
        Placement: FromIterator<(GridIdx, GridValue)>,
    {
        let mut cur = PlainGrid::new();
        copy(grid, &mut cur);
        match solve_rec(
            &mut cur,
            IIdx::iter().cartesian_product(JIdx::iter()),
            IIdx::iter()
                .cartesian_product(JIdx::iter())
                .filter(|idx| grid[*idx].is_none())
                .count(),
        ) {
            Ok(SudokuStatus::Complete) => (),
            Ok(SudokuStatus::Incomplete) | Err(()) => unreachable!(
                "Assuming sudoku grid was valid and incomplete we would always find a solution"
            ),
        }
        IIdx::iter()
            .cartesian_product(JIdx::iter())
            .filter(|idx| grid[*idx].is_none())
            .map(|idx| (idx, cur[idx].unwrap()))
            .collect::<Placement>()
    }
}
