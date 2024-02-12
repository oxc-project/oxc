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
    pub fn match_vectored(
        &self,
        data: &[u8; Self::ALIGNMENT],
        padding: usize,
    ) -> Option<(usize, u8)> {
        let ptr = data.as_ptr();
        // SAFETY:
        // data is aligned and has ALIGNMENT bytes
        unsafe { self.match_delimiters(ptr, padding) }.map(|pos| (pos, data[pos]))
    }

    // same with avx2, but neon doesn't have a _mm256_movemask_epi8 instruction
    // so, we need to use a different approach(offsetz)
    #[inline]
    unsafe fn match_delimiters(&self, ptr: *const u8, padding: usize) -> Option<usize> {
        let data = vld1q_u8(ptr);
        let col_idx = vandq_u8(data, vdupq_n_u8(0b1111));
        let col = vqtbl1q_u8(self.table, col_idx);
        let row_idx = vshrq_n_u8(data, 4);
        let row = vqtbl1q_u8(self.arf, row_idx);
        let tmp = vandq_u8(col, row);
        let result = vceqq_u8(tmp, row);
        offsetz(result, self.reverse, padding)
    }
}

#[inline]
unsafe fn offsetz(x: uint8x16_t, reverse: bool, padding: usize) -> Option<usize> {
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
    // if the match is in the padding, we should ignore it
    if pos >= 16 - padding {
        None
    } else {
        Some(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::MatchTable;
    use crate::lexer::{source::Source, UniquePromise};
    #[test]
    fn neon_match_non_ascii() {
        let table = seq_macro::seq!(b in 0u8..=255 {
            // find non ascii
            MatchTable::new([#(b.is_ascii_alphanumeric() || b == b'_' || b == b'$',)*], true)
        });
        let data = ["AAAAAAAA\"\rAAAAAA", "AAAAAAAAAAAAAAA\"", "AAAAAAAAAAAAAAAA", "AAAAAAAA"]
            .map(|x| Source::new(x, UniquePromise::new_for_tests()));
        let expected = [
            (Some((8, b'"')), MatchTable::ALIGNMENT),
            (Some((15, b'"')), MatchTable::ALIGNMENT),
            (None, MatchTable::ALIGNMENT),
            (None, 8),
        ];

        for (idx, d) in data.into_iter().enumerate() {
            let (data, actual_len) =
                unsafe { d.peek_n_with_padding::<{ MatchTable::ALIGNMENT }>() }.unwrap();
            let result = table.match_vectored(&data, 16 - actual_len);
            assert_eq!((result, actual_len), expected[idx]);
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
            let result = table.match_vectored(d.as_bytes().try_into().unwrap(), 0);
            assert_eq!(result, expected[idx]);
        }
    }
}
