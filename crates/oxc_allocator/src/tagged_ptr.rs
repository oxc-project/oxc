//! Tagged pointer packing a payload pointer and a `u8` tag into 8 bytes.
//!
//! The stored address is `(payload_addr << 8) | tag` - the tag occupies the low byte,
//! and the payload address is recovered by shifting right 8 bits.
//! This requires payload addresses to be less than 2^56, which holds on all current
//! 64-bit platforms (x86-64 and AArch64 use at most 52-bit / 48-bit virtual addresses).
//!
//! Note: This is a 64-bit-only prototype. There is no 32-bit fallback.

// All methods are no-ops or 1-2 instructions
#![expect(clippy::inline_always)]

use std::{fmt, marker::PhantomData, num::NonZeroUsize, ptr::NonNull};

/// A tagged pointer: a [`NonNull`] payload pointer and a `u8` tag packed into 8 bytes.
///
/// Encoding: stored address = `(payload_addr << 8) | tag`.
///
/// Because the payload address is non-null, the shifted stored value is never 0,
/// so [`NonNull`] gives `Option<TaggedPtr>` (and types wrapping `TaggedPtr`) an 8-byte niche.
///
/// The lifetime `'a` ties the tagged pointer to the arena allocation it points into.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TaggedPtr<'a> {
    ptr: NonNull<u8>,
    _marker: PhantomData<&'a ()>,
}

impl TaggedPtr<'_> {
    /// Create a new [`TaggedPtr`] from a tag and a payload pointer.
    ///
    /// # SAFETY
    ///
    /// * `ptr`'s address must be less than 2^56 (checked with a `debug_assert!`).
    /// * Callers which later dereference the pointer returned by [`ptr`](Self::ptr) are
    ///   responsible for ensuring `ptr` is valid for the payload type they cast it to.
    #[inline(always)]
    pub unsafe fn new(tag: u8, ptr: NonNull<u8>) -> Self {
        debug_assert!(
            ptr.addr().get() < (1_usize << 56),
            "TaggedPtr payload address must be < 2^56"
        );
        let ptr = ptr.map_addr(|addr| {
            // SAFETY: `addr` is non-zero (from `NonNull`) and `< 2^56` (caller guarantee),
            // so `addr << 8` is non-zero. `| tag` only sets low bits, so result is non-zero.
            unsafe { NonZeroUsize::new_unchecked((addr.get() << 8) | usize::from(tag)) }
        });
        Self { ptr, _marker: PhantomData }
    }

    /// Get the tag (the low byte of the stored address).
    #[inline(always)]
    pub fn tag(self) -> u8 {
        #[expect(clippy::cast_possible_truncation)]
        {
            self.ptr.addr().get() as u8
        }
    }

    /// Get the payload pointer (stored address shifted right by 8 bits).
    ///
    /// Uses [`NonNull::map_addr`], so the returned pointer retains the provenance
    /// of the pointer passed to [`new`](Self::new).
    #[inline(always)]
    pub fn ptr(self) -> NonNull<u8> {
        self.ptr.map_addr(|addr| {
            // SAFETY: Stored address is `payload_addr << 8 | tag`, where `payload_addr`
            // is non-zero, so `addr >> 8` is non-zero.
            unsafe { NonZeroUsize::new_unchecked(addr.get() >> 8) }
        })
    }
}

impl fmt::Debug for TaggedPtr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaggedPtr").field("tag", &self.tag()).field("ptr", &self.ptr()).finish()
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use super::TaggedPtr;

    #[test]
    fn round_trip() {
        // Real allocation, so the pointer is a genuine heap address
        let boxed = Box::new(0x1234_5678_9ABC_DEF0_u64);
        let ptr = NonNull::from(&*boxed).cast::<u8>();

        for tag in [0_u8, 1, 50, 255] {
            // SAFETY: `ptr` is a valid heap address, below 2^56 on all supported platforms
            let tagged = unsafe { TaggedPtr::new(tag, ptr) };
            assert_eq!(tagged.tag(), tag);
            assert_eq!(tagged.ptr(), ptr);
            // Dereference through the recovered pointer to prove provenance is intact
            // SAFETY: recovered pointer is the original `ptr`, pointing to a live `u64`
            let value = unsafe { *tagged.ptr().cast::<u64>().as_ref() };
            assert_eq!(value, 0x1234_5678_9ABC_DEF0_u64);
        }
    }

    #[test]
    fn copy_and_debug() {
        let boxed = Box::new(42_u8);
        let ptr = NonNull::from(&*boxed);
        // SAFETY: `ptr` is a valid heap address, below 2^56 on all supported platforms
        let tagged = unsafe { TaggedPtr::new(7, ptr) };
        let copy = tagged;
        assert_eq!(copy.tag(), tagged.tag());
        assert_eq!(copy.ptr(), tagged.ptr());
        let s = format!("{tagged:?}");
        assert!(s.contains("tag: 7"));
    }

    #[test]
    fn size_and_niche() {
        assert_eq!(size_of::<TaggedPtr<'_>>(), 8);
        assert_eq!(size_of::<Option<TaggedPtr<'_>>>(), 8);
    }
}
