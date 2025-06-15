use s2::cancellation_flag::Atomic;
use s2::format::{read_from_string, RowMajorAscii};
use s2::grid::{ArrGridRowMajor, GridMutWithDefault};
use s2::solver::{GreedySolver, Solver};
use s2::status::eval_status;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn create_grid() -> ArrGridRowMajor {
    let grid = r#"
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
    read_from_string(&RowMajorAscii::default(), grid).unwrap()
}

fn main() {
    let cancel = Arc::new(Atomic::new());
    let cancellation_flag = cancel.clone();
    let solve = thread::spawn(move || {
        let cancellation_flag = cancellation_flag.clone();
        let grid = create_grid();
        println!("{grid:?}");
        println!("{:?}", eval_status(&grid));
        match GreedySolver::new().solve::<_, _, Vec<_>>(cancellation_flag.as_ref(), &grid) {
            Err(err) => println!("{err:?}"),
            Ok(diff) => {
                let grid = ArrGridRowMajor::with_diff(&grid, diff.into_iter());
                println!("{:?}", eval_status(&grid));
                println!("{grid:?}")
            }
        }
    });
    // Before introduction of naked singles this example used to timeout.
    thread::sleep(Duration::from_secs(30));
    cancel.cancel();
    solve.join().unwrap();
}
