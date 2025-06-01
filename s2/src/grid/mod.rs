use itertools::Itertools;
use std::cmp::Ordering;
use std::convert::{Into, TryFrom};
use std::iter::zip;
use std::ops::{Index, IndexMut};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter as EnumIterMacro};

mod plain_grid;
pub use plain_grid::PlainGrid;

#[derive(
    Debug, Default, Clone, Copy, EnumIterMacro, EnumCountMacro, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum IIdx {
    #[default]
    I0,
    I1,
    I2,
    I3,
    I4,
    I5,
    I6,
    I7,
    I8,
}

impl TryFrom<&usize> for IIdx {
    type Error = ();

    fn try_from(v: &usize) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::I0),
            1 => Ok(Self::I1),
            2 => Ok(Self::I2),
            3 => Ok(Self::I3),
            4 => Ok(Self::I4),
            5 => Ok(Self::I5),
            6 => Ok(Self::I6),
            7 => Ok(Self::I7),
            8 => Ok(Self::I8),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for IIdx {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        (&v).try_into()
    }
}

impl From<&IIdx> for u8 {
    fn from(v: &IIdx) -> u8 {
        match v {
            IIdx::I0 => 0,
            IIdx::I1 => 1,
            IIdx::I2 => 2,
            IIdx::I3 => 3,
            IIdx::I4 => 4,
            IIdx::I5 => 5,
            IIdx::I6 => 6,
            IIdx::I7 => 7,
            IIdx::I8 => 8,
        }
    }
}

impl From<IIdx> for u8 {
    fn from(v: IIdx) -> u8 {
        (&v).into()
    }
}

impl From<&IIdx> for usize {
    fn from(v: &IIdx) -> usize {
        let v: u8 = v.into();
        v.into()
    }
}

impl From<IIdx> for usize {
    fn from(v: IIdx) -> usize {
        (&v).into()
    }
}

#[derive(
    Debug, Default, Clone, Copy, EnumIterMacro, EnumCountMacro, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum JIdx {
    #[default]
    J0,
    J1,
    J2,
    J3,
    J4,
    J5,
    J6,
    J7,
    J8,
}

impl TryFrom<&usize> for JIdx {
    type Error = ();

    fn try_from(item: &usize) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(Self::J0),
            1 => Ok(Self::J1),
            2 => Ok(Self::J2),
            3 => Ok(Self::J3),
            4 => Ok(Self::J4),
            5 => Ok(Self::J5),
            6 => Ok(Self::J6),
            7 => Ok(Self::J7),
            8 => Ok(Self::J8),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for JIdx {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        (&v).try_into()
    }
}

impl From<&JIdx> for u8 {
    fn from(v: &JIdx) -> u8 {
        match v {
            JIdx::J0 => 0,
            JIdx::J1 => 1,
            JIdx::J2 => 2,
            JIdx::J3 => 3,
            JIdx::J4 => 4,
            JIdx::J5 => 5,
            JIdx::J6 => 6,
            JIdx::J7 => 7,
            JIdx::J8 => 8,
        }
    }
}

impl From<JIdx> for u8 {
    fn from(v: JIdx) -> u8 {
        (&v).into()
    }
}

impl From<&JIdx> for usize {
    fn from(v: &JIdx) -> usize {
        let v: u8 = v.into();
        v.into()
    }
}

impl From<JIdx> for usize {
    fn from(v: JIdx) -> usize {
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
    i: IIdx,
    j: JIdx,
}

impl From<&(IIdx, JIdx)> for GridIdx {
    fn from(v: &(IIdx, JIdx)) -> Self {
        Self {
            i: v.0.clone(),
            j: v.1.clone(),
        }
    }
}

impl From<(IIdx, JIdx)> for GridIdx {
    fn from(v: (IIdx, JIdx)) -> Self {
        (&v).into()
    }
}

impl From<(&IIdx, JIdx)> for GridIdx {
    fn from(v: (&IIdx, JIdx)) -> Self {
        Self {
            i: v.0.clone(),
            j: v.1,
        }
    }
}

impl From<(IIdx, &JIdx)> for GridIdx {
    fn from(v: (IIdx, &JIdx)) -> Self {
        Self {
            i: v.0,
            j: v.1.clone(),
        }
    }
}

impl From<&GridIdx> for (IIdx, JIdx) {
    fn from(v: &GridIdx) -> Self {
        (v.i.clone(), v.j.clone())
    }
}

impl From<GridIdx> for (IIdx, JIdx) {
    fn from(v: GridIdx) -> Self {
        (&v).into()
    }
}

impl GridIdx {
    const COUNT: usize = IIdx::COUNT * JIdx::COUNT;

    fn row_major(&self) -> usize {
        let i: usize = self.i.into();
        let j: usize = self.j.into();
        i * IIdx::COUNT + j
    }

    fn try_of_row_major(idx: usize) -> Result<Self, ()> {
        let i: IIdx = (idx / IIdx::COUNT).try_into()?;
        let j: JIdx = (idx % JIdx::COUNT).try_into()?;
        Ok((i, j).into())
    }

    fn col_major(&self) -> usize {
        let i: usize = self.i.into();
        let j: usize = self.j.into();
        j * JIdx::COUNT + i
    }

    fn try_of_col_major(idx: usize) -> Result<Self, ()> {
        let j: JIdx = (idx / JIdx::COUNT).try_into()?;
        let i: IIdx = (idx % IIdx::COUNT).try_into()?;
        Ok((i, j).into())
    }

    fn iter_row_wise() -> impl Iterator<Item = Self> {
        IIdx::iter().cartesian_product(JIdx::iter()).map(Into::into)
    }

    fn iter_col_wise() -> impl Iterator<Item = Self> {
        JIdx::iter()
            .cartesian_product(IIdx::iter())
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
        GridIdx::iter_row_wise().map(|idx| (idx, self[idx].clone()))
    }

    fn iter_col_wise(&self) -> impl Iterator<Item = (GridIdx, Option<GridValue>)> {
        GridIdx::iter_col_wise().map(|idx| (idx, self[idx].clone()))
    }

    fn iter_values_row_wise(&self) -> impl Iterator<Item = Option<GridValue>> {
        self.iter_row_wise().map(|(_, x)| x)
    }

    fn iter_values_col_wise(&self) -> impl Iterator<Item = Option<GridValue>> {
        self.iter_col_wise().map(|(_, x)| x)
    }

    fn iter_set_row_wise(&self) -> impl Iterator<Item = (GridIdx, GridValue)> {
        self.iter_row_wise()
            .filter_map(|(idx, value)| Some((idx, value?)))
    }

    fn iter_set_col_wise(&self) -> impl Iterator<Item = (GridIdx, GridValue)> {
        self.iter_col_wise()
            .filter_map(|(idx, value)| Some((idx, value?)))
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
        T: Grid + IndexMut<GridIdx, Output = Option<GridValue>> + Default + Sized,
    {
        let mut dst = T::default();
        self.iter_row_wise()
            .for_each(|(idx, value)| dst[idx] = value);
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
        T: Index<GridIdx, Output = Option<GridValue>> + ?Sized,
    {
        GridIdx::iter_row_wise().for_each(|idx| self[idx] = src[idx])
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
