//! Prettier
//!
//! A port of <https://github.com/prettier/prettier>

#![allow(clippy::wildcard_imports)]

mod comment;
mod doc;
mod format;
mod macros;
mod needs_parens;
mod options;
mod printer;

use std::{iter::Peekable, vec};

use doc::DocBuilder;
use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, AstKind, CommentKind, Trivias};
use oxc_syntax::identifier::is_line_terminator;

use crate::{doc::Doc, format::Format, printer::Printer};

pub use crate::options::{ArrowParens, EndOfLine, PrettierOptions, QuoteProps, TrailingComma};

pub struct Prettier<'a> {
    allocator: &'a Allocator,

    source_text: &'a str,

    options: PrettierOptions,

    /// A stack of comments that will be carefully placed in the right places.
    trivias: Peekable<vec::IntoIter<(u32, u32, CommentKind)>>,

    /// The stack of AST Nodes
    /// See <https://github.com/prettier/prettier/blob/main/src/common/ast-path.js>
    nodes: Vec<AstKind<'a>>,
}

impl<'a> DocBuilder<'a> for Prettier<'a> {
    #[inline]
    fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
}

impl<'a> Prettier<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        trivias: Trivias,
        options: PrettierOptions,
    ) -> Self {
        Self {
            allocator,
            source_text,
            options,
            trivias: trivias.into_iter().peekable(),
            nodes: vec![],
        }
    }

    pub fn build(mut self, program: &Program<'a>) -> String {
        let doc = program.format(&mut self);
        Printer::new(doc, self.source_text, self.options, self.allocator).build()
    }

    pub fn doc(mut self, program: &Program<'a>) -> Doc<'a> {
        program.format(&mut self)
    }

    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.nodes.push(kind);
    }

    fn leave_node(&mut self) {
        self.nodes.pop();
    }

    fn current_kind(&self) -> AstKind<'a> {
        self.nodes[self.nodes.len() - 1]
    }

    fn parent_kind(&self) -> AstKind<'a> {
        self.nodes[self.nodes.len() - 2]
    }

    fn parent_parent_kind(&self) -> Option<AstKind<'a>> {
        let len = self.nodes.len();
        (len >= 3).then(|| self.nodes[len - 3])
    }

    fn nth_parent_kind(&self, n: usize) -> Option<AstKind<'a>> {
        let len = self.nodes.len();
        (len > n).then(|| self.nodes[len - n - 1])
    }

    /// A hack for erasing the lifetime requirement.
    #[allow(clippy::unused_self)]
    fn alloc<T>(&self, t: &T) -> &'a T {
        // SAFETY:
        // This should be safe as long as `src` is an reference from the allocator.
        // But honestly, I'm not really sure if this is safe.
        unsafe { std::mem::transmute(t) }
    }

    fn should_print_es5_comma(&self) -> bool {
        self.should_print_comma_impl(false)
    }

    #[allow(unused)]
    fn should_print_all_comma(&self) -> bool {
        self.should_print_comma_impl(true)
    }

    fn should_print_comma_impl(&self, level_all: bool) -> bool {
        let trailing_comma = self.options.trailing_comma;
        trailing_comma.is_all() || (trailing_comma.is_es5() && !level_all)
    }

    fn is_next_line_empty(&self, end: u32) -> bool {
        self.source_text[end as usize..].chars().nth(1).is_some_and(|c| c == '\n')
    }

    #[allow(clippy::cast_possible_truncation)]
    fn skip_newline(&self, start_index: Option<u32>, backwards: bool) -> Option<u32> {
        let start_index = start_index?;
        let c = if backwards {
            self.source_text[..=start_index as usize].chars().next_back()
        } else {
            self.source_text[start_index as usize..].chars().next()
        }?;
        if is_line_terminator(c) {
            let len = c.len_utf8() as u32;
            return Some(if backwards { start_index - len } else { start_index + len });
        }
        Some(start_index)
    }

    fn skip_spaces(&self, start_index: u32, backwards: bool) -> Option<u32> {
        let mut index = start_index;
        if backwards {
            for c in self.source_text[..=start_index as usize].chars().rev() {
                if !matches!(c, ' ' | '\t') {
                    return Some(index);
                }
                index -= 1_u32;
            }
        } else {
            for c in self.source_text[start_index as usize..].chars() {
                if !matches!(c, ' ' | '\t') {
                    return Some(index);
                }
                index += 1_u32;
            }
        }
        None
    }

    fn has_newline(&self, start_index: u32, backwards: bool) -> bool {
        if (backwards && start_index == 0)
            || (!backwards && start_index as usize == self.source_text.len())
        {
            return false;
        }
        let start_index = if backwards { start_index - 1 } else { start_index };
        let idx = self.skip_spaces(start_index, backwards);
        let idx2 = self.skip_newline(idx, backwards);
        idx != idx2
    }

    fn is_previous_line_empty(&self, start_index: u32) -> bool {
        let idx = start_index - 1;
        let idx = self.skip_spaces(idx, true);
        let Some(idx) = self.skip_newline(idx, true) else { return false };
        let idx = self.skip_spaces(idx, true);
        let idx2 = self.skip_newline(idx, true);
        idx != idx2
    }
}
