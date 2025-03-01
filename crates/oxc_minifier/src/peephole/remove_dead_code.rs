use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::{constant_evaluation::ConstantEvaluation, side_effects::MayHaveSideEffects};
use oxc_traverse::Ancestor;

use crate::{ctx::Ctx, keep_var::KeepVar};

use super::{LatePeepholeOptimizations, PeepholeOptimizations};

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeRemoveDeadCode.java>
impl<'a, 'b> PeepholeOptimizations {
    pub fn remove_dead_code_exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: Ctx<'a, '_>) {
        if let Some(new_stmt) = match stmt {
            Statement::BlockStatement(s) => Self::try_optimize_block(s, ctx),
            Statement::IfStatement(s) => self.try_fold_if(s, ctx),
            Statement::ForStatement(s) => self.try_fold_for(s, ctx),
            Statement::ExpressionStatement(s) => Self::try_fold_iife(s, ctx),
            Statement::TryStatement(s) => Self::try_fold_try(s, ctx),
            Statement::LabeledStatement(s) => Self::try_fold_labeled(s, ctx),
            _ => None,
        } {
            *stmt = new_stmt;
            self.mark_current_function_as_changed();
        }

        self.try_fold_expression_stmt(stmt, ctx);
    }

    pub fn remove_dead_code_exit_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) {
        if let Some(folded_expr) = match expr {
            Expression::ConditionalExpression(e) => self.try_fold_conditional_expression(e, ctx),
            Expression::SequenceExpression(sequence_expression) => {
                Self::try_fold_sequence_expression(sequence_expression, ctx)
            }
            _ => None,
        } {
            *expr = folded_expr;
            self.mark_current_function_as_changed();
        }
    }

    /// Removes dead code thats comes after `return`, `throw`, `continue` and `break` statements.
    pub fn remove_dead_code_exit_statements(
        &mut self,
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: Ctx<'a, '_>,
    ) {
        // Remove code after `return` and `throw` statements
        let mut index = None;
        'outer: for (i, stmt) in stmts.iter().enumerate() {
            if stmt.is_jump_statement() {
                index.replace(i);
                break;
            }
            // Double check block statements folded by if statements above
            if let Statement::BlockStatement(block_stmt) = stmt {
                for stmt in &block_stmt.body {
                    if stmt.is_jump_statement() {
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
            // Keep module syntax and function declaration
            if s.is_module_declaration()
                || matches!(s.as_declaration(), Some(Declaration::FunctionDeclaration(_)))
            {
                return true;
            }
            false
        });

        let all_hoisted = keep_var.all_hoisted();
        if let Some(stmt) = keep_var.get_variable_declaration_statement() {
            stmts.push(stmt);
            if !all_hoisted {
                self.mark_current_function_as_changed();
            }
        }

        if stmts.len() != len {
            self.mark_current_function_as_changed();
        }
    }

    /// Remove block from single line blocks
    /// `{ block } -> block`
    fn try_optimize_block(
        stmt: &mut BlockStatement<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Statement<'a>> {
        match stmt.body.len() {
            0 => {
                let parent = ctx.parent();
                if parent.is_while_statement()
                    || parent.is_do_while_statement()
                    || parent.is_for_statement()
                    || parent.is_for_in_statement()
                    || parent.is_for_of_statement()
                    || parent.is_block_statement()
                    || parent.is_program()
                {
                    // Remove the block if it is empty and the parent is a block statement.
                    return Some(ctx.ast.statement_empty(stmt.span));
                }
                None
            }
            1 => {
                let s = &stmt.body[0];
                if matches!(s, Statement::VariableDeclaration(decl) if !decl.kind.is_var())
                    || matches!(s, Statement::ClassDeclaration(_))
                    || matches!(s, Statement::FunctionDeclaration(_))
                {
                    return None;
                }
                Some(stmt.body.remove(0))
            }
            _ => None,
        }
    }

    fn try_fold_if(
        &mut self,
        if_stmt: &mut IfStatement<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Statement<'a>> {
        // Descend and remove `else` blocks first.
        match &mut if_stmt.alternate {
            Some(Statement::IfStatement(alternate)) => {
                if let Some(new_stmt) = self.try_fold_if(alternate, ctx) {
                    if matches!(new_stmt, Statement::EmptyStatement(_)) {
                        if_stmt.alternate = None;
                    } else {
                        if_stmt.alternate = Some(new_stmt);
                    }
                    self.mark_current_function_as_changed();
                }
            }
            Some(Statement::BlockStatement(s)) if s.body.is_empty() => {
                if_stmt.alternate = None;
                self.mark_current_function_as_changed();
            }
            Some(Statement::EmptyStatement(_)) => {
                if_stmt.alternate = None;
                self.mark_current_function_as_changed();
            }
            _ => {}
        }

        if let Some(boolean) = if_stmt.test.get_side_free_boolean_value(&ctx) {
            // Use "1" and "0" instead of "true" and "false" to be shorter.
            // And also prevent swapping consequent and alternate when `!0` is encourtnered.
            if let Expression::BooleanLiteral(b) = &if_stmt.test {
                if_stmt.test = ctx.ast.expression_numeric_literal(
                    b.span,
                    if b.value { 1.0 } else { 0.0 },
                    None,
                    NumberBase::Decimal,
                );
            }
            let mut keep_var = KeepVar::new(ctx.ast);
            if boolean {
                if let Some(alternate) = &if_stmt.alternate {
                    keep_var.visit_statement(alternate);
                }
            } else {
                keep_var.visit_statement(&if_stmt.consequent);
            };
            if let Some(var_stmt) = keep_var.get_variable_declaration_statement() {
                if boolean {
                    if_stmt.alternate = Some(var_stmt);
                } else {
                    if_stmt.consequent = var_stmt;
                }
                return None;
            }
            return Some(if boolean {
                ctx.ast.move_statement(&mut if_stmt.consequent)
            } else {
                if_stmt.alternate.as_mut().map_or_else(
                    || ctx.ast.statement_empty(if_stmt.span),
                    |alternate| ctx.ast.move_statement(alternate),
                )
            });
        }
        None
    }

    fn try_fold_for(
        &mut self,
        for_stmt: &mut ForStatement<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Statement<'a>> {
        let test_boolean =
            for_stmt.test.as_ref().and_then(|test| test.evaluate_value_to_boolean(&ctx));
        if for_stmt.test.as_ref().is_some_and(|test| test.may_have_side_effects(&ctx)) {
            return None;
        }
        match test_boolean {
            Some(false) => match &mut for_stmt.init {
                Some(ForStatementInit::VariableDeclaration(var_init)) => {
                    let mut keep_var = KeepVar::new(ctx.ast);
                    keep_var.visit_statement(&for_stmt.body);
                    let mut var_decl = keep_var.get_variable_declaration();
                    if var_init.kind.is_var() {
                        if let Some(var_decl) = &mut var_decl {
                            var_decl
                                .declarations
                                .splice(0..0, ctx.ast.move_vec(&mut var_init.declarations));
                        } else {
                            var_decl =
                                Some(ctx.ast.alloc(ctx.ast.move_variable_declaration(var_init)));
                        }
                    }
                    Some(var_decl.map_or_else(
                        || ctx.ast.statement_empty(for_stmt.span),
                        Statement::VariableDeclaration,
                    ))
                }
                None => {
                    let mut keep_var = KeepVar::new(ctx.ast);
                    keep_var.visit_statement(&for_stmt.body);
                    Some(keep_var.get_variable_declaration().map_or_else(
                        || ctx.ast.statement_empty(for_stmt.span),
                        Statement::VariableDeclaration,
                    ))
                }
                _ => None,
            },
            Some(true) => {
                // Remove the test expression.
                for_stmt.test = None;
                self.mark_current_function_as_changed();
                None
            }
            None => None,
        }
    }

    /// Remove meaningless labeled statements.
    ///
    /// ```js
    /// a: break a;
    /// ```
    fn try_fold_labeled(s: &mut LabeledStatement<'a>, ctx: Ctx<'a, 'b>) -> Option<Statement<'a>> {
        let id = s.label.name.as_str();
        // Check the first statement in the block, or just the `break [id] ` statement.
        // Check if we need to remove the whole block.
        match &mut s.body {
            Statement::BreakStatement(break_stmt)
                if break_stmt.label.as_ref().is_some_and(|l| l.name.as_str() == id) => {}
            Statement::BlockStatement(block) if block.body.first().is_some_and(|first| matches!(first, Statement::BreakStatement(break_stmt) if break_stmt.label.as_ref().is_some_and(|l| l.name.as_str() == id))) => {}
            Statement::EmptyStatement(_) => {
                return Some(ctx.ast.statement_empty(s.span))
            }
            _ => return None,
        }
        let mut var = KeepVar::new(ctx.ast);
        var.visit_statement(&s.body);
        let var_decl = var.get_variable_declaration_statement();
        var_decl.unwrap_or_else(|| ctx.ast.statement_empty(s.span)).into()
    }

    fn try_fold_expression_stmt(&mut self, stmt: &mut Statement<'a>, ctx: Ctx<'a, 'b>) {
        let Statement::ExpressionStatement(expr_stmt) = stmt else { return };
        // We need to check if it is in arrow function with `expression: true`.
        // This is the only scenario where we can't remove it even if `ExpressionStatement`.
        if let Ancestor::ArrowFunctionExpressionBody(body) = ctx.ancestry.ancestor(1) {
            if *body.expression() {
                return;
            }
        }

        if self.remove_unused_expression(&mut expr_stmt.expression, ctx) {
            *stmt = ctx.ast.statement_empty(expr_stmt.span);
            self.mark_current_function_as_changed();
        }
    }

    fn try_fold_try(s: &mut TryStatement<'a>, ctx: Ctx<'a, 'b>) -> Option<Statement<'a>> {
        if !s.block.body.is_empty() {
            return None;
        }
        if let Some(finalizer) = &mut s.finalizer {
            if finalizer.body.is_empty() {
                Some(ctx.ast.statement_empty(s.span))
            } else {
                let mut block = ctx.ast.block_statement(finalizer.span, ctx.ast.vec());
                std::mem::swap(&mut **finalizer, &mut block);
                Some(Statement::BlockStatement(ctx.ast.alloc(block)))
            }
        } else {
            if let Some(handler) = &s.handler {
                if handler.body.body.iter().any(|s| matches!(s, Statement::VariableDeclaration(_)))
                {
                    return None;
                }
            }
            Some(ctx.ast.statement_empty(s.span))
        }
    }

    /// Try folding conditional expression (?:) if the condition results of the condition is known.
    fn try_fold_conditional_expression(
        &mut self,
        expr: &mut ConditionalExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        // Bail `let o = { f() { assert.ok(this !== o); } }; (true ? o.f : false)(); (true ? o.f : false)``;`
        let parent = ctx.ancestry.parent();
        if parent.is_tagged_template_expression()
            || matches!(parent, Ancestor::CallExpressionCallee(_))
        {
            return None;
        }

        expr.test.evaluate_value_to_boolean(&ctx).map(|v| {
            if expr.test.may_have_side_effects(&ctx) {
                // "(a, true) ? b : c" => "a, b"
                let exprs = ctx.ast.vec_from_iter([
                    {
                        let mut test = ctx.ast.move_expression(&mut expr.test);
                        self.remove_unused_expression(&mut test, ctx);
                        test
                    },
                    ctx.ast.move_expression(if v {
                        &mut expr.consequent
                    } else {
                        &mut expr.alternate
                    }),
                ]);
                ctx.ast.expression_sequence(expr.span, exprs)
            } else {
                ctx.ast.move_expression(if v { &mut expr.consequent } else { &mut expr.alternate })
            }
        })
    }

    fn try_fold_sequence_expression(
        sequence_expr: &mut SequenceExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        let should_keep_as_sequence_expr = matches!(
            ctx.parent(),
            Ancestor::CallExpressionCallee(_) | Ancestor::TaggedTemplateExpressionTag(_)
        );

        if should_keep_as_sequence_expr && sequence_expr.expressions.len() == 2 {
            return None;
        }

        let (should_fold, new_len) = sequence_expr.expressions.iter().enumerate().fold(
            (false, 0),
            |(mut should_fold, mut new_len), (i, expr)| {
                if i == sequence_expr.expressions.len() - 1 || expr.may_have_side_effects(&ctx) {
                    new_len += 1;
                } else {
                    should_fold = true;
                }
                (should_fold, new_len)
            },
        );

        if new_len == 0 {
            return Some(ctx.ast.expression_null_literal(sequence_expr.span));
        }

        if should_fold {
            let mut new_exprs = ctx.ast.vec_with_capacity(new_len);
            let len = sequence_expr.expressions.len();
            for (i, expr) in sequence_expr.expressions.iter_mut().enumerate() {
                if i == len - 1 || expr.may_have_side_effects(&ctx) {
                    new_exprs.push(ctx.ast.move_expression(expr));
                }
            }

            if should_keep_as_sequence_expr && new_exprs.len() == 1 {
                let number = ctx.ast.expression_numeric_literal(
                    sequence_expr.span,
                    1.0,
                    None,
                    NumberBase::Decimal,
                );
                new_exprs.insert(0, number);
            }

            if new_exprs.len() == 1 {
                return Some(new_exprs.pop().unwrap());
            }

            return Some(ctx.ast.expression_sequence(sequence_expr.span, new_exprs));
        }

        None
    }

    fn try_fold_iife(e: &ExpressionStatement<'a>, ctx: Ctx<'a, 'b>) -> Option<Statement<'a>> {
        let Expression::CallExpression(e) = &e.expression else { return None };
        if !e.arguments.is_empty() {
            return None;
        }
        let (params_empty, body_empty) = match &e.callee {
            Expression::FunctionExpression(f) => (f.params.is_empty(), f.body.as_ref()?.is_empty()),
            Expression::ArrowFunctionExpression(f) => (f.params.is_empty(), f.body.is_empty()),
            _ => return None,
        };
        (params_empty && body_empty).then(|| ctx.ast.statement_empty(e.span))
    }
}

impl<'a> LatePeepholeOptimizations {
    pub fn remove_dead_code_exit_class_body(body: &mut ClassBody<'a>, _ctx: Ctx<'a, '_>) {
        body.body.retain(|e| !matches!(e, ClassElement::StaticBlock(s) if s.body.is_empty()));
    }

    pub fn remove_empty_spread_arguments(args: &mut Vec<'a, Argument<'a>>) {
        if args.len() != 1 {
            return;
        }
        let Argument::SpreadElement(e) = &args[0] else { return };
        let Expression::ArrayExpression(e) = &e.argument else { return };
        if e.elements.is_empty() {
            args.drain(..);
        }
    }
}

/// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeRemoveDeadCodeTest.java>
#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    #[test]
    fn test_fold_block() {
        test("{{foo()}}", "foo()");
        test("{foo();{}}", "foo()");
        test("{{foo()}{}}", "foo()");
        test("{{foo()}{bar()}}", "foo(), bar()");
        test("{if(false)foo(); {bar()}}", "bar()");
        test("{if(false)if(false)if(false)foo(); {bar()}}", "bar()");

        test("{'hi'}", "");
        test("{x==3}", "x");
        test("{`hello ${foo}`}", "`${foo}`");
        test("{ (function(){x++}) }", "");
        test("{ (function foo(){x++; foo()}) }", "");
        test("function f(){return;}", "function f(){}");
        test("function f(){return 3;}", "function f(){return 3}");
        test("function f(){if(x)return; x=3; return; }", "function f(){ x ||= 3; }");
        test("{x=3;;;y=2;;;}", "x=3, y=2");

        // Cases to test for empty block.
        // test("while(x()){x}", "while(x());");
        test("while(x()){x()}", "for(;x();)x()");
        // test("for(x=0;x<100;x++){x}", "for(x=0;x<100;x++);");
        // test("for(x in y){x}", "for(x in y);");
        // test("for (x of y) {x}", "for(x of y);");
        test("for (let x = 1; x <10; x++ ) {}", "for (let x = 1; x <10; x++ );");
        test("for (var x = 1; x <10; x++ ) {}", "for (var x = 1; x <10; x++ );");
        test("do { } while (true)", "do;while(!0)");
        test(
            "function z(a) {
              {
                for (var i = 0; i < a; i++) {}
                foo()
              }
              bar()
            }",
            "function z(a) {
              for (var i = 0; i < a; i++);
              foo(), bar()
            }",
        );
    }

    #[test]
    fn test_remove_no_op_labelled_statement() {
        test("a: break a;", "");
        test("a: { break a; }", "");

        test("a: { break a; console.log('unreachable'); }", "");
        test("a: { break a; var x = 1; } x = 2;", "var x; x = 2;");

        test("b: { var x = 1; } x = 2;", "b: var x = 1; x = 2;");
        test("a: b: { var x = 1; } x = 2;", "a: b: var x = 1; x = 2;");
        test("foo:;", "");
    }

    #[test]
    fn test_fold_useless_for() {
        test("for(;false;) { foo() }", "");
        test("for(;void 0;) { foo() }", "");
        test("for(;undefined;) { foo() }", "");
        test("for(;true;) foo() ", "for(;;) foo() ");
        test_same("for(;;) foo()");
        test("for(;false;) { var a = 0; }", "var a");
        test("for(;false;) { const a = 0; }", "");
        test("for(;false;) { let a = 0; }", "");

        // Make sure it plays nice with minimizing
        test("for(;false;) { foo(); continue }", "");

        test("for (var { c, x: [d] } = {}; 0;);", "var { c, x: [d] } = {};");
        test("for (var se = [1, 2]; false;);", "var se = [1, 2];");
        test("for (var se = [1, 2]; false;) { var a = 0; }", "var se = [1, 2], a;");

        test("for (foo = bar; false;) {}", "for (foo = bar; !1;);");
        // test("l1:for(;false;) {  }", "");
    }

    #[test]
    fn test_minimize_loop_with_constant_condition_vanilla_for() {
        test("for(;true;) foo()", "for(;;) foo()");
        test("for(;0;) foo()", "");
        test("for(;0.0;) foo()", "");
        test("for(;NaN;) foo()", "");
        test("for(;null;) foo()", "");
        test("for(;undefined;) foo()", "");
        test("for(;'';) foo()", "");
    }

    #[test]
    fn test_fold_try_statement() {
        test("try { throw 0 } catch (e) { foo() }", "try { throw 0 } catch { foo() }");
        test("try {} catch (e) { var foo }", "try {} catch { var foo }");
        test("try {} catch (e) { foo() }", "");
        test("try {} catch (e) { foo() } finally {}", "");
        test("try {} finally { foo() }", "foo()");
        test("try {} catch (e) { foo() } finally { bar() }", "bar()");
        test("try {} finally { var x = foo() }", "var x = foo()");
        test("try {} catch (e) { foo() } finally { var x = bar() }", "var x = bar()");
        test("try {} finally { let x = foo() }", "{ let x = foo() }");
        test("try {} catch (e) { foo() } finally { let x = bar() }", "{ let x = bar();}");
        test("try {} catch (e) { } finally {}", "");
    }

    #[test]
    fn test_fold_if_statement() {
        test("if (foo) {}", "foo");
        test("if (foo) {} else {}", "foo");
        test("if (false) {}", "");
        test("if (true) {}", "");
    }

    #[test]
    fn test_fold_conditional() {
        test("true ? foo() : bar()", "foo()");
        test("false ? foo() : bar()", "bar()");
        test_same("foo() ? bar() : baz()");
        test("foo && false ? foo() : bar()", "(foo, bar());");
    }

    #[test]
    fn test_fold_iife() {
        test_same("var k = () => {}");
        test_same("var k = function () {}");
        // test("var a = (() => {})()", "var a = /* @__PURE__ */ (() => {})();");
        test("(() => {})()", "");
        // test("(() => a())()", "a();");
        // test("(() => { a() })()", "a();");
        // test("(() => { return a() })()", "a();");
        // test("(() => { let b = a; b() })()", "a();");
        // test("(() => { let b = a; return b() })()", "a();");
        test("(async () => {})()", "");
        test_same("(async () => { a() })()");
        // test("(async () => { let b = a; b() })()", "(async () => a())();");
        // test("var a = (function() {})()", "var a = /* @__PURE__ */ function() {}();");
        test("(function() {})()", "");
        test("(function*() {})()", "");
        test("(async function() {})()", "");
        test_same("(function() { a() })()");
        test_same("(function*() { a() })()");
        test_same("(async function() { a() })()");
        // test("(() => x)()", "x;");
        // test("/* @__PURE__ */ (() => x)()", "");
        // test("/* @__PURE__ */ (() => x)(y, z)", "y, z;");
    }

    #[test]
    fn test_remove_empty_static_block() {
        test("class Foo { static {}; foo }", "class Foo { foo }");
        test_same("class Foo { static { foo() } }");
    }

    #[test]
    fn keep_module_syntax() {
        test_same("throw foo; export let bar");
        test_same("throw foo; export default bar");
    }

    #[test]
    fn remove_empty_spread_arguments() {
        test("foo(...[])", "foo()");
        test("new Foo(...[])", "new Foo()");
    }

    #[test]
    fn remove_unreachable() {
        test("while(true) { break a; unreachable;}", "for(;;) break a");
        test("while(true) { continue a; unreachable;}", "for(;;) continue a");
        test("while(true) { throw a; unreachable;}", "for(;;) throw a");
        test("while(true) { return a; unreachable;}", "for(;;) return a");
    }
}
