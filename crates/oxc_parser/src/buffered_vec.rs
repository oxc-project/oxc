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

/// Collect a parser list with one exact allocation for up to four elements.
/// Longer lists spill directly into an arena vector with capacity eight.
#[inline]
pub fn collect_list<'a, T>(
    allocator: &impl GetAllocator<'a>,
    mut next: impl FnMut() -> Option<T>,
) -> ArenaVec<'a, T> {
    let Some(first) = next() else { return ArenaVec::new_in(allocator) };
    let Some(second) = next() else { return ArenaVec::from_value_in(first, allocator) };
    let Some(third) = next() else {
        return ArenaVec::from_array_in([first, second], allocator);
    };
    let Some(fourth) = next() else {
        return ArenaVec::from_array_in([first, second, third], allocator);
    };
    let Some(fifth) = next() else {
        return ArenaVec::from_array_in([first, second, third, fourth], allocator);
    };
    let mut list = ArenaVec::with_capacity_in(8, allocator);
    list.extend([first, second, third, fourth, fifth]);
    while let Some(element) = next() {
        list.push(element);
    }
    list
}
