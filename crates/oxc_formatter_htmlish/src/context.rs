use oxc_formatter_core::{FormatContext, SourceText};

use crate::{options::HtmlishFormatOptions, tree::Tree};

/// Formatting context for htmlish.
pub struct HtmlishFormatContext<'a> {
    options: HtmlishFormatOptions,
    source_text: SourceText<'a>,
    tree: &'a Tree<'a>,
}

impl<'a> HtmlishFormatContext<'a> {
    pub fn new(options: HtmlishFormatOptions, tree: &'a Tree<'a>) -> Self {
        Self { options, source_text: SourceText::new(tree.source()), tree }
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
