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

const PADDING_BYTES: &str = match str::from_utf8(&[0; PaddedStringView::PADDING_SIZE]) {
    Ok(res) => res,
    Err(_) => unreachable!(),
};

impl PaddedStringView {
    pub const PADDING_SIZE: usize = 16;

    /// # Errors
    ///
    /// This function will return an error if `path` does not already exist.
    /// Other errors may also be returned according to [`OpenOptions::open`].
    ///
    /// It will also return an error if it encounters while reading an error
    /// of a kind other than [`io::ErrorKind::Interrupted`],
    /// or if the contents of the file are not valid UTF-8.
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let size = usize::try_from(size).map_or(usize::MAX, |x| x);
        let mut string = String::with_capacity(size + Self::PADDING_SIZE);
        file.read_to_string(&mut string)?;
        string.push_str(PADDING_BYTES);

        Ok(Self { wrapped: string })
    }

    #[must_use]
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
    fn from(s: &str) -> Self {
        let mut string = String::with_capacity(s.len() + Self::PADDING_SIZE);
        string.push_str(s);
        string.push_str(PADDING_BYTES);

        Self { wrapped: string }
    }
}

impl From<&String> for PaddedStringView {
    #[inline]
    fn from(s: &String) -> Self {
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
