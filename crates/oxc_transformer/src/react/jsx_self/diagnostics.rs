use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_span::Span;

#[derive(Debug, Error, Diagnostic)]
#[error("Duplicate __self prop found.")]
#[diagnostic(severity(warning))]
pub struct DuplicateSelfProp(#[label] pub Span);
