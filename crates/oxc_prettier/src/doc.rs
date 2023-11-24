//! Prettier IR
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/commands.md>

use oxc_allocator::{Allocator, Box, String, Vec};
use std::fmt;

use crate::{array, line, ss};

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
    /// This is used to implement trailing comments.
    /// It's not practical to constantly check where the line ends to avoid accidentally printing some code at the end of a comment.
    /// `lineSuffix` buffers docs passed to it and flushes them before any new line.
    LineSuffix(Vec<'a, Doc<'a>>),
    /// Print something if the current `group` or the current element of `fill` breaks and something else if it doesn't.
    IfBreak(Box<'a, Doc<'a>>),
    /// This is an alternative type of group which behaves like text layout:
    /// it's going to add a break whenever the next element doesn't fit in the line anymore.
    /// The difference with `group` is that it's not going to break all the separators, just the ones that are at the end of lines.
    Fill(Fill<'a>),
    /// Include this anywhere to force all parent groups to break.
    BreakParent,
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

#[derive(Debug)]
pub struct Fill<'a> {
    pub parts: Vec<'a, Doc<'a>>,
}

impl<'a> Fill<'a> {
    pub fn new(docs: Vec<'a, Doc<'a>>) -> Self {
        Self { parts: docs }
    }
    pub fn drain_out_pair(&mut self) -> (Option<Doc<'a>>, Option<Doc<'a>>) {
        let content = if self.parts.len() > 0 { Some(self.parts.remove(0)) } else { None };
        let whitespace = if self.parts.len() > 0 { Some(self.parts.remove(0)) } else { None };
        (content, whitespace)
    }
    pub fn dequeue(&mut self) -> Option<Doc<'a>> {
        if self.parts.len() > 0 {
            Some(self.parts.remove(0))
        } else {
            None
        }
    }
    pub fn enqueue(&mut self, doc: Doc<'a>) {
        self.parts.insert(0, doc);
    }
    pub fn parts(&self) -> &[Doc<'a>] {
        &self.parts
    }
    pub fn take_parts(self) -> Vec<'a, Doc<'a>> {
        self.parts
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
pub trait DocBuilder<'a> {
    fn allocator(&self) -> &'a Allocator;
    #[inline]
    fn vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator())
    }

    #[inline]
    fn str(&self, s: &str) -> Doc<'a> {
        Doc::Str(String::from_str_in(s, self.allocator()).into_bump_str())
    }

    #[inline]
    fn boxed(&self, doc: Doc<'a>) -> Box<'a, Doc<'a>> {
        Box(self.allocator().alloc(doc))
    }

    #[allow(unused)]
    fn join(&self, separator: Separator, docs: std::vec::Vec<Doc<'a>>) -> Vec<'a, Doc<'a>> {
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
        Doc::Fill(fill) => {
            string.push_str("fill([\n");
            let parts = fill.parts();
            for (idx, doc) in parts.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != parts.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("])");
        }
        Doc::LineSuffix(docs) => {
            string.push_str("lineSuffix(");
            for (idx, doc) in docs.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != docs.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push(')');
        }
        Doc::BreakParent => {
            string.push_str("BreakParent");
        }
    }

    string
}
