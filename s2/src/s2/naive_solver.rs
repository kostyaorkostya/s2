use super::grid::{copy, GridIdx, GridValue, IIdx, JIdx};
use super::plain_grid::PlainGrid;
use super::solver::Solver;
use std::iter::zip;
use std::ops::Index;
use strum::IntoEnumIterator;

#[derive(Debug, Default)]
pub struct NaiveSolver {}

impl Solver for NaiveSolver {
    fn solve<Grid, Placement>(&self, grid: &Grid) -> Placement
    where
        Grid: Index<GridIdx, Output = Option<GridValue>>,
        Placement: FromIterator<(GridIdx, GridValue)>,
    {
        let mut current = PlainGrid::new();
        copy(grid, &mut current);
        zip(IIdx::iter(), JIdx::iter())
            .filter(|idx| grid[idx.clone()].is_none())
            .map(|idx| (idx, current[idx].unwrap()))
            .collect::<Placement>()
    }
}

impl NaiveSolver {
    pub fn new() -> Self {
        Self::default()
    }
}
