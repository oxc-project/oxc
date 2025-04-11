use oxc_ast::ast::Program;

use super::Comments;
use crate::options::FormatOptions;

/// Context object storing data relevant when formatting an object.
#[derive(Debug, Clone)]
pub struct FormatContext<'ast> {
    options: FormatOptions,

    source_text: &'ast str,

    comments: Comments,
}

impl<'ast> FormatContext<'ast> {
    pub fn new(program: &'ast Program<'ast>, options: FormatOptions) -> Self {
        Self {
            options,
            source_text: program.source_text,
            comments: Comments::from_oxc_comments(program),
        }
    }

    /// Returns the formatting options
    pub fn options(&self) -> &FormatOptions {
        &self.options
    }

    /// Returns a reference to the program's comments.
    pub fn comments(&self) -> &Comments {
        &self.comments
    }

    /// Returns the formatting options
    pub fn source_text(&self) -> &'ast str {
        self.source_text
    }
}
