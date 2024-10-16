use crate::ToInt32;

pub trait StringCharAt {
    /// `String.prototype.charAt ( pos )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.charat>
    fn char_at(&self, index: Option<f64>) -> Option<char>;
}

impl StringCharAt for &str {
    #[expect(clippy::cast_sign_loss)]
    fn char_at(&self, index: Option<f64>) -> Option<char> {
        let index = index.map_or(0, |x| x.to_int_32() as isize);
        if index < 0 {
            None
        } else {
            self.chars().nth(index as usize)
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_evaluate_string_char_at() {
        use crate::string_char_at::StringCharAt;
        assert_eq!("test".char_at(Some(0.0)), Some('t'));
        assert_eq!("test".char_at(Some(1.0)), Some('e'));
        assert_eq!("test".char_at(Some(2.0)), Some('s'));
        assert_eq!("test".char_at(Some(-1.0)), None);
        assert_eq!("test".char_at(Some(-1.1)), None);
        assert_eq!("test".char_at(Some(-1_073_741_825.0)), None);
    }
}
