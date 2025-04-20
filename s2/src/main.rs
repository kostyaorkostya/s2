mod s2;

use s2::*;

fn create_test_grid_1() -> PlainGrid {
    let mut grid = PlainGrid::new();
    let idx: Idx = (IIdx::I1, JIdx::J3);
    grid[idx] = GridValue::V3;
    grid
}

fn main() {
    let grid = create_test_grid_1();
    println!("{}", grid);
    let is_complate = is_complete(grid);
    println!("{}", is_complete);
}
