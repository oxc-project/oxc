use crate::ToInt32;

pub trait StringCharAt {
    /// `String.prototype.charAt ( pos )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.charat>
    fn char_at(&self, index: Option<f64>) -> Option<char>;
}

impl StringCharAt for &str {
    #[expect(clippy::cast_sign_loss)]
    fn char_at(&self, index: Option<f64>) -> Option<char> {
        let index = index.unwrap_or(0.0);
        if index.fract() != 0.0 || index.is_nan() || index.is_infinite() {
            return None;
        }
        let index = index.to_int_32() as isize;
        if index < 0 {
            None
        } else {
            self.encode_utf16().nth(index as usize).and_then(|n| char::from_u32(u32::from(n)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::StringCharAt;

    #[test]
    fn test_evaluate_string_char_at() {
        let s = "test";
        assert_eq!(s.char_at(Some(0.0)), Some('t'));
        assert_eq!(s.char_at(Some(1.0)), Some('e'));
        assert_eq!(s.char_at(Some(2.0)), Some('s'));
        assert_eq!(s.char_at(Some(0.5)), None);
        assert_eq!(s.char_at(Some(-1.0)), None);
        assert_eq!(s.char_at(Some(-1.1)), None);
        assert_eq!(s.char_at(Some(-1_073_741_825.0)), None);
    }
}
