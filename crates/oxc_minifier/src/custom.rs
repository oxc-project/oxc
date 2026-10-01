use oxc_ast::ast::{Expression, SequenceExpression};

pub(crate) trait SequenceExpressionExt<'a> {
    fn last_expression(&self) -> Option<&Expression<'a>>;

    fn pop_last_expression(&mut self) -> Option<Expression<'a>>;
}

impl<'a> SequenceExpressionExt<'a> for SequenceExpression<'a> {
    fn last_expression(&self) -> Option<&Expression<'a>> {
        let last = self.expressions.last()?;
        match last {
            Expression::SequenceExpression(seq) if !seq.expressions.is_empty() => {
                seq.last_expression()
            }
            last => Some(last),
        }
    }

    fn pop_last_expression(&mut self) -> Option<Expression<'a>> {
        if let Some(Expression::SequenceExpression(seq)) = self.expressions.last_mut() {
            let last = seq.pop_last_expression()?;
            if seq.expressions.is_empty() {
                self.expressions.pop();
            }
            return Some(last);
        }

        self.expressions.pop()
    }
}
