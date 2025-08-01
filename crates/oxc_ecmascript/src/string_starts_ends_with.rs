use crate::to_integer_or_infinity::ToIntegerOrInfinity;

/// `String.prototype.startsWith ( searchString [ , position ] )`
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.startswith>
pub trait StringStartsWith {
    fn starts_with_at(&self, search_string: &str, position: Option<f64>) -> bool;
}

impl StringStartsWith for &str {
    fn starts_with_at(&self, search_string: &str, position: Option<f64>) -> bool {
        let len = self.chars().count();

        // 1. Let pos be ? ToIntegerOrInfinity(position).
        let pos: f64 = position.map_or(0.0, |x| x.to_integer_or_infinity());

        // 2. Let start be min(max(pos, 0), len).
        let start = pos.max(0.0).min(len as f64) as usize;

        // 3. Let searchLength be the length of searchString.
        let search_length = search_string.chars().count();

        // 4. If searchLength + start > len, return false.
        if search_length + start > len {
            return false;
        }

        // 5. Let substring be the substring of S from start to start + searchLength.
        let substring: String = self.chars().skip(start).take(search_length).collect();

        // 6. Return SameValueNonNumeric(substring, searchString).
        substring == search_string
    }
}

impl StringStartsWith for String {
    fn starts_with_at(&self, search_string: &str, position: Option<f64>) -> bool {
        self.as_str().starts_with_at(search_string, position)
    }
}

/// `String.prototype.endsWith ( searchString [ , length ] )`
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.endswith>
pub trait StringEndsWith {
    fn ends_with_at(&self, search_string: &str, length: Option<f64>) -> bool;
}

impl StringEndsWith for &str {
    fn ends_with_at(&self, search_string: &str, length: Option<f64>) -> bool {
        let len = self.chars().count();

        // 1. Let endPosition be ? ToIntegerOrInfinity(length).
        let end_position: f64 = length.map_or(len as f64, |x| x.to_integer_or_infinity());

        // 2. Let pos be min(max(endPosition, 0), len).
        let pos = end_position.max(0.0).min(len as f64) as usize;

        // 3. Let searchLength be the length of searchString.
        let search_length = search_string.chars().count();

        // 4. If searchLength > pos, return false.
        if search_length > pos {
            return false;
        }

        // 5. Let start be pos - searchLength.
        let start = pos - search_length;

        // 6. Let substring be the substring of S from start to pos.
        let substring: String = self.chars().skip(start).take(search_length).collect();

        // 7. Return SameValueNonNumeric(substring, searchString).
        substring == search_string
    }
}

impl StringEndsWith for String {
    fn ends_with_at(&self, search_string: &str, length: Option<f64>) -> bool {
        self.as_str().ends_with_at(search_string, length)
    }
}

#[cfg(test)]
mod test {
    use super::{StringEndsWith, StringStartsWith};

    #[test]
    fn test_string_starts_with() {
        // Basic functionality
        assert_eq!("hello world".starts_with_at("hello", None), true);
        assert_eq!("hello world".starts_with_at("world", None), false);
        assert_eq!("hello world".starts_with_at("", None), true);
        assert_eq!("".starts_with_at("", None), true);
        assert_eq!("".starts_with_at("hello", None), false);

        // With position
        assert_eq!("hello world".starts_with_at("world", Some(6.0)), true);
        assert_eq!("hello world".starts_with_at("hello", Some(1.0)), false);
        assert_eq!("hello world".starts_with_at("ello", Some(1.0)), true);

        // Negative position (should be treated as 0)
        assert_eq!("hello world".starts_with_at("hello", Some(-1.0)), true);

        // Position greater than length
        assert_eq!("hello".starts_with_at("hello", Some(10.0)), false);
        assert_eq!("hello".starts_with_at("", Some(10.0)), true);

        // Unicode support
        assert_eq!("café".starts_with_at("ca", None), true);
        assert_eq!("🦀🦀🦀".starts_with_at("🦀", None), true);
        assert_eq!("🦀🦀🦀".starts_with_at("🦀🦀", Some(1.0)), true);

        // String type
        let owned = String::from("hello world");
        assert_eq!(owned.starts_with_at("hello", None), true);
    }

    #[test]
    fn test_string_ends_with() {
        // Basic functionality
        assert_eq!("hello world".ends_with_at("world", None), true);
        assert_eq!("hello world".ends_with_at("hello", None), false);
        assert_eq!("hello world".ends_with_at("", None), true);
        assert_eq!("".ends_with_at("", None), true);
        assert_eq!("".ends_with_at("hello", None), false);

        // With length
        assert_eq!("hello world".ends_with_at("hello", Some(5.0)), true);
        assert_eq!("hello world".ends_with_at("world", Some(5.0)), false);
        assert_eq!("hello world".ends_with_at("ell", Some(4.0)), true);

        // Length greater than string length
        assert_eq!("hello".ends_with_at("hello", Some(10.0)), true);

        // Zero length
        assert_eq!("hello".ends_with_at("", Some(0.0)), true);
        assert_eq!("hello".ends_with_at("hello", Some(0.0)), false);

        // Unicode support
        assert_eq!("café".ends_with_at("fé", None), true);
        assert_eq!("🦀🦀🦀".ends_with_at("🦀", None), true);
        assert_eq!("🦀🦀🦀".ends_with_at("🦀🦀", Some(2.0)), true);

        // String type
        let owned = String::from("hello world");
        assert_eq!(owned.ends_with_at("world", None), true);
    }
}
