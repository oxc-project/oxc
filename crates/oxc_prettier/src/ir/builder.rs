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
pub fn softline<'a>() -> Doc<'a> {
    Doc::Line(Line::softline())
}
pub fn hardline<'a>() -> [Doc<'a>; 2] {
    [Doc::Line(Line::hardline()), Doc::BreakParent]
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

    fn join(&self, separator: Separator, docs: std::vec::Vec<Doc<'a>>) -> Vec<'a, Doc<'a>> {
        let mut parts = self.vec();
        for (i, doc) in docs.into_iter().enumerate() {
            if i != 0 {
                parts.push(match separator {
                    Separator::Softline => Doc::Line(Line::softline()),
                    Separator::Hardline => Doc::Line(Line::hardline()),
                    Separator::CommaLine => array(p_vec!(self, text(","), line())),
                });
            }
            parts.push(doc);
        }
        parts
    }
}
