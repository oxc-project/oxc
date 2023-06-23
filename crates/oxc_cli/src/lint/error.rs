use std::{
    error::Error as STDError,
    fmt::{self, Display},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Error {
    path: PathBuf,
    error: io::Error,
}

impl STDError for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} when accessing \"{}\"", self.error, self.path.display())
    }
}

impl Error {
    fn new(path: PathBuf, error: io::Error) -> Self {
        Self { path, error }
    }
}

// allow simple conversion from io::Result using the `with_path` method
pub trait ErrorWithPath<T> {
    /// Convert an io::Result to lint::error, which allows associating a path with the error for
    /// better error messages.
    fn with_path(self, path: impl AsRef<Path>) -> Result<T>;
}

impl<T> ErrorWithPath<T> for io::Result<T> {
    fn with_path(self, path: impl AsRef<Path>) -> Result<T> {
        match self {
            Ok(t) => Ok(t),
            Err(error) => Err(Error::new(path.as_ref().to_path_buf(), error)),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
