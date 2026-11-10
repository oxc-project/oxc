use oxc_allocator::{ArenaVec, GetAllocator};

/// Buffer the first list element to avoid growing a singleton arena allocation.
pub struct BufferedVec<'a, T> {
    first: Option<T>,
    vec: ArenaVec<'a, T>,
}

impl<'a, T> BufferedVec<'a, T> {
    #[inline]
    pub fn new_in(allocator: &impl GetAllocator<'a>) -> Self {
        Self { first: None, vec: ArenaVec::new_in(allocator) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        usize::from(self.first.is_some()) + self.vec.len()
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        if let Some(first) = self.first.take() {
            self.vec.extend([first, value]);
        } else if self.vec.is_empty() {
            self.first = Some(value);
        } else {
            self.vec.push(value);
        }
    }

    #[inline]
    pub fn into_vec(mut self) -> ArenaVec<'a, T> {
        self.vec.extend(self.first);
        self.vec
    }
}
