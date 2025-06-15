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
        let complete = ArrGridRowMajor::with_diff(
            &grid,
            GreedySolver::new()
                .solve::<_, _, Vec<_>>(cancellation_flag.as_ref(), &grid)
                .unwrap()
                .into_iter(),
        );
        println!("{:?}", eval_status(&complete));
        println!("{complete:?}");
    });
    thread::sleep(Duration::from_secs(1));
    cancel.cancel();
    solve.join().unwrap();
}
