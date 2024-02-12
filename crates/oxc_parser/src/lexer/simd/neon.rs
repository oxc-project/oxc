use super::tabulate;
use core::arch::aarch64::*;

#[derive(Debug)]
pub struct MatchTable {
    table: uint8x16_t,
    arf: uint8x16_t,
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
        let first_found = offsetz(result, self.reverse);
        // reach the end of the segment, so no delimiter found
        if first_found == Self::ALIGNMENT {
            None
        } else {
            Some(first_found)
        }
    }
}

#[inline]
unsafe fn offsetz(x: uint8x16_t, reverse: bool) -> usize {
    #[inline]
    fn clz(x: u64) -> usize {
        // perf: rust will unroll this loop
        // and it's much faster than rbit + clz so voila
        for (i, b) in x.to_ne_bytes().iter().copied().enumerate() {
            if b != 0 {
                return i;
            }
        }
        7 // Technically not reachable since zero-guarded
    }

    let x = if reverse { vmvnq_u8(x) } else { x };
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
fn neon_match_non_ascii() {
    let table = seq_macro::seq!(b in 0u8..=255 {
        // find non ascii
        MatchTable::new([#(b.is_ascii_alphanumeric() || b == b'_' || b == b'$',)*], true)
    });
    let data = ["AAAAAAAA\"\rAAAAAA", "AAAAAAAAAAAAAAA\""];
    let expected = [Some((8, b'"')), Some((15, b'"'))];

    for (idx, d) in data.into_iter().enumerate() {
        let result = table.match_vectored(d.as_bytes().try_into().unwrap());
        assert_eq!(result, expected[idx]);
    }
}

#[test]
fn neon_match_non_space() {
    let table = seq_macro::seq!(b in 0u8..=255 {
        // find non ascii
        MatchTable::new([#(matches!(b, b' ' | b'\t' | b'\r' | b'\n'),)*], false)
    });
    let data = ["AAAAAAAA\"\rAAAAAA", "AAAAAAAAAAAAAAA\r"];
    let expected = [Some((9, b'\r')), Some((15, b'\r'))];

    for (idx, d) in data.into_iter().enumerate() {
        let result = table.match_vectored(d.as_bytes().try_into().unwrap());
        assert_eq!(result, expected[idx]);
    }
}
