use std::{borrow::Cow, error::Error, fmt, ops::Deref};

/// Error returned by [`SourceType::from_path`] when the file extension is not
/// found or recognized.
///
/// [`SourceType::from_path`]: `crate::SourceType::from_path`
#[derive(Debug)]
pub struct UnknownExtension(/* msg */ pub(crate) Cow<'static, str>);

impl Deref for UnknownExtension {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}
impl UnknownExtension {
    #[inline]
    pub(crate) fn new<S: Into<Cow<'static, str>>>(ext: S) -> Self {
        Self(ext.into())
    }
}

impl fmt::Display for UnknownExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown file extension: {}", self.0)
    }
}

impl Error for UnknownExtension {}
