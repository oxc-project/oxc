use crate::to_integer_or_infinity::ToIntegerOrInfinity;

/// `String.prototype.slice ( start , end )`
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.slice>
pub trait StringSlice {
    fn slice(&self, start: Option<f64>, end: Option<f64>) -> String;
}

impl StringSlice for &str {
    fn slice(&self, start: Option<f64>, end: Option<f64>) -> String {
        let len = self.chars().count();
        
        // 1. Let intStart be ? ToIntegerOrInfinity(start).
        let int_start = start.map_or(0.0, |x| x.to_integer_or_infinity());
        
        // 2. If intStart is -∞, let from be 0.
        // 3. Else if intStart < 0, let from be max(len + intStart, 0).
        // 4. Else, let from be min(intStart, len).
        let from = if int_start == f64::NEG_INFINITY {
            0
        } else if int_start < 0.0 {
            (len as f64 + int_start).max(0.0) as usize
        } else {
            (int_start as usize).min(len)
        };
        
        // 5. If end is undefined, let intEnd be len; else let intEnd be ? ToIntegerOrInfinity(end).
        let int_end = end.map_or(len as f64, |x| x.to_integer_or_infinity());
        
        // 6. If intEnd is -∞, let to be 0.
        // 7. Else if intEnd < 0, let to be max(len + intEnd, 0).
        // 8. Else, let to be min(intEnd, len).
        let to = if int_end == f64::NEG_INFINITY {
            0
        } else if int_end < 0.0 {
            (len as f64 + int_end).max(0.0) as usize
        } else {
            (int_end as usize).min(len)
        };
        
        // 9. Let span be max(to - from, 0).
        let span = if to > from { to - from } else { 0 };
        
        // 10. Return the substring of S from from to from + span.
        if span == 0 {
            String::new()
        } else {
            self.chars().skip(from).take(span).collect()
        }
    }
}

impl StringSlice for String {
    fn slice(&self, start: Option<f64>, end: Option<f64>) -> String {
        self.as_str().slice(start, end)
    }
}

#[cfg(test)]
mod test {
    use super::StringSlice;

    #[test]
    fn test_string_slice() {
        // Basic functionality
        assert_eq!("hello".slice(Some(1.0), Some(4.0)), "ell");
        assert_eq!("hello".slice(Some(1.0), None), "ello");
        assert_eq!("hello".slice(None, Some(4.0)), "hell");
        assert_eq!("hello".slice(None, None), "hello");
        
        // Negative indices
        assert_eq!("hello".slice(Some(-4.0), Some(-1.0)), "ell");
        assert_eq!("hello".slice(Some(-3.0), None), "llo");
        assert_eq!("hello".slice(None, Some(-1.0)), "hell");
        
        // Edge cases
        assert_eq!("hello".slice(Some(10.0), Some(20.0)), ""); // start > length
        assert_eq!("hello".slice(Some(3.0), Some(1.0)), ""); // start > end
        assert_eq!("hello".slice(Some(-10.0), Some(-20.0)), ""); // negative start > negative end
        assert_eq!("hello".slice(Some(-10.0), Some(10.0)), "hello"); // very negative start
        
        // Zero values
        assert_eq!("hello".slice(Some(0.0), Some(0.0)), "");
        assert_eq!("hello".slice(Some(0.0), Some(5.0)), "hello");
        
        // Unicode support
        assert_eq!("café".slice(Some(1.0), Some(3.0)), "af");
        assert_eq!("🦀🦀🦀".slice(Some(1.0), Some(2.0)), "🦀");
        
        // String type
        let owned = String::from("hello");
        assert_eq!(owned.slice(Some(1.0), Some(4.0)), "ell");
    }
}