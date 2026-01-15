#![allow(clippy::inline_always, clippy::missing_panics_doc)] // FIXME: all these needs to be fixed.

mod ast_nodes;
#[cfg(feature = "detect_code_removal")]
mod detect_code_removal;
mod external_formatter;
mod formatter;
mod ir_transform;
mod options;
mod parentheses;
mod print;
mod service;
mod utils;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;

pub use crate::external_formatter::{
    EmbeddedFormatterCallback, ExternalCallbacks, TailwindCallback,
};
pub use crate::ir_transform::options::*;
pub use crate::options::*;
pub use crate::service::*;
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
    options: FormatOptions,
}

impl<'a> Formatter<'a> {
    pub fn new(allocator: &'a Allocator, options: FormatOptions) -> Self {
        Self { allocator, options }
    }

    /// Formats the given AST `Program` and returns the formatted string.
    pub fn build(self, program: &Program<'a>) -> String {
        let formatted = self.format(program);
        formatted.print().unwrap().into_code()
    }

    #[inline]
    pub fn format(self, program: &'a Program<'a>) -> Formatted<'a> {
        self.format_with_external_callbacks(program, None)
    }

    #[inline]
    pub fn format_with_external_callbacks(
        self,
        program: &'a Program<'a>,
        external_callbacks: Option<ExternalCallbacks>,
    ) -> Formatted<'a> {
        let parent = self.allocator.alloc(AstNodes::Dummy());
        let program_node = AstNode::new(program, parent, self.allocator);

        let context = FormatContext::new(
            program.source_text,
            program.source_type,
            &program.comments,
            self.allocator,
            self.options,
            external_callbacks,
        );

        let mut formatted = formatter::format(
            context,
            formatter::Arguments::new(&[formatter::Argument::new(&program_node)]),
        );

        // Basic formatting and `document.propagate_expand()` are already done here.
        // Now apply additional transforms if enabled.
        if let Some(sort_imports_options) = &formatted.context().options().experimental_sort_imports
            && let Some(transformed_elements) = SortImportsTransform::transform(
                formatted.document(),
                sort_imports_options,
                self.allocator,
            )
        {
            formatted.document_mut().replace_elements(transformed_elements);
        }

        formatted
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum JsLabels {
    MemberChain,
    /// For `ir_transform/sort_imports`
    ImportDeclaration,
    /// For `ir_transform/sort_imports`
    /// Marks `alignable_comment` (Block comment where each line starts with `*`)
    /// to distinguish from other text content like template literals that may contain `/*`.
    AlignableBlockComment,
}

impl Label for JsLabels {
    fn id(&self) -> u64 {
        *self as u64
    }

    fn debug_name(&self) -> &'static str {
        match self {
            Self::MemberChain => "MemberChain",
            Self::ImportDeclaration => "ImportDeclaration",
            Self::AlignableBlockComment => "AlignableBlockComment",
        }
    }
}
