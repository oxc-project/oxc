use oxc_ast::ast::*;

use crate::{array, doc::Doc, indent, ss, Prettier};

pub(super) fn adjust_clause<'a>(
    p: &Prettier<'a>,
    node: &Statement<'a>,
    clause: Doc<'a>,
    force_space: bool,
) -> Doc<'a> {
    if matches!(node, Statement::EmptyStatement(_)) {
        return ss!(";");
    }

    if matches!(node, Statement::BlockStatement(_)) || force_space {
        return array![p, ss!(" "), clause];
    }

    indent![p, Doc::Line, clause]
}
