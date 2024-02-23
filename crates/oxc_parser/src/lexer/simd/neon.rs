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

    // return a iterator of (position, byte) for lazy evaluation
    #[inline]
    pub fn matches<'a>(
        &self,
        seg: &'a [u8; ALIGNMENT],
        actual_len: usize,
    ) -> impl Iterator<Item = (usize, u8)> + 'a {
        // SAFETY:
        // data is aligned and has ALIGNMENT bytes
        unsafe { self.match_delimiters(seg, actual_len) }
    }

    // same with avx2, but neon doesn't have a _mm256_movemask_epi8 instruction
    // so, we need to use a different approach(offsetz)
    #[inline]
    unsafe fn match_delimiters<'a>(
        &self,
        seg: &'a [u8; ALIGNMENT],
        actual_len: usize,
    ) -> MatchTableIter<'a> {
        let ptr = seg.as_ptr();
        let data = vld1q_u8(ptr);
        let col_idx = vandq_u8(data, vdupq_n_u8(0x8F));
        let col = vqtbl1q_u8(vld1q_u8(self.table.as_ptr()), col_idx);
        let row_idx = vshrq_n_u8(data, 4);
        let row = vqtbl1q_u8(vld1q_u8(self.arf.as_ptr()), row_idx);
        let tmp = vandq_u8(col, row);
        let result = vceqq_u8(tmp, row);
        offsetz(result, seg, self.reverse, actual_len)
    }
}

struct MatchTableIter<'a> {
    seg: &'a [u8; ALIGNMENT],
    low: [u8; 8],
    high: [u8; 8],
    low_upper: usize,
    high_upper: usize,
    pos: Option<usize>, // the last position of the matched byte
}

impl<'a> Iterator for MatchTableIter<'a> {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let mut start = match self.pos {
            Some(pos) => pos + 1,
            None => 0,
        };
        let mut offset = 0;
        // lookup at low 8 bytes
        while start < self.low_upper {
            let b = self.low[start];
            let found = self.seg[start];
            if b != 0 {
                self.pos = Some(start);
                return Some((offset, found));
            }
            offset += 1;
            start += 1;
        }
        // lookup at high 8 bytes
        while start < self.high_upper {
            let b = self.high[start - 8];
            let found = self.seg[start];
            if b != 0 {
                self.pos = Some(start);
                return Some((offset, found));
            }
            offset += 1;
            start += 1;
        }
        // reach the end of the segment data
        None
    }
}

#[inline]
unsafe fn offsetz(
    x: uint8x16_t,
    seg: &[u8; ALIGNMENT],
    reverse: bool,
    actual_len: usize,
) -> MatchTableIter<'_> {
    let x = if reverse { vmvnq_u8(x) } else { x };
    // Extract two u64
    let x = vreinterpretq_u64_u8(x);
    // Extract to general purpose registers to perform clz
    let low: u64 = vgetq_lane_u64::<0>(x);
    let high: u64 = vgetq_lane_u64::<1>(x);
    MatchTableIter {
        seg,
        low: low.to_ne_bytes(),
        high: high.to_ne_bytes(),
        pos: None,
        low_upper: 8.min(actual_len),
        high_upper: 16.min(actual_len),
    }
}

#[cfg(test)]
mod tests {
    use super::{MatchTable, ALIGNMENT};
    use crate::lexer::{source::Source, UniquePromise};
    #[test]
    fn neon_find_non_ascii() {
        let table = seq_macro::seq!(b in 0u8..=255 {
            MatchTable::new([#(!(b.is_ascii_alphanumeric() || b == b'_' || b == b'$'),)*], true)
        });
        let data = [
            "AAAAAAAA\"\rAAAAAA",
            "AAAAAAAAAAAAAAA\"",
            "AAAAAAAAAAAAAAAA",
            "AAAAAAAA",
            "AAAAAAAA\r",
            "AAAAAAAAAAAAAAA\r",
        ]
        .map(|x| Source::new(x, UniquePromise::new_for_tests()));
        let expected = [
            (vec![Some((8, b'"')), Some((0, b'\r')), None], ALIGNMENT),
            (vec![Some((15, b'"')), None], ALIGNMENT),
            (vec![None], ALIGNMENT),
            (vec![None], 8),
            (vec![Some((8, b'\r')), None], 9),
            (vec![Some((15, b'\r')), None], ALIGNMENT),
        ];

        for (idx, d) in data.into_iter().enumerate() {
            let pos = d.position();
            let (data, actual_len) =
                unsafe { pos.peek_n_with_padding::<ALIGNMENT>(d.end_addr()) }.unwrap();
            let mut result = table.matches(&data, actual_len);
            for val in &expected[idx].0 {
                assert_eq!(result.next(), *val);
            }
            assert_eq!(actual_len, expected[idx].1);
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
            let mut result = table.matches(&data, actual_len);
            assert_eq!((result.next(), actual_len), expected[idx]);
        }
    }
}
