use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::{constant_evaluation::ConstantEvaluation, side_effects::MayHaveSideEffects};
use oxc_span::GetSpan;
use oxc_traverse::Ancestor;

use crate::{ctx::Ctx, keep_var::KeepVar};

use super::{LatePeepholeOptimizations, PeepholeOptimizations, State};

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeRemoveDeadCode.java>
impl<'a, 'b> PeepholeOptimizations {
    pub fn remove_dead_code_exit_statement(
        &self,
        stmt: &mut Statement<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) {
        if let Some(new_stmt) = match stmt {
            Statement::BlockStatement(s) => Self::try_optimize_block(s, ctx),
            Statement::IfStatement(s) => Self::try_fold_if(s, state, ctx),
            Statement::ForStatement(s) => self.try_fold_for(s, state, ctx),
            Statement::TryStatement(s) => Self::try_fold_try(s, ctx),
            Statement::LabeledStatement(s) => Self::try_fold_labeled(s, ctx),
            _ => None,
        } {
            *stmt = new_stmt;
            state.changed = true;
        }

        self.try_fold_expression_stmt(stmt, state, ctx);
    }

    pub fn remove_dead_code_exit_expression(
        &self,
        expr: &mut Expression<'a>,
        state: &mut State,
        ctx: Ctx<'a, '_>,
    ) {
        if let Some(folded_expr) = match expr {
            Expression::ConditionalExpression(e) => {
                self.try_fold_conditional_expression(e, state, ctx)
            }
            Expression::SequenceExpression(sequence_expression) => {
                self.try_fold_sequence_expression(sequence_expression, state, ctx)
            }
            _ => None,
        } {
            *expr = folded_expr;
            state.changed = true;
        }
    }

    /// Removes dead code thats comes after `return`, `throw`, `continue` and `break` statements.
    pub fn remove_dead_code_exit_statements(
        &self,
        stmts: &mut Vec<'a, Statement<'a>>,
        state: &mut State,
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
                state.changed = true;
            }
        }

        if stmts.len() != len {
            state.changed = true;
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
        if_stmt: &mut IfStatement<'a>,
        state: &mut State,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Statement<'a>> {
        // Descend and remove `else` blocks first.
        match &mut if_stmt.alternate {
            Some(Statement::IfStatement(alternate)) => {
                if let Some(new_stmt) = Self::try_fold_if(alternate, state, ctx) {
                    if matches!(new_stmt, Statement::EmptyStatement(_)) {
                        if_stmt.alternate = None;
                    } else {
                        if_stmt.alternate = Some(new_stmt);
                    }
                    state.changed = true;
                }
            }
            Some(Statement::BlockStatement(s)) if s.body.is_empty() => {
                if_stmt.alternate = None;
                state.changed = true;
            }
            Some(Statement::EmptyStatement(_)) => {
                if_stmt.alternate = None;
                state.changed = true;
            }
            _ => {}
        }

        if let Some(boolean) = if_stmt.test.evaluate_value_to_boolean(&ctx) {
            let test_has_side_effects = if_stmt.test.may_have_side_effects(&ctx);
            // Use "1" and "0" instead of "true" and "false" to be shorter.
            // And also prevent swapping consequent and alternate when `!0` is encountered.
            if !test_has_side_effects {
                if_stmt.test = ctx.ast.expression_numeric_literal(
                    if_stmt.test.span(),
                    if boolean { 1.0 } else { 0.0 },
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
            }
            let var_stmt = keep_var.get_variable_declaration_statement();
            let has_var_stmt = var_stmt.is_some();
            if let Some(var_stmt) = var_stmt {
                if boolean {
                    if_stmt.alternate = Some(var_stmt);
                } else {
                    if_stmt.consequent = var_stmt;
                }
                return None;
            }
            if test_has_side_effects {
                if !has_var_stmt {
                    if boolean {
                        if_stmt.alternate = None;
                    } else {
                        if_stmt.consequent = ctx.ast.statement_empty(if_stmt.consequent.span());
                    }
                }
                return Some(
                    ctx.ast.statement_if(
                        if_stmt.span,
                        if_stmt.test.take_in(ctx.ast.allocator),
                        if_stmt.consequent.take_in(ctx.ast.allocator),
                        if_stmt
                            .alternate
                            .as_mut()
                            .map(|alternate| alternate.take_in(ctx.ast.allocator)),
                    ),
                );
            }
            return Some(if boolean {
                if_stmt.consequent.take_in(ctx.ast.allocator)
            } else {
                if_stmt.alternate.as_mut().map_or_else(
                    || ctx.ast.statement_empty(if_stmt.span),
                    |alternate| alternate.take_in(ctx.ast.allocator),
                )
            });
        }
        None
    }

    fn try_fold_for(
        &self,
        for_stmt: &mut ForStatement<'a>,
        state: &mut State,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Statement<'a>> {
        if let Some(init) = &mut for_stmt.init {
            if let Some(init) = init.as_expression_mut() {
                if self.remove_unused_expression(init, state, ctx) {
                    for_stmt.init = None;
                    state.changed = true;
                }
            }
        }
        if let Some(update) = &mut for_stmt.update {
            if self.remove_unused_expression(update, state, ctx) {
                for_stmt.update = None;
                state.changed = true;
            }
        }

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
                                .splice(0..0, var_init.declarations.take_in(ctx.ast.allocator));
                        } else {
                            var_decl = Some(var_init.take_in_box(ctx.ast.allocator));
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
                state.changed = true;
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

    fn try_fold_expression_stmt(
        &self,
        stmt: &mut Statement<'a>,
        state: &mut State,
        ctx: Ctx<'a, 'b>,
    ) {
        let Statement::ExpressionStatement(expr_stmt) = stmt else { return };
        // We need to check if it is in arrow function with `expression: true`.
        // This is the only scenario where we can't remove it even if `ExpressionStatement`.
        if let Ancestor::ArrowFunctionExpressionBody(body) = ctx.ancestry.ancestor(1) {
            if *body.expression() {
                return;
            }
        }

        if self.remove_unused_expression(&mut expr_stmt.expression, state, ctx) {
            *stmt = ctx.ast.statement_empty(expr_stmt.span);
            state.changed = true;
        }
    }

    fn try_fold_try(s: &mut TryStatement<'a>, ctx: Ctx<'a, 'b>) -> Option<Statement<'a>> {
        if let Some(handler) = &mut s.handler {
            if s.block.body.is_empty() {
                let mut var = KeepVar::new(ctx.ast);
                var.visit_block_statement(&handler.body);
                handler.body.body.clear();
                if let Some(var_decl) = var.get_variable_declaration_statement() {
                    handler.body.body.push(var_decl);
                }
            }
        }

        if let Some(finalizer) = &s.finalizer {
            if finalizer.body.is_empty() && s.handler.is_some() {
                s.finalizer = None;
            }
        }

        if s.block.body.is_empty()
            && s.handler.as_ref().is_none_or(|handler| handler.body.body.is_empty())
        {
            if let Some(finalizer) = &mut s.finalizer {
                let mut block = ctx.ast.block_statement(finalizer.span, ctx.ast.vec());
                std::mem::swap(&mut **finalizer, &mut block);
                Some(Statement::BlockStatement(ctx.ast.alloc(block)))
            } else {
                Some(ctx.ast.statement_empty(s.span))
            }
        } else {
            None
        }
    }

    /// Try folding conditional expression (?:) if the condition results of the condition is known.
    fn try_fold_conditional_expression(
        &self,
        expr: &mut ConditionalExpression<'a>,
        state: &mut State,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        expr.test.evaluate_value_to_boolean(&ctx).map(|v| {
            if expr.test.may_have_side_effects(&ctx) {
                // "(a, true) ? b : c" => "a, b"
                let exprs = ctx.ast.vec_from_array([
                    {
                        let mut test = expr.test.take_in(ctx.ast.allocator);
                        self.remove_unused_expression(&mut test, state, ctx);
                        test
                    },
                    if v {
                        expr.consequent.take_in(ctx.ast.allocator)
                    } else {
                        expr.alternate.take_in(ctx.ast.allocator)
                    },
                ]);
                ctx.ast.expression_sequence(expr.span, exprs)
            } else {
                let result_expr = if v {
                    expr.consequent.take_in(ctx.ast.allocator)
                } else {
                    expr.alternate.take_in(ctx.ast.allocator)
                };

                let should_keep_as_sequence_expr =
                    Self::should_keep_indirect_access(&result_expr, ctx);
                // "(1 ? a.b : 0)()" => "(0, a.b)()"
                if should_keep_as_sequence_expr {
                    ctx.ast.expression_sequence(
                        expr.span,
                        ctx.ast.vec_from_array([
                            ctx.ast.expression_numeric_literal(
                                expr.span,
                                0.0,
                                None,
                                NumberBase::Decimal,
                            ),
                            result_expr,
                        ]),
                    )
                } else {
                    result_expr
                }
            }
        })
    }

    fn try_fold_sequence_expression(
        &self,
        sequence_expr: &mut SequenceExpression<'a>,
        state: &mut State,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        let should_keep_as_sequence_expr = sequence_expr
            .expressions
            .last()
            .is_some_and(|last_expr| Self::should_keep_indirect_access(last_expr, ctx));
        if should_keep_as_sequence_expr
            && sequence_expr.expressions.len() == 2
            && sequence_expr.expressions.first().unwrap().is_number_0()
        {
            return None;
        }

        let old_len = sequence_expr.expressions.len();
        let mut i = 0;
        sequence_expr.expressions.retain_mut(|e| {
            i += 1;
            if should_keep_as_sequence_expr && i == old_len - 1 {
                if self.remove_unused_expression(e, state, ctx) {
                    *e = ctx.ast.expression_numeric_literal(
                        e.span(),
                        0.0,
                        None,
                        NumberBase::Decimal,
                    );
                    state.changed = true;
                }
                return true;
            }
            if i == old_len {
                return true;
            }
            !self.remove_unused_expression(e, state, ctx)
        });
        if sequence_expr.expressions.len() == 1 {
            return Some(sequence_expr.expressions.pop().unwrap());
        }

        if sequence_expr.expressions.len() != old_len {
            state.changed = true;
        }
        None
    }

    /// Whether the indirect access should be kept.
    /// For example, `(0, foo.bar)()` should not be transformed to `foo.bar()`.
    /// Example case: `let o = { f() { assert.ok(this !== o); } }; (true && o.f)(); (true && o.f)``;`
    ///
    /// * `access_value` - The expression that may need to be kept as indirect reference (`foo.bar` in the example above)
    pub fn should_keep_indirect_access(access_value: &Expression<'a>, ctx: Ctx<'a, 'b>) -> bool {
        match ctx.parent() {
            Ancestor::CallExpressionCallee(_) | Ancestor::TaggedTemplateExpressionTag(_) => {
                match access_value {
                    Expression::Identifier(id) => id.name == "eval" && ctx.is_global_reference(id),
                    match_member_expression!(Expression) => true,
                    _ => false,
                }
            }
            Ancestor::UnaryExpressionArgument(unary) => match unary.operator() {
                UnaryOperator::Typeof => {
                    // Example case: `typeof (0, foo)` (error) -> `typeof foo` (no error)
                    if let Expression::Identifier(id) = access_value {
                        ctx.is_global_reference(id)
                    } else {
                        false
                    }
                }
                UnaryOperator::Delete => {
                    match access_value {
                        // Example case: `delete (0, foo)` (no error) -> `delete foo` (error)
                        Expression::Identifier(_)
                        // Example case: `delete (0, foo.#a)` (no error) -> `delete foo.#a` (error)
                        | Expression::PrivateFieldExpression(_)
                        // Example case: `typeof (0, foo.bar)` (noop) -> `typeof foo.bar` (deletes bar)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::StaticMemberExpression(_) => true,
                        // Example case: `typeof (0, foo?.bar)` (noop) -> `typeof foo?.bar` (deletes bar)
                        Expression::ChainExpression(chain) => {
                            matches!(&chain.expression, match_member_expression!(ChainElement))
                        }
                        _ => false,
                    }
                }
                _ => false,
            },
            _ => false,
        }
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
        test("try {} catch (e) { var foo; bar() } finally {}", "try {} catch { var foo }");
        test(
            "try {} catch (e) { var foo; bar() } finally { baz() }",
            "try {} catch { var foo } finally { baz() }",
        );
        test("try {} catch (e) { foo() }", "");
        test("try {} catch (e) { foo() } finally {}", "");
        test("try {} finally { foo() }", "foo()");
        test("try {} catch (e) { foo() } finally { bar() }", "bar()");
        test("try {} finally { var x = foo() }", "var x = foo()");
        test("try {} catch (e) { foo() } finally { var x = bar() }", "var x = bar()");
        test("try {} finally { let x = foo() }", "{ let x = foo() }");
        test("try {} catch (e) { foo() } finally { let x = bar() }", "{ let x = bar();}");
        test("try {} catch (e) { } finally {}", "");
        test("try { foo() } catch (e) { bar() } finally {}", "try { foo() } catch { bar() }");
        test_same("try { foo() } catch { bar() } finally { baz() }");
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

        test("var a; (true ? a : 0)()", "var a; a()");
        test("var a; (true ? a.b : 0)()", "var a; (0, a.b)()");
        test("var a; (false ? 0 : a)()", "var a; a()");
        test("var a; (false ? 0 : a.b)()", "var a; (0, a.b)()");
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

    #[test]
    fn remove_unused_expressions_in_sequence() {
        test("true, foo();", "foo();");
        test("(0, foo)();", "foo();");
        test("(0, foo)``;", "foo``;");
        test("(0, foo)?.();", "foo?.();");
        test_same("(0, eval)();"); // this can be compressed to `eval?.()`
        test_same("(0, eval)``;"); // this can be compressed to `eval?.()`
        test_same("(0, eval)?.();"); // this can be compressed to `eval?.()`
        test("var eval; (0, eval)();", "var eval; eval();");
        test_same("(0, foo.bar)();");
        test_same("(0, foo.bar)``;");
        test_same("(0, foo.bar)?.();");
        test("(true, foo.bar)();", "(0, foo.bar)();");
        test("(true, true, foo.bar)();", "(0, foo.bar)();");
        test("var foo; (true, foo.bar)();", "var foo; (0, foo.bar)();");
        test("var foo; (true, true, foo.bar)();", "var foo; (0, foo.bar)();");

        test("typeof (0, foo);", "foo");
        test_same("v = typeof (0, foo);");
        test("var foo; typeof (0, foo);", "var foo;");
        test("var foo; v = typeof (0, foo);", "var foo; v = typeof foo");
        test("typeof 0", "");

        test_same("delete (0, foo);");
        test_same("delete (0, foo.#bar);");
        test_same("delete (0, foo.bar);");
        test_same("delete (0, foo[bar]);");
        test_same("delete (0, foo?.bar);");
    }

    #[test]
    fn remove_unused_expressions_in_for() {
        test(
            "var i; for (i = 0, 0; i < 10; i++) foo(i);",
            "var i; for (i = 0; i < 10; i++) foo(i);",
        );
        test(
            "var i; for (i = 0; i < 10; 0, i++, 0) foo(i);",
            "var i; for (i = 0; i < 10; i++) foo(i);",
        );
    }
}
