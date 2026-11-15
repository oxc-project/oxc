//! Lookup tables which the SIMD implementations of `compress_blocks` use to turn
//! the token-start bitmap into a list of token positions, 16 bits at a time.

use std::{mem::offset_of, sync::OnceLock};

use oxc_simd::cpu_version;

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
/// That is used as a byte shuffle control to gather the kinds of those tokens, and added to
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
    #[cpu_version(Generic)]
    fn new() -> Box<Self> {
        // SAFETY: the tables are bytes, for which zero is a value.
        let mut luts = unsafe { Box::<Self>::new_zeroed().assume_init() };
        let PairLuts { lut0z, lutpad } = &mut *luts;

        for (m, (lut0z_line, lutpad_line)) in lut0z.iter_mut().zip(lutpad.iter_mut()).enumerate() {
            lutpad_line[8..16].fill(0x80);

            let mut k = 0usize;
            for bit in 0..8usize {
                if (m >> bit) & 1 != 0 {
                    lut0z_line[k] = bit as u8;
                    lutpad_line[k + 8] = (bit + 8) as u8;
                    k += 1;
                }
            }
        }

        luts
    }
}

/// Initialize the global [`PairLuts`] instance.
#[cpu_version(Generic)]
pub(super) fn init_pair_luts() {
    PAIR_LUTS.get_or_init(PairLuts::new);
}

/// Get the PairLuts, initializing them on first use.
#[cpu_version(Generic)]
#[inline]
pub(super) fn get_pair_luts() -> &'static PairLuts {
    PAIR_LUTS.get_or_init(PairLuts::new)
}
