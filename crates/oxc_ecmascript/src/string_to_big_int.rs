use num_bigint::BigInt;
use num_traits::{Num, Zero};

use oxc_str::JSStr;
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};

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
        // `StringIntegerLiteral` allows `StrWhiteSpace` (`WhiteSpace` and `LineTerminator`)
        // around the literal. This is not the same set as Rust's `char::is_whitespace`.
        let s = self.trim_matches(|c| is_white_space(c) || is_line_terminator(c));

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
                _ => 10,
            };

            if radix != 10 {
                let digits = &s[2..];
                // `NonDecimalIntegerLiteral[~Sep]` has no sign and no separators,
                // but `from_str_radix` accepts both.
                if !digits.chars().all(|c| c.is_digit(radix)) {
                    return None;
                }
                return BigInt::from_str_radix(digits, radix).ok();
            }
        }

        // `SignedInteger[~Sep]`: an optional sign followed by decimal digits, without separators.
        let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        BigInt::from_str_radix(s, 10).ok()
    }
}

#[cfg(test)]
mod test {
    use num_bigint::BigInt;
    use oxc_allocator::Allocator;
    use oxc_str::{JSStr, JSStrBuilder};

    use super::StringToBigInt;

    #[test]
    fn test_string_to_big_int() {
        assert_eq!("".string_to_big_int(), Some(BigInt::from(0)));
        assert_eq!(" 12 ".string_to_big_int(), Some(BigInt::from(12)));
        assert_eq!("\u{000B}1\u{FEFF}".string_to_big_int(), Some(BigInt::from(1)));
        assert_eq!("012".string_to_big_int(), Some(BigInt::from(12)));
        assert_eq!("-12".string_to_big_int(), Some(BigInt::from(-12)));
        assert_eq!("+12".string_to_big_int(), Some(BigInt::from(12)));
        assert_eq!("0x10".string_to_big_int(), Some(BigInt::from(16)));
        assert_eq!("0O10".string_to_big_int(), Some(BigInt::from(8)));
        assert_eq!("0b10".string_to_big_int(), Some(BigInt::from(2)));
        // U+0085 is not `WhiteSpace` or `LineTerminator` in ECMAScript
        assert_eq!("\u{0085}1".string_to_big_int(), None);
        assert_eq!("1_0".string_to_big_int(), None);
        assert_eq!("0x1_0".string_to_big_int(), None);
        assert_eq!("0x-1".string_to_big_int(), None);
        assert_eq!("0x+1".string_to_big_int(), None);
        assert_eq!("-0x1".string_to_big_int(), None);
        assert_eq!("0x".string_to_big_int(), None);
        assert_eq!("-".string_to_big_int(), None);
        assert_eq!("1n".string_to_big_int(), None);
        assert_eq!("1.0".string_to_big_int(), None);
        assert_eq!("1 2".string_to_big_int(), None);
    }

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
