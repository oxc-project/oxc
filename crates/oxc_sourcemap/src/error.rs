use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    /// a VLQ string was malformed and data was left over
    VlqLeftover,
    /// a VLQ string was empty and no values could be decoded.
    VlqNoValues,
    /// The input encoded a number that didn't fit into i64.
    VlqOverflow,
    /// `serde_json` parsing failure
    BadJson(serde_json::Error),
    /// a mapping segment had an unsupported size
    BadSegmentSize(u32),
    /// a reference to a non existing source was encountered
    BadSourceReference(u32),
    /// a reference to a non existing name was encountered
    BadNameReference(u32),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::VlqLeftover => write!(f, "VLQ string was malformed and data was left over"),
            Error::VlqNoValues => write!(f, "VLQ string was empty and no values could be decoded"),
            Error::VlqOverflow => write!(f, "The input encoded a number that didn't fit into i64"),
            Error::BadJson(err) => write!(f, "JSON parsing error: {err}"),
            Error::BadSegmentSize(size) => {
                write!(f, "Mapping segment had an unsupported size of {size}")
            }
            Error::BadSourceReference(idx) => {
                write!(f, "Reference to non-existing source at position {idx}")
            }
            Error::BadNameReference(idx) => {
                write!(f, "Reference to non-existing name at position {idx}")
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Self::BadJson(err) = self {
            Some(err)
        } else {
            None
        }
    }
}

/// The result of decoding.
pub type Result<T> = std::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::BadJson(err)
    }
}
