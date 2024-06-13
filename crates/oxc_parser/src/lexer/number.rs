//! Parsing utilities for converting Javascript numbers to Rust f64
//! code copied from [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/parser/src/numeric_value.rs)

use num_bigint::BigInt;
use num_traits::Num as _;
use std::borrow::Cow;

use super::kind::Kind;

pub fn parse_int(s: &str, kind: Kind, has_sep: bool) -> Result<f64, &'static str> {
    let s = if has_sep { Cow::Owned(s.replace('_', "")) } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));

    parse_int_without_underscores(&s, kind)
}

pub fn parse_float(s: &str, has_sep: bool) -> Result<f64, &'static str> {
    let s = if has_sep { Cow::Owned(s.replace('_', "")) } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));

    parse_float_without_underscores(&s)
}

/// This function assumes `s` has had all numeric separators (`_`) removed.
/// Parsing will fail if this assumption is violated.
fn parse_int_without_underscores(s: &str, kind: Kind) -> Result<f64, &'static str> {
    if kind == Kind::Decimal {
        return parse_float_without_underscores(s);
    }
    match kind {
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

/// This function assumes `s` has had all numeric separators (`_`) removed.
/// Parsing will fail if this assumption is violated.
fn parse_float_without_underscores(s: &str) -> Result<f64, &'static str> {
    s.parse::<f64>().map_err(|_| "invalid float")
}

/// NOTE: bit shifting here is is safe and much faster than f64::mul_add.
/// It's safe because we're sure this number is an integer - if it wasn't, it
/// would be a [`Kind::Float`] instead. It's fast because shifting usually takes
/// 1 clock cycle on the ALU, while multiplication+addition uses the FPU and is
/// much slower. Addtiionally, this loop often gets unrolled by rustc since
/// these numbers are usually not long. On x84_64, FMUL has a latency of 4 clock
/// cycles, which doesn't include addition. Some platorms support mul + add in a
/// single instruction, but many others do not.
#[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_binary(s: &str) -> f64 {
    // b'0' is 0x30 and b'1' is 0x31.
    // So we can convert from binary digit to its value with `c & 1`.
    // This is produces more compact assembly than `c - b'0'`.
    // https://godbolt.org/z/1vvrK78jf
    const fn byte_to_value(c: u8) -> u8 {
        debug_assert!(c == b'0' || c == b'1');
        c & 1
    }
    #[cfg(test)]
    {
        static_assertions::const_assert_eq!(byte_to_value(b'0'), 0);
        static_assertions::const_assert_eq!(byte_to_value(b'1'), 1);
    }

    debug_assert!(!s.is_empty());

    let mut result = 0_u64;
    for c in s.as_bytes() {
        result <<= 1;
        result |= byte_to_value(*c) as u64;
    }
    result as f64
}

#[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_octal(s: &str) -> f64 {
    // b'0' is 0x30 and b'7' is 0x37.
    // So we can convert from any octal digit to its value with `c & 7`.
    // This is produces more compact assembly than `c - b'0'`.
    // https://godbolt.org/z/9rYTsMoMM
    const fn byte_to_value(c: u8) -> u8 {
        debug_assert!(c >= b'0' && c <= b'7');
        c & 7
    }
    #[cfg(test)]
    {
        static_assertions::const_assert_eq!(byte_to_value(b'0'), 0);
        static_assertions::const_assert_eq!(byte_to_value(b'7'), 7);
    }

    debug_assert!(!s.is_empty());

    let mut result = 0_u64;
    for c in s.as_bytes() {
        result <<= 3;
        result |= byte_to_value(*c) as u64;
    }
    result as f64
}

#[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_hex(s: &str) -> f64 {
    // b'0' is 0x30 and b'9' is 0x39.
    // b'A' is 0x41 and b'F' is 0x46.
    // b'a' is 0x61 and b'f' is 0x66.
    // So we can convert from a digit to its value with `c & 15`,
    // and from `A-F` or `a-f` to its value with `(c & 15) + 9`.
    // We could use `(c & 7) + 9` for `A-F`, but we use `(c & 15) + 9`
    // so that both branches share the same `c & 15` operation.
    // This is produces more slightly more assembly than explicitly matching all possibilities,
    // but only because compiler unrolls the loop.
    // https://godbolt.org/z/5fsdv8rGo
    const fn byte_to_value(c: u8) -> u8 {
        debug_assert!(
            (c >= b'0' && c <= b'9') || (c >= b'A' && c <= b'F') || (c >= b'a' && c <= b'f')
        );
        if c < b'A' {
            c & 15 // 0-9
        } else {
            (c & 15) + 9 // A-F or a-f
        }
    }
    #[cfg(test)]
    {
        static_assertions::const_assert_eq!(byte_to_value(b'0'), 0);
        static_assertions::const_assert_eq!(byte_to_value(b'9'), 9);
        static_assertions::const_assert_eq!(byte_to_value(b'A'), 10);
        static_assertions::const_assert_eq!(byte_to_value(b'F'), 15);
        static_assertions::const_assert_eq!(byte_to_value(b'a'), 10);
        static_assertions::const_assert_eq!(byte_to_value(b'f'), 15);
    }

    debug_assert!(!s.is_empty());

    let mut result = 0_u64;
    for c in s.as_bytes() {
        result <<= 4;
        result |= byte_to_value(*c) as u64;
    }
    result as f64
}

pub fn parse_big_int(s: &str, kind: Kind, has_sep: bool) -> Result<BigInt, &'static str> {
    let s = if has_sep { Cow::Owned(s.replace('_', "")) } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));
    parse_big_int_without_underscores(&s, kind)
}

/// This function assumes `s` has had all numeric separators (`_`) removed.
/// Parsing will fail if this assumption is violated.
fn parse_big_int_without_underscores(s: &str, kind: Kind) -> Result<BigInt, &'static str> {
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
    // NOTE: BigInt::from_bytes does a utf8 check, then uses from_str_radix
    // under the hood. We already have a string, so we can just use that
    // directly.
    BigInt::from_str_radix(s, radix).map_err(|_| "invalid bigint")
}

#[cfg(test)]
#[allow(clippy::unreadable_literal, clippy::mixed_case_hex_literals)]
mod test {

    use super::{parse_float, parse_int, Kind};

    #[allow(clippy::cast_precision_loss)]
    fn assert_all_ints_eq<I>(test_cases: I, kind: Kind, has_sep: bool)
    where
        I: IntoIterator<Item = (&'static str, i64)>,
    {
        for (s, expected) in test_cases {
            let parsed = parse_int(s, kind, has_sep);
            assert_eq!(
                parsed,
                Ok(expected as f64),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }
    fn assert_all_floats_eq<I>(test_cases: I, has_sep: bool)
    where
        I: IntoIterator<Item = (&'static str, f64)>,
    {
        for (s, expected) in test_cases {
            let parsed = parse_float(s, has_sep);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn test_int_precision() {
        assert_eq!(parse_int("9007199254740991", Kind::Decimal, false), Ok(9007199254740991.0));
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn test_float_precision() {
        let cases = vec![
            ("1.7976931348623157e+308", 1.7976931348623157e+308),
            ("0.000000001", 0.000_000_001),
        ];
        assert_all_floats_eq(cases, false);
    }

    #[test]
    fn test_parse_int_no_sep() {
        let decimal: Vec<(&str, i64)> = vec![
            // normal
            ("0", 0),
            ("-0", 0),
            ("1", 1),
            ("-1", -1),
            ("000000000000", 0),
            ("-000000000000", 0),
            ("9007199254740991", 9007199254740991), // max safe integer, 2^53 - 1
            ("-9007199254740990", -9007199254740990), // min safe integer, -(2^53 - 1)
        ];
        let binary = vec![
            ("0b0", 0b0),
            ("0b1", 0b1),
            ("0b10", 0b10),
            ("0b110001001000100", 0b110001001000100),
            ("0b110001001000100", 0b110001001000100),
        ];
        let octal = vec![("0o0", 0o0), ("0o1", 0o1), ("0o10", 0o10), ("0o777", 0o777)];
        let hex: Vec<(&str, i64)> = vec![
            ("0x0", 0x0),
            ("0X0", 0x0),
            ("0xFF", 0xFF),
            ("0xc", 0xc), // :)
            ("0xdeadbeef", 0xdeadbeef),
            ("0xFfEeDdCcBbAa", 0xFfEeDdCcBbAa),
        ];

        assert_all_ints_eq(decimal, Kind::Decimal, false);
        assert_all_ints_eq(binary, Kind::Binary, false);
        assert_all_ints_eq(octal, Kind::Octal, false);
        assert_all_ints_eq(hex, Kind::Hex, false);
    }

    #[test]
    fn test_parse_int_with_sep() {
        let decimal: Vec<(&str, i64)> = vec![
            // still works without separators
            ("0", 0),
            ("-0", 0),
            ("1", 1),
            ("-1", -1),
            ("1_000_000", 1_000_000),
            ("-1_000_000", -1_000_000),
            ("000000000000", 0),
            ("-000000000000", 0),
            ("9_007_199_254_740_991", 9_007_199_254_740_991), // max safe integer, 2^53 - 1
            ("-9_007_199_254_740_990", -9_007_199_254_740_990), // min safe integer, -(2^53 - 1)
            // still works for illegal tokens
            ("1___000_000", 1_000_000),
            ("1_", 1),
            ("_1", 1),
        ];

        let binary = vec![
            ("0b0", 0b0),
            ("0b1", 0b1),
            ("0b10", 0b10),
            ("0b110001001000100", 0b110001001000100),
            ("0b110001001000100", 0b110001001000100),
            ("0b1_1000_1001_0001_0000", 0b1_1000_1001_0001_0000),
            // still works for illegal tokens
            ("0b1_0000__0000", 0b1_0000_0000),
            ("0b1_", 0b1),
            ("0b_0", 0b0),
        ];

        let octal = vec![
            ("0o0", 0o0),
            ("0o1", 0o1),
            ("0o10", 0o10),
            ("0o777", 0o777),
            ("0o7_7_7", 0o777),
            ("0o77_73_72", 0o77_73_72),
            // still works for illegal tokens
            ("0o1_0000__0000", 0o100_000_000),
            ("0o1_", 0o1),
            ("0o_0", 0o0),
        ];

        let hex: Vec<(&str, i64)> = vec![
            // still works without separators
            ("0x0", 0x0),
            ("0X0", 0x0),
            ("0xFF", 0xFF),
            ("0xFF_AA_11", 0xFFAA11),
            ("0xdead_beef", 0xdead_beef),
            ("0xFf_Ee_Dd_Cc_Bb_Aa", 0xFfEe_DdCc_BbAa),
            ("0xFfEe_DdCc_BbAa", 0xFfEe_DdCc_BbAa),
            // still works for illegal tokens
            ("0x1_0000__0000", 0x100_000_000),
            ("0x1_", 0x1),
            ("0x_0", 0x0),
        ];

        assert_all_ints_eq(decimal, Kind::Decimal, true);
        assert_all_ints_eq(binary, Kind::Binary, true);
        assert_all_ints_eq(octal, Kind::Octal, true);
        assert_all_ints_eq(hex, Kind::Hex, true);
    }

    #[test]
    fn test_decimal() {
        let no_sep: Vec<(&'static str, f64)> =
            vec![("0", 0.0), ("1.0", 1.0), ("1.1", 1.1), ("25.125", 25.125)];

        let sep: Vec<(&'static str, f64)> = vec![
            ("1_000.0", 1000.0),
            ("1.5_000", 1.5),
            // works on invalid tokens
            ("_0._5", 0.5),
            ("0._5", 0.5),
            ("0.5_", 0.5),
        ];

        // parse_int() handles Kind::Decimal as a float. Should we check if
        // a '.' is encountered during lexing and pick which parser to use?
        assert_all_floats_eq(no_sep.clone(), false);
        assert_all_floats_eq(sep.clone(), true);
        for (s, expected) in no_sep {
            let parsed = parse_int(s, Kind::Decimal, false);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
        for (s, expected) in sep {
            let parsed = parse_int(s, Kind::Decimal, true);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }
}
