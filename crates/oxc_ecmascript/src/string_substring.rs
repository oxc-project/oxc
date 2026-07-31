use crate::to_integer_or_infinity::{ToIntegerOrInfinity, ToIntegerOrInfinityResult};

pub trait StringSubstring {
    /// `String.prototype.substring ( start , end ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.substring>
    fn substring(&self, start: Option<f64>, end: Option<f64>) -> Option<String>;
}

impl StringSubstring for &str {
    fn substring(&self, start: Option<f64>, end: Option<f64>) -> Option<String> {
        let string = self.encode_utf16().collect::<Vec<_>>();
        let to_index = |value: f64| match value.to_integer_or_infinity_as_i64() {
            ToIntegerOrInfinityResult::Infinity => string.len(),
            ToIntegerOrInfinityResult::NegativeInfinity => 0,
            ToIntegerOrInfinityResult::Value(value) if value <= 0 => 0,
            ToIntegerOrInfinityResult::Value(value) => {
                usize::try_from(value).unwrap_or(string.len()).min(string.len())
            }
        };
        let start = start.map_or(0, to_index);
        let end = end.map_or(string.len(), to_index);
        if start > end {
            return Some(String::new());
        }

        String::from_utf16(&string[start..end]).ok()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_prototype_last_index_of() {
        use super::StringSubstring;
        assert_eq!("foo".substring(Some(1.0), None).as_deref(), Some("oo"));
        assert_eq!("foo".substring(Some(1.0), Some(2.0)).as_deref(), Some("o"));
        assert_eq!("foo".substring(Some(1.0), Some(1.0)).as_deref(), Some(""));
        assert_eq!("foo".substring(Some(1.0), Some(0.0)).as_deref(), Some(""));
        assert_eq!("foo".substring(Some(0.0), Some(0.0)).as_deref(), Some(""));
        assert_eq!("foo".substring(Some(0.0), Some(1.0)).as_deref(), Some("f"));
        assert_eq!("abc".substring(Some(0.0), Some(2.0)).as_deref(), Some("ab"));
        assert_eq!("abcde".substring(Some(0.0), Some(2.0)).as_deref(), Some("ab"));
        assert_eq!("😀a".substring(Some(2.0), None).as_deref(), Some("a"));
        assert_eq!("😀a".substring(Some(1.0), None), None);
    }
}
