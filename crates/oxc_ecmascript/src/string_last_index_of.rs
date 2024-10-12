use crate::ToInt32;

pub trait StringLastIndexOf {
    /// `String.prototype.lastIndexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.lastindexof>
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringLastIndexOf for &str {
    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let Some(search_value) = search_value else { return -1 };
        let from_index =
            from_index.map_or(usize::MAX, |x| x.to_int_32().max(0) as usize + search_value.len());
        self.chars()
            .take(from_index)
            .collect::<String>()
            .rfind(search_value)
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
        assert_eq!("test test test".last_index_of(None, Some(1.0)), -1);
        assert_eq!("abcdef".last_index_of(Some("b"), None), 1);
    }
}
