use crate::to_integer_or_infinity::ToIntegerOrInfinity;

/// `String.prototype.repeat ( count )`
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.repeat>
pub trait StringRepeat {
    fn repeat_string(&self, count: f64) -> Result<String, &'static str>;
}

impl StringRepeat for &str {
    fn repeat_string(&self, count: f64) -> Result<String, &'static str> {
        // 1. Let n be ? ToIntegerOrInfinity(count).
        let n = count.to_integer_or_infinity();

        // 2. If n < 0 or n is +∞, throw a RangeError exception.
        if n < 0.0 || n == f64::INFINITY {
            return Err("RangeError: repeat count must be non-negative and finite");
        }

        // 3. If n is 0, return the empty String.
        if n == 0.0 {
            return Ok(String::new());
        }

        let count_usize = n as usize;

        // Prevent excessive memory usage
        if count_usize > 1_000_000 {
            return Err("RangeError: repeat count too large");
        }

        // 4. Return the string value that is made from n copies of S appended together.
        Ok(self.repeat(count_usize))
    }
}

impl StringRepeat for String {
    fn repeat_string(&self, count: f64) -> Result<String, &'static str> {
        self.as_str().repeat_string(count)
    }
}

/// `String.prototype.trim ( )`
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.trim>
pub trait StringTrim {
    fn trim_string(&self) -> String;
}

impl StringTrim for &str {
    fn trim_string(&self) -> String {
        // The trim method removes leading and trailing white space and line terminator characters
        self.trim().to_string()
    }
}

impl StringTrim for String {
    fn trim_string(&self) -> String {
        self.as_str().trim_string()
    }
}

/// `String.prototype.trimStart ( )`
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.trimstart>
pub trait StringTrimStart {
    fn trim_start_string(&self) -> String;
}

impl StringTrimStart for &str {
    fn trim_start_string(&self) -> String {
        self.trim_start().to_string()
    }
}

impl StringTrimStart for String {
    fn trim_start_string(&self) -> String {
        self.as_str().trim_start_string()
    }
}

/// `String.prototype.trimEnd ( )`
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.trimend>
pub trait StringTrimEnd {
    fn trim_end_string(&self) -> String;
}

impl StringTrimEnd for &str {
    fn trim_end_string(&self) -> String {
        self.trim_end().to_string()
    }
}

impl StringTrimEnd for String {
    fn trim_end_string(&self) -> String {
        self.as_str().trim_end_string()
    }
}

#[cfg(test)]
mod test {
    use super::{StringRepeat, StringTrim, StringTrimEnd, StringTrimStart};

    #[test]
    fn test_string_repeat() {
        // Basic functionality
        assert_eq!("abc".repeat_string(0.0).unwrap(), "");
        assert_eq!("abc".repeat_string(1.0).unwrap(), "abc");
        assert_eq!("abc".repeat_string(3.0).unwrap(), "abcabcabc");
        assert_eq!("".repeat_string(5.0).unwrap(), "");

        // Fractional counts (should be truncated)
        assert_eq!("ab".repeat_string(2.5).unwrap(), "abab");

        // Error cases
        assert!("abc".repeat_string(-1.0).is_err());
        assert!("abc".repeat_string(f64::INFINITY).is_err());

        // NaN becomes 0 after ToIntegerOrInfinity, so should succeed
        assert_eq!("abc".repeat_string(f64::NAN).unwrap(), "");

        // Unicode support
        assert_eq!("🦀".repeat_string(3.0).unwrap(), "🦀🦀🦀");

        // String type
        let owned = String::from("test");
        assert_eq!(owned.repeat_string(2.0).unwrap(), "testtest");
    }

    #[test]
    fn test_string_trim() {
        // Basic functionality
        assert_eq!("  hello  ".trim_string(), "hello");
        assert_eq!("hello".trim_string(), "hello");
        assert_eq!("  ".trim_string(), "");
        assert_eq!("".trim_string(), "");

        // Different whitespace characters
        assert_eq!("\t\n\r hello \t\n\r".trim_string(), "hello");
        assert_eq!("\u{00A0}hello\u{00A0}".trim_string(), "hello"); // Non-breaking space

        // Unicode support
        assert_eq!("  café  ".trim_string(), "café");

        // String type
        let owned = String::from("  test  ");
        assert_eq!(owned.trim_string(), "test");
    }

    #[test]
    fn test_string_trim_start() {
        // Basic functionality
        assert_eq!("  hello  ".trim_start_string(), "hello  ");
        assert_eq!("hello  ".trim_start_string(), "hello  ");
        assert_eq!("  hello".trim_start_string(), "hello");
        assert_eq!("hello".trim_start_string(), "hello");

        // Different whitespace characters
        assert_eq!("\t\n\r hello \t\n\r".trim_start_string(), "hello \t\n\r");

        // String type
        let owned = String::from("  test");
        assert_eq!(owned.trim_start_string(), "test");
    }

    #[test]
    fn test_string_trim_end() {
        // Basic functionality
        assert_eq!("  hello  ".trim_end_string(), "  hello");
        assert_eq!("  hello".trim_end_string(), "  hello");
        assert_eq!("hello  ".trim_end_string(), "hello");
        assert_eq!("hello".trim_end_string(), "hello");

        // Different whitespace characters
        assert_eq!("\t\n\r hello \t\n\r".trim_end_string(), "\t\n\r hello");

        // String type
        let owned = String::from("test  ");
        assert_eq!(owned.trim_end_string(), "test");
    }
}
