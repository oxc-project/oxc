use std::fmt;

use crate::{miette::JSONReportHandler, Diagnostic, GraphicalReportHandler};

#[allow(clippy::large_enum_variant)] // Lerge size is fine because this is a singleton
#[derive(Debug)]
#[non_exhaustive]
pub enum DiagnosticReporter {
    Graphical(GraphicalReportHandler), // 288 bytes
    Json(JSONReportHandler),
}

impl Default for DiagnosticReporter {
    fn default() -> Self {
        Self::Graphical(GraphicalReportHandler::new())
    }
}

impl DiagnosticReporter {
    pub fn render_report<T: fmt::Write>(&self, f: &mut T, diagnostic: &(dyn Diagnostic)) {
        match self {
            Self::Graphical(handler) => handler.render_report(f, diagnostic).unwrap(),
            Self::Json(handler) => handler.render_report(f, diagnostic).unwrap(),
        }
    }
}
