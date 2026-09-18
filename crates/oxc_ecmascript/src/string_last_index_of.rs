use oxc_str::JSStr;

pub trait StringLastIndexOf {
    /// `String.prototype.lastIndexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.lastindexof>
    ///
    /// The position and the result are UTF-16 code unit indices.
    fn last_index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize;
}

impl StringLastIndexOf for &str {
    #[expect(
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn last_index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize {
        let search = search_value.unwrap_or(JSStr::from("undefined"));
        // A search value holding a lone surrogate half can still match the
        // matching half of a formed pair, which only the code unit search
        // sees.
        let Some(search) = search.as_str() else {
            return last_index_of_code_units(JSStr::from(*self), search, from_index);
        };
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

impl StringLastIndexOf for JSStr<'_> {
    fn last_index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize {
        // A value without lone surrogates delegates to the UTF-8 search,
        // which routes a surrogate search value back to the code unit path.
        if let Some(value) = self.as_str() {
            return value.last_index_of(search_value, from_index);
        }
        last_index_of_code_units(
            *self,
            search_value.unwrap_or(JSStr::from("undefined")),
            from_index,
        )
    }
}

#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn last_index_of_code_units(value: JSStr<'_>, search: JSStr<'_>, from_index: Option<f64>) -> isize {
    // Steps 6-7: `ToIntegerOrInfinity(position)`, except that an absent
    // position and NaN mean positive infinity for this method.
    let position = from_index
        .map_or(f64::INFINITY, |p| if p.is_nan() { f64::INFINITY } else { p.trunc() })
        .max(0.0);
    // An empty search matches at the clamped position itself.
    if search.is_empty() {
        return position.min(value.len_utf16() as f64) as isize;
    }
    // Steps 9-10: last occurrence starting at or before the position,
    // compared over UTF-16 code units, which is the domain the
    // specification defines. A lone surrogate is a single unit, so a
    // surrogate half in the search value matches both a lone half and the
    // same half of a formed pair.
    // The saturating float cast turns an infinite position into a bound
    // past any string length.
    let bound = position as usize;
    let mut remaining = value.encode_utf16();
    let mut result = -1;
    for index in 0..=bound {
        let mut candidate = remaining.clone();
        if search.encode_utf16().all(|unit| candidate.next() == Some(unit)) {
            result = index as isize;
        }
        if remaining.next().is_none() {
            break;
        }
    }
    result
}

#[cfg(test)]
mod test {
    use oxc_str::JSStr;

    #[test]
    fn test_prototype_last_index_of() {
        use super::StringLastIndexOf;
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(15.0)), 10);
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(14.0)), 10);
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(10.0)), 10);
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(9.0)), 5);
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(6.0)), 5);
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(5.0)), 5);
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(4.0)), 0);
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(0.0)), 0);
        assert_eq!("test test test".last_index_of(Some(JSStr::from("notpresent")), Some(0.0)), -1);
        assert_eq!("undefined".last_index_of(None, None), 0);
        assert_eq!("test test test".last_index_of(None, None), -1);
        assert_eq!("abcdef".last_index_of(Some(JSStr::from("b")), None), 1);
        // An empty search matches at the clamped position; a search longer
        // than the string never matches.
        assert_eq!("abc".last_index_of(Some(JSStr::from("")), None), 3);
        assert_eq!("abc".last_index_of(Some(JSStr::from("")), Some(1.0)), 1);
        assert_eq!("ab".last_index_of(Some(JSStr::from("abc")), Some(9.0)), -1);
        // A position inside an astral character rounds down for a real
        // search, the byte window rounds down to a boundary, and an empty
        // search matches at the position itself.
        assert_eq!("a\u{1F600}b".last_index_of(Some(JSStr::from("a")), Some(2.0)), 0);
        assert_eq!("a\u{1F600}b".last_index_of(Some(JSStr::from("\u{1F600}")), Some(2.0)), 1);
        assert_eq!("\u{1F600}x".last_index_of(Some(JSStr::from("x")), Some(1.0)), -1);
        assert_eq!("\u{1F600}".last_index_of(Some(JSStr::from("")), Some(1.0)), 1);

        // Positions count UTF-16 code units, so an astral character is two.
        assert_eq!("a\u{1F600}b".last_index_of(Some(JSStr::from("b")), None), 3);
        assert_eq!("a\u{1F600}b".last_index_of(Some(JSStr::from("\u{1F600}")), None), 1);
        assert_eq!("\u{1F600}\u{1F600}".last_index_of(Some(JSStr::from("\u{1F600}")), None), 2);
        assert_eq!(
            "\u{1F600}\u{1F600}".last_index_of(Some(JSStr::from("\u{1F600}")), Some(1.0)),
            0
        );

        // NaN means searching from the end, and the empty search matches there.
        assert_eq!("test test test".last_index_of(Some(JSStr::from("test")), Some(f64::NAN)), 10);
        assert_eq!("abcd".last_index_of(Some(JSStr::from("")), None), 4);
        assert_eq!("abcd".last_index_of(Some(JSStr::from("")), Some(2.0)), 2);
    }

    #[test]
    fn test_string_last_index_of_with_lone_surrogates() {
        use oxc_allocator::Allocator;
        use oxc_str::JSStrBuilder;

        use super::StringLastIndexOf;

        let allocator = Allocator::new();
        let js_str = |units: &[u16]| {
            let mut builder = JSStrBuilder::new_in(&allocator);
            builder.push_utf16(units);
            builder.into_js_str()
        };

        // "a\u{D800}b\u{D800}b": a lone surrogate is one code unit, and the
        // position bounds the start of the match.
        let value = js_str(&[0x61, 0xD800, 0x62, 0xD800, 0x62]);
        assert_eq!(value.last_index_of(Some(JSStr::from("b")), None), 4);
        assert_eq!(value.last_index_of(Some(JSStr::from("b")), Some(3.0)), 2);
        assert_eq!(value.last_index_of(Some(JSStr::from("b")), Some(1.0)), -1);
        // NaN means searching from the end.
        assert_eq!(value.last_index_of(Some(JSStr::from("b")), Some(f64::NAN)), 4);
        assert_eq!(value.last_index_of(Some(JSStr::from("c")), None), -1);

        // "a\u{D800}b": an empty search matches at the clamped position.
        let value = js_str(&[0x61, 0xD800, 0x62]);
        assert_eq!(value.last_index_of(Some(JSStr::from("")), None), 3);
        assert_eq!(value.last_index_of(Some(JSStr::from("")), Some(1.0)), 1);
        // An absent search value means the string "undefined".
        assert_eq!(value.last_index_of(None, None), -1);
        let value = js_str(&[0x75, 0x6E, 0x64, 0x65, 0x66, 0x69, 0x6E, 0x65, 0x64, 0xD800]);
        assert_eq!(value.last_index_of(None, None), 0);

        // "a\u{1F600}\u{D800}b": an astral character is two code units.
        let value = js_str(&[0x61, 0xD83D, 0xDE00, 0xD800, 0x62]);
        assert_eq!(value.last_index_of(Some(JSStr::from("\u{1F600}")), None), 1);
        assert_eq!(value.last_index_of(Some(JSStr::from("b")), None), 4);

        // A lone half in the search value matches the same half of a formed
        // pair, whether the receiver is UTF-8 or holds lone surrogates.
        let lead = js_str(&[0xD83D]);
        let trail = js_str(&[0xDE00]);
        assert_eq!("a\u{1F600}b".last_index_of(Some(lead), None), 1);
        assert_eq!("a\u{1F600}b".last_index_of(Some(trail), None), 2);
        assert_eq!("a\u{1F600}b".last_index_of(Some(lead), Some(0.0)), -1);
        let value = js_str(&[0x61, 0xD800, 0xD83D, 0xDE00]);
        assert_eq!(value.last_index_of(Some(lead), None), 2);
        assert_eq!(value.last_index_of(Some(js_str(&[0xD800])), None), 1);
    }
}
