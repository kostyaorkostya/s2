use itertools::Itertools;
use std::cmp::Ordering;
use std::convert::{Into, TryFrom};
use std::ops::{Index, IndexMut};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter as EnumIterMacro};

mod plain_grid;
pub use plain_grid::PlainGrid;

#[derive(Debug, Clone, Copy, EnumIterMacro, EnumCountMacro, PartialEq, Eq, PartialOrd, Ord)]
pub enum IIdx {
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

impl TryFrom<usize> for IIdx {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
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

impl From<IIdx> for usize {
    fn from(value: IIdx) -> usize {
        match value {
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

#[derive(Debug, Clone, Copy, EnumIterMacro, EnumCountMacro, PartialEq, Eq, PartialOrd, Ord)]
pub enum JIdx {
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

impl TryFrom<usize> for JIdx {
    type Error = ();

    fn try_from(item: usize) -> Result<Self, Self::Error> {
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

impl From<JIdx> for usize {
    fn from(value: JIdx) -> usize {
        match value {
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

impl TryFrom<usize> for GridValue {
    type Error = ();

    fn try_from(item: usize) -> Result<Self, Self::Error> {
        match item {
            1 => Ok(Self::V1),
            2 => Ok(Self::V2),
            3 => Ok(Self::V3),
            4 => Ok(Self::V4),
            5 => Ok(Self::V5),
            6 => Ok(Self::V6),
            7 => Ok(Self::V7),
            8 => Ok(Self::V8),
            9 => Ok(Self::V9),
            _ => Err(()),
        }
    }
}

impl From<GridValue> for usize {
    fn from(value: GridValue) -> usize {
        match value {
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

impl std::fmt::Display for GridValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x: char = match self {
            Self::V1 => '1',
            Self::V2 => '2',
            Self::V3 => '3',
            Self::V4 => '4',
            Self::V5 => '5',
            Self::V6 => '6',
            Self::V7 => '7',
            Self::V8 => '8',
            Self::V9 => '9',
        };
        write!(f, "{}", x)
    }
}

pub type GridIdx = (IIdx, JIdx);

pub fn to_row_major(idx: GridIdx) -> usize {
    let i: usize = idx.0.into();
    let j: usize = idx.1.into();
    i * IIdx::COUNT + j
}

pub fn try_of_row_major(idx: usize) -> Result<GridIdx, ()> {
    let i: IIdx = (idx / IIdx::COUNT).try_into()?;
    let j: JIdx = (idx % JIdx::COUNT).try_into()?;
    Ok((i, j))
}

pub fn of_row_major(idx: usize) -> GridIdx {
    try_of_row_major(idx).unwrap()
}

pub fn to_col_major(idx: GridIdx) -> usize {
    let i: usize = idx.0.into();
    let j: usize = idx.1.into();
    j * JIdx::COUNT + i
}

pub fn try_of_col_major(idx: usize) -> Result<GridIdx, ()> {
    let j: JIdx = (idx / JIdx::COUNT).try_into()?;
    let i: IIdx = (idx % IIdx::COUNT).try_into()?;
    Ok((i, j))
}

pub fn copy<GridSrc, GridDst>(src: &GridSrc, dst: &mut GridDst)
where
    GridSrc: Index<GridIdx, Output = Option<GridValue>>,
    GridDst: IndexMut<GridIdx, Output = Option<GridValue>>,
{
    IIdx::iter()
        .cartesian_product(JIdx::iter())
        .for_each(|idx| dst[idx] = src[idx]);
}

pub fn copy_into<GridSrc, GridDst>(src: &GridSrc) -> GridDst
where
    GridSrc: Index<GridIdx, Output = Option<GridValue>>,
    GridDst: IndexMut<GridIdx, Output = Option<GridValue>> + Default,
{
    let mut dst = GridDst::default();
    copy(src, &mut dst);
    dst
}

pub fn apply<Grid, Placement>(grid: &mut Grid, placement: Placement)
where
    Grid: IndexMut<GridIdx, Output = Option<GridValue>>,
    Placement: Iterator<Item = (GridIdx, GridValue)>,
{
    placement.for_each(|(idx, value)| grid[idx] = Some(value))
}

pub fn copy_and_apply<GridSrc, GridDst, Placement>(src: &GridSrc, placement: Placement) -> GridDst
where
    GridSrc: Index<GridIdx, Output = Option<GridValue>>,
    GridDst: IndexMut<GridIdx, Output = Option<GridValue>> + Default,
    Placement: Iterator<Item = (GridIdx, GridValue)>,
{
    let mut dst = GridDst::default();
    copy(src, &mut dst);
    apply(&mut dst, placement);
    dst
}

pub fn eq<T, U>(this: &T, other: &U) -> bool
where
    T: Index<GridIdx, Output = Option<GridValue>>,
    U: Index<GridIdx, Output = Option<GridValue>>,
{
    IIdx::iter()
        .cartesian_product(JIdx::iter())
        .all(|idx| this[idx] == other[idx])
}

pub fn partial_cmp<T, U>(this: &T, other: &U) -> Option<std::cmp::Ordering>
where
    T: Index<GridIdx, Output = Option<GridValue>>,
    U: Index<GridIdx, Output = Option<GridValue>>,
{
    for i in IIdx::iter() {
        for j in JIdx::iter() {
            match this[(i, j)].partial_cmp(&other[(i, j)]) {
                None => return None,
                Some(Ordering::Equal) => continue,
                res @ Some(_) => return res,
            }
        }
    }

    Some(Ordering::Equal)
}
