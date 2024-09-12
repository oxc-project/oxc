use super::{Eval, EvalContext, EvalResult, VOID};
use oxc_ast::ast::{BlockStatement, EmptyStatement};

impl<'a> Eval<'a> for BlockStatement<'a> {
    #[inline]
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        for stmt in &self.body {
            let _ = stmt.eval(ctx)?;
        }
        VOID
    }
}

impl<'a> Eval<'a> for EmptyStatement {
    #[inline]
    fn eval(&self, _ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        VOID
    }
}
