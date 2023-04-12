//! Diagnostics Wrapper
//! Exports `thiserror` and `miette`

mod graphic_reporter;
mod graphical_theme;

pub use graphic_reporter::GraphicalReportHandler;
pub use miette;
pub use thiserror;

pub type Error = miette::Error;
pub type Severity = miette::Severity;
pub type Report = miette::Report;

pub type Result<T> = std::result::Result<T, Error>;
