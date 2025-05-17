use super::Solver;
use crate::grid::PlainGrid;
use crate::grid::{copy_into, GridIdx, GridValue, IIdx, JIdx};
use fixedbitset::FixedBitSet;
use itertools::Itertools;
use std::ops::{BitOr, Index};
use strum::{EnumCount, IntoEnumIterator};

#[derive(Debug)]
struct Constraint(FixedBitSet);

impl Default for Constraint {
    fn default() -> Self {
        Self(FixedBitSet::with_capacity(GridValue::COUNT))
    }
}

impl Constraint {
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
            .map(|x| GridValue::try_from(x + 1).unwrap())
            .collect::<C>()
    }

    fn option_count(&self) -> usize {
        self.0.count_zeroes(..)
    }
}

impl BitOr for &Constraint {
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

    fn of_grid<T>(grid: &T) -> Self
    where
        T: Index<GridIdx, Output = Option<GridValue>>,
    {
        let mut res = Self::new();
        IIdx::iter()
            .cartesian_product(JIdx::iter())
            .filter_map(|idx| grid[idx].map(|x| (idx, x)))
            .for_each(|(idx, value)| res.set(idx, value));
        res
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
    let empty_cells = IIdx::iter()
        .cartesian_product(JIdx::iter())
        .filter(|idx| cur[*idx].is_none())
        .map(|idx| (idx, constraints.option_count(idx)))
        .sorted_by_key(|(_, option_count)| *option_count)
        .collect::<Vec<_>>();
    match &empty_cells[..] {
        [] => return true,
        [.., (_, 0)] => return false,
        _ => (),
    };

    for (idx, _) in empty_cells {
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

    false
}

impl GreedySolver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Solver for GreedySolver {
    fn solve<Grid, Placement>(&self, grid: &Grid) -> Result<Placement, ()>
    where
        Grid: Index<GridIdx, Output = Option<GridValue>>,
        Placement: FromIterator<(GridIdx, GridValue)>,
    {
        let mut cur: PlainGrid = copy_into(grid);
        // TODO(kostya): remove debugging
        let tmp: PlainGrid = copy_into(grid);
        let mut constraints = Constraints::of_grid(grid);
        if solve_rec(&mut cur, &mut constraints) {
            Ok(IIdx::iter()
                .cartesian_product(JIdx::iter())
                .filter(|idx| grid[*idx].is_none())
                .map(|idx| {
                    (
                        idx,
                        cur[idx].expect(&format!("{:?}\n{:?}\n\n{:?}", idx, tmp, cur)),
                    )
                })
                .collect::<Placement>())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod greedy_solver_test {
    use super::{GreedySolver, Solver};
    use crate::format::{read_from_string, write_string, RowMajorAscii};
    use crate::grid::{copy_and_apply, PlainGrid};

    #[test]
    fn test_feasible() {
        let given = r#"
53__7____
6__195___
_98____6_
8___6___3
4__8_3__1
7___2___6
_6____28_
___419__5
____8__79
"#
        .trim();
        let expected = r#"
534678912
672195348
198342567
859761423
426853791
713924856
961537284
287419635
345286179
"#
        .trim();
        let given: PlainGrid = read_from_string(&RowMajorAscii::default(), given).unwrap();
        let complete = write_string(
            &RowMajorAscii::default(),
            &copy_and_apply::<_, PlainGrid, _>(
                &given,
                GreedySolver::new()
                    .solve::<_, Vec<_>>(&given)
                    .unwrap()
                    .into_iter(),
            ),
        );
        assert_eq!(&expected, &complete);
    }

    #[test]
    fn test_infeasible() {
        let given = r#"
_271_5___
15__34___
936___7__
_8_72_456
____4_1__
__1____3_
___913_4_
___456___
_4_8_____
"#
        .trim();
        let given: PlainGrid = read_from_string(&RowMajorAscii::default(), given).unwrap();
        let solution = GreedySolver::new().solve::<_, Vec<_>>(&given);
        assert_eq!(solution, Err(()));
    }
}
