use super::{HiddenSets, Solver, SolverError};
use crate::cancellation_flag::CancellationFlag;
use crate::grid;
use crate::grid::{ArrGridRowMajor, CellIdx, Digit, Grid, GridDiff, GridMut, GridMutWithDefault};
use crate::permutator::Permutator;
use crate::util::{BoolMatrix9x9, Domain, SliceGroupByIterator};
use std::array;
use std::iter::{empty, once, zip};
use strum::EnumCount;

// TODO(kostya): delete
const DEBUG_ITER_STATE: bool = false;
const DEBUG_ITER_STATE_EACH: bool = false;
const DEBUG_RECURSION_DEPTH: bool = false;
const DEBUG_TOTAL_ITER_COUNT: bool = true;

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

    fn constraint_indices(idx: CellIdx) -> (u8, u8, u8) {
        (idx.row.into(), idx.col.into(), idx.box_() as u8)
    }

    fn set(&mut self, idx: CellIdx, value: Digit) {
        let (i, j, box_) = Self::constraint_indices(idx);
        let value: u8 = value.into();
        self.rows.set((i, value));
        self.cols.set((j, value));
        self.boxes.set((box_, value));
    }

    fn set_many<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (CellIdx, Digit)>,
    {
        for (idx, elt) in iter {
            self.set(idx, elt)
        }
    }

    fn unset(&mut self, idx: CellIdx, value: Digit) {
        let (i, j, box_) = Self::constraint_indices(idx);
        let value: u8 = value.into();
        self.rows.unset((i, value));
        self.cols.unset((j, value));
        self.boxes.unset((box_, value));
    }

    fn unset_many<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (CellIdx, Digit)>,
    {
        for (idx, elt) in iter {
            self.unset(idx, elt)
        }
    }

    fn domain(&self, idx: CellIdx) -> Domain {
        let (i, j, box_) = Self::constraint_indices(idx);
        (self.rows.row(i) | self.cols.row(j) | self.boxes.row(box_)).into()
    }
}

#[derive(Debug)]
struct EmptyCellsByDomainSize {
    len: [u8; Digit::COUNT + 1],
    elts: [[CellIdx; CellIdx::COUNT]; Digit::COUNT + 1],
}

impl Default for EmptyCellsByDomainSize {
    fn default() -> Self {
        Self {
            len: [0; Digit::COUNT + 1],
            elts: [[CellIdx::default(); CellIdx::COUNT]; Digit::COUNT + 1],
        }
    }
}

impl EmptyCellsByDomainSize {
    fn clear(&mut self) {
        self.len.fill(0);
    }

    fn init<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (CellIdx, u8)>,
    {
        self.clear();
        iter.for_each(|(idx, domain_size)| {
            let domain_size = domain_size as usize;
            let len: &mut u8 = &mut self.len[domain_size];
            self.elts[domain_size][*len as usize] = idx;
            *len += 1;
        })
    }

    fn iter(&self) -> impl Iterator<Item = &CellIdx> + '_ {
        zip(self.len.iter(), self.elts.iter())
            .flat_map(|(len, elts)| elts[..(*len as usize)].iter())
    }
}

#[derive(Debug, Default)]
struct GroupedByUnit {
    rows_lens: [u8; grid::DIM],
    rows: [[(Domain, CellIdx); grid::DIM]; grid::DIM],
    cols_lens: [u8; grid::DIM],
    cols: [[(Domain, CellIdx); grid::DIM]; grid::DIM],
    boxes_lens: [u8; grid::DIM],
    boxes: [[(Domain, CellIdx); grid::DIM]; grid::DIM],
}

impl GroupedByUnit {
    fn clear(&mut self) {
        self.rows_lens.fill(0);
        self.cols_lens.fill(0);
        self.boxes_lens.fill(0);
    }

    fn init<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (CellIdx, Domain)>,
    {
        self.clear();
        iter.for_each(|(idx, domain)| {
            let row: usize = idx.row.into();
            let col: usize = idx.col.into();
            let box_: usize = idx.box_();
            self.rows[row][self.rows_lens[row] as usize] = (domain, idx);
            self.cols[col][self.cols_lens[col] as usize] = (domain, idx);
            self.boxes[box_][self.boxes_lens[box_] as usize] = (domain, idx);
            self.rows_lens[row] += 1;
            self.cols_lens[col] += 1;
            self.boxes_lens[box_] += 1;
        });
        // TODO(kostya): random shuffle within the set that has the same domain within unit.
        zip(self.rows_lens.iter(), self.rows.iter_mut())
            .chain(zip(self.cols_lens.iter(), self.cols.iter_mut()))
            .chain(zip(self.boxes_lens.iter(), self.boxes.iter_mut()))
            .filter(|(len, _)| **len > 1)
            .for_each(|(len, unit)| unit[..(*len as usize)].sort_unstable())
    }

    fn iter_equal_domains(&self) -> impl Iterator<Item = &[(Domain, CellIdx)]> {
        zip(self.rows_lens.iter(), self.rows.iter())
            .chain(zip(self.cols_lens.iter(), self.cols.iter()))
            .chain(zip(self.boxes_lens.iter(), self.boxes.iter()))
            .filter(|(len, _)| **len > 0)
            .flat_map(|(len, unit)| {
                SliceGroupByIterator::<(Domain, CellIdx), _>::new(
                    &unit[..(*len as usize)],
                    |lhs, rhs| lhs.0 == rhs.0,
                )
            })
    }

    fn iter_units(&self) -> impl Iterator<Item = &[(Domain, CellIdx)]> {
        zip(self.rows_lens.iter(), self.rows.iter())
            .chain(zip(self.cols_lens.iter(), self.cols.iter()))
            .chain(zip(self.boxes_lens.iter(), self.boxes.iter()))
            .filter(|(len, _)| **len > 0)
            .map(|(len, unit)| &unit[..(*len as usize)])
    }
}

#[derive(Debug, Default)]
struct StackFrame {
    count: u64, // TODO(kostya): remove it
    empty_cells: EmptyCellsByDomainSize,
    grouped_by_unit: GroupedByUnit,
    permutator: Permutator<5, Digit>,
    hidden_sets: HiddenSets<CellIdx>,
}

impl StackFrame {
    fn clear(&mut self) {
        self.empty_cells.clear();
        self.grouped_by_unit.clear();
    }
}

const SOLVER_RECURSIVE_DEPTH: usize = CellIdx::COUNT + 1;

#[derive(Debug)]
struct Stack([StackFrame; SOLVER_RECURSIVE_DEPTH]);

impl Default for Stack {
    fn default() -> Self {
        Self(array::from_fn(|_| StackFrame::default()))
    }
}

impl Stack {
    fn iter(&self) -> impl Iterator<Item = &StackFrame> {
        self.0.iter()
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
struct Diff([(CellIdx, Digit); CellIdx::COUNT]);

impl Default for Diff {
    fn default() -> Self {
        Self([(CellIdx::default(), Digit::default()); CellIdx::COUNT])
    }
}

impl Diff {
    fn iter(&self, len: usize) -> impl Iterator<Item = GridDiff> {
        self.0[..len]
            .iter()
            .map(|(idx, value)| GridDiff::Set(*idx, *value))
    }
}

struct DiffTail<'a>(&'a mut [(CellIdx, Digit)]);

impl<'a> From<&'a mut [(CellIdx, Digit)]> for DiffTail<'a> {
    fn from(slice: &'a mut [(CellIdx, Digit)]) -> Self {
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
        I: Iterator<Item = (CellIdx, Digit)>,
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
        I: Iterator<Item = (CellIdx, Digit)>,
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
    I: Iterator<Item = (CellIdx, Digit)>,
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
    if DEBUG_RECURSION_DEPTH {
        frame.count += 1;
    }

    if DEBUG_ITER_STATE && (DEBUG_ITER_STATE_EACH || cancellation_flag.count() % (1u64 << 14) == 0)
    {
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

    // Check if cancelled. This must happen __after__ the check for completeness or infiasibility,
    // as caller relies on it and is using `cancellation_flag` counter to tell if the grid had
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
                |digits| {
                    solve_inner(
                        zip(
                            with_equal_domain.iter().map(|(_, x)| x).copied(),
                            digits.iter().copied(),
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
        Some(ret) => return ret,
    };

    match frame
        .grouped_by_unit
        .iter_units()
        .filter_map(|unit| {
            frame.hidden_sets.init(unit.iter().copied());
            (1u8..=5u8)
                .filter_map(|hidden_set_size| {
                    frame
                        .hidden_sets
                        .map_first(hidden_set_size, |domain, hidden_set| {
                            frame.permutator.try_find(
                                domain.iter(),
                                |digits| {
                                    solve_inner(
                                        zip(hidden_set.iter().copied(), digits.iter().copied()),
                                        cancellation_flag,
                                        grid,
                                        constraints,
                                        stack,
                                        diff,
                                    )
                                },
                                SolverError::is_cancelled,
                            )
                        })
                })
                .next()
        })
        .next()
    {
        None => (),
        Some(ret) => return ret,
    };

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
            });
        if DEBUG_TOTAL_ITER_COUNT {
            println!("Total iterations count: {:?}", cancellation_flag.count());
        }
        if DEBUG_RECURSION_DEPTH {
            println!("Recursion depth statistics:");
            mem.stack
                .iter()
                .enumerate()
                .for_each(|(depth, frame)| println!("{:?}: {:?}", depth, frame.count))
        }
        let len = len?;
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
            &eval_status(&complete)
                .unwrap_or_else(|err| panic!("{:?\n}{:?}\n{:?}", err, grid, &complete))
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
