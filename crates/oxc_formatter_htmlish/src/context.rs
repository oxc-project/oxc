use oxc_formatter_core::{FormatContext, SourceText};

use crate::{format::HtmlishEmbeddedIn, options::HtmlishFormatOptions, tree::Tree};

/// Formatting context for htmlish.
pub struct HtmlishFormatContext<'a> {
    options: HtmlishFormatOptions,
    source_text: SourceText<'a>,
    tree: &'a Tree<'a>,
    embedded_in: Option<HtmlishEmbeddedIn>,
}

impl<'a> HtmlishFormatContext<'a> {
    pub fn new(
        options: HtmlishFormatOptions,
        tree: &'a Tree<'a>,
        embedded_in: Option<HtmlishEmbeddedIn>,
    ) -> Self {
        Self { options, source_text: SourceText::new(tree.source()), tree, embedded_in }
    }

    pub fn embedded_in(&self) -> Option<HtmlishEmbeddedIn> {
        self.embedded_in
    }

    /// The preprocess table, with the arena lifetime so it outlives borrows of the formatter.
    pub fn tree(&self) -> &'a Tree<'a> {
        self.tree
    }
}

impl FormatContext for HtmlishFormatContext<'_> {
    type Options = HtmlishFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        &self.source_text
    }
}
