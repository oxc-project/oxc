use oxc_ast::ast::{Expression, Statement};
use oxc_ast::VisitResult;

#[derive(Default)]
pub struct TransformResult<'a> {
    keep: bool,
    remove: bool,
    replacement: Vec<TransformReplaceKind<'a>>,
}

impl<'a> VisitResult for TransformResult<'a> {
    fn keep() -> Self {
        let mut result = Self::default();
        result.keep = true;
        result
    }

    fn remove() -> Self {
        let mut result = Self::default();
        result.remove = true;
        result
    }

    fn replace() -> Self {
        Self::default()
    }
}

pub enum TransformReplaceKind<'a> {
    Expression(Expression<'a>),
    Statement(Statement<'a>),
}

impl<'a> TransformResult<'a> {
    pub fn get_replacement(self) -> Option<Vec<TransformReplaceKind<'a>>> {
        if self.replacement.is_empty() {
            None
        } else {
            Some(self.replacement)
        }
    }

    pub fn with_expression(mut self, expr: Expression<'a>) -> Self {
        self.check_keep_state();
        self.replacement = vec![TransformReplaceKind::Expression(expr)];
        self
    }

    pub fn with_statement(mut self, stmt: Statement<'a>) -> Self {
        self.check_keep_state();
        self.replacement = vec![TransformReplaceKind::Statement(stmt)];
        self
    }

    pub fn with_many_expressions(mut self, exprs: Vec<Expression<'a>>) -> Self {
        self.check_keep_state();
        self.replacement = exprs.into_iter().map(TransformReplaceKind::Expression).collect();
        self
    }

    pub fn with_many_statements(mut self, stmts: Vec<Statement<'a>>) -> Self {
        self.check_keep_state();
        self.replacement = stmts.into_iter().map(TransformReplaceKind::Statement).collect();
        self
    }

    fn check_keep_state(&self) {
        if self.keep {
            panic!("Cannot replace a node, as the result has been marked as keep node!")
        }
    }
}
