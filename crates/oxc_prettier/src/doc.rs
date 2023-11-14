//! Prettier IR
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/commands.md>

use oxc_allocator::{String, Vec};

use crate::Prettier;

#[derive(Debug)]
pub enum Doc<'a> {
    Str(&'a str),
    // perf: can we use &[Doc] here?
    Array(Vec<'a, Doc<'a>>),
    /// Increase the level of indentation.
    Indent(Vec<'a, Doc<'a>>),
    /// Mark a group of items which the printer should try to fit on one line.
    /// This is the basic command to tell the printer when to break.
    /// Groups are usually nested, and the printer will try to fit everything on one line,
    /// but if it doesn't fit it will break the outermost group first and try again.
    /// It will continue breaking groups until everything fits (or there are no more groups to break).
    Group(Vec<'a, Doc<'a>>),
    /// Specify a line break.
    /// If an expression fits on one line, the line break will be replaced with a space.
    /// Line breaks always indent the next line with the current level of indentation.
    Line,
    /// Specify a line break.
    /// The difference from line is that if the expression fits on one line, it will be replaced with nothing.
    Softline,
    /// Specify a line break that is **always** included in the output,
    /// no matter if the expression fits on one line or not.
    Hardline,
    /// Print something if the current `group` or the current element of `fill` breaks and something else if it doesn't.
    IfBreak {
        break_contents: Vec<'a, Doc<'a>>,
        flat_contents: Vec<'a, Doc<'a>>,
    },
}

impl<'a> Doc<'a> {
    #[must_use]
    pub fn if_break(break_contents: Vec<'a, Doc<'a>>, flat_contents: Vec<'a, Doc<'a>>) -> Self {
        Doc::IfBreak { break_contents, flat_contents }
    }
}

/// Doc Builder
impl<'a> Prettier<'a> {
    #[inline]
    pub fn vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    #[inline]
    pub fn str(&self, s: &str) -> Doc<'a> {
        Doc::Str(String::from_str_in(s, self.allocator).into_bump_str())
    }
}
