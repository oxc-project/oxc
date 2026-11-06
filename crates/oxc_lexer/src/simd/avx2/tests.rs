//! Tests for the methods of `Vec128<Avx2>` and `Vec256<Avx2>`
//! which only AVX2 has, or which behave differently on AVX2.
//!
//! `simd::tests` has the tests which apply to every CPU type.
//!
//! Each test returns early without checking anything if the CPU doesn't support AVX2.

use std::array;

use crate::{
    cpu::DynCpu,
    simd::{
        Vec128, Vec256,
        tests::{pattern, to_bytes128, to_bytes256, to_u32s},
    },
};

/// Get the scalar equivalent of `movemask`.
fn top_bits(bytes: &[u8]) -> u32 {
    bytes.iter().enumerate().fold(0, |mask, (i, byte)| mask | (u32::from(byte >> 7) << i))
}

/// Get the scalar equivalent of `shuffle_u8s` on 16 bytes.
fn shuffle(bytes: &[u8], control: &[u8]) -> Vec<u8> {
    let select = |&c: &u8| if c & 0x80 == 0 { bytes[(c & 0x0F) as usize] } else { 0 };
    control.iter().map(select).collect()
}

#[test]
fn vec128_movemask() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b): ([u8; 16], [u8; 16]) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec128::read_unaligned(&a, cpu), Vec128::read_unaligned(&b, cpu));

    assert_eq!(va.movemask(), top_bits(&a));
    assert_eq!(vb.movemask(), top_bits(&b));
}

#[test]
fn vec128_shuffle_u8s() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 16] = pattern(37, 11);
    let v = Vec128::read_unaligned(&bytes, cpu);

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
fn vec256_movemask() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let (a, b): ([u8; 32], [u8; 32]) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec256::read_unaligned(&a, cpu), Vec256::read_unaligned(&b, cpu));

    assert_eq!(va.movemask(), top_bits(&a));
    assert_eq!(vb.movemask(), top_bits(&b));
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
fn vec256_shuffle_u32s() {
    let DynCpu::Avx2(cpu) = DynCpu::detect() else { return };

    let s = Vec256::read_unaligned(&[0u32, 1, 2, 3, 4, 5, 6, 7], cpu);

    // Only bottom 3 bits of indices are used
    let indices = Vec256::read_unaligned(&[7u32, 0, 3, 3, 8 + 1, 2, 6, 4], cpu);
    assert_eq!(to_u32s(s.shuffle_u32s(indices)), [7, 0, 3, 3, 1, 2, 6, 4]);
}
