use oxc_allocator::{ArenaVec, GetAllocator};

/// Keep short lists on the stack until their final length is known.
pub(super) struct BufferedVec<'a, T> {
    buffer: [Option<T>; 4],
    buffered_len: usize,
    vec: ArenaVec<'a, T>,
}

impl<'a, T> BufferedVec<'a, T> {
    pub(super) fn new_in(allocator: &impl GetAllocator<'a>) -> Self {
        Self {
            buffer: std::array::from_fn(|_| None),
            buffered_len: 0,
            vec: ArenaVec::new_in(allocator),
        }
    }

    #[inline]
    pub(super) fn push(&mut self, value: T) {
        if self.buffered_len < self.buffer.len() {
            self.buffer[self.buffered_len] = Some(value);
            self.buffered_len += 1;
        } else {
            if self.vec.is_empty() {
                self.vec.reserve(8);
                self.vec.extend(self.buffer.iter_mut().map(|element| element.take().unwrap()));
            }
            self.vec.push(value);
        }
    }

    pub(super) fn into_vec(mut self) -> ArenaVec<'a, T> {
        if self.vec.is_empty() {
            self.vec.extend(self.buffer.into_iter().take(self.buffered_len).map(Option::unwrap));
        }
        self.vec
    }
}
