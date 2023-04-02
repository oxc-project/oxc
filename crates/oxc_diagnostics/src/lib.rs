//! Diagnostics Wrapper
//! Exports `thiserror` and `miette`

mod graphic_reporter;
mod graphical_theme;

use std::path::PathBuf;

pub use graphic_reporter::GraphicalReportHandler;
pub use miette;
use miette::Diagnostic;
use oxc_ast::{Atom, Span};
pub use thiserror;
use thiserror::Error;

pub type Error = miette::Error;
pub type Severity = miette::Severity;
pub type Report = miette::Report;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Diagnostic)]
#[error("Identifier `{0}` has already been declared")]
#[diagnostic()]
pub struct Redeclaration(
    pub Atom,
    #[label("`{0}` has already been declared here")] pub Span,
    #[label("It can not be redeclared here")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("File is too long to fit on the screen")]
#[diagnostic(help("{0:?} seems like a minified file"))]
pub struct MinifiedFileError(pub PathBuf);
