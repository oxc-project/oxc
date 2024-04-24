use serde::Deserialize;

use super::ProtocolError;

#[derive(Debug, Deserialize)]
pub(super) struct Response<T> {
    // pub seq: usize,
    // #[serde(rename = "type")]
    // pub kind: &'a str,
    // pub command: &'a str,
    // pub request_seq: usize,
    pub success: bool,
    #[serde(default = "Option::default")]
    pub body: Option<T>,
    #[serde(default = "Option::default")]
    pub message: Option<String>,
}

impl<'a, T> From<Response<T>> for Result<T, ProtocolError> {
    fn from(value: Response<T>) -> Self {
        if value.success {
            value.body.ok_or_else(|| ProtocolError::ResultMissing)
        } else {
            Self::Err(ProtocolError::CommandFailed(value.message.unwrap_or("unknown error".into())))
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
pub struct EmptyResponse {}

#[derive(Debug, Deserialize)]
pub struct BoolResponse {
    pub result: bool,
}
