use super::{Solver, SolverError};
use crate::grid::{
    ArrGridRowMajor, Grid, GridDiff, GridIdx, GridMut, GridMutWithDefault, GridValue,
};
use std::ops::BitOr;
use strum::EnumCount;
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
        T: Grid + ?Sized,
    {
        let mut t = Self::new();
        grid.iter_set().for_each(|(idx, value)| t.set(idx, value));
        t
    }

    fn constraint_indices(idx: GridIdx) -> (u8, u8, u8) {
        let (i, j): (u8, u8) = (idx.i.into(), idx.j.into());
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

    fn domain_mask(&self, idx: GridIdx) -> Bits9 {
        let (i, j, box_) = Self::constraint_indices(idx);
        self.rows.row(i) | self.cols.row(j) | self.boxes.row(box_)
    }

    fn domain_size(&self, idx: GridIdx) -> u8 {
        self.domain_mask(idx).count_zeros()
    }

    fn domain<E>(&self, idx: GridIdx, e: &mut E)
    where
        E: Extend<GridValue>,
    {
        e.extend(
            self.domain_mask(idx)
                .iter_zeros()
                .map(|x| x.try_into().unwrap()),
        )
    }
}

#[derive(Debug, Default)]
struct SolverStackFrame {
    empty_cells: ArrayVec<[(GridIdx, u8); GridIdx::COUNT]>,
    domain: ArrayVec<[GridValue; GridValue::COUNT]>,
}

impl SolverStackFrame {
    fn clear(&mut self) {
        self.empty_cells.clear();
        self.domain.clear();
    }
}

const SOLVER_RECURSIVE_DEPTH: usize = GridIdx::COUNT + 1;

#[derive(Debug)]
struct SolverStack([SolverStackFrame; SOLVER_RECURSIVE_DEPTH]);

impl Default for SolverStack {
    fn default() -> Self {
        Self(std::array::from_fn(|_| SolverStackFrame::default()))
    }
}

impl SolverStack {
    fn with<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut SolverStackTail<'_>, &mut SolverStackFrame) -> R,
    {
        SolverStackTail::from(self).with_frame(f)
    }
}

#[derive(Debug)]
struct SolverState {
    stack: SolverStack,
    constraints: Constraints,
}

impl SolverState {
    fn of_grid<T>(grid: &T) -> Self
    where
        T: Grid + ?Sized,
    {
        Self {
            stack: Default::default(),
            constraints: Constraints::of_grid(grid),
        }
    }
}

struct SolverStackTail<'a>(&'a mut [SolverStackFrame]);

impl<'a> From<&'a mut [SolverStackFrame]> for SolverStackTail<'a> {
    fn from(slice: &'a mut [SolverStackFrame]) -> Self {
        Self(slice)
    }
}

impl<'a> From<&'a mut SolverStack> for SolverStackTail<'a> {
    fn from(stack: &'a mut SolverStack) -> Self {
        Self(&mut stack.0[..])
    }
}

impl<'caller> SolverStackTail<'caller> {
    fn with_frame<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut SolverStackTail<'_>, &mut SolverStackFrame) -> R,
    {
        let (frame, tail) = self.0.split_first_mut().unwrap();
        frame.clear();
        f(&mut tail.into(), frame)
    }
}

fn solve_rec<G>(
    stack: &mut SolverStackTail<'_>,
    frame: &mut SolverStackFrame,
    cur: &mut G,
    constraints: &mut Constraints,
) -> bool
where
    G: GridMut,
{
    frame.empty_cells.extend(
        cur.iter_unset()
            .map(|idx| (idx, constraints.domain_size(idx))),
    );
    // radix sort it? there are only 10 possible values to sort by
    frame.empty_cells.sort_by_key(|(_, x)| *x);

    match &frame.empty_cells[..] {
        [] => true,
        [(_, 0)] | [.., (_, 0)] => false,
        empty_cells => empty_cells
            .iter()
            .map(|(x, _)| x)
            .find_map(|idx| {
                frame.domain.clear();
                constraints.domain(*idx, &mut frame.domain);
                for value in frame.domain {
                    cur[*idx] = Some(value);
                    constraints.set(*idx, value);
                    if stack.with_frame(|stack, frame| solve_rec(stack, frame, cur, constraints)) {
                        return Some(true);
                    } else {
                        constraints.unset(*idx, value);
                        cur[*idx] = None;
                    }
                }
                None
            })
            .unwrap_or(false),
    }
}

#[derive(Debug, Default)]
pub struct GreedySolver;

impl GreedySolver {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Solver for GreedySolver {
    fn solve<T, U>(&self, grid: &T) -> Result<U, SolverError>
    where
        T: Grid + ?Sized,
        U: FromIterator<GridDiff>,
    {
        let mut mem = Box::new((ArrGridRowMajor::copy_of(grid), SolverState::of_grid(grid)));
        if mem
            .1
            .stack
            .with(|stack, frame| solve_rec(stack, frame, &mut mem.0, &mut mem.1.constraints))
        {
            Ok(grid.diff(&mem.0).collect::<U>())
        } else {
            Err(SolverError)
        }
    }
}

#[cfg(test)]
mod greedy_solver_test {
    use super::{GreedySolver, Solver, SolverError};
    use crate::format::{read_from_string, write_string, RowMajorAscii};
    use crate::grid::{ArrGridRowMajor, GridMutWithDefault};

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
        let given: ArrGridRowMajor = read_from_string(&RowMajorAscii::default(), given).unwrap();
        let complete = write_string(
            &RowMajorAscii::default(),
            &ArrGridRowMajor::with_diff(
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
        let given: ArrGridRowMajor = read_from_string(&RowMajorAscii::default(), given).unwrap();
        let solution = GreedySolver::new().solve::<_, Vec<_>>(&given);
        assert_eq!(solution, Err(SolverError));
    }
}
