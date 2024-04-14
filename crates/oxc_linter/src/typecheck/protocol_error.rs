use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
#[derive(Debug, Error, Diagnostic)]
#[diagnostic()]
pub enum ProtocolError {
    #[error("unexpected character")]
    UnexpectedCharacter,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    StrUtf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    StringUtf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("command failed")]
    CommandFailed(String),
    #[error("missing result")]
    ResultMissing,
}
