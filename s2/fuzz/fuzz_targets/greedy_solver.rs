#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate s2;
use s2::cancellation_flag::NeverCancelled;
use s2::format::{read_from_string, RowMajorAscii};
use s2::grid::{ArrGridRowMajor, GridMutWithDefault};
use s2::solver::{GreedySolver, Solver};
use s2::status::{eval_status, SudokuStatus};

fuzz_target!(|data: &[u8]| {
    if let Ok(grid) = str::from_utf8(data) {
        if let Ok(grid) =
            read_from_string::<_, ArrGridRowMajor>(&RowMajorAscii::default(), &grid.trim())
        {
            if let Ok(status) = eval_status(&grid) {
                match status {
                    SudokuStatus::Complete => (),
                    SudokuStatus::Incomplete => {
                        match GreedySolver::new()
                            .solve::<_, _, Vec<_>>(&NeverCancelled::new(), &grid)
                        {
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
