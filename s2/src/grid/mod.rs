use itertools::Itertools;
use std::cmp::Ordering;
use std::convert::{Into, TryFrom};
use std::iter::zip;
use std::ops::{Index, IndexMut};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter as EnumIterMacro};

mod arr_grid;
pub type ArrGridRowMajor = arr_grid::ArrGrid<true>;
pub type ArrGridColMajor = arr_grid::ArrGrid<false>;

pub const DIM: usize = 9;

#[derive(
    Debug, Default, Clone, Copy, EnumIterMacro, EnumCountMacro, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum RowIdx {
    #[default]
    Row0,
    Row1,
    Row2,
    Row3,
    Row4,
    Row5,
    Row6,
    Row7,
    Row8,
}

impl TryFrom<&usize> for RowIdx {
    type Error = ();

    fn try_from(v: &usize) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Row0),
            1 => Ok(Self::Row1),
            2 => Ok(Self::Row2),
            3 => Ok(Self::Row3),
            4 => Ok(Self::Row4),
            5 => Ok(Self::Row5),
            6 => Ok(Self::Row6),
            7 => Ok(Self::Row7),
            8 => Ok(Self::Row8),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for RowIdx {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        (&v).try_into()
    }
}

impl From<&RowIdx> for u8 {
    fn from(v: &RowIdx) -> u8 {
        match v {
            RowIdx::Row0 => 0,
            RowIdx::Row1 => 1,
            RowIdx::Row2 => 2,
            RowIdx::Row3 => 3,
            RowIdx::Row4 => 4,
            RowIdx::Row5 => 5,
            RowIdx::Row6 => 6,
            RowIdx::Row7 => 7,
            RowIdx::Row8 => 8,
        }
    }
}

impl From<RowIdx> for u8 {
    fn from(v: RowIdx) -> u8 {
        (&v).into()
    }
}

impl From<&RowIdx> for usize {
    fn from(v: &RowIdx) -> usize {
        let v: u8 = v.into();
        v.into()
    }
}

impl From<RowIdx> for usize {
    fn from(v: RowIdx) -> usize {
        (&v).into()
    }
}

#[derive(
    Debug, Default, Clone, Copy, EnumIterMacro, EnumCountMacro, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum ColIdx {
    #[default]
    Col0,
    Col1,
    Col2,
    Col3,
    Col4,
    Col5,
    Col6,
    Col7,
    Col8,
}

impl TryFrom<&usize> for ColIdx {
    type Error = ();

    fn try_from(item: &usize) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(Self::Col0),
            1 => Ok(Self::Col1),
            2 => Ok(Self::Col2),
            3 => Ok(Self::Col3),
            4 => Ok(Self::Col4),
            5 => Ok(Self::Col5),
            6 => Ok(Self::Col6),
            7 => Ok(Self::Col7),
            8 => Ok(Self::Col8),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for ColIdx {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        (&v).try_into()
    }
}

impl From<&ColIdx> for u8 {
    fn from(v: &ColIdx) -> u8 {
        match v {
            ColIdx::Col0 => 0,
            ColIdx::Col1 => 1,
            ColIdx::Col2 => 2,
            ColIdx::Col3 => 3,
            ColIdx::Col4 => 4,
            ColIdx::Col5 => 5,
            ColIdx::Col6 => 6,
            ColIdx::Col7 => 7,
            ColIdx::Col8 => 8,
        }
    }
}

impl From<ColIdx> for u8 {
    fn from(v: ColIdx) -> u8 {
        (&v).into()
    }
}

impl From<&ColIdx> for usize {
    fn from(v: &ColIdx) -> usize {
        let v: u8 = v.into();
        v.into()
    }
}

impl From<ColIdx> for usize {
    fn from(v: ColIdx) -> usize {
        (&v).into()
    }
}

#[derive(
    Debug, Default, Clone, Copy, EnumIterMacro, EnumCountMacro, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum GridValue {
    #[default]
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
}

impl GridValue {
    pub fn into_ascii(&self) -> u8 {
        b'1' + u8::from(self)
    }

    pub fn try_from_ascii(ascii: u8) -> Result<Self, ()> {
        ascii.checked_sub(b'1').ok_or(())?.try_into()
    }
}

#[cfg(test)]
mod grid_value_ascii {
    use super::GridValue;
    use strum::IntoEnumIterator;

    #[test]
    fn test_roundtrip() {
        let expected = GridValue::iter().collect::<Vec<_>>();
        let actual = expected
            .iter()
            .map(|x| GridValue::into_ascii(&x))
            .map(|x| GridValue::try_from_ascii(x).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(&expected, &actual);
    }
}

impl TryFrom<&usize> for GridValue {
    type Error = ();

    fn try_from(v: &usize) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::V1),
            1 => Ok(Self::V2),
            2 => Ok(Self::V3),
            3 => Ok(Self::V4),
            4 => Ok(Self::V5),
            5 => Ok(Self::V6),
            6 => Ok(Self::V7),
            7 => Ok(Self::V8),
            8 => Ok(Self::V9),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for GridValue {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        (&v).try_into()
    }
}

impl TryFrom<&u8> for GridValue {
    type Error = ();

    fn try_from(v: &u8) -> Result<Self, Self::Error> {
        (*v as usize).try_into()
    }
}

impl TryFrom<u8> for GridValue {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        (&v).try_into()
    }
}

impl From<&GridValue> for u8 {
    fn from(v: &GridValue) -> u8 {
        match v {
            GridValue::V1 => 0,
            GridValue::V2 => 1,
            GridValue::V3 => 2,
            GridValue::V4 => 3,
            GridValue::V5 => 4,
            GridValue::V6 => 5,
            GridValue::V7 => 6,
            GridValue::V8 => 7,
            GridValue::V9 => 8,
        }
    }
}

impl From<GridValue> for u8 {
    fn from(v: GridValue) -> u8 {
        (&v).into()
    }
}

impl From<&GridValue> for usize {
    fn from(v: &GridValue) -> usize {
        let v: u8 = v.into();
        v.into()
    }
}

impl From<GridValue> for usize {
    fn from(v: GridValue) -> usize {
        (&v).into()
    }
}

impl std::fmt::Display for GridValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x: char = self.into_ascii().into();
        write!(f, "{x}")
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridIdx {
    pub row: RowIdx,
    pub col: ColIdx,
}

impl From<&(RowIdx, ColIdx)> for GridIdx {
    fn from(v: &(RowIdx, ColIdx)) -> Self {
        Self {
            row: v.0.clone(),
            col: v.1.clone(),
        }
    }
}

impl From<(RowIdx, ColIdx)> for GridIdx {
    fn from(v: (RowIdx, ColIdx)) -> Self {
        (&v).into()
    }
}

impl From<(&RowIdx, ColIdx)> for GridIdx {
    fn from(v: (&RowIdx, ColIdx)) -> Self {
        Self {
            row: v.0.clone(),
            col: v.1,
        }
    }
}

impl From<(RowIdx, &ColIdx)> for GridIdx {
    fn from(v: (RowIdx, &ColIdx)) -> Self {
        Self {
            row: v.0,
            col: v.1.clone(),
        }
    }
}

impl From<&GridIdx> for (RowIdx, ColIdx) {
    fn from(v: &GridIdx) -> Self {
        (v.row.clone(), v.col.clone())
    }
}

impl From<GridIdx> for (RowIdx, ColIdx) {
    fn from(v: GridIdx) -> Self {
        (&v).into()
    }
}

impl GridIdx {
    pub const COUNT: usize = RowIdx::COUNT * ColIdx::COUNT;

    pub fn row_major(&self) -> usize {
        let i: usize = self.row.into();
        let j: usize = self.col.into();
        i * RowIdx::COUNT + j
    }

    pub fn try_of_row_major(idx: usize) -> Result<Self, ()> {
        let i: RowIdx = (idx / RowIdx::COUNT).try_into()?;
        let j: ColIdx = (idx % ColIdx::COUNT).try_into()?;
        Ok((i, j).into())
    }

    pub fn col_major(&self) -> usize {
        let i: usize = self.row.into();
        let j: usize = self.col.into();
        j * ColIdx::COUNT + i
    }

    pub fn try_of_col_major(idx: usize) -> Result<Self, ()> {
        let j: ColIdx = (idx / ColIdx::COUNT).try_into()?;
        let i: RowIdx = (idx % RowIdx::COUNT).try_into()?;
        Ok((i, j).into())
    }

    pub fn box_(&self) -> usize {
        let i: usize = self.row.into();
        let j: usize = self.col.into();
        (i / 3 * 3) + j / 3
    }

    pub fn iter_row_wise() -> impl Iterator<Item = Self> {
        RowIdx::iter().cartesian_product(ColIdx::iter()).map(Into::into)
    }

    pub fn iter_col_wise() -> impl Iterator<Item = Self> {
        ColIdx::iter()
            .cartesian_product(RowIdx::iter())
            .map(|(j, i)| (i, j))
            .map(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GridDiff {
    Set(GridIdx, GridValue),
    Unset(GridIdx),
}

pub trait Grid: Index<GridIdx, Output = Option<GridValue>> {
    fn iter_row_wise(&self) -> impl Iterator<Item = (GridIdx, Option<GridValue>)> {
        GridIdx::iter_row_wise().map(|idx| (idx, self[idx]))
    }

    fn iter_col_wise(&self) -> impl Iterator<Item = (GridIdx, Option<GridValue>)> {
        GridIdx::iter_col_wise().map(|idx| (idx, self[idx]))
    }

    fn iter(&self) -> impl Iterator<Item = (GridIdx, Option<GridValue>)> {
        self.iter_row_wise()
    }

    fn iter_values_row_wise(&self) -> impl Iterator<Item = Option<GridValue>> {
        self.iter_row_wise().map(|(_, x)| x)
    }

    fn iter_values_col_wise(&self) -> impl Iterator<Item = Option<GridValue>> {
        self.iter_col_wise().map(|(_, x)| x)
    }

    fn iter_values(&self) -> impl Iterator<Item = Option<GridValue>> {
        self.iter_values_row_wise()
    }

    fn iter_set_row_wise(&self) -> impl Iterator<Item = (GridIdx, GridValue)> {
        self.iter_row_wise()
            .filter_map(|(idx, value)| Some((idx, value?)))
    }

    fn iter_set_col_wise(&self) -> impl Iterator<Item = (GridIdx, GridValue)> {
        self.iter_col_wise()
            .filter_map(|(idx, value)| Some((idx, value?)))
    }

    fn iter_set(&self) -> impl Iterator<Item = (GridIdx, GridValue)> {
        self.iter_set_row_wise()
    }

    fn iter_unset_row_wise(&self) -> impl Iterator<Item = GridIdx> {
        self.iter_row_wise().filter_map(|(idx, value)| match value {
            None => Some(idx),
            Some(_) => None,
        })
    }

    fn iter_unset_col_wise(&self) -> impl Iterator<Item = GridIdx> {
        self.iter_col_wise().filter_map(|(idx, value)| match value {
            None => Some(idx),
            Some(_) => None,
        })
    }

    fn iter_unset(&self) -> impl Iterator<Item = GridIdx> {
        self.iter_unset_row_wise()
    }

    fn diff<T>(&self, other: &T) -> impl Iterator<Item = GridDiff>
    where
        T: Grid + ?Sized,
    {
        zip(self.iter_row_wise(), other.iter_row_wise()).filter_map(|((idx, this), (_, other))| {
            match (this, other) {
                (None, None) => None,
                (None | Some(_), Some(x)) => Some(GridDiff::Set(idx, x)),
                (Some(_), None) => Some(GridDiff::Unset(idx)),
            }
        })
    }

    fn copy_into<T>(&self) -> T
    where
        T: GridMutWithDefault + Sized,
    {
        let mut dst = T::default();
        self.iter().for_each(|(idx, value)| dst[idx] = value);
        dst
    }
}

pub trait GridMut: Grid + IndexMut<GridIdx, Output = Option<GridValue>> {
    fn clear(&mut self) {
        self.unset_from_iter(GridIdx::iter_row_wise())
    }

    fn apply_diff<T>(&mut self, diff: T)
    where
        T: Iterator<Item = GridDiff>,
    {
        for diff in diff {
            match diff {
                GridDiff::Set(idx, value) => self[idx] = Some(value),
                GridDiff::Unset(idx) => self[idx] = None,
            }
        }
    }

    fn set_from_iter<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (GridIdx, GridValue)>,
    {
        iter.for_each(|(idx, value)| self[idx] = Some(value))
    }

    fn unset_from_iter<I>(&mut self, iter: I)
    where
        I: Iterator<Item = GridIdx>,
    {
        iter.for_each(|idx| self[idx] = None)
    }

    fn assign<T>(&mut self, src: &T)
    where
        T: Grid + ?Sized,
    {
        src.iter().for_each(|(idx, value)| self[idx] = value)
    }
}

pub trait GridMutWithDefault: GridMut + Default {
    fn with_diff<T, I>(src: &T, diff: I) -> Self
    where
        T: Grid + ?Sized,
        I: Iterator<Item = GridDiff>,
    {
        let mut dst: Self = src.copy_into();
        dst.apply_diff(diff);
        dst
    }

    fn of_set<I>(iter: I) -> Self
    where
        I: Iterator<Item = (GridIdx, GridValue)>,
    {
        let mut dst: Self = Default::default();
        dst.set_from_iter(iter);
        dst
    }

    fn from_fn<F>(f: F) -> Self
    where
        F: Fn(GridIdx) -> Option<GridValue>,
    {
        let mut dst: Self = Default::default();
        GridIdx::iter_row_wise().for_each(|idx| dst[idx] = f(idx));
        dst
    }

    fn copy_of<T>(other: &T) -> Self
    where
        T: Grid + ?Sized,
    {
        other.copy_into()
    }
}

pub fn eq<T, U>(this: &T, other: &U) -> bool
where
    T: Grid,
    U: Grid,
{
    zip(this.iter_values_row_wise(), other.iter_values_row_wise())
        .all(|(this, other)| this == other)
}

pub fn cmp<T, U>(this: &T, other: &U) -> Ordering
where
    T: Grid,
    U: Grid,
{
    zip(this.iter_values_row_wise(), other.iter_values_row_wise())
        .filter_map(|(this, other)| match this.cmp(&other) {
            Ordering::Equal => None,
            res @ (Ordering::Less | Ordering::Greater) => Some(res),
        })
        .next()
        .unwrap_or(Ordering::Equal)
}

pub fn fmt<T>(this: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
where
    T: Grid,
{
    let s = crate::format::write_string(&crate::format::RowMajorAscii::default(), this);
    f.write_str(&s)
}
