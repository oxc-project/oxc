use std::borrow::Cow;

/// <https://github.com/tafia/quick-xml/blob/6e34a730853fe295d68dc28460153f08a5a12955/src/escapei.rs#L84-L86>
pub fn xml_escape(raw: &str) -> Cow<str> {
    xml_escape_impl(raw, |ch| matches!(ch, b'<' | b'>' | b'&' | b'\'' | b'\"'))
}

fn xml_escape_impl<F: Fn(u8) -> bool>(raw: &str, escape_chars: F) -> Cow<str> {
    let bytes = raw.as_bytes();
    let mut escaped = None;
    let mut iter = bytes.iter();
    let mut pos = 0;
    while let Some(i) = iter.position(|&b| escape_chars(b)) {
        if escaped.is_none() {
            escaped = Some(Vec::with_capacity(raw.len()));
        }
        let escaped = escaped.as_mut().expect("initialized");
        let new_pos = pos + i;
        escaped.extend_from_slice(&bytes[pos..new_pos]);
        match bytes[new_pos] {
            b'<' => escaped.extend_from_slice(b"&lt;"),
            b'>' => escaped.extend_from_slice(b"&gt;"),
            b'\'' => escaped.extend_from_slice(b"&apos;"),
            b'&' => escaped.extend_from_slice(b"&amp;"),
            b'"' => escaped.extend_from_slice(b"&quot;"),

            // This set of escapes handles characters that should be escaped
            // in elements of xs:lists, because those characters works as
            // delimiters of list elements
            b'\t' => escaped.extend_from_slice(b"&#9;"),
            b'\n' => escaped.extend_from_slice(b"&#10;"),
            b'\r' => escaped.extend_from_slice(b"&#13;"),
            b' ' => escaped.extend_from_slice(b"&#32;"),
            _ => unreachable!(
                "Only '<', '>','\', '&', '\"', '\\t', '\\r', '\\n', and ' ' are escaped"
            ),
        }
        pos = new_pos + 1;
    }

    if let Some(mut escaped) = escaped {
        if let Some(raw) = bytes.get(pos..) {
            escaped.extend_from_slice(raw);
        }

        // SAFETY: we operate on UTF-8 input and search for an one byte chars only,
        // so all slices that was put to the `escaped` is a valid UTF-8 encoded strings
        Cow::Owned(unsafe { String::from_utf8_unchecked(escaped) })
    } else {
        Cow::Borrowed(raw)
    }
}
