//! compound expressions (e.g. sequence exprs, ternaries)

use oxc_ast::ast::{ConditionalExpression, SequenceExpression};

use crate::{Eval, EvalContext, EvalResult, Value};

// 13.16.1 Comma Operator - Runtime Semantics: Evaluation
// https://262.ecma-international.org/15.0/index.html#sec-comma-operator-runtime-semantics-evaluation
impl<'a> Eval<'a> for SequenceExpression<'a> {
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        // 1. Let lref be ? Evaluation of Expression.
        // 2. Perform ? GetValue(lref).
        // 3. Let rref be ? Evaluation of AssignmentExpression.
        // 4. Return ? GetValue(rref).

        let len = self.expressions.len();
        if len == 0 {
            // I don't think this should ever happen, but better safe than sorry
            return Ok(Value::Undefined);
        }

        let last_index = self.expressions.len() - 1;
        for (i, expr) in self.expressions.iter().enumerate() {
            if i == last_index {
                break;
            }
            let _ = expr.eval(ctx)?;
            // NOTE: ReferenceRecord isn't implemented yet, so there's no GetValue
        }

        self.expressions[last_index].eval(ctx)
    }
}

impl<'a> Eval<'a> for ConditionalExpression<'a> {
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        // 1. Let lref be ? Evaluation of ShortCircuitExpression.
        // 2. Let lval be ToBoolean(? GetValue(lref)).
        let lval = self.test.eval(ctx)?.to_boolean();

        // 3. If lval is true, then
        if lval {
            // a. Let trueRef be ? Evaluation of the first AssignmentExpression.
            // b. Return ? GetValue(trueRef).
            self.consequent.eval(ctx)
        } else {
            // a. Let falseRef be ? Evaluation of the second AssignmentExpression.
            // b. Return ? GetValue(falseRef).
            self.alternate.eval(ctx)
        }
    }
}
