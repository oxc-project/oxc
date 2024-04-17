use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_span::Span;

#[derive(Debug, Error, Diagnostic)]
#[error("Duplicate __source prop found.")]
#[diagnostic(severity(warning))]
pub struct DuplicateSourceProp(#[label] pub Span);
