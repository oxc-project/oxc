use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
use oxc_traverse::Ancestor;

use crate::{ctx::Ctx, keep_var::KeepVar};

use super::PeepholeOptimizations;

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeRemoveDeadCode.java>
impl<'a> PeepholeOptimizations {
    /// Remove block from single line blocks
    /// `{ block } -> block`
    pub fn try_optimize_block(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::BlockStatement(s) = stmt else { return };
        match s.body.len() {
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
                    *stmt = ctx.ast.statement_empty(s.span);
                    ctx.state.changed = true;
                }
            }
            1 => {
                let first = &s.body[0];
                if matches!(first, Statement::VariableDeclaration(decl) if !decl.kind.is_var())
                    || matches!(first, Statement::ClassDeclaration(_))
                    || matches!(first, Statement::FunctionDeclaration(_))
                {
                    return;
                }
                *stmt = s.body.remove(0);
                ctx.state.changed = true;
            }
            _ => {}
        }
    }

    pub fn try_fold_if(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };
        // Descend and remove `else` blocks first.
        match &mut if_stmt.alternate {
            Some(Statement::IfStatement(_)) => {
                Self::try_fold_if(if_stmt.alternate.as_mut().unwrap(), ctx);
            }
            Some(Statement::BlockStatement(s)) if s.body.is_empty() => {
                if_stmt.alternate = None;
            }
            Some(Statement::EmptyStatement(_)) => {
                if_stmt.alternate = None;
            }
            _ => {}
        }

        if let Some(boolean) = if_stmt.test.evaluate_value_to_boolean(ctx) {
            let test_has_side_effects = if_stmt.test.may_have_side_effects(ctx);
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
            let var_stmt = keep_var
                .get_variable_declaration_statement()
                .and_then(|stmt| Self::remove_unused_variable_declaration(stmt, ctx));
            let has_var_stmt = var_stmt.is_some();
            if let Some(var_stmt) = var_stmt {
                if boolean {
                    if_stmt.alternate = Some(var_stmt);
                } else {
                    if_stmt.consequent = var_stmt;
                }
                return;
            }
            if test_has_side_effects {
                if !has_var_stmt {
                    if boolean {
                        if_stmt.alternate = None;
                    } else {
                        if_stmt.consequent = ctx.ast.statement_empty(if_stmt.consequent.span());
                    }
                }
                return;
            }
            *stmt = if boolean {
                if_stmt.consequent.take_in(ctx.ast)
            } else if let Some(alternate) = if_stmt.alternate.take() {
                alternate
            } else {
                ctx.ast.statement_empty(if_stmt.span)
            };
            ctx.state.changed = true;
        }
    }

    pub fn try_fold_for(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::ForStatement(for_stmt) = stmt else { return };
        if let Some(init) = &mut for_stmt.init {
            if let Some(init) = init.as_expression_mut() {
                if Self::remove_unused_expression(init, ctx) {
                    for_stmt.init = None;
                    ctx.state.changed = true;
                }
            }
        }
        if let Some(update) = &mut for_stmt.update {
            if Self::remove_unused_expression(update, ctx) {
                for_stmt.update = None;
                ctx.state.changed = true;
            }
        }

        let test_boolean =
            for_stmt.test.as_ref().and_then(|test| test.evaluate_value_to_boolean(ctx));
        if for_stmt.test.as_ref().is_some_and(|test| test.may_have_side_effects(ctx)) {
            return;
        }
        match test_boolean {
            Some(false) => match &for_stmt.init {
                Some(ForStatementInit::VariableDeclaration(_)) => {
                    let mut keep_var = KeepVar::new(ctx.ast);
                    keep_var.visit_statement(&for_stmt.body);
                    let mut var_decl = keep_var.get_variable_declaration();
                    let Some(ForStatementInit::VariableDeclaration(var_init)) = &mut for_stmt.init
                    else {
                        return;
                    };
                    if var_init.kind.is_var() {
                        if let Some(var_decl) = &mut var_decl {
                            var_decl
                                .declarations
                                .splice(0..0, var_init.declarations.take_in(ctx.ast));
                        } else {
                            var_decl = Some(var_init.take_in_box(ctx.ast));
                        }
                    }
                    *stmt = var_decl.map_or_else(
                        || ctx.ast.statement_empty(for_stmt.span),
                        Statement::VariableDeclaration,
                    );
                    ctx.state.changed = true;
                }
                None => {
                    let mut keep_var = KeepVar::new(ctx.ast);
                    keep_var.visit_statement(&for_stmt.body);
                    *stmt = keep_var.get_variable_declaration().map_or_else(
                        || ctx.ast.statement_empty(for_stmt.span),
                        Statement::VariableDeclaration,
                    );
                    ctx.state.changed = true;
                }
                _ => {}
            },
            Some(true) => {
                // Remove the test expression.
                for_stmt.test = None;
                ctx.state.changed = true;
            }
            None => {}
        }
    }

    /// Remove meaningless labeled statements.
    ///
    /// ```js
    /// a: break a;
    /// ```
    pub fn try_fold_labeled(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::LabeledStatement(s) = stmt else { return };
        let id = s.label.name.as_str();
        // Check the first statement in the block, or just the `break [id] ` statement.
        // Check if we need to remove the whole block.
        match &mut s.body {
            Statement::BreakStatement(break_stmt)
                if break_stmt.label.as_ref().is_some_and(|l| l.name.as_str() == id) => {}
            Statement::BlockStatement(block) if block.body.first().is_some_and(|first| matches!(first, Statement::BreakStatement(break_stmt) if break_stmt.label.as_ref().is_some_and(|l| l.name.as_str() == id))) => {}
            Statement::EmptyStatement(_) => {
                *stmt = ctx.ast.statement_empty(s.span);
                ctx.state.changed = true;
                return;
            }
            _ => return
        }
        let mut var = KeepVar::new(ctx.ast);
        var.visit_statement(&s.body);
        let var_decl = var.get_variable_declaration_statement();
        *stmt = var_decl.unwrap_or_else(|| ctx.ast.statement_empty(s.span));
        ctx.state.changed = true;
    }

    pub fn try_fold_expression_stmt(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::ExpressionStatement(expr_stmt) = stmt else { return };
        // We need to check if it is in arrow function with `expression: true`.
        // This is the only scenario where we can't remove it even if `ExpressionStatement`.
        if let Ancestor::ArrowFunctionExpressionBody(body) = ctx.ancestry.ancestor(1) {
            if *body.expression() {
                return;
            }
        }

        if Self::remove_unused_expression(&mut expr_stmt.expression, ctx) {
            *stmt = ctx.ast.statement_empty(expr_stmt.span);
            ctx.state.changed = true;
        }
    }

    pub fn try_fold_try(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::TryStatement(s) = stmt else { return };
        if let Some(handler) = &s.handler {
            if s.block.body.is_empty() {
                let mut var = KeepVar::new(ctx.ast);
                var.visit_block_statement(&handler.body);
                let Some(handler) = &mut s.handler else { return };
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
            *stmt = if let Some(finalizer) = &mut s.finalizer {
                let mut block = ctx.ast.block_statement(finalizer.span, ctx.ast.vec());
                std::mem::swap(&mut **finalizer, &mut block);
                Statement::BlockStatement(ctx.ast.alloc(block))
            } else {
                ctx.ast.statement_empty(s.span)
            };
            ctx.state.changed = true;
        }
    }

    /// Try folding conditional expression (?:) if the condition results of the condition is known.
    pub fn try_fold_conditional_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::ConditionalExpression(e) = expr else { return };
        let Some(v) = e.test.evaluate_value_to_boolean(ctx) else { return };
        ctx.state.changed = true;
        *expr = if e.test.may_have_side_effects(ctx) {
            // "(a, true) ? b : c" => "a, b"
            let exprs = ctx.ast.vec_from_array([
                {
                    let mut test = e.test.take_in(ctx.ast);
                    Self::remove_unused_expression(&mut test, ctx);
                    test
                },
                if v { e.consequent.take_in(ctx.ast) } else { e.alternate.take_in(ctx.ast) },
            ]);
            ctx.ast.expression_sequence(e.span, exprs)
        } else {
            let result_expr =
                if v { e.consequent.take_in(ctx.ast) } else { e.alternate.take_in(ctx.ast) };
            let should_keep_as_sequence_expr = Self::should_keep_indirect_access(&result_expr, ctx);
            // "(1 ? a.b : 0)()" => "(0, a.b)()"
            if should_keep_as_sequence_expr {
                ctx.ast.expression_sequence(
                    e.span,
                    ctx.ast.vec_from_array([
                        ctx.ast.expression_numeric_literal(e.span, 0.0, None, NumberBase::Decimal),
                        result_expr,
                    ]),
                )
            } else {
                result_expr
            }
        };
    }

    pub fn remove_sequence_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::SequenceExpression(e) = expr else { return };
        let should_keep_as_sequence_expr = e
            .expressions
            .last()
            .is_some_and(|last_expr| Self::should_keep_indirect_access(last_expr, ctx));
        if should_keep_as_sequence_expr
            && e.expressions.len() == 2
            && e.expressions.first().unwrap().is_number_0()
        {
            return;
        }
        let old_len = e.expressions.len();
        let mut i = 0;
        e.expressions.retain_mut(|e| {
            i += 1;
            if should_keep_as_sequence_expr && i == old_len - 1 {
                if Self::remove_unused_expression(e, ctx) {
                    *e = ctx.ast.expression_numeric_literal(
                        e.span(),
                        0.0,
                        None,
                        NumberBase::Decimal,
                    );
                    ctx.state.changed = true;
                }
                return true;
            }
            if i == old_len {
                return true;
            }
            !Self::remove_unused_expression(e, ctx)
        });
        if e.expressions.len() != old_len {
            ctx.state.changed = true;
        }
        if e.expressions.len() == 1 {
            *expr = e.expressions.pop().unwrap();
            ctx.state.changed = true;
        }
    }

    pub fn keep_track_of_pure_functions(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        match stmt {
            Statement::FunctionDeclaration(f) => {
                if let Some(body) = &f.body {
                    Self::try_save_pure_function(
                        f.id.as_ref(),
                        &f.params,
                        body,
                        f.r#async,
                        f.generator,
                        ctx,
                    );
                }
            }
            Statement::VariableDeclaration(decl) => {
                for d in &decl.declarations {
                    if let BindingPatternKind::BindingIdentifier(id) = &d.id.kind {
                        match &d.init {
                            Some(Expression::ArrowFunctionExpression(a)) => {
                                Self::try_save_pure_function(
                                    Some(id),
                                    &a.params,
                                    &a.body,
                                    a.r#async,
                                    false,
                                    ctx,
                                );
                            }
                            Some(Expression::FunctionExpression(f)) => {
                                if let Some(body) = &f.body {
                                    Self::try_save_pure_function(
                                        Some(id),
                                        &f.params,
                                        body,
                                        f.r#async,
                                        f.generator,
                                        ctx,
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn try_save_pure_function(
        id: Option<&BindingIdentifier<'a>>,
        params: &FormalParameters<'a>,
        body: &FunctionBody<'a>,
        r#async: bool,
        generator: bool,
        ctx: &mut Ctx<'a, '_>,
    ) {
        if r#async || generator {
            return;
        }
        // `function foo({}) {} foo(null)` is runtime type error.
        if !params.items.iter().all(|pat| pat.pattern.kind.is_binding_identifier()) {
            return;
        }
        if body.statements.iter().any(|stmt| stmt.may_have_side_effects(ctx)) {
            return;
        }
        let Some(symbol_id) = id.and_then(|id| id.symbol_id.get()) else { return };
        if ctx.scoping().get_resolved_references(symbol_id).all(|r| r.flags().is_read_only()) {
            ctx.state.pure_functions.insert(
                symbol_id,
                if body.is_empty() { Some(ConstantValue::Undefined) } else { None },
            );
        }
    }

    pub fn remove_dead_code_call_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::CallExpression(e) = expr else { return };
        if let Expression::Identifier(ident) = &e.callee {
            let reference_id = ident.reference_id();
            if let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() {
                if matches!(
                    ctx.state.pure_functions.get(&symbol_id),
                    Some(Some(ConstantValue::Undefined))
                ) {
                    let mut exprs =
                        Self::fold_arguments_into_needed_expressions(&mut e.arguments, ctx);
                    if exprs.is_empty() {
                        *expr = ctx.ast.void_0(e.span);
                        ctx.state.changed = true;
                        return;
                    }
                    exprs.push(ctx.ast.void_0(e.span));
                    *expr = ctx.ast.expression_sequence(e.span, exprs);
                    ctx.state.changed = true;
                }
            }
        }
    }

    /// Whether the indirect access should be kept.
    /// For example, `(0, foo.bar)()` should not be transformed to `foo.bar()`.
    /// Example case: `let o = { f() { assert.ok(this !== o); } }; (true && o.f)(); (true && o.f)``;`
    ///
    /// * `access_value` - The expression that may need to be kept as indirect reference (`foo.bar` in the example above)
    pub fn should_keep_indirect_access(access_value: &Expression<'a>, ctx: &Ctx<'a, '_>) -> bool {
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

    pub fn remove_dead_code_exit_class_body(body: &mut ClassBody<'a>, _ctx: &mut Ctx<'a, '_>) {
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
    use crate::{
        CompressOptions, CompressOptionsUnused,
        tester::{default_options, test, test_options, test_same, test_same_options},
    };

    #[track_caller]
    fn test_unused(source_text: &str, expected: &str) {
        let options =
            CompressOptions { unused: CompressOptionsUnused::Remove, ..default_options() };
        test_options(source_text, expected, &options);
    }

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
        test("a: { break a; var x = 1; } x = 2;", "var x = 2;");

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
        test("if (false) { var a; console.log(a) }", "if (0) var a");
        test_unused("if (false) { var a; console.log(a) }", "");
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

        test("(function () { return; var a })()", "(function () { return; var a })()");
        test_unused("(function () { return; var a })()", "");
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

    #[test]
    fn remove_constant_value() {
        test("const foo = false; if (foo) { console.log('foo') }", "const foo = !1;");
    }

    #[test]
    fn remove_empty_function() {
        let options = CompressOptions::smallest();
        test_options("function foo() {} foo()", "", &options);
        test_options("function foo() {} foo(); foo()", "", &options);
        test_options("var foo = () => {}; foo()", "", &options);
        test_options("var foo = () => {}; foo(a)", "a", &options);
        test_options("var foo = () => {}; foo(a, b)", "a, b", &options);
        test_options("var foo = () => {}; foo(...a, b)", "[...a], b", &options);
        test_options("var foo = () => {}; foo(...a, ...b)", "[...a], [...b]", &options);
        test_options("var foo = () => {}; x = foo()", "x = void 0", &options);
        test_options("var foo = () => {}; x = foo(a(), b())", "x = (a(), b(), void 0)", &options);
        test_options("var foo = function () {}; foo()", "", &options);

        test_same_options("function foo({}) {} foo()", &options);
        test_options("var foo = ({}) => {}; foo()", "(({}) => {})()", &options);
        test_options("var foo = function ({}) {}; foo()", "(function ({}) {})()", &options);

        test_same_options("async function foo({}) {} foo()", &options);
        test_options("var foo = async ({}) => {}; foo()", "(async ({}) => {})()", &options);
        test_options(
            "var foo = async function ({}) {}; foo()",
            "(async function ({}) {})()",
            &options,
        );

        test_same_options("function* foo({}) {} foo()", &options);
        test_options("var foo = function*({}) {}; foo()", "(function*({}) {})()", &options);
    }
}
