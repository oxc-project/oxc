use std::path::PathBuf;

use crate::request::RequestError;

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveError {
    NotFound,
    RequestError(RequestError),
    JSONError(JSONError),
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
        Self::JSONError(JSONError {
            path,
            message: error.to_string(),
            line: error.line(),
            column: error.column(),
        })
    }
}
