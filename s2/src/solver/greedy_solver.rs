use super::Solver;
use crate::grid::PlainGrid;
use crate::grid::{copy_into, GridIdx, GridValue, IIdx, JIdx};
use itertools::Itertools;
use std::ops::{BitOr, Index};
use strum::{EnumCount, IntoEnumIterator};
use tinyvec::ArrayVec;

#[derive(Debug, Default)]
struct Bits9(u16);

impl Bits9 {
    fn count_zeros(&self) -> u8 {
        u16::from(self).count_zeros().try_into().unwrap()
    }

    fn iter_zeros(&self) -> impl Iterator<Item = u8> + '_ {
        (0..9u8).filter(move |&pos| (self.0 & (1u16 << pos)) == 0)
    }
}

impl From<&u16> for Bits9 {
    fn from(v: &u16) -> Self {
        Self(*v)
    }
}

impl From<u16> for Bits9 {
    fn from(v: u16) -> Self {
        Self::from(&v)
    }
}

impl From<&Bits9> for u16 {
    fn from(v: &Bits9) -> Self {
        v.0 & ((1u16 << 9) - 1)
    }
}

impl From<Bits9> for u16 {
    fn from(v: Bits9) -> Self {
        (&v).into()
    }
}

impl BitOr for Bits9 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self.0 | rhs.0).into()
    }
}

#[derive(Debug, Default)]
struct BoolMatrix9x9(u128);

impl BoolMatrix9x9 {
    fn set(&mut self, idx: (u8, u8)) {
        self.0 |= 1u128 << (idx.0 * 9 + idx.1)
    }

    fn unset(&mut self, idx: (u8, u8)) {
        self.0 &= !(1u128 << (idx.0 * 9 + idx.1))
    }

    fn row(&self, idx: u8) -> Bits9 {
        ((self.0 >> (idx * 9)) as u16).into()
    }
}

#[derive(Debug, Default)]
struct Constraints {
    rows: BoolMatrix9x9,
    cols: BoolMatrix9x9,
    boxes: BoolMatrix9x9,
}

impl Constraints {
    fn new() -> Self {
        Default::default()
    }

    fn of_grid<T>(grid: &T) -> Self
    where
        T: Index<GridIdx, Output = Option<GridValue>>,
    {
        let mut t = Self::new();
        IIdx::iter()
            .cartesian_product(JIdx::iter())
            .filter_map(|idx| grid[idx].map(|x| (idx, x)))
            .for_each(|(idx, value)| t.set(idx, value));
        t
    }

    fn constraint_indices(idx: GridIdx) -> (u8, u8, u8) {
        let (i, j): (u8, u8) = (idx.0.into(), idx.1.into());
        (i, j, ((i / 3 * 3) + j / 3))
    }

    fn set(&mut self, idx: GridIdx, value: GridValue) {
        let (i, j, box_) = Self::constraint_indices(idx);
        let value: u8 = value.into();
        self.rows.set((i, value));
        self.cols.set((j, value));
        self.boxes.set((box_, value));
    }

    fn unset(&mut self, idx: GridIdx, value: GridValue) {
        let (i, j, box_) = Self::constraint_indices(idx);
        let value: u8 = value.into();
        self.rows.unset((i, value));
        self.cols.unset((j, value));
        self.boxes.unset((box_, value));
    }

    fn option_mask(&self, idx: GridIdx) -> Bits9 {
        let (i, j, box_) = Self::constraint_indices(idx);
        self.rows.row(i) | self.cols.row(j) | self.boxes.row(box_)
    }

    fn option_count(&self, idx: GridIdx) -> u8 {
        self.option_mask(idx).count_zeros()
    }

    fn options<E>(&self, idx: GridIdx, e: &mut E)
    where
        E: Extend<GridValue>,
    {
        e.extend(
            self.option_mask(idx)
                .iter_zeros()
                .map(|x| x.try_into().unwrap()),
        )
    }
}

#[derive(Debug, Default)]
struct SolverStackFrame {
    empty_cells: ArrayVec<[((IIdx, JIdx), u8); IIdx::COUNT * JIdx::COUNT]>,
    options: ArrayVec<[GridValue; GridValue::COUNT]>,
}

impl SolverStackFrame {
    fn clear(&mut self) {
        self.empty_cells.clear();
        self.options.clear();
    }
}

const SOLVER_RECURSIVE_DEPTH: usize = IIdx::COUNT * JIdx::COUNT + 1;

#[derive(Debug)]
struct SolverStack {
    frames: [SolverStackFrame; SOLVER_RECURSIVE_DEPTH],
    next: usize,
}

impl Default for SolverStack {
    fn default() -> Self {
        Self {
            frames: std::array::from_fn(|_| SolverStackFrame::default()),
            next: 0,
        }
    }
}

impl SolverStack {
    fn new() -> Self {
        Default::default()
    }

    fn push(&mut self) -> &mut SolverStackFrame {
        let frame = &mut self.frames[self.next];
        frame.clear();
        self.next += 1;
        frame
    }

    fn pop(&mut self) {
        self.next -= 1;
    }

    fn with_frame<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut SolverStackFrame) -> R,
    {
        let frame = self.push();
        let result = f(frame);
        self.pop();
        result
    }
}

fn solve_rec(stack: &mut SolverStack, cur: &mut PlainGrid, constraints: &mut Constraints) -> bool {
    let frame = stack.push();

    frame.empty_cells.extend(
        IIdx::iter()
            .cartesian_product(JIdx::iter())
            .filter(|idx| cur[*idx].is_none())
            .map(|idx| (idx, constraints.option_count(idx))),
    );
    frame.empty_cells.sort_by_key(|(_, x)| *x);

    let result = match &frame.empty_cells[..] {
        [] => true,
        [(_, 0)] | [.., (_, 0)] => false,
        empty_cells => {
            let mut complete = false;
            'outer: for (idx, _) in empty_cells {
                constraints.options(*idx, &mut frame.options);
                for value in frame.options {
                    cur[*idx] = Some(value);
                    constraints.set(*idx, value);
                    if solve_rec(stack, cur, constraints) {
                        complete = true;
                        break 'outer;
                    } else {
                        constraints.unset(*idx, value);
                        cur[*idx] = None;
                    }
                }
            }
            complete
        }
    };
    stack.pop();
    result
}

#[derive(Debug, Default)]
pub struct GreedySolver;

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
        let mut state = Box::new((SolverStack::new(), Constraints::of_grid(grid)));
        if solve_rec(&mut state.0, &mut cur, &mut state.1) {
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
