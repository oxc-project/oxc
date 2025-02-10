pub trait StringToNumber {
    fn string_to_number(&self) -> f64;
}

/// `StringToNumber`
///
/// <https://tc39.es/ecma262/#sec-stringtonumber>
impl StringToNumber for &str {
    fn string_to_number(&self) -> f64 {
        let s = *self;
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
