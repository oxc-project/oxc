use oxc_str::JSStr;

use crate::ToInt32;

pub trait StringLastIndexOf {
    /// `String.prototype.lastIndexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.lastindexof>
    fn last_index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize;
}

impl StringLastIndexOf for &str {
    fn last_index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize {
        JSStr::from(*self).last_index_of(search_value, from_index)
    }
}

impl StringLastIndexOf for JSStr<'_> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    fn last_index_of(&self, search_value: Option<JSStr<'_>>, from_index: Option<f64>) -> isize {
        let search = search_value.unwrap_or(JSStr::from("undefined"));
        // A value and search value without lone surrogates use the UTF-8 search.
        if let (Some(value), Some(search)) = (self.as_str(), search.as_str()) {
            return last_index_of_str(value, search, from_index);
        }
        // Steps 6-7: `ToIntegerOrInfinity(position)`, except that an absent position and NaN mean
        // positive infinity for this method.
        let position = from_index
            .map_or(f64::INFINITY, |p| if p.is_nan() { f64::INFINITY } else { p.trunc() })
            .max(0.0);
        // An empty search matches at the clamped position itself,
        // which may sit inside a surrogate pair.
        if search.is_empty() {
            return position.min(self.len_utf16() as f64) as isize;
        }
        // Steps 9-10: last occurrence starting at or before the position.
        if let Some(search) = search.as_str() {
            return crate::string_search::last_index_of_utf8(*self, search, position as usize)
                .map_or(-1, |index| index as isize);
        }
        // A surrogate needle can match the same half of a formed pair.
        crate::string_search::last_index_of_code_units(*self, search, position as usize)
            .map_or(-1, |index| index as isize)
    }
}

#[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
fn last_index_of_str(value: &str, search_value: &str, from_index: Option<f64>) -> isize {
    let from_index =
        from_index.map_or(usize::MAX, |x| x.to_int_32().max(0) as usize + search_value.len());
    value
        .chars()
        .take(from_index)
        .collect::<String>()
        .rfind(search_value)
        .map_or(-1, |index| index as isize)
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
    }

    #[test]
    fn test_string_last_index_of_with_lone_surrogates() {
        use oxc_allocator::Allocator;
        use oxc_str::JSStrBuilder;

        use super::StringLastIndexOf;

        let allocator = Allocator::new();
        let js_str = |units: &[u16]| {
            let mut builder = JSStrBuilder::new_in(&&allocator);
            builder.push_utf16(units);
            builder.into_js_str()
        };

        // "a\u{D800}b\u{D800}b": a lone surrogate is one code unit,
        // and the position bounds the start of the match.
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
        // A position inside the pair still finds a match starting at the pair.
        assert_eq!(value.last_index_of(Some(JSStr::from("\u{1F600}")), Some(2.0)), 1);
        assert_eq!(value.last_index_of(Some(JSStr::from("b")), Some(2.0)), -1);
        assert_eq!(value.last_index_of(Some(JSStr::from("\u{1F600}")), Some(0.0)), -1);

        // The bound applies to the start even when a match extends beyond it.
        let value =
            js_str(&[0xE9, 0xD800, 0x61, 0x62, 0x61, 0x62, 0x61, 0xD83D, 0xDE00, 0x61, 0x62]);
        assert_eq!(value.last_index_of(Some(JSStr::from("aba")), Some(3.0)), 2);
        assert_eq!(value.last_index_of(Some(JSStr::from("aba")), Some(5.0)), 4);
        assert_eq!(value.last_index_of(Some(JSStr::from("\u{1F600}")), Some(8.0)), 7);
        assert_eq!(value.last_index_of(Some(JSStr::from("ab")), Some(8.0)), 4);

        // A lone half in the search value matches the same half of a formed pair,
        // whether the receiver is UTF-8 or holds lone surrogates.
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
