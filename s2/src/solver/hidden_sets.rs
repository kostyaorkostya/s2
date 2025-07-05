use crate::grid::DIM;
use crate::util::{BoolMatrix9x9, Domain};
use std::iter::zip;

#[derive(Debug, Default)]
pub struct HiddenSets<T>
where
    T: Default,
{
    elts_per_digit: [(u8, [T; DIM]); DIM],
    eq: BoolMatrix9x9,
}

impl<T> HiddenSets<T>
where
    T: Default,
{
    fn clear(&mut self) {
        self.elts_per_digit.iter_mut().for_each(|(len, _)| *len = 0);
        self.eq.clear();
    }

    fn init_eq_rec(elt: &[T], elt_idx: u8, tail: &[(u8, [T; DIM])], eq: &mut BoolMatrix9x9)
    where
        T: Eq,
    {
        tail.iter()
            .enumerate()
            .filter(|(_, other)| {
                let other = &other.1[..(other.0 as usize)];
                other == elt
            })
            .map(|(tail_idx, _)| elt_idx + 1u8 + (tail_idx as u8))
            .for_each(|other_idx| {
                eq.set((elt_idx, other_idx));
                eq.set((other_idx, elt_idx))
            });
        if let Some((elt, tail)) = tail.split_first() {
            Self::init_eq_rec(&elt.1[..(elt.0 as usize)], elt_idx + 1, tail, eq)
        }
    }

    pub fn init<I>(&mut self, iter: I)
    where
        I: Iterator<Item = (Domain, T)>,
        T: Copy + Eq,
    {
        self.clear();
        iter.for_each(|(domain, idx)| {
            domain.iter().for_each(|digit| {
                let elts = &mut self.elts_per_digit[digit as usize];
                elts.1[elts.0 as usize] = idx;
                elts.0 += 1;
            })
        });
        if let Some((elt, tail)) = self.elts_per_digit[..].split_first() {
            Self::init_eq_rec(&elt.1[..(elt.0 as usize)], 0, tail, &mut self.eq)
        }
    }

    pub fn map_first<F, R>(&self, size: u8, f: F) -> Option<R>
    where
        F: for<'a> FnOnce(Domain, &'a [T]) -> R,
    {
        zip(self.elts_per_digit.iter(), self.eq.iter_rows())
            .filter(|(_, eq)| eq.count_ones() == size)
            .next()
            .map(|(elts, eq)| f(eq.into(), &elts.1[..(elts.0 as usize)]))
    }
}
