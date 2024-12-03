use crate::ir::{Doc, IndentIfBreak};

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
        Doc::Str(_) => false,
    }
}
