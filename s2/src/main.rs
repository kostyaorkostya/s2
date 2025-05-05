mod s2;

use s2::*;

fn create_grid_1() -> PlainGrid {
    let mut grid = PlainGrid::new();
    grid[(IIdx::I0, JIdx::J0)] = Some(GridValue::V5);
    grid[(IIdx::I0, JIdx::J1)] = Some(GridValue::V3);
    grid[(IIdx::I0, JIdx::J4)] = Some(GridValue::V7);
    grid[(IIdx::I1, JIdx::J0)] = Some(GridValue::V6);
    grid[(IIdx::I1, JIdx::J3)] = Some(GridValue::V1);
    grid[(IIdx::I1, JIdx::J4)] = Some(GridValue::V9);
    grid[(IIdx::I1, JIdx::J5)] = Some(GridValue::V5);
    grid[(IIdx::I2, JIdx::J7)] = Some(GridValue::V6);
    grid[(IIdx::I2, JIdx::J2)] = Some(GridValue::V8);
    grid[(IIdx::I2, JIdx::J1)] = Some(GridValue::V9);
    grid[(IIdx::I3, JIdx::J0)] = Some(GridValue::V8);
    grid[(IIdx::I3, JIdx::J4)] = Some(GridValue::V6);
    grid[(IIdx::I3, JIdx::J8)] = Some(GridValue::V3);
    grid[(IIdx::I4, JIdx::J0)] = Some(GridValue::V4);
    grid[(IIdx::I4, JIdx::J3)] = Some(GridValue::V8);
    grid[(IIdx::I4, JIdx::J5)] = Some(GridValue::V3);
    grid[(IIdx::I4, JIdx::J8)] = Some(GridValue::V1);
    grid[(IIdx::I5, JIdx::J0)] = Some(GridValue::V7);
    grid[(IIdx::I5, JIdx::J4)] = Some(GridValue::V2);
    grid[(IIdx::I5, JIdx::J8)] = Some(GridValue::V6);
    grid[(IIdx::I6, JIdx::J1)] = Some(GridValue::V6);
    grid[(IIdx::I6, JIdx::J6)] = Some(GridValue::V2);
    grid[(IIdx::I6, JIdx::J7)] = Some(GridValue::V8);
    grid[(IIdx::I7, JIdx::J3)] = Some(GridValue::V4);
    grid[(IIdx::I7, JIdx::J4)] = Some(GridValue::V1);
    grid[(IIdx::I7, JIdx::J5)] = Some(GridValue::V9);
    grid[(IIdx::I7, JIdx::J8)] = Some(GridValue::V5);
    grid[(IIdx::I8, JIdx::J4)] = Some(GridValue::V8);
    grid[(IIdx::I8, JIdx::J7)] = Some(GridValue::V7);
    grid[(IIdx::I8, JIdx::J8)] = Some(GridValue::V9);
    grid
}

fn main() {
    let grid = create_grid_1();
    println!("{}", grid);
    println!("{:?}", eval_status(&grid));
    let complete = copy_and_apply::<_, PlainGrid, _>(
        &grid,
        NaiveSolver::new().solve::<_, Vec<_>>(&grid).into_iter(),
    );
    println!("{:?}", eval_status(&complete));
    println!("{}", complete);
}
