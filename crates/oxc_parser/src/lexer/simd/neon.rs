use super::tabulate;
use core::arch::aarch64::*;

#[derive(Debug)]
pub struct MatchTable {
    table: uint8x16_t,
    arf: uint8x16_t,
    //The result of the match is reversed.
    reverse: bool,
}

impl MatchTable {
    pub const ALIGNMENT: usize = 16;

    pub fn new(bytes: [bool; 256], reverse: bool) -> Self {
        let table = tabulate(bytes);
        let table = unsafe { vld1q_u8(table.as_ptr()) };
        let arf = unsafe {
            vld1q_u8([1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128].as_ptr())
        };
        Self { table, arf, reverse }
    }

    #[inline]
    pub fn match_vectored(&self, data: &[u8; Self::ALIGNMENT]) -> Option<(usize, u8)> {
        let ptr = data.as_ptr();
        // SAFETY:
        // data is aligned and has ALIGNMENT bytes
        unsafe { self.match_delimiters(ptr) }.map(|pos| (pos, data[pos]))
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
        offsetz(result, self.reverse)
    }
}

#[inline]
unsafe fn offsetz(x: uint8x16_t, reverse: bool) -> Option<usize> {
    #[inline]
    fn clz(x: u64) -> usize {
        // perf: rust will unroll this loop
        // and it's much faster than rbit + clz so voila
        for (i, b) in x.to_ne_bytes().into_iter().enumerate() {
            if b != 0 {
                return i;
            }
        }
        8 // Technically not reachable since zero-guarded
    }

    let x = if reverse { vmvnq_u8(x) } else { x };
    // Extract two u64
    let x = vreinterpretq_u64_u8(x);
    // Extract to general purpose registers to perform clz
    let low: u64 = vgetq_lane_u64::<0>(x);
    let high: u64 = vgetq_lane_u64::<1>(x);
    let pos = if low != 0 {
        clz(low)
    } else if high != 0 {
        8 + clz(high)
    } else {
        // all zero means no match
        return None;
    };
    Some(pos)
}
