use std::{
    alloc::Layout,
    cell::Cell,
    hash::{BuildHasher, Hash},
    mem::MaybeUninit,
    ptr::NonNull,
    slice,
};

use crate::{Allocator, Box, HashMap, Vec};

/// Option to pass to [`CloneIn::clone_in_impl`] to determine how to clone semantic IDs.
///
/// * [`CloneInSemanticIds::With`] to clone by copying current value.
/// * [`CloneInSemanticIds::Without`] to clone by substituting a dummy value -
///   `NonMaxU32(0)` for a plain ID, `None` for an optional ID.
///
/// This is designed so that cloning semantic IDs is branchless and cheap.
/// See `SemanticId` trait in `oxc_syntax` crate.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CloneInSemanticIds {
    With = 0,
    Without = u32::MAX,
}

/// A trait to explicitly clone an object into an arena allocator.
///
/// As a convention `Cloned` associated type should always be the same as `Self`,
/// It'd only differ in the lifetime, Here's an example:
///
/// ```
/// # use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds, Vec};
/// # struct Struct<'a> {a: Vec<'a, u8>, b: u8}
///
/// impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Struct<'old_alloc> {
///     type Cloned = Struct<'new_alloc>;
///
///     fn clone_in_impl(
///         &self,
///         with_semantic_ids: CloneInSemanticIds,
///         allocator: &'new_alloc Allocator,
///     ) -> Self::Cloned {
///         Struct {
///             a: self.a.clone_in_impl(with_semantic_ids, allocator),
///             b: self.b.clone_in_impl(with_semantic_ids, allocator),
///         }
///     }
/// }
/// ```
///
/// Implementations of this trait on non-allocated items usually delegate to `Clone::clone`.
/// However, it **isn't** guaranteed.
pub trait CloneIn<'new_alloc>: Sized {
    /// The type of the cloned object.
    ///
    /// This should always be `Self` with a different lifetime.
    type Cloned;

    /// Clone `self` into the given `allocator`. `allocator` may be the same one
    /// that `self` is already in.
    // `#[inline(always)]` because it just delegates to `clone_in_impl`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        self.clone_in_impl(CloneInSemanticIds::Without, allocator)
    }

    /// Almost identical as `clone_in`, but for some special type, it will also clone the semantic ids.
    /// Please use this method only if you make sure semantic info is synced with the ast node.
    // `#[inline(always)]` because it just delegates to `clone_in_impl`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        self.clone_in_impl(CloneInSemanticIds::With, allocator)
    }

    /// Clone `self` into `allocator`, threading whether semantic ids should be preserved as a
    /// runtime `with_semantic_ids` flag rather than as two separate methods.
    ///
    /// This is the method that implementors provide.
    /// `clone_in` and `clone_in_with_semantic_ids` are thin wrappers around it.
    ///
    /// It exists so the `CloneIn` derive can emit a *single* recursive traversal that serves both:
    /// * `clone_in`: `with_semantic_ids == CloneInSemanticIds::Without`
    /// * `clone_in_with_semantic_ids`: `with_semantic_ids == CloneInSemanticIds::With`
    ///
    /// This is instead of monomorphizing two near-identical traversals over the whole AST.
    /// ID fields copy existing values when `with_semantic_ids == CloneInSemanticIds::With`,
    /// and reset to their default when `CloneInSemanticIds::Without`.
    ///
    /// Prefer calling `clone_in` or `clone_in_with_semantic_ids` at call sites —
    /// they name the intent and delegate here.
    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned;
}

impl<'alloc, T, C> CloneIn<'alloc> for Option<T>
where
    T: CloneIn<'alloc, Cloned = C>,
{
    type Cloned = Option<C>;

    #[inline]
    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'alloc Allocator,
    ) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in_impl(with_semantic_ids, allocator))
    }
}

impl<'new_alloc, T, C> CloneIn<'new_alloc> for Box<'_, T>
where
    T: CloneIn<'new_alloc, Cloned = C>,
{
    type Cloned = Box<'new_alloc, C>;

    #[inline]
    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Box::new_in(self.as_ref().clone_in_impl(with_semantic_ids, allocator), &allocator)
    }
}

impl<'new_alloc, T, C> CloneIn<'new_alloc> for Box<'_, [T]>
where
    T: CloneIn<'new_alloc, Cloned = C>,
{
    type Cloned = Box<'new_alloc, [C]>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        let ptr = clone_slice_in(self.as_ref(), with_semantic_ids, allocator);

        // SAFETY: `ptr` points to the cloned `[C]`, allocated in `allocator`'s arena.
        // The returned `Box`'s lifetime matches the `Allocator` the data was allocated in.
        unsafe { Box::from_non_null(ptr) }
    }
}

impl<'new_alloc, T, C> CloneIn<'new_alloc> for Vec<'_, T>
where
    T: CloneIn<'new_alloc, Cloned = C>,
    // TODO: This lifetime bound possibly shouldn't be required.
    // https://github.com/oxc-project/oxc/pull/9656#issuecomment-2719762898
    C: 'new_alloc,
{
    type Cloned = Vec<'new_alloc, C>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        let slice = self.as_slice();

        // Empty `Vec`s are common in ASTs. Short-circuit to skip making a zero-sized allocation.
        if slice.is_empty() {
            return Vec::new_in(&allocator);
        }

        let ptr = clone_slice_in(slice, with_semantic_ids, allocator);
        let len = slice.len();

        // Reconstruct a `Vec` owning the cloned `[C]`. Length and capacity are both `slice.len()`:
        // the allocation holds exactly the cloned elements, with no spare capacity.
        // SAFETY: `ptr` points to `len` initialized `C`s allocated in `allocator`'s arena,
        // valid for the returned `Vec`'s lifetime (tied to `allocator`).
        unsafe { Vec::from_raw_parts_in(ptr.cast::<C>(), len, len, &allocator) }
    }
}

/// Allocate space for a clone of `slice` in `allocator`'s arena, clone each item of `slice` into
/// it (via [`CloneIn::clone_in_impl`]), and return a pointer to the resulting initialized `[C]`.
///
/// Shared by the `Box<[T]>` and `Vec<T>` [`CloneIn`] impls - the only part not shared between them
/// is wrapping the returned pointer back up as a `Box` or `Vec`.
///
/// `#[inline]` so the compile-time layout check and the allocation const-fold into each caller,
/// and callers optimize around the returned pointer (e.g. the `Vec` impl's raw-parts reconstruction).
#[inline]
fn clone_slice_in<'new_alloc, T, C>(
    slice: &[T],
    with_semantic_ids: CloneInSemanticIds,
    allocator: &'new_alloc Allocator,
) -> NonNull<[C]>
where
    T: CloneIn<'new_alloc, Cloned = C>,
{
    // Compile-time check that `T` and `C` have identical size and alignment - which they always will
    // with intended usage that `T` and `C` are same types, just with different lifetimes.
    // This guarantees that layout of clone is same as layout of `slice`,
    // so we can create `Layout` with `for_value`, which has no runtime checks.
    const {
        assert!(
            size_of::<C>() == size_of::<T>() && align_of::<C>() == align_of::<T>(),
            "Size and alignment of `T` and `<T as CloneIn>::Cloned` must be the same"
        );
    }

    let layout = Layout::for_value(slice);

    let dst_ptr = allocator.alloc_layout(layout).cast::<MaybeUninit<C>>().as_ptr();

    // SAFETY: We allocated space for `slice.len()` items of type `C`, starting at `dst_ptr`.
    // `MaybeUninit<C>` has the same layout as `C`, so this is a valid view of that
    // (still uninitialized) memory region as a slice of `slice.len()` elements.
    let dst = unsafe { slice::from_raw_parts_mut(dst_ptr, slice.len()) };

    // Clone each item of `slice` into `dst`.
    // `C` isn't `Drop`, and allocation is in the arena, so we don't need to worry about a panic
    // in the loop - can't lead to a memory leak.
    clone_between_slices(slice, dst, with_semantic_ids, allocator);

    // `clone_between_slices` initialized every element of `dst`, so we can view it as `&mut [C]`,
    // reusing `dst`'s provenance rather than re-deriving a fresh slice from `dst_ptr`.
    // SAFETY: All `slice.len()` elements of `dst` were just initialized.
    let new_slice = unsafe { dst.assume_init_mut() };

    NonNull::from(new_slice)
}

/// Clone each item of `src` into `dst` (via [`CloneIn::clone_in_impl`]).
///
/// `src` and `dst` are expected to be the same length - only `src.len().min(dst.len())` items are cloned.
/// Callers pass equal-length slices, so on return every element of `dst` is initialized.
///
/// # Why an out-of-line function taking slices
///
/// The clone loop lives in this separate function, taking source and destination as slice *parameters*,
/// rather than being written inline in the callers. LLVM IR `noalias` is only emitted for reference-typed
/// function parameters (references created mid-function carry no aliasing information), and it survives
/// inlining as scoped-alias metadata - so this shape lets LLVM prove `src` and `dst` are disjoint.
///
/// For trivially-cloneable types, that collapses the loop to a single `memcpy`.
/// For types with real `CloneIn` impls, it enables vectorization without a runtime overlap check.
///
/// No drop guard is needed to guard against a panic mid-loop. `C` is never `Drop` (with intended
/// usage `C` is `T` with a different lifetime), and the destination is arena-allocated, so a panic
/// part-way through leaks nothing - see the callers' comments.
#[expect(clippy::inline_always)]
#[inline(always)] // To ensure compiler sees that `src.len()` and `dst.len()` are the same
fn clone_between_slices<'new_alloc, T, C>(
    src: &[T],
    dst: &mut [MaybeUninit<C>],
    with_semantic_ids: CloneInSemanticIds,
    allocator: &'new_alloc Allocator,
) where
    T: CloneIn<'new_alloc, Cloned = C>,
{
    for (src_item, dst_item) in src.iter().zip(dst.iter_mut()) {
        dst_item.write(src_item.clone_in_impl(with_semantic_ids, allocator));
    }
}

impl<'new_alloc, K, V, CK, CV, S> CloneIn<'new_alloc> for HashMap<'_, K, V, S>
where
    K: CloneIn<'new_alloc, Cloned = CK>,
    V: CloneIn<'new_alloc, Cloned = CV>,
    CK: Hash + Eq,
    S: Default + BuildHasher,
{
    type Cloned = HashMap<'new_alloc, CK, CV, S>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        // Keys in original hash map are guaranteed to be unique.
        // Unfortunately, we have no static guarantee that `CloneIn` maintains that uniqueness
        // - original keys (`K`) are guaranteed unique, but cloned keys (`CK`) might not be.
        // If we did have that guarantee, we could use the faster `insert_unique_unchecked` here.
        // `hashbrown::HashMap` also has a faster cloning method in its `Clone` implementation,
        // but those APIs are not exposed, and `Clone` doesn't support custom allocators.
        // So sadly this is a lot slower than it could be, especially for `Copy` types.
        let mut cloned = HashMap::with_capacity_in(self.len(), allocator);
        for (key, value) in self {
            cloned.insert(
                key.clone_in_impl(with_semantic_ids, allocator),
                value.clone_in_impl(with_semantic_ids, allocator),
            );
        }
        cloned
    }
}

impl<'alloc, T, C> CloneIn<'alloc> for Cell<T>
where
    T: Copy + CloneIn<'alloc, Cloned = C>,
{
    type Cloned = Cell<C>;

    #[inline]
    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'alloc Allocator,
    ) -> Self::Cloned {
        Cell::new(self.get().clone_in_impl(with_semantic_ids, allocator))
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for &str {
    type Cloned = &'new_alloc str;

    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        allocator.alloc_str(self)
    }
}

macro_rules! impl_clone_in {
    ($($t:ty)*) => {
        $(
            impl<'alloc> CloneIn<'alloc> for $t {
                type Cloned = Self;
                #[inline(always)]
                fn clone_in_impl(&self, _with_semantic_ids: CloneInSemanticIds, _: &'alloc Allocator) -> Self {
                    *self
                }
            }
        )*
    }
}

impl_clone_in! {
    usize u8 u16 u32 u64 u128
    isize i8 i16 i32 i64 i128
    f32 f64
    bool char
}

#[cfg(test)]
mod test {
    use super::{Allocator, CloneIn, HashMap, Vec};

    #[test]
    fn clone_in_boxed_slice() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut original = Vec::from_iter_in([1, 2, 3], &allocator).into_boxed_slice();

        let cloned = original.clone_in(allocator);
        let cloned2 = original.clone_in_with_semantic_ids(allocator);
        original[1] = 4;

        assert_eq!(original.as_ref(), &[1, 4, 3]);
        assert_eq!(cloned.as_ref(), &[1, 2, 3]);
        assert_eq!(cloned2.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn clone_in_empty_boxed_slice() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        // Exercises the zero-sized `alloc_layout` path in `clone_slice_in`
        let original = Vec::<u32>::new_in(&allocator).into_boxed_slice();

        let cloned = original.clone_in(allocator);
        let cloned2 = original.clone_in_with_semantic_ids(allocator);

        assert_eq!(cloned.as_ref(), &[] as &[u32]);
        assert_eq!(cloned2.as_ref(), &[] as &[u32]);
    }

    #[test]
    fn clone_in_vec() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut original = Vec::with_capacity_in(8, &allocator);
        original.extend_from_slice(&[1, 2, 3]);

        let cloned = original.clone_in(allocator);
        let cloned2 = original.clone_in_with_semantic_ids(allocator);
        original[1] = 4;

        assert_eq!(original.as_slice(), &[1, 4, 3]);
        assert_eq!(cloned.as_slice(), &[1, 2, 3]);
        assert_eq!(cloned.capacity(), 3);
        assert_eq!(cloned2.as_slice(), &[1, 2, 3]);
        assert_eq!(cloned2.capacity(), 3);
    }

    #[test]
    fn clone_in_empty_vec() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        // Exercises the `slice.is_empty()` short-circuit to `Vec::new_in`
        let original = Vec::<u32>::new_in(&allocator);

        let cloned = original.clone_in(allocator);
        let cloned2 = original.clone_in_with_semantic_ids(allocator);

        assert_eq!(cloned.as_slice(), &[] as &[u32]);
        assert_eq!(cloned.capacity(), 0);
        assert_eq!(cloned2.as_slice(), &[] as &[u32]);
        assert_eq!(cloned2.capacity(), 0);
    }

    #[test]
    fn clone_in_hash_map() {
        let allocator = Allocator::default();

        let mut original: HashMap<'_, &str, &str> = HashMap::with_capacity_in(8, &allocator);
        original.extend(&[("x", "xx"), ("y", "yy"), ("z", "zz")]);

        let cloned = original.clone_in(&allocator);
        let cloned2 = original.clone_in_with_semantic_ids(&allocator);
        *original.get_mut("y").unwrap() = "changed";

        let mut original_as_vec = original.iter().collect::<std::vec::Vec<_>>();
        original_as_vec.sort_unstable();
        assert_eq!(original_as_vec, &[(&"x", &"xx"), (&"y", &"changed"), (&"z", &"zz")]);

        assert_eq!(cloned.capacity(), 3);
        let mut cloned_as_vec = cloned.iter().collect::<std::vec::Vec<_>>();
        cloned_as_vec.sort_unstable();
        assert_eq!(cloned_as_vec, &[(&"x", &"xx"), (&"y", &"yy"), (&"z", &"zz")]);

        assert_eq!(cloned2.capacity(), 3);
        let mut cloned2_as_vec = cloned2.iter().collect::<std::vec::Vec<_>>();
        cloned2_as_vec.sort_unstable();
        assert_eq!(cloned2_as_vec, &[(&"x", &"xx"), (&"y", &"yy"), (&"z", &"zz")]);
    }
}
