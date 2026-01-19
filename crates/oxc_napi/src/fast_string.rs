//! Fast string conversion for NAPI.
//!
//! This module provides `FastString`, an optimized string type for converting
//! Rust strings to JavaScript strings with minimal overhead.
//!
//! For ASCII-only strings (all bytes < 128), it uses external Latin1 string creation
//! which can avoid copying the string data into V8's heap (when the `napi10` feature
//! is enabled and supported by Node.js).
//! For non-ASCII strings, it falls back to standard UTF-8 string conversion.

use std::ptr;

use napi::{
    bindgen_prelude::{Result, ToNapiValue},
    sys,
};

#[cfg(feature = "napi10")]
use napi::{Env, JsStringLatin1};

/// A string wrapper optimized for zero-copy conversion to JavaScript strings.
///
/// `FastString` analyzes the input string at construction time and chooses
/// the optimal conversion strategy:
///
/// - **ASCII strings**: Uses Latin1 external string creation, which can avoid
///   copying the string data into V8's heap (when supported by Node.js and
///   the `napi10` feature is enabled).
/// - **Non-ASCII strings**: Falls back to standard UTF-8 string conversion.
///
/// # Example
///
/// ```ignore
/// use oxc_napi::FastString;
///
/// let fast = FastString::new("hello world".to_string());
/// // When converted to NAPI value, uses optimized Latin1 path
///
/// let fast = FastString::new("hello世界".to_string());
/// // Falls back to standard UTF-8 conversion
/// ```
pub struct FastString {
    inner: FastStringInner,
}

enum FastStringInner {
    /// All bytes are < 128, can use external Latin1 string
    Ascii(Vec<u8>),
    /// Has non-ASCII bytes, must use standard UTF-8 copy
    NonAscii(String),
}

/// Check if all bytes in the string are ASCII (< 128).
/// Uses SIMD-friendly reduction for better performance on large strings.
#[inline]
fn is_ascii_fast(bytes: &[u8]) -> bool {
    // Use a bitwise OR reduction - if any byte has the high bit set,
    // the result will have the high bit set
    bytes.iter().fold(0u8, |acc, &b| acc | b) < 128
}

impl FastString {
    /// Create a new `FastString` from a `String`.
    ///
    /// This analyzes the string to determine if it's ASCII-only,
    /// and stores it in the appropriate internal representation.
    #[inline]
    pub fn new(s: String) -> Self {
        // Use Rust's optimized is_ascii() which uses SIMD on supported platforms
        if s.is_ascii() {
            Self { inner: FastStringInner::Ascii(s.into_bytes()) }
        } else {
            Self { inner: FastStringInner::NonAscii(s) }
        }
    }

    /// Create a new `FastString` from a string slice.
    ///
    /// This clones the string data.
    #[inline]
    pub fn from_slice(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl From<String> for FastString {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for FastString {
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_slice(s)
    }
}

impl ToNapiValue for FastString {
    /// Convert the `FastString` to a NAPI value.
    ///
    /// For ASCII strings with the `napi10` feature enabled, this uses
    /// `JsStringLatin1::from_data()` which creates an external string
    /// without copying (when V8 supports it).
    /// For non-ASCII strings, it uses standard UTF-8 string conversion.
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
        match val.inner {
            FastStringInner::Ascii(bytes) => {
                #[cfg(feature = "napi10")]
                {
                    // Use external string API for zero-copy
                    if bytes.is_empty() {
                        return unsafe { create_latin1_string(env, &bytes) };
                    }
                    let env_wrapper = Env::from_raw(env);
                    let js_string = JsStringLatin1::from_data(&env_wrapper, bytes)?;
                    unsafe { ToNapiValue::to_napi_value(env, js_string) }
                }

                #[cfg(not(feature = "napi10"))]
                {
                    // Fallback to Latin1 copy
                    unsafe { create_latin1_string(env, &bytes) }
                }
            }
            FastStringInner::NonAscii(s) => {
                // Fall back to standard UTF-8 string conversion
                unsafe { ToNapiValue::to_napi_value(env, s) }
            }
        }
    }
}

/// Create a Latin1 string by copying the data (used as fallback).
///
/// # Safety
///
/// The `env` must be a valid NAPI environment pointer.
#[inline]
unsafe fn create_latin1_string(env: sys::napi_env, bytes: &[u8]) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    // SAFETY: napi_create_string_latin1 is safe to call with valid env and data
    #[expect(clippy::cast_possible_wrap)]
    let status = unsafe {
        sys::napi_create_string_latin1(
            env,
            bytes.as_ptr().cast(),
            bytes.len() as isize,
            &raw mut result,
        )
    };
    if status != sys::Status::napi_ok {
        return Err(napi::Error::from_status(napi::Status::from(status)));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_detection() {
        // ASCII string
        let fast = FastString::new("hello world".to_string());
        assert!(matches!(fast.inner, FastStringInner::Ascii(_)));

        // Non-ASCII string (contains Unicode)
        let fast = FastString::new("hello\u{4e2d}\u{6587}".to_string());
        assert!(matches!(fast.inner, FastStringInner::NonAscii(_)));

        // Empty string is ASCII
        let fast = FastString::new(String::new());
        assert!(matches!(fast.inner, FastStringInner::Ascii(_)));

        // String with control characters (still < 128)
        let fast = FastString::new("hello\t\n\r".to_string());
        assert!(matches!(fast.inner, FastStringInner::Ascii(_)));
    }

    #[test]
    fn test_is_ascii_fast() {
        assert!(is_ascii_fast(b"hello world"));
        assert!(is_ascii_fast(b""));
        assert!(is_ascii_fast(b"\t\n\r"));
        assert!(!is_ascii_fast(&[128]));
        assert!(!is_ascii_fast(&[255]));
        assert!(!is_ascii_fast(b"hello\x80world"));
    }

    #[test]
    fn test_from_impls() {
        let _fast: FastString = "hello".into();
        let _fast: FastString = String::from("world").into();
    }
}
