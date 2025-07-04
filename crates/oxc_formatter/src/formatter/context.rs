use std::cell::RefCell;

use oxc_allocator::Allocator;
use oxc_ast::{
    Comment,
    ast::{FunctionBody, Program},
};
use oxc_span::{GetSpan, SourceType, Span};

use crate::{formatter::FormatElement, generated::ast_nodes::AstNode, options::FormatOptions};

use super::Comments;

/// Context object storing data relevant when formatting an object.
#[derive(Clone)]
pub struct FormatContext<'ast> {
    options: FormatOptions,

    source_text: &'ast str,

    source_type: SourceType,

    comments: Comments<'ast>,

    cached_function_body: Option<(Span, FormatElement<'ast>)>,

    allocator: &'ast Allocator,
}

impl std::fmt::Debug for FormatContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatContext")
            .field("options", &self.options)
            .field("source_text", &self.source_text)
            .field("source_type", &self.source_type)
            .field("comments", &self.comments)
            .field("cached_function_body", &self.cached_function_body)
            .finish()
    }
}

impl<'ast> FormatContext<'ast> {
    pub fn new(
        program: &'ast Program<'ast>,
        allocator: &'ast Allocator,
        options: FormatOptions,
    ) -> Self {
        Self {
            options,
            source_text: program.source_text,
            source_type: program.source_type,
            comments: Comments::new(program.source_text, &program.comments),
            cached_function_body: None,
            allocator,
        }
    }

    /// Returns the formatting options
    pub fn options(&self) -> &FormatOptions {
        &self.options
    }

    /// Returns a reference to the program's comments.
    pub fn comments(&self) -> &Comments<'ast> {
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
    ) -> Option<FormatElement<'ast>> {
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
        formatted: FormatElement<'ast>,
    ) {
        self.cached_function_body = Some((body.span(), formatted));
    }

    pub(crate) fn increment_printed_count(&mut self) {
        self.comments.increment_printed_count();
    }

    pub fn allocator(&self) -> &'ast Allocator {
        self.allocator
    }
}
