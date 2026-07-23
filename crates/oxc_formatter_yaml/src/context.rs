use std::cell::Cell;

use oxc_formatter_core::{FormatContext, SourceText};
use oxc_span::Span;

use crate::{comments::Comments, options::YamlFormatOptions};

/// Formatting context for YAML.
pub struct YamlFormatContext<'a> {
    options: YamlFormatOptions,
    source_text: SourceText<'a>,
    comments: Comments<'a>,
    /// Number of enclosing block collections (Prettier's `parentIndent` for block scalars).
    /// Maintained by `write_mapping` / `write_sequence`.
    collection_depth: Cell<u32>,
    /// End offset of the stream's last descendant node;
    /// block scalars compare against it for Prettier's `isLastDescendantNode`.
    last_descendant_end: u32,
}

impl<'a> YamlFormatContext<'a> {
    pub fn new(
        options: YamlFormatOptions,
        source_code: &'a str,
        comments: &'a [Span],
        last_descendant_end: u32,
    ) -> Self {
        Self {
            options,
            source_text: SourceText::new(source_code),
            comments: Comments::new(comments),
            collection_depth: Cell::new(0),
            last_descendant_end,
        }
    }

    pub fn collection_depth(&self) -> &Cell<u32> {
        &self.collection_depth
    }

    pub fn last_descendant_end(&self) -> u32 {
        self.last_descendant_end
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

impl FormatContext for YamlFormatContext<'_> {
    type Options = YamlFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        &self.source_text
    }
}
