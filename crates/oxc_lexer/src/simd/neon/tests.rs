//! Tests for the methods of `Vec128<Neon>` and `Vec256<Neon>`
//! which only Neon has, or which behave differently on Neon.
//!
//! `simd::tests` has the tests which apply to every CPU type.
//!
//! Each test returns early without checking anything if the CPU doesn't support Neon.

use std::array;

use crate::{
    cpu::DynCpu,
    simd::{
        Vec128, Vec256,
        tests::{map2, pattern, to_bytes128, to_bytes256},
    },
};

#[test]
fn vec128_shift() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 16] = pattern(37, 11);
    let v = Vec128::read_unaligned(&bytes, cpu);

    assert_eq!(to_bytes128(v.shl_u8::<3>()), bytes.map(|x| x << 3));
    assert_eq!(to_bytes128(v.shr_u8::<3>()), bytes.map(|x| x >> 3));
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
fn vec128_test_u8() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let (a, b): ([u8; 16], [u8; 16]) = (pattern(37, 11), pattern(101, 200));
    let (va, vb) = (Vec128::read_unaligned(&a, cpu), Vec128::read_unaligned(&b, cpu));

    let test = to_bytes128(va.test_u8(vb));
    assert_eq!(test, map2(a, b, |x, y| if x & y != 0 { 0xFF } else { 0 }));

    let test = to_bytes128(va.test_u8(0x88));
    assert_eq!(test, a.map(|x| if x & 0x88 != 0 { 0xFF } else { 0 }));
}

#[test]
fn vec128_shuffle_u8s() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let bytes: [u8; 16] = pattern(37, 11);
    let v = Vec128::read_unaligned(&bytes, cpu);

    // Include 16, 0x1F and 0x7F, which AVX2 would treat as in range
    let control: [u8; 16] = [15, 0, 16, 0x1F, 0x7F, 0x80, 0xFF, 7, 8, 1, 2, 3, 14, 13, 0x8F, 9];
    let shuffled = to_bytes128(v.shuffle_u8s(Vec128::read_unaligned(&control, cpu)));
    assert_eq!(shuffled, control.map(|c| if c < 16 { bytes[c as usize] } else { 0 }));
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
fn vec256_blend() {
    let DynCpu::Neon(cpu) = DynCpu::detect() else { return };

    let (a, b, mask) = (pattern(37, 11), pattern(101, 200), pattern(23, 5));
    let v = Vec256::read_unaligned(&a, cpu)
        .blend(Vec256::read_unaligned(&b, cpu), Vec256::read_unaligned(&mask, cpu));
    let expected: [u8; 32] = array::from_fn(|i| (a[i] & !mask[i]) | (b[i] & mask[i]));
    assert_eq!(to_bytes256(v), expected);

    let v = Vec256::read_unaligned(&a, cpu).blend(0x85, Vec256::read_unaligned(&mask, cpu));
    let expected: [u8; 32] = array::from_fn(|i| (a[i] & !mask[i]) | (0x85 & mask[i]));
    assert_eq!(to_bytes256(v), expected);
}
