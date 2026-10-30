use std::ptr;

/// Load 64 bits starting at byte `i` as a `u64`.
///
/// The load is unaligned, so neither `src` nor `i` has any alignment requirement.
///
/// # SAFETY
///
/// `src.add(i)` must be valid for reads of 8 bytes.
#[inline(always)]
pub unsafe fn load64(src: *const u8, i: usize) -> u64 {
    ptr::read_unaligned(src.add(i) as *const u64)
}

/// Compare every byte of chunk `x` against `b`.
///
/// Returns a `u64` where each of the 8 bytes is `0x80`
/// where the byte in `x` equals `b`, and `0x00` where it does not.
///
/// On a little-endian target, `trailing_zeros() >> 3` gives the offset of the first match.
/// This is the SWAR counterpart of `Vec256::eq_u8` followed by `Vec256::movemask`.
#[inline(always)]
pub fn eqm(x: u64, b: u8) -> u64 {
    let lo = 0x0101_0101_0101_0101u64;
    let low7 = 0x7F7F_7F7F_7F7F_7F7Fu64;
    let y = x ^ lo.wrapping_mul(b as u64);
    !((y & low7).wrapping_add(low7) | y | low7)
}
