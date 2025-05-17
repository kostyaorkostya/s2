use s2::format::{read_from_string, RowMajorAscii};
use s2::grid::{copy_and_apply, PlainGrid};
use s2::solver::{GreedySolver, Solver};
use s2::status::eval_status;

fn create_grid() -> PlainGrid {
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
    println!("{:?}", grid);
    println!("{:?}", eval_status(&grid));
    let complete = copy_and_apply::<_, PlainGrid, _>(
        &grid,
        GreedySolver::new()
            .solve::<_, Vec<_>>(&grid)
            .unwrap()
            .into_iter(),
    );
    println!("{:?}", eval_status(&complete));
    println!("{:?}", complete);
}
