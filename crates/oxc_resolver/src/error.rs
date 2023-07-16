use std::path::PathBuf;

use crate::request::RequestError;

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveError {
    NotFound,
    Request(RequestError),
    ExtensionAlias,
    JSON(JSONError),
}

#[derive(Debug, Eq, PartialEq)]
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
