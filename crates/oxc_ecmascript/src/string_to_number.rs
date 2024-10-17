pub trait StringToNumber {
    fn string_to_number(&self) -> f64;
}

/// `StringToNumber`
///
/// <https://tc39.es/ecma262/#sec-stringtonumber>
impl StringToNumber for &str {
    fn string_to_number(&self) -> f64 {
        let s = self.trim_matches(is_trimmable_whitespace);

        match s {
            "" => return 0.0,
            "-Infinity" => return f64::NEG_INFINITY,
            "Infinity" | "+Infinity" => return f64::INFINITY,
            // Make sure that no further variants of "infinity" are parsed.
            "inf" | "-inf" | "+inf" => return f64::NAN,
            _ => {}
        }

        let mut bytes = s.bytes();

        if s.len() > 2 && bytes.next() == Some(b'0') {
            let radix: u32 = match bytes.next() {
                Some(b'x' | b'X') => 16,
                Some(b'o' | b'O') => 8,
                Some(b'b' | b'B') => 2,
                _ => 0,
            };

            if radix != 0 {
                let s = &s[2..];

                // Fast path
                if let Ok(value) = u32::from_str_radix(s, radix) {
                    return f64::from(value);
                }

                // Slow path
                let mut value: f64 = 0.0;
                for c in bytes {
                    if let Some(digit) = char::from(c).to_digit(radix) {
                        value = value.mul_add(f64::from(radix), f64::from(digit));
                    } else {
                        return f64::NAN;
                    }
                }
                return value;
            }
        }

        s.parse::<f64>().unwrap_or(f64::NAN)
    }
}

// <https://github.com/boa-dev/boa/blob/94d08fe4e68791ceca3c4b7d94ccc5f3588feeb3/core/string/src/lib.rs#L55>
/// Helper function to check if a `char` is trimmable.
pub(crate) const fn is_trimmable_whitespace(c: char) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as ecma standard does
    //
    // Rust uses \p{White_Space} by default, which also includes:
    // `\u{0085}' (next line)
    // And does not include:
    // '\u{FEFF}' (zero width non-breaking space)
    // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
    matches!(
        c,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' |
    // Unicode Space_Separator category
    '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' |
    // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
    '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}'
    )
}
