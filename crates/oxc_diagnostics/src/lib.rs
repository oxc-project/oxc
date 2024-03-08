//! Diagnostics Wrapper
//! Exports `thiserror` and `miette`

mod graphic_reporter;
mod graphical_theme;
mod reporter;
mod service;

use std::path::PathBuf;

pub use miette;
pub use thiserror;

pub use crate::{
    graphic_reporter::GraphicalReportHandler,
    graphical_theme::GraphicalTheme,
    service::{DiagnosticSender, DiagnosticService, DiagnosticTuple},
};

pub type Error = miette::Error;
pub type Severity = miette::Severity;
pub type Report = miette::Report;

pub type Result<T> = std::result::Result<T, Error>;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("File is too long to fit on the screen")]
#[diagnostic(help("{0:?} seems like a minified file"))]
pub struct MinifiedFileError(pub PathBuf);

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to open file {0:?} with error \"{1}\"")]
#[diagnostic(help("Failed to open file {0:?} with error \"{1}\""))]
pub struct FailedToOpenFileError(pub PathBuf, pub std::io::Error);
