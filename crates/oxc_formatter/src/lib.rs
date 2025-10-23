#![allow(
    unused,
    clippy::inline_always,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_ref_mut,
    clippy::todo,
    clippy::unused_self,
    clippy::enum_variant_names,
    clippy::struct_field_names
)] // FIXME: all these needs to be fixed.

mod ast_nodes;
mod formatter;
mod ir_transform;
mod options;
mod parentheses;
mod service;
mod utils;
mod write;

use std::{
    cell::{Cell, UnsafeCell},
    fmt::{self, Display},
    marker::PhantomData,
    mem::{self, transmute},
    vec::IntoIter,
};

use oxc_allocator::{Address, Allocator, GetAddress};
use oxc_ast::{AstKind, ast::*};
use rustc_hash::{FxHashMap, FxHashSet};
use write::FormatWrite;

pub use crate::options::*;
pub use crate::service::{
    oxfmtrc::Oxfmtrc,
    source_type::{enable_jsx_source_type, get_supported_source_type},
};
use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{FormatContext, Formatted, format_element::document::Document},
    ir_transform::SortImportsTransform,
};

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

    /// Formats the given AST `Program` and returns the IR before printing.
    pub fn doc(mut self, program: &'a Program<'a>) -> Document<'a> {
        let formatted = self.format(program);
        formatted.into_document()
    }

    /// Formats the given AST `Program` and returns the formatted string.
    pub fn build(mut self, program: &Program<'a>) -> String {
        let formatted = self.format(program);
        formatted.print().unwrap().into_code()
    }

    pub fn format(mut self, program: &'a Program<'a>) -> Formatted<'a> {
        let parent = self.allocator.alloc(AstNodes::Dummy());
        let program_node = AstNode::new(program, parent, self.allocator);

        let source_text = program.source_text;
        self.source_text = source_text;

        let experimental_sort_imports = self.options.experimental_sort_imports;

        let context = FormatContext::new(
            program.source_text,
            program.source_type,
            &program.comments,
            self.allocator,
            self.options,
        );
        let mut formatted = formatter::format(
            context,
            formatter::Arguments::new(&[formatter::Argument::new(&program_node)]),
        )
        .unwrap();

        // Basic formatting and `document.propagate_expand()` are already done here.
        // Now apply additional transforms if enabled.
        if let Some(sort_imports_options) = experimental_sort_imports {
            let sort_imports = SortImportsTransform::new(sort_imports_options);
            formatted.apply_transform(|doc| sort_imports.transform(doc));
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
