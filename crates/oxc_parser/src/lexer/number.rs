//! Parsing utilities for converting Javascript numbers to Rust f64
//! code copied from [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/parser/src/numeric_value.rs)

use num_bigint::BigInt;

use super::kind::Kind;

// the string passed in has `_` removed from the lexer
pub fn parse_int(s: &str, kind: Kind) -> Result<f64, &'static str> {
    match kind {
        Kind::Decimal => parse_float(s),
        Kind::Binary => Ok(parse_binary(&s[2..])),
        Kind::Octal => {
            let s = if s.starts_with("0o") || s.starts_with("0O") {
                &s[2..]
            } else {
                s // legacy octal
            };
            Ok(parse_octal(s))
        }
        Kind::Hex => Ok(parse_hex(&s[2..])),
        _ => unreachable!(),
    }
}

pub fn parse_float(s: &str) -> Result<f64, &'static str> {
    s.parse::<f64>().map_err(|_| "invalid float")
}

fn parse_binary(s: &str) -> f64 {
    debug_assert!(!s.is_empty());

    let mut result = 0_f64;

    for c in s.as_bytes().iter().filter(|s| s != &&b'_') {
        #[allow(clippy::cast_lossless)]
        let value = (c - b'0') as f64;
        result = result.mul_add(2.0, value);
    }

    result
}

fn parse_octal(s: &str) -> f64 {
    debug_assert!(!s.is_empty());

    let mut result = 0_f64;

    for c in s.as_bytes().iter().filter(|s| s != &&b'_') {
        #[allow(clippy::cast_lossless)]
        let value = (c - b'0') as f64;
        result = result.mul_add(8.0, value);
    }

    result
}

fn parse_hex(s: &str) -> f64 {
    debug_assert!(!s.is_empty());

    let mut result = 0_f64;

    for c in s.as_bytes().iter().filter(|s| s != &&b'_') {
        let value = match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'F' => c - b'A' + 10,
            b'a'..=b'f' => c - b'a' + 10,
            _ => unreachable!("invalid hex syntax {}", s),
        };
        #[allow(clippy::cast_lossless)]
        let value = value as f64;
        result = result.mul_add(16.0, value);
    }

    result
}

pub fn parse_big_int(s: &str, kind: Kind) -> Result<BigInt, &'static str> {
    let s = match kind {
        Kind::Decimal => s,
        Kind::Binary | Kind::Octal | Kind::Hex => &s[2..],
        _ => unreachable!(),
    };
    let radix = match kind {
        Kind::Decimal => 10,
        Kind::Binary => 2,
        Kind::Octal => 8,
        Kind::Hex => 16,
        _ => unreachable!(),
    };
    BigInt::parse_bytes(s.as_bytes(), radix).ok_or("invalid bigint")
}
