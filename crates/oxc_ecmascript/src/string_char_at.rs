use num_traits::ToPrimitive;

use crate::to_integer_or_infinity::ToIntegerOrInfinityResult;

pub trait StringCharAt {
    /// `String.prototype.charAt ( pos )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.charat>
    fn char_at(&self, pos: Option<f64>) -> StringCharAtResult;
}

impl StringCharAt for &str {
    fn char_at(&self, pos: Option<f64>) -> StringCharAtResult {
        use crate::to_integer_or_infinity::ToIntegerOrInfinity;

        let position = pos.unwrap_or(0.0).to_integer_or_infinity_as_i64();
        let position = match position {
            ToIntegerOrInfinityResult::Value(v) if v >= 0 => v.to_usize().unwrap(),
            _ => return StringCharAtResult::OutOfRange,
        };

        self.encode_utf16().nth(position).map_or(StringCharAtResult::OutOfRange, |n| {
            char::from_u32(u32::from(n))
                .map_or(StringCharAtResult::InvalidChar(n), StringCharAtResult::Value)
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StringCharAtResult {
    Value(char),
    InvalidChar(u16),
    OutOfRange,
}

#[cfg(test)]
mod test {
    use crate::string_char_at::StringCharAtResult;

    use super::StringCharAt;

    #[test]
    fn test_evaluate_string_char_at() {
        let s = "test";
        assert_eq!(s.char_at(Some(0.0)), StringCharAtResult::Value('t'));
        assert_eq!(s.char_at(Some(1.0)), StringCharAtResult::Value('e'));
        assert_eq!(s.char_at(Some(2.0)), StringCharAtResult::Value('s'));
        assert_eq!(s.char_at(Some(4.0)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(0.5)), StringCharAtResult::Value('t'));
        assert_eq!(s.char_at(None), StringCharAtResult::Value('t'));
        assert_eq!(s.char_at(Some(f64::INFINITY)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(f64::NEG_INFINITY)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(-1.0)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(-1.1)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(-1_073_741_825.0)), StringCharAtResult::OutOfRange);
    }
}
