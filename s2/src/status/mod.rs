use crate::grid::{GridIdx, GridValue, IIdx, JIdx};
use itertools::Itertools;
use std::iter::{repeat, zip};
use std::ops::{Index, IndexMut};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::Display as DisplayMacros;

#[derive(Debug, Default, DisplayMacros, Eq, PartialEq)]
pub enum SudokuStatus {
    #[default]
    Incomplete,
    Complete,
}

#[derive(Debug, Default)]
pub struct SudokuStatusError;

impl From<SudokuStatus> for bool {
    fn from(value: SudokuStatus) -> Self {
        match value {
            SudokuStatus::Incomplete => false,
            SudokuStatus::Complete => true,
        }
    }
}

impl From<bool> for SudokuStatus {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Incomplete,
            true => Self::Complete,
        }
    }
}

#[derive(Debug, Default)]
struct Counter([u8; GridValue::COUNT]);

impl Index<GridValue> for Counter {
    type Output = u8;

    fn index(&self, grid_value: GridValue) -> &Self::Output {
        &self.0[usize::from(grid_value)]
    }
}

impl IndexMut<GridValue> for Counter {
    fn index_mut(&mut self, grid_value: GridValue) -> &mut Self::Output {
        &mut self.0[usize::from(grid_value)]
    }
}

impl std::iter::FromIterator<Option<GridValue>> for Counter {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<GridValue>>,
    {
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

    fn eval_status(&self) -> Result<SudokuStatus, SudokuStatusError> {
        let mut complete = true;
        for cnt in self.0 {
            match cnt {
                0 => complete = false,
                1 => (),
                _ => return Err(SudokuStatusError),
            }
        }
        Ok(complete.into())
    }
}

pub fn eval_status<T>(grid: &T) -> Result<SudokuStatus, SudokuStatusError>
where
    T: Index<GridIdx, Output = Option<GridValue>>,
{
    let rows = IIdx::iter()
        .map(|i| {
            zip(repeat(i), JIdx::iter())
                .map(|idx| grid[idx.into()])
                .collect::<Counter>()
                .eval_status()
                .map(Into::into)
        })
        .try_fold(true, |acc, x| x.map(|x| acc && x))?;
    let cols = JIdx::iter()
        .map(|j| {
            zip(IIdx::iter(), repeat(j))
                .map(|idx| grid[idx.into()])
                .collect::<Counter>()
                .eval_status()
                .map(Into::into)
        })
        .try_fold(true, |acc, x| x.map(|x| acc && x))?;
    let sub3x3s = (0..3)
        .cartesian_product(0..3)
        .map(|(i_subgrid, j_subgrid)| {
            (0..3)
                .cartesian_product(0..3)
                .map(|(i_in_subgrid, j_in_subgrid)| {
                    (
                        IIdx::try_from(i_subgrid * 3 + i_in_subgrid).unwrap(),
                        JIdx::try_from(j_subgrid * 3 + j_in_subgrid).unwrap(),
                    )
                })
                .map(|idx| grid[idx.into()])
                .collect::<Counter>()
                .eval_status()
                .map(Into::into)
        })
        .try_fold(true, |acc, x| x.map(|x| acc && x))?;
    Ok((rows && cols && sub3x3s).into())
}
