use oxc_allocator::Vec;

use crate::{doc::Doc, ss, Prettier};

use super::Format;

impl<'a> Prettier<'a> {
    pub(super) fn print_object_properties<F: Format<'a>>(
        &mut self,
        properties: &Vec<'a, F>,
    ) -> Doc<'a> {
        let mut parts = self.vec();

        parts.push(ss!("{"));
        if self.options.bracket_spacing {
            parts.push(Doc::Line);
        } else {
            parts.push(Doc::Softline);
        };

        let len = properties.len();
        properties.iter().map(|prop| prop.format(self)).enumerate().for_each(|(i, prop)| {
            parts.push(prop);
            if i < len - 1 {
                parts.push(Doc::Str(","));
                parts.push(Doc::Line);
            }
        });

        if self.options.bracket_spacing {
            parts.push(Doc::Line);
        } else {
            parts.push(Doc::Softline);
        }
        parts.push(ss!("}"));

        Doc::Group(parts)
    }
}
