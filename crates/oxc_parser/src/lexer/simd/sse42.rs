#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::tabulate16;

pub const ALIGNMENT: usize = 16;

#[derive(Debug)]
pub struct MatchTable {
    table: [u8; ALIGNMENT],
    arf: [u8; ALIGNMENT],
    lsh: [u8; ALIGNMENT],
    reverse: bool,
}

impl MatchTable {
    pub const fn new(bytes: [bool; 256], reverse: bool) -> Self {
        let table = tabulate16(bytes, reverse);
        // arf: 0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000, 0b00000100, 0b00000010, 0b00000001
        // for input data match each row of the table, be like a mask
        #[rustfmt::skip]
        let arf = [
            0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let lsh = [0x0f; 16];
        Self { table, arf, lsh, reverse }
    }

    // match 16 bytes at a time, return the position of the first found delimiter
    #[inline]
    pub fn matches<'a>(
        &'a self,
        seg: &'a [u8; ALIGNMENT],
        actual_len: usize,
    ) -> impl Iterator<Item = (usize, u8)> + 'a {
        // SAFETY:
        // data is aligned and has ALIGNMENT bytes
        unsafe { self.match_delimiters(seg, actual_len) }
    }

    // match 16 bytes at a time, return the position of the first found delimiter
    #[inline]
    #[allow(overflowing_literals, clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    unsafe fn match_delimiters<'a>(
        &'a self,
        seg: &'a [u8; ALIGNMENT],
        actual_len: usize,
    ) -> MatchTableIter<'a> {
        let ptr = seg.as_ptr();
        let data = _mm_lddqu_si128(ptr.cast());
        // lower 4 bits of each byte in data are the column index
        // get the table column of each data byte
        let col = _mm_shuffle_epi8(_mm_lddqu_si128(self.table.as_ptr().cast()), data);
        // higher 4 bits of each byte in data are the row index
        let row_idx =
            _mm_and_si128(_mm_lddqu_si128(self.lsh.as_ptr().cast()), _mm_srli_epi16(data, 4));
        // get the table row of each data byte
        let row = _mm_shuffle_epi8(_mm_lddqu_si128(self.arf.as_ptr().cast()), row_idx);
        // row & col returns the matched delimiter in the table
        let bits = _mm_and_si128(row, col);
        // unmatched element's bits are 0b00000000
        // so, compare with zero, v[x] = 0b11111111 if the element x is unmatched
        let v = _mm_cmpeq_epi8(bits, _mm_setzero_si128());
        // get the leading bit of each byte, v = 0b000000001000...
        // if the byte is unmatched, the corresponding location in `r` is 1, opposite the bit is 0
        let r = _mm_movemask_epi8(v) as u16;
        // unmatched bits are 1, so we need to count the trailing ones(little-endian)
        let data_bits = if self.reverse { !r } else { r };
        MatchTableIter { seg, data_bits, actual_len, pos: 0 }
    }
}

struct MatchTableIter<'a> {
    seg: &'a [u8; ALIGNMENT],
    data_bits: u16, // each bit represents a byte in the segment
    actual_len: usize,
    pos: usize, // the last position of the matched byte
}

impl<'a> Iterator for MatchTableIter<'a> {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.pos..self.actual_len {
            let mask = 1 << i;
            // check if the byte is a zero
            if self.data_bits & mask == 0 {
                let offset = i - self.pos;
                // set next pos
                self.pos = i + 1;
                return Some((offset, self.seg[i]));
            }
        }
        None
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
