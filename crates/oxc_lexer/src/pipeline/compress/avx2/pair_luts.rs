//! Lookup tables which `compress_blocks` uses to turn the token-start bitmap into a list of token positions,
//! 16 bits at a time.

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
/// The struct is aligned to 64 bytes, and `lut0z` is a multiple of 64 bytes long,
/// so each 32-byte row of `lutpad` sits within a single cache line.
/// The 16-byte loads from it never cross a cache line boundary.
#[repr(C, align(64))]
pub struct PairLuts {
    pub(super) lut0z: [[u8; 8]; 256],
    pub(super) lutpad: [[u8; 32]; 256],
}

/// Ensure `lutpad` field is aligned on a 64-byte boundary.
const _: () = assert!(size_of::<[[u8; 8]; 256]>().is_multiple_of(64));

impl PairLuts {
    /// Create [`PairLuts`] lookup tables.
    pub fn new() -> Self {
        let mut lut0z = [[0; 8]; 256];
        let mut lutpad = [[0; 32]; 256];

        for m in 0..256usize {
            let mut k = 0usize;
            for bit in 0..8usize {
                if (m >> bit) & 1 != 0 {
                    lut0z[m][k] = bit as u8;
                    lutpad[m][8 + k] = (bit + 8) as u8;
                    k += 1;
                }
            }

            for j in k..8 {
                lutpad[m][8 + j] = 0x80;
            }
        }

        Self { lut0z, lutpad }
    }
}
