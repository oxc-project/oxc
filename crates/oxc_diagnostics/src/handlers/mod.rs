//! Renderers included with `oxc_diagnostics`.

pub use graphical::GraphicalReportHandler;
pub use json::JSONReportHandler;
pub use theme::GraphicalTheme;

mod graphical;
mod json;
mod theme;
