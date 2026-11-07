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

                // Fast path
                if let Ok(value) = u32::from_str_radix(s, radix) {
                    return f64::from(value);
                }

                let bits_per_digit = radix.ilog2();
                let mut significand = 0_u64;
                let mut exponent = 0_u32;
                let mut sticky = 0_u32;
                for c in bytes {
                    let Some(digit) = char::from(c).to_digit(radix) else {
                        return f64::NAN;
                    };
                    if significand < (1 << 54) {
                        significand = (significand << bits_per_digit) | u64::from(digit);
                    } else {
                        exponent = (exponent + bits_per_digit).min(1024);
                        sticky |= digit;
                    }
                }
                if exponent > 1023 {
                    return f64::INFINITY;
                }

                // Preserve discarded nonzero digits below the rounding bit so the cast
                // distinguishes exact ties from values above them.
                significand |= u64::from(sticky != 0);
                #[expect(clippy::cast_precision_loss)]
                let value = significand as f64;
                return value * f64::from_bits(u64::from(exponent + 1023) << 52);
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
    use num_bigint::BigUint;

    use super::StringToNumber;

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn rounds_nondecimal_strings_once() {
        for bits in 53..128 {
            let base = 1_u128 << bits;
            let half_ulp = 1_u128 << (bits - 53);
            for offset in [
                half_ulp - 1,
                half_ulp,
                half_ulp + 1,
                3 * half_ulp - 1,
                3 * half_ulp,
                3 * half_ulp + 1,
            ] {
                let value = base + offset;
                for (prefix, digits) in [
                    ("0b", format!("{value:b}")),
                    ("0o", format!("{value:o}")),
                    ("0x", format!("{value:x}")),
                    ("0B", format!("{value:b}")),
                    ("0O", format!("{value:o}")),
                    ("0X", format!("{value:X}")),
                ] {
                    for leading in ["", "00000000000000000000"] {
                        let input = format!("{prefix}{leading}{digits}");
                        assert_eq!(input.as_str().string_to_number(), value as f64, "{input}");
                    }
                }
            }
        }
    }

    #[test]
    fn rounds_nondecimal_strings_at_float_limits() {
        for (binary, expected) in [
            (
                format!("1{}1{}1", "0".repeat(52), "0".repeat(969)),
                f64::from_bits(0x7fe0_0000_0000_0001),
            ),
            (format!("{}{}", "1".repeat(53), "0".repeat(971)), f64::MAX),
            (format!("{}0{}", "1".repeat(53), "1".repeat(970)), f64::MAX),
            (format!("{}{}", "1".repeat(54), "0".repeat(970)), f64::INFINITY),
            (format!("1{}", "0".repeat(1100)), f64::INFINITY),
            ("0".repeat(1100), 0.0),
            (format!("{}1", "0".repeat(1100)), 1.0),
        ] {
            let integer = BigUint::parse_bytes(binary.as_bytes(), 2).unwrap();
            for (radix, prefix) in [(2, "0b"), (8, "0o"), (16, "0x")] {
                let digits = integer.to_str_radix(radix);
                for leading in ["", "0".repeat(1100).as_str()] {
                    let input = format!("{prefix}{leading}{digits}");
                    assert_eq!(input.as_str().string_to_number(), expected, "{input}");
                    for suffix in ["g", "_", "\u{00e9}"] {
                        let invalid = format!("{input}{suffix}");
                        assert!(invalid.as_str().string_to_number().is_nan(), "{invalid}");
                    }
                }
            }
        }
    }

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
