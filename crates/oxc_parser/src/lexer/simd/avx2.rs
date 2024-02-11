#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::tabulate;
use crate::lexer::source::Source;

pub(crate) const ALIGNMENT: usize = 32;

pub struct LookupTable<const N: usize> {
    table: __m256i,
    arf: __m256i,
    lsh: __m256i,
}

impl<const N: usize> LookupTable<N> {
    #[allow(non_snake_case, overflowing_literals)]
    pub fn new(delimiters: [u8; N]) -> Self {
        // SAFETY:
        // delimiters must be an ASCII character and checked by `tabulate`
        unsafe {
            let table = tabulate(delimiters);
            let table = _mm256_lddqu_si256(table.as_ptr().cast());
            // arf: 0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000, 0b00000100, 0b00000010, 0b00000001
            // for input data match each row of the table, be like a mask
            #[rustfmt::skip]
            let arf = _mm256_setr_epi8(
                0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            );
            let lsh = _mm256_set1_epi8(0x0f);
            Self { table, arf, lsh }
        }
    }

    // match 32 bytes at a time, return the position of the first found delimiter
    #[inline]
    pub fn match_vectored(&self, source: &Source) -> (Option<usize>, usize) {
        if let Some((seg, actual_len)) = source.peek_n_with_padding::<ALIGNMENT>() {
            let ptr = seg.as_ptr();
            // SAFETY:
            // seg is aligned and has ALIGNMENT bytes
            (unsafe { self.match_delimiters(ptr) }, actual_len)
        } else {
            (None, 0)
        }
    }

    // match 32 bytes at a time, return the position of the first found delimiter
    #[inline]
    #[allow(overflowing_literals, clippy::cast_sign_loss)]
    unsafe fn match_delimiters(&self, ptr: *const u8) -> Option<usize> {
        let data = _mm256_lddqu_si256(ptr.cast());
        // lower 4 bits of each byte in data are the column index
        // get the table column of each data byte
        let col = _mm256_shuffle_epi8(self.table, data);
        // higher 4 bits of each byte in data are the row index
        let row_idx = _mm256_and_si256(self.lsh, _mm256_srli_epi16(data, 4));
        // get the table row of each data byte
        let row = _mm256_shuffle_epi8(self.arf, row_idx);
        // row & col returns the matched delimiter in the table
        let bits = _mm256_and_si256(row, col);
        // unmatched element's bits are 0b00000000
        // so, compare with zero, v[x] = 0b11111111 if the element x is unmatched
        let v = _mm256_cmpeq_epi8(bits, _mm256_setzero_si256());
        // get the leading bit of each byte, v = 0b000000001000...
        // if the byte is unmatched, the corresponding location in `r` is 1, opposite the bit is 0
        let r = _mm256_movemask_epi8(v) as u32;
        // unmatched bits are 1, so we need to count the trailing ones(little-endian)
        let unmatched = r.trailing_ones() as usize;
        // reach the end of the segment, so no delimiter found
        if unmatched == ALIGNMENT {
            None
        } else {
            Some(unmatched)
        }
    }
}
