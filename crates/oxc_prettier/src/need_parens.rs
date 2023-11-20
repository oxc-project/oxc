use oxc_ast::AstKind;

use crate::{array, doc::Doc, ss, Prettier};

impl<'a> Prettier<'a> {
    pub(crate) fn wrap_parens(&self, doc: Doc<'a>, kind: AstKind<'a>) -> Doc<'a> {
        if self.need_parens(kind) {
            array![self, ss!("("), doc, ss!(")")]
        } else {
            doc
        }
    }

    fn need_parens(&self, kind: AstKind<'a>) -> bool {
        match kind {
            // Only statements don't need parentheses.
            kind if kind.is_statement() => return false,
            AstKind::SequenceExpression(_) => {
                let parent = self.parent_kind();

                if matches!(parent, AstKind::Program(_)) {
                    return false;
                }
            }
            AstKind::ObjectExpression(_) => {
                let parent = self.parent_kind();

                if matches!(parent, AstKind::Program(_)) {
                    return true;
                }
            }
            AstKind::AssignmentExpression(_) => {
                let parent = self.parent_kind();

                if matches!(parent, AstKind::ArrowExpression(arrow_expr) if arrow_expr.expression) {
                    return true;
                }
            }
            // NOTE: This is a fallback which should be removed when all code are ported.
            AstKind::ParenthesizedExpression(_) => return true,
            _ => {}
        }

        false
    }
}
