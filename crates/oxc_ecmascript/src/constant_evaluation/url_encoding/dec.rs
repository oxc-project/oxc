// based on https://github.com/kornelski/rust_urlencoding/blob/a617c89d16f390e3ab4281ea68c514660b111301/src/dec.rs#L21
// MIT license: https://github.com/kornelski/rust_urlencoding/blob/a617c89d16f390e3ab4281ea68c514660b111301/LICENSE

use std::borrow::Cow;
use std::panic::panic_any;

/// Implements <https://tc39.es/ecma262/2025/multipage/global-object.html#sec-decode>
#[inline]
pub fn decode(
    data_str: Cow<'_, str>,
    should_not_decode: impl Fn(u8) -> bool,
) -> Option<Cow<'_, str>> {
    let data = data_str.as_bytes();
    let offset = data.iter().take_while(|&&c| c != b'%').count();
    if offset >= data.len() {
        return Some(data_str);
    }

    let mut decoded = Vec::new();
    if decoded.try_reserve(data.len()).is_err() {
        panic_any("OOM"); // more efficient codegen than built-in OOM handler
    }
    let mut out = NeverRealloc(&mut decoded);

    let (ascii, mut data) = data.split_at(offset);
    out.extend_from_slice(ascii);

    loop {
        let mut parts = data.splitn(2, |&c| c == b'%');
        // first the decoded non-% part
        let non_escaped_part = parts.next().unwrap();
        let rest = parts.next();
        if rest.is_none() && out.0.is_empty() {
            // if empty there were no '%' in the string
            return Some(data_str);
        }
        out.extend_from_slice(non_escaped_part);

        // then decode one %xx
        match rest {
            Some(rest) => {
                let Some(&[first, second]) = rest.get(0..2) else {
                    // 4.c.i.
                    return None;
                };
                let (Some(first_val), Some(second_val)) =
                    (from_hex_digit(first), from_hex_digit(second))
                else {
                    // 4.c.iii.
                    return None;
                };
                let char = (first_val << 4) | second_val;
                if should_not_decode(char) {
                    out.extend_from_slice(&[b'%', first, second]);
                } else {
                    out.push(char);
                }
                data = &rest[2..];
            }
            None => break,
        }
    }
    Some(Cow::Owned(String::from_utf8(decoded).ok()?))
}

#[inline]
fn from_hex_digit(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

struct NeverRealloc<'a, T>(pub &'a mut Vec<T>);

impl<T> NeverRealloc<'_, T> {
    #[inline]
    pub fn push(&mut self, val: T) {
        // these branches only exist to remove redundant reallocation code
        // (the capacity is always sufficient)
        if self.0.len() != self.0.capacity() {
            self.0.push(val);
        }
    }

    #[inline]
    pub fn extend_from_slice(&mut self, val: &[T])
    where
        T: Clone,
    {
        if self.0.capacity() - self.0.len() >= val.len() {
            self.0.extend_from_slice(val);
        }
    }
}

#[test]
fn dec_borrows() {
    assert!(matches!(decode("hello".into(), |_| false), Some(Cow::Borrowed("hello"))));
    assert!(matches!(decode("hello%20".into(), |_| false), Some(Cow::Owned(s)) if s == "hello "));
    assert!(matches!(decode("%20hello".into(), |_| false), Some(Cow::Owned(s)) if s == " hello"));
}
