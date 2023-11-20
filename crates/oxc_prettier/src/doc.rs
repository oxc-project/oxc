//! Prettier IR
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/commands.md>

use oxc_allocator::{Box, String, Vec};
use std::fmt;

use crate::{array, line, ss, Prettier};

#[derive(Debug)]
pub enum Doc<'a> {
    Str(&'a str),
    // perf: can we use &[Doc] here?
    Array(Vec<'a, Doc<'a>>),
    /// Increase the level of indentation.
    Indent(Vec<'a, Doc<'a>>),
    IndentIfBreak(Vec<'a, Doc<'a>>),
    /// Mark a group of items which the printer should try to fit on one line.
    /// This is the basic command to tell the printer when to break.
    /// Groups are usually nested, and the printer will try to fit everything on one line,
    /// but if it doesn't fit it will break the outermost group first and try again.
    /// It will continue breaking groups until everything fits (or there are no more groups to break).
    Group(Group<'a>),
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
    IfBreak(Box<'a, Doc<'a>>),
}

#[derive(Debug)]
pub struct Group<'a> {
    pub contents: Vec<'a, Doc<'a>>,
    pub should_break: bool,
}

impl<'a> Group<'a> {
    pub fn new(contents: Vec<'a, Doc<'a>>, should_break: bool) -> Self {
        Self { contents, should_break }
    }
}

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum Separator {
    Softline,
    Hardline,
    CommaLine, // [",", line]
}

/// Doc Builder
impl<'a> Prettier<'a> {
    #[inline]
    pub(crate) fn vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    #[inline]
    pub(crate) fn str(&self, s: &str) -> Doc<'a> {
        Doc::Str(String::from_str_in(s, self.allocator).into_bump_str())
    }

    #[inline]
    pub(crate) fn boxed(&self, doc: Doc<'a>) -> Box<'a, Doc<'a>> {
        Box(self.allocator.alloc(doc))
    }

    #[allow(unused)]
    pub(crate) fn join(
        &self,
        separator: Separator,
        docs: std::vec::Vec<Doc<'a>>,
    ) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        for (i, doc) in docs.into_iter().enumerate() {
            if i != 0 {
                parts.push(match separator {
                    Separator::Softline => Doc::Softline,
                    Separator::Hardline => Doc::Hardline,
                    Separator::CommaLine => array![self, ss!(","), line!()],
                });
            }
            parts.push(doc);
        }
        parts
    }
}

impl<'a> fmt::Display for Doc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{})", print_doc_to_debug(self))
    }
}

// https://github.com/prettier/prettier/blob/main/src/document/debug.js
fn print_doc_to_debug(doc: &Doc<'_>) -> std::string::String {
    use std::string::String;
    let mut string = String::new();
    match doc {
        Doc::Str(s) => {
            string.push('"');
            string.push_str(s);
            string.push('"');
        }
        Doc::Array(docs) => {
            string.push_str("[\n");
            for (idx, doc) in docs.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != docs.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("]\n");
        }
        Doc::Indent(contents) => {
            string.push_str("indent([");
            for (idx, doc) in contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != contents.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("])");
        }
        Doc::IndentIfBreak(contents) => {
            string.push_str("indentIfBreak(");
            string.push_str("[\n");
            for (idx, doc) in contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != contents.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("]) \n");
        }
        Doc::Group(group) => {
            string.push_str("group([\n");
            for (idx, doc) in group.contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != group.contents.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("], { shouldBreak: ");
            string.push_str(&group.should_break.to_string());
            string.push_str(" })");
        }
        Doc::Line => {
            string.push_str("line");
        }
        Doc::Softline => {
            string.push_str("softline");
        }
        Doc::Hardline => {
            string.push_str("hardline");
        }
        Doc::IfBreak(break_contents) => {
            string.push_str("ifBreak(");
            string.push_str(&print_doc_to_debug(break_contents));
            string.push(')');
        }
    }

    string
}
