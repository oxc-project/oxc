use oxc_allocator::Vec;
use oxc_ast::{ast::*, Visit};
use oxc_span::SPAN;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::node_util::IsLiteralValue;
use crate::{keep_var::KeepVar, node_util::NodeUtil, tri::Tri, CompressorPass};

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeRemoveDeadCode.java>
pub struct PeepholeRemoveDeadCode {
    changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeRemoveDeadCode {
    fn changed(&self) -> bool {
        self.changed
    }

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = false;
        oxc_traverse::walk_program(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeRemoveDeadCode {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(new_stmt) = match stmt {
            Statement::IfStatement(if_stmt) => self.try_fold_if(if_stmt, ctx),
            Statement::ForStatement(for_stmt) => self.try_fold_for(for_stmt, ctx),
            Statement::ExpressionStatement(expr_stmt) => {
                Self::try_fold_expression_stmt(expr_stmt, ctx)
            }
            _ => None,
        } {
            *stmt = new_stmt;
            self.changed = true;
        }
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if stmts.iter().any(|stmt| matches!(stmt, Statement::EmptyStatement(_))) {
            stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        }
        self.dead_code_elimination(stmts, ctx);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(folded_expr) = match expr {
            Expression::ConditionalExpression(e) => Self::try_fold_conditional_expression(e, ctx),
            _ => None,
        } {
            *expr = folded_expr;
            self.changed = true;
        }
    }
}

impl<'a> PeepholeRemoveDeadCode {
    pub fn new() -> Self {
        Self { changed: false }
    }

    /// Removes dead code thats comes after `return` statements after inlining `if` statements
    fn dead_code_elimination(
        &mut self,
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Remove code after `return` and `throw` statements
        let mut index = None;
        'outer: for (i, stmt) in stmts.iter().enumerate() {
            if matches!(stmt, Statement::ReturnStatement(_) | Statement::ThrowStatement(_)) {
                index.replace(i);
                break;
            }
            // Double check block statements folded by if statements above
            if let Statement::BlockStatement(block_stmt) = stmt {
                for stmt in &block_stmt.body {
                    if matches!(stmt, Statement::ReturnStatement(_) | Statement::ThrowStatement(_))
                    {
                        index.replace(i);
                        break 'outer;
                    }
                }
            }
        }

        let Some(index) = index else { return };
        if index == stmts.len() - 1 {
            return;
        }

        let mut keep_var = KeepVar::new(ctx.ast);

        for stmt in stmts.iter().skip(index + 1) {
            keep_var.visit_statement(stmt);
        }

        let mut i = 0;
        let len = stmts.len();
        stmts.retain(|s| {
            i += 1;
            if i - 1 <= index {
                return true;
            }
            // keep function declaration
            if matches!(s.as_declaration(), Some(Declaration::FunctionDeclaration(_))) {
                return true;
            }
            false
        });

        let all_hoisted = keep_var.all_hoisted();
        if let Some(stmt) = keep_var.get_variable_declaration_statement() {
            stmts.push(stmt);
            if !all_hoisted {
                self.changed = true;
            }
        }

        if stmts.len() != len {
            self.changed = true;
        }
    }

    fn try_fold_if(
        &mut self,
        if_stmt: &mut IfStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        // Descend and remove `else` blocks first.
        if let Some(Statement::IfStatement(alternate)) = &mut if_stmt.alternate {
            if let Some(new_stmt) = self.try_fold_if(alternate, ctx) {
                if matches!(new_stmt, Statement::EmptyStatement(_)) {
                    if_stmt.alternate = None;
                    self.changed = true;
                } else {
                    if_stmt.alternate = Some(new_stmt);
                }
            }
        }

        match ctx.get_boolean_value(&if_stmt.test) {
            Tri::True => {
                // self.changed = true;
                Some(ctx.ast.move_statement(&mut if_stmt.consequent))
            }
            Tri::False => {
                Some(if let Some(alternate) = &mut if_stmt.alternate {
                    ctx.ast.move_statement(alternate)
                } else {
                    // Keep hoisted `vars` from the consequent block.
                    let mut keep_var = KeepVar::new(ctx.ast);
                    keep_var.visit_statement(&if_stmt.consequent);
                    keep_var
                        .get_variable_declaration_statement()
                        .unwrap_or_else(|| ctx.ast.statement_empty(SPAN))
                })
                // self.changed = true;
            }
            Tri::Unknown => None,
        }
    }

    fn try_fold_for(
        &mut self,
        for_stmt: &mut ForStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        let test_boolean =
            for_stmt.test.as_ref().map_or(Tri::Unknown, |test| ctx.get_boolean_value(test));
        match test_boolean {
            Tri::False => {
                // Remove the entire `for` statement.
                // Check vars in statement
                let mut keep_var = KeepVar::new(ctx.ast);
                keep_var.visit_statement(&for_stmt.body);
                Some(
                    keep_var
                        .get_variable_declaration_statement()
                        .unwrap_or_else(|| ctx.ast.statement_empty(SPAN)),
                )
            }
            Tri::True => {
                // Remove the test expression.
                for_stmt.test = None;
                self.changed = true;
                None
            }
            Tri::Unknown => None,
        }
    }

    fn try_fold_expression_stmt(
        stmt: &mut ExpressionStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        // We need to check if it is in arrow function with `expression: true`.
        // This is the only scenario where we can't remove it even if `ExpressionStatement`.
        // TODO find a better way to handle this.

        stmt.expression
            .is_literal_value(false)
            .then(|| {
                if let Ancestor::ArrowFunctionExpressionBody(body) = ctx.ancestry.ancestor(1) {
                    if *body.expression() {
                        return None;
                    }
                }
                Some(ctx.ast.statement_empty(SPAN))
            })
            .unwrap_or(None)
    }

    /// Try folding conditional expression (?:) if the condition results of the condition is known.
    fn try_fold_conditional_expression(
        expr: &mut ConditionalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match ctx.get_boolean_value(&expr.test) {
            Tri::True => {
                // Bail `let o = { f() { assert.ok(this !== o); } }; (true ? o.f : false)(); (true ? o.f : false)``;`
                let parent = ctx.ancestry.parent();
                if parent.is_tagged_template_expression()
                    || matches!(parent, Ancestor::CallExpressionCallee(_))
                {
                    return None;
                }
                Some(ctx.ast.move_expression(&mut expr.consequent))
            }
            Tri::False => Some(ctx.ast.move_expression(&mut expr.alternate)),
            Tri::Unknown => None,
        }
    }
}

/// <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeRemoveDeadCodeTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, positive: &str) {
        let allocator = Allocator::default();
        let mut pass = super::PeepholeRemoveDeadCode::new();
        tester::test(&allocator, source_text, positive, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    fn fold_same(js: &str) {
        test_same(js);
    }

    fn fold(js: &str, expected: &str) {
        test(js, expected);
    }

    #[test]
    #[ignore]
    fn test_remove_no_op_labelled_statement() {
        fold("a: break a;", "");
        fold("a: { break a; }", "");

        fold(
            //
            "a: { break a; console.log('unreachable'); }", //
            "",
        );
        fold(
            //
            "a: { break a; var x = 1; } x = 2;", //
            "var x; x = 2;",
        );

        fold_same("b: { var x = 1; } x = 2;");
        fold_same("a: b: { var x = 1; } x = 2;");
    }

    #[test]
    fn test_fold_useless_for() {
        fold("for(;false;) { foo() }", "");
        fold("for(;void 0;) { foo() }", "");
        fold("for(;undefined;) { foo() }", "");
        fold("for(;true;) foo() ", "for(;;) foo() ");
        fold_same("for(;;) foo()");
        fold("for(;false;) { var a = 0; }", "var a");
        fold("for(;false;) { const a = 0; }", "");
        fold("for(;false;) { let a = 0; }", "");

        // Make sure it plays nice with minimizing
        fold("for(;false;) { foo(); continue }", "");

        // fold("l1:for(;false;) {  }", "");

        // TODO handle single block statement
        fold_same("for(;a;) { foo(); }");
    }

    #[test]
    fn test_minimize_loop_with_constant_condition_vanilla_for() {
        fold("for(;true;) foo()", "for(;;) foo()");
        fold("for(;0;) foo()", "");
        fold("for(;0.0;) foo()", "");
        fold("for(;NaN;) foo()", "");
        fold("for(;null;) foo()", "");
        fold("for(;undefined;) foo()", "");
        fold("for(;'';) foo()", "");
    }

    #[test]
    fn test_object_literal() {
        fold("({})", "");
        fold("({a:1})", "");
        // fold("({a:foo()})", "foo()");
        // fold("({'a':foo()})", "foo()");
        // Object-spread may trigger getters.
        fold_same("({...a})");
        fold_same("({...foo()})");
    }

    #[test]
    fn test_array_literal() {
        fold("([])", "");
        fold("([1])", "");
        // fold("([a])", "a");
        // fold("([foo()])", "foo()");
    }

    #[test]
    #[ignore]
    fn test_array_literal_containing_spread() {
        fold_same("([...c])");
        fold("([4, ...c, a])", "([...c])");
        fold("([foo(), ...c, bar()])", "(foo(), [...c], bar())");
        fold("([...a, b, ...c])", "([...a], [...c])");
        fold_same("([...b, ...c])"); // It would also be fine if the spreads were split apart.
    }
}
