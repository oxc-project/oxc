use crate::ToInt32;

pub trait StringSubstring {
    /// `String.prototype.substring ( start , end ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.substring>
    fn substring(&self, start: Option<f64>, end: Option<f64>) -> String;
}

impl StringSubstring for &str {
    #[expect(clippy::cast_sign_loss)]
    fn substring(&self, start: Option<f64>, end: Option<f64>) -> String {
        let start = start.map_or(0, |x| x.to_int_32().max(0) as usize);
        let end = end.map_or(usize::MAX, |x| x.to_int_32().max(0) as usize);
        let start = start.min(self.len());
        let end = end.min(self.len());
        if start > end {
            return String::new();
        }
        self.chars().skip(start).take(end - start).collect()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_prototype_last_index_of() {
        use super::StringSubstring;
        assert_eq!("foo".substring(Some(1.0), None), "oo");
        assert_eq!("foo".substring(Some(1.0), Some(2.0)), "o");
        assert_eq!("foo".substring(Some(1.0), Some(1.0)), "");
        assert_eq!("foo".substring(Some(1.0), Some(0.0)), "");
        assert_eq!("foo".substring(Some(0.0), Some(0.0)), "");
        assert_eq!("foo".substring(Some(0.0), Some(1.0)), "f");
        assert_eq!("abc".substring(Some(0.0), Some(2.0)), "ab");
        assert_eq!("abcde".substring(Some(0.0), Some(2.0)), "ab");
    }
}
