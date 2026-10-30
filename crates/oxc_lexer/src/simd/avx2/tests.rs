use std::{array, mem::transmute};

use crate::{
    cpu::DynCpu,
    simd::{
        operand::Operand128,
        tests::{map2, pattern},
    },
};

use super::{Vec128, Vec256};

/// Wrapper to align a value on 32.
#[repr(align(32))]
struct Aligned<T>(T);

/// Transform an array of bytes as if it was an array of `u16`s.
fn map_u16(bytes: [u8; 32], f: impl Fn(u16) -> u16) -> [u8; 32] {
    let mut out = [0; 32];
    for i in (0..32).step_by(2) {
        let value = f(u16::from_le_bytes([bytes[i], bytes[i + 1]]));
        out[i..i + 2].copy_from_slice(&value.to_le_bytes());
    }
    out
}

/// Get the scalar equivalent of `movemask`.
fn top_bits(bytes: &[u8]) -> u32 {
    bytes.iter().enumerate().fold(0, |mask, (i, byte)| mask | (u32::from(byte >> 7) << i))
}

/// Get the scalar equivalent of `shuffle_u8s` on 16 bytes.
fn shuffle(bytes: &[u8], control: &[u8]) -> Vec<u8> {
    let select = |&c: &u8| if c & 0x80 == 0 { bytes[(c & 0x0F) as usize] } else { 0 };
    control.iter().map(select).collect()
}

fn to_bytes256(v: Vec256) -> [u8; 32] {
    let mut out = [0; 32];
    // SAFETY: `out` is valid for writing 32 bytes
    unsafe { v.store_unaligned(out.as_mut_ptr()) };
    out
}

fn to_u32s(v: Vec256) -> [u32; 8] {
    let mut out = [0; 8];
    // SAFETY: `out` is valid for writing 32 bytes
    unsafe { v.store_unaligned(out.as_mut_ptr().cast::<u8>()) };
    out
}

fn to_bytes128(v: Vec128) -> [u8; 16] {
    let mut out = [0; 16];
    // SAFETY: `out` is valid for writing 16 bytes
    unsafe { v.store_unaligned(out.as_mut_ptr()) };
    out
}

#[test]
fn vec256_create() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    assert_eq!(to_bytes256(Vec256::zero(cpu)), [0; 32]);
    assert_eq!(to_bytes256(Vec256::zero(Vec256::zero(cpu).cpu())), [0; 32]);
    assert_eq!(to_bytes256(Vec256::splat(0x85, cpu)), [0x85; 32]);
    assert_eq!(to_bytes256(Vec256::splat(0x85, cpu).eq_u8(0x85)), [0xFF; 32]);
    assert_eq!(to_u32s(Vec256::splat_u32(0x8234_5678, cpu)), [0x8234_5678; 8]);

    let bytes = pattern(37, 11);
    assert_eq!(to_bytes256(Vec256::read_unaligned(&bytes, cpu)), bytes);

    let values = [1, 2, 3, 0x8000_0000, 5, 6, 7, u32::MAX];
    assert_eq!(to_u32s(Vec256::read_unaligned(&values, cpu)), values);
}

#[test]
fn vec256_from_vec128() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 16] = pattern(37, 11);
    let v = Vec128::read_unaligned(&bytes, cpu);

    let splatted = to_bytes256(Vec256::splat_v128(v));
    assert_eq!(splatted[..16], bytes);
    assert_eq!(splatted[16..], bytes);

    let expected: [u32; 8] = array::from_fn(|i| u32::from(bytes[i]));
    assert_eq!(to_u32s(Vec256::widen_u8_to_u32(v)), expected);
}

#[test]
fn vec256_read_unaligned() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 32] = pattern(37, 11);
    // SAFETY: These types are all 32 bytes, and all bit patterns are valid for them
    let vectors = unsafe {
        [
            Vec256::read_unaligned(&bytes, cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [i8; 32]>(bytes), cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [u16; 16]>(bytes), cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [i16; 16]>(bytes), cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [u32; 8]>(bytes), cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [i32; 8]>(bytes), cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [u64; 4]>(bytes), cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [i64; 4]>(bytes), cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [u128; 2]>(bytes), cpu),
            Vec256::read_unaligned(&transmute::<[u8; 32], [i128; 2]>(bytes), cpu),
        ]
    };
    for v in vectors {
        assert_eq!(to_bytes256(v), bytes);
    }
}

#[test]
fn vec256_load_store() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let src = Aligned(pattern::<32>(37, 11));
    let mut dest = Aligned([0; 32]);
    // SAFETY: `src` and `dest` are 32 bytes and aligned on 32
    unsafe { Vec256::load(src.0.as_ptr(), cpu).store(dest.0.as_mut_ptr()) };
    assert_eq!(dest.0, src.0);

    let src = pattern::<34>(37, 11);
    let mut dest = [0; 34];
    // SAFETY: `src` and `dest` are valid for 32 bytes starting at offset 1
    unsafe {
        let v = Vec256::load_unaligned(src.as_ptr().add(1), cpu);
        v.store_unaligned(dest.as_mut_ptr().add(1));
    }
    assert_eq!(dest[1..33], src[1..33]);
    assert_eq!((dest[0], dest[33]), (0, 0));

    let values = [1, u64::MAX, 0x0102_0304_0506_0708, 1 << 63];
    let bytes = to_bytes256(Vec256::read_unaligned(&values, cpu));
    assert_eq!(bytes[..], *values.map(u64::to_le_bytes).as_flattened());
}

#[test]
fn vec256_bitwise() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec256::read_unaligned(&a, cpu), Vec256::read_unaligned(&b, cpu));

    assert_eq!(to_bytes256(va & vb), map2(a, b, |x, y| x & y));
    assert_eq!(to_bytes256(va | vb), map2(a, b, |x, y| x | y));
    assert_eq!(to_bytes256(va ^ vb), map2(a, b, |x, y| x ^ y));
    assert_eq!(to_bytes256(!va), a.map(|x| !x));
    assert_eq!(to_bytes256(va & 0x3C), a.map(|x| x & 0x3C));
    assert_eq!(to_bytes256(va | 0x3C), a.map(|x| x | 0x3C));
    assert_eq!(to_bytes256(va ^ 0x3C), a.map(|x| x ^ 0x3C));

    let mut v = va;
    v &= vb;
    assert_eq!(to_bytes256(v), map2(a, b, |x, y| x & y));

    let mut v = va;
    v |= vb;
    assert_eq!(to_bytes256(v), map2(a, b, |x, y| x | y));

    let mut v = va;
    v ^= vb;
    assert_eq!(to_bytes256(v), map2(a, b, |x, y| x ^ y));

    let mut v = va;
    v &= 0x3C;
    assert_eq!(to_bytes256(v), a.map(|x| x & 0x3C));

    let mut v = va;
    v |= 0x3C;
    assert_eq!(to_bytes256(v), a.map(|x| x | 0x3C));

    let mut v = va;
    v ^= 0x3C;
    assert_eq!(to_bytes256(v), a.map(|x| x ^ 0x3C));
}

#[test]
fn vec256_is_zero() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    assert!(Vec256::zero(cpu).is_zero());
    assert!(!Vec256::splat(0xFF, cpu).is_zero());

    // Check every byte, with only the lowest or highest bit set
    for i in 0..32 {
        for bit in [0x01, 0x80] {
            let mut bytes = [0u8; 32];
            bytes[i] = bit;
            assert!(!Vec256::read_unaligned(&bytes, cpu).is_zero());
        }
    }
}

#[test]
fn vec256_add_u32() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let a: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, u32::MAX];
    let b: [u32; 8] = [10, 20, 30, 0x1_0000, 50, 60, 70, 2];
    let sum = Vec256::read_unaligned(&a, cpu).add_u32(Vec256::read_unaligned(&b, cpu));
    assert_eq!(to_u32s(sum), [11, 22, 33, 0x1_0004, 55, 66, 77, 1]);

    let sum = Vec256::read_unaligned(&a, cpu).add_u32(16);
    assert_eq!(to_u32s(sum), [17, 18, 19, 20, 21, 22, 23, 15]);
}

#[test]
fn vec256_add_sub() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec256::read_unaligned(&a, cpu), Vec256::read_unaligned(&b, cpu));

    assert_eq!(to_bytes256(va.add_u8(vb)), map2(a, b, u8::wrapping_add));
    assert_eq!(to_bytes256(va.saturating_add_u8(vb)), map2(a, b, u8::saturating_add));
    assert_eq!(to_bytes256(va.sub_u8(vb)), map2(a, b, u8::wrapping_sub));
    assert_eq!(to_bytes256(va.saturating_sub_u8(vb)), map2(a, b, u8::saturating_sub));

    assert_eq!(to_bytes256(va.add_u8(0x90)), a.map(|x| x.wrapping_add(0x90)));
    assert_eq!(to_bytes256(va.saturating_add_u8(0x90)), a.map(|x| x.saturating_add(0x90)));
    assert_eq!(to_bytes256(va.sub_u8(0x90)), a.map(|x| x.wrapping_sub(0x90)));
    assert_eq!(to_bytes256(va.saturating_sub_u8(0x90)), a.map(|x| x.saturating_sub(0x90)));
}

#[test]
fn vec256_compare() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, mut b): ([u8; 32], [u8; 32]) = (pattern(37, 11), pattern(101, 200));
    b[3] = a[3];
    b[30] = a[30];
    let (va, vb) = (Vec256::read_unaligned(&a, cpu), Vec256::read_unaligned(&b, cpu));

    let eq = to_bytes256(va.eq_u8(vb));
    assert_eq!(eq, map2(a, b, |x, y| if x == y { 0xFF } else { 0 }));
    assert_eq!(eq.iter().filter(|&&x| x != 0).count(), 2);

    let eq = to_bytes256(va.eq_u8(a[5]));
    assert_eq!(eq, a.map(|x| if x == a[5] { 0xFF } else { 0 }));

    // `0xFF` is -1, so less than 1 as a signed integer
    let gt = to_bytes256(va.gt_i8(vb));
    assert_eq!(gt, map2(a, b, |x, y| if x.cast_signed() > y.cast_signed() { 0xFF } else { 0 }));
    let lt = to_bytes256(va.lt_i8(vb));
    assert_eq!(lt, map2(a, b, |x, y| if x.cast_signed() < y.cast_signed() { 0xFF } else { 0 }));

    // Include values either side of the thresholds
    let c: [u8; 32] = array::from_fn(|i| [0, 1, 0x1F, 0x20, 0x21, 0x7F, 0x80, 0xFF][i % 8]);
    let vc = Vec256::read_unaligned(&c, cpu);
    for n in [0, 0x20] {
        let gt = to_bytes256(vc.gt_i8(n));
        assert_eq!(gt, c.map(|x| if x.cast_signed() > n { 0xFF } else { 0 }));
        let lt = to_bytes256(vc.lt_i8(n));
        assert_eq!(lt, c.map(|x| if x.cast_signed() < n { 0xFF } else { 0 }));
    }
}

#[test]
fn vec256_all_eq_u8() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let a: [u8; 32] = pattern(37, 11);
    let va = Vec256::read_unaligned(&a, cpu);
    assert!(va.all_eq_u8(va));
    assert!(!va.all_eq_u8(a[0]));
    assert!(Vec256::splat(0x85, cpu).all_eq_u8(0x85));
    assert!(!Vec256::splat(0x85, cpu).all_eq_u8(0x84));

    // Check every byte, with a difference of only the lowest or highest bit
    for i in 0..32 {
        for bit in [0x01, 0x80] {
            let mut b = a;
            b[i] ^= bit;
            assert!(!va.all_eq_u8(Vec256::read_unaligned(&b, cpu)));
        }
    }
}

#[test]
fn vec256_shift() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let bytes = pattern(37, 11);
    let v = Vec256::read_unaligned(&bytes, cpu);
    assert_eq!(to_bytes256(v.shl_u16::<3>()), map_u16(bytes, |x| x << 3));
    assert_eq!(to_bytes256(v.shr_u16::<4>()), map_u16(bytes, |x| x >> 4));
}

#[test]
fn vec256_shuffle_u8s() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let bytes = pattern(37, 11);
    let control: [u8; 32] = pattern(23, 5);
    assert!(control.iter().any(|c| c & 0x80 != 0) && control.iter().any(|c| c & 0x80 == 0));

    let v = Vec256::read_unaligned(&bytes, cpu);
    let shuffled = to_bytes256(v.shuffle_u8s(Vec256::read_unaligned(&control, cpu)));
    assert_eq!(shuffled[..16], shuffle(&bytes[..16], &control[..16]));
    assert_eq!(shuffled[16..], shuffle(&bytes[16..], &control[16..]));
}

#[test]
fn vec256_blend() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b, mask) = (pattern(37, 11), pattern(101, 200), pattern(23, 5));
    let v = Vec256::read_unaligned(&a, cpu)
        .blend(Vec256::read_unaligned(&b, cpu), Vec256::read_unaligned(&mask, cpu));
    let expected: [u8; 32] = array::from_fn(|i| if mask[i] & 0x80 != 0 { b[i] } else { a[i] });
    assert_eq!(to_bytes256(v), expected);

    let v = Vec256::read_unaligned(&a, cpu).blend(0x85, Vec256::read_unaligned(&mask, cpu));
    let expected: [u8; 32] = array::from_fn(|i| if mask[i] & 0x80 != 0 { 0x85 } else { a[i] });
    assert_eq!(to_bytes256(v), expected);
}

#[test]
fn vec256_rearrange_u32s() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let s = Vec256::read_unaligned(&[0u32, 1, 2, 3, 4, 5, 6, 7], cpu);
    let o = Vec256::read_unaligned(&[10u32, 11, 12, 13, 14, 15, 16, 17], cpu);

    // Only bottom 3 bits of indices are used
    let indices = Vec256::read_unaligned(&[7u32, 0, 3, 3, 8 + 1, 2, 6, 4], cpu);
    assert_eq!(to_u32s(s.shuffle_u32s(indices)), [7, 0, 3, 3, 1, 2, 6, 4]);

    assert_eq!(to_u32s(s.unpack_lo_u32(o)), [0, 10, 1, 11, 4, 14, 5, 15]);
    assert_eq!(to_u32s(s.unpack_hi_u32(o)), [2, 12, 3, 13, 6, 16, 7, 17]);
    assert_eq!(to_u32s(s.join_lo_halves(o)), [0, 1, 2, 3, 10, 11, 12, 13]);
    assert_eq!(to_u32s(s.join_hi_halves(o)), [4, 5, 6, 7, 14, 15, 16, 17]);
}

#[test]
fn vec256_movemask() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b): ([u8; 32], [u8; 32]) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec256::read_unaligned(&a, cpu), Vec256::read_unaligned(&b, cpu));

    assert_eq!(va.movemask(), top_bits(&a));
    assert_eq!(vb.movemask(), top_bits(&b));
}

#[test]
fn vec128_create() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    assert_eq!(to_bytes128(Vec128::zero(cpu)), [0; 16]);
    assert_eq!(to_bytes128(Vec128::splat(0x85, cpu)), [0x85; 16]);

    let bytes = to_bytes128(Vec128::splat_u32(0x8234_5678, cpu));
    assert_eq!(bytes[..], *[0x8234_5678u32; 4].map(u32::to_le_bytes).as_flattened());
    assert_eq!(to_bytes128(0x8234_5678u32.into_vec128(cpu)), bytes);

    let bytes = pattern(37, 11);
    assert_eq!(to_bytes128(Vec128::read_unaligned(&bytes, cpu)), bytes);

    let bytes: [u8; 8] = pattern(37, 11);
    let v = to_bytes128(Vec128::read_low_unaligned(&bytes, cpu));
    assert_eq!(v[..8], bytes);
    assert_eq!(v[8..], [0; 8]);
}

#[test]
fn vec128_read_unaligned() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 16] = pattern(37, 11);
    // SAFETY: These types are all 16 bytes, and all bit patterns are valid for them
    let vectors = unsafe {
        [
            Vec128::read_unaligned(&bytes, cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [i8; 16]>(bytes), cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [u16; 8]>(bytes), cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [i16; 8]>(bytes), cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [u32; 4]>(bytes), cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [i32; 4]>(bytes), cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [u64; 2]>(bytes), cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [i64; 2]>(bytes), cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [u128; 1]>(bytes), cpu),
            Vec128::read_unaligned(&transmute::<[u8; 16], [i128; 1]>(bytes), cpu),
        ]
    };
    for v in vectors {
        assert_eq!(to_bytes128(v), bytes);
    }
}

#[test]
fn vec128_read_low_unaligned() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 8] = pattern(37, 11);
    // SAFETY: These types are all 8 bytes, and all bit patterns are valid for them
    let vectors = unsafe {
        [
            Vec128::read_low_unaligned(&bytes, cpu),
            Vec128::read_low_unaligned(&transmute::<[u8; 8], [i8; 8]>(bytes), cpu),
            Vec128::read_low_unaligned(&transmute::<[u8; 8], [u16; 4]>(bytes), cpu),
            Vec128::read_low_unaligned(&transmute::<[u8; 8], [i16; 4]>(bytes), cpu),
            Vec128::read_low_unaligned(&transmute::<[u8; 8], [u32; 2]>(bytes), cpu),
            Vec128::read_low_unaligned(&transmute::<[u8; 8], [i32; 2]>(bytes), cpu),
            Vec128::read_low_unaligned(&transmute::<[u8; 8], [u64; 1]>(bytes), cpu),
            Vec128::read_low_unaligned(&transmute::<[u8; 8], [i64; 1]>(bytes), cpu),
        ]
    };
    for v in vectors {
        let result = to_bytes128(v);
        assert_eq!(result[..8], bytes);
        assert_eq!(result[8..], [0; 8]);
    }
}

#[test]
fn vec128_load_store() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let src = Aligned(pattern::<16>(37, 11));
    let mut dest = Aligned([0; 16]);
    // SAFETY: `src` and `dest` are 16 bytes and aligned on 32, which is also aligned on 16
    unsafe { Vec128::load(src.0.as_ptr(), cpu).store(dest.0.as_mut_ptr()) };
    assert_eq!(dest.0, src.0);

    let src = pattern::<18>(37, 11);
    let mut dest = [0; 18];
    // SAFETY: `src` and `dest` are valid for 16 bytes starting at offset 1
    unsafe {
        let v = Vec128::load_unaligned(src.as_ptr().add(1), cpu);
        v.store_unaligned(dest.as_mut_ptr().add(1));
    }
    assert_eq!(dest[1..17], src[1..17]);
    assert_eq!((dest[0], dest[17]), (0, 0));

    let src = pattern::<10>(37, 11);
    // SAFETY: `src` is valid for reading 8 bytes starting at offset 1
    let v = unsafe { Vec128::load_low_unaligned(src.as_ptr().add(1), cpu) };
    let bytes = to_bytes128(v);
    assert_eq!(bytes[..8], src[1..9]);
    assert_eq!(bytes[8..], [0; 8]);

    let mut dest = [0; 10];
    // SAFETY: `dest` is valid for writing 8 bytes starting at offset 1
    unsafe { Vec128::splat(0x85, cpu).store_low_unaligned(dest.as_mut_ptr().add(1)) };
    assert_eq!(dest[1..9], [0x85; 8]);
    assert_eq!((dest[0], dest[9]), (0, 0));

    let values: [u64; 2] = [0x0102_0304_0506_0708, 1 << 63];
    let bytes = to_bytes128(Vec128::read_unaligned(&values, cpu));
    assert_eq!(bytes[..], *values.map(u64::to_le_bytes).as_flattened());
}

#[test]
fn vec128_bitwise() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec128::read_unaligned(&a, cpu), Vec128::read_unaligned(&b, cpu));

    assert_eq!(to_bytes128(va & vb), map2(a, b, |x, y| x & y));
    assert_eq!(to_bytes128(va | vb), map2(a, b, |x, y| x | y));
    assert_eq!(to_bytes128(va ^ vb), map2(a, b, |x, y| x ^ y));
    assert_eq!(to_bytes128(!va), a.map(|x| !x));
    assert_eq!(to_bytes128(va & 0x3C), a.map(|x| x & 0x3C));
    assert_eq!(to_bytes128(va | 0x3C), a.map(|x| x | 0x3C));
    assert_eq!(to_bytes128(va ^ 0x3C), a.map(|x| x ^ 0x3C));

    let mut v = va;
    v &= vb;
    assert_eq!(to_bytes128(v), map2(a, b, |x, y| x & y));

    let mut v = va;
    v |= vb;
    assert_eq!(to_bytes128(v), map2(a, b, |x, y| x | y));

    let mut v = va;
    v ^= vb;
    assert_eq!(to_bytes128(v), map2(a, b, |x, y| x ^ y));

    let mut v = va;
    v &= 0x3C;
    assert_eq!(to_bytes128(v), a.map(|x| x & 0x3C));

    let mut v = va;
    v |= 0x3C;
    assert_eq!(to_bytes128(v), a.map(|x| x | 0x3C));

    let mut v = va;
    v ^= 0x3C;
    assert_eq!(to_bytes128(v), a.map(|x| x ^ 0x3C));
}

#[test]
fn vec128_is_zero() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    assert!(Vec128::zero(cpu).is_zero());
    assert!(!Vec128::splat(0xFF, cpu).is_zero());

    // Check every byte, with only the lowest or highest bit set
    for i in 0..16 {
        for bit in [0x01, 0x80] {
            let mut bytes = [0u8; 16];
            bytes[i] = bit;
            assert!(!Vec128::read_unaligned(&bytes, cpu).is_zero());
        }
    }
}

#[test]
fn vec128_add_sub() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec128::read_unaligned(&a, cpu), Vec128::read_unaligned(&b, cpu));

    assert_eq!(to_bytes128(va.add_u8(vb)), map2(a, b, u8::wrapping_add));
    assert_eq!(to_bytes128(va.saturating_add_u8(vb)), map2(a, b, u8::saturating_add));
    assert_eq!(to_bytes128(va.sub_u8(vb)), map2(a, b, u8::wrapping_sub));
    assert_eq!(to_bytes128(va.saturating_sub_u8(vb)), map2(a, b, u8::saturating_sub));

    assert_eq!(to_bytes128(va.add_u8(0x90)), a.map(|x| x.wrapping_add(0x90)));
    assert_eq!(to_bytes128(va.saturating_add_u8(0x90)), a.map(|x| x.saturating_add(0x90)));
    assert_eq!(to_bytes128(va.sub_u8(0x90)), a.map(|x| x.wrapping_sub(0x90)));
    assert_eq!(to_bytes128(va.saturating_sub_u8(0x90)), a.map(|x| x.saturating_sub(0x90)));
}

#[test]
fn vec128_compare() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, mut b): ([u8; 16], [u8; 16]) = (pattern(37, 11), pattern(101, 200));
    b[3] = a[3];
    let (va, vb) = (Vec128::read_unaligned(&a, cpu), Vec128::read_unaligned(&b, cpu));

    let eq = to_bytes128(va.eq_u8(vb));
    assert_eq!(eq, map2(a, b, |x, y| if x == y { 0xFF } else { 0 }));
    assert_eq!(eq[3], 0xFF);

    let eq = to_bytes128(va.eq_u8(a[5]));
    assert_eq!(eq, a.map(|x| if x == a[5] { 0xFF } else { 0 }));

    let gt = to_bytes128(va.gt_i8(vb));
    assert_eq!(gt, map2(a, b, |x, y| if x.cast_signed() > y.cast_signed() { 0xFF } else { 0 }));
    let lt = to_bytes128(va.lt_i8(vb));
    assert_eq!(lt, map2(a, b, |x, y| if x.cast_signed() < y.cast_signed() { 0xFF } else { 0 }));

    // Include values either side of the thresholds
    let c: [u8; 16] = array::from_fn(|i| [0, 1, 0x1F, 0x20, 0x21, 0x7F, 0x80, 0xFF][i % 8]);
    let vc = Vec128::read_unaligned(&c, cpu);
    for n in [0, 0x20] {
        let gt = to_bytes128(vc.gt_i8(n));
        assert_eq!(gt, c.map(|x| if x.cast_signed() > n { 0xFF } else { 0 }));
        let lt = to_bytes128(vc.lt_i8(n));
        assert_eq!(lt, c.map(|x| if x.cast_signed() < n { 0xFF } else { 0 }));
    }
}

#[test]
fn vec128_all_eq_u8() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let a: [u8; 16] = pattern(37, 11);
    let va = Vec128::read_unaligned(&a, cpu);
    assert!(va.all_eq_u8(va));
    assert!(!va.all_eq_u8(a[0]));
    assert!(Vec128::splat(0x85, cpu).all_eq_u8(0x85));
    assert!(!Vec128::splat(0x85, cpu).all_eq_u8(0x84));

    // Check every byte, with a difference of only the lowest or highest bit
    for i in 0..16 {
        for bit in [0x01, 0x80] {
            let mut b = a;
            b[i] ^= bit;
            assert!(!va.all_eq_u8(Vec128::read_unaligned(&b, cpu)));
        }
    }
}

#[test]
fn vec128_rearrange_bytes() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 16] = pattern(37, 11);
    let v = Vec128::read_unaligned(&bytes, cpu);

    let shifted = to_bytes128(v.shr_bytes::<3>());
    assert_eq!(shifted[..13], bytes[3..]);
    assert_eq!(shifted[13..], [0; 3]);

    let control: [u8; 16] = pattern(23, 5);
    assert!(control.iter().any(|c| c & 0x80 != 0) && control.iter().any(|c| c & 0x80 == 0));
    let shuffled = to_bytes128(v.shuffle_u8s(Vec128::read_unaligned(&control, cpu)));
    assert_eq!(shuffled[..], shuffle(&bytes, &control));
}

#[test]
fn vec128_blend() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b, mask) = (pattern(37, 11), pattern(101, 200), pattern(23, 5));
    let v = Vec128::read_unaligned(&a, cpu)
        .blend(Vec128::read_unaligned(&b, cpu), Vec128::read_unaligned(&mask, cpu));
    let expected: [u8; 16] = array::from_fn(|i| if mask[i] & 0x80 != 0 { b[i] } else { a[i] });
    assert_eq!(to_bytes128(v), expected);

    let v = Vec128::read_unaligned(&a, cpu).blend(0x85, Vec128::read_unaligned(&mask, cpu));
    let expected: [u8; 16] = array::from_fn(|i| if mask[i] & 0x80 != 0 { 0x85 } else { a[i] });
    assert_eq!(to_bytes128(v), expected);
}

#[test]
fn vec128_movemask() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b): ([u8; 16], [u8; 16]) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec128::read_unaligned(&a, cpu), Vec128::read_unaligned(&b, cpu));

    assert_eq!(va.movemask(), top_bits(&a));
    assert_eq!(vb.movemask(), top_bits(&b));
}
