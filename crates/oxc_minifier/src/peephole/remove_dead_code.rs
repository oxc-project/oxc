use oxc_allocator::Vec;
use oxc_ast::{ast::*, Visit};
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, IsLiteralValue, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
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

        if let Statement::ExpressionStatement(s) = stmt {
            if let Some(new_stmt) = Self::try_fold_expression_stmt(s, ctx) {
                *stmt = new_stmt;
                self.mark_current_function_as_changed();
            }
        }
    }

    pub fn remove_dead_code_exit_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) {
        if let Some(folded_expr) = match expr {
            Expression::ConditionalExpression(e) => Self::try_fold_conditional_expression(e, ctx),
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
        // Avoid compressing `if (x) { var x = 1 }` to `if (x) var x = 1` due to different
        // semantics according to AnnexB, which lead to different semantics.
        if stmt.body.len() == 1 && !stmt.body[0].is_declaration() {
            return Some(stmt.body.remove(0));
        }
        if stmt.body.len() == 0 {
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
        }
        None
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

        if let Some(boolean) = ctx.get_side_free_boolean_value(&if_stmt.test) {
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
        let test_boolean = for_stmt.test.as_ref().and_then(|test| ctx.get_boolean_value(test));
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

    fn try_fold_expression_stmt(
        stmt: &mut ExpressionStatement<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Statement<'a>> {
        // We need to check if it is in arrow function with `expression: true`.
        // This is the only scenario where we can't remove it even if `ExpressionStatement`.
        if let Ancestor::ArrowFunctionExpressionBody(body) = ctx.ancestry.ancestor(1) {
            if *body.expression() {
                return None;
            }
        }

        if stmt.expression.is_literal_value(false) {
            return Some(ctx.ast.statement_empty(stmt.span));
        }

        match &mut stmt.expression {
            Expression::MetaProperty(e) => Some(ctx.ast.statement_empty(e.span)),
            Expression::ArrayExpression(expr) => Self::try_fold_array_expression(expr, ctx),
            Expression::ObjectExpression(object_expr) => {
                Self::try_fold_object_expression(object_expr, ctx)
            }
            Expression::TemplateLiteral(template_lit) => {
                if !template_lit.expressions.is_empty() {
                    return None;
                }
                let mut expressions = ctx.ast.move_vec(&mut template_lit.expressions);
                if expressions.len() == 0 {
                    return Some(ctx.ast.statement_empty(stmt.span));
                } else if expressions.len() == 1 {
                    return Some(
                        ctx.ast.statement_expression(template_lit.span, expressions.pop().unwrap()),
                    );
                }
                Some(ctx.ast.statement_expression(
                    template_lit.span,
                    ctx.ast.expression_sequence(template_lit.span, expressions),
                ))
            }
            Expression::FunctionExpression(function_expr) if function_expr.id.is_none() => {
                Some(ctx.ast.statement_empty(stmt.span))
            }
            Expression::ArrowFunctionExpression(_) => Some(ctx.ast.statement_empty(stmt.span)),
            // `typeof x` -> ``
            Expression::UnaryExpression(unary_expr)
                if unary_expr.operator.is_typeof()
                    && unary_expr.argument.is_identifier_reference() =>
            {
                Some(ctx.ast.statement_empty(stmt.span))
            }
            // `typeof x.y` -> `x.y`, `void x` -> `x`
            // `+0n` -> `Uncaught TypeError: Cannot convert a BigInt value to a number`
            Expression::UnaryExpression(unary_expr)
                if matches!(unary_expr.operator, UnaryOperator::Typeof | UnaryOperator::Void) =>
            {
                Some(ctx.ast.statement_expression(
                    unary_expr.span,
                    ctx.ast.move_expression(&mut unary_expr.argument),
                ))
            }
            Expression::NewExpression(e) => {
                let Expression::Identifier(ident) = &e.callee else { return None };
                let len = e.arguments.len();
                if match ident.name.as_str() {
                    "WeakSet" | "WeakMap" if ctx.is_global_reference(ident) => match len {
                        0 => true,
                        1 => match e.arguments[0].as_expression()? {
                            Expression::NullLiteral(_) => true,
                            Expression::ArrayExpression(e) => e.elements.is_empty(),
                            e if ctx.is_expression_undefined(e) => true,
                            _ => false,
                        },
                        _ => false,
                    },
                    "Date" if ctx.is_global_reference(ident) => match len {
                        0 => true,
                        1 => {
                            let arg = e.arguments[0].as_expression()?;
                            let ty = ValueType::from(arg);
                            matches!(
                                ty,
                                ValueType::Null
                                    | ValueType::Undefined
                                    | ValueType::Boolean
                                    | ValueType::Number
                                    | ValueType::String
                            ) && !ctx.expression_may_have_side_effects(arg)
                        }
                        _ => false,
                    },
                    "Set" | "Map" if ctx.is_global_reference(ident) => match len {
                        0 => true,
                        1 => match e.arguments[0].as_expression()? {
                            Expression::NullLiteral(_) => true,
                            e if ctx.is_expression_undefined(e) => true,
                            _ => false,
                        },
                        _ => false,
                    },
                    _ => false,
                } {
                    return Some(ctx.ast.statement_empty(e.span));
                }
                None
            }
            _ => None,
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

    // `([1,2,3, foo()])` -> `foo()`
    fn try_fold_array_expression(
        array_expr: &mut ArrayExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Statement<'a>> {
        let mut transformed_elements = ctx.ast.vec();
        let mut pending_spread_elements = ctx.ast.vec();

        if array_expr.elements.len() == 0
            || array_expr.elements.iter().all(|el| match el {
                ArrayExpressionElement::SpreadElement(_) => true,
                ArrayExpressionElement::Identifier(ident) => ctx.is_global_reference(ident),
                _ => false,
            })
        {
            return None;
        }

        for el in &mut array_expr.elements {
            match el {
                ArrayExpressionElement::SpreadElement(_) => {
                    let spread_element = ctx.ast.move_array_expression_element(el);
                    pending_spread_elements.push(spread_element);
                }
                ArrayExpressionElement::Elision(_) => {}
                match_expression!(ArrayExpressionElement) => {
                    let el = el.to_expression_mut();
                    let el_expr = ctx.ast.move_expression(el);
                    if !el_expr.is_literal_value(false)
                        && !matches!(&el_expr, Expression::Identifier(ident) if !ctx.is_global_reference(ident))
                    {
                        if pending_spread_elements.len() > 0 {
                            // flush pending spread elements
                            transformed_elements.push(ctx.ast.expression_array(
                                el_expr.span(),
                                pending_spread_elements,
                                None,
                            ));
                            pending_spread_elements = ctx.ast.vec();
                        }
                        transformed_elements.push(el_expr);
                    }
                }
            }
        }

        if pending_spread_elements.len() > 0 {
            transformed_elements.push(ctx.ast.expression_array(
                array_expr.span,
                pending_spread_elements,
                None,
            ));
        }

        if transformed_elements.is_empty() {
            return Some(ctx.ast.statement_empty(array_expr.span));
        } else if transformed_elements.len() == 1 {
            return Some(
                ctx.ast.statement_expression(array_expr.span, transformed_elements.pop().unwrap()),
            );
        }

        Some(ctx.ast.statement_expression(
            array_expr.span,
            ctx.ast.expression_sequence(array_expr.span, transformed_elements),
        ))
    }

    // `{a: 1, b: 2, c: foo()}` -> `foo()`
    fn try_fold_object_expression(
        _object_expr: &mut ObjectExpression<'a>,
        _ctx: Ctx<'a, 'b>,
    ) -> Option<Statement<'a>> {
        None
    }

    /// Try folding conditional expression (?:) if the condition results of the condition is known.
    fn try_fold_conditional_expression(
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

        match ctx.get_boolean_value(&expr.test) {
            Some(v) => {
                if ctx.expression_may_have_side_effects(&expr.test) {
                    let mut exprs = ctx.ast.vec_with_capacity(2);
                    exprs.push(ctx.ast.move_expression(&mut expr.test));
                    exprs.push(ctx.ast.move_expression(if v {
                        &mut expr.consequent
                    } else {
                        &mut expr.alternate
                    }));
                    Some(ctx.ast.expression_sequence(expr.span, exprs))
                } else {
                    Some(ctx.ast.move_expression(if v {
                        &mut expr.consequent
                    } else {
                        &mut expr.alternate
                    }))
                }
            }
            None => None,
        }
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
                if i == sequence_expr.expressions.len() - 1
                    || ctx.expression_may_have_side_effects(expr)
                {
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
                if i == len - 1 || ctx.expression_may_have_side_effects(expr) {
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
        // test("{{foo()}{bar()}}", "foo();bar()");
        test("{if(false)foo(); {bar()}}", "bar()");
        test("{if(false)if(false)if(false)foo(); {bar()}}", "bar()");

        test("{'hi'}", "");
        test("{x==3}", "x == 3");
        test("{`hello ${foo}`}", "`hello ${foo}`");
        test("{ (function(){x++}) }", "");
        test("function f(){return;}", "function f(){}");
        test("function f(){return 3;}", "function f(){return 3}");
        // test_same("function f(){if(x)return; x=3; return; }");
        // test("{x=3;;;y=2;;;}", "x=3;y=2");

        // Cases to test for empty block.
        // test("while(x()){x}", "while(x());");
        test("while(x()){x()}", "for(;x();)x()");
        // test("for(x=0;x<100;x++){x}", "for(x=0;x<100;x++);");
        // test("for(x in y){x}", "for(x in y);");
        // test("for (x of y) {x}", "for(x of y);");
        test("for (let x = 1; x <10; x++ ) {}", "for (let x = 1; x <10; x++ );");
        test("for (var x = 1; x <10; x++ ) {}", "for (var x = 1; x <10; x++ );");
        test("do { } while (true)", "do;while(!0)");
    }

    #[test]
    fn test_remove_no_op_labelled_statement() {
        test("a: break a;", "");
        test("a: { break a; }", "");

        test("a: { break a; console.log('unreachable'); }", "");
        test("a: { break a; var x = 1; } x = 2;", "var x; x = 2;");

        test_same("b: { var x = 1; } x = 2;");
        test_same("a: b: { var x = 1; } x = 2;");
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
    #[ignore]
    fn test_object_literal() {
        test("({})", "");
        test("({a:1})", "");
        test("({a:foo()})", "foo()");
        test("({'a':foo()})", "foo()");
        // Object-spread may trigger getters.
        test_same("({...a})");
        test_same("({...foo()})");

        test("({ [bar()]: foo() })", "bar(), foo()");
        test_same("({ ...baz, [bar()]: foo() })");
    }

    #[test]
    fn test_array_literal() {
        test("([])", "");
        test("([1])", "");
        test("([a])", "[a]");
        test("var a; ([a])", "var a;");
        test("([foo()])", "foo()");
        test_same("baz.map((v) => [v])");
    }

    #[test]
    fn test_array_literal_containing_spread() {
        test_same("([...c])");
        test("([4, ...c, a])", "([...c], a)");
        test("var a; ([4, ...c, a])", "var a; ([...c])");
        test("([foo(), ...c, bar()])", "(foo(), [...c], bar())");
        test("([...a, b, ...c])", "([...a, b, ...c])");
        test("var b; ([...a, b, ...c])", "var b; ([...a, ...c])");
        test_same("([...b, ...c])"); // It would also be fine if the spreads were split apart.
    }

    #[test]
    fn test_fold_unary_expression_statement() {
        test("typeof x", "");
        test("typeof x?.y", "x?.y");
        test("typeof x.y", "x.y");
        test("typeof x.y.z()", "x.y.z()");
        test("void x", "x");
        test("void x?.y", "x?.y");
        test("void x.y", "x.y");
        test("void x.y.z()", "x.y.z()");

        // Removed in `MinimizeConditions`, to keep this pass idempotent for DCE.
        test_same("!x");
        test_same("!x?.y");
        test_same("!x.y");
        test_same("!x.y.z()");
        test_same("-x.y.z()");

        test_same("delete x");
        test_same("delete x.y");
        test_same("delete x.y.z()");
        test_same("+0n"); // Uncaught TypeError: Cannot convert a BigInt value to a number
    }

    #[test]
    fn test_fold_sequence_expr() {
        test("('foo', 'bar', 'baz')", "");
        test("('foo', 'bar', baz())", "baz()");
        test("('foo', bar(), baz())", "bar(), baz()");
        test("(() => {}, bar(), baz())", "bar(), baz()");
        test("(function k() {}, k(), baz())", "k(), baz()");
        test_same("(0, o.f)();");
        test("var obj = Object((null, 2, 3), 1, 2);", "var obj = Object(3, 1, 2);");
        test_same("(0 instanceof 0, foo)");
        test_same("(0 in 0, foo)");
        test_same("React.useEffect(() => (isMountRef.current = !1, () => { isMountRef.current = !0; }), [])");
    }

    #[test]
    fn test_fold_try_statement() {
        test("try { throw 0 } catch (e) { foo() }", "try { throw 0 } catch { foo() }");
        test("try {} catch (e) { var foo }", "try {} catch { var foo }");
        test("try {} catch (e) { foo() }", "");
        test("try {} catch (e) { foo() } finally {}", "");
        test("try {} finally { foo() }", "foo()");
        test("try {} catch (e) { foo() } finally { bar() }", "bar()");
        test("try {} finally { var x = foo() }", "{ var x = foo() }");
        test("try {} catch (e) { foo() } finally { var x = bar() }", "{ var x = bar() }");
        test("try {} finally { let x = foo() }", "{ let x = foo() }");
        test("try {} catch (e) { foo() } finally { let x = bar() }", "{ let x = bar();}");
        test("try {} catch (e) { } finally {}", "");
    }

    #[test]
    fn test_fold_if_statement() {
        test("if (foo) {}", "foo");
        // FIXME
        // test("if (foo) {} else {}", "foo");
        test("if (false) {}", "");
        test("if (true) {}", "");
    }

    #[test]
    fn test_fold_conditional() {
        test("true ? foo() : bar()", "foo()");
        test("false ? foo() : bar()", "bar()");
        test_same("foo() ? bar() : baz()");
        test("foo && false ? foo() : bar()", "(foo && !1, bar());");
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
    fn remove_expression_statement() {
        test("void 0", "");
        test("-1", "");
        test("!1", "");
        test("1", "");
        test("import.meta", "");
    }

    #[test]
    fn test_new_constructor_side_effect() {
        test("new WeakSet()", "");
        test("new WeakSet(null)", "");
        test("new WeakSet(void 0)", "");
        test("new WeakSet([])", "");
        test_same("new WeakSet([x])");
        test_same("new WeakSet(x)");
        test("new WeakMap()", "");
        test("new WeakMap(null)", "");
        test("new WeakMap(void 0)", "");
        test("new WeakMap([])", "");
        test_same("new WeakMap([x])");
        test_same("new WeakMap(x)");
        test("new Date()", "");
        test("new Date('')", "");
        test("new Date(0)", "");
        test("new Date(null)", "");
        test("new Date(true)", "");
        test("new Date(false)", "");
        test("new Date(undefined)", "");
        test_same("new Date(x)");
        test("new Set()", "");
        // test("new Set([a, b, c])", "");
        test("new Set(null)", "");
        test("new Set(undefined)", "");
        test("new Set(void 0)", "");
        test_same("new Set(x)");
        test("new Map()", "");
        test("new Map(null)", "");
        test("new Map(undefined)", "");
        test("new Map(void 0)", "");
        // test_same("new Map([x])");
        test_same("new Map(x)");
        // test("new Map([[a, b], [c, d]])", "");
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
