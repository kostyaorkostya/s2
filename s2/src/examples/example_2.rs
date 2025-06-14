use s2::cancellation_token::NeverCancelled;
use s2::format::{read_from_string, RowMajorAscii};
use s2::grid::{ArrGridRowMajor, GridMutWithDefault};
use s2::solver::{GreedySolver, Solver};
use s2::status::eval_status;

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
    let grid = create_grid();
    println!("{grid:?}");
    println!("{:?}", eval_status(&grid));
    let complete = ArrGridRowMajor::with_diff(
        &grid,
        GreedySolver::new()
            .solve::<_, _, Vec<_>>(&NeverCancelled::new(), &grid)
            .unwrap()
            .into_iter(),
    );
    println!("{:?}", eval_status(&complete));
    println!("{complete:?}");
}
