#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::tabulate;
use crate::lexer::source::Source;

pub(crate) const ALIGNMENT: usize = 32;

pub struct LookupTable<const N: usize> {
    table: __m256i, // for cols lookup
    arf: __m256i,   // for rows lookup
    lsh: __m256i,   // for shifting
}

impl<const N: usize> LookupTable<N> {
    #[allow(non_snake_case, overflowing_literals)]
    pub fn new(delimiters: [u8; N]) -> Self {
        // SAFETY:
        // delimiters must be an ASCII character and checked by `tabulate`
        unsafe {
            let table = tabulate(delimiters);
            let table = _mm256_lddqu_si256(table.as_ptr().cast());
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
        let rbms = _mm256_shuffle_epi8(self.table, data);
        let cols = _mm256_and_si256(self.lsh, _mm256_srli_epi16(data, 4));
        let bits = _mm256_and_si256(_mm256_shuffle_epi8(self.arf, cols), rbms);
        let v = _mm256_cmpeq_epi8(bits, _mm256_setzero_si256());
        let r = _mm256_movemask_epi8(v) as u32;
        // unmatched bits are 1, so we need to count the leading zeros
        let unmatched = r.trailing_ones() as usize;
        if unmatched == ALIGNMENT {
            None
        } else {
            Some(unmatched)
        }
    }
}
