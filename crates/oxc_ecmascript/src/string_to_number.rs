use oxc_str::JSStr;
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};

pub trait StringToNumber {
    fn string_to_number(&self) -> f64;
}

impl StringToNumber for JSStr<'_> {
    fn string_to_number(&self) -> f64 {
        // A string containing a lone surrogate can never be a StringNumericLiteral,
        // so it converts to NaN.
        self.as_str().map_or(f64::NAN, |value| value.string_to_number())
    }
}

/// `StringToNumber`
///
/// <https://tc39.es/ecma262/#sec-stringtonumber>
impl StringToNumber for &str {
    fn string_to_number(&self) -> f64 {
        let s = self.trim_matches(is_str_white_space_char);
        match s {
            "" => return 0.0,
            "-Infinity" => return f64::NEG_INFINITY,
            "Infinity" | "+Infinity" => return f64::INFINITY,
            _ => {
                // Make sure that no further variants of "infinity" are parsed by `f64::parse`.
                // Note that alphabetical characters are not case-sensitive.
                // <https://doc.rust-lang.org/std/primitive.f64.html#method.from_str>
                let mut bytes = s.trim_start_matches(['-', '+']).bytes();
                if bytes
                    .next()
                    .filter(|c| c.eq_ignore_ascii_case(&b'i'))
                    .and_then(|_| bytes.next().filter(|c| c.eq_ignore_ascii_case(&b'n')))
                    .and_then(|_| bytes.next().filter(|c| c.eq_ignore_ascii_case(&b'f')))
                    .is_some()
                {
                    return f64::NAN;
                }
            }
        }

        let mut bytes = s.bytes();

        if s.len() > 2 && bytes.next() == Some(b'0') {
            // `| 32` converts upper case ASCII letters to lower case.
            // A bit more efficient than testing for `b'x' | b'X'`.
            // https://godbolt.org/z/Korrhd4TE
            let radix: u32 = match bytes.next().unwrap() | 32 {
                b'x' => 16,
                b'o' => 8,
                b'b' => 2,
                _ => 0,
            };

            if radix != 0 {
                let s = &s[2..];

                // `from_str_radix` accepts a leading sign, but the ECMAScript grammar for
                // non-decimal integer literals does not allow one.
                if s.starts_with(['+', '-']) {
                    return f64::NAN;
                }

                // Fast path
                if let Ok(value) = u32::from_str_radix(s, radix) {
                    return f64::from(value);
                }

                // Slow path
                //
                // The mathematical value of the literal must be rounded to the nearest
                // double in a single step, but accumulating into an `f64` (even with
                // `mul_add`) rounds at every step and can be off by 1 ULP for values
                // beyond 2^53. Instead, accumulate exactly into a `u128`; once it
                // overflows, the remaining digits only contribute lower-order bits
                // (all non-decimal radices are powers of two), which are tracked as a
                // sticky "inexact" flag for a final round-to-nearest-even.
                let bits_per_digit = radix.trailing_zeros();
                let mut mantissa: u128 = 0;
                let mut dropped_digits: u32 = 0;
                let mut dropped_nonzero = false;
                for c in s.bytes() {
                    let Some(digit) = char::from(c).to_digit(radix) else {
                        return f64::NAN;
                    };
                    let Some(next) = mantissa
                        .checked_mul(u128::from(radix))
                        .and_then(|m| m.checked_add(u128::from(digit)))
                    else {
                        dropped_digits += 1;
                        dropped_nonzero |= digit != 0;
                        continue;
                    };
                    mantissa = next;
                }

                if mantissa == 0 {
                    // All digits were zero.
                    return 0.0;
                }
                let bit_len = 128 - mantissa.leading_zeros();

                if dropped_digits == 0 && bit_len <= 53 {
                    // Exact: the value fits in 53 significand bits.
                    #[expect(clippy::cast_precision_loss)]
                    return mantissa as f64;
                }

                // The top 53 bits plus one guard bit always fall inside `mantissa`
                // here: with dropped digits `bit_len >= 124`, and without them
                // `bit_len > 53`.
                let rounded = mantissa >> (bit_len - 53);
                let guard = (mantissa >> (bit_len - 54)) & 1 == 1;
                let inexact = dropped_nonzero || (mantissa & ((1_u128 << (bit_len - 54)) - 1)) != 0;
                let rounded =
                    if guard && (inexact || rounded & 1 == 1) { rounded + 1 } else { rounded };

                let exponent =
                    bit_len.cast_signed() + (dropped_digits * bits_per_digit).cast_signed() - 53;
                #[expect(clippy::cast_precision_loss)]
                return rounded as f64 * 2.0_f64.powi(exponent);
            }
        }

        s.parse::<f64>().unwrap_or(f64::NAN)
    }
}

/// whether the char is a StrWhiteSpaceChar
/// <https://tc39.es/ecma262/#sec-tonumber-applied-to-the-string-type>
fn is_str_white_space_char(c: char) -> bool {
    is_white_space(c) || is_line_terminator(c)
}

#[cfg(test)]
mod tests {
    use super::StringToNumber;

    #[test]
    fn trims_ecmascript_whitespace_from_both_ends() {
        const WHITESPACE: &str = "\u{0009}\u{000B}\u{000C}\u{0020}\u{00A0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{202F}\u{205F}\u{3000}\u{FEFF}\u{000A}\u{000D}\u{2028}\u{2029}";

        assert_eq!(format!("{WHITESPACE}1{WHITESPACE}").as_str().string_to_number(), 1.0);
        assert_eq!(format!("{WHITESPACE}0x10{WHITESPACE}").as_str().string_to_number(), 16.0);
        assert_eq!(
            format!("{WHITESPACE}Infinity{WHITESPACE}").as_str().string_to_number(),
            f64::INFINITY
        );
        assert_eq!(format!("{WHITESPACE}{WHITESPACE}").as_str().string_to_number(), 0.0);
    }

    #[test]
    fn does_not_trim_non_ecmascript_whitespace() {
        assert!("1\u{0085}".string_to_number().is_nan());
    }

    #[test]
    fn js_str_converts_like_str_unless_it_has_a_lone_surrogate() {
        let allocator = oxc_allocator::Allocator::new();
        assert_eq!(oxc_str::JSStr::from(" 12 ").string_to_number(), 12.0);
        let mut builder = oxc_str::JSStrBuilder::new_in(&&allocator);
        builder.push_str("1");
        builder.push_code_unit(0xD800);
        assert!(builder.into_js_str().string_to_number().is_nan());
    }
}

#[test]
fn non_decimal_integer_literals_reject_signs() {
    // The grammar for `NonDecimalIntegerLiteral` does not allow a sign, but
    // `u32::from_str_radix` accepts one.
    assert!("0x+1".string_to_number().is_nan());
    assert!("0x-1".string_to_number().is_nan());
    assert!("0b+1".string_to_number().is_nan());
    assert!("0o+1".string_to_number().is_nan());
    assert!("0X-AB".string_to_number().is_nan());
    assert!("+0x1".string_to_number().is_nan());
}

#[test]
fn non_decimal_large_values_round_to_nearest_even() {
    // 0x451a7bb930fb529 is 311214273333277993 exactly; the nearest double is
    // 311214273333278016 (step-by-step f64 accumulation gives ...777952).
    assert_eq!("0x451a7bb930fb529".string_to_number(), 311_214_273_333_278_016.0);
    // 2^53 + 1 is an exact tie between 2^53 and 2^53 + 2; ties-to-even picks 2^53.
    assert_eq!("0x20000000000001".string_to_number(), 9_007_199_254_740_992.0);
    // 2^53 + 3 is nearer to 2^53 + 4.
    assert_eq!("0x20000000000003".string_to_number(), 9_007_199_254_740_996.0);
    // 2^54 + 1 is a tie; ties-to-even picks 2^54.
    assert_eq!("0x40000000000001".string_to_number(), 18_014_398_509_481_984.0);
    // 2^64 - 1 rounds up to 2^64.
    assert_eq!("0xFFFFFFFFFFFFFFFF".string_to_number(), 18_446_744_073_709_551_616.0);
    // 2^160: mantissa overflows u32 and the remaining zero digits are dropped.
    assert_eq!(
        concat!("0x1", "0000000000000000000000000000000000000000").string_to_number(),
        1_461_501_637_330_902_918_203_684_832_716_283_019_655_932_542_976.0
    );
}

#[test]
fn non_decimal_sticky_digits_break_ties() {
    // (2^53 + 1) * 2^112 is an exact tie; appending a further nonzero digit makes
    // the value inexact, so it rounds up to (2^52 + 1) * 2^117.
    assert_eq!(
        concat!("0x20000000000001", "0000000000000000000000000000").string_to_number(),
        2_f64.powi(165)
    );
    assert_eq!(
        concat!("0x20000000000001", "0000000000000000000000000000", "1").string_to_number(),
        (2_f64.powi(52) + 1.0) * 2_f64.powi(117)
    );
}

#[test]
fn non_decimal_invalid_and_zero() {
    assert_eq!("0x00000000000000000000000000000000000000".string_to_number(), 0.0);
    assert!("0x1g".string_to_number().is_nan());
    assert!("0x".string_to_number().is_nan());
    assert_eq!(concat!("0x", "0000000000000000000000000000000001").string_to_number(), 1.0);
}
