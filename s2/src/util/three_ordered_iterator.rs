use std::cmp::Ordering;
use std::mem::replace;

pub struct ThreeOrderedIterator<T, Cmp, I1, I2, I3>
where
    Cmp: Fn(&T, &T) -> Ordering,
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
    I3: Iterator<Item = T>,
{
    cur: [Option<T>; 3],
    compare: Cmp,
    i1: I1,
    i2: I2,
    i3: I3,
}

impl<T, Cmp, I1, I2, I3> ThreeOrderedIterator<T, Cmp, I1, I2, I3>
where
    Cmp: Fn(&T, &T) -> Ordering,
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
    I3: Iterator<Item = T>,
{
    fn new(compare: Cmp, mut i1: I1, mut i2: I2, mut i3: I3) -> Self {
        let cur = [i1.next(), i2.next(), i3.next()];
        Self {
            cur,
            compare,
            i1,
            i2,
            i3,
        }
    }

    fn min_cur_idx(&self) -> Option<usize> {
        self.cur
            .iter()
            .enumerate()
            .min_by(|(_, lhs), (_, rhs)| match (lhs, rhs) {
                (Some(lhs), Some(rhs)) => (self.compare)(lhs, rhs),
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (None, None) => Ordering::Equal,
            })
            .map(|(idx, _)| idx)
    }
}

impl<T, Cmp, I1, I2, I3> Iterator for ThreeOrderedIterator<T, Cmp, I1, I2, I3>
where
    Cmp: Fn(&T, &T) -> Ordering,
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
    I3: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.min_cur_idx()?;
        let next = match idx {
            0 => self.i1.next(),
            1 => self.i2.next(),
            2 => self.i3.next(),
            _ => panic!("unreachable"),
        };
        replace(&mut self.cur[idx], next)
    }
}
