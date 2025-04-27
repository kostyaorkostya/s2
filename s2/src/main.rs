mod s2;

use s2::*;

fn create_test_grid_1() -> PlainGrid {
    let mut grid = PlainGrid::new();
    grid[(IIdx::I1, JIdx::J3)] = Some(GridValue::V3);
    grid
}

fn main() {
    let grid = create_test_grid_1();
    println!("{}", grid);
    let status = eval_status(&grid);
    println!("{:?}", status);
}
