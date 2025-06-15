use super::{Solver, SolverError};
use crate::cancellation_flag::CancellationFlag;
use crate::grid::{
    ArrGridRowMajor, Grid, GridDiff, GridIdx, GridMut, GridMutWithDefault, GridValue,
};
use bit_iter::BitIter;
use std::iter::{empty, once, zip};
use std::ops::BitOr;
use strum::EnumCount;

#[derive(Debug, Default, Copy, Clone)]
struct Bits9(u16);

impl Bits9 {
    fn count_zeros(&self) -> u8 {
        (u16::from(self).count_zeros() - (16 - 9))
            .try_into()
            .unwrap()
    }

    fn iter_zeros(&self) -> impl Iterator<Item = u8> + use<> {
        BitIter::from(!self.0).filter(|x| *x < 9).map(|x| x as u8)
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

    fn domain(&self, idx: GridIdx) -> impl Iterator<Item = GridValue> + use<> {
        self.domain_mask(idx)
            .iter_zeros()
            .map(move |x| x.try_into().unwrap())
    }
}

#[derive(Debug, Default)]
struct Domain([GridValue; GridValue::COUNT]);

impl Domain {
    fn with<I, F, R>(&mut self, iter: I, f: F) -> R
    where
        I: Iterator<Item = GridValue>,
        F: FnOnce(&[GridValue]) -> R,
    {
        let mut cnt = 0;
        for (arr_elt, elt) in zip(self.0.iter_mut(), iter) {
            *arr_elt = elt;
            cnt += 1;
        }
        f(&self.0[..cnt])
    }
}

#[derive(Debug)]
struct EmptyCellsByDomainSize {
    len: [u8; GridValue::COUNT + 1],
    elts: [[GridIdx; GridIdx::COUNT]; GridValue::COUNT + 1],
}

impl Default for EmptyCellsByDomainSize {
    fn default() -> Self {
        Self {
            len: [0; GridValue::COUNT + 1],
            elts: [[GridIdx::default(); GridIdx::COUNT]; GridValue::COUNT + 1],
        }
    }
}

impl EmptyCellsByDomainSize {
    fn clear(&mut self) {
        self.len.fill(0);
    }

    fn insert<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (GridIdx, u8)>,
    {
        iter.for_each(|(idx, domain_size)| {
            let domain_size = domain_size as usize;
            self.elts[domain_size][self.len[domain_size] as usize] = idx;
            self.len[domain_size] += 1
        })
    }

    fn iter(&self) -> impl Iterator<Item = &GridIdx> + '_ {
        zip(self.len.iter(), self.elts.iter())
            .flat_map(|(len, elts)| elts[..(*len as usize)].iter())
    }

    fn maybe_ok_or_infeasible(&self) -> Option<bool> {
        let (zero_sized_len, non_zero_sized_lens) = self.len[..].split_first().unwrap();
        if *zero_sized_len != 0 {
            Some(false)
        } else if non_zero_sized_lens.iter().all(|len| *len == 0) {
            Some(true)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
struct StackFrame {
    empty_cells: EmptyCellsByDomainSize,
    domain: Domain,
}

impl StackFrame {
    fn clear(&mut self) {
        self.empty_cells.clear();
    }
}

const SOLVER_RECURSIVE_DEPTH: usize = GridIdx::COUNT + 1;

#[derive(Debug)]
struct Stack([StackFrame; SOLVER_RECURSIVE_DEPTH]);

impl Default for Stack {
    fn default() -> Self {
        Self(std::array::from_fn(|_| StackFrame::default()))
    }
}

struct StackTail<'a>(&'a mut [StackFrame]);

impl<'a> From<&'a mut [StackFrame]> for StackTail<'a> {
    fn from(slice: &'a mut [StackFrame]) -> Self {
        Self(slice)
    }
}

impl<'a> From<&'a mut Stack> for StackTail<'a> {
    fn from(stack: &'a mut Stack) -> Self {
        Self(&mut stack.0[..])
    }
}

impl<'caller> StackTail<'caller> {
    fn with<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut StackFrame, &mut StackTail<'_>) -> R,
    {
        let (frame, tail) = self.0.split_first_mut().unwrap();
        frame.clear();
        f(frame, &mut tail.into())
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

    fn with<I, G, F>(
        &mut self,
        iter: I,
        grid: &mut G,
        constraints: &mut Constraints,
        f: F,
    ) -> Result<usize, SolverError>
    where
        I: Iterator<Item = (GridIdx, GridValue)>,
        G: GridMut,
        F: FnOnce(&mut G, &mut Constraints, &mut DiffTail<'_>) -> Result<usize, SolverError>,
    {
        let cnt = self.push(iter);
        let (head, tail) = self.0.split_at_mut(cnt);
        grid.set_from_iter(head.iter().copied());
        constraints.set_many(head.iter().copied());
        let len = match f(grid, constraints, &mut tail.into()) {
            ok @ Ok(_) => ok,
            err @ Err(_) => {
                constraints.unset_many(head.iter().copied());
                grid.unset_from_iter(head.iter().map(|(x, _)| x).copied());
                err
            }
        }?;
        Ok(len + cnt)
    }
}

#[derive(Debug)]
struct RateLimitedCancellationFlag<'a, const RATE: u64, C>
where
    C: CancellationFlag,
{
    count: u64,
    cancellation_flag: &'a C,
}

impl<'a, const RATE: u64, C> RateLimitedCancellationFlag<'a, RATE, C>
where
    C: CancellationFlag,
{
    fn new(cancellation_flag: &'a C) -> Self {
        Self {
            count: 0,
            cancellation_flag,
        }
    }

    fn cancelled(&mut self) -> bool {
        self.count += 1;
        self.count % RATE == 0 && self.cancellation_flag.cancelled()
    }

    fn never_checked(&self) -> bool {
        self.count == 0
    }
}

#[derive(Debug, Default)]
struct SolverState {
    stack: Stack,
    diff: Diff,
    grid: ArrGridRowMajor,
    constraints: Constraints,
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

fn solve<const RATE: u64, C, G>(
    cancellation_flag: &mut RateLimitedCancellationFlag<'_, RATE, C>,
    frame: &mut StackFrame,
    grid: &mut G,
    constraints: &mut Constraints,
    stack: &mut StackTail<'_>,
    diff: &mut DiffTail<'_>,
) -> Result<usize, SolverError>
where
    C: CancellationFlag,
    G: GridMut,
{
    frame.empty_cells.insert(
        grid.iter_unset()
            .map(|idx| (idx, constraints.domain_size(idx))),
    );

    if let Some(ok_or_infeasible) = frame.empty_cells.maybe_ok_or_infeasible() {
        return if ok_or_infeasible {
            Ok(0)
        } else {
            Err(SolverError::Infeasible)
        };
    } else if cancellation_flag.cancelled() {
        return Err(SolverError::Cancelled);
    }

    frame
        .empty_cells
        .iter()
        .map(|idx| {
            frame.domain.with(constraints.domain(*idx), |domain| {
                domain
                    .iter()
                    .map(|value| {
                        diff.with(
                            once((*idx, *value)),
                            grid,
                            constraints,
                            |grid, constraints, diff| {
                                stack.with(|frame, stack| {
                                    solve(cancellation_flag, frame, grid, constraints, stack, diff)
                                })
                            },
                        )
                    })
                    .find_map(SolverError::ok_or_cancelled)
                    .ok_or(SolverError::Infeasible)?
            })
        })
        .find_map(SolverError::ok_or_cancelled)
        .ok_or(SolverError::Infeasible)?
}

#[derive(Debug, Default)]
pub struct GreedySolver;

impl GreedySolver {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Solver for GreedySolver {
    fn solve<C, T, U>(&self, cancellation_flag: &C, grid: &T) -> Result<U, SolverError>
    where
        C: CancellationFlag,
        T: Grid + ?Sized,
        U: FromIterator<GridDiff>,
    {
        let mut cancellation_flag: RateLimitedCancellationFlag<'_, { 1u64 << 10 }, _> =
            RateLimitedCancellationFlag::new(cancellation_flag);
        let mut mem = Box::new(SolverState::of_grid(grid));
        let len = StackTail::from(&mut mem.stack)
            .with(|frame, stack| {
                DiffTail::from(&mut mem.diff).with(
                    empty(),
                    &mut mem.grid,
                    &mut mem.constraints,
                    |grid, constraints, diff| {
                        solve(
                            &mut cancellation_flag,
                            frame,
                            grid,
                            constraints,
                            stack,
                            diff,
                        )
                    },
                )
            })
            .map_err(|err| match err {
                err @ (SolverError::Cancelled | SolverError::ConstraintsViolated) => err,
                err @ SolverError::Infeasible => {
                    if cancellation_flag.never_checked() {
                        SolverError::ConstraintsViolated
                    } else {
                        err
                    }
                }
            })?;
        Ok(mem.diff.iter(len).collect::<U>())
    }
}

#[cfg(test)]
mod greedy_solver_test {
    use super::{GreedySolver, Solver, SolverError};
    use crate::cancellation_flag::{AlreadyCancelled, NeverCancelled};
    use crate::format::{read_from_string, write_string, RowMajorAscii};
    use crate::grid::{ArrGridRowMajor, GridMutWithDefault};

    #[test]
    fn test_empty() {
        let expected = r#"
123456789
456789123
789123456
231674895
875912364
694538217
317265948
542897631
968341572"#
            .trim();
        let given = ArrGridRowMajor::new();
        let complete = write_string(
            &RowMajorAscii::default(),
            &ArrGridRowMajor::with_diff(
                &given,
                GreedySolver::new()
                    .solve::<_, _, Vec<_>>(&NeverCancelled::new(), &given)
                    .unwrap()
                    .into_iter(),
            ),
        );
        assert_eq!(&expected, &complete);
    }

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
                    .solve::<_, _, Vec<_>>(&NeverCancelled::new(), &given)
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
        let solution = GreedySolver::new().solve::<_, _, Vec<_>>(&AlreadyCancelled::new(), &given);
        assert_eq!(solution, Err(SolverError::Cancelled));
    }
}
