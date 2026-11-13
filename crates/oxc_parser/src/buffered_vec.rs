use oxc_allocator::{ArenaVec, GetAllocator};

/// Buffer the first list elements before allocating in the arena.
pub struct BufferedVec<'a, T, const N: usize = 1> {
    buffer: [Option<T>; N],
    len: usize,
    vec: ArenaVec<'a, T>,
}

impl<'a, T, const N: usize> BufferedVec<'a, T, N> {
    #[inline]
    pub fn new_in(allocator: &impl GetAllocator<'a>) -> Self {
        Self { buffer: std::array::from_fn(|_| None), len: 0, vec: ArenaVec::new_in(allocator) }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        if self.len < N {
            self.buffer[self.len] = Some(value);
        } else {
            if self.len == N {
                self.vec.reserve(N * 2);
                self.vec.extend(self.buffer.iter_mut().filter_map(Option::take));
            }
            self.vec.push(value);
        }
        self.len += 1;
    }

    #[inline]
    pub fn into_vec(mut self) -> ArenaVec<'a, T> {
        if self.len <= N {
            self.vec.reserve_exact(self.len);
            self.vec.extend(self.buffer.into_iter().flatten());
        }
        self.vec
    }
}

impl<'a, T, const N: usize> IntoIterator for BufferedVec<'a, T, N> {
    type Item = T;
    type IntoIter = std::iter::Chain<
        std::iter::Flatten<std::array::IntoIter<Option<T>, N>>,
        <ArenaVec<'a, T> as IntoIterator>::IntoIter,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter().flatten().chain(self.vec)
    }
}
