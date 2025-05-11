mod s2;

use s2::*;

fn create_grid_1() -> PlainGrid {
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
    let grid = create_grid_1();
    println!("{:?}", grid);
    println!("{:?}", eval_status(&grid));
    let complete = copy_and_apply::<_, PlainGrid, _>(
        &grid,
        GreedySolver::new().solve::<_, Vec<_>>(&grid).into_iter(),
    );
    println!("{:?}", eval_status(&complete));
    println!("{:?}", complete);
}
