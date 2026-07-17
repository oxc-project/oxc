//! Arena Box.
//!
//! Originally based on [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/ast/src/arena.rs)
//!
//! PROTOTYPE: `Box` is now a *compressed pointer*.
//!
//! All arena memory comes from a single process-global reserved address range
//! (the "cage" - see [`crate::cage`]). `Box` stores a 32-bit scaled offset from the cage
//! base (`NonZeroU32`) instead of a full 64-bit pointer:
//!
//! ```text
//! compressed = (address - cage_base) >> 3
//! address    = cage_base + (compressed << 3)
//! ```
//!
//! * `Box<T>` is 4 bytes. `Option<Box<T>>` is also 4 bytes (`NonZeroU32` niche).
//!   `#[repr(C, u8)]` AST enums with boxed payloads shrink from 16 to 8 bytes.
//! * The scale of 8 covers the whole 32 GiB cage with a `u32`, and requires all boxed
//!   allocations to be 8-byte-aligned. [`Box::new_in`] over-aligns allocations to 8 to
//!   guarantee this.
//! * `T` must be `Sized`. Boxed slices are a separate (uncompressed) type -
//!   [`crate::BoxedSlice`].

use std::{
    self,
    alloc::Layout,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU32,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::{GetAllocator, cage};

/// Compress a pointer to a cage offset.
///
/// # Panics
/// Panics if the pointer is not within the cage.
/// In debug builds, also asserts the pointer is 8-byte-aligned.
//
// `#[inline(always)]` because this is a hot path, and it's only a few instructions.
#[expect(clippy::inline_always)]
#[inline(always)]
fn compress<T>(ptr: NonNull<T>) -> NonZeroU32 {
    let offset = ptr.addr().get().wrapping_sub(cage::cage_base());
    debug_assert!(
        offset & ((1 << cage::COMPRESSED_SCALE_SHIFT) - 1) == 0,
        "pointer-compression: pointer is not 8-byte-aligned"
    );
    let scaled = offset >> cage::COMPRESSED_SCALE_SHIFT;
    // "scaled offset fits in `NonZeroU32`" is exactly equivalent to
    // "address is within the cage, and not at the (burned) cage base itself".
    // This is a real check (not `debug_assert!`) because a silently-wrong compressed pointer
    // would be later dereferenced = UB. It's a predictable branch; cost is negligible.
    match u32::try_from(scaled) {
        Ok(scaled) => match NonZeroU32::new(scaled) {
            Some(compressed) => compressed,
            None => compress_failed(),
        },
        Err(_) => compress_failed(),
    }
}

#[cold]
#[inline(never)]
fn compress_failed() -> ! {
    panic!(
        "pointer-compression: pointer is outside the cage. \
         Only memory allocated in an `oxc_allocator::Allocator` can be stored in a `Box`. \
         (`fixed_size` / `from_raw_parts` allocators are backed by memory outside the cage, \
         and are not supported by this prototype.)"
    )
}

/// Decompress a cage offset back to a pointer.
///
/// The returned pointer is valid if `compressed` was produced by [`compress`] from a valid
/// in-cage pointer. For [`Box::dangling`]'s sentinel value, the returned pointer is
/// within the cage reservation but must never be dereferenced.
//
// `#[inline(always)]` because this is a hot path (every `Box` deref), and it's only
// a load + shift + add.
#[expect(clippy::inline_always)]
#[inline(always)]
fn decompress<T>(compressed: NonZeroU32) -> NonNull<T> {
    let base = cage::cage_base_ptr();
    let offset = (compressed.get() as usize) << cage::COMPRESSED_SCALE_SHIFT;
    // SAFETY: `offset` is non-zero (compressed is `NonZeroU32`), so `base + offset` is non-null
    // even in the degenerate un-initialized-cage case (`base` is null then, but a `Box` can only
    // exist if something was allocated in the cage, which initializes it).
    // `offset < CAGE_SIZE` always, so the addition stays within the cage mapping.
    unsafe { NonNull::new_unchecked(base.add(offset).cast::<T>()) }
}

/// Layout for allocating a `T` in a `Box`, over-aligned to at least 8.
///
/// The compression scheme (scale 8) requires all boxed values to be 8-byte-aligned.
/// The arena's per-allocation minimum alignment is 1, so alignment must be requested here.
const fn boxed_layout<T>() -> Layout {
    let align = if align_of::<T>() > 8 { align_of::<T>() } else { 8 };
    match Layout::from_size_align(size_of::<T>(), align) {
        Ok(layout) => layout,
        Err(_) => panic!("invalid layout for boxed value"),
    }
}

/// A `Box` without [`Drop`], which stores its data in the arena allocator.
///
/// # Compressed pointer
///
/// `Box` is 4 bytes: a `NonZeroU32` scaled offset from the base of the process-global
/// memory "cage" which all arena memory is allocated from. See module docs.
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop). Memory is released in bulk
/// when the allocator is dropped, without dropping the individual objects in the arena.
///
/// Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
/// which own memory allocations outside the arena.
///
/// Static checks make this impossible to do. [`Box::new_in`] will refuse to compile if called
/// with a [`Drop`] type.
//
// Field is a concrete `NonZeroU32` (not a type projection), so `T` remains covariant,
// exactly as with the previous `NonNull<T>` representation.
#[repr(transparent)]
pub struct Box<'alloc, T>(NonZeroU32, PhantomData<(&'alloc (), T)>);

const _: () = {
    assert!(size_of::<Box<'static, u64>>() == 4);
    assert!(align_of::<Box<'static, u64>>() == 4);
    // `NonZeroU32` niche: `Option<Box<T>>` is no bigger than `Box<T>`
    assert!(size_of::<Option<Box<'static, u64>>>() == 4);
};

impl<T> Box<'_, T> {
    /// Const assertion that `T` is not `Drop`.
    /// Must be referenced in all methods which create a `Box`.
    const ASSERT_T_IS_NOT_DROP: () =
        assert!(!std::mem::needs_drop::<T>(), "Cannot create a Box<T> where T is a Drop type");
}

impl<'alloc, T> Box<'alloc, T> {
    /// Allocate `value` into the memory arena, and receive a [`Box`] which owns the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Box};
    ///
    /// let arena = Allocator::default();
    /// let arena = &arena;
    /// let in_arena: Box<i32> = Box::new_in(5, &arena);
    /// ```
    ///
    /// The `Box` cannot outlive the `Allocator`. This fails to compile:
    ///
    /// ```compile_fail
    /// use oxc_allocator::{Allocator, Box};
    ///
    /// let boxed = {
    ///     let allocator = Allocator::default();
    ///     let allocator = &allocator;
    ///     Box::new_in(5, &allocator)
    /// };
    /// assert_eq!(*boxed, 5);
    /// ```
    //
    // `#[inline(always)]` because this is a hot path and allocation is a very small function.
    // We always want it to be inlined.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn new_in<A: GetAllocator<'alloc>>(value: T, allocator: &A) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        if size_of::<T>() == 0 {
            // Zero-sized values are not stored anywhere; use a sentinel offset whose decompressed
            // address (`cage_base + max(align_of::<T>(), 8)`) is aligned for `T` (the cage base is
            // page-aligned). Reading a ZST from that address is a no-op.
            const {
                assert!(
                    size_of::<T>() != 0 || align_of::<T>() <= 4096,
                    "Cannot Box a zero-sized type with alignment > 4096"
                );
            }
            let align = align_of::<T>().max(8);
            #[expect(clippy::cast_possible_truncation)]
            let compressed = (align >> cage::COMPRESSED_SCALE_SHIFT) as u32;
            // SAFETY: `align >= 8`, so `compressed >= 1`
            let compressed = unsafe { NonZeroU32::new_unchecked(compressed) };
            return Self(compressed, PhantomData);
        }

        let allocator = allocator.allocator();
        let ptr = allocator.alloc_layout(const { boxed_layout::<T>() }).cast::<T>();
        // SAFETY: `ptr` is a fresh allocation of `size_of::<T>()` bytes, aligned for `T`.
        unsafe { ptr.write(value) };
        Self(compress(ptr), PhantomData)
    }

    /// Create a fake [`Box`] with a dangling "pointer" (sentinel offset).
    ///
    /// # SAFETY
    /// Safe to create, but must never be dereferenced, as does not point to a valid `T`.
    /// Only purpose is for mocking types without allocating for const assertions.
    pub const unsafe fn dangling() -> Self {
        Self(NonZeroU32::MAX, PhantomData)
    }

    /// Take ownership of the value stored in this [`Box`], consuming the box in
    /// the process.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Box};
    ///
    /// let arena = Allocator::default();
    /// let arena = &arena;
    ///
    /// // Put `5` into the arena and on the heap.
    /// let boxed: Box<i32> = Box::new_in(5, &arena);
    /// // Move it back to the stack. `boxed` has been consumed.
    /// let i = boxed.unbox();
    ///
    /// assert_eq!(i, 5);
    /// ```
    #[inline]
    pub fn unbox(self) -> T {
        // SAFETY:
        // This pointer read is safe because the pointer is guaranteed to be unique - not just now,
        // but we're guaranteed it's not borrowed from some other reference. This in turn is because
        // we never construct a `Box` with a borrowed reference, only with a fresh one just
        // allocated from an `Arena`.
        unsafe { ptr::read(decompress::<T>(self.0).as_ptr()) }
    }
}

impl<T> Box<'_, T> {
    /// Get a [`NonNull`] pointer pointing to the [`Box`]'s contents.
    ///
    /// The pointer is not valid for writes.
    ///
    /// The caller must ensure that the `Box` outlives the pointer this
    /// function returns, or else it will end up dangling.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Box};
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    /// let boxed = Box::new_in(123_u64, &allocator);
    /// let ptr = Box::as_non_null(&boxed);
    /// ```
    //
    // `#[inline(always)]` because this is only a few instructions
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn as_non_null(boxed: &Self) -> NonNull<T> {
        decompress(boxed.0)
    }

    /// Consume a [`Box`] and return a [`NonNull`] pointer to its contents.
    //
    // `#[inline(always)]` because this is only a few instructions
    #[expect(clippy::inline_always, clippy::needless_pass_by_value)]
    #[inline(always)]
    pub fn into_non_null(boxed: Self) -> NonNull<T> {
        decompress(boxed.0)
    }

    /// Create a [`Box`] from a [`NonNull`] pointer.
    ///
    /// # Panics
    ///
    /// Panics if the pointer does not point into the arena cage
    /// (i.e. into memory allocated by an [`Allocator`](crate::Allocator)).
    ///
    /// # SAFETY
    ///
    /// * Pointer must point to a valid `T`.
    /// * Pointer must point to within an `Allocator`.
    /// * Pointer must be 8-byte-aligned (all boxed allocations are - this is only a concern
    ///   for pointers which did not originate from a `Box`).
    /// * Caller must ensure that the pointer is valid for the lifetime of the `Box`.
    //
    // PROTOTYPE: No longer `const` - compression requires reading the runtime cage base.
    pub unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(compress(ptr), PhantomData)
    }
}

impl<T> Deref for Box<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: `self.0` always decompresses to a unique pointer, allocated from an `Arena`
        // in `Box::new_in`
        unsafe { decompress::<T>(self.0).as_ref() }
    }
}

impl<T> DerefMut for Box<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: `self.0` always decompresses to a unique pointer, allocated from an `Arena`
        // in `Box::new_in`
        unsafe { decompress::<T>(self.0).as_mut() }
    }
}

impl<T> AsRef<T> for Box<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for Box<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: Display> Display for Box<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T: Debug> Debug for Box<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

// Unused right now.
// impl<'alloc, T> PartialEq for Box<'alloc, T>
// where
// T: PartialEq<T> + ?Sized,
// {
// fn eq(&self, other: &Box<'alloc, T>) -> bool {
// PartialEq::eq(&**self, &**other)
// }
// }

#[cfg(feature = "serialize")]
impl<T: Serialize> Serialize for Box<'_, T> {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.deref().serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<T: ESTree> ESTree for Box<'_, T> {
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        self.deref().serialize(serializer);
    }
}

impl<T: Hash> Hash for Box<'_, T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

#[cfg(test)]
mod test {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use crate::Allocator;

    use super::Box;

    #[test]
    fn box_deref_mut() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut b = Box::new_in("x", &allocator);
        let b = &mut *b;
        *b = allocator.alloc("v");
        assert_eq!(*b, "v");
    }

    #[test]
    fn box_roundtrip() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in(123_u64, &allocator);
        assert_eq!(*b, 123);
        let b2 = Box::new_in([1u32, 2, 3], &allocator);
        assert_eq!(*b2, [1, 2, 3]);
        assert_eq!(*b, 123);
        assert_eq!(b.unbox(), 123);
        assert_eq!(b2.unbox(), [1, 2, 3]);
    }

    #[test]
    fn box_unbox() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in(3.14_f64, &allocator);
        assert!((b.unbox() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn box_sizes() {
        assert_eq!(size_of::<Box<'static, u64>>(), 4);
        assert_eq!(size_of::<Option<Box<'static, u64>>>(), 4);
        assert_eq!(size_of::<Box<'static, [u8; 1000]>>(), 4);
    }

    #[test]
    fn box_pointer_roundtrip() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in(0xABCD_u32, &allocator);
        let ptr = Box::as_non_null(&b);
        // Boxed allocations are 8-byte-aligned (required by compression scale)
        assert!(ptr.addr().get().is_multiple_of(8));
        // SAFETY: `ptr` points to a valid `u32` in the arena
        let b2 = unsafe { Box::from_non_null(ptr) };
        assert_eq!(*b2, 0xABCD);
    }

    #[test]
    fn box_zst() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in((), &allocator);
        #[expect(clippy::unit_cmp, clippy::let_unit_value)]
        {
            let unit = b.unbox();
            assert_eq!(unit, ());
        }
    }

    #[test]
    fn box_many_alloc_deref() {
        // Exercise growth across multiple chunks
        let allocator = Allocator::default();
        let allocator = &allocator;
        let boxes: std::vec::Vec<Box<u64>> =
            (0..10_000).map(|i| Box::new_in(i, &allocator)).collect();
        for (i, b) in boxes.iter().enumerate() {
            assert_eq!(**b, i as u64);
        }
    }

    #[test]
    fn box_debug() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in("x", &allocator);
        let b = format!("{b:?}");
        assert_eq!(b, "\"x\"");
    }

    #[test]
    fn box_hash() {
        fn hash(val: &impl Hash) -> u64 {
            let mut hasher = DefaultHasher::default();
            val.hash(&mut hasher);
            hasher.finish()
        }

        let allocator = Allocator::default();
        let allocator = &allocator;
        let a = Box::new_in("x", &allocator);
        let b = Box::new_in("x", &allocator);

        assert_eq!(hash(&a), hash(&b));
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn box_serialize() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in("x", &allocator);
        let s = serde_json::to_string(&b).unwrap();
        assert_eq!(s, r#""x""#);
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn box_serialize_estree() {
        use oxc_estree::{CompactSerializer, ESTree};

        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in("x", &allocator);

        let mut serializer = CompactSerializer::default();
        b.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(s, r#""x""#);
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_box_variant_lifetime<'a: 'b, 'b, T>(program: Box<'a, T>) -> Box<'b, T> {
            program
        }

        // `T` must also be covariant, so `Box<'a, Expression<'a>>` can be shortened
        fn _assert_box_variant_inner_lifetime<'a: 'b, 'b>(b: Box<'a, &'a u64>) -> Box<'b, &'b u64> {
            b
        }
    }
}
