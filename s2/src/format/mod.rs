use super::grid::{Grid, GridIdx, GridMut, GridMutWithDefault, IIdx};
use std::io::{Cursor, Read, Write};
use strum::EnumCount;

mod row_major_ascii;
pub use row_major_ascii::RowMajorAscii;

pub trait ReadFormatter {
    type ReadError;

    fn read<R, G>(&self, reader: &mut R, grid: &mut G) -> Result<(), Self::ReadError>
    where
        R: Read,
        G: GridMut + ?Sized;

    fn read_from_bytes<G>(&self, b: &[u8], grid: &mut G) -> Result<(), Self::ReadError>
    where
        G: GridMut + ?Sized,
    {
        self.read(&mut Cursor::new(b), grid)
    }

    fn read_from_string<G>(&self, s: &str, grid: &mut G) -> Result<(), Self::ReadError>
    where
        G: GridMut + ?Sized,
    {
        self.read_from_bytes::<G>(s.as_bytes(), grid)
    }
}

pub trait WriteFormatter {
    fn write<G, W>(&self, grid: &G, writer: &mut W) -> std::io::Result<usize>
    where
        G: Grid + ?Sized,
        W: Write;
}

pub fn read_from_string<F, G>(f: &F, s: &str) -> Result<G, F::ReadError>
where
    F: ReadFormatter,
    G: GridMutWithDefault,
{
    let mut grid = G::default();
    f.read_from_string(s, &mut grid)?;
    Ok(grid)
}

pub fn write_string<F, G>(f: &F, grid: &G) -> String
where
    F: WriteFormatter,
    G: Grid + ?Sized,
{
    let mut cursor = Cursor::new(Vec::with_capacity(GridIdx::COUNT + IIdx::COUNT - 1));
    f.write(grid, &mut cursor).unwrap();
    String::from_utf8(cursor.into_inner()).unwrap()
}
