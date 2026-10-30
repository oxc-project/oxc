use std::{array, mem::transmute};

use crate::{
    cpu::DynCpu,
    simd::{
        operand::Operand128,
        tests::{map2, pattern},
    },
};

use super::{Vec128, Vec256};

fn to_bytes128(v: Vec128) -> [u8; 16] {
    let mut out = [0; 16];
    // SAFETY: `out` is valid for writing 16 bytes
    unsafe { v.store_unaligned(out.as_mut_ptr()) };
    out
}

/// Get the bytes of a [`Vec256`].
fn to_bytes256(v: Vec256) -> [u8; 32] {
    let (low, high) = v.into_halves();
    let (low, high) = (to_bytes128(low), to_bytes128(high));
    array::from_fn(|i| if i < 16 { low[i] } else { high[i - 16] })
}

#[test]
fn vec128_create() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
fn vec128_shift() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 16] = pattern(37, 11);
    let v = Vec128::read_unaligned(&bytes, cpu);

    assert_eq!(to_bytes128(v.shl_u8::<3>()), bytes.map(|x| x << 3));
    assert_eq!(to_bytes128(v.shr_u8::<3>()), bytes.map(|x| x >> 3));
}

#[test]
fn vec128_is_zero() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
fn vec128_pairwise_add_u8() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let (a, b): ([u8; 16], [u8; 16]) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec128::read_unaligned(&a, cpu), Vec128::read_unaligned(&b, cpu));

    let expected: [u8; 16] = array::from_fn(|i| {
        let (bytes, pair) = if i < 8 { (&a, i) } else { (&b, i - 8) };
        bytes[pair * 2].wrapping_add(bytes[pair * 2 + 1])
    });
    assert_eq!(to_bytes128(va.pairwise_add_u8(vb)), expected);
}

#[test]
fn vec128_compare() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let (a, mut b): ([u8; 16], [u8; 16]) = (pattern(37, 11), pattern(101, 200));
    b[3] = a[3];
    let (va, vb) = (Vec128::read_unaligned(&a, cpu), Vec128::read_unaligned(&b, cpu));

    let eq = to_bytes128(va.eq_u8(vb));
    assert_eq!(eq, map2(a, b, |x, y| if x == y { 0xFF } else { 0 }));
    assert_eq!(eq[3], 0xFF);

    let eq = to_bytes128(va.eq_u8(a[5]));
    assert_eq!(eq, a.map(|x| if x == a[5] { 0xFF } else { 0 }));

    let test = to_bytes128(va.test_u8(vb));
    assert_eq!(test, map2(a, b, |x, y| if x & y != 0 { 0xFF } else { 0 }));

    let test = to_bytes128(va.test_u8(0x88));
    assert_eq!(test, a.map(|x| if x & 0x88 != 0 { 0xFF } else { 0 }));

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
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 16] = pattern(37, 11);
    let v = Vec128::read_unaligned(&bytes, cpu);

    let shifted = to_bytes128(v.shr_bytes::<3>());
    assert_eq!(shifted[..13], bytes[3..]);
    assert_eq!(shifted[13..], [0; 3]);

    // Include 16, 0x1F and 0x7F, which AVX2 would treat as in range
    let control: [u8; 16] = [15, 0, 16, 0x1F, 0x7F, 0x80, 0xFF, 7, 8, 1, 2, 3, 14, 13, 0x8F, 9];
    let shuffled = to_bytes128(v.shuffle_u8s(Vec128::read_unaligned(&control, cpu)));
    assert_eq!(shuffled, control.map(|c| if c < 16 { bytes[c as usize] } else { 0 }));
}

#[test]
fn vec256_create() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    assert_eq!(to_bytes256(Vec256::splat(0x85, cpu)), [0x85; 32]);

    let bytes = to_bytes256(Vec256::splat_u32(0x8234_5678, cpu));
    assert_eq!(bytes[..], *[0x8234_5678u32; 8].map(u32::to_le_bytes).as_flattened());
}

#[test]
fn vec256_load_unaligned() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let src = pattern::<34>(37, 11);
    // SAFETY: `src` is valid for reading 32 bytes starting at offset 1
    let v = unsafe { Vec256::load_unaligned(src.as_ptr().add(1), cpu) };
    assert_eq!(to_bytes256(v)[..], src[1..33]);
}

#[test]
fn vec256_read_unaligned() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

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
fn vec256_bitwise() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let (a, b): ([u8; 32], [u8; 32]) = (pattern(37, 11), pattern(101, 200));
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
fn vec256_halves() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let (low, high): ([u8; 16], [u8; 16]) = (pattern(37, 11), pattern(101, 200));
    let (vlow, vhigh) = (Vec128::read_unaligned(&low, cpu), Vec128::read_unaligned(&high, cpu));

    let (out_low, out_high) = Vec256::from_halves(vlow, vhigh).into_halves();
    assert_eq!(to_bytes128(out_low), low);
    assert_eq!(to_bytes128(out_high), high);
}

#[test]
fn vec256_shuffle_u8s() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let table: [u8; 32] = pattern(37, 11);
    let v = Vec256::read_unaligned(&table, cpu);

    let control: [u8; 16] = [31, 0, 16, 15, 32, 0x7F, 0x80, 0xFF, 17, 8, 24, 33, 47, 1, 30, 14];
    let expected = control.map(|c| if c < 32 { table[c as usize] } else { 0 });
    let shuffled = v.shuffle_u8s(Vec128::read_unaligned(&control, cpu));
    assert_eq!(to_bytes128(shuffled), expected);
}

#[test]
fn vec128_blend() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let (a, b, mask) = (pattern(37, 11), pattern(101, 200), pattern(23, 5));
    let v = Vec128::read_unaligned(&a, cpu)
        .blend(Vec128::read_unaligned(&b, cpu), Vec128::read_unaligned(&mask, cpu));
    let expected: [u8; 16] = array::from_fn(|i| (a[i] & !mask[i]) | (b[i] & mask[i]));
    assert_eq!(to_bytes128(v), expected);

    let v = Vec128::read_unaligned(&a, cpu).blend(0x85, Vec128::read_unaligned(&mask, cpu));
    let expected: [u8; 16] = array::from_fn(|i| (a[i] & !mask[i]) | (0x85 & mask[i]));
    assert_eq!(to_bytes128(v), expected);
}

#[test]
fn vec128_get_u64() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let values: [u64; 2] = [0x0102_0304_0506_0708, 1 << 63];
    let v = Vec128::read_unaligned(&values, cpu);
    assert_eq!(v.get_u64::<0>(), values[0]);
    assert_eq!(v.get_u64::<1>(), values[1]);
}
