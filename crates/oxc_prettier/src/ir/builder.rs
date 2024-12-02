use oxc_allocator::{Allocator, Box, String, Vec};

use crate::{
    ir::{Doc, Line},
    p_vec,
};

pub fn text<'a>(s: &'a str) -> Doc<'a> {
    Doc::Str(s)
}
pub fn space<'a>() -> Doc<'a> {
    Doc::Str(" ")
}

pub fn line<'a>() -> Doc<'a> {
    Doc::Line(Line::default())
}
/// Specify a line break.
/// The difference from line is that if the expression fits on one line, it will be replaced with nothing.
pub fn softline<'a>() -> Doc<'a> {
    Doc::Line(Line { soft: true, ..Line::default() })
}
/// Specify a line break that is **always** included in the output,
/// no matter if the expression fits on one line or not.
pub fn hardline<'a>() -> [Doc<'a>; 2] {
    let hardline = Doc::Line(Line { hard: true, ..Line::default() });
    [hardline, Doc::BreakParent]
}

pub fn indent<'a>(items: Vec<'a, Doc<'a>>) -> Doc<'a> {
    Doc::Indent(items)
}

pub fn array<'a>(items: Vec<'a, Doc<'a>>) -> Doc<'a> {
    Doc::Array(items)
}

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

    // TODO: Just use `Doc` instead of `Separator`...?
    fn join(&self, separator: Separator, docs: std::vec::Vec<Doc<'a>>) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        for (i, doc) in docs.into_iter().enumerate() {
            if i != 0 {
                match separator {
                    Separator::Softline => parts.push(softline()),
                    Separator::Hardline => parts.extend(hardline()),
                    Separator::CommaLine => parts.push(array(p_vec!(self, text(","), line()))),
                }
            }
            parts.push(doc);
        }
        parts
    }
}
