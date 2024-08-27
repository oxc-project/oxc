use std::{
    alloc::{self, handle_alloc_error, Layout},
    marker::PhantomData,
    mem::{align_of, size_of, MaybeUninit},
    ptr::{self, NonNull},
};

use crate::ancestor::{Ancestor, AncestorType};

// Must be a power of 2
const INITIAL_STACK_CAPACITY: usize = 64; // 64 entries = 640 bytes

// Max capacity. Largest power of two byte size is `(isize::MAX as usize + 1) / 2`.
// Doubling from that size would be `isize::MAX + 1` which is larger than maximum allocation
// size of `isize::MAX`.
const MAX_CAPACITY: usize = (isize::MAX as usize + 1) / 2 / size_of::<*const ()>();

/// Traverse ancestry context.
///
/// Contains a stack of `Ancestor`s, and provides methods to get parent/ancestor of current node.
///
/// `walk_*` methods push/pop `Ancestor`s to `stack` when entering/exiting nodes.
///
/// `Ancestor<'a, 't>` is an owned type.
/// * `'a` is lifetime of AST nodes.
/// * `'t` is lifetime of the `Ancestor` (derived from `&'t TraverseAncestry`).
///
/// `'t` is constrained in `parent`, `ancestor` and `ancestors` methods to only live as long as
/// the `&'t TraverseAncestry` passed to the method.
/// i.e. `Ancestor`s can only live as long as `enter_*` or `exit_*` method in which they're obtained,
/// and cannot "escape" those methods.
/// This is required for soundness. If an `Ancestor` could be retained longer, the references that
/// can be got from it could alias a `&mut` reference to the same AST node.
///
/// # SAFETY
/// This type MUST NOT be mutable by consumer.
///
/// The safety scheme is entirely reliant on `stack` being in sync with the traversal,
/// to prevent consumer from accessing fields of nodes which traversal has passed through,
/// so as to not violate Rust's aliasing rules.
/// If consumer could alter `stack` in any way, they could break the safety invariants and cause UB.
///
/// We prevent this in 3 ways:
/// 1. `TraverseAncestry`'s `stack` field is private.
/// 2. Public methods of `TraverseAncestry` provide no means for mutating `stack`.
/// 3. Visitors receive a `&mut TraverseCtx`, but cannot overwrite its `ancestry` field because they:
///    a. cannot create a new `TraverseAncestry` - `TraverseAncestry::new` is private.
///    b. cannot obtain an owned `TraverseAncestry` from a `&TraverseAncestry`
///       - `TraverseAncestry` is not `Clone`.
pub struct TraverseAncestry<'a> {
    len: usize,
    capacity: usize,
    types_ptr: NonNull<AncestorType>,
    ptrs_ptr: NonNull<*const ()>,
    _marker: PhantomData<&'a ()>,
}

// Public methods
impl<'a> TraverseAncestry<'a> {
    /// Get parent of current node.
    #[inline]
    pub fn parent<'t>(&'t self) -> Ancestor<'a, 't> {
        debug_assert!(self.len > 0);
        // SAFETY: Stack contains 1 entry initially. Entries are pushed as traverse down the AST,
        // and popped as go back up. So even when visiting `Program`, the initial entry is in the stack.
        // So `self.len - 1` cannot wrap around, and is always a valid index.
        // Function signature constrains `Ancestor`'s `'t` lifetime to lifetime of `&'t self`.
        // The `Ancestor` is guaranteed valid for `'t`. It is not possible to obtain a `&mut` ref
        // to any AST node which this `Ancestor` gives access to during `'t`.
        unsafe { self.get_unchecked(self.len - 1) }
    }

    /// Get ancestor of current node.
    ///
    /// `level` is number of levels above parent.
    /// `ancestor(0)` is equivalent to `parent()` (but better to use `parent()` as it's more efficient).
    ///
    /// If `level` is out of bounds (above `Program`), returns `Ancestor::None`.
    #[inline]
    pub fn ancestor<'t>(&'t self, level: usize) -> Ancestor<'a, 't> {
        // Behavior with different values:
        // `len = 1, level = 0` -> return `Ancestor::None` from else branch
        // `len = 1, level = 1` -> return `Ancestor::None` from else branch (out of bounds)
        // `len = 3, level = 0` -> return parent (index 2)
        // `len = 3, level = 1` -> return grandparent (index 1)
        // `len = 3, level = 2` -> return `Ancestor::None` from else branch
        // `len = 3, level = 3` -> return `Ancestor::None` from else branch (out of bounds)

        // `self.len` is always at least 1, so `self.len - 1` cannot wrap around.
        // `level <= last_index` would also work here, but `level < last_index` avoids a read from memory
        // when that read would just get `Ancestor::None` anyway.
        debug_assert!(self.len > 0);
        let last_index = self.len - 1;
        if level < last_index {
            // SAFETY: We just checked that `level < last_index` so `last_index - level` cannot wrap around,
            // and `last_index - level` must be a valid index.
            // Function signature constrains `Ancestor`'s `'t` lifetime to lifetime of `&'t self`.
            // The `Ancestor` is guaranteed valid for `'t`. It is not possible to obtain a `&mut` ref
            // to any AST node which this `Ancestor` gives access to during `'t`.
            unsafe { self.get_unchecked(last_index - level) }
        } else {
            Ancestor::None
        }
    }

    /// Get iterator over ancestors, starting with parent and working up.
    ///
    /// Last `Ancestor` returned will be `Program`. `Ancestor::None` is not included in iteration.
    pub fn ancestors<'t>(&'t self) -> Ancestors<'a, 't> {
        // SAFETY: `self.len` is always at least 1, which satisfies assumption in `Ancestors::next`.
        // Function signature constrains `Ancestors`'s `'t` lifetime to lifetime of `&'t self`.
        // The `Ancestor`s generated by `Ancestors`, will also have lifetime `'t`, and are
        // guaranteed valid for `'t`. It is not possible to obtain a `&mut` ref to any AST node which
        // this `Ancestors` gives access to during `'t`.
        debug_assert!(self.len > 0);
        Ancestors {
            index: self.len,
            types_ptr: self.types_ptr,
            ptrs_ptr: self.ptrs_ptr,
            _marker: PhantomData,
        }
    }

    /// Get depth in the AST.
    ///
    /// Count includes current node. i.e. in `Program`, depth is 1.
    #[inline]
    pub fn ancestors_depth(&self) -> usize {
        self.len
    }
}

/// Iterator over `Ancestor`s.
///
/// Returned by [`TraverseAncestry::ancestors`].
pub struct Ancestors<'a, 't> {
    // Last index that was read. Always greater than 0.
    index: usize,
    types_ptr: NonNull<AncestorType>,
    ptrs_ptr: NonNull<*const ()>,
    _marker: PhantomData<(&'a (), &'t ())>,
}

impl<'a, 't> Iterator for Ancestors<'a, 't> {
    type Item = Ancestor<'a, 't>;

    fn next(&mut self) -> Option<Self::Item> {
        // Index is always > 0 when `Ancestors` is created, and never drops below 1,
        // so `self.index - 1` cannot wrap around
        let next_index = self.index - 1;
        // Don't yield `Ancestor::None` at index 0
        if next_index > 0 {
            self.index = next_index;
            // SAFETY: `index` is in bounds when `Ancestors` is created, and is only decremented
            Some(unsafe { get_unchecked(next_index, self.types_ptr, self.ptrs_ptr) })
        } else {
            None
        }
    }
}

// Methods used internally within crate.
impl<'a> TraverseAncestry<'a> {
    /// Create new `TraverseAncestry`.
    ///
    /// # SAFETY
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    pub(super) fn new() -> Self {
        fn alloc<T>() -> NonNull<T> {
            // SAFETY: This is a valid layout
            let layout = unsafe {
                Layout::from_size_align_unchecked(
                    size_of::<T>() * INITIAL_STACK_CAPACITY,
                    align_of::<T>(),
                )
            };
            // SAFETY: Layout has non-zero size
            let ptr = unsafe { alloc::alloc(layout) }.cast::<T>();
            let Some(ptr) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };
            ptr
        }

        // If 1st allocation succeeded, but 2nd failed then 1st allocation will not get freed.
        // The program is crashing at this point, so I don't think a memory leak is a problem.
        // It's a leak, not UB.
        let types_ptr: NonNull<AncestorType> = alloc();
        let ptrs_ptr: NonNull<*const ()> = alloc();

        // SAFETY: We just allocated space for this, so write is in bounds
        unsafe { types_ptr.as_ptr().write(AncestorType::None) };

        Self { len: 1, capacity: INITIAL_STACK_CAPACITY, types_ptr, ptrs_ptr, _marker: PhantomData }
    }

    /// Get `Ancestor` at specified `index` of the stack, without bounds check.
    ///
    /// Ancestor's `'t` lifetime is constrained to `'t` of `&'t self`.
    ///
    /// # SAFETY
    /// * `index` must not be out of bounds i.e. must not be '>= self.len`.
    /// * Caller must ensure that the way returned `Ancestor` is used does not allow illegal aliasing.
    unsafe fn get_unchecked<'t>(&'t self, index: usize) -> Ancestor<'a, 't> {
        // SAFETY: Caller guarantees `index` is in bounds
        debug_assert!(index < self.len);
        get_unchecked(index, self.types_ptr, self.ptrs_ptr)
    }

    /// Push item onto ancestry stack.
    ///
    /// # SAFETY
    /// `ty` and `ptr` must correspond i.e. the 2 together form a valid `Ancestor`.
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    pub(crate) fn push_stack(&mut self, ty: AncestorType, ptr: *const ()) -> PopToken {
        // Grow if insufficient capacity
        if self.len == self.capacity {
            self.grow();
        }

        // SAFETY: We just ensured sufficient capacity. Caller promises `ty` and `ptr` are a valid pair.
        unsafe {
            self.types_ptr.as_ptr().add(self.len).write(ty);
            self.ptrs_ptr.as_ptr().add(self.len).write(ptr);
        }
        self.len += 1;

        // Return `PopToken` which can be used to pop this entry off again
        PopToken(())
    }

    // Grow allocations (double their size).
    #[cold]
    #[inline(never)]
    fn grow(&mut self) {
        unsafe fn realloc<T>(
            ptr: NonNull<T>,
            current_capacity: usize,
            new_capacity: usize,
        ) -> *mut T {
            let layout = Layout::from_size_align_unchecked(
                size_of::<T>() * current_capacity,
                align_of::<T>(),
            );
            let new_size = size_of::<T>() * new_capacity;
            alloc::realloc(ptr.as_ptr().cast::<u8>(), layout, new_size).cast::<T>()
        }

        unsafe fn handle_fail<T>(new_capacity: usize) {
            handle_alloc_error(Layout::from_size_align_unchecked(
                size_of::<T>() * new_capacity,
                align_of::<T>(),
            ));
        }

        let current_capacity = self.capacity;
        assert!(current_capacity < MAX_CAPACITY);
        let new_capacity = current_capacity * 2;

        // It's possible (but unlikely) that 1st allocation succeeds and 2nd fails.
        // If behaviour on `handle_alloc_error` is to panic rather than abort (not default behaviour),
        // this would cause the wrong capacity to be used when freeing 2nd allocation in `Drop`.
        // Handle this eventuality by setting `self.len` to 0 if this happens.
        // `Drop` impl will detect that signal and free half the stated capacity for 2nd allocation.
        // SAFETY: `capacity` and pointers are kept updated as the type grows.
        unsafe {
            let new_ptr = realloc(self.types_ptr, current_capacity, new_capacity);
            if let Some(new_ptr) = NonNull::new(new_ptr) {
                self.types_ptr = new_ptr;
                self.capacity = new_capacity;
            } else {
                handle_fail::<AncestorType>(new_capacity);
            }

            let new_ptr = realloc(self.ptrs_ptr, current_capacity, new_capacity);
            if let Some(new_ptr) = NonNull::new(new_ptr) {
                self.ptrs_ptr = new_ptr;
            } else {
                self.len = 0;
                handle_fail::<*const ()>(new_capacity);
            }
        }
    }

    /// Pop last item off ancestry stack.
    ///
    /// # SAFETY
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    #[allow(unused_variables, clippy::needless_pass_by_value, clippy::unnecessary_safety_comment)]
    pub(crate) unsafe fn pop_stack(&mut self, token: PopToken) {
        debug_assert!(self.len >= 2);
        // SAFETY: `PopToken`s are only created in `push_stack`, so the fact that caller provides one
        // guarantees that a push has happened. This method consumes the token which guarantees another
        // pop hasn't occurred already corresponding to that push.
        // Therefore the stack cannot by empty.
        // The stack starts with 1 entry, so also it cannot be left empty after this pop.
        self.len -= 1;
    }

    /// Retag last item on ancestry stack.
    ///
    /// i.e. Alter type of entry on top of stack, without changing the "payload" it contains
    /// of pointer to the ancestor node.
    ///
    /// This is purely a performance optimization. If the last item on stack already contains the
    /// correct pointer, then we only need to write the new type.
    ///
    /// `retag_stack` is only a single 2-byte write operation.
    ///
    /// # SAFETY
    /// * Stack must have length of at least 2 (so we are not retagging dummy root `Ancestor`).
    /// * Last item on stack must contain pointer to type corresponding to provided `AncestorType`.
    ///
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    #[allow(clippy::unnecessary_safety_comment)]
    pub(crate) unsafe fn retag_stack(&mut self, ty: AncestorType) {
        debug_assert!(self.len >= 2);
        // SAFETY: Caller guarantees stack has minimum length of 2, so this write is not out of bounds.
        // Caller also guarantees provided `AncestorType` matches pointer on stack.
        self.types_ptr.as_ptr().add(self.len - 1).write(ty);
    }
}

/// Get `Ancestor` for a specified index.
///
/// # SAFETY
/// Index must be in bounds.
/// Lifetimes of returned `Ancestor` are set by caller. They must ensure those lifetimes are sound.
#[allow(clippy::unnecessary_safety_comment)]
unsafe fn get_unchecked<'a, 't>(
    index: usize,
    types_ptr: NonNull<AncestorType>,
    ptrs_ptr: NonNull<*const ()>,
) -> Ancestor<'a, 't> {
    const _: () = {
        assert!(size_of::<Ancestor>() == size_of::<*const ()>() * 2);
        assert!(align_of::<Ancestor>() == align_of::<*const ()>());
    };

    // Create an `Ancestor` by writing the type (enum discriminant) and pointer separately,
    // after looking them up from the storage.
    // SAFETY: Caller guarantees `index` is in bounds.
    // These are the only bytes which require to be initialized in an `Ancestor`.
    // 6 bytes in middle can stay uninitialized.
    // The offset of fields is guaranteed by `Ancestor` being `#[repr(C, u16)]`
    // and the const assertions above.
    let mut ancestor = MaybeUninit::<Ancestor<'a, 't>>::uninit();
    let ancestor_ptr = ptr::addr_of_mut!(ancestor);
    let ty = *types_ptr.as_ptr().add(index).as_ref().unwrap_unchecked();
    ancestor_ptr.cast::<AncestorType>().write(ty);
    let ptr = *ptrs_ptr.as_ptr().add(index).as_ref().unwrap_unchecked();
    ancestor_ptr.cast::<*const ()>().add(1).write(ptr);
    ancestor.assume_init()
}

impl<'a> Drop for TraverseAncestry<'a> {
    fn drop(&mut self) {
        unsafe fn dealloc<T>(ptr: NonNull<T>, capacity: usize) {
            let layout =
                Layout::from_size_align_unchecked(size_of::<T>() * capacity, align_of::<T>());
            alloc::dealloc(ptr.as_ptr().cast::<u8>(), layout);
        }

        // SAFETY: This type always allocates (never has `capacity == 0`).
        // Pointers and capacity are kept updated as the type grows.
        unsafe {
            let mut capacity = self.capacity;
            dealloc(self.types_ptr, capacity);

            // If a reallocation succeeded for 1st allocation, but failed for 2nd, `self.len` is set to 0
            // to indicate that allocated size for 2nd allocation is half the size of `self.capacity`
            // (see `grow` method)
            if self.len == 0 {
                capacity /= 2;
            }
            dealloc(self.ptrs_ptr, capacity);
        }
    }
}

/// Zero sized token which allows popping from stack. Used to ensure push and pop always correspond.
/// Inner field is private to this module so can only be created by methods in this file.
/// It is not `Clone` or `Copy`, so no way to obtain one except in this file.
/// Only method which generates a `PopToken` is `push_stack`, and `pop_stack` consumes one,
/// which guarantees you can't have more pops than pushes.
pub(crate) struct PopToken(());
