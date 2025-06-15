#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate s2;
use s2::cancellation_flag::Atomic;
use s2::format::{read_from_string, RowMajorAscii};
use s2::grid::{ArrGridRowMajor, Grid, GridDiff, GridMutWithDefault};
use s2::solver::{GreedySolver, Solver, SolverError};
use s2::status::{eval_status, SudokuStatus};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn solve_with_timeout<T>(grid: &T, timeout: Duration) -> Result<Vec<GridDiff>, SolverError>
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
    // Before introduction of naked singles this example used to timeout.
    thread::sleep(timeout);
    cancel.cancel();
    solve.join().unwrap()
}

fuzz_target!(|data: &[u8]| {
    if let Ok(grid) = str::from_utf8(data) {
        if let Ok(grid) =
            read_from_string::<_, ArrGridRowMajor>(&RowMajorAscii::default(), &grid.trim())
        {
            if let Ok(status) = eval_status(&grid) {
                match status {
                    SudokuStatus::Complete => (),
                    SudokuStatus::Incomplete => {
                        match solve_with_timeout(&grid, Duration::from_secs(5)) {
                            Err(SolverError::Cancelled) => panic!("timed out\n{:?}", grid),
                            Err(_) => (),
                            Ok(diff) => {
                                let complete = ArrGridRowMajor::with_diff(&grid, diff.into_iter());
                                match eval_status(&complete) {
                                    Err(_) | Ok(SudokuStatus::Incomplete) => {
                                        panic!("{:?}\n{:?}", grid, complete)
                                    }
                                    Ok(SudokuStatus::Complete) => (),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
});
