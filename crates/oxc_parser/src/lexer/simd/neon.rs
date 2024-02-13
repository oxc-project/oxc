use super::tabulate16;
use core::arch::aarch64::*;

pub const ALIGNMENT: usize = 16;

#[derive(Debug)]
pub struct MatchTable {
    table: [u8; ALIGNMENT],
    arf: [u8; ALIGNMENT],
    //The result of the match is reversed.
    reverse: bool,
}

impl MatchTable {
    pub const fn new(bytes: [bool; 256], reverse: bool) -> Self {
        let table = tabulate16(bytes, reverse);
        let arf = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];
        Self { table, arf, reverse }
    }

    #[inline]
    pub fn matches(&self, data: &[u8; ALIGNMENT], actual_len: usize) -> Option<(usize, u8)> {
        let ptr = data.as_ptr();
        // SAFETY:
        // data is aligned and has ALIGNMENT bytes
        unsafe { self.match_delimiters(ptr) }.map(|pos| (pos, data[pos])).and_then(|(pos, b)| {
            if pos >= actual_len {
                None
            } else {
                Some((pos, b))
            }
        })
    }

    // same with avx2, but neon doesn't have a _mm256_movemask_epi8 instruction
    // so, we need to use a different approach(offsetz)
    #[inline]
    unsafe fn match_delimiters(&self, ptr: *const u8) -> Option<usize> {
        let data = vld1q_u8(ptr);
        let col_idx = vandq_u8(data, vdupq_n_u8(0x8F));
        let col = vqtbl1q_u8(vld1q_u8(self.table.as_ptr()), col_idx);
        let row_idx = vshrq_n_u8(data, 4);
        let row = vqtbl1q_u8(vld1q_u8(self.arf.as_ptr()), row_idx);
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

#[cfg(test)]
mod tests {
    use super::{MatchTable, ALIGNMENT};
    use crate::lexer::{source::Source, UniquePromise};
    #[test]
    fn neon_find_non_ascii() {
        let table = seq_macro::seq!(b in 0u8..=255 {
            // find non ascii
            MatchTable::new([#(b.is_ascii_alphanumeric() || b == b'_' || b == b'$',)*], true)
        });
        let data = [
            "AAAAAAAA\"\rAAAAAA",
            "AAAAAAAAAAAAAAA\"",
            "AAAAAAAAAAAAAAAA",
            "AAAAAAAA",
            "AAAAAAAA\"\rAAAAAA",
            "AAAAAAAAAAAAAAA\r",
        ]
        .map(|x| Source::new(x, UniquePromise::new_for_tests()));
        let expected = [
            (Some((8, b'"')), ALIGNMENT),
            (Some((15, b'"')), ALIGNMENT),
            (None, ALIGNMENT),
            (None, 8),
            (Some((8, b'\"')), ALIGNMENT),
            (Some((15, b'\r')), ALIGNMENT),
        ];

        for (idx, d) in data.into_iter().enumerate() {
            let pos = d.position();
            let (data, actual_len) =
                unsafe { pos.peek_n_with_padding::<ALIGNMENT>(d.end_addr()) }.unwrap();
            let result = table.matches(&data, actual_len);
            assert_eq!((result, actual_len), expected[idx]);
        }
    }

    #[test]
    fn neon_find_single_quote_string() {
        let table = seq_macro::seq!(b in 0u8..=255 {
            // find non ascii
            MatchTable::new([#(matches!(b, b'\'' | b'\r' | b'\n' | b'\\'),)*], false)
        });
        let s1 = String::from(138u8 as char);
        let data = [&s1].map(|x| Source::new(x, UniquePromise::new_for_tests()));
        let expected = [(None, 2)];

        for (idx, d) in data.into_iter().enumerate() {
            let pos = d.position();
            let (data, actual_len) =
                unsafe { pos.peek_n_with_padding::<ALIGNMENT>(d.end_addr()) }.unwrap();
            let result = table.matches(&data, actual_len);
            assert_eq!((result, actual_len), expected[idx]);
        }
    }
}
