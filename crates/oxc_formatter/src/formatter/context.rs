use oxc_ast::ast::{FunctionBody, Program};
use oxc_span::{GetSpan, SourceType, Span};

use crate::{formatter::FormatElement, generated::ast_nodes::AstNode, options::FormatOptions};

use super::Comments;

/// Context object storing data relevant when formatting an object.
#[derive(Debug, Clone)]
pub struct FormatContext<'ast> {
    options: FormatOptions,

    source_text: &'ast str,

    source_type: SourceType,

    comments: Comments,

    cached_function_body: Option<(Span, FormatElement)>,
}

impl<'ast> FormatContext<'ast> {
    pub fn new(program: &'ast Program<'ast>, options: FormatOptions) -> Self {
        Self {
            options,
            source_text: program.source_text,
            source_type: program.source_type,
            comments: Comments::from_oxc_comments(program),
            cached_function_body: None,
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

    /// Returns the source type
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    /// Returns the formatted content for the passed function body if it is cached or `None` if the currently
    /// cached content belongs to another function body or the cache is empty.
    ///
    /// See [JsFormatContext::cached_function_body] for more in depth documentation.
    pub(crate) fn get_cached_function_body(
        &self,
        body: &AstNode<'ast, FunctionBody<'ast>>,
    ) -> Option<FormatElement> {
        self.cached_function_body.as_ref().and_then(|(expected_body_span, formatted)| {
            if *expected_body_span == body.span() { Some(formatted.clone()) } else { None }
        })
    }

    /// Sets the currently cached formatted function body.
    ///
    /// See [JsFormatContext::cached_function_body] for more in depth documentation.
    pub(crate) fn set_cached_function_body(
        &mut self,
        body: &AstNode<'ast, FunctionBody<'ast>>,
        formatted: FormatElement,
    ) {
        self.cached_function_body = Some((body.span(), formatted));
    }
}
