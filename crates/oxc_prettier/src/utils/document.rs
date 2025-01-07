use crate::{
    dynamic_text,
    ir::{Doc, IndentIfBreak, JoinSeparator},
    join, Prettier,
};

pub fn will_break(doc: &mut Doc<'_>) -> bool {
    let check_array =
        |arr: &mut oxc_allocator::Vec<'_, Doc<'_>>| arr.iter_mut().rev().any(|doc| will_break(doc));

    match doc {
        Doc::BreakParent => true,
        Doc::Group(group) => {
            if group.should_break {
                return true;
            }
            if let Some(expanded_states) = &mut group.expanded_states {
                if expanded_states.iter_mut().rev().any(will_break) {
                    return true;
                }
            }
            check_array(&mut group.contents)
        }
        Doc::IfBreak(d) => will_break(&mut d.break_contents),
        Doc::Array(arr) | Doc::Indent(arr) | Doc::LineSuffix(arr) => check_array(arr),
        Doc::IndentIfBreak(IndentIfBreak { contents, .. }) => will_break(contents),
        Doc::Fill(doc) => check_array(&mut doc.contents),
        Doc::Line(doc) => doc.hard,
        Doc::Str(_) | Doc::LineSuffixBoundary => false,
    }
}

/// Handle line continuation.
/// This does not recursively handle the doc, expects single `Doc::Str`.
pub fn replace_end_of_line<'a>(
    p: &Prettier<'a>,
    doc: Doc<'a>,
    replacement: JoinSeparator,
) -> Doc<'a> {
    let Doc::Str(text) = doc else {
        return doc;
    };

    let lines = text.split('\n').map(|line| dynamic_text!(p, line)).collect::<Vec<_>>();
    join!(p, replacement, lines)
}
