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
        let Some(search_value) = search_value else {
            return -1;
        };
        let result = self.chars().skip(from_index).collect::<String>().find(search_value);
        result.map(|index| index + from_index).map_or(-1, |index| index as isize)
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
    }
}
