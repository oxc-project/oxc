//! Lookup tables which `compress_blocks` uses to turn the token-start bitmap into a list of token positions,
//! 16 bits at a time.

use std::{
    mem::{MaybeUninit, offset_of},
    sync::OnceLock,
};

/// Single copy of [`PairLuts`], shared across all threads.
///
/// `Box<PairLuts>` not `PairLuts`, to avoid uninitialized `PairLuts` being stored
/// in the binary's data section, which would bloat binary by ~10 KB.
static PAIR_LUTS: OnceLock<Box<PairLuts>> = OnceLock::new();

/// Lookup tables for converting a 16-bit mask into a list of the offsets of its set bits.
///
/// `compress_blocks` splits each 16 bits of the token-start bitmap into 2 bytes,
/// `sub0` (low) and `sub1` (high), and looks each one up in its own table:
///
/// * `lut0z[sub0]` is the offsets of `sub0`'s set bits (0-7) in ascending order, padded with zeros.
/// * `lutpad[sub1]` is 32 bytes, made of 8 zeros, then the offsets of `sub1`'s set bits plus 8 (8-15)
///   in ascending order, padded with 0x80 up to byte 16, then 16 more zeros.
///
/// `compress_blocks` loads 16 bytes of `lutpad[sub1]` starting at byte `8 - pc0`,
/// where `pc0` is the number of bits set in `sub0`. This puts `pc0` zeros in front of `sub1`'s offsets,
/// so OR-ing it with `lut0z[sub0]` gives the offsets of all the set bits in the 16-bit mask, in order.
/// That is used as a `vpshufb` control to gather the kinds of those tokens, and added to
/// the block's base position to get their start positions.
///
/// The struct is aligned to 128 bytes, so it occupies the minimum number of pairs of 64-byte cache lines.
/// `lut0z` is a multiple of 64 bytes long, so each 32-byte row of `lutpad` sits within a 64-byte cache line.
/// The 16-byte loads from it never cross a cache line boundary.
#[repr(C, align(128))]
pub struct PairLuts {
    pub(super) lut0z: [[u8; 8]; 256],
    pub(super) lutpad: [[u8; 32]; 256],
}

/// Ensure `lutpad` field is aligned on a 64-byte boundary.
const _: () = assert!(offset_of!(PairLuts, lutpad).is_multiple_of(64));

impl PairLuts {
    /// Create [`PairLuts`] lookup tables.
    ///
    /// It's constructed directly on the heap, instead of creating a `PairLuts`
    /// and then passing it to `Box::new`. The latter results in 2 calls to `memset`
    /// to zero both arrays, then all the data being copied twice - first from
    /// 2 stack temporaries for `lut0z` and `lutpad` arrays into another stack temporary
    /// for `PairLuts`, then a second copy of `PairLuts` from the stack to the heap.
    fn new() -> Box<Self> {
        // Allocate space for `PairLuts` on the heap, uninitialized
        let mut luts = Box::<PairLuts>::new_uninit();

        // Get mut references to the fields of `PairLuts` as arrays of `MaybeUninit`s.
        // SAFETY: These references are valid as they're the same type as the struct fields.
        // Using `&raw mut` ensures no intermediate references are created to uninitialized data.
        let (lut0z, lutpad) = unsafe {
            unsafe fn uninit_array_mut<'a, const N: usize, T>(
                ptr: *mut [T; N],
            ) -> &'a mut [MaybeUninit<T>; N] {
                unsafe { ptr.cast::<[MaybeUninit<T>; N]>().as_mut().unwrap_unchecked() }
            }

            let ptr = luts.as_mut_ptr();
            let lut0z = uninit_array_mut(&raw mut (*ptr).lut0z);
            let lutpad = uninit_array_mut(&raw mut (*ptr).lutpad);
            (lut0z, lutpad)
        };

        for m in 0..256usize {
            let lut0z_line = lut0z[m].write([0; 8]);

            #[rustfmt::skip]
            let lutpad_line = lutpad[m].write([
                0,    0,    0,    0,    0,    0,    0,    0,
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
                0,    0,    0,    0,    0,    0,    0,    0,
                0,    0,    0,    0,    0,    0,    0,    0,
            ]);

            let mut k = 0usize;
            for bit in 0..8usize {
                if (m >> bit) & 1 != 0 {
                    lut0z_line[k] = bit as u8;
                    lutpad_line[k + 8] = (bit + 8) as u8;
                    k += 1;
                }
            }
        }

        // SAFETY: All bytes of both arrays are now initialized
        unsafe { luts.assume_init() }
    }
}

/// Initialize the global [`PairLuts`] instance.
///
/// This method must be called before calling [`get_pair_luts`].
pub fn init_pair_luts() {
    PAIR_LUTS.get_or_init(PairLuts::new);
}

/// Get reference to [`PairLuts`].
///
/// # SAFETY
///
/// [`init_pair_luts`] must have been called before calling this.
pub(super) unsafe fn get_pair_luts() -> &'static PairLuts {
    unsafe { PAIR_LUTS.get().unwrap_unchecked() }
}
