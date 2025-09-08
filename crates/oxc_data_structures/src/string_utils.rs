//! TODO: Add more documentation here.
//! Efficient string manipulation utilities.

/// Optimized string utilities for common operations.
pub trait StrExt {
    /// Optimized `trim_start`, assuming input is likely to be only ASCII characters. Falls
    /// back to standard `trim_start` if non-ASCII characters are encountered.
    #[must_use]
    fn ascii_trim_start(&self) -> &str;
    /// Optimized `trim_end`, assuming input is likely to be only ASCII characters. Falls
    /// back to standard `trim_end` if non-ASCII characters are encountered.
    #[must_use]
    fn ascii_trim_end(&self) -> &str;
    /// Optimized `trim`, assuming input is likely to be only ASCII characters. Falls
    /// back to standard `trim` if non-ASCII characters are encountered.
    #[must_use]
    fn ascii_trim(&self) -> &str;
}

impl StrExt for str {
    fn ascii_trim_start(&self) -> &str {
        let mut iter = self.as_bytes().iter().enumerate();
        let index = loop {
            if let Some((index, &byte)) = iter.next() {
                match byte {
                    _ if is_ascii_whitespace(byte) => continue,
                    _ if !byte.is_ascii() => return cold_branch(|| self.trim_start()),
                    _ => break index,
                }
            }
            return "";
        };
        unsafe { self.get_unchecked(index..) }
    }

    fn ascii_trim_end(&self) -> &str {
        let mut iter = self.as_bytes().iter().enumerate().rev();
        let index = loop {
            if let Some((index, &byte)) = iter.next() {
                match byte {
                    _ if is_ascii_whitespace(byte) => continue,
                    _ if !byte.is_ascii() => return cold_branch(|| self.trim_end()),
                    _ => break index,
                }
            }
            return "";
        };
        unsafe { self.get_unchecked(..index + 1) }
    }

    fn ascii_trim(&self) -> &str {
        let mut iter = self.as_bytes().iter().enumerate();
        let start = loop {
            if let Some((index, &byte)) = iter.next() {
                match byte {
                    _ if is_ascii_whitespace(byte) => continue,
                    _ if !byte.is_ascii() => return cold_branch(|| self.trim()),
                    _ => break index,
                }
            }
            return "";
        };
        let mut iter = self.as_bytes().iter().enumerate().rev();
        let end = loop {
            if let Some((index, &byte)) = iter.next() {
                match byte {
                    _ if is_ascii_whitespace(byte) => continue,
                    _ if !byte.is_ascii() => return cold_branch(|| self.trim()),
                    _ => break index + 1,
                }
            }
            return "";
        };
        unsafe { self.get_unchecked(start..end) }
    }
}

#[inline]
fn is_ascii_whitespace(byte: u8) -> bool {
    const VT: u8 = 0x0B;
    const FF: u8 = 0x0C;
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | VT | FF)
}

/// Call a closure while hinting to compiler that this branch is rarely taken.
///
/// "Cold trampoline function", suggested in:
/// <https://users.rust-lang.org/t/is-cold-the-only-reliable-way-to-hint-to-branch-predictor/106509/2>
#[cold]
#[inline(never)]
pub fn cold_branch<F: FnOnce() -> T, T>(f: F) -> T {
    f()
}

#[cfg(test)]
mod test {
    use crate::string_utils::StrExt as _;

    #[test]
    fn test_ascii_trim_start() {
        assert_eq!("   hello".ascii_trim_start(), "hello");
        assert_eq!("\n\t  hello".ascii_trim_start(), "hello");
        assert_eq!("hello".ascii_trim_start(), "hello");
        assert_eq!("   ".ascii_trim_start(), "");
        assert_eq!("".ascii_trim_start(), "");
        assert_eq!("   こんにちは".ascii_trim_start(), "こんにちは");
        assert_eq!("   hello こんにちは".ascii_trim_start(), "hello こんにちは");
    }

    #[test]
    fn test_ascii_trim_end() {
        assert_eq!("hello   ".ascii_trim_end(), "hello");
        assert_eq!("hello  \n\t ".ascii_trim_end(), "hello");
        assert_eq!("  hello  \n\t ".ascii_trim_end(), "  hello");
        assert_eq!("hello".ascii_trim_end(), "hello");
        assert_eq!("   ".ascii_trim_end(), "");
        assert_eq!("".ascii_trim_end(), "");
        assert_eq!("  こんにちは   ".ascii_trim_end(), "  こんにちは");
        assert_eq!("  こんにちは hello   ".ascii_trim_end(), "  こんにちは hello");
    }

    #[test]
    fn test_ascii_trim() {
        assert_eq!("   hello   ".ascii_trim(), "hello");
        assert_eq!("  \n\t hello  \n\t ".ascii_trim(), "hello");
        assert_eq!("  \n\t hello".ascii_trim(), "hello");
        assert_eq!("hello  ".ascii_trim(), "hello");
        assert_eq!("hello".ascii_trim(), "hello");
        assert_eq!("   ".ascii_trim(), "");
        assert_eq!("".ascii_trim(), "");
        assert_eq!("   こんにちは   ".ascii_trim(), "こんにちは");
        assert_eq!("   hello こんにちは   ".ascii_trim(), "hello こんにちは");
    }
}
