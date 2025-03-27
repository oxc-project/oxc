use super::{
    CommentStyle, Comments, FormatOptions, FormatRule, SimpleFormatOptions, SourceComment,
};
use oxc_ast::ast::Program;

use crate::context::JsFormatOptions;

/// Context object storing data relevant when formatting an object.
#[derive(Debug, Clone)]
pub struct FormatContext<'a> {
    options: JsFormatOptions,

    source_text: &'a str,

    comments: Comments,
}

impl<'a> FormatContext<'a> {
    pub fn new(program: &'a Program<'a>, options: JsFormatOptions) -> Self {
        Self {
            options,
            source_text: program.source_text,
            comments: Comments::from_oxc_comments(&program.comments),
        }
    }

    /// Returns the formatting options
    pub fn options(&self) -> &JsFormatOptions {
        &self.options
    }

    /// Returns a reference to the program's comments.
    pub fn comments(&self) -> &Comments {
        &self.comments
    }

    /// Returns the formatting options
    pub fn source_text(&self) -> &'a str {
        self.source_text
    }
}
