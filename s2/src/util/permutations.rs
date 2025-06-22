use std::array;
#[derive(Debug, Default)]
enum State {
    #[default]
    Preamble,
    Loop,
}

struct Permutator<const LENGTH: usize> {
    state: State,
    i: u8,
    stack: [u8; LENGTH],
}

impl<const LENGTH: usize> Default for Permutator<LENGTH> {
    fn default() -> Self {
        Self {
            state: State::Preamble,
            i: 0,
            stack: [0u8; LENGTH],
        }
    }
}

impl<const LENGTH: usize> Permutator<LENGTH> {
    fn new() -> Self {
        Default::default()
    }

    fn next<T>(&mut self, arr: &mut [T; LENGTH]) {
        while self.i < LENGTH as u8 {
            match self.state {
                State::Preamble => {
                    self.i = 1;
                    self.state = State::Loop;
                    return;
                }
                State::Loop => {
                    if self.stack[self.i as usize] < self.i {
                        arr.swap(
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
}

pub fn try_find<const LENGTH: usize, I, Elt, F, T, E, Cancelled>(
    mut iter: I,
    mut f: F,
    cancelled: Cancelled,
) -> Result<T, E>
where
    Elt: Copy,
    I: Iterator<Item = Elt>,
    F: for<'a> FnMut(&'a mut dyn Iterator<Item = Elt>) -> Result<T, E>,
    Cancelled: Fn(&E) -> bool,
{
    let mut arr: [Elt; LENGTH] = array::from_fn(|_| iter.next().unwrap());
    if let Some(_) = iter.next() {
        panic!("too many")
    };
    let mut permutator = Permutator::<LENGTH>::new();
    let cnt = (1..=LENGTH).product();
    (1..=cnt)
        .map(|i| {
            permutator.next(&mut arr);
            (i, f(&mut arr.iter().copied()))
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

#[cfg(test)]
mod test {
    use super::Permutator;

    #[test]
    fn size_1() {
        let mut arr = [1];
        let mut perm = Permutator::<1>::new();
        perm.next(&mut arr);
        assert_eq!([1], arr);
    }

    #[test]
    fn size_2() {
        let mut arr = [1, 2];
        let mut perm = Permutator::<2>::new();
        perm.next(&mut arr);
        assert_eq!([1, 2], arr);
        perm.next(&mut arr);
        assert_eq!([2, 1], arr);
    }

    #[test]
    fn size_3() {
        let mut arr = [1, 2, 3];
        let mut perm = Permutator::<3>::new();
        perm.next(&mut arr);
        assert_eq!([1, 2, 3], arr);
        perm.next(&mut arr);
        assert_eq!([2, 1, 3], arr);
        perm.next(&mut arr);
        assert_eq!([3, 1, 2], arr);
        perm.next(&mut arr);
        assert_eq!([1, 3, 2], arr);
        perm.next(&mut arr);
        assert_eq!([2, 3, 1], arr);
        perm.next(&mut arr);
        assert_eq!([3, 2, 1], arr);
    }
}
