// Based on https://github.com/kornelski/rust_urlencoding/blob/a617c89d16f390e3ab4281ea68c514660b111301/src/enc.rs
// MIT license: https://github.com/kornelski/rust_urlencoding/blob/a617c89d16f390e3ab4281ea68c514660b111301/LICENSE

use std::borrow::Cow;

/// Implements <https://tc39.es/ecma262/2025/multipage/global-object.html#sec-encode>
/// # Safety
/// `should_encode` should only return false for characters that are ascii
#[must_use]
pub unsafe fn encode(data_str: Cow<'_, str>, should_encode: impl Fn(u8) -> bool) -> Cow<'_, str> {
    let data = data_str.as_bytes();
    // add maybe extra capacity, but try not to exceed allocator's bucket size
    let mut escaped = String::new();
    let _ = escaped.try_reserve(data.len() | 15);
    let unmodified = encode_into(data, should_encode, |s| {
        escaped.push_str(s);
    });
    if unmodified {
        return data_str;
    }
    Cow::Owned(escaped)
}

fn encode_into(
    mut data: &[u8],
    should_encode: impl Fn(u8) -> bool,
    mut push_str: impl FnMut(&str),
) -> bool {
    let mut pushed = false;
    loop {
        // Fast path to skip over safe chars at the beginning of the remaining string
        let ascii_len = data.iter().take_while(|&&c| !should_encode(c)).count();

        let (safe, rest) = if ascii_len >= data.len() {
            if !pushed {
                return true;
            }
            (data, &[][..]) // redundant to optimize out a panic in split_at
        } else {
            data.split_at(ascii_len)
        };
        pushed = true;
        if !safe.is_empty() {
            // SAFETY: should_encode has checked it's ASCII
            push_str(unsafe { str::from_utf8_unchecked(safe) });
        }
        if rest.is_empty() {
            break;
        }

        match rest.split_first() {
            Some((byte, rest)) => {
                let enc = &[b'%', to_hex_digit(byte >> 4), to_hex_digit(byte & 15)];
                // SAFETY: `%` is a valid UTF-8 char and to_hex_digit returns a valid UTF-8 char
                push_str(unsafe { str::from_utf8_unchecked(enc) });
                data = rest;
            }
            None => break,
        }
    }
    false
}

#[inline]
fn to_hex_digit(digit: u8) -> u8 {
    match digit {
        0..=9 => b'0' + digit,
        10..=255 => b'A' - 10 + digit,
    }
}

/// `alwaysUnescaped` in `Encode`
/// <https://tc39.es/ecma262/2025/multipage/global-object.html#sec-encode>
const URI_ALWAYS_UNESCAPED: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-.!~*'()";

pub fn is_uri_always_unescaped(c: u8) -> bool {
    URI_ALWAYS_UNESCAPED.contains(&c)
}
