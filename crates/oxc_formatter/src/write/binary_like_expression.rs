use oxc_ast::ast::*;

use crate::{
    Format,
    formatter::{FormatResult, Formatter},
};

pub enum BinaryLikeExpression<'a, 'b> {
    LogicalExpression(&'b LogicalExpression<'a>),
    BinaryExpression(&'b BinaryExpression<'a>),
}

impl<'a> Format<'a> for BinaryLikeExpression<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::LogicalExpression(expr) => expr.fmt(f),
            Self::BinaryExpression(expr) => expr.fmt(f),
        }
    }
}
