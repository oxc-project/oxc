use oxc_allocator::Vec;

use crate::{doc::Doc, group, if_break, ss, Prettier};

use super::Format;

impl<'a> Prettier<'a> {
    pub(super) fn print_object_properties<F: Format<'a>>(
        &mut self,
        properties: &Vec<'a, F>,
    ) -> Doc<'a> {
        let mut parts = self.vec();
        parts.push(ss!("{"));

        let mut indent_parts = self.vec();

        if self.options.bracket_spacing {
            indent_parts.push(Doc::Line);
        } else {
            indent_parts.push(Doc::Softline);
        };

        let len = properties.len();
        properties.iter().map(|prop| prop.format(self)).enumerate().for_each(|(i, prop)| {
            indent_parts.push(prop);
            if i < len - 1 {
                indent_parts.push(Doc::Str(","));
                indent_parts.push(Doc::Line);
            }
        });

        parts.push(group!(self, Doc::Indent(indent_parts)));
        parts.push(if_break!(self, ","));

        if self.options.bracket_spacing {
            parts.push(Doc::Line);
        } else {
            parts.push(Doc::Softline);
        }

        parts.push(ss!("}"));

        Doc::Group(parts)
    }
}
