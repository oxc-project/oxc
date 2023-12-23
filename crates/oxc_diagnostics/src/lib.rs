//! Diagnostics Wrapper
//! Exports `thiserror` and `miette`

mod service;

use std::path::PathBuf;

pub use crate::service::{DiagnosticSender, DiagnosticService, DiagnosticTuple};
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

/// Initialize miette
///
/// Some options are forced for modern terminals, which should reduce number of system calls.
///
/// # Panics
///
/// * miette hook fails to install
pub fn init_miette() {
    use miette::{set_hook, GraphicalTheme, MietteHandlerOpts, ThemeCharacters, ThemeStyles};
    use owo_colors::style;
    let theme = GraphicalTheme {
        characters: ThemeCharacters::unicode(),
        styles: ThemeStyles { error: style().fg_rgb::<225, 80, 80>().bold(), ..ThemeStyles::rgb() },
    };
    let opts = MietteHandlerOpts::new()
        .width(400)
        .terminal_links(false)
        .unicode(true)
        .color(true)
        .graphical_theme(theme);
    set_hook(Box::new(move |_| Box::new(opts.clone().build()))).unwrap();
}
