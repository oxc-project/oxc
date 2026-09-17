use oxc_str::JSStr;

use crate::{
    StringCharAt, string_char_at::StringCharAtResult,
    to_integer_or_infinity::ToIntegerOrInfinityResult,
};

pub trait StringCharCodeAt {
    /// `String.prototype.charCodeAt ( pos )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.charcodeat>
    ///
    /// The position is a UTF-16 code unit index; the result is the code unit
    /// at that position, which may be a lone or paired surrogate.
    fn char_code_at(&self, index: Option<f64>) -> Option<u32>;
}

impl StringCharCodeAt for &str {
    fn char_code_at(&self, index: Option<f64>) -> Option<u32> {
        match self.char_at(index) {
            StringCharAtResult::Value(c) => Some(c as u32),
            StringCharAtResult::InvalidChar(v) => Some(u32::from(v)),
            StringCharAtResult::OutOfRange => None,
        }
    }
}

impl StringCharCodeAt for JSStr<'_> {
    fn char_code_at(&self, index: Option<f64>) -> Option<u32> {
        use crate::to_integer_or_infinity::ToIntegerOrInfinity;

        // A value without lone surrogates delegates to the UTF-8 lookup.
        if let Some(value) = self.as_str() {
            return value.char_code_at(index);
        }
        let position = match index.unwrap_or(0.0).to_integer_or_infinity_as_i64() {
            ToIntegerOrInfinityResult::Value(value) if value >= 0 => usize::try_from(value).ok()?,
            _ => return None,
        };
        // A lone surrogate is itself a code unit, so every in-range position
        // has a value.
        self.encode_utf16().nth(position).map(u32::from)
    }
}

#[cfg(test)]
mod test {
    use super::StringCharCodeAt;

    #[test]
    fn test_evaluate_char_code_at() {
        let s = "abcde";
        assert_eq!(s.char_code_at(Some(0.0)), Some(97));
        assert_eq!(s.char_code_at(Some(1.0)), Some(98));
        assert_eq!(s.char_code_at(Some(2.0)), Some(99));
        assert_eq!(s.char_code_at(Some(3.0)), Some(100));
        assert_eq!(s.char_code_at(Some(4.0)), Some(101));
        assert_eq!(s.char_code_at(Some(5.0)), None);
        assert_eq!(s.char_code_at(Some(-1.0)), None);
        assert_eq!(s.char_code_at(None), Some(97));
        assert_eq!(s.char_code_at(Some(0.0)), Some(97));
        assert_eq!(s.char_code_at(Some(f64::NAN)), Some(97));
        assert_eq!(s.char_code_at(Some(f64::INFINITY)), None);

        // An astral character is two code units, each a surrogate half.
        let astral = "\u{1F600}";
        assert_eq!(astral.char_code_at(Some(0.0)), Some(0xD83D));
        assert_eq!(astral.char_code_at(Some(1.0)), Some(0xDE00));
        assert_eq!(astral.char_code_at(Some(2.0)), None);
    }

    #[test]
    fn test_evaluate_char_code_at_with_lone_surrogates() {
        use oxc_allocator::Allocator;
        use oxc_str::JSStrBuilder;

        // "a\u{D800}b": the lone surrogate is the code unit at position 1.
        let allocator = Allocator::new();
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_utf16(&[0x61, 0xD800, 0x62]);
        let value = builder.into_js_str();
        assert_eq!(value.char_code_at(Some(0.0)), Some(97));
        assert_eq!(value.char_code_at(Some(1.0)), Some(0xD800));
        assert_eq!(value.char_code_at(Some(2.0)), Some(98));
        assert_eq!(value.char_code_at(Some(3.0)), None);
        assert_eq!(value.char_code_at(Some(-1.0)), None);
        assert_eq!(value.char_code_at(None), Some(97));
        assert_eq!(value.char_code_at(Some(f64::NAN)), Some(97));
        assert_eq!(value.char_code_at(Some(f64::INFINITY)), None);
    }
}
