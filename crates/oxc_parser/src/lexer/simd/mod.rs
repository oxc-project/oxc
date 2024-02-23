#[cfg(target_feature = "avx2")]
mod avx2;
#[cfg(target_feature = "neon")]
mod neon;
#[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.2"))]
mod sse42;

#[cfg(all(
    not(target_feature = "avx2"),
    not(target_feature = "sse4.2"),
    not(target_feature = "neon")
))]
use crate::lexer::search::SafeByteMatchTable;

#[cfg(target_feature = "avx2")]
pub const ALIGNMENT: usize = avx2::ALIGNMENT;
#[cfg(target_feature = "neon")]
pub const ALIGNMENT: usize = neon::ALIGNMENT;
#[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.2"))]
pub const ALIGNMENT: usize = sse42::ALIGNMENT;
#[cfg(all(
    not(target_feature = "avx2"),
    not(target_feature = "sse4.2"),
    not(target_feature = "neon")
))]
pub const ALIGNMENT: usize = 16;

#[derive(Debug)]
pub(crate) struct MatchTable {
    #[cfg(target_feature = "avx2")]
    table: avx2::MatchTable,
    #[cfg(target_feature = "neon")]
    table: neon::MatchTable,
    #[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.2"))]
    table: sse42::MatchTable,
    #[cfg(all(
        not(target_feature = "avx2"),
        not(target_feature = "sse4.2"),
        not(target_feature = "neon")
    ))]
    table: SafeByteMatchTable,
}

impl MatchTable {
    #[allow(unused_variables)]
    pub const fn new(bytes: [bool; 256], reverse: bool) -> Self {
        Self {
            #[cfg(target_feature = "avx2")]
            table: avx2::MatchTable::new(bytes, reverse),
            #[cfg(target_feature = "neon")]
            table: neon::MatchTable::new(bytes, reverse),
            #[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.2"))]
            table: sse42::MatchTable::new(bytes, reverse),
            #[cfg(all(
                not(target_feature = "avx2"),
                not(target_feature = "sse4.2"),
                not(target_feature = "neon")
            ))]
            table: SafeByteMatchTable::new(bytes),
        }
    }

    #[inline]
    pub fn matches<'a>(
        &'a self,
        seg: &'a [u8; ALIGNMENT],
        actual_len: usize,
    ) -> impl Iterator<Item = (usize, u8)> + 'a {
        self.table.matches(seg, actual_len)
    }
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
#[allow(dead_code)]
#[inline]
const fn tabulate16(bytes: [bool; 256], reverse: bool) -> [u8; 16] {
    let mut table = [0u8; 16];
    let mut i = 0;
    while i < 256 {
        let set = if reverse { !bytes[i] } else { bytes[i] };
        if set {
            debug_assert!(i < 128, "delimiter must be an ASCII character");
            // lower 4 bits is the column index, higher 4 bits is the row index
            let (col, row) = (i & 0x0F, i >> 4);
            // use bitwise `or`` to combine the row with the same column
            table[col] |= 1 << row;
        }
        i += 1;
    }
    table
}

#[allow(dead_code)]
#[inline]
const fn tabulate32(bytes: [bool; 256], reverse: bool) -> [u8; 32] {
    let mut table = [0u8; 32];
    let mut i = 0;
    while i < 256 {
        let set = if reverse { !bytes[i] } else { bytes[i] };
        if set {
            debug_assert!(i < 128, "delimiter must be an ASCII character");
            // lower 4 bits is the column index, higher 4 bits is the row index
            let (col, row) = (i & 0x0F, i >> 4);
            // use bitwise `or`` to combine the row with the same column
            table[col] |= 1 << row;
            table[col + 16] |= 1 << row;
        }
        i += 1;
    }
    table
}
