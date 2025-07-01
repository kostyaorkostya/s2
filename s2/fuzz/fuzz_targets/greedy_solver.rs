#![no_main]
//noinspection SpellCheckingInspection
#[macro_use]
extern crate libfuzzer_sys;
extern crate s2;
use s2::cancellation_flag::Atomic;
use s2::format::{read_from_string, RowMajorAscii};
use s2::grid::{ArrGridRowMajor, Grid, GridMutWithDefault};
use s2::solver::{GreedySolver, Solver, SolverError};
use s2::status::{eval_status, SudokuStatus};
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

fuzz_target!(|data: &[u8]| {
    if let Ok(grid) = str::from_utf8(data) && let Ok(grid) =
            read_from_string::<_, ArrGridRowMajor>(&RowMajorAscii::default(), grid.trim())
            && let Ok(status) = eval_status(&grid) {
                match status {
                    SudokuStatus::Complete => (),
                    SudokuStatus::Incomplete => {
                        match solve_with_timeout(&grid, Duration::from_secs(5)) {
                            Err(SolverError::Cancelled) => panic!("timed out\n{:?}", grid),
                            Err(SolverError::Infeasible) => (),
                            Err(SolverError::ConstraintsViolated) => panic!("unexpected"),
                            Ok(_) => (),
                        }
                    }
                }
            }
});
