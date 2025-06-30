use std::iter::zip;

#[derive(Debug, Default)]
enum State {
    #[default]
    Preamble,
    Loop,
}

#[derive(Debug)]
pub struct Permutator<const LENGTH: usize, Elt>
where
    Elt: Default + Copy,
{
    state: State,
    i: u8,
    stack: [u8; LENGTH],
    arr: [Elt; LENGTH],
}

impl<const LENGTH: usize, T> Default for Permutator<LENGTH, T>
where
    T: Default + Copy,
{
    fn default() -> Self {
        if LENGTH > (u8::MAX as usize) {
            panic!("too big");
        }
        Self {
            state: State::Preamble,
            i: 0,
            stack: [0u8; LENGTH],
            arr: [Default::default(); LENGTH],
        }
    }
}
impl<const LENGTH: usize, Elt> Permutator<LENGTH, Elt>
where
    Elt: Default + Copy,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        Default::default()
    }

    // Heap's algorithm turned into a state machine that yields one permutation at a time.
    fn next(&mut self, len: usize) {
        while (self.i as usize) < len {
            match self.state {
                State::Preamble => {
                    self.i = 1;
                    self.state = State::Loop;
                    return;
                }
                State::Loop => {
                    if (self.stack[self.i as usize] as usize) < self.i as usize {
                        self.arr.swap(
                            self.i as usize,
                            if self.i % 2 == 0 {
                                0u8
                            } else {
                                self.stack[self.i as usize]
                            } as usize,
                        );
                        self.stack[self.i as usize] += 1;
                        self.i = 1;
                        return;
                    } else {
                        self.stack[self.i as usize] = 0;
                        self.i += 1;
                    }
                }
            }
        }
    }

    pub fn try_find<I, F, T, E, Cancelled>(
        &mut self,
        iter: I,
        mut f: F,
        cancelled: Cancelled,
    ) -> Result<T, E>
    where
        I: Iterator<Item = Elt>,
        F: for<'a> FnMut(&'a [Elt]) -> Result<T, E>,
        Cancelled: Fn(&E) -> bool,
    {
        self.stack.fill(0);
        let len = zip(self.arr.iter_mut(), iter)
            .enumerate()
            .map(|(cnt, (elt_mut, elt))| {
                *elt_mut = elt;
                cnt
            })
            .last()
            .unwrap()
            + 1;
        let cnt = (1..=len).product();
        (1..=cnt)
            .map(|i| {
                self.next(len);
                (i, f(&self.arr[..len]))
            })
            .find(|(i, res)| {
                *i == cnt
                    || (match res {
                        Ok(_) => true,
                        Err(err) => cancelled(err),
                    })
            })
            .unwrap()
            .1
    }

    #[allow(dead_code)]
    pub fn for_each<I, F>(&mut self, iter: I, mut f: F)
    where
        I: Iterator<Item = Elt>,
        F: for<'a> FnMut(&'a [Elt]),
    {
        self.try_find(
            iter,
            move |perm| {
                f(perm);
                Err(())
            },
            |_: &()| false,
        )
        .unwrap_or(())
    }
}

#[cfg(test)]
mod test {
    use super::Permutator;

    fn all<const LENGTH: usize, Elt, I>(
        mutator: &mut Permutator<LENGTH, Elt>,
        iter: I,
    ) -> Vec<Vec<Elt>>
    where
        I: Iterator<Item = Elt>,
        Elt: Default + Copy,
    {
        let mut all = vec![];
        mutator.for_each(iter, |perm| {
            all.push(perm.to_vec());
        });
        all
    }

    #[test]
    fn size_1() {
        let all = all(&mut Permutator::<5, _>::new(), 1..=1);
        assert_eq!([vec![1]], all[..]);
    }

    #[test]
    fn size_2() {
        let all = all(&mut Permutator::<5, _>::new(), 1..=2);
        assert_eq!([vec![1, 2], vec![2, 1]], all[..]);
    }

    #[test]
    fn size_3() {
        let all = all(&mut Permutator::<5, _>::new(), 1..=3);
        assert_eq!(
            [
                vec![1, 2, 3],
                vec![2, 1, 3],
                vec![3, 1, 2],
                vec![1, 3, 2],
                vec![2, 3, 1],
                vec![3, 2, 1]
            ],
            all[..]
        );
    }
}
