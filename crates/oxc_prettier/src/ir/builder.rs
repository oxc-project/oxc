use oxc_allocator::{Allocator, Box, String, Vec};

use crate::{
    ir::{Doc, Fill, IfBreak, Line},
    p_vec, GroupId,
};

#[derive(Clone, Copy)]
pub enum Separator {
    #[allow(unused)]
    Softline,
    Hardline,
    CommaLine, // [",", line]
}

pub trait DocBuilder<'a> {
    fn allocator(&self) -> &'a Allocator;

    #[inline]
    fn vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(self.allocator())
    }
    fn vec_single<T>(&self, value: T) -> Vec<'a, T> {
        let mut vec = Vec::with_capacity_in(1, self.allocator());
        vec.push(value);
        vec
    }

    #[inline]
    fn string(&self, s: &str) -> &'a str {
        String::from_str_in(s, self.allocator()).into_bump_str()
    }

    #[inline]
    fn boxed(&self, doc: Doc<'a>) -> Box<'a, Doc<'a>> {
        Box::new_in(doc, self.allocator())
    }

    fn text(&self, s: &'a str) -> Doc<'a> {
        Doc::Str(s)
    }
    fn space(&self) -> Doc<'a> {
        Doc::Str(" ")
    }

    fn line(&self) -> Doc<'a> {
        Doc::Line(Line::default())
    }
    /// Specify a line break.
    /// The difference from line is that if the expression fits on one line, it will be replaced with nothing.
    fn softline(&self) -> Doc<'a> {
        Doc::Line(Line { soft: true, ..Line::default() })
    }
    /// Specify a line break that is **always** included in the output,
    /// no matter if the expression fits on one line or not.
    fn hardline(&self) -> [Doc<'a>; 2] {
        let hardline = Doc::Line(Line { hard: true, ..Line::default() });
        [hardline, Doc::BreakParent]
    }

    fn indent(&self, items: Vec<'a, Doc<'a>>) -> Doc<'a> {
        Doc::Indent(items)
    }

    fn array(&self, items: Vec<'a, Doc<'a>>) -> Doc<'a> {
        Doc::Array(items)
    }

    fn fill(&self, parts: Vec<'a, Doc<'a>>) -> Doc<'a> {
        Doc::Fill(Fill { parts })
    }

    fn if_break(
        &self,
        break_contents: Doc<'a>,
        flat_contents: Doc<'a>,
        group_id: Option<GroupId>,
    ) -> Doc<'a> {
        Doc::IfBreak(IfBreak {
            break_contents: self.boxed(break_contents),
            flat_contents: self.boxed(flat_contents),
            group_id,
        })
    }

    // TODO: Just use `Doc` instead of `Separator`...?
    fn join(&self, separator: Separator, docs: std::vec::Vec<Doc<'a>>) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        for (i, doc) in docs.into_iter().enumerate() {
            if i != 0 {
                match separator {
                    Separator::Softline => parts.push(self.softline()),
                    Separator::Hardline => parts.extend(self.hardline()),
                    Separator::CommaLine => {
                        parts.push(self.array(p_vec!(self, self.text(","), self.line())))
                    }
                }
            }
            parts.push(doc);
        }
        parts
    }
}
