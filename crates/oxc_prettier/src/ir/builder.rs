use oxc_allocator::{Allocator, Box, IntoIn, String, Vec};

use crate::{
    ir::{Doc, Fill, Group, IfBreak, IndentIfBreak, Line},
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

    fn indent(&self, contents: Vec<'a, Doc<'a>>) -> Doc<'a> {
        Doc::Indent(contents)
    }

    fn array(&self, contents: Vec<'a, Doc<'a>>) -> Doc<'a> {
        Doc::Array(contents)
    }

    fn fill(&self, contents: Vec<'a, Doc<'a>>) -> Doc<'a> {
        Doc::Fill(Fill { contents })
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

    fn indent_if_break(&self, contents: Vec<'a, Doc<'a>>, group_id: Option<GroupId>) -> Doc<'a> {
        Doc::IndentIfBreak(IndentIfBreak { contents, group_id })
    }

    fn group(&self, contents: Doc<'a>) -> Doc<'a> {
        Doc::Group(Group {
            contents: self.vec_single(contents),
            should_break: false,
            expanded_states: None,
            group_id: None,
        })
    }
    fn group_with_opts(
        &self,
        contents: Doc<'a>,
        should_break: bool,
        group_id: Option<GroupId>,
    ) -> Doc<'a> {
        Doc::Group(Group {
            contents: self.vec_single(contents),
            should_break,
            expanded_states: None,
            group_id,
        })
    }

    fn conditional_group(
        &self,
        contents: Doc<'a>,
        alternatives: std::vec::Vec<Doc<'a>>,
        group_id: Option<GroupId>,
    ) -> Doc<'a> {
        let contents = self.vec_single(contents);
        let expanded_states = Vec::from_iter_in(alternatives, self.allocator());
        Doc::Group(Group {
            contents,
            should_break: false,
            expanded_states: Some(expanded_states),
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
