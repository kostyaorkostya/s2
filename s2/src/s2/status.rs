use super::grid::*;
use itertools::Itertools;
use std::iter::{repeat, zip};
use std::ops::{Index, IndexMut};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::Display as DisplayMacros;

#[derive(Debug, Default, DisplayMacros)]
pub enum SudokuStatus {
    #[default]
    Incomplete,
    Complete,
}

impl From<bool> for SudokuStatus {
    fn from(value: bool) -> Self {
        if value {
            Self::Complete
        } else {
            Self::Incomplete
        }
    }
}

impl Into<bool> for SudokuStatus {
    fn into(self) -> bool {
        match self {
            Self::Incomplete => false,
            Self::Complete => true,
        }
    }
}

#[derive(Debug, Default)]
struct Counter([u8; GridValue::COUNT]);

impl Index<GridValue> for Counter {
    type Output = u8;

    fn index(&self, grid_value: GridValue) -> &Self::Output {
        &self.0[<super::grid::GridValue as Into<usize>>::into(grid_value)]
    }
}

impl IndexMut<GridValue> for Counter {
    fn index_mut(&mut self, grid_value: GridValue) -> &mut Self::Output {
        &mut self.0[<super::grid::GridValue as Into<usize>>::into(grid_value)]
    }
}

impl std::iter::FromIterator<Option<GridValue>> for Counter {
    fn from_iter<I: IntoIterator<Item = Option<GridValue>>>(iter: I) -> Self {
        let mut counter = Self::new();
        for grid_value in iter {
            match grid_value {
                None => (),
                Some(grid_value) => counter[grid_value] += 1,
            }
        }
        counter
    }
}

impl Counter {
    fn new() -> Self {
        Default::default()
    }

    fn eval_status(&self) -> Result<SudokuStatus, ()> {
        let mut complete = true;
        for cnt in self.0 {
            match cnt {
                0 => complete = false,
                1 => (),
                _ => return Err(()),
            }
        }
        Ok(complete.into())
    }
}

pub fn eval_status<T>(grid: &T) -> Result<SudokuStatus, ()>
where
    T: Index<GridIdx, Output = Option<GridValue>>,
{
    Ok((IIdx::iter()
        .map(|i| {
            zip(repeat(i), JIdx::iter())
                .map(|x| grid[x])
                .collect::<Counter>()
                .eval_status()
                .map(Into::into)
        })
        .try_fold(true, |acc, x| x.map(|x| acc && x))?
        && JIdx::iter()
            .map(|j| {
                zip(IIdx::iter(), repeat(j))
                    .map(|x| grid[x])
                    .collect::<Counter>()
                    .eval_status()
                    .map(Into::into)
            })
            .try_fold(true, |acc, x| x.map(|x| acc && x))?
        && (0..3)
            .cartesian_product(0..3)
            .map(|(i_subgrid, j_subgrid)| {
                (0..3)
                    .cartesian_product(0..3)
                    .map(|(i_in_subgrid, j_in_subgrid)| {
                        (
                            (i_subgrid * 3 + i_in_subgrid).try_into().unwrap(),
                            (j_subgrid * 3 + j_in_subgrid).try_into().unwrap(),
                        )
                    })
                    .map(|x| grid[x])
                    .collect::<Counter>()
                    .eval_status()
                    .map(Into::into)
            })
            .try_fold(true, |acc, x| x.map(|x| acc && x))?)
    .into())
}
