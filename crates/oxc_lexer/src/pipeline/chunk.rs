// Raw kernels: the unsafe fn boundary is the reviewed surface, not each operation inside.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]

use std::ptr;

use oxc_simd::{Neon, Vec128, VecData128, cpu_version};

/// Load 64 bits starting at byte `i` as a `u64`.
///
/// The load is unaligned, so neither `src` nor `i` has any alignment requirement.
///
/// # SAFETY
///
/// `src.add(i)` must be valid for reads of 8 bytes.
#[cpu_version(Generic)]
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
#[cpu_version(Generic)]
#[inline(always)]
pub fn eqm(x: u64, b: u8) -> u64 {
    let lo = 0x0101_0101_0101_0101u64;
    let low7 = 0x7F7F_7F7F_7F7F_7F7Fu64;
    let y = x ^ lo.wrapping_mul(b as u64);
    !((y & low7).wrapping_add(low7) | y | low7)
}

/// One bit per lane, which [`mm32`] and [`mm64`] sum into place.
///
/// The pattern repeats every 8 bytes, because each 8 lanes sum into 1 byte of the result.
pub(super) const LANE_BITS: VecData128 =
    VecData128::from_bytes([1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128]);

/// Gather the top bit of each byte of `a` then `b` into a 32-bit mask.
///
/// Every lane must be `0xFF` or `0x00`, or the lanes' sums carry into each other.
///
/// Neon has no `movemask`, so each lane is masked with its bit value,
/// and pairwise adds fold the lanes together until one `u32` holds the whole mask.
#[cpu_version(Neon)]
#[inline(always)]
pub fn mm32(a: Vec128<Neon>, b: Vec128<Neon>) -> u32 {
    let m = LANE_BITS.read(a.cpu());
    let s = (a & m).pairwise_add_u8(b & m);
    let s = s.pairwise_add_u8(s);
    s.pairwise_add_u8(s).get_u64::<0>() as u32
}

/// Gather the top bit of each byte of `a`, `b`, `c` then `d` into a 64-bit mask.
///
/// Every lane must be `0xFF` or `0x00`. See [`mm32`].
#[cpu_version(Neon)]
#[inline(always)]
pub fn mm64(a: Vec128<Neon>, b: Vec128<Neon>, c: Vec128<Neon>, d: Vec128<Neon>) -> u64 {
    let m = LANE_BITS.read(a.cpu());
    let ab = (a & m).pairwise_add_u8(b & m);
    let cd = (c & m).pairwise_add_u8(d & m);
    let s = ab.pairwise_add_u8(cd);
    s.pairwise_add_u8(s).get_u64::<0>()
}
