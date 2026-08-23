use crate::to_integer_or_infinity::{ToIntegerOrInfinity, ToIntegerOrInfinityResult};

pub trait StringIndexOf {
    /// `String.prototype.indexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.indexof>
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringIndexOf for &str {
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let string = self.encode_utf16().collect::<Vec<_>>();
        let search_value = search_value.unwrap_or("undefined").encode_utf16().collect::<Vec<_>>();
        let from_index = match from_index.unwrap_or(0.0).to_integer_or_infinity_as_i64() {
            ToIntegerOrInfinityResult::Infinity => string.len(),
            ToIntegerOrInfinityResult::NegativeInfinity => 0,
            ToIntegerOrInfinityResult::Value(value) if value <= 0 => 0,
            ToIntegerOrInfinityResult::Value(value) => {
                usize::try_from(value).unwrap_or(string.len()).min(string.len())
            }
        };
        if search_value.is_empty() {
            return isize::try_from(from_index).unwrap_or(-1);
        }

        string[from_index..]
            .windows(search_value.len())
            .position(|window| window == search_value)
            .and_then(|index| isize::try_from(index + from_index).ok())
            .unwrap_or(-1)
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
        assert_eq!("test test test".index_of(None, Some(0.0)), -1);
        assert_eq!("undefined".index_of(None, Some(0.0)), 0);
        assert_eq!("éa".index_of(Some("a"), None), 1);
        assert_eq!("中a".index_of(Some("a"), None), 1);
        assert_eq!("😀a".index_of(Some("a"), None), 2);
    }
}
