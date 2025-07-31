use crate::to_integer_or_infinity::ToIntegerOrInfinity;

/// `String.prototype.includes ( searchString [ , position ] )`
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.includes>
pub trait StringIncludes {
    fn includes(&self, search_string: &str, position: Option<f64>) -> bool;
}

impl StringIncludes for &str {
    fn includes(&self, search_string: &str, position: Option<f64>) -> bool {
        let len = self.chars().count();
        
        // 1. Let pos be ? ToIntegerOrInfinity(position).
        let pos: f64 = position.map_or(0.0, |x| x.to_integer_or_infinity());
        
        // 2. Let start be min(max(pos, 0), len).
        let start = pos.max(0.0).min(len as f64) as usize;
        
        // 3. Let substring be the substring of S from start.
        let substring = if start >= len {
            ""
        } else {
            &self.chars().skip(start).collect::<String>()
        };
        
        // 4. Return true if substring contains searchString; otherwise, return false.
        substring.contains(search_string)
    }
}

impl StringIncludes for String {
    fn includes(&self, search_string: &str, position: Option<f64>) -> bool {
        self.as_str().includes(search_string, position)
    }
}

#[cfg(test)]
mod test {
    use super::StringIncludes;

    #[test]
    fn test_string_includes() {
        // Basic functionality
        assert_eq!("hello world".includes("world", None), true);
        assert_eq!("hello world".includes("WORLD", None), false);
        assert_eq!("hello world".includes("", None), true);
        assert_eq!("".includes("", None), true);
        assert_eq!("".includes("hello", None), false);
        
        // With position
        assert_eq!("hello world".includes("world", Some(0.0)), true);
        assert_eq!("hello world".includes("world", Some(6.0)), true);
        assert_eq!("hello world".includes("world", Some(7.0)), false);
        assert_eq!("hello world".includes("hello", Some(1.0)), false);
        
        // Negative position (should be treated as 0)
        assert_eq!("hello world".includes("hello", Some(-1.0)), true);
        assert_eq!("hello world".includes("world", Some(-5.0)), true);
        
        // Position greater than length
        assert_eq!("hello".includes("hello", Some(10.0)), false);
        assert_eq!("hello".includes("", Some(10.0)), true);
        
        // Unicode support
        assert_eq!("café".includes("af", None), true);
        assert_eq!("café".includes("é", None), true);
        assert_eq!("🦀🦀🦀".includes("🦀", None), true);
        assert_eq!("🦀🦀🦀".includes("🦀🦀", Some(1.0)), true);
        
        // String type
        let owned = String::from("hello world");
        assert_eq!(owned.includes("world", None), true);
    }
}