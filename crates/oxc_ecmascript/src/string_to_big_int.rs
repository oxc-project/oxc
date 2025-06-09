use num_bigint::BigInt;
use num_traits::{Num, Zero};

/// `StringToBigInt`
///
/// <https://tc39.es/ecma262/#sec-stringtobigint>
pub trait StringToBigInt<'a> {
    fn string_to_big_int(&self) -> Option<BigInt>;
}

impl StringToBigInt<'_> for &str {
    fn string_to_big_int(&self) -> Option<BigInt> {
        if self.contains('\u{000b}') {
            // vertical tab is not always whitespace
            return None;
        }

        let s = self.trim();

        if s.is_empty() {
            return Some(BigInt::zero());
        }

        if s.len() > 2 && s.as_bytes()[0] == b'0' {
            // `| 32` converts upper case ASCII letters to lower case.
            // A bit more efficient than testing for `b'x' | b'X'`.
            // https://godbolt.org/z/Korrhd4TE
            let radix: u32 = match s.as_bytes()[1] | 32 {
                b'x' => 16,
                b'o' => 8,
                b'b' => 2,
                _ => return None,
            };

            return BigInt::from_str_radix(&s[2..], radix).ok();
        }

        BigInt::from_str_radix(s, 10).ok()
    }
}
