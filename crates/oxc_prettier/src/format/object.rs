use oxc_allocator::Vec;

use crate::{doc::Doc, group, if_break, ss, Prettier};

use super::Format;

pub(super) fn print_object_properties<'a, F: Format<'a>>(
    p: &mut Prettier<'a>,
    properties: &Vec<'a, F>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(ss!("{"));

    let mut indent_parts = p.vec();

    if p.options.bracket_spacing {
        indent_parts.push(Doc::Line);
    } else {
        indent_parts.push(Doc::Softline);
    };

    let len = properties.len();
    properties.iter().map(|prop| prop.format(p)).enumerate().for_each(|(i, prop)| {
        indent_parts.push(prop);
        if i < len - 1 {
            indent_parts.push(Doc::Str(","));
            indent_parts.push(Doc::Line);
        }
    });

    parts.push(group!(p, Doc::Indent(indent_parts)));
    parts.push(if_break!(p, ","));

    if p.options.bracket_spacing {
        parts.push(Doc::Line);
    } else {
        parts.push(Doc::Softline);
    }

    parts.push(ss!("}"));

    Doc::Group(parts)
}
