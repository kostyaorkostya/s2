use std::iter::{zip, Iterator};

pub struct SliceGroupByIterator<'a, T, F>
where
    F: Fn(&T, &T) -> bool,
{
    slice: &'a [T],
    equal: F,
    from: usize,
    to: usize,
}

impl<'a, T, F> SliceGroupByIterator<'a, T, F>
where
    F: Fn(&T, &T) -> bool,
{
    fn new(slice: &'a [T], equal: F) -> Self {
        Self {
            slice,
            equal,
            from: 0,
            to: 0,
        }
    }
}

impl<'a, T, F> Iterator for SliceGroupByIterator<'a, T, F>
where
    F: Fn(&T, &T) -> bool,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.to == self.slice.len() {
            return None;
        }

        self.from = self.to;
        self.to += zip(
            self.slice[self.to..].iter(),
            self.slice[self.to..].iter().skip(1),
        )
        .take_while(|(this, next)| (self.equal)(this, next))
        .count()
            + 1;
        Some(&self.slice[self.from..self.to])
    }
}
