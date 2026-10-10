use oxc_str::JSStr;

use crate::ToInt32;

pub trait StringLastIndexOf {
    /// `String.prototype.lastIndexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.lastindexof>
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringLastIndexOf for &str {
    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let search_value = search_value.unwrap_or("undefined");
        let from_index =
            from_index.map_or(usize::MAX, |x| x.to_int_32().max(0) as usize + search_value.len());
        self.chars()
            .take(from_index)
            .collect::<String>()
            .rfind(search_value)
            .map_or(-1, |index| index as isize)
    }
}

impl StringLastIndexOf for JSStr<'_> {
    #[expect(
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        // A value without lone surrogates delegates to the UTF-8 search.
        if let Some(value) = self.as_str() {
            return value.last_index_of(search_value, from_index);
        }
        let search = search_value.unwrap_or("undefined");
        // Steps 6-7: `ToIntegerOrInfinity(position)`, except that an absent position and NaN mean
        // positive infinity for this method.
        let position = from_index
            .map_or(f64::INFINITY, |p| if p.is_nan() { f64::INFINITY } else { p.trunc() })
            .max(0.0);
        // An empty search matches at the clamped position itself.
        if search.is_empty() {
            return position.min(self.len_utf16() as f64) as isize;
        }
        crate::string_search::last_index_of_utf8(*self, search, position as usize)
            .map_or(-1, |index| index as isize)
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
        assert_eq!(value.last_index_of(Some("b"), None), 4);
        assert_eq!(value.last_index_of(Some("b"), Some(3.0)), 2);
        assert_eq!(value.last_index_of(Some("b"), Some(1.0)), -1);
        // NaN means searching from the end.
        assert_eq!(value.last_index_of(Some("b"), Some(f64::NAN)), 4);
        assert_eq!(value.last_index_of(Some("c"), None), -1);

        // "a\u{D800}b": an empty search matches at the clamped position.
        let value = js_str(&[0x61, 0xD800, 0x62]);
        assert_eq!(value.last_index_of(Some(""), None), 3);
        assert_eq!(value.last_index_of(Some(""), Some(1.0)), 1);
        // An absent search value means the string "undefined".
        assert_eq!(value.last_index_of(None, None), -1);
        let value = js_str(&[0x75, 0x6E, 0x64, 0x65, 0x66, 0x69, 0x6E, 0x65, 0x64, 0xD800]);
        assert_eq!(value.last_index_of(None, None), 0);

        // "a\u{1F600}\u{D800}b": an astral character is two code units.
        let value = js_str(&[0x61, 0xD83D, 0xDE00, 0xD800, 0x62]);
        assert_eq!(value.last_index_of(Some("\u{1F600}"), None), 1);
        assert_eq!(value.last_index_of(Some("b"), None), 4);
        // A position inside the pair still finds a match starting at the pair.
        assert_eq!(value.last_index_of(Some("\u{1F600}"), Some(2.0)), 1);
        assert_eq!(value.last_index_of(Some("b"), Some(2.0)), -1);
        assert_eq!(value.last_index_of(Some("\u{1F600}"), Some(0.0)), -1);

        // The bound applies to the start even when a match extends beyond it.
        let value =
            js_str(&[0xE9, 0xD800, 0x61, 0x62, 0x61, 0x62, 0x61, 0xD83D, 0xDE00, 0x61, 0x62]);
        assert_eq!(value.last_index_of(Some("aba"), Some(3.0)), 2);
        assert_eq!(value.last_index_of(Some("aba"), Some(5.0)), 4);
        assert_eq!(value.last_index_of(Some("\u{1F600}"), Some(8.0)), 7);
        assert_eq!(value.last_index_of(Some("ab"), Some(8.0)), 4);
    }
}
