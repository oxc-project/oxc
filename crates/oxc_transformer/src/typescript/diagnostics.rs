use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_span::Span;

#[derive(Debug, Error, Diagnostic)]
#[error("`import lib = require(...);` is only supported when compiling modules to CommonJS.\nPlease consider using `import lib from '...';` alongside Typescript's --allowSyntheticDefaultImports option, or add @babel/plugin-transform-modules-commonjs to your Babel config.")]
#[diagnostic(severity(warning))]
pub struct ImportEqualsRequireUnsupported(#[label] pub Span);
