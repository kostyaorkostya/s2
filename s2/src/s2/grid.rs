use std::convert::{Into, TryFrom};
use std::ops::Index;
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
            0 => Ok(IIdx::I0),
            1 => Ok(IIdx::I1),
            2 => Ok(IIdx::I2),
            3 => Ok(IIdx::I3),
            4 => Ok(IIdx::I4),
            5 => Ok(IIdx::I5),
            6 => Ok(IIdx::I6),
            7 => Ok(IIdx::I7),
            8 => Ok(IIdx::I8),
            _ => Err(()),
        }
    }
}

impl Into<usize> for IIdx {
    fn into(self) -> usize {
        match self {
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
            0 => Ok(JIdx::J0),
            1 => Ok(JIdx::J1),
            2 => Ok(JIdx::J2),
            3 => Ok(JIdx::J3),
            4 => Ok(JIdx::J4),
            5 => Ok(JIdx::J5),
            6 => Ok(JIdx::J6),
            7 => Ok(JIdx::J7),
            8 => Ok(JIdx::J8),
            _ => Err(()),
        }
    }
}

impl Into<usize> for JIdx {
    fn into(self) -> usize {
        match self {
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
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
}

impl TryFrom<usize> for GridValue {
    type Error = ();

    fn try_from(item: usize) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(GridValue::V0),
            1 => Ok(GridValue::V1),
            2 => Ok(GridValue::V2),
            3 => Ok(GridValue::V3),
            4 => Ok(GridValue::V4),
            5 => Ok(GridValue::V5),
            6 => Ok(GridValue::V6),
            7 => Ok(GridValue::V7),
            8 => Ok(GridValue::V8),
            _ => Err(()),
        }
    }
}

impl Into<usize> for GridValue {
    fn into(self) -> usize {
        match self {
            GridValue::V0 => 0,
            GridValue::V1 => 1,
            GridValue::V2 => 2,
            GridValue::V3 => 3,
            GridValue::V4 => 4,
            GridValue::V5 => 5,
            GridValue::V6 => 6,
            GridValue::V7 => 7,
            GridValue::V8 => 8,
        }
    }
}

impl Into<char> for GridValue {
    fn into(self) -> char {
        match self {
            GridValue::V0 => '0',
            GridValue::V1 => '1',
            GridValue::V2 => '2',
            GridValue::V3 => '3',
            GridValue::V4 => '4',
            GridValue::V5 => '5',
            GridValue::V6 => '6',
            GridValue::V7 => '7',
            GridValue::V8 => '8',
        }
    }
}

pub type Idx = (IIdx, JIdx);

pub fn to_row_major(idx: Idx) -> usize {
    let i: usize = idx.0.into();
    let j: usize = idx.1.into();
    i * IIdx::COUNT + j
}

pub fn to_column_major(idx: Idx) -> usize {
    let i: usize = idx.0.into();
    let j: usize = idx.1.into();
    j * JIdx::COUNT + i
}

pub fn render<T>(grid: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
where
    T: Index<Idx, Output = Option<GridValue>>,
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
            write!(f, "\n")?;
            write!(f, "{}\n:", "_".repeat(JIdx::COUNT * 2 - 1))?;
        }
    }
    Ok(())
}
