//! The [`GraphicalReportHandler`] type and its builder API.
//!
//! This module holds the handler's theme, terminal width, and link style. The
//! actual rendering lives in the sibling modules (`report`, `snippet`, …).

use std::io::{self, IsTerminal};

use crate::GraphicalTheme;

#[derive(Debug, Clone)]
pub struct GraphicalReportHandler {
    /// How to render links.
    pub(super) links: LinkStyle,
    /// Terminal width to wrap at.
    pub(super) termwidth: usize,
    /// How to style reports.
    pub(super) theme: GraphicalTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LinkStyle {
    Link,
    Text,
}

impl GraphicalReportHandler {
    /// Create a new `GraphicalReportHandler` with the default
    /// [`GraphicalTheme`]. This will use both unicode characters and colors.
    #[must_use]
    pub fn new() -> Self {
        let is_terminal = io::stdout().is_terminal() && io::stderr().is_terminal();
        Self {
            links: if is_terminal { LinkStyle::Link } else { LinkStyle::Text },
            termwidth: 400,
            theme: GraphicalTheme::new(is_terminal),
        }
    }

    /// Create a new `GraphicalReportHandler` with a given [`GraphicalTheme`].
    #[must_use]
    pub fn new_themed(theme: GraphicalTheme) -> Self {
        Self { links: LinkStyle::Link, termwidth: 200, theme }
    }

    /// Whether to enable error code linkification using [`Diagnostic::url()`](crate::Diagnostic::url).
    #[must_use]
    pub fn with_links(mut self, links: bool) -> Self {
        self.links = if links { LinkStyle::Link } else { LinkStyle::Text };
        self
    }

    /// Set a theme for this handler.
    #[must_use]
    pub fn with_theme(mut self, theme: GraphicalTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the width to wrap the report at.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.termwidth = width;
        self
    }
}

impl Default for GraphicalReportHandler {
    fn default() -> Self {
        Self::new()
    }
}
