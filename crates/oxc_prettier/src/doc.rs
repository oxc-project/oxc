//! Prettier IR
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/commands.md>

use oxc_allocator::{String, Vec};

use crate::Prettier;

pub enum Doc<'a> {
    Str(&'a str),
    // perf: can we use &[Doc] here?
    Array(Vec<'a, Doc<'a>>),
    Indent(Vec<'a, Doc<'a>>),
    Group(Vec<'a, Doc<'a>>),
    /// Specify a line break.
    /// If an expression fits on one line, the line break will be replaced with a space.
    /// Line breaks always indent the next line with the current level of indentation.
    Line,
    Softline,
    Hardline,
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
