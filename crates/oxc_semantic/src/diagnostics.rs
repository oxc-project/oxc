use oxc_ast::{Atom, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};

#[derive(Debug, Error, Diagnostic)]
#[error("Identifier `{0}` has already been declared")]
#[diagnostic()]
pub struct Redeclaration(
    pub Atom,
    #[label("`{0}` has already been declared here")] pub Span,
    #[label("It can not be redeclared here")] pub Span,
);
