use super::{CellIdx, Digit};
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy)]
pub struct ArrGrid<const ROW_MAJOR: bool>([Option<Digit>; CellIdx::COUNT]);

impl<const ROW_MAJOR: bool> ArrGrid<ROW_MAJOR> {
    pub fn new() -> Self {
        Default::default()
    }

    fn inner_idx(idx: CellIdx) -> usize {
        if ROW_MAJOR {
            idx.row_major()
        } else {
            idx.col_major()
        }
    }
}

impl<const ROW_MAJOR: bool> Default for ArrGrid<ROW_MAJOR> {
    fn default() -> Self {
        Self([None; CellIdx::COUNT])
    }
}

impl<const ROW_MAJOR: bool> Index<CellIdx> for ArrGrid<ROW_MAJOR> {
    type Output = Option<Digit>;

    fn index(&self, idx: CellIdx) -> &Self::Output {
        &self.0[Self::inner_idx(idx)]
    }
}

impl<const ROW_MAJOR: bool> IndexMut<CellIdx> for ArrGrid<ROW_MAJOR> {
    fn index_mut(&mut self, idx: CellIdx) -> &mut Self::Output {
        &mut self.0[Self::inner_idx(idx)]
    }
}

impl super::Grid for ArrGrid<true> {
    fn iter_row_wise(&self) -> impl Iterator<Item = (CellIdx, Option<Digit>)> {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, value)| (CellIdx::try_of_row_major(idx).unwrap(), value.clone()))
    }

    fn iter_col_wise(&self) -> impl Iterator<Item = (CellIdx, Option<Digit>)> {
        CellIdx::iter_col_wise().map(|idx| (idx, self[idx].clone()))
    }

    fn iter(&self) -> impl Iterator<Item = (CellIdx, Option<Digit>)> {
        self.iter_row_wise()
    }
}

impl super::Grid for ArrGrid<false> {
    fn iter_row_wise(&self) -> impl Iterator<Item = (CellIdx, Option<Digit>)> {
        CellIdx::iter_row_wise().map(|idx| (idx, self[idx].clone()))
    }

    fn iter_col_wise(&self) -> impl Iterator<Item = (CellIdx, Option<Digit>)> {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, value)| (CellIdx::try_of_row_major(idx).unwrap(), value.clone()))
    }

    fn iter(&self) -> impl Iterator<Item = (CellIdx, Option<Digit>)> {
        self.iter_col_wise()
    }
}

impl<const ROW_MAJOR: bool> super::GridMut for ArrGrid<ROW_MAJOR> where
    ArrGrid<ROW_MAJOR>: super::Grid
{
}

impl<const ROW_MAJOR: bool> super::GridMutWithDefault for ArrGrid<ROW_MAJOR> where
    ArrGrid<ROW_MAJOR>: super::Grid
{
}

impl<const ROW_MAJOR: bool, T> PartialEq<T> for ArrGrid<ROW_MAJOR>
where
    T: super::Grid,
    ArrGrid<ROW_MAJOR>: super::Grid,
{
    fn eq(&self, other: &T) -> bool {
        super::eq(self, other)
    }
}

impl<const ROW_MAJOR: bool, T> PartialOrd<T> for ArrGrid<ROW_MAJOR>
where
    T: super::Grid,
    ArrGrid<ROW_MAJOR>: super::Grid,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        Some(super::cmp(self, other))
    }
}

impl<const ROW_MAJOR: bool> std::fmt::Debug for ArrGrid<ROW_MAJOR>
where
    ArrGrid<ROW_MAJOR>: super::Grid,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::fmt(self, f)
    }
}
