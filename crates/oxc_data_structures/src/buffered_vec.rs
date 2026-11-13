//! Vectors that defer allocation by buffering their first elements inline.

/// Backing storage for a [`BufferedVec`].
///
/// Implementations must preserve insertion order and follow [`Vec`] semantics for
/// reserving capacity, pushing, extending, and consuming elements.
pub trait BufferedVecBacking: IntoIterator {
    /// Whether the backing vector contains no elements.
    fn is_empty(&self) -> bool;

    /// Reserve capacity for at least `additional` more elements.
    fn reserve(&mut self, additional: usize);

    /// Reserve capacity without deliberately overallocating.
    fn reserve_exact(&mut self, additional: usize);

    /// Append an element.
    fn push(&mut self, value: Self::Item);

    /// Append all elements from an iterator.
    fn extend(&mut self, iter: impl IntoIterator<Item = Self::Item>);
}

/// Buffer the first `N` elements inline before using a backing vector.
///
/// Short lists only allocate when converted with [`Self::into_vec`]. Consuming
/// them through [`IntoIterator`] avoids allocating altogether. Longer lists spill
/// into the backing vector with capacity for at least `2 * N` elements.
pub struct BufferedVec<V: BufferedVecBacking, const N: usize = 4> {
    buffer: [Option<V::Item>; N],
    len: usize,
    vec: V,
}

impl<V: BufferedVecBacking, const N: usize> BufferedVec<V, N> {
    /// Create a buffer with an empty backing vector.
    ///
    /// # Panics
    /// Panics if `vec` is not empty.
    #[inline]
    pub fn new(vec: V) -> Self {
        assert!(vec.is_empty(), "BufferedVec requires an empty backing vector");
        Self { buffer: std::array::from_fn(|_| None), len: 0, vec }
    }

    /// Append an element, spilling into the backing vector when the buffer fills.
    #[inline]
    pub fn push(&mut self, value: V::Item) {
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

    /// Move all elements into the backing vector, reserving only the final length
    /// if the list still fits in the inline buffer.
    #[inline]
    pub fn into_vec(mut self) -> V {
        if self.len <= N {
            self.vec.reserve_exact(self.len);
            self.vec.extend(self.buffer.into_iter().flatten());
        }
        self.vec
    }
}

impl<V: BufferedVecBacking, const N: usize> IntoIterator for BufferedVec<V, N> {
    type Item = V::Item;
    type IntoIter =
        std::iter::Chain<std::iter::Flatten<std::array::IntoIter<Option<V::Item>, N>>, V::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter().flatten().chain(self.vec)
    }
}

impl<T> BufferedVecBacking for Vec<T> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }

    #[inline]
    fn reserve_exact(&mut self, additional: usize) {
        self.reserve_exact(additional);
    }

    #[inline]
    fn push(&mut self, value: T) {
        self.push(value);
    }

    #[inline]
    fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        Extend::extend(self, iter);
    }
}
