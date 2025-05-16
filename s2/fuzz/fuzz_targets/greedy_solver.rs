#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate s2;
use s2::format::{read_from_string, RowMajorAscii};
use s2::grid::{copy_and_apply, PlainGrid};
use s2::solver::{GreedySolver, Solver};
use s2::status::{eval_status, SudokuStatus};

fuzz_target!(|data: &[u8]| {
    if let Ok(grid) = std::str::from_utf8(data) {
        if let Ok(grid) = read_from_string::<_, PlainGrid>(&RowMajorAscii::default(), &grid.trim())
        {
            if let Ok(status) = eval_status(&grid) {
                match status {
                    SudokuStatus::Complete => (),
                    SudokuStatus::Incomplete => {
                        let complete = copy_and_apply::<_, PlainGrid, _>(
                            &grid,
                            GreedySolver::new().solve::<_, Vec<_>>(&grid).into_iter(),
                        );
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
});
