use core::fmt;
use std::io::Read;
use std::ops;
use std::str;
use std::str::FromStr;
use std::{fs::File, io, path::Path};

/// A simple read-only String wrapper that pads the string with 16 bytes
/// This enables ignoring the last 16 bytes when parsing with SIMD
#[derive(Clone, Debug)]
pub struct PaddedStringView {
    wrapped: String,
}

trait TestTrait: Sized {}

impl TestTrait for PaddedStringView {}

// 16 empty bytes - TODO better way to define this?
const PADDING_BYTES: &'static str = match str::from_utf8(&[0; PaddedStringView::PADDING_SIZE])
// const PADDING_BYTES: &'static str = "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
// const PADDING_BYTES: [char; 16] = ['\0'; 16];

{
    Ok(res) => res,
    Err(_) => unreachable!(),
};

impl PaddedStringView {
    // Review note: this needs to be kept in sync with ELEMENTS in SIMD.
    // Should ELEMENTS take from here?
    pub const PADDING_SIZE: usize = 16;

    pub fn read_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let mut string = String::with_capacity(size as usize + PaddedStringView::PADDING_SIZE);
        file.read_to_string(&mut string)?;
        string.push_str(PADDING_BYTES);
        // string.extend(PADDING_BYTES);

        Ok(Self { wrapped: string })
    }

    pub fn unpadded_str(&self) -> &str {
        &(self.wrapped[0..self.wrapped.len() - 16])
    }
}

impl FromStr for PaddedStringView {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl From<&str> for PaddedStringView {
    #[inline]
    fn from(s: &str) -> PaddedStringView {
        let mut string = String::with_capacity(s.len() + PaddedStringView::PADDING_SIZE);
        string.push_str(s);
        string.push_str(PADDING_BYTES);

        Self { wrapped: string }
    }
}

impl From<&String> for PaddedStringView {
    #[inline]
    fn from(s: &String) -> PaddedStringView {
        Self::from(s.as_str())
    }
}

impl ops::Deref for PaddedStringView {
    type Target = String;

    #[inline]
    fn deref(&self) -> &String {
        &self.wrapped
    }
}

impl fmt::Display for PaddedStringView {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
