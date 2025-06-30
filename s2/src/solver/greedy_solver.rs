use super::{Solver, SolverError};
use crate::cancellation_flag::CancellationFlag;
use crate::grid;
use crate::grid::{
    ArrGridRowMajor, Grid, GridDiff, GridIdx, GridMut, GridMutWithDefault, GridValue,
};
use crate::permutator::Permutator;
use crate::util::{BoolMatrix9x9, Domain, SliceGroupByIterator};
use std::array;
use std::iter::{empty, once, zip};
use strum::EnumCount;

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

    fn from_grid<T>(grid: &T) -> Self
    where
        T: Grid + ?Sized,
    {
        let mut t = Self::new();
        grid.iter_set().for_each(|(idx, value)| t.set(idx, value));
        t
    }

    fn constraint_indices(idx: GridIdx) -> (u8, u8, u8) {
        (idx.i.into(), idx.j.into(), idx.box_() as u8)
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

    fn domain(&self, idx: GridIdx) -> Domain {
        let (i, j, box_) = Self::constraint_indices(idx);
        (self.rows.row(i) | self.cols.row(j) | self.boxes.row(box_)).into()
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

    fn init<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (GridIdx, u8)>,
    {
        self.clear();
        iter.for_each(|(idx, domain_size)| {
            let domain_size = domain_size as usize;
            let len: &mut u8 = &mut self.len[domain_size];
            self.elts[domain_size][*len as usize] = idx;
            *len += 1;
        })
    }

    fn iter(&self) -> impl Iterator<Item = &GridIdx> + '_ {
        zip(self.len.iter(), self.elts.iter())
            .flat_map(|(len, elts)| elts[..(*len as usize)].iter())
    }
}

#[derive(Debug, Default)]
struct GroupedByUnit {
    rows: [(u8, [(Domain, GridIdx); grid::DIM]); grid::DIM],
    cols: [(u8, [(Domain, GridIdx); grid::DIM]); grid::DIM],
    boxes: [(u8, [(Domain, GridIdx); grid::DIM]); grid::DIM],
}

impl GroupedByUnit {
    fn clear(&mut self) {
        self.rows[..].iter_mut().for_each(|(len, _)| *len = 0);
        self.cols[..].iter_mut().for_each(|(len, _)| *len = 0);
        self.boxes[..].iter_mut().for_each(|(len, _)| *len = 0);
    }

    fn init<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (GridIdx, Domain)>,
    {
        self.clear();
        iter.for_each(|(idx, domain)| {
            let row: &mut (u8, [(Domain, GridIdx); grid::DIM]) = &mut self.rows[usize::from(idx.i)];
            row.1[row.0 as usize] = (domain, idx);
            row.0 += 1;
            let col: &mut (u8, [(Domain, GridIdx); grid::DIM]) = &mut self.cols[usize::from(idx.j)];
            col.1[col.0 as usize] = (domain, idx);
            col.0 += 1;
            let box_: &mut (u8, [(Domain, GridIdx); grid::DIM]) = &mut self.boxes[idx.box_()];
            box_.1[box_.0 as usize] = (domain, idx);
            box_.0 += 1;
        });
        // TODO(kostya): random shuffle within the set that has the same domain within unit.
        self.rows
            .iter_mut()
            .chain(self.cols.iter_mut())
            .chain(self.boxes.iter_mut())
            .for_each(|unit| unit.1[..(unit.0 as usize)].sort())
    }

    fn iter_equal_domains(&self) -> impl Iterator<Item = &[(Domain, GridIdx)]> {
        self.rows
            .iter()
            .chain(self.cols.iter())
            .chain(self.cols.iter())
            .flat_map(|unit| {
                SliceGroupByIterator::<(Domain, GridIdx), _>::new(
                    &unit.1[..(unit.0 as usize)],
                    |lhs, rhs| lhs.0 == rhs.0,
                )
            })
    }
}

#[derive(Debug, Default)]
struct StackFrame {
    empty_cells: EmptyCellsByDomainSize,
    grouped_by_unit: GroupedByUnit,
    permutator: Permutator<5, GridValue>,
}

impl StackFrame {
    fn clear(&mut self) {
        self.empty_cells.clear();
        self.grouped_by_unit.clear();
    }
}

const SOLVER_RECURSIVE_DEPTH: usize = GridIdx::COUNT + 1;

#[derive(Debug)]
struct Stack([StackFrame; SOLVER_RECURSIVE_DEPTH]);

impl Default for Stack {
    fn default() -> Self {
        Self(array::from_fn(|_| StackFrame::default()))
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

impl<'caller> DiffTail<'caller> {
    fn push<I>(&mut self, iter: I) -> usize
    where
        I: Iterator<Item = (GridIdx, GridValue)>,
    {
        let mut cnt = 0;
        for (mut_elt, elt) in zip(self.0.iter_mut(), iter) {
            *mut_elt = elt;
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

    fn count(&self) -> u64 {
        self.count
    }

    fn never_checked(&self) -> bool {
        self.count() == 0
    }
}

#[derive(Debug, Default)]
struct State {
    stack: Stack,
    diff: Diff,
    grid: ArrGridRowMajor,
    constraints: Constraints,
}

impl State {
    fn from_grid<T>(grid: &T) -> Self
    where
        T: Grid + ?Sized,
    {
        Self {
            grid: ArrGridRowMajor::copy_of(grid),
            constraints: Constraints::from_grid(grid),
            ..Default::default()
        }
    }
}

fn solve_inner<const RATE: u64, I, C, G>(
    diff: I,
    cancellation_flag: &mut RateLimitedCancellationFlag<'_, RATE, C>,
    grid: &mut G,
    constraints: &mut Constraints,
    stack: &mut StackTail<'_>,
    diff_tail: &mut DiffTail<'_>,
) -> Result<usize, SolverError>
where
    I: Iterator<Item = (GridIdx, GridValue)>,
    C: CancellationFlag,
    G: GridMut,
{
    diff_tail.with(diff, grid, constraints, |grid, constraints, diff| {
        stack.with(|frame, stack| solve(cancellation_flag, frame, grid, constraints, stack, diff))
    })
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
    if cancellation_flag.count() % (1u64 << 14) == 0 {
        // TODO(kostya): delete
        println!("=====DEBUG===== step={}", cancellation_flag.count());
        println!(
            "{}",
            crate::format::write_string(&crate::format::RowMajorAscii::default(), grid)
        );
        println!("=====DEBUG=====");
    }

    frame
        .grouped_by_unit
        .init(grid.iter_unset().map(|idx| (idx, constraints.domain(idx))));

    match frame
        .grouped_by_unit
        .iter_equal_domains()
        .map(|with_equal_domain| {
            let domain_size = with_equal_domain.first().unwrap().0.size();
            domain_size < (with_equal_domain.len() as u8)
        })
        .enumerate()
        .fold(
            (0, false),
            |(_, infeasible), (cnt, domain_size_less_than_elt_cnt)| {
                (cnt, infeasible || domain_size_less_than_elt_cnt)
            },
        ) {
        (0, _) => return Ok(0),
        (_, true) => return Err(SolverError::Infeasible),
        _ => (),
    }

    // Check if cancelled. This must happen __after__ the check for completeness or infisibility,
    // as calleer relies on it and is using `cancellation_flag` counter to tell if the grid had
    // constraints violation from the start.
    if cancellation_flag.cancelled() {
        return Err(SolverError::Cancelled);
    }

    // Look for naked sets.
    match (1u8..=5u8)
        .flat_map(|naked_set_size| {
            frame
                .grouped_by_unit
                .iter_equal_domains()
                .filter(move |with_equal_domain| {
                    with_equal_domain.len() == naked_set_size as usize
                        && with_equal_domain.first().unwrap().0.size() == naked_set_size
                })
        })
        .next()
        .map(|with_equal_domain| {
            frame.permutator.try_find(
                with_equal_domain.first().unwrap().0.iter(),
                |values| {
                    solve_inner(
                        zip(
                            with_equal_domain.iter().map(|(_, x)| x).copied(),
                            values.iter().copied(),
                        ),
                        cancellation_flag,
                        grid,
                        constraints,
                        stack,
                        diff,
                    )
                },
                SolverError::is_cancelled,
            )
        }) {
        None => (),
        Some(res) => return res,
    };

    // TODO(kostya): look for hidden sets

    frame.empty_cells.init(
        grid.iter_unset()
            .map(|idx| (idx, constraints.domain(idx).size())),
    );

    frame
        .empty_cells
        .iter()
        .map(|idx| {
            constraints
                .domain(*idx)
                .iter()
                .map(|value| {
                    solve_inner(
                        once((*idx, value)),
                        cancellation_flag,
                        grid,
                        constraints,
                        stack,
                        diff,
                    )
                })
                .find_map(SolverError::ok_or_cancelled)
                .ok_or(SolverError::Infeasible)?
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
        let mut mem = Box::new(State::from_grid(grid));
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
mod test {
    use super::{GreedySolver, Solver, SolverError};
    use crate::cancellation_flag::{Atomic, NeverCancelled};
    use crate::format::{read_from_string, write_string, RowMajorAscii};
    use crate::grid::{ArrGridRowMajor, Grid, GridMutWithDefault};
    use crate::status::{eval_status, SudokuStatus};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn solve_with_timeout<T>(grid: &T, timeout: Duration) -> Result<ArrGridRowMajor, SolverError>
    where
        T: Grid,
    {
        let grid = ArrGridRowMajor::copy_of(grid);
        let cancel = Arc::new(Atomic::new());
        let cancellation_flag = cancel.clone();
        let solve = thread::spawn(move || {
            let cancellation_flag = cancellation_flag.clone();
            GreedySolver::new().solve::<_, _, Vec<_>>(cancellation_flag.as_ref(), &grid)
        });
        thread::sleep(timeout);
        cancel.cancel();
        let diff = solve.join().unwrap()?;
        let complete = ArrGridRowMajor::with_diff(&grid, diff.into_iter());
        assert_eq!(
            &SudokuStatus::Complete,
            &eval_status(&complete).expect(&format!("{:?}\n{:?}", grid, &complete))
        );
        Ok(complete)
    }

    #[test]
    fn test_empty() {
        let expected = r#"
123456789
456789123
789123456
261594378
374812965
598637214
612345897
835971642
947268531"#
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
        let complete = solve_with_timeout(&given, Duration::from_secs(1))
            .map(|grid| write_string(&RowMajorAscii::default(), &grid));
        assert_eq!(Err(SolverError::Infeasible), complete);
    }

    #[test]
    fn test_fuzzing_crash_1() {
        // Used to timeout before introduction of locked sets.
        let given = r#"
_________
_________
_________
_________
_________
_________
_________
_________
8________
"#
        .trim();
        let expected = r#"
145236789
267589134
389147256
431852967
572693841
698714325
713468592
924375618
856921473
"#
        .trim();
        let given: ArrGridRowMajor = read_from_string(&RowMajorAscii::default(), given).unwrap();
        let complete = solve_with_timeout(&given, Duration::from_secs(1))
            .map(|grid| write_string(&RowMajorAscii::default(), &grid));
        assert_eq!(&expected, &complete.unwrap());
    }

    #[test]
    fn test_fuzzing_crash_2() {
        // TODO(kostya): times out, but should be infeasible
        let given = r#"
3417_6___
____958__
_______7_
__916____
754______
___958___
_958__2__
____7_6__
______958
"#
        .trim();
        let expected = r#"
_________
_________
_________
_________
_________
_________
_________
_________
_________
"#
        .trim();
        let given: ArrGridRowMajor = read_from_string(&RowMajorAscii::default(), given).unwrap();
        let complete = solve_with_timeout(&given, Duration::from_secs(1))
            .map(|grid| write_string(&RowMajorAscii::default(), &grid));
        assert_eq!(&expected, &complete.unwrap());
    }
}
