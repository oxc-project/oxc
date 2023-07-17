use std::path::{Path, PathBuf};

use crate::request::RequestError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResolveError {
    /// Path not found
    NotFound(Box<Path>),

    /// Ignored path
    ///
    /// Derived from ignored path (false value) from browser field in package.json
    /// ```json
    /// {
    ///     "browser": {
    ///         "./module": false
    ///     }
    /// }
    /// ```
    /// See <https://github.com/defunctzombie/package-browser-field-spec#ignore-a-module>
    Ignored(Box<Path>),

    /// The provided path request cannot be parsed
    Request(RequestError),

    /// All of the aliased extension are not found
    ExtensionAlias,

    /// JSON parse error
    JSON(JSONError),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JSONError {
    pub path: PathBuf,
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ResolveError {
    pub(crate) fn from_serde_json_error(path: PathBuf, error: &serde_json::Error) -> Self {
        Self::JSON(JSONError {
            path,
            message: error.to_string(),
            line: error.line(),
            column: error.column(),
        })
    }
}
