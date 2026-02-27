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
use oxc_ast::Comment;
use oxc_ast::ast::*;
use oxc_span::SourceType;

pub use crate::ast_nodes::{AstNode, AstNodes};
pub use crate::external_formatter::{
    EmbeddedDocFormatterCallback, EmbeddedFormatterCallback, ExternalCallbacks, TailwindCallback,
};
pub use crate::formatter::format_element::tag::{
    Align, Condition, DedentMode, Group, GroupMode, Tag,
};
pub use crate::formatter::format_element::{
    BestFittingElement, FormatElement, LineMode, PrintMode, TextWidth,
};
pub use crate::formatter::{Format, Formatted};
pub use crate::formatter::{GroupId, UniqueGroupIdBuilder};
pub use crate::ir_transform::options::*;
pub use crate::options::*;
pub use crate::print::{FormatVueBindingParams, FormatVueScriptGeneric};
pub use crate::service::*;
use crate::{formatter::FormatContext, ir_transform::SortImportsTransform};
#[cfg(feature = "detect_code_removal")]
pub use detect_code_removal::detect_code_removal;
pub use oxc_formatter_core as formatter_core;
pub use oxc_formatter_core::{
    Format as CoreFormat, FormatContext as CoreFormatContext, Formatter as CoreFormatter,
};

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
        let program_node = AstNode::new(program, AstNodes::Dummy(), self.allocator);

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
        if let Some(sort_imports_options) = &formatted.context().options().sort_imports
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

    /// Formats an arbitrary value that implements `Format` and returns the `Formatted` IR.
    ///
    /// Unlike `format()` which requires a full `Program`,
    /// this method accepts any value that implements `Format`.
    /// This enables fragment formatting for embedded contexts like:
    /// - Vue: `v-for`, `v-slot`, and `<script generic="...">`
    /// - etc...
    ///
    /// `SortImportsTransform` is skipped since it only applies to whole `Program` formatting.
    pub fn format_node<F: formatter::Format<'a>>(
        self,
        node: &F,
        source_text: &'a str,
        source_type: SourceType,
        comments: &'a [Comment],
        external_callbacks: Option<ExternalCallbacks>,
    ) -> Formatted<'a> {
        let context = FormatContext::new(
            source_text,
            source_type,
            comments,
            self.allocator,
            self.options,
            external_callbacks,
        );
        formatter::format(context, formatter::Arguments::new(&[formatter::Argument::new(node)]))
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
