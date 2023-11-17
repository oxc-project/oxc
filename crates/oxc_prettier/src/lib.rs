//! Prettier
//!
//! A port of <https://github.com/prettier/prettier>

mod comment;
mod doc;
mod format;
mod macros;
mod options;
mod printer;
mod util;

use std::{iter::Peekable, vec};

use doc::Doc;
use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, CommentKind, Trivias};

pub use crate::doc::DocPrinter;
pub use crate::options::{ArrowParens, PrettierOptions, QuoteProps, TrailingComma};
use crate::{format::Format, printer::Printer};

pub struct Prettier<'a> {
    allocator: &'a Allocator,

    source_text: &'a str,

    options: PrettierOptions,

    /// A stack of comments that will be carefully placed in the right places.
    trivias: Peekable<vec::IntoIter<(u32, u32, CommentKind)>>,
}

impl<'a> Prettier<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        trivias: Trivias,
        options: PrettierOptions,
    ) -> Self {
        let trivias = trivias.into_iter().peekable();
        Self { allocator, source_text, options, trivias }
    }

    pub fn build(mut self, program: &Program<'a>) -> String {
        let doc = program.format(&mut self);
        Printer::new(doc, self.source_text, self.options).build()
    }

    pub fn doc(mut self, program: &Program<'a>) -> Doc<'a> {
        program.format(&mut self)
    }
}
