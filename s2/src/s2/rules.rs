use super::grid::*;
use std::ops::{Index, IndexMut};
use strum::EnumCount;

#[derive(Debug, Default)]
struct Used([bool; GridValue::COUNT]);

impl Index<GridValue> for Used {
    type Output = bool;

    fn index(&self, grid_value: GridValue) -> &Self::Output {
        &self.0[grid_value]
    }
}

impl IndexMut<GridValue> for Used {
    fn index_mut(&mut self, grid_value: GridValue) -> &mut Self::Output {
        &mut self.0[grid_value]
    }
}

impl Used {
    fn is_complete(&self) -> bool {
        for x in self.0 {
            if !x {
                return false;
            }
        }
        true
    }
}

pub fn is_complete(&grid: impl Grid) -> bool {
    for i in IIdx::iter() {
        for j in JIdx::iter() {
            match grid[(i, j)] {
                None => return false,
                Some(_) => (),
            }
        }
    }

    for i in IIdx::iter() {
        let mut used = Used::new();
        JIdx::iter()
            .map(|j| grid[(i, j)].unwrap_or_default())
            .inspect(|x| used[x] = true);
        if !used.is_complete() {
            return false;
        }
    }

    for j in JIdx::iter() {
        let mut used = Used::new();
        IIdx::iter()
            .map(|i| grid[(i, j)].unwrap_or_default())
            .inspect(|x| used[x] = true);
        if !used.is_complete() {
            return false;
        }
    }

    for i_subgrid in 0..3 {
        for j_subgrid in 0..3 {
            let mut used = Used::new();
            for i_in_subgrid in 0..3 {
                for j_in_subgrid in 0..3 {
                    let i = IIdx::from(i_subgrid * 3 + i_in_subgrid);
                    let j = JIdx::from(j_subgrid * 3 + j_in_subgrid);
                    used[grid[(i, j)]] = true;
                }
            }
            if !used.is_complete() {
                return false;
            }
        }
    }

    return true;
}
