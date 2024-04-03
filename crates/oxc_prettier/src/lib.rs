//! Prettier
//!
//! A port of <https://github.com/prettier/prettier>

#![allow(clippy::wildcard_imports)]

mod binaryish;
mod comments;
mod doc;
mod format;
mod macros;
mod needs_parens;
mod options;
mod printer;
mod utils;

use std::{iter::Peekable, vec};

use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, AstKind, CommentKind, Trivias};
use oxc_span::Span;
use oxc_syntax::identifier::is_line_terminator;

use crate::{doc::Doc, doc::DocBuilder, format::Format, printer::Printer};

pub use crate::options::{ArrowParens, EndOfLine, PrettierOptions, QuoteProps, TrailingComma};

type GroupId = u32;
#[derive(Default)]
struct GroupIdBuilder {
    id: GroupId,
}

impl GroupIdBuilder {
    pub fn next_id(&mut self) -> GroupId {
        self.id += 1;
        self.id
    }
}

#[derive(Debug, Default)]
pub struct PrettierArgs {
    expand_first_arg: bool,
    expand_last_arg: bool,
}

pub struct Prettier<'a> {
    allocator: &'a Allocator,

    source_text: &'a str,

    options: PrettierOptions,

    /// A stack of comments that will be carefully placed in the right places.
    trivias: Peekable<vec::IntoIter<(CommentKind, Span)>>,

    /// The stack of AST Nodes
    /// See <https://github.com/prettier/prettier/blob/main/src/common/ast-path.js>
    stack: Vec<AstKind<'a>>,

    group_id_builder: GroupIdBuilder,
    args: PrettierArgs,
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
        trivias: &Trivias,
        options: PrettierOptions,
    ) -> Self {
        Self {
            allocator,
            source_text,
            options,
            trivias: trivias.comments().collect::<Vec<_>>().into_iter().peekable(),
            stack: vec![],
            group_id_builder: GroupIdBuilder::default(),
            args: PrettierArgs::default(),
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
        self.stack.push(kind);
    }

    fn leave_node(&mut self) {
        self.stack.pop();
    }

    fn current_kind(&self) -> AstKind<'a> {
        self.stack[self.stack.len() - 1]
    }

    fn parent_kind(&self) -> AstKind<'a> {
        self.stack[self.stack.len() - 2]
    }

    fn parent_parent_kind(&self) -> Option<AstKind<'a>> {
        let len = self.stack.len();
        (len >= 3).then(|| self.stack[len - 3])
    }

    #[allow(unused)]
    fn nth_parent_kind(&self, n: usize) -> Option<AstKind<'a>> {
        let len = self.stack.len();
        (len > n).then(|| self.stack[len - n - 1])
    }

    /// A hack for erasing the lifetime requirement.
    #[allow(clippy::unused_self)]
    fn alloc<T>(&self, t: &T) -> &'a T {
        // SAFETY:
        // This should be safe as long as `src` is an reference from the allocator.
        // But honestly, I'm not really sure if this is safe.
        #[allow(unsafe_code)]
        unsafe {
            std::mem::transmute(t)
        }
    }

    pub fn semi(&self) -> Option<Doc<'a>> {
        self.options.semi.then(|| Doc::Str(";"))
    }

    pub fn should_print_es5_comma(&self) -> bool {
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

    fn is_next_line_empty(&self, span: Span) -> bool {
        self.is_next_line_empty_after_index(span.end)
    }

    fn is_next_line_empty_after_index(&self, start_index: u32) -> bool {
        let mut old_idx = None;
        let mut idx = Some(start_index);
        while idx != old_idx {
            old_idx = idx;
            idx = self.skip_to_line_end(idx);
            idx = self.skip_inline_comment(idx);
            idx = self.skip_spaces(idx, /* backwards */ false);
        }
        idx = self.skip_trailing_comment(idx);
        idx = self.skip_newline(idx, /* backwards */ false);
        idx.is_some_and(|idx| self.has_newline(idx, /* backwards */ false))
    }

    fn skip_trailing_comment(&self, start_index: Option<u32>) -> Option<u32> {
        let start_index = start_index?;
        let mut chars = self.source_text[start_index as usize..].chars();
        let c = chars.next()?;
        if c != '/' {
            return Some(start_index);
        }
        let c = chars.next()?;
        if c != '/' {
            return Some(start_index);
        }
        self.skip_everything_but_new_line(Some(start_index), /* backwards */ false)
    }

    #[allow(clippy::unused_self)]
    fn skip_inline_comment(&self, start_index: Option<u32>) -> Option<u32> {
        let start_index = start_index?;
        Some(start_index)
    }

    fn skip_to_line_end(&self, start_index: Option<u32>) -> Option<u32> {
        self.skip(start_index, false, |c| matches!(c, ' ' | '\t' | ',' | ';'))
    }

    fn skip_spaces(&self, start_index: Option<u32>, backwards: bool) -> Option<u32> {
        self.skip(start_index, backwards, |c| matches!(c, ' ' | '\t'))
    }

    fn skip_everything_but_new_line(
        &self,
        start_index: Option<u32>,
        backwards: bool,
    ) -> Option<u32> {
        self.skip(start_index, backwards, |c| !is_line_terminator(c))
    }

    fn skip<F>(&self, start_index: Option<u32>, backwards: bool, f: F) -> Option<u32>
    where
        F: Fn(char) -> bool,
    {
        let start_index = start_index?;
        let mut index = start_index;
        if backwards {
            for c in self.source_text[..=start_index as usize].chars().rev() {
                if !f(c) {
                    return Some(index);
                }
                index -= 1_u32;
            }
        } else {
            for c in self.source_text[start_index as usize..].chars() {
                if !f(c) {
                    return Some(index);
                }
                index += 1_u32;
            }
        }
        None
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

    fn has_newline(&self, start_index: u32, backwards: bool) -> bool {
        if (backwards && start_index == 0)
            || (!backwards && start_index as usize == self.source_text.len())
        {
            return false;
        }
        let start_index = if backwards { start_index - 1 } else { start_index };
        let idx = self.skip_spaces(Some(start_index), backwards);
        let idx2 = self.skip_newline(idx, backwards);
        idx != idx2
    }

    fn is_previous_line_empty(&self, start_index: u32) -> bool {
        let idx = start_index - 1;
        let idx = self.skip_spaces(Some(idx), true);
        let idx = self.skip_newline(idx, true);
        let idx = self.skip_spaces(idx, true);
        let idx2 = self.skip_newline(idx, true);
        idx != idx2
    }

    fn next_id(&mut self) -> GroupId {
        self.group_id_builder.next_id()
    }
}
