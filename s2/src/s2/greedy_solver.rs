use super::grid::{copy_into, GridIdx, GridValue, IIdx, JIdx};
use super::plain_grid::PlainGrid;
use super::solver::Solver;
use fixedbitset::FixedBitSet;
use itertools::Itertools;
use std::ops::{BitOr, Index};
use strum::{EnumCount, IntoEnumIterator};

#[derive(Debug, Default)]
struct Constraint(FixedBitSet);

impl Constraint {
    fn new() -> Self {
        Self(FixedBitSet::with_capacity(GridValue::COUNT))
    }

    fn violates(&self, value: GridValue) -> bool {
        let idx: usize = value.into();
        self.0.contains(idx)
    }

    fn set(&mut self, value: GridValue) {
        let idx: usize = value.into();
        self.0.insert(idx)
    }

    fn unset(&mut self, value: GridValue) {
        let idx: usize = value.into();
        self.0.remove(idx)
    }

    fn options<C>(&self) -> C
    where
        C: FromIterator<GridValue>,
    {
        self.0
            .zeroes()
            .map(|x| GridValue::try_from(x).unwrap())
            .collect::<C>()
    }

    fn option_count(&self) -> usize {
        self.0.count_zeroes(..)
    }
}

impl<'a> BitOr for &'a Constraint {
    type Output = Constraint;

    fn bitor(self, rhs: &Constraint) -> Constraint {
        Constraint(&self.0 | &rhs.0)
    }
}

#[derive(Debug, Default)]
struct Constraints {
    rows: [Constraint; IIdx::COUNT],
    cols: [Constraint; JIdx::COUNT],
    sub3x3s: [Constraint; (IIdx::COUNT / 3) * (JIdx::COUNT / 3)],
}

impl Constraints {
    fn new() -> Self {
        Default::default()
    }

    fn violates(&self, idx: GridIdx, value: GridValue) -> bool {
        let (i, j): (usize, usize) = (idx.0.into(), idx.1.into());
        self.rows[i].violates(value)
            || self.cols[j].violates(value)
            || self.sub3x3s[(i / 3 * 3) + j / 3].violates(value)
    }

    fn set(&mut self, idx: GridIdx, value: GridValue) {
        let (i, j): (usize, usize) = (idx.0.into(), idx.1.into());
        self.rows[i].set(value);
        self.cols[j].set(value);
        self.sub3x3s[(i / 3 * 3) + j / 3].set(value);
    }

    fn unset(&mut self, idx: GridIdx, value: GridValue) {
        let (i, j): (usize, usize) = (idx.0.into(), idx.1.into());
        self.rows[i].unset(value);
        self.cols[j].unset(value);
        self.sub3x3s[(i / 3 * 3) + j / 3].unset(value);
    }

    fn options<C>(&self, idx: GridIdx) -> C
    where
        C: FromIterator<GridValue>,
    {
        let (i, j): (usize, usize) = (idx.0.into(), idx.1.into());
        (&(&self.rows[i] | &self.cols[j]) | &self.sub3x3s[(i / 3 * 3) + j / 3]).options()
    }

    fn option_count(&self, idx: GridIdx) -> usize {
        let (i, j): (usize, usize) = (idx.0.into(), idx.1.into());
        (&(&self.rows[i] | &self.cols[j]) | &self.sub3x3s[(i / 3 * 3) + j / 3]).option_count()
    }
}

#[derive(Debug, Default)]
pub struct GreedySolver {}

fn solve_rec(cur: &mut PlainGrid, constraints: &mut Constraints) -> bool {
    let options = IIdx::iter()
        .cartesian_product(JIdx::iter())
        .filter(|idx| cur[*idx].is_none())
        .sorted_by_cached_key(|idx| constraints.option_count(*idx))
        .collect::<Vec<_>>();
    if options.is_empty() {
        return true;
    }

    for idx in options {
        let options: Vec<_> = constraints.options(idx);
        for value in options {
            cur[idx] = Some(value);
            constraints.set(idx, value);
            if solve_rec(cur, constraints) {
                return true;
            } else {
                constraints.unset(idx, value);
                cur[idx] = None;
            }
        }
    }

    return false;
}

impl GreedySolver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Solver for GreedySolver {
    fn solve<Grid, Placement>(&self, grid: &Grid) -> Placement
    where
        Grid: Index<GridIdx, Output = Option<GridValue>>,
        Placement: FromIterator<(GridIdx, GridValue)>,
    {
        let mut cur: PlainGrid = copy_into(grid);
        let mut constraints = Constraints::new();
        let _: bool = solve_rec(&mut cur, &mut constraints);
        IIdx::iter()
            .cartesian_product(JIdx::iter())
            .filter(|idx| grid[*idx].is_none())
            .map(|idx| (idx, cur[idx].unwrap()))
            .collect::<Placement>()
    }
}
