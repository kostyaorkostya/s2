use s2::format::{read_from_string, RowMajorAscii};
use s2::grid::{copy_and_apply, PlainGrid};
use s2::solver::{GreedySolver, Solver};
use s2::status::eval_status;

fn create_grid() -> PlainGrid {
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
    let grid = create_grid();
    println!("{grid:?}");
    println!("{:?}", eval_status(&grid));
    let complete = copy_and_apply::<_, PlainGrid, _>(
        &grid,
        GreedySolver::new()
            .solve::<_, Vec<_>>(&grid)
            .unwrap()
            .into_iter(),
    );
    println!("{:?}", eval_status(&complete));
    println!("{complete:?}");
}
