#![allow(clippy::inline_always, clippy::missing_panics_doc)] // FIXME: all these needs to be fixed.

mod ast_nodes;
#[cfg(feature = "detect_code_removal")]
mod detect_code_removal;
mod embedded_formatter;
mod formatter;
mod ir_transform;
mod options;
mod parentheses;
mod service;
mod utils;
mod write;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;

pub use crate::embedded_formatter::{EmbeddedFormatter, EmbeddedFormatterCallback};
pub use crate::ir_transform::options::*;
pub use crate::options::*;
pub use crate::service::{oxfmtrc::Oxfmtrc, parse_utils::*};
use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{FormatContext, Formatted},
    ir_transform::SortImportsTransform,
};
#[cfg(feature = "detect_code_removal")]
pub use detect_code_removal::detect_code_removal;

use self::formatter::prelude::tag::Label;

pub struct Formatter<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    options: FormatOptions,
}

impl<'a> Formatter<'a> {
    pub fn new(allocator: &'a Allocator, options: FormatOptions) -> Self {
        Self { allocator, source_text: "", options }
    }

    /// Formats the given AST `Program` and returns the formatted string.
    pub fn build(self, program: &Program<'a>) -> String {
        let formatted = self.format(program);
        formatted.print().unwrap().into_code()
    }

    #[inline]
    pub fn format(self, program: &'a Program<'a>) -> Formatted<'a> {
        self.format_impl(program, None)
    }

    #[inline]
    pub fn format_with_embedded(
        self,
        program: &'a Program<'a>,
        embedded_formatter: EmbeddedFormatter,
    ) -> Formatted<'a> {
        self.format_impl(program, Some(embedded_formatter))
    }

    pub fn format_impl(
        mut self,
        program: &'a Program<'a>,
        embedded_formatter: Option<EmbeddedFormatter>,
    ) -> Formatted<'a> {
        let parent = self.allocator.alloc(AstNodes::Dummy());
        let program_node = AstNode::new(program, parent, self.allocator);

        let source_text = program.source_text;
        self.source_text = source_text;

        let experimental_sort_imports = self.options.experimental_sort_imports.clone();

        let context = FormatContext::new(
            program.source_text,
            program.source_type,
            &program.comments,
            self.allocator,
            self.options,
            embedded_formatter,
        );

        let mut formatted = formatter::format(
            context,
            formatter::Arguments::new(&[formatter::Argument::new(&program_node)]),
        );

        // Basic formatting and `document.propagate_expand()` are already done here.
        // Now apply additional transforms if enabled.
        if let Some(sort_imports_options) = experimental_sort_imports {
            let sort_imports = SortImportsTransform::new(sort_imports_options);
            formatted.apply_transform(|doc| sort_imports.transform(doc, self.allocator));
        }

        formatted
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum JsLabels {
    MemberChain,
    /// For `ir_transform/sort_imports`
    ImportDeclaration,
}

impl Label for JsLabels {
    fn id(&self) -> u64 {
        *self as u64
    }

    fn debug_name(&self) -> &'static str {
        match self {
            Self::MemberChain => "MemberChain",
            Self::ImportDeclaration => "ImportDeclaration",
        }
    }
}
