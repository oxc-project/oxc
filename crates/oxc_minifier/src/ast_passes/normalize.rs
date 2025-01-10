use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{ctx::Ctx, CompressorPass};

/// Normalize AST
///
/// Make subsequent AST passes easier to analyze:
///
/// * convert whiles to fors
/// * convert `Infinity` to `f64::INFINITY`
/// * convert `NaN` to `f64::NaN`
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/Normalize.java>
pub struct Normalize;

impl<'a> CompressorPass<'a> for Normalize {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for Normalize {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::WhileStatement(_) => {
                Self::convert_while_to_for(stmt, ctx);
            }
            Statement::IfStatement(s) => Self::wrap_to_avoid_ambiguous_else(s, ctx),
            _ => {}
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::Identifier(_) = expr {
            Self::convert_infinity_or_nan_into_number(expr, ctx);
        }
    }
}

impl<'a> Normalize {
    pub fn new() -> Self {
        Self
    }

    fn convert_while_to_for(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::WhileStatement(while_stmt) = ctx.ast.move_statement(stmt) else { return };
        let while_stmt = while_stmt.unbox();
        let for_stmt = ctx.ast.alloc_for_statement_with_scope_id(
            while_stmt.span,
            None,
            Some(while_stmt.test),
            None,
            while_stmt.body,
            ctx.create_child_scope_of_current(ScopeFlags::empty()),
        );
        *stmt = Statement::ForStatement(for_stmt);
    }

    fn convert_infinity_or_nan_into_number(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::Identifier(ident) = expr {
            let ctx = Ctx(ctx);
            let value = if ctx.is_identifier_infinity(ident) {
                f64::INFINITY
            } else if ctx.is_identifier_nan(ident) {
                f64::NAN
            } else {
                return;
            };
            *expr =
                ctx.ast.expression_numeric_literal(ident.span, value, None, NumberBase::Decimal);
        }
    }

    // Wrap to avoid ambiguous else.
    // `if (foo) if (bar) baz else quaz` ->  `if (foo) { if (bar) baz else quaz }`
    fn wrap_to_avoid_ambiguous_else(if_stmt: &mut IfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::IfStatement(if2) = &mut if_stmt.consequent {
            if if2.alternate.is_some() {
                let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
                if_stmt.consequent =
                    Statement::BlockStatement(ctx.ast.alloc_block_statement_with_scope_id(
                        if_stmt.consequent.span(),
                        ctx.ast.vec1(ctx.ast.move_statement(&mut if_stmt.consequent)),
                        scope_id,
                    ));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::Normalize::new();
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    #[test]
    fn test_while() {
        // Verify while loops are converted to FOR loops.
        test("while(c < b) foo()", "for(; c < b;) foo()");
    }
}
