use oxc_ast::ast::{ArrayExpressionElement, Expression, Program, Statement};

use crate::{utils::Value, LintContext};

pub trait ListenerMap {
    fn report_effects(&self, _ctx: &LintContext) {}
    fn report_effects_when_assigned(&self, _ctx: &LintContext) {}
    fn report_effects_when_called(&self, _ctx: &LintContext) {}
    fn report_effects_when_mutated(&self, _ctx: &LintContext) {}
    fn get_value_and_report_effects(&self, _ctx: &LintContext) -> Option<Value> {
        None
    }
}

impl<'a> ListenerMap for Program<'a> {
    fn report_effects(&self, ctx: &LintContext) {
        self.body.iter().for_each(|stmt| stmt.report_effects(ctx));
    }
}

impl<'a> ListenerMap for Statement<'a> {
    fn report_effects(&self, ctx: &LintContext) {
        if let Self::ExpressionStatement(expr_stmt) = self {
            expr_stmt.expression.report_effects(ctx);
        }
    }
}

impl<'a> ListenerMap for Expression<'a> {
    fn report_effects(&self, ctx: &LintContext) {
        #[allow(clippy::single_match)]
        match self {
            Self::ArrayExpression(array_expr) => {
                array_expr.elements.iter().for_each(|el| el.report_effects(ctx));
            }
            _ => {}
        }
    }
}

impl<'a> ListenerMap for ArrayExpressionElement<'a> {
    fn report_effects(&self, ctx: &LintContext) {
        match self {
            Self::Expression(expr) => expr.report_effects(ctx),
            Self::SpreadElement(spreed) => {
                spreed.argument.report_effects(ctx);
            }
            Self::Elision(_) => {}
        }
    }
}
