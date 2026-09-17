use num_bigint::BigInt;
use num_traits::{Num, Zero};

use oxc_str::JSStr;

/// `StringToBigInt`
///
/// <https://tc39.es/ecma262/#sec-stringtobigint>
pub trait StringToBigInt<'a> {
    fn string_to_big_int(&self) -> Option<BigInt>;
}

impl StringToBigInt<'_> for JSStr<'_> {
    fn string_to_big_int(&self) -> Option<BigInt> {
        // A string containing a lone surrogate is not a StringIntegerLiteral,
        // so it converts to undefined like any other non-numeric text.
        self.as_str().and_then(|value| value.string_to_big_int())
    }
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

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use oxc_allocator::Allocator;
    use oxc_str::{JSStr, JSStrBuilder};

    use super::StringToBigInt;

    #[test]
    fn js_str_converts_like_str_unless_it_has_a_lone_surrogate() {
        let allocator = Allocator::new();
        assert_eq!(JSStr::from(" 12 ").string_to_big_int(), Some(BigInt::from(12)));
        let mut builder = JSStrBuilder::new_in(&&allocator);
        builder.push_str("1");
        builder.push_code_unit(0xD800);
        assert_eq!(builder.into_js_str().string_to_big_int(), None);
    }
}
