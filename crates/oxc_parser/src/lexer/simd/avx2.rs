#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::tabulate32;

#[derive(Debug)]
pub struct MatchTable {
    table: [u8; 32],
    arf: [u8; 32],
    lsh: [u8; 32],
    reverse: bool,
}

impl MatchTable {
    pub const ALIGNMENT: usize = 32;

    pub const fn new(bytes: [bool; 256], reverse: bool) -> Self {
        let table = tabulate32(bytes);
        // arf: 0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000, 0b00000100, 0b00000010, 0b00000001
        // for input data match each row of the table, be like a mask
        #[rustfmt::skip]
        let arf = [
            0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let lsh = [0x0f; 32];
        Self { table, arf, lsh, reverse }
    }

    // match 32 bytes at a time, return the position of the first found delimiter
    #[inline]
    pub fn matches(&self, data: &[u8; Self::ALIGNMENT], actual_len: usize) -> Option<(usize, u8)> {
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

    // match 32 bytes at a time, return the position of the first found delimiter
    #[inline]
    #[allow(overflowing_literals, clippy::cast_sign_loss)]
    unsafe fn match_delimiters(&self, ptr: *const u8) -> Option<usize> {
        let data = _mm256_lddqu_si256(ptr.cast());
        // lower 4 bits of each byte in data are the column index
        // get the table column of each data byte
        let col = _mm256_shuffle_epi8(_mm256_lddqu_si256(self.table.as_ptr().cast()), data);
        // higher 4 bits of each byte in data are the row index
        let row_idx = _mm256_and_si256(
            _mm256_lddqu_si256(self.lsh.as_ptr().cast()),
            _mm256_srli_epi16(data, 4),
        );
        // get the table row of each data byte
        let row = _mm256_shuffle_epi8(_mm256_lddqu_si256(self.arf.as_ptr().cast()), row_idx);
        // row & col returns the matched delimiter in the table
        let bits = _mm256_and_si256(row, col);
        // unmatched element's bits are 0b00000000
        // so, compare with zero, v[x] = 0b11111111 if the element x is unmatched
        let v = _mm256_cmpeq_epi8(bits, _mm256_setzero_si256());
        // get the leading bit of each byte, v = 0b000000001000...
        // if the byte is unmatched, the corresponding location in `r` is 1, opposite the bit is 0
        let r = _mm256_movemask_epi8(v) as u32;
        // unmatched bits are 1, so we need to count the trailing ones(little-endian)
        let unmatched = if self.reverse { r.trailing_zeros() } else { r.trailing_ones() } as usize;
        // reach the end of the segment, so no delimiter found
        if unmatched == Self::ALIGNMENT {
            None
        } else {
            Some(unmatched)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MatchTable;
    use crate::lexer::{source::Source, UniquePromise};
    #[test]
    fn avx2_find_non_ascii() {
        let table = seq_macro::seq!(b in 0u8..=255 {
            // find non ascii
            MatchTable::new([#(b.is_ascii_alphanumeric() || b == b'_' || b == b'$',)*], true)
        });
        let data = [
            "AAAAAAAA\"\rAAAAAAAAAAAAAA\"\rAAAAAA",
            "AAAAAAAAAAAAAAA\"AAAAAAAAAAAAAAA\"",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAA",
            "AAAAAAAA\"\rAAAAAAAAAAAAAA\"\rAAAAAA",
            "AAAAAAAAAAAAAAA\rAAAAAAAAAAAAAAA\r",
        ]
        .map(|x| Source::new(x, UniquePromise::new_for_tests()));
        let expected = [
            (Some((8, b'"')), MatchTable::ALIGNMENT),
            (Some((15, b'"')), MatchTable::ALIGNMENT),
            (None, MatchTable::ALIGNMENT),
            (None, 8),
            (Some((8, b'\"')), MatchTable::ALIGNMENT),
            (Some((15, b'\r')), MatchTable::ALIGNMENT),
        ];

        for (idx, d) in data.into_iter().enumerate() {
            let pos = d.position();
            let (data, actual_len) =
                unsafe { pos.peek_n_with_padding::<{ MatchTable::ALIGNMENT }>(d.end_addr()) }
                    .unwrap();
            let result = table.matches(&data, actual_len);
            assert_eq!((result, actual_len), expected[idx]);
        }
    }

    #[test]
    fn avx2_find_single_quote_string() {
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
                unsafe { pos.peek_n_with_padding::<{ MatchTable::ALIGNMENT }>(d.end_addr()) }
                    .unwrap();
            let result = table.matches(&data, actual_len);
            assert_eq!((result, actual_len), expected[idx]);
        }
    }
}
