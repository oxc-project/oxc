use serde::Deserialize;

use super::ProtocolError;

#[derive(Debug, Deserialize)]
pub(super) struct Response<'a, T> {
    // pub seq: usize,
    // #[serde(rename = "type")]
    // pub kind: &'a str,
    // pub command: &'a str,
    // pub request_seq: usize,
    pub success: bool,
    pub body: Option<T>,
    pub message: Option<&'a str>,
}

impl<'a, T> From<Response<'a, T>> for Result<T, ProtocolError> {
    fn from(value: Response<'a, T>) -> Self {
        if value.success {
            value.body.ok_or_else(|| ProtocolError::ResultMissing)
        } else {
            Self::Err(ProtocolError::CommandFailed(
                value.message.unwrap_or("unknown error").to_owned(),
            ))
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct StatusResponse {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeResponse {
    pub kind: String,
    pub text: String,
    #[serde(rename = "type")]
    pub type_text: String,
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct BoolResponse {
    pub result: bool,
}
