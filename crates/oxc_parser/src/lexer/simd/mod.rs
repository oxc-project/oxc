#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
mod avx2;
#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(all(not(target_feature = "avx2"), not(target_arch = "aarch64")))]
mod swar;

use crate::lexer::source::Source;
use once_cell::sync::Lazy;

pub(crate) struct Position {
    // the offset of the first found delimiter
    pub(crate) offset: Option<usize>,
    // the number of actual remaining bytes in the source
    // sometimes theres a chance that the source is shorter than the alignment with padding
    pub(crate) actual_len: usize,
    // the maximum length of each segment, in avx2, it's 32 bytes
    #[allow(dead_code)]
    pub(crate) alignment: usize,
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
static AVX2_STRING_LITERAL_LOOKUP_TABLE: Lazy<avx2::LookupTable<5>> =
    Lazy::new(|| avx2::LookupTable::new([b'\r', b'\n', b'"', b'\'', b'\\']));

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
pub(crate) fn string_literal_lookup(source: &Source) -> Position {
    let (offset, actual_len) = AVX2_STRING_LITERAL_LOOKUP_TABLE.match_vectored(source);
    Position { offset, actual_len, alignment: avx2::ALIGNMENT }
}

#[cfg(target_arch = "aarch64")]
static NEON_STRING_LITERAL_LOOKUP_TABLE: Lazy<neon::LookupTable<5>> =
    Lazy::new(|| neon::LookupTable::new([b'\r', b'\n', b'"', b'\'', b'\\']));

#[cfg(target_arch = "aarch64")]
pub(crate) fn string_literal_lookup(source: &Source) -> Position {
    let (offset, actual_len) = NEON_STRING_LITERAL_LOOKUP_TABLE.match_vectored(source);
    Position { offset, actual_len, alignment: neon::ALIGNMENT }
}

#[cfg(all(not(target_feature = "avx2"), not(target_arch = "aarch64")))]
static SWAR_STRING_LITERAL_LOOKUP_TABLE: Lazy<swar::LookupTable<5>> =
    Lazy::new(|| swar::LookupTable::new([b'\r', b'\n', b'"', b'\'', b'\\']));

#[cfg(all(not(target_feature = "avx2"), not(target_arch = "aarch64")))]
pub(crate) fn string_literal_lookup(source: &Source) -> Position {
    let (offset, actual_len) = SWAR_STRING_LITERAL_LOOKUP_TABLE.match_vectored(source);
    Position { offset, actual_len, alignment: swar::ALIGNMENT }
}

// Create an ascii table with given delimiters, only fill
// in the given delimiters with 1, leave other positions as 0.
// then return a vector with each columns
//
// For example:
// ```
// if delimiter = '\r' then table[0][13] = 1
// and tabulate = [0,0,0,0,0,0,0,0,0,0,0,0,0,0b1000000,0,0]
// ```
// The row's value of the 13th colum in table is `1 0 0 0 0 0 0 0`,
//
// Create a 8x16 table for ASCII characters, and arrange in rows according
// to ASCII code.
//
// Table:
// r\c 0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15
//
//  0  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
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
//
// Returns a 16 column(element) vector, each column is a 8-bit mask for
// match the delimiter.
#[inline]
fn tabulate<const N: usize>(delimiters: [u8; N]) -> [u8; 16] {
    let mut table = [0u8; 16];
    for d in delimiters {
        debug_assert!(d < 128, "delimiter must be an ASCII character");
        // lower 4 bits is the column index, higher 4 bits is the row index
        let (col, row) = (d & 0x0F, d >> 4);
        // use bitwise `or`` to combine the row with the same column
        table[col as usize] |= 1 << row;
    }
    table
}
