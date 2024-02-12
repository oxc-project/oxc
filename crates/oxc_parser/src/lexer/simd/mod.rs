#[cfg(target_feature = "avx2")]
mod avx2;
#[cfg(target_feature = "neon")]
mod neon;
#[cfg(all(not(target_feature = "avx2"), not(target_feature = "neon")))]
mod swar;

#[derive(Debug)]
pub(crate) struct LookupTable {
    #[cfg(target_feature = "avx2")]
    table: avx2::LookupTable,
    #[cfg(target_feature = "neon")]
    table: neon::LookupTable,
    #[cfg(all(not(target_feature = "avx2"), not(target_arch = "aarch64")))]
    table: swar::LookupTable,
}

impl LookupTable {
    #[cfg(target_feature = "avx2")]
    pub const ALIGNMENT: usize = avx2::LookupTable::ALIGNMENT;
    #[cfg(target_feature = "neon")]
    pub const ALIGNMENT: usize = neon::LookupTable::ALIGNMENT;
    #[cfg(all(not(target_feature = "avx2"), not(target_feature = "neon")))]
    pub const ALIGNMENT: usize = swar::LookupTable::ALIGNMENT;

    pub fn new<const N: usize>(delimiters: [u8; N]) -> Self {
        Self {
            #[cfg(target_feature = "avx2")]
            table: avx2::LookupTable::new(delimiters),
            #[cfg(target_feature = "neon")]
            table: neon::LookupTable::new(delimiters),
            #[cfg(all(not(target_feature = "avx2"), not(target_feature = "neon")))]
            table: swar::LookupTable::new(delimiters),
        }
    }

    #[inline]
    pub fn match_vectored(&self, data: &[u8; Self::ALIGNMENT]) -> Option<usize> {
        self.table.match_vectored(data)
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
