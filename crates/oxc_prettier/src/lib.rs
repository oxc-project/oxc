//! Prettier
//!
//! A port of <https://github.com/prettier/prettier>

mod comment;
mod doc;
mod format;
mod macros;
mod printer;

use std::{
    iter::{Peekable, Rev},
    vec,
};

use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, CommentKind, Trivias};

use crate::{format::Format, printer::Printer};

pub struct PrettierOptions {
    /// Print width (in characters).
    /// Default: 80
    #[allow(unused)]
    print_width: usize,

    /// Print semicolons at the ends of statements.
    /// Default: true
    semi: bool,
}

impl Default for PrettierOptions {
    fn default() -> Self {
        Self { semi: true, print_width: 80 }
    }
}

pub struct Prettier<'a> {
    allocator: &'a Allocator,

    source_text: &'a str,

    options: PrettierOptions,

    /// A stack of comments that will be carefully placed in the right places.
    trivias: Peekable<Rev<vec::IntoIter<(u32, u32, CommentKind)>>>,
}

impl<'a> Prettier<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        trivias: Trivias,
        options: PrettierOptions,
    ) -> Self {
        let trivias = trivias.into_iter().rev().peekable();
        Self { allocator, source_text, options, trivias }
    }

    pub fn build(mut self, program: &Program<'a>) -> String {
        let doc = program.format(&mut self);
        Printer::new(doc, self.options).build()
    }
}
