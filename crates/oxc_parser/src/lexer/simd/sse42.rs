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
        let table = tabulate16(bytes);
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

    // match 32 bytes at a time, return the position of the first found delimiter
    #[inline]
    #[allow(overflowing_literals, clippy::cast_sign_loss)]
    unsafe fn match_delimiters(&self, ptr: *const u8) -> Option<usize> {
        let data = _mm_lddqu_si128(ptr.cast());
        // lower 4 bits of each byte in data are the column index
        // get the table column of each data byte
        let col = _mm_shuffle_epi8(_mm_lddqu_si128(self.table.as_ptr().cast()), data);
        // higher 4 bits of each byte in data are the row index
        let row_idx =
            _mm_and_si128(_mm_lddqu_si128(self.lsh.as_ptr().cast()), _mm_srli_epi16(data, 4));
        // get the table row of each data byte
        let row = _mm_shuffle_epi8(_mm256_lddqu_si128(self.arf.as_ptr().cast()), row_idx);
        // row & col returns the matched delimiter in the table
        let bits = _mm_and_si128(row, col);
        // unmatched element's bits are 0b00000000
        // so, compare with zero, v[x] = 0b11111111 if the element x is unmatched
        let v = _mm_cmpeq_epi8(bits, _mm_setzero_si128());
        // get the leading bit of each byte, v = 0b000000001000...
        // if the byte is unmatched, the corresponding location in `r` is 1, opposite the bit is 0
        let r = _mm_movemask_epi8(v) as u32;
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
