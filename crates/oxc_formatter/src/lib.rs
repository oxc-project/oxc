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

mod generated {
    pub mod ast_nodes;
    pub mod format;
    pub mod format_write;
}
mod formatter;
mod options;
mod parentheses;
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
use crate::{
    formatter::FormatContext,
    generated::ast_nodes::{AstNode, AstNodes},
};

pub struct Formatter<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    options: FormatOptions,
}

impl<'a> Formatter<'a> {
    pub fn new(allocator: &'a Allocator, options: FormatOptions) -> Self {
        Self { allocator, source_text: "", options }
    }

    pub fn build(mut self, program: &Program<'a>) -> String {
        let parent = self.allocator.alloc(AstNodes::Dummy());
        let program_node = AstNode::new(program, parent, self.allocator);

        let source_text = program.source_text;
        self.source_text = source_text;
        let context = FormatContext::new(program, self.options);
        let formatted = formatter::format(
            program,
            context,
            formatter::Arguments::new(&[formatter::Argument::new(&program_node)]),
        )
        .unwrap();
        formatted.print().unwrap().into_code()
    }
}
