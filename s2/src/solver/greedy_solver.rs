use super::{Solver, SolverError};
use crate::grid::{
    ArrGridRowMajor, Grid, GridDiff, GridIdx, GridMut, GridMutWithDefault, GridValue,
};
use std::iter::{once, zip};
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

    fn set_many<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (GridIdx, GridValue)>,
    {
        for (idx, elt) in iter {
            self.set(idx, elt)
        }
    }

    fn unset(&mut self, idx: GridIdx, value: GridValue) {
        let (i, j, box_) = Self::constraint_indices(idx);
        let value: u8 = value.into();
        self.rows.unset((i, value));
        self.cols.unset((j, value));
        self.boxes.unset((box_, value));
    }

    fn unset_many<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (GridIdx, GridValue)>,
    {
        for (idx, elt) in iter {
            self.unset(idx, elt)
        }
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
        SolverStackTail::from(self).with(f)
    }
}

#[derive(Debug)]
struct Diff([(GridIdx, GridValue); GridIdx::COUNT]);

impl Default for Diff {
    fn default() -> Self {
        Self([(GridIdx::default(), GridValue::default()); GridIdx::COUNT])
    }
}

impl Diff {
    fn iter(&self, len: usize) -> impl Iterator<Item = GridDiff> {
        self.0[..len]
            .iter()
            .map(|(idx, value)| GridDiff::Set(*idx, *value))
    }
}

struct DiffTail<'a>(&'a mut [(GridIdx, GridValue)]);

impl<'a> From<&'a mut [(GridIdx, GridValue)]> for DiffTail<'a> {
    fn from(slice: &'a mut [(GridIdx, GridValue)]) -> Self {
        Self(slice)
    }
}

impl<'a> From<&'a mut Diff> for DiffTail<'a> {
    fn from(diff: &'a mut Diff) -> Self {
        (&mut diff.0[..]).into()
    }
}

impl<'a> DiffTail<'a> {
    fn push<I>(&mut self, iter: I) -> usize
    where
        I: Iterator<Item = (GridIdx, GridValue)>,
    {
        let mut cnt = 0;
        for (arr_elt, elt) in zip(self.0.iter_mut(), iter) {
            *arr_elt = elt;
            cnt += 1;
        }
        cnt
    }

    fn with<I, F>(&mut self, iter: I, f: F) -> Result<usize, SolverError>
    where
        I: Iterator<Item = (GridIdx, GridValue)>,
        F: FnOnce(&[(GridIdx, GridValue)], &mut DiffTail<'_>) -> Result<usize, SolverError>,
    {
        let cnt = self.push(iter);
        let (head, tail) = self.0.split_at_mut(cnt);
        Ok(f(&head, &mut tail.into())? + cnt)
    }
}

#[derive(Debug, Default)]
struct SolverState {
    stack: SolverStack,
    grid: ArrGridRowMajor,
    constraints: Constraints,
    diff: Diff,
}

impl SolverState {
    fn of_grid<T>(grid: &T) -> Self
    where
        T: Grid + ?Sized,
    {
        Self {
            grid: ArrGridRowMajor::copy_of(grid),
            constraints: Constraints::of_grid(grid),
            ..Default::default()
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
    fn with<F, R>(&mut self, f: F) -> R
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
    diff: &mut DiffTail<'_>,
) -> Result<usize, SolverError>
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
        [] => Ok(0),
        [(_, 0)] | [.., (_, 0)] => Err(SolverError),
        empty_cells => empty_cells
            .iter()
            .flat_map(|(idx, _)| {
                frame.domain.clear();
                constraints.domain(*idx, &mut frame.domain);
                frame.domain.iter().map(|value| {
                    diff.with(once((*idx, *value)), |set, diff| {
                        cur.set_from_iter(set.iter().copied());
                        constraints.set_many(set.iter().copied());
                        match stack
                            .with(|stack, frame| solve_rec(stack, frame, cur, constraints, diff))
                        {
                            ok @ Ok(_) => ok,
                            err @ Err(_) => {
                                constraints.unset_many(set.iter().copied());
                                cur.unset_from_iter(set.iter().map(|(x, _)| x).copied());
                                err
                            }
                        }
                    })
                })
            })
            .find_map(Result::ok)
            .ok_or(SolverError),
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
        let mut mem = Box::new(SolverState::of_grid(grid));
        let len = mem.stack.with(|stack, frame| {
            solve_rec(
                stack,
                frame,
                &mut mem.grid,
                &mut mem.constraints,
                &mut (&mut mem.diff).into(),
            )
        })?;
        Ok(mem.diff.iter(len).collect::<U>())
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
