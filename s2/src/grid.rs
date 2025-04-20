use std::conversion::From

#[derive(Debug)]
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

impl From<i32> for IIdx {
    fn from(item: i32) -> Self {
        match item {
            0 => I0,
            1 => I1,
            2 => I2,
            3 => I3,
            4 => I4,
            5 => I5,
            6 => I6,
            7 => I7,
            8 => I8,
        }
    }
}

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
