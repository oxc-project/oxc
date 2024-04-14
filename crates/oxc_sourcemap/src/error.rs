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

/// The result of decoding.
pub type Result<T> = std::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::BadJson(err)
    }
}
