use oxc_formatter_core::{FormatContext, SourceText};

use crate::{
    comments::{Comments, CssComment},
    format::CssEmbeddedIn,
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
    /// Inside an ICSS rule (`:import(...)` / `:export`): property names keep
    /// their case (Prettier's `insideIcssRuleNode`).
    in_icss_rule: std::cell::Cell<bool>,
    /// What the part is embedded in (embedded entry point only);
    /// css-in-js gates the printer's placeholder handling.
    embedded_in: Option<CssEmbeddedIn>,
}

impl<'a> CssFormatContext<'a> {
    pub fn new(
        options: CssFormatOptions,
        source_code: &'a str,
        comments: &'a [CssComment],
        embedded_in: Option<CssEmbeddedIn>,
    ) -> Self {
        Self {
            options,
            source_text: SourceText::new(source_code),
            comments: Comments::new(comments),
            in_less_detached: std::cell::Cell::new(false),
            in_icss_rule: std::cell::Cell::new(false),
            embedded_in,
        }
    }

    /// Whether the source may contain css-in-js `${}` placeholder markers.
    pub fn template_placeholders(&self) -> bool {
        self.embedded_in == Some(CssEmbeddedIn::CssInJsTemplate)
    }

    /// The source is an HTML `style` attribute's value.
    pub fn is_style_attribute(&self) -> bool {
        self.embedded_in == Some(CssEmbeddedIn::HtmlStyleAttribute)
    }

    pub fn in_less_detached(&self) -> &std::cell::Cell<bool> {
        &self.in_less_detached
    }

    pub fn in_icss_rule(&self) -> &std::cell::Cell<bool> {
        &self.in_icss_rule
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
