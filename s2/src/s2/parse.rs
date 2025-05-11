use super::grid::{of_row_major, GridIdx, GridValue, IIdx, JIdx};
use std::io::Read;
use std::ops::IndexMut;
use std::slice;
use strum::EnumCount;

fn parse_row_major_ascii_human_readable_grid<R, G>(
    reader: &mut R,
    empty_cell: Option<char>,
) -> Result<G, ()>
where
    R: Read,
    G: IndexMut<GridIdx, Output = Option<GridValue>> + Default,
{
    let empty_cell: u8 = empty_cell.unwrap_or('_').try_into().unwrap();
    let mut idx = 0;
    let mut grid = G::default();
    loop {
        let mut c = 0;
        match reader.read_exact(slice::from_mut(&mut c)) {
            Err(_) => {
                // TODO(kostya): better error type
                return Err(());
            }
            Ok(()) => {
                if empty_cell == c {
                    grid[of_row_major(idx)] = None;
                    idx += 1;
                } else if c.is_ascii_digit() && c != b'0' {
                    grid[of_row_major(idx)] = Some(((c - b'0') as usize).try_into().unwrap());
                    idx += 1;
                } else if c.is_ascii_whitespace() {
                    continue;
                } else {
                    // TODO(kostya): better error type
                    return Err(());
                }
            }
        }

        if idx >= IIdx::COUNT * JIdx::COUNT {
            return Ok(grid);
        }
    }
}
