#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::lexer::source::Source;
use itertools::Itertools;

const ALIGNMENT: usize = 32;

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
    // source length must be at least 32 bytes
    #[inline]
    pub fn match_vectored(&self, source: &Source) -> usize {
        debug_assert!(
            source.remaining_len() >= ALIGNMENT,
            "source length must be at least {ALIGNMENT} bytes",
        );
        let ptr = source.as_ptr();
        // SAFETY:
        // we have checked that the source length is at least 32 bytes
        unsafe { self.match_delimiters_32_avx(ptr) }
    }

    // match 32 bytes at a time, return the position of the first found delimiter
    #[inline]
    #[allow(
        overflowing_literals,
        clippy::cast_sign_loss,
        clippy::ptr_as_ptr,
        clippy::cast_ptr_alignment
    )]
    unsafe fn match_delimiters_32_avx(&self, ptr: *const u8) -> usize {
        let data = _mm256_lddqu_si256(ptr as *const _);
        let rbms = _mm256_shuffle_epi8(self.table, data);
        let cols = _mm256_and_si256(self.lsh, _mm256_srli_epi16(data, 4));
        let bits = _mm256_and_si256(_mm256_shuffle_epi8(self.arf, cols), rbms);
        let v = _mm256_cmpeq_epi8(bits, _mm256_setzero_si256());
        let r = _mm256_movemask_epi8(v) as u32;
        // unmatched bits are 1, so we need to count the leading zeros
        r.trailing_ones() as usize
    }
}

// Create an ascii table with given delimiters, only fill
// in the given delimiters with 1, leave other positions as 0.
// then return a vector with each columns
// For example:
// delimiter = '\r'
// table[0][13] = 1
// the 13th element of returned vector is 1 0 0 0 0 0 0 0,
// due to the use of little-endian layout in AVX2, so the result
// is 0x01
//
//     0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15
//
//  0  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//                                   \n       \r
//  1  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//  2  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     \w !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
//  3  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
//  4  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
//  5  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
//  6  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
//  7  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  del
#[inline]
#[allow(clippy::cast_possible_truncation)]
unsafe fn tabulate<const N: usize>(delimiters: [u8; N]) -> __m256i {
    let mut table = [false; 128];
    for delimiter in delimiters {
        debug_assert!(*delimiter < 128, "delimiter must be an ASCII character");
        table[*delimiter as usize] = true;
    }

    let mut table_iter = (0..16)
        .map(|i| (0..8).rev().fold(0, |acc, j| (acc << 1) | u32::from(table[i * 8 + j])) as i8);

    // itertools only support 12 elements tuple, so we need to split it into two parts
    let (e00, e01, e02, e03, e04, e05, e06, e07, e08, e09, e10, e11) =
        table_iter.next_tuple().unwrap();
    let (e12, e13, e14, e15) = table_iter.next_tuple().unwrap();
    #[rustfmt::skip]
    _mm256_setr_epi8(
        e00, e01, e02, e03, e04, e05, e06, e07, e08, e09, e10, e11, e12, e13, e14, e15,
        e00, e01, e02, e03, e04, e05, e06, e07, e08, e09, e10, e11, e12, e13, e14, e15,
    )
}
