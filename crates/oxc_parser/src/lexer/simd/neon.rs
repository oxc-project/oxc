use super::tabulate;
use core::arch::aarch64::*;

#[derive(Debug)]
pub struct LookupTable {
    table: uint8x16_t,
    arf: uint8x16_t,
}

impl LookupTable {
    pub const ALIGNMENT: usize = 16;

    pub fn new<const N: usize>(delimiters: [u8; N]) -> Self {
        let table = tabulate(delimiters);
        let table = unsafe { vld1q_u8(table.as_ptr()) };
        let arf = unsafe {
            vld1q_u8([1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128].as_ptr())
        };
        Self { table, arf }
    }

    #[inline]
    pub fn match_vectored(&self, data: &[u8; Self::ALIGNMENT]) -> Option<usize> {
        let ptr = data.as_ptr();
        // SAFETY:
        // seg is aligned and has ALIGNMENT bytes
        unsafe { self.match_delimiters(ptr) }
    }

    // same with avx2, but neon doesn't have a _mm256_movemask_epi8 instruction
    // so, we need to use a different approach(offsetz)
    #[inline]
    unsafe fn match_delimiters(&self, ptr: *const u8) -> Option<usize> {
        let data = vld1q_u8(ptr);
        let col_idx = vandq_u8(data, vdupq_n_u8(0b1111));
        let col = vqtbl1q_u8(self.table, col_idx);
        let row_idx = vshrq_n_u8(data, 4);
        let row = vqtbl1q_u8(self.arf, row_idx);
        let tmp = vandq_u8(col, row);
        let result = vceqq_u8(tmp, row);
        let unmatched = offsetz(result);
        // reach the end of the segment, so no delimiter found
        if unmatched == Self::ALIGNMENT {
            None
        } else {
            Some(unmatched)
        }
    }
}

#[inline]
unsafe fn offsetz(x: uint8x16_t) -> usize {
    #[inline]
    fn clz(x: u64) -> usize {
        // perf: rust will unroll this loop
        // and it's much faster than rbit + clz so voila
        for (i, b) in x.to_ne_bytes().iter().copied().enumerate() {
            if b != 0 {
                return i;
            }
        }
        8 // Technically not reachable since zero-guarded
    }

    // Extract two u64
    let x = vreinterpretq_u64_u8(x);
    // Extract to general purpose registers to perform clz
    let low: u64 = vgetq_lane_u64::<0>(x);
    let high: u64 = vgetq_lane_u64::<1>(x);
    if low != 0 {
        clz(low)
    } else if high != 0 {
        8 + clz(high)
    } else {
        16
    }
}

#[test]
fn neon_match() {
    use crate::{lexer::source::Source, parser_parse::UniquePromise};
    let table = LookupTable::new([b'\r', b'\n', b'"', b'\'', b'\\']);
    let unique = UniquePromise::new_for_tests();
    let source = Source::new(
        r#""hello world!hello world!hello world!hello world!hello world!hello world!hello world!hello world!hello world!""#,
        unique,
    );
    let (offset, actual_len) = table.match_vectored(&source);
    assert_eq!(offset, Some(0));
    assert_eq!(actual_len, LookupTable::ALIGNMENT);
}
