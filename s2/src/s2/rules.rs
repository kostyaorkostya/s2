use super::grid::*;
use std::ops::{Index, IndexMut};
use strum::{EnumCount, IntoEnumIterator};

#[derive(Debug, Default)]
struct Used([bool; GridValue::COUNT]);

impl Index<GridValue> for Used {
    type Output = bool;

    fn index(&self, grid_value: GridValue) -> &Self::Output {
        let idx: usize = grid_value.into();
        &self.0[idx]
    }
}

impl IndexMut<GridValue> for Used {
    fn index_mut(&mut self, grid_value: GridValue) -> &mut Self::Output {
        let idx: usize = grid_value.into();
        &mut self.0[idx]
    }
}

impl Used {
    fn new() -> Self {
        Default::default()
    }

    fn is_complete(&self) -> bool {
        for x in self.0 {
            if !x {
                return false;
            }
        }
        true
    }
}

pub fn is_complete<T>(grid: &T) -> bool
where
    T: Index<Idx, Output = Option<GridValue>>,
{
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
            .for_each(|x| {
                used[x.clone()] = true;
            });
        if !used.is_complete() {
            return false;
        }
    }

    for j in JIdx::iter() {
        let mut used = Used::new();
        IIdx::iter()
            .map(|i| grid[(i, j)].unwrap_or_default())
            .for_each(|x| used[x.clone()] = true);
        if !used.is_complete() {
            return false;
        }
    }

    for i_subgrid in 0..3 {
        for j_subgrid in 0..3 {
            let mut used = Used::new();
            for i_in_subgrid in 0..3 {
                for j_in_subgrid in 0..3 {
                    let i: IIdx = (i_subgrid * 3 + i_in_subgrid).try_into().unwrap();
                    let j: JIdx = (j_subgrid * 3 + j_in_subgrid).try_into().unwrap();
                    used[grid[(i, j)].unwrap_or_default()] = true;
                }
            }
            if !used.is_complete() {
                return false;
            }
        }
    }

    return true;
}
