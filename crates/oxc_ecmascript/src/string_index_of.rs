use oxc_str::JSStr;

pub trait StringIndexOf {
    /// `String.prototype.indexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.indexof>
    ///
    /// The position and the result are UTF-16 code unit indices.
    fn index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize;
}

impl StringIndexOf for &str {
    #[expect(clippy::cast_possible_wrap, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize {
        let search = search_value.unwrap_or(JSStr::from("undefined"));
        // A search value holding a lone surrogate half can still match the
        // matching half of a formed pair, which only the code unit search
        // sees.
        let Some(search) = search.as_str() else {
            return index_of_code_units(JSStr::from(*self), search, from_index);
        };
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

impl StringIndexOf for JSStr<'_> {
    fn index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize {
        // A value without lone surrogates delegates to the UTF-8 search,
        // which routes a surrogate search value back to the code unit path.
        if let Some(value) = self.as_str() {
            return value.index_of(search_value, from_index);
        }
        index_of_code_units(*self, search_value.unwrap_or(JSStr::from("undefined")), from_index)
    }
}

#[expect(clippy::cast_possible_wrap, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn index_of_code_units(value: JSStr<'_>, search: JSStr<'_>, from_index: Option<f64>) -> isize {
    // Step 8: `ToIntegerOrInfinity(position)`; an absent position and NaN
    // both give 0. The saturating float cast is exact for any position
    // that can land inside a string.
    let start = from_index
        .map_or(0.0, |position| if position.is_nan() { 0.0 } else { position.trunc() })
        .max(0.0) as usize;
    // Step 9 clamps into the string. An empty search matches at the
    // clamped position itself.
    if search.is_empty() {
        return value.len_utf16().min(start) as isize;
    }
    // Step 10: first occurrence at or after `start`, compared over UTF-16
    // code units, which is the domain the specification defines. A lone
    // surrogate is a single unit, so a surrogate half in the search value
    // matches both a lone half and the same half of a formed pair.
    let mut remaining = value.encode_utf16();
    let mut position = 0;
    loop {
        if position >= start {
            let mut candidate = remaining.clone();
            if search.encode_utf16().all(|unit| candidate.next() == Some(unit)) {
                return position as isize;
            }
        }
        if remaining.next().is_none() {
            return -1;
        }
        position += 1;
    }
}

#[cfg(test)]
mod test {
    use oxc_str::JSStr;

    #[test]
    fn test_string_index_of() {
        use super::StringIndexOf;

        assert_eq!("test test test".index_of(Some(JSStr::from("t")), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some(JSStr::from("t")), Some(1.0)), 3);
        assert_eq!("test test test".index_of(Some(JSStr::from("t")), Some(4.0)), 5);
        assert_eq!("test test test".index_of(Some(JSStr::from("t")), Some(4.1)), 5);
        assert_eq!("test test test".index_of(Some(JSStr::from("t")), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some(JSStr::from("t")), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some(JSStr::from("t")), Some(-1.1)), 0);
        assert_eq!("test test test".index_of(Some(JSStr::from("t")), Some(-1_073_741_825.0)), 0);
        assert_eq!("test test test".index_of(Some(JSStr::from("e")), Some(0.0)), 1);
        assert_eq!("test test test".index_of(Some(JSStr::from("s")), Some(0.0)), 2);
        assert_eq!("test test test".index_of(Some(JSStr::from("test")), Some(4.0)), 5);
        assert_eq!("test test test".index_of(Some(JSStr::from("test")), Some(5.0)), 5);
        assert_eq!("test test test".index_of(Some(JSStr::from("test")), Some(6.0)), 10);
        assert_eq!("test test test".index_of(Some(JSStr::from("test")), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some(JSStr::from("test")), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some(JSStr::from("not found")), Some(-1.0)), -1);
        assert_eq!("test test test".index_of(Some(JSStr::from("test")), Some(-1_073_741_825.0)), 0);
        assert_eq!("test test test".index_of(Some(JSStr::from("notpresent")), Some(0.0)), -1);
        assert_eq!("undefined".index_of(None, Some(0.0)), 0);
        assert_eq!("test test test".index_of(None, Some(0.0)), -1);

        // An empty search matches at the clamped position; a search longer
        // than the string never matches.
        assert_eq!("abc".index_of(Some(JSStr::from("")), Some(99.0)), 3);
        assert_eq!("".index_of(Some(JSStr::from("")), None), 0);
        assert_eq!("ab".index_of(Some(JSStr::from("abc")), None), -1);

        // A position inside an astral character rounds up for a real search,
        // but an empty search matches at the position itself.
        assert_eq!("a\u{1F600}b".index_of(Some(JSStr::from("b")), Some(2.0)), 3);
        assert_eq!("\u{1F600}".index_of(Some(JSStr::from("")), Some(1.0)), 1);
        assert_eq!("\u{1F600}".index_of(Some(JSStr::from("\u{1F600}")), Some(1.0)), -1);

        // Positions count UTF-16 code units, so an astral character is two.
        assert_eq!("a\u{1F600}b".index_of(Some(JSStr::from("b")), None), 3);
        assert_eq!("a\u{1F600}b".index_of(Some(JSStr::from("\u{1F600}")), None), 1);
        assert_eq!("\u{1F600}\u{1F600}".index_of(Some(JSStr::from("\u{1F600}")), Some(1.0)), 2);
        assert_eq!("a\u{1F600}b".index_of(Some(JSStr::from("b")), Some(3.0)), 3);
        assert_eq!("a\u{1F600}b".index_of(Some(JSStr::from("b")), Some(4.0)), -1);

        // The start clamps to the length before an empty search matches.
        assert_eq!("abc".index_of(Some(JSStr::from("")), Some(10.0)), 3);
        // A start beyond `i32` does not wrap to a small position.
        assert_eq!("aa".index_of(Some(JSStr::from("a")), Some(3e9)), -1);
        assert_eq!("aa".index_of(Some(JSStr::from("a")), Some(f64::NAN)), 0);
    }

    #[test]
    fn test_string_index_of_with_lone_surrogates() {
        use oxc_allocator::Allocator;
        use oxc_str::JSStrBuilder;

        use super::StringIndexOf;

        let allocator = Allocator::new();
        let js_str = |units: &[u16]| {
            let mut builder = JSStrBuilder::new_in(&allocator);
            builder.push_utf16(units);
            builder.into_js_str()
        };

        // "a\u{D800}b": a lone surrogate is one code unit.
        let value = js_str(&[0x61, 0xD800, 0x62]);
        assert_eq!(value.index_of(Some(JSStr::from("a")), None), 0);
        assert_eq!(value.index_of(Some(JSStr::from("b")), None), 2);
        assert_eq!(value.index_of(Some(JSStr::from("b")), Some(2.0)), 2);
        assert_eq!(value.index_of(Some(JSStr::from("b")), Some(3.0)), -1);
        assert_eq!(value.index_of(Some(JSStr::from("c")), None), -1);
        // An empty search matches at the clamped position.
        assert_eq!(value.index_of(Some(JSStr::from("")), None), 0);
        assert_eq!(value.index_of(Some(JSStr::from("")), Some(2.0)), 2);
        assert_eq!(value.index_of(Some(JSStr::from("")), Some(5.0)), 3);
        // An absent search value means the string "undefined".
        assert_eq!(value.index_of(None, None), -1);
        let value = js_str(&[0x75, 0x6E, 0x64, 0x65, 0x66, 0x69, 0x6E, 0x65, 0x64, 0xD800]);
        assert_eq!(value.index_of(None, None), 0);

        // "x\u{D83D}y\u{DE00}z": separated lone halves never match the
        // surrogate pair of an astral search value.
        let value = js_str(&[0x78, 0xD83D, 0x79, 0xDE00, 0x7A]);
        assert_eq!(value.index_of(Some(JSStr::from("\u{1F600}")), None), -1);

        // "a\u{1F600}\u{D800}b": an astral character is two code units.
        let value = js_str(&[0x61, 0xD83D, 0xDE00, 0xD800, 0x62]);
        assert_eq!(value.index_of(Some(JSStr::from("\u{1F600}")), None), 1);
        assert_eq!(value.index_of(Some(JSStr::from("b")), None), 4);

        // "a\u{D800}a": a start beyond `i32` does not wrap, and NaN means 0.
        let value = js_str(&[0x61, 0xD800, 0x61]);
        assert_eq!(value.index_of(Some(JSStr::from("a")), Some(3e9)), -1);
        assert_eq!(value.index_of(Some(JSStr::from("a")), Some(f64::NAN)), 0);

        // Adjacent halves form a supplementary character and take the UTF-8 path.
        let value = js_str(&[0xD83D, 0xDE00]);
        assert_eq!(value.as_str(), Some("\u{1F600}"));
        assert_eq!(value.index_of(Some(JSStr::from("\u{1F600}")), None), 0);

        // A lone half in the search value matches the same half of a formed
        // pair, whether the receiver is UTF-8 or holds lone surrogates.
        let lead = js_str(&[0xD83D]);
        let trail = js_str(&[0xDE00]);
        assert_eq!("a\u{1F600}b".index_of(Some(lead), None), 1);
        assert_eq!("a\u{1F600}b".index_of(Some(trail), None), 2);
        assert_eq!("a\u{1F600}b".index_of(Some(lead), Some(2.0)), -1);
        let value = js_str(&[0x61, 0xD800, 0xD83D, 0xDE00]);
        assert_eq!(value.index_of(Some(lead), None), 2);
        assert_eq!(value.index_of(Some(js_str(&[0xD800])), None), 1);
        // A search value of two lone halves matches the formed pair.
        assert_eq!(value.index_of(Some(js_str(&[0xD83D, 0xDE00])), None), 2);
    }
}
