pub trait StringIndexOf {
    /// `String.prototype.indexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.indexof>
    ///
    /// The position and the result are UTF-16 code unit indices.
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringIndexOf for &str {
    #[expect(clippy::cast_possible_wrap, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let search = search_value.unwrap_or("undefined");
        // Step 8: `ToIntegerOrInfinity(position)`; an absent position and NaN
        // both give 0. The saturating float cast is exact for any position
        // that can land inside a string.
        let start = from_index
            .map_or(0.0, |position| if position.is_nan() { 0.0 } else { position.trunc() })
            .max(0.0) as usize;
        // Step 9 clamps into the string. An empty search matches at the
        // clamped position itself, which may sit inside a surrogate pair.
        if search.is_empty() {
            return self.encode_utf16().count().min(start) as isize;
        }
        // Step 10: first occurrence at or after `start`, searched over bytes:
        // a UTF-8 match can only begin on a character boundary, so byte
        // matches are exactly the UTF-16 matches. The first boundary at or
        // past `start` also rounds a position inside an astral character up.
        let mut units = 0;
        let from = self
            .char_indices()
            .find_map(|(offset, c)| {
                if units >= start {
                    return Some(offset);
                }
                units += c.len_utf16();
                None
            })
            .unwrap_or(self.len());
        self[from..]
            .find(search)
            .map_or(-1, |position| self[..from + position].encode_utf16().count() as isize)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_string_index_of() {
        use super::StringIndexOf;

        assert_eq!("test test test".index_of(Some("t"), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(1.0)), 3);
        assert_eq!("test test test".index_of(Some("t"), Some(4.0)), 5);
        assert_eq!("test test test".index_of(Some("t"), Some(4.1)), 5);
        assert_eq!("test test test".index_of(Some("t"), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(-1.1)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(-1_073_741_825.0)), 0);
        assert_eq!("test test test".index_of(Some("e"), Some(0.0)), 1);
        assert_eq!("test test test".index_of(Some("s"), Some(0.0)), 2);
        assert_eq!("test test test".index_of(Some("test"), Some(4.0)), 5);
        assert_eq!("test test test".index_of(Some("test"), Some(5.0)), 5);
        assert_eq!("test test test".index_of(Some("test"), Some(6.0)), 10);
        assert_eq!("test test test".index_of(Some("test"), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some("test"), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some("not found"), Some(-1.0)), -1);
        assert_eq!("test test test".index_of(Some("test"), Some(-1_073_741_825.0)), 0);
        assert_eq!("test test test".index_of(Some("notpresent"), Some(0.0)), -1);
        assert_eq!("undefined".index_of(None, Some(0.0)), 0);
        assert_eq!("test test test".index_of(None, Some(0.0)), -1);

        // An empty search matches at the clamped position; a search longer
        // than the string never matches.
        assert_eq!("abc".index_of(Some(""), Some(99.0)), 3);
        assert_eq!("".index_of(Some(""), None), 0);
        assert_eq!("ab".index_of(Some("abc"), None), -1);

        // A position inside an astral character rounds up for a real search,
        // but an empty search matches at the position itself.
        assert_eq!("a\u{1F600}b".index_of(Some("b"), Some(2.0)), 3);
        assert_eq!("\u{1F600}".index_of(Some(""), Some(1.0)), 1);
        assert_eq!("\u{1F600}".index_of(Some("\u{1F600}"), Some(1.0)), -1);

        // Positions count UTF-16 code units, so an astral character is two.
        assert_eq!("a\u{1F600}b".index_of(Some("b"), None), 3);
        assert_eq!("a\u{1F600}b".index_of(Some("\u{1F600}"), None), 1);
        assert_eq!("\u{1F600}\u{1F600}".index_of(Some("\u{1F600}"), Some(1.0)), 2);
        assert_eq!("a\u{1F600}b".index_of(Some("b"), Some(3.0)), 3);
        assert_eq!("a\u{1F600}b".index_of(Some("b"), Some(4.0)), -1);

        // The start clamps to the length before an empty search matches.
        assert_eq!("abc".index_of(Some(""), Some(10.0)), 3);
        // A start beyond `i32` does not wrap to a small position.
        assert_eq!("aa".index_of(Some("a"), Some(3e9)), -1);
        assert_eq!("aa".index_of(Some("a"), Some(f64::NAN)), 0);
    }
}
