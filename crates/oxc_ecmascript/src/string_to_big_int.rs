use num_traits::Zero;

use num_bigint::BigInt;

/// `StringToBigInt`
///
/// <https://tc39.es/ecma262/#sec-stringtobigint>
pub trait StringToBigInt<'a> {
    fn string_to_big_int(&self) -> Option<BigInt>;
}

impl<'a> StringToBigInt<'a> for &str {
    fn string_to_big_int(&self) -> Option<BigInt> {
        if self.contains('\u{000b}') {
            // vertical tab is not always whitespace
            return None;
        }

        let s = self.trim();

        if s.is_empty() {
            return Some(BigInt::zero());
        }

        if s.len() > 2 && s.starts_with('0') {
            let radix: u32 = match s.chars().nth(1) {
                Some('x' | 'X') => 16,
                Some('o' | 'O') => 8,
                Some('b' | 'B') => 2,
                _ => 0,
            };

            if radix == 0 {
                return None;
            }

            return BigInt::parse_bytes(s[2..].as_bytes(), radix);
        }

        return BigInt::parse_bytes(s.as_bytes(), 10);
    }
}
