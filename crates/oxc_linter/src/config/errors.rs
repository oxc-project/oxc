use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
    Report,
};
use std::path::PathBuf;

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to parse config {0:?} with error {1:?}")]
#[diagnostic()]
pub struct FailedToParseConfigJsonError(pub PathBuf, pub String);

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to parse eslint config")]
#[diagnostic()]
pub struct FailedToParseConfigError(#[related] pub Vec<Report>);

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to parse config at {0:?} with error {1:?}")]
#[diagnostic()]
pub struct FailedToParseConfigPropertyError(pub &'static str, pub &'static str);

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to rule value {0:?} with error {1:?}")]
#[diagnostic()]
pub struct FailedToParseRuleValueError(pub String, pub &'static str);

#[derive(Debug, Error, Diagnostic)]
#[error(r#"Failed to parse rule severity, expected one of "allow", "off", "deny", "error" or "warn", but got {0:?}"#)]
#[diagnostic()]
pub struct FailedToParseAllowWarnDenyFromStringError(pub String);

#[derive(Debug, Error, Diagnostic)]
#[error(r#"Failed to parse rule severity, expected one of `0`, `1` or `2`, but got {0:?}"#)]
#[diagnostic()]
pub struct FailedToParseAllowWarnDenyFromNumberError(pub String);

#[derive(Debug, Error, Diagnostic)]
#[error(r#"Failed to parse rule severity, expected a string or a number, but got {0:?}"#)]
#[diagnostic()]
pub struct FailedToParseAllowWarnDenyFromJsonValueError(pub String);
