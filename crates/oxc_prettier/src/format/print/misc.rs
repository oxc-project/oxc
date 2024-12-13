use oxc_ast::{ast::*, AstKind};
use oxc_span::Span;

use crate::{array, indent, ir::Doc, line, text, Prettier};

pub fn adjust_clause<'a>(
    p: &Prettier<'a>,
    node: &Statement<'a>,
    clause: Doc<'a>,
    force_space: bool,
) -> Doc<'a> {
    if matches!(node, Statement::EmptyStatement(_)) {
        return text!(";");
    }

    if matches!(node, Statement::BlockStatement(_)) || force_space {
        return array!(p, [text!(" "), clause]);
    }

    indent!(p, [line!(), clause])
}

pub fn has_new_line_in_range(text: &str, start: u32, end: u32) -> bool {
    text[(start as usize)..(end as usize)].contains('\n')
}

pub fn in_parentheses(kind: AstKind, text: &str, span: Span) -> bool {
    if matches!(
        kind,
        AstKind::IfStatement(_)
            | AstKind::SwitchStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
    ) {
        return false;
    }

    if span.start == 0 || span.end == u32::try_from(text.len()).unwrap_or_default() {
        return false;
    }
    let text = text.as_bytes();
    for i in (0..span.start as usize).rev() {
        let char = text[i];
        if char.is_ascii_whitespace() {
            continue;
        }
        if char == b'(' {
            break;
        }
        return false;
    }

    for char in text.iter().skip(usize::try_from(span.end).unwrap_or_default()) {
        if char.is_ascii_whitespace() {
            continue;
        }
        return char == &b')';
    }

    false
}
