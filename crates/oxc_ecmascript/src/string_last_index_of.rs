pub trait StringLastIndexOf {
    /// `String.prototype.lastIndexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.lastindexof>
    ///
    /// The position and the result are UTF-16 code unit indices.
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringLastIndexOf for &str {
    #[expect(
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let search = search_value.unwrap_or("undefined");
        // Steps 6-7: `ToIntegerOrInfinity(position)`, except that an absent
        // position and NaN mean positive infinity for this method.
        let position = from_index
            .map_or(f64::INFINITY, |p| if p.is_nan() { f64::INFINITY } else { p.trunc() })
            .max(0.0);
        // An empty search matches at the clamped position itself, which may
        // sit inside a surrogate pair.
        if search.is_empty() {
            return position.min(self.encode_utf16().count() as f64) as isize;
        }
        // Steps 9-10: last occurrence starting at or before the position,
        // searched over bytes: a UTF-8 match can only begin on a character
        // boundary, so byte matches are exactly the UTF-16 matches. The last
        // boundary at or before the position also rounds a position inside
        // an astral character down.
        let mut units = 0.0;
        let mut offset = self.len();
        for (byte, c) in self.char_indices() {
            let next = units + c.len_utf16() as f64;
            if next > position {
                offset = byte;
                break;
            }
            units = next;
        }
        // The byte window may end inside a character; no match can end
        // there, because match ends also fall on boundaries.
        let mut window = (offset + search.len()).min(self.len());
        while !self.is_char_boundary(window) {
            window -= 1;
        }
        self[..window]
            .rfind(search)
            .map_or(-1, |position| self[..position].encode_utf16().count() as isize)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_prototype_last_index_of() {
        use super::StringLastIndexOf;
        assert_eq!("test test test".last_index_of(Some("test"), Some(15.0)), 10);
        assert_eq!("test test test".last_index_of(Some("test"), Some(14.0)), 10);
        assert_eq!("test test test".last_index_of(Some("test"), Some(10.0)), 10);
        assert_eq!("test test test".last_index_of(Some("test"), Some(9.0)), 5);
        assert_eq!("test test test".last_index_of(Some("test"), Some(6.0)), 5);
        assert_eq!("test test test".last_index_of(Some("test"), Some(5.0)), 5);
        assert_eq!("test test test".last_index_of(Some("test"), Some(4.0)), 0);
        assert_eq!("test test test".last_index_of(Some("test"), Some(0.0)), 0);
        assert_eq!("test test test".last_index_of(Some("notpresent"), Some(0.0)), -1);
        assert_eq!("undefined".last_index_of(None, None), 0);
        assert_eq!("test test test".last_index_of(None, None), -1);
        assert_eq!("abcdef".last_index_of(Some("b"), None), 1);
        // An empty search matches at the clamped position; a search longer
        // than the string never matches.
        assert_eq!("abc".last_index_of(Some(""), None), 3);
        assert_eq!("abc".last_index_of(Some(""), Some(1.0)), 1);
        assert_eq!("ab".last_index_of(Some("abc"), Some(9.0)), -1);
        // A position inside an astral character rounds down for a real
        // search, the byte window rounds down to a boundary, and an empty
        // search matches at the position itself.
        assert_eq!("a\u{1F600}b".last_index_of(Some("a"), Some(2.0)), 0);
        assert_eq!("a\u{1F600}b".last_index_of(Some("\u{1F600}"), Some(2.0)), 1);
        assert_eq!("\u{1F600}x".last_index_of(Some("x"), Some(1.0)), -1);
        assert_eq!("\u{1F600}".last_index_of(Some(""), Some(1.0)), 1);

        // Positions count UTF-16 code units, so an astral character is two.
        assert_eq!("a\u{1F600}b".last_index_of(Some("b"), None), 3);
        assert_eq!("a\u{1F600}b".last_index_of(Some("\u{1F600}"), None), 1);
        assert_eq!("\u{1F600}\u{1F600}".last_index_of(Some("\u{1F600}"), None), 2);
        assert_eq!("\u{1F600}\u{1F600}".last_index_of(Some("\u{1F600}"), Some(1.0)), 0);

        // NaN means searching from the end, and the empty search matches there.
        assert_eq!("test test test".last_index_of(Some("test"), Some(f64::NAN)), 10);
        assert_eq!("abcd".last_index_of(Some(""), None), 4);
        assert_eq!("abcd".last_index_of(Some(""), Some(2.0)), 2);
    }
}
