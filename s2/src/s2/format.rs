use super::grid::{of_row_major, GridIdx, GridValue, IIdx, JIdx};
use itertools::Itertools;
use std::io::{Cursor, Read, Write};
use std::ops::{Index, IndexMut};
use std::slice;
use strum::{EnumCount, IntoEnumIterator};

pub trait ReadFormatter {
    type ReadError;

    // TODO(kostya): better error type
    fn read<R, G>(&self, reader: &mut R, grid: &mut G) -> Result<(), Self::ReadError>
    where
        R: Read,
        G: IndexMut<GridIdx, Output = Option<GridValue>>;
}

pub trait WriteFormatter {
    fn write<G, W>(&self, grid: &G, writer: &mut W) -> std::io::Result<usize>
    where
        G: Index<GridIdx, Output = Option<GridValue>>,
        W: Write;
}

pub struct RowMajorAscii {
    empty_cell: u8,
    row_separator: Option<u8>,
}

impl ReadFormatter for RowMajorAscii {
    type ReadError = ();

    fn read<R, G>(&self, reader: &mut R, grid: &mut G) -> Result<(), Self::ReadError>
    where
        R: Read,
        G: IndexMut<GridIdx, Output = Option<GridValue>>,
    {
        let mut idx = 0;
        loop {
            let mut c = 0;
            match reader.read_exact(slice::from_mut(&mut c)) {
                Err(_) => {
                    // TODO(kostya): better error type
                    return Err(());
                }
                Ok(()) => {
                    if self.empty_cell == c {
                        grid[of_row_major(idx)] = None;
                        idx += 1;
                    } else if self.row_separator.map_or(false, |x| x == c) {
                        continue;
                    } else if c.is_ascii_digit() && c != b'0' {
                        grid[of_row_major(idx)] = Some(((c - b'0') as usize).try_into().unwrap());
                        idx += 1;
                    } else {
                        // TODO(kostya): better error type
                        return Err(());
                    }
                }
            }

            if idx >= IIdx::COUNT * JIdx::COUNT {
                return Ok(());
            }
        }
    }
}

impl WriteFormatter for RowMajorAscii {
    fn write<G, W>(&self, grid: &G, writer: &mut W) -> std::io::Result<usize>
    where
        G: Index<GridIdx, Output = Option<GridValue>>,
        W: Write,
    {
        IIdx::iter()
            .cartesian_product(JIdx::iter())
            .try_fold(0, |res, idx| {
                let cell = grid[idx]
                    .map(|x| (b'0' + u8::try_from(usize::from(x)).unwrap()))
                    .unwrap_or(self.empty_cell);
                let cell = writer.write(slice::from_ref(&cell))?;
                let row_separator = if idx.0 != IIdx::I8 && idx.1 == JIdx::J8 {
                    self.row_separator
                        .map_or(Ok(0), |x| writer.write(slice::from_ref(&x)))
                } else {
                    Ok(0)
                }?;
                Ok(res + cell + row_separator)
            })
    }
}

impl RowMajorAscii {
    pub fn default() -> Self {
        Self {
            empty_cell: b'_',
            row_separator: Some(b'\n'),
        }
    }

    pub fn new(empty_cell: Option<char>, row_separator: Option<Option<char>>) -> Self {
        let empty_cell: u8 = empty_cell.unwrap_or('_').try_into().unwrap();
        let row_separator: Option<u8> = row_separator
            .unwrap_or(Some('\n'))
            .map(|x| x.try_into().unwrap());
        Self {
            empty_cell,
            row_separator,
        }
    }
}

pub fn read_from_string_into<F, G>(s: &str, f: &F, grid: &mut G) -> Result<(), F::ReadError>
where
    F: ReadFormatter,
    G: IndexMut<GridIdx, Output = Option<GridValue>>,
{
    let mut cursor = Cursor::new(s.as_bytes());
    f.read(&mut cursor, grid)
}

pub fn read_from_string<F, G>(s: &str, f: &F) -> Result<G, F::ReadError>
where
    F: ReadFormatter,
    G: IndexMut<GridIdx, Output = Option<GridValue>> + Default,
{
    let mut grid = G::default();
    read_from_string_into(s, f, &mut grid)?;
    Ok(grid)
}

pub fn write_string<F, G>(f: &F, grid: &G) -> String
where
    F: WriteFormatter,
    G: Index<GridIdx, Output = Option<GridValue>>,
{
    let mut cursor = Cursor::new(Vec::with_capacity(
        IIdx::COUNT * JIdx::COUNT + IIdx::COUNT - 1,
    ));
    f.write(grid, &mut cursor).unwrap();
    String::from_utf8(cursor.into_inner()).unwrap()
}

#[cfg(test)]
mod test {
    use super::super::PlainGrid;
    use super::write_string;
    use super::*;

    #[test]
    fn test_string_of_empty_grid() {
        let grid = PlainGrid::new();
        let f = RowMajorAscii::new(None, None);
        let s = write_string(&f, &grid);
        assert_eq!(
            &s,
            r#"
_________
_________
_________
_________
_________
_________
_________
_________
_________
"#
            .trim()
        );
    }
}
