/// A macro that generates a struct-of-arrays (SoA) container backed by a single allocation.
///
/// Instead of N separate `IndexVec`s each with their own `ptr + len + capacity`,
/// this generates a struct with:
/// - A single allocation containing all field arrays contiguously
/// - A single `len` and `cap` (stored as `u32`)
/// - Per-field typed accessors with a single bounds check
/// - A single capacity check on `push`
///
/// This is modeled on Zig's `MultiArrayList`.
///
/// # Usage
///
/// ```ignore
/// multi_index_vec! {
///     pub struct ScopeTable<ScopeId> {
///         pub parent_ids => parent_ids_mut: Option<ScopeId>,
///         pub node_ids => node_ids_mut: NodeId,
///         pub flags => flags_mut: ScopeFlags,
///     }
/// }
/// ```
///
/// The `=> name_mut` part specifies the mutable accessor name (needed because
/// declarative macros cannot concatenate identifiers).
macro_rules! multi_index_vec {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident<$idx:ty> {
            $(
                $fvis:vis $fname:ident => $fname_mut:ident : $fty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            /// Pointer to the base of the single allocation.
            /// Layout: `[Field1 × cap][Field2 × cap]...` with alignment padding between fields.
            base: ::core::ptr::NonNull<u8>,
            /// Cached pointers to the start of each field's array within the allocation.
            $( $fname: ::core::ptr::NonNull<$fty>, )*
            /// Number of elements currently stored.
            len: u32,
            /// Total capacity of each field array.
            cap: u32,
        }

        // SAFETY: All data is fully owned by this struct. The raw pointers do not
        // alias with any other pointers. This is equivalent to Vec's Send/Sync impls
        // which are safe because Vec owns its data.
        unsafe impl Send for $name
        where
            $( $fty: Send, )*
        {}
        // SAFETY: Immutable access to the data through `&self` does not allow mutation.
        // Mutable access requires `&mut self` which guarantees exclusive access.
        unsafe impl Sync for $name
        where
            $( $fty: Sync, )*
        {}

        #[expect(clippy::allow_attributes, reason = "macro-generated methods may not all be used")]
        impl $name {
            /// Maximum number of elements this table can hold while still being
            /// representable by the index type.
            #[inline(always)]
            fn max_len() -> usize {
                <$idx as ::oxc_index::Idx>::MAX.saturating_add(1)
            }

            /// Compute the memory layout for a given capacity.
            /// Returns `None` if `cap == 0`.
            fn compute_layout(cap: usize) -> Option<::core::alloc::Layout> {
                if cap == 0 {
                    return None;
                }
                let mut size = 0usize;
                let mut max_align = 1usize;
                $(
                    let align = ::core::mem::align_of::<$fty>();
                    if align > max_align { max_align = align; }
                    size = (size + align - 1) & !(align - 1);
                    size += ::core::mem::size_of::<$fty>().checked_mul(cap).expect("capacity overflow");
                )*
                Some(::core::alloc::Layout::from_size_align(size, max_align).expect("invalid layout"))
            }

            /// Set cached field pointers based on the base allocation pointer and capacity.
            ///
            /// # Safety
            /// `base` must point to a valid allocation of the layout returned by
            /// `compute_layout(cap)`. The allocation must be at least as aligned as
            /// the maximum field alignment.
            unsafe fn set_pointers(&mut self, base: *mut u8, cap: usize) {
                let mut _offset = 0usize;
                $(
                    let align = ::core::mem::align_of::<$fty>();
                    _offset = (_offset + align - 1) & !(align - 1);
                    #[expect(clippy::cast_ptr_alignment)]
                    {
                        // SAFETY: `_offset` is within the allocation, and properly aligned
                        // for `$fty` because `compute_layout` aligns each field's offset,
                        // and the base pointer has at least `max_align` alignment.
                        self.$fname = unsafe {
                            ::core::ptr::NonNull::new_unchecked(
                                base.add(_offset).cast::<$fty>()
                            )
                        };
                    }
                    _offset += ::core::mem::size_of::<$fty>() * cap;
                )*
            }

            /// Create a new empty container.
            #[inline]
            $vis fn new() -> Self {
                Self {
                    base: ::core::ptr::NonNull::dangling(),
                    $( $fname: ::core::ptr::NonNull::dangling(), )*
                    len: 0,
                    cap: 0,
                }
            }

            /// Returns the number of elements.
            #[inline]
            $vis fn len(&self) -> usize {
                self.len as usize
            }

            /// Returns `true` if there are no elements.
            #[inline]
            $vis fn is_empty(&self) -> bool {
                self.len == 0
            }

            /// Reserve capacity for at least `additional` more elements.
            $vis fn reserve(&mut self, additional: usize) {
                let required = (self.len as usize).checked_add(additional).expect("capacity overflow");
                assert!(
                    required <= Self::max_len(),
                    "capacity exceeds index type range"
                );
                if required <= self.cap as usize {
                    return;
                }
                self.grow_to(required);
            }

            /// Grow the allocation to hold at least `min_cap` elements.
            fn grow_to(&mut self, min_cap: usize) {
                let max_len = Self::max_len();
                assert!(min_cap <= max_len, "capacity exceeds index type range");
                let new_cap = if self.cap == 0 {
                    min_cap.max(4)
                } else {
                    (self.cap as usize).checked_mul(2).expect("capacity overflow").max(min_cap)
                }
                .min(max_len);

                let old_cap = self.cap as usize;
                let old_len = self.len as usize;
                let new_layout = Self::compute_layout(new_cap)
                    .expect("new capacity must be > 0");

                // SAFETY: All pointer arithmetic is within bounds of the allocations.
                // Old data is copied field-by-field from the old allocation to the new one.
                // The old allocation is freed after the copy.
                unsafe {
                    let new_base = ::std::alloc::alloc(new_layout);
                    if new_base.is_null() {
                        ::std::alloc::handle_alloc_error(new_layout);
                    }

                    // Copy old data to new allocation.
                    if old_cap > 0 {
                        let old_base = self.base.as_ptr();
                        let mut _old_offset = 0usize;
                        let mut _new_offset = 0usize;
                        $(
                            let align = ::core::mem::align_of::<$fty>();
                            let elem_size = ::core::mem::size_of::<$fty>();
                            _old_offset = (_old_offset + align - 1) & !(align - 1);
                            _new_offset = (_new_offset + align - 1) & !(align - 1);
                            ::core::ptr::copy_nonoverlapping(
                                old_base.add(_old_offset),
                                new_base.add(_new_offset),
                                elem_size * old_len,
                            );
                            _old_offset += elem_size * old_cap;
                            _new_offset += elem_size * new_cap;
                        )*

                        let old_layout = Self::compute_layout(old_cap).unwrap();
                        ::std::alloc::dealloc(old_base, old_layout);
                    }

                    self.base = ::core::ptr::NonNull::new_unchecked(new_base);
                    self.set_pointers(new_base, new_cap);
                    self.cap = u32::try_from(new_cap).expect("capacity exceeds u32");
                }
            }

            /// Push a new element to all field arrays. Returns the index of the new element.
            $vis fn push(&mut self, $( $fname: $fty ),*) -> $idx {
                if self.len == self.cap {
                    self.grow_to(self.len as usize + 1);
                }
                let idx = self.len as usize;
                // SAFETY: `idx < cap` is guaranteed because we just grew if needed.
                // The field pointers are valid and properly aligned for their types.
                unsafe {
                    $(
                        self.$fname.as_ptr().add(idx).write($fname);
                    )*
                }
                self.len += 1;
                // SAFETY: `idx < len <= cap <= max_len = Idx::MAX + 1`, so `idx <= Idx::MAX`.
                unsafe { <$idx as ::oxc_index::Idx>::from_usize_unchecked(idx) }
            }

            /// Iterate over all valid indices.
            $vis fn iter_ids(&self) -> impl Iterator<Item = $idx> {
                let len = self.len as usize;
                // SAFETY: Valid indices are `0..len`, and by invariant `len <= Idx::MAX + 1`.
                (0..len).map(|i| unsafe { <$idx as ::oxc_index::Idx>::from_usize_unchecked(i) })
            }

            #[inline(always)]
            fn checked_idx(&self, id: $idx) -> usize {
                let idx = ::oxc_index::Idx::index(id);
                assert!(idx < self.len as usize, "index out of bounds");
                idx
            }

            // Per-field accessors.
            $(
                #[inline]
                $fvis fn $fname(&self, id: $idx) -> &$fty {
                    let idx = self.checked_idx(id);
                    // SAFETY: `idx` is validated by `checked_idx`.
                    // The pointer is aligned and valid for reads.
                    unsafe { &*self.$fname.as_ptr().add(idx) }
                }

                #[inline]
                #[allow(dead_code)]
                $fvis fn $fname_mut(&mut self, id: $idx) -> &mut $fty {
                    let idx = self.checked_idx(id);
                    // SAFETY: `idx` is validated by `checked_idx`.
                    // The pointer is aligned and valid. `&mut self` guarantees no aliasing.
                    unsafe { &mut *self.$fname.as_ptr().add(idx) }
                }
            )*
        }

        impl Clone for $name
        where
            $( $fty: Clone, )*
        {
            fn clone(&self) -> Self {
                let len = self.len as usize;
                let mut result = Self::new();
                if len > 0 {
                    result.reserve(len);
                    for i in 0..len {
                        // SAFETY: `i < len`, and `len <= max_len = Idx::MAX + 1`,
                        // so `i <= Idx::MAX`.
                        let id = unsafe { <$idx as ::oxc_index::Idx>::from_usize_unchecked(i) };
                        // `push` handles destination initialization and len updates,
                        // so this remains panic-safe if a field clone panics.
                        result.push(
                            $( self.$fname(id).clone() ),*
                        );
                    }
                }
                result
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                if self.cap == 0 {
                    return;
                }
                let len = self.len as usize;
                $(
                    if ::core::mem::needs_drop::<$fty>() {
                        for i in 0..len {
                            // SAFETY: `i < len <= cap`, so the pointer is within the allocation.
                            // Each element is only dropped once.
                            unsafe {
                                ::core::ptr::drop_in_place(self.$fname.as_ptr().add(i));
                            }
                        }
                    }
                )*
                let layout = Self::compute_layout(self.cap as usize).unwrap();
                // SAFETY: `self.base` was allocated with this layout.
                unsafe {
                    ::std::alloc::dealloc(self.base.as_ptr(), layout);
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("len", &self.len)
                    .field("cap", &self.cap)
                    .finish()
            }
        }
    };
}

pub(crate) use multi_index_vec;

#[cfg(test)]
mod tests {
    oxc_index::define_index_type! {
        struct TestId = usize;
    }

    multi_index_vec! {
        struct TestTable<TestId> {
            values => values_mut: u32,
        }
    }

    oxc_index::define_index_type! {
        struct SmallId = u8;
        MAX_INDEX = 3;
    }

    multi_index_vec! {
        struct SmallTable<SmallId> {
            values => values_mut: u32,
        }
    }

    oxc_index::define_index_type! {
        struct StringId = usize;
    }

    multi_index_vec! {
        struct StringTable<StringId> {
            values => values_mut: String,
        }
    }

    #[test]
    fn iter_ids_are_valid() {
        let mut table = TestTable::new();
        assert!(table.is_empty());
        table.reserve(3);
        table.push(1);
        table.push(2);
        table.push(3);
        assert_eq!(table.len(), 3);

        let ids = table.iter_ids().collect::<Vec<_>>();
        assert_eq!(ids.len(), 3);
        assert_eq!(usize::from(ids[0]), 0);
        assert_eq!(usize::from(ids[1]), 1);
        assert_eq!(usize::from(ids[2]), 2);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn accessor_panics_on_invalid_index() {
        let mut table = TestTable::new();
        table.push(1);
        let _ = table.values(TestId::from(1usize));
    }

    #[test]
    #[should_panic(expected = "capacity exceeds index type range")]
    fn reserve_panics_when_exceeding_index_range() {
        let mut table = SmallTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
        let _ = table.iter_ids().count();
        table.reserve(5);
    }

    #[test]
    #[should_panic(expected = "capacity exceeds index type range")]
    fn push_panics_when_exceeding_index_range() {
        let mut table = SmallTable::new();
        let id = table.push(1);
        assert_eq!(*table.values(id), 1);
        table.push(2);
        table.push(3);
        table.push(4);
        table.push(5);
    }

    #[test]
    fn clone_clones_non_copy_fields() {
        let mut table = StringTable::new();
        assert!(table.is_empty());
        table.reserve(1);
        let id = table.push(String::from("a"));
        assert_eq!(table.len(), 1);
        let _ = table.iter_ids().count();
        let mut clone = table.clone();

        table.values_mut(id).push('1');
        clone.values_mut(id).push('2');

        assert_eq!(table.values(id), "a1");
        assert_eq!(clone.values(id), "a2");
    }
}
