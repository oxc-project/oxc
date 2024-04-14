use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_span::Span;

#[derive(Debug, Error, Diagnostic)]
#[error("pragma and pragmaFrag cannot be set when runtime is automatic.")]
#[diagnostic(severity(warning), help("Remove `pragma` and `pragmaFrag` options."))]
pub struct PragmaAndPragmaFragCannotBeSet;

#[derive(Debug, Error, Diagnostic)]
#[error("importSource cannot be set when runtime is classic.")]
#[diagnostic(severity(warning), help("Remove `importSource` option."))]
pub struct ImportSourceCannotBeSet;

#[derive(Debug, Error, Diagnostic)]
#[error("Namespace tags are not supported by default. React's JSX doesn't support namespace tags. You can set `throwIfNamespace: false` to bypass this warning.")]
#[diagnostic(severity(warning))]
pub struct NamespaceDoesNotSupport(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Please provide an explicit key value. Using \"key\" as a shorthand for \"key={{true}}\" is not allowed.")]
#[diagnostic(severity(warning))]
pub struct ValuelessKey(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Spread children are not supported in React.")]
#[diagnostic(severity(warning))]
pub struct SpreadChildrenAreNotSupported(#[label] pub Span);
