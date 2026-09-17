use oxc_str::JSStr;

use crate::ToInt32;

pub trait StringIndexOf {
    /// `String.prototype.indexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.indexof>
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringIndexOf for &str {
    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let from_index = from_index.map_or(0, |x| x.to_int_32().max(0)) as usize;
        let search_value = search_value.unwrap_or("undefined");
        let result = self.chars().skip(from_index).collect::<String>().find(search_value);
        result.map(|index| index + from_index).map_or(-1, |index| index as isize)
    }
}

impl StringIndexOf for JSStr<'_> {
    #[expect(clippy::cast_possible_wrap, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        // A value without lone surrogates delegates to the UTF-8 search.
        if let Some(value) = self.as_str() {
            return value.index_of(search_value, from_index);
        }
        let search = search_value.unwrap_or("undefined");
        // Step 8: `ToIntegerOrInfinity(position)`.
        // An absent position and NaN both give 0.
        // The saturating float cast is exact for any position that can land inside a string.
        let start = from_index
            .map_or(0.0, |position| if position.is_nan() { 0.0 } else { position.trunc() })
            .max(0.0) as usize;
        // Step 9 clamps into the string.
        // An empty search matches at the clamped position itself.
        if search.is_empty() {
            return self.len_utf16().min(start) as isize;
        }
        crate::string_search::index_of_utf8(*self, search, start).map_or(-1, |index| index as isize)
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
        assert_eq!("test test test".index_of(Some("test"), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some("test"), Some(-1_073_741_825.0)), 0);
        assert_eq!("test test test".index_of(Some("test"), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some("notpresent"), Some(0.0)), -1);
        assert_eq!("undefined".index_of(None, Some(0.0)), 0);
        assert_eq!("test test test".index_of(None, Some(0.0)), -1);
    }

    #[test]
    fn test_string_index_of_with_lone_surrogates() {
        use oxc_allocator::Allocator;
        use oxc_str::JSStrBuilder;

        use super::StringIndexOf;

        let allocator = Allocator::new();
        let js_str = |units: &[u16]| {
            let mut builder = JSStrBuilder::new_in(&&allocator);
            builder.push_utf16(units);
            builder.into_js_str()
        };

        // "a\u{D800}b": a lone surrogate is one code unit.
        let value = js_str(&[0x61, 0xD800, 0x62]);
        assert_eq!(value.index_of(Some("a"), None), 0);
        assert_eq!(value.index_of(Some("b"), None), 2);
        assert_eq!(value.index_of(Some("b"), Some(2.0)), 2);
        assert_eq!(value.index_of(Some("b"), Some(3.0)), -1);
        assert_eq!(value.index_of(Some("c"), None), -1);
        // An empty search matches at the clamped position.
        assert_eq!(value.index_of(Some(""), None), 0);
        assert_eq!(value.index_of(Some(""), Some(2.0)), 2);
        assert_eq!(value.index_of(Some(""), Some(5.0)), 3);
        // An absent search value means the string "undefined".
        assert_eq!(value.index_of(None, None), -1);
        let value = js_str(&[0x75, 0x6E, 0x64, 0x65, 0x66, 0x69, 0x6E, 0x65, 0x64, 0xD800]);
        assert_eq!(value.index_of(None, None), 0);

        // "x\u{D83D}y\u{DE00}z": separated lone halves never match the surrogate pair of an
        // astral search value.
        let value = js_str(&[0x78, 0xD83D, 0x79, 0xDE00, 0x7A]);
        assert_eq!(value.index_of(Some("\u{1F600}"), None), -1);

        // "a\u{1F600}\u{D800}b": an astral character is two code units.
        let value = js_str(&[0x61, 0xD83D, 0xDE00, 0xD800, 0x62]);
        assert_eq!(value.index_of(Some("\u{1F600}"), None), 1);
        assert_eq!(value.index_of(Some("b"), None), 4);
        // A position inside the pair rounds up past it.
        assert_eq!(value.index_of(Some("b"), Some(2.0)), 4);
        assert_eq!(value.index_of(Some("\u{1F600}"), Some(2.0)), -1);

        // "a\u{D800}a": a start beyond `i32` does not wrap, and NaN means 0.
        let value = js_str(&[0x61, 0xD800, 0x61]);
        assert_eq!(value.index_of(Some("a"), Some(3e9)), -1);
        assert_eq!(value.index_of(Some("a"), Some(f64::NAN)), 0);

        // Adjacent halves form a supplementary character and take the UTF-8 path.
        let value = js_str(&[0xD83D, 0xDE00]);
        assert_eq!(value.as_str(), Some("\u{1F600}"));
        assert_eq!(value.index_of(Some("\u{1F600}"), None), 0);

        // Byte offsets differ from code unit indices, and matches may overlap.
        let value =
            js_str(&[0xE9, 0xD800, 0x61, 0x62, 0x61, 0x62, 0x61, 0xD83D, 0xDE00, 0x61, 0x62]);
        assert_eq!(value.index_of(Some("aba"), Some(3.0)), 4);
        assert_eq!(value.index_of(Some("b"), Some(8.0)), 10);
        assert_eq!(value.index_of(Some("\u{1F600}"), Some(8.0)), -1);
    }
}
