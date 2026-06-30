use oxc_formatter_core::{FormatContext, SourceText};
use oxc_span::Span;

use crate::{comments::Comments, options::GraphqlFormatOptions};

/// Formatting context for GraphQL.
pub struct GraphqlFormatContext<'a> {
    options: GraphqlFormatOptions,
    source_text: SourceText<'a>,
    comments: Comments<'a>,
}

impl<'a> GraphqlFormatContext<'a> {
    pub fn new(options: GraphqlFormatOptions, source_code: &'a str, comments: &'a [Span]) -> Self {
        Self {
            options,
            source_text: SourceText::new(source_code),
            comments: Comments::new(comments),
        }
    }

    /// Returns the source text with the arena lifetime (vs the trait's borrow-elided `&str`).
    /// Slices taken via this method carry the `'a` lifetime,
    /// so they don't have to be re-allocated for `text(...)`.
    pub fn source_text(&self) -> SourceText<'a> {
        self.source_text
    }

    /// Returns the comment cursor.
    pub fn comments(&self) -> &Comments<'a> {
        &self.comments
    }
}

impl FormatContext for GraphqlFormatContext<'_> {
    type Options = GraphqlFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        &self.source_text
    }
}
