#[cfg(target_feature = "avx2")]
mod avx2;
#[cfg(target_feature = "neon")]
mod neon;
#[cfg(all(not(target_feature = "avx2"), not(target_feature = "neon")))]
mod swar;

#[derive(Debug)]
pub(crate) struct MatchTable {
    #[cfg(target_feature = "avx2")]
    table: avx2::MatchTable,
    #[cfg(target_feature = "neon")]
    table: neon::MatchTable,
    #[cfg(all(not(target_feature = "avx2"), not(target_arch = "aarch64")))]
    table: swar::MatchTable,
}

impl MatchTable {
    #[cfg(target_feature = "avx2")]
    pub const ALIGNMENT: usize = avx2::MatchTable::ALIGNMENT;
    #[cfg(target_feature = "neon")]
    pub const ALIGNMENT: usize = neon::MatchTable::ALIGNMENT;
    #[cfg(all(not(target_feature = "avx2"), not(target_feature = "neon")))]
    pub const ALIGNMENT: usize = swar::MatchTable::ALIGNMENT;

    pub fn new(bytes: [bool; 256], reverse: bool) -> Self {
        Self {
            #[cfg(target_feature = "avx2")]
            table: avx2::MatchTable::new(bytes, reverse),
            #[cfg(target_feature = "neon")]
            table: neon::MatchTable::new(bytes, reverse),
            #[cfg(all(not(target_feature = "avx2"), not(target_feature = "neon")))]
            table: swar::MatchTable::new(bytes, reverse),
        }
    }

    #[inline]
    pub fn match_vectored(
        &self,
        data: &[u8; Self::ALIGNMENT],
        actual_len: usize,
    ) -> Option<(usize, u8)> {
        self.table.match_vectored(data).and_then(|(pos, b)| {
            if pos >= actual_len {
                None
            } else {
                Some((pos, b))
            }
        })
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
const fn tabulate(bytes: [bool; 256]) -> [u8; 16] {
    let mut table = [0u8; 16];
    let mut i = 0;
    loop {
        let set = bytes[i];
        if set {
            debug_assert!(i < 128, "delimiter must be an ASCII character");
            // lower 4 bits is the column index, higher 4 bits is the row index
            let (col, row) = (i & 0x0F, i >> 4);
            // use bitwise `or`` to combine the row with the same column
            table[col] |= 1 << row;
        }
        i += 1;
        if i == 256 {
            break;
        }
    }
    table
}

#[cfg(test)]
mod tests {
    use crate::lexer::simd::MatchTable;
    use crate::lexer::{source::Source, UniquePromise};
    #[test]
    fn neon_find_non_ascii() {
        let table = seq_macro::seq!(b in 0u8..=255 {
            // find non ascii
            MatchTable::new([#(b.is_ascii_alphanumeric() || b == b'_' || b == b'$',)*], true)
        });
        let data = [
            "AAAAAAAA\"\rAAAAAA",
            "AAAAAAAAAAAAAAA\"",
            "AAAAAAAAAAAAAAAA",
            "AAAAAAAA",
            "AAAAAAAA\"\rAAAAAA",
            "AAAAAAAAAAAAAAA\r",
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
            let result = table.match_vectored(&data, actual_len);
            assert_eq!((result, actual_len), expected[idx]);
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
                unsafe { pos.peek_n_with_padding::<{ MatchTable::ALIGNMENT }>(d.end_addr()) }
                    .unwrap();
            let result = table.match_vectored(&data, actual_len);
            assert_eq!((result, actual_len), expected[idx]);
        }
    }
}
