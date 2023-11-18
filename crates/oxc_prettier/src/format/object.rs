use oxc_allocator::Vec;

use crate::{
    doc::{Doc, Group},
    group, if_break, line, softline, ss, Prettier,
};

use super::Format;

pub(super) fn print_object_properties<'a, F: Format<'a>>(
    p: &mut Prettier<'a>,
    properties: &Vec<'a, F>,
) -> Doc<'a> {
    let left_brace = ss!("{");
    let right_brace = ss!("}");

    let content = if properties.is_empty() {
        group![p, left_brace, softline!(), right_brace]
    } else {
        let mut parts = p.vec();
        parts.push(ss!("{"));

        let mut indent_parts = p.vec();
        indent_parts.push(if p.options.bracket_spacing { line!() } else { softline!() });
        for (i, prop) in properties.iter().enumerate() {
            indent_parts.push(prop.format(p));
            if i < properties.len() - 1 {
                indent_parts.push(Doc::Str(","));
                indent_parts.push(Doc::Line);
            }
        }
        parts.push(group!(p, Doc::Indent(indent_parts)));
        parts.push(if_break!(p, ","));

        if p.options.bracket_spacing {
            parts.push(Doc::Line);
        } else {
            parts.push(Doc::Softline);
        }

        parts.push(ss!("}"));
        Doc::Group(Group { docs: parts, group_id: None })
    };

    content
}
