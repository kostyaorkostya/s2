use itertools::Itertools;
use std::convert::{Into, TryFrom};
use std::ops::{Index, IndexMut};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter as EnumIterMacro};

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

    fn try_from(item: usize) -> Result<Self, Self::Error> {
        match item {
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

impl Into<usize> for IIdx {
    fn into(self) -> usize {
        match self {
            Self::I0 => 0,
            Self::I1 => 1,
            Self::I2 => 2,
            Self::I3 => 3,
            Self::I4 => 4,
            Self::I5 => 5,
            Self::I6 => 6,
            Self::I7 => 7,
            Self::I8 => 8,
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

impl Into<usize> for JIdx {
    fn into(self) -> usize {
        match self {
            Self::J0 => 0,
            Self::J1 => 1,
            Self::J2 => 2,
            Self::J3 => 3,
            Self::J4 => 4,
            Self::J5 => 5,
            Self::J6 => 6,
            Self::J7 => 7,
            Self::J8 => 8,
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

impl Into<usize> for GridValue {
    fn into(self) -> usize {
        match self {
            Self::V1 => 0,
            Self::V2 => 1,
            Self::V3 => 2,
            Self::V4 => 3,
            Self::V5 => 4,
            Self::V6 => 5,
            Self::V7 => 6,
            Self::V8 => 7,
            Self::V9 => 8,
        }
    }
}

impl Into<char> for GridValue {
    fn into(self) -> char {
        match self {
            Self::V1 => '1',
            Self::V2 => '2',
            Self::V3 => '3',
            Self::V4 => '4',
            Self::V5 => '5',
            Self::V6 => '6',
            Self::V7 => '7',
            Self::V8 => '8',
            Self::V9 => '9',
        }
    }
}

pub type GridIdx = (IIdx, JIdx);

pub fn to_row_major(idx: GridIdx) -> usize {
    let i: usize = idx.0.into();
    let j: usize = idx.1.into();
    i * IIdx::COUNT + j
}

pub fn to_column_major(idx: GridIdx) -> usize {
    let i: usize = idx.0.into();
    let j: usize = idx.1.into();
    j * JIdx::COUNT + i
}

pub fn render<T>(grid: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
where
    T: Index<GridIdx, Output = Option<GridValue>>,
{
    for i in IIdx::iter() {
        for j in JIdx::iter() {
            let cell: char = grid[(i, j)].map_or(' ', |x| x.into());
            write!(f, "{}", cell)?;
            if j != JIdx::J8 {
                write!(f, "|")?;
            }
        }
        if i != IIdx::I8 {
            writeln!(f, "")?;
            writeln!(f, "{}", "_".repeat(JIdx::COUNT * 2 - 1))?;
        }
    }
    Ok(())
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
