//! Diagnostics Wrapper
//! Exports `thiserror` and `miette`

mod graphic_reporter;
mod graphical_theme;
mod service;

use std::path::PathBuf;

pub use crate::service::{DiagnosticSender, DiagnosticService, DiagnosticTuple};
pub use graphic_reporter::{GraphicalReportHandler, GraphicalTheme};
pub use miette;
pub use thiserror;

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

/// Initialize data that relies on system calls (e.g. `is_atty`) so they don't block subsequent threads.
pub fn init_miette() {
    let c = supports_color::on(supports_color::Stream::Stderr).is_some();
    let w = terminal_size::terminal_size()
        .unwrap_or((terminal_size::Width(80), terminal_size::Height(0)))
        .0
         .0 as usize;
    let u = supports_unicode::on(supports_unicode::Stream::Stderr);
    miette::set_hook(Box::new(move |_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .color(c)
                .terminal_links(false)
                .width(w)
                .unicode(u)
                .build(),
        )
    }))
    .unwrap();
}
