use oxc_formatter_core::{FormatContext, SourceText};

use crate::{
    comments::{Comments, CssComment},
    options::CssFormatOptions,
};

/// Formatting context for CSS/SCSS/Less.
pub struct CssFormatContext<'a> {
    options: CssFormatOptions,
    source_text: SourceText<'a>,
    comments: Comments<'a>,
    /// Inside a Less detached ruleset (`@var: { ... }`): property names keep
    /// their case (Prettier checks `parentNode.variable`).
    in_less_detached: std::cell::Cell<bool>,
    /// Current block nesting depth (rules/at-rules).
    block_depth: std::cell::Cell<u32>,
    /// The source may contain css-in-js `${}` placeholder markers
    /// (embedded entry point only); gates the printer's placeholder handling.
    template_placeholders: bool,
}

impl<'a> CssFormatContext<'a> {
    pub fn new(
        options: CssFormatOptions,
        source_code: &'a str,
        comments: &'a [CssComment],
        template_placeholders: bool,
    ) -> Self {
        Self {
            options,
            source_text: SourceText::new(source_code),
            comments: Comments::new(comments),
            in_less_detached: std::cell::Cell::new(false),
            block_depth: std::cell::Cell::new(0),
            template_placeholders,
        }
    }

    /// Whether the source may contain css-in-js `${}` placeholder markers.
    pub fn template_placeholders(&self) -> bool {
        self.template_placeholders
    }

    pub fn block_depth(&self) -> &std::cell::Cell<u32> {
        &self.block_depth
    }

    pub fn in_less_detached(&self) -> &std::cell::Cell<bool> {
        &self.in_less_detached
    }

    /// Returns the source text with the arena lifetime (vs the trait's borrow-elided `&str`).
    pub fn source_text(&self) -> SourceText<'a> {
        self.source_text
    }

    /// Returns the comment cursor.
    pub fn comments(&self) -> &Comments<'a> {
        &self.comments
    }
}

impl FormatContext for CssFormatContext<'_> {
    type Options = CssFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        &self.source_text
    }
}
