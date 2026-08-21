use crate::to_integer_or_infinity::{ToIntegerOrInfinity, ToIntegerOrInfinityResult};

pub trait StringLastIndexOf {
    /// `String.prototype.lastIndexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.lastindexof>
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringLastIndexOf for &str {
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let string = self.encode_utf16().collect::<Vec<_>>();
        let search_value = search_value.unwrap_or("undefined").encode_utf16().collect::<Vec<_>>();
        let from_index = match from_index {
            None => string.len(),
            Some(value) if value.is_nan() => string.len(),
            Some(value) => match value.to_integer_or_infinity_as_i64() {
                ToIntegerOrInfinityResult::Infinity => string.len(),
                ToIntegerOrInfinityResult::NegativeInfinity => 0,
                ToIntegerOrInfinityResult::Value(value) if value <= 0 => 0,
                ToIntegerOrInfinityResult::Value(value) => {
                    usize::try_from(value).unwrap_or(string.len()).min(string.len())
                }
            },
        };
        if search_value.is_empty() {
            return isize::try_from(from_index).unwrap_or(-1);
        }
        if search_value.len() > string.len() {
            return -1;
        }

        let from_index = from_index.min(string.len() - search_value.len());
        string[..from_index + search_value.len()]
            .windows(search_value.len())
            .rposition(|window| window == search_value)
            .and_then(|index| isize::try_from(index).ok())
            .unwrap_or(-1)
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
        assert_eq!("test test test".last_index_of(None, Some(1.0)), -1);
        assert_eq!("abcdef".last_index_of(Some("b"), None), 1);
        assert_eq!("undefined".last_index_of(None, None), 0);
        assert_eq!("a😀a".last_index_of(Some("a"), None), 3);
        assert_eq!("aba".last_index_of(Some("b"), Some(f64::NAN)), 1);
    }
}
