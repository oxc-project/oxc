//! Tagged pointer for encoding a `u8` discriminant in unused high bits of a 64-bit pointer.
//!
//! On x86-64 and aarch64, user-space heap addresses use at most 47-48 bits,
//! leaving bits 48-55 available to store an 8-bit tag. This allows AST enum types
//! like `Expression` and `Statement` to shrink from 16 bytes (`#[repr(C, u8)]` enum
//! with a `Box` payload) to 8 bytes (a single tagged pointer).
//!
//! # Platform support
//!
//! This module is only available on 64-bit targets (`target_pointer_width = "64"`).
//! On 32-bit targets, the traditional `#[repr(C, u8)]` layout is used instead.
//!
//! # Strict provenance
//!
//! All pointer manipulation uses [`NonNull::map_addr`] (stable since Rust 1.75),
//! which preserves pointer provenance and is MIRI-clean.

use std::{marker::PhantomData, num::NonZeroUsize, ptr::NonNull};

/// Number of bits to shift the tag into the high bits of the pointer.
const TAG_SHIFT: u32 = 48;

/// Mask to extract the raw address from a tagged pointer (lower 48 bits).
const ADDR_MASK: usize = (1_usize << 48) - 1;

/// A tagged pointer that stores a `u8` discriminant in bits 48-55 of a 64-bit pointer.
///
/// This is the building block for compact AST enum representations. Instead of storing
/// the discriminant as a separate byte (which requires 7 bytes of padding alongside an
/// 8-byte pointer), the discriminant is encoded directly into unused pointer bits.
///
/// # Size
///
/// `TaggedPtr` is 8 bytes. `Option<TaggedPtr>` is also 8 bytes (null niche optimization).
///
/// # Safety invariants
///
/// - The underlying pointer (after masking off the tag) must point to a valid, aligned
///   allocation within an `Allocator` arena.
/// - The tag must be a valid discriminant for the enum type being represented.
/// - The type `T` used with [`as_ref`](TaggedPtr::as_ref) / [`as_mut`](TaggedPtr::as_mut)
///   must match the type that was originally stored.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TaggedPtr<'a>(NonNull<()>, PhantomData<&'a ()>);

impl<'a> TaggedPtr<'a> {
    /// Create a new tagged pointer from a discriminant and a pointer to an AST node.
    ///
    /// # Safety
    ///
    /// - `ptr` must point to a valid `T` allocated within an `Allocator`.
    /// - The raw address of `ptr` must fit in 48 bits (i.e., bits 48-63 must be zero).
    ///   This is guaranteed for all user-space heap allocations on x86-64 and aarch64.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn new<T>(discriminant: u8, ptr: NonNull<T>) -> Self {
        let addr = ptr.as_ptr() as usize;
        debug_assert!(
            addr & !ADDR_MASK == 0,
            "TaggedPtr: pointer uses high bits (address: {addr:#x}). \
             This platform may not be supported."
        );
        let tagged_addr = addr | ((discriminant as usize) << TAG_SHIFT);
        // SAFETY: `tagged_addr` is non-zero because `ptr` is non-null (lower bits are non-zero),
        // and we only OR in high bits. `map_addr` preserves provenance.
        let tagged_ptr =
            ptr.cast::<()>().map_addr(|_| unsafe { NonZeroUsize::new_unchecked(tagged_addr) });
        Self(tagged_ptr, PhantomData)
    }

    /// Extract the discriminant (tag) from the pointer.
    #[expect(clippy::inline_always, clippy::cast_possible_truncation)]
    #[inline(always)]
    pub fn discriminant(self) -> u8 {
        // Shift right by 48 bits to isolate the 8-bit tag in bits 48-55.
        // Truncation from usize to u8 is intentional -- we only stored 8 bits of tag.
        (self.0.as_ptr() as usize >> TAG_SHIFT) as u8
    }

    /// Extract the raw (untagged) pointer, cast to type `T`.
    ///
    /// # Safety
    ///
    /// The caller must ensure `T` matches the type that was originally stored.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn as_ptr<T>(self) -> NonNull<T> {
        self.0
            .map_addr(|addr| {
                // SAFETY: The original pointer was non-null, and masking off high bits
                // preserves the non-zero lower bits.
                unsafe { NonZeroUsize::new_unchecked(addr.get() & ADDR_MASK) }
            })
            .cast()
    }

    /// Get a shared reference to the pointed-to value.
    ///
    /// # Safety
    ///
    /// - `T` must match the type that was originally stored.
    /// - The pointed-to value must be valid for the lifetime `'a`.
    /// - Standard aliasing rules apply (no concurrent mutable references).
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn as_ref<T>(self) -> &'a T {
        // SAFETY: Caller guarantees `T` is correct and the pointer is valid for `'a`.
        unsafe { self.as_ptr::<T>().as_ref() }
    }

    /// Get a mutable reference to the pointed-to value.
    ///
    /// # Safety
    ///
    /// - `T` must match the type that was originally stored.
    /// - The pointed-to value must be valid for the lifetime `'a`.
    /// - The caller must have exclusive access (no other references exist).
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn as_mut<T>(&mut self) -> &'a mut T {
        // SAFETY: Caller guarantees `T` is correct, the pointer is valid, and access is exclusive.
        unsafe { self.as_ptr::<T>().as_mut() }
    }
}

// SAFETY: `TaggedPtr` is a raw pointer wrapper that does not own the data.
// Same as `NonNull`, `Send`/`Sync` are gated by the outer type (e.g. `Expression`).
unsafe impl Send for TaggedPtr<'_> {}
// SAFETY: See `Send` impl above.
unsafe impl Sync for TaggedPtr<'_> {}

#[cfg(test)]
mod tests {
    use std::mem;
    use std::ptr::NonNull;

    use super::*;

    #[test]
    fn size_assertions() {
        assert_eq!(mem::size_of::<TaggedPtr<'_>>(), 8);
        assert_eq!(mem::size_of::<Option<TaggedPtr<'_>>>(), 8);
    }

    #[test]
    fn round_trip_discriminant() {
        let value: u64 = 42;
        let ptr = NonNull::from(&value);

        for disc in [0_u8, 1, 50, 69, 127, 255] {
            // SAFETY: `ptr` points to a valid `u64` on the stack.
            let tagged = unsafe { TaggedPtr::new(disc, ptr) };
            assert_eq!(tagged.discriminant(), disc, "discriminant round-trip failed for {disc}");
        }
    }

    #[test]
    fn round_trip_pointer() {
        let value: u64 = 123_456;
        let ptr = NonNull::from(&value);

        // SAFETY: `ptr` points to a valid `u64`.
        let tagged = unsafe { TaggedPtr::new(7, ptr) };
        let recovered = unsafe { tagged.as_ptr::<u64>() };
        assert_eq!(recovered, ptr.cast(), "pointer round-trip failed");
    }

    #[test]
    fn as_ref_returns_correct_value() {
        let value: u32 = 0xDEAD_BEEF;
        let ptr = NonNull::from(&value);

        // SAFETY: `ptr` points to a valid `u32`.
        let tagged = unsafe { TaggedPtr::new(42, ptr) };
        let r = unsafe { tagged.as_ref::<u32>() };
        assert_eq!(*r, 0xDEAD_BEEF);
    }

    #[test]
    fn as_mut_can_modify_value() {
        let mut value: u32 = 1;
        let ptr = NonNull::from(&mut value);

        // SAFETY: `ptr` points to a valid `u32`, and we have exclusive access.
        let mut tagged = unsafe { TaggedPtr::new(0, ptr) };
        let r = unsafe { tagged.as_mut::<u32>() };
        *r = 2;
        assert_eq!(value, 2);
    }

    #[test]
    fn different_discriminants_same_pointer() {
        let value: u64 = 999;
        let ptr = NonNull::from(&value);

        // SAFETY: `ptr` is valid.
        let t1 = unsafe { TaggedPtr::new(10, ptr) };
        let t2 = unsafe { TaggedPtr::new(20, ptr) };

        assert_eq!(t1.discriminant(), 10);
        assert_eq!(t2.discriminant(), 20);

        let p1 = unsafe { t1.as_ptr::<u64>() };
        let p2 = unsafe { t2.as_ptr::<u64>() };
        assert_eq!(p1, p2);
    }

    #[test]
    fn copy_semantics() {
        let value: u64 = 0;
        let ptr = NonNull::from(&value);

        // SAFETY: `ptr` is valid.
        let tagged = unsafe { TaggedPtr::new(5, ptr) };
        let copied = tagged; // Copy
        assert_eq!(tagged.discriminant(), copied.discriminant());
    }

    #[test]
    fn with_arena_allocator() {
        use crate::Allocator;

        let allocator = Allocator::default();
        let boxed = crate::Box::new_in(42_u64, &allocator);
        let ptr = crate::Box::as_non_null(&boxed);

        // SAFETY: `ptr` points to a valid `u64` in the arena.
        let tagged = unsafe { TaggedPtr::new(7, ptr) };
        assert_eq!(tagged.discriminant(), 7);
        let r = unsafe { tagged.as_ref::<u64>() };
        assert_eq!(*r, 42);
    }

    /// Proof-of-concept: demonstrates that TaggedPtr can represent an Expression-like
    /// tagged enum in 8 bytes (vs 16 bytes for the current `#[repr(C, u8)]` enum).
    ///
    /// This test simulates the target design where:
    /// - Expression is a `#[repr(transparent)]` struct wrapping TaggedPtr
    /// - All 43+ variants (discriminants 0-50) are stored as tagged pointers
    /// - Pattern matching uses a `.kind()` view enum
    /// - Option<Expression> is also 8 bytes (null niche)
    #[test]
    fn proof_of_concept_expression_like_enum() {
        use crate::Allocator;

        // Simulate AST node types (simplified).
        // #[repr(C)] ensures span_start is at offset 0, matching real AST nodes.
        #[repr(C)]
        #[derive(Debug, PartialEq)]
        struct BooleanLiteral {
            span_start: u32,
            span_end: u32,
            value: bool,
        }

        #[repr(C)]
        #[derive(Debug, PartialEq)]
        struct NumericLiteral {
            span_start: u32,
            span_end: u32,
            value: f64,
        }

        #[repr(C)]
        #[derive(Debug, PartialEq)]
        struct Identifier {
            span_start: u32,
            span_end: u32,
            name_len: u32,
        }

        // The tagged expression type — 8 bytes!
        #[repr(transparent)]
        struct CompactExpression<'a>(TaggedPtr<'a>);

        // View enum for matching
        #[derive(Debug)]
        enum CompactExpressionKind<'a> {
            BooleanLiteral(&'a BooleanLiteral),
            NumericLiteral(&'a NumericLiteral),
            Identifier(&'a Identifier),
        }

        impl<'a> CompactExpression<'a> {
            fn boolean_literal(inner: crate::Box<'a, BooleanLiteral>) -> Self {
                Self(unsafe { TaggedPtr::new(0, crate::Box::into_non_null(inner)) })
            }

            fn numeric_literal(inner: crate::Box<'a, NumericLiteral>) -> Self {
                Self(unsafe { TaggedPtr::new(2, crate::Box::into_non_null(inner)) })
            }

            fn identifier(inner: crate::Box<'a, Identifier>) -> Self {
                Self(unsafe { TaggedPtr::new(7, crate::Box::into_non_null(inner)) })
            }

            fn kind(&self) -> CompactExpressionKind<'a> {
                unsafe {
                    match self.0.discriminant() {
                        0 => CompactExpressionKind::BooleanLiteral(self.0.as_ref()),
                        2 => CompactExpressionKind::NumericLiteral(self.0.as_ref()),
                        7 => CompactExpressionKind::Identifier(self.0.as_ref()),
                        _ => unreachable!(),
                    }
                }
            }

            fn is_identifier(&self) -> bool {
                self.0.discriminant() == 7
            }

            fn as_identifier(&self) -> Option<&'a Identifier> {
                if self.0.discriminant() == 7 {
                    Some(unsafe { self.0.as_ref() })
                } else {
                    None
                }
            }

            /// Optimized GetSpan — single pointer deref, no match needed.
            /// All AST nodes have span at offset 0.
            fn span_start(&self) -> u32 {
                // SAFETY: All variant types have span_start: u32 at offset 0.
                unsafe { *self.0.as_ptr::<u32>().as_ref() }
            }
        }

        // === Size assertions ===
        assert_eq!(mem::size_of::<CompactExpression>(), 8, "Expression should be 8 bytes");
        assert_eq!(
            mem::size_of::<Option<CompactExpression>>(),
            8,
            "Option<Expression> should be 8 bytes (null niche)"
        );

        // For comparison: the current 16-byte representation
        #[repr(C, u8)]
        #[allow(dead_code)]
        enum OldExpression<'a> {
            BooleanLiteral(crate::Box<'a, BooleanLiteral>) = 0,
            NumericLiteral(crate::Box<'a, NumericLiteral>) = 2,
            Identifier(crate::Box<'a, Identifier>) = 7,
        }
        assert_eq!(mem::size_of::<OldExpression>(), 16, "Old Expression is 16 bytes");

        // === Functionality tests ===
        let allocator = Allocator::default();

        // Create expressions
        let bool_expr = CompactExpression::boolean_literal(crate::Box::new_in(
            BooleanLiteral { span_start: 0, span_end: 4, value: true },
            &allocator,
        ));
        let num_expr = CompactExpression::numeric_literal(crate::Box::new_in(
            NumericLiteral { span_start: 10, span_end: 12, value: 42.0 },
            &allocator,
        ));
        let id_expr = CompactExpression::identifier(crate::Box::new_in(
            Identifier { span_start: 20, span_end: 23, name_len: 3 },
            &allocator,
        ));

        // Pattern matching via kind()
        match bool_expr.kind() {
            CompactExpressionKind::BooleanLiteral(lit) => {
                assert!(lit.value);
                assert_eq!(lit.span_start, 0);
            }
            _ => panic!("expected BooleanLiteral"),
        }

        match num_expr.kind() {
            CompactExpressionKind::NumericLiteral(lit) => {
                assert_eq!(lit.value, 42.0);
            }
            _ => panic!("expected NumericLiteral"),
        }

        // is_* and as_* accessors
        assert!(id_expr.is_identifier());
        assert!(!id_expr.is_identifier() && false || id_expr.is_identifier());
        assert_eq!(id_expr.as_identifier().unwrap().name_len, 3);
        assert!(bool_expr.as_identifier().is_none());

        // Optimized span access (single deref, no match)
        assert_eq!(bool_expr.span_start(), 0);
        assert_eq!(num_expr.span_start(), 10);
        assert_eq!(id_expr.span_start(), 20);

        // Option<CompactExpression> is also 8 bytes
        let some_expr: Option<CompactExpression> = Some(CompactExpression::boolean_literal(
            crate::Box::new_in(
                BooleanLiteral { span_start: 0, span_end: 1, value: false },
                &allocator,
            ),
        ));
        assert!(some_expr.is_some());
        let none_expr: Option<CompactExpression> = None;
        assert!(none_expr.is_none());
    }
}
