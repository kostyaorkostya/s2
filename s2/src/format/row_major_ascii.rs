use super::{ReadFormatter, WriteFormatter};
use crate::grid::{Grid, CellIdx, GridMut, Digit, RowIdx, ColIdx};
use std::io::{Read, Write};
use std::slice;
use strum::EnumCount;

#[derive(Debug)]
pub struct RowMajorAscii {
    empty_cell: u8,
    row_sep: Option<u8>,
}

#[derive(Debug)]
struct RowMajorAsciiReadState<'a> {
    row_sep_expected: bool,
    row_major_idx: usize,
    formatter: &'a RowMajorAscii,
}

impl<'a> RowMajorAsciiReadState<'a> {
    fn new<'b>(formatter: &'a RowMajorAscii) -> Self
    where
        'b: 'a,
    {
        Self {
            row_sep_expected: false,
            row_major_idx: 0,
            formatter,
        }
    }

    fn inc(&mut self) {
        let idx = CellIdx::try_of_row_major(self.row_major_idx).unwrap();
        if idx.row < RowIdx::Row8 && idx.col == ColIdx::Col8 {
            self.row_sep_expected = self.formatter.row_sep.is_some();
        };
        self.row_major_idx += 1;
    }

    fn is_row_sep_expected(&self) -> bool {
        self.formatter.row_sep.is_some() && self.row_sep_expected
    }

    fn saw_row_sep(&mut self) {
        self.row_sep_expected = false;
    }

    fn is_done(&self) -> bool {
        self.row_major_idx >= RowIdx::COUNT * ColIdx::COUNT
    }
}

impl ReadFormatter for RowMajorAscii {
    // TODO(kostya): better error type
    type ReadError = ();

    fn read<R, G>(&self, reader: &mut R, grid: &mut G) -> Result<(), Self::ReadError>
    where
        R: Read,
        G: GridMut + ?Sized,
    {
        let mut state = RowMajorAsciiReadState::new(self);
        let is_cell = |c: u8| c.is_ascii_digit() && c != b'0';
        let is_empty = |c: u8| c == self.empty_cell;
        let is_row_sep = |c: u8| self.row_sep == Some(c);
        loop {
            let idx = CellIdx::try_of_row_major(state.row_major_idx).unwrap();
            let mut c = 0;
            match reader.read_exact(slice::from_mut(&mut c)) {
                Err(_) => return Err(()),
                Ok(()) => {
                    if state.is_row_sep_expected() {
                        if is_row_sep(c) {
                            state.saw_row_sep();
                            continue;
                        } else if c.is_ascii_whitespace() {
                            continue;
                        } else {
                            return Err(());
                        }
                    } else if c.is_ascii_whitespace() {
                        continue;
                    } else if is_cell(c) {
                        grid[idx] = Some(Digit::try_from_ascii(c).unwrap());
                        state.inc();
                    } else if is_empty(c) {
                        grid[idx] = None;
                        state.inc();
                    } else {
                        return Err(());
                    }
                }
            }

            if state.is_done() {
                return Ok(());
            }
        }
    }
}

impl WriteFormatter for RowMajorAscii {
    fn write<G, W>(&self, grid: &G, writer: &mut W) -> std::io::Result<usize>
    where
        G: Grid + ?Sized,
        W: Write,
    {
        CellIdx::iter_row_wise().try_fold(0, |res, idx| {
            let cell = grid[idx].map(|x| x.as_ascii()).unwrap_or(self.empty_cell);
            let cell = writer.write(slice::from_ref(&cell))?;
            let row_sep = if idx.row != RowIdx::Row8 && idx.col == ColIdx::Col8 {
                self.row_sep
                    .map_or(Ok(0), |x| writer.write(slice::from_ref(&x)))
            } else {
                Ok(0)
            }?;
            Ok(res + cell + row_sep)
        })
    }
}

impl Default for RowMajorAscii {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl RowMajorAscii {
    pub fn new(empty_cell: Option<char>, row_sep: Option<Option<char>>) -> Self {
        let empty_cell: u8 = empty_cell.unwrap_or('_').try_into().unwrap();
        let row_sep: Option<u8> = row_sep.unwrap_or(Some('\n')).map(|x| x.try_into().unwrap());
        Self {
            empty_cell,
            row_sep,
        }
    }
}

#[cfg(test)]
mod row_major_ascii_test {
    use super::super::{read_from_string, write_string, ReadFormatter, WriteFormatter};
    use super::RowMajorAscii;
    use crate::grid::{ArrGridRowMajor, Grid, GridMutWithDefault};

    fn grid_roundtrip<F, Src, Dst>(f: &F, src: &Src) -> Dst
    where
        F: WriteFormatter + ReadFormatter,
        F::ReadError: std::fmt::Debug,
        Src: Grid + ?Sized,
        Dst: GridMutWithDefault,
    {
        let s = write_string(f, src);
        read_from_string(f, &s).unwrap()
    }

    fn str_roundtrip<F>(f: &F, s: &str) -> String
    where
        F: WriteFormatter + ReadFormatter,
        F::ReadError: std::fmt::Debug,
    {
        let s: ArrGridRowMajor = read_from_string(f, s).unwrap();
        write_string(f, &s)
    }

    #[test]
    fn test_string_of_empty_grid() {
        let expected = r#"
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
        .trim();
        let f = RowMajorAscii::default();
        let grid = ArrGridRowMajor::default();
        let actual = write_string(&f, &grid);
        assert_eq!(&expected, &actual);
    }

    #[test]
    fn test_empty_grid_roundtrip() {
        let f = RowMajorAscii::default();
        let src = ArrGridRowMajor::default();
        let dst: ArrGridRowMajor = grid_roundtrip(&f, &src);
        assert_eq!(&src, &dst);
    }

    #[test]
    fn test_non_empty() {
        let expected = r#"
53__7____
6__195___
_98____6_
8___6___3
4__8_3__1
7___2___6
_6____28_
___419__5
____8__79
"#
        .trim();
        let f = RowMajorAscii::default();
        let actual = str_roundtrip(&f, &expected);
        assert_eq!(&expected, &actual);
    }
}
