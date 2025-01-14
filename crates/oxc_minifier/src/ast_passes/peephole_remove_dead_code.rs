use oxc_allocator::Vec;
use oxc_ast::{ast::*, Visit};
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, IsLiteralValue},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
use oxc_traverse::{traverse_mut_with_ctx, Ancestor, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{ctx::Ctx, keep_var::KeepVar, CompressorPass};

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeRemoveDeadCode.java>
pub struct PeepholeRemoveDeadCode {
    pub(crate) changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeRemoveDeadCode {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        self.changed = false;
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeRemoveDeadCode {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = Ctx(ctx);
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
            self.changed = true;
        }

        if let Statement::ExpressionStatement(s) = stmt {
            if let Some(new_stmt) = Self::try_fold_expression_stmt(s, ctx) {
                *stmt = new_stmt;
                self.changed = true;
            }
        }
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if stmts.iter().any(|stmt| matches!(stmt, Statement::EmptyStatement(_))) {
            stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        }
        self.dead_code_elimination(stmts, Ctx(ctx));
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = Ctx(ctx);
        if let Some(folded_expr) = match expr {
            Expression::ConditionalExpression(e) => Self::try_fold_conditional_expression(e, ctx),
            Expression::SequenceExpression(sequence_expression) => {
                Self::try_fold_sequence_expression(sequence_expression, ctx)
            }
            _ => None,
        } {
            *expr = folded_expr;
            self.changed = true;
        }
    }
}

impl<'a, 'b> PeepholeRemoveDeadCode {
    pub fn new() -> Self {
        Self { changed: false }
    }

    /// Removes dead code thats comes after `return` statements after inlining `if` statements
    fn dead_code_elimination(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: Ctx<'a, 'b>) {
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
                    self.changed = true;
                }
            }
            Some(Statement::BlockStatement(s)) if s.body.is_empty() => {
                if_stmt.alternate = None;
                self.changed = true;
            }
            Some(Statement::EmptyStatement(_)) => {
                if_stmt.alternate = None;
                self.changed = true;
            }
            _ => {}
        }

        // `if (test) {}` -> `test`
        if if_stmt.alternate.is_none()
            && match &if_stmt.consequent {
                Statement::EmptyStatement(_) => true,
                Statement::BlockStatement(s) => s.body.is_empty(),
                _ => false,
            }
        {
            let expr = ctx.ast.move_expression(&mut if_stmt.test);
            return Some(ctx.ast.statement_expression(if_stmt.span, expr));
        }

        if let Some(boolean) = ctx.get_side_free_boolean_value(&if_stmt.test) {
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
                self.changed = true;
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
        // TODO find a better way to handle this.

        if let Ancestor::ArrowFunctionExpressionBody(body) = ctx.ancestry.ancestor(1) {
            if *body.expression() {
                return None;
            }
        }

        stmt.expression
            .is_literal_value(false)
            .then(|| Some(ctx.ast.statement_empty(stmt.span)))
            .unwrap_or_else(|| match &mut stmt.expression {
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
                            ctx.ast.statement_expression(
                                template_lit.span,
                                expressions.pop().unwrap(),
                            ),
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
                    if matches!(
                        unary_expr.operator,
                        UnaryOperator::Typeof | UnaryOperator::Void
                    ) =>
                {
                    Some(ctx.ast.statement_expression(
                        unary_expr.span,
                        ctx.ast.move_expression(&mut unary_expr.argument),
                    ))
                }
                _ => None,
            })
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
            || array_expr
                .elements
                .iter()
                .all(|el| matches!(el, ArrayExpressionElement::SpreadElement(_)))
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
                        && !matches!(el_expr, Expression::Identifier(_))
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
            Some(true) => Some(ctx.ast.move_expression(&mut expr.consequent)),
            Some(false) => Some(ctx.ast.move_expression(&mut expr.alternate)),
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
                if i == sequence_expr.expressions.len() - 1 || expr.may_have_side_effects() {
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
                if i == len - 1 || expr.may_have_side_effects() {
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

/// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeRemoveDeadCodeTest.java>
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
    fn test_fold_block() {
        fold("{{foo()}}", "foo()");
        fold("{foo();{}}", "foo()");
        fold("{{foo()}{}}", "foo()");
        // fold("{{foo()}{bar()}}", "foo();bar()");
        fold("{if(false)foo(); {bar()}}", "bar()");
        fold("{if(false)if(false)if(false)foo(); {bar()}}", "bar()");

        fold("{'hi'}", "");
        fold("{x==3}", "x == 3");
        fold("{`hello ${foo}`}", "`hello ${foo}`");
        fold("{ (function(){x++}) }", "");
        fold_same("function f(){return;}");
        fold("function f(){return 3;}", "function f(){return 3}");
        // fold_same("function f(){if(x)return; x=3; return; }");
        // fold("{x=3;;;y=2;;;}", "x=3;y=2");

        // Cases to test for empty block.
        // fold("while(x()){x}", "while(x());");
        fold("while(x()){x()}", "for(;x();)x()");
        // fold("for(x=0;x<100;x++){x}", "for(x=0;x<100;x++);");
        // fold("for(x in y){x}", "for(x in y);");
        // fold("for (x of y) {x}", "for(x of y);");
        fold("for (let x = 1; x <10; x++ ) {}", "for (let x = 1; x <10; x++ );");
        fold("for (var x = 1; x <10; x++ ) {}", "for (var x = 1; x <10; x++ );");
        fold("do { } while (true)", "do;while(true)");
    }

    #[test]
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
        fold("foo:;", "");
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

        fold("for (var { c, x: [d] } = {}; 0;);", "var { c, x: [d] } = {};");
        fold("for (var se = [1, 2]; false;);", "var se = [1, 2];");
        fold("for (var se = [1, 2]; false;) { var a = 0; }", "var se = [1, 2], a;");

        fold("for (foo = bar; false;) {}", "for (foo = bar; false;);");
        // fold("l1:for(;false;) {  }", "");
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
    #[ignore]
    fn test_object_literal() {
        fold("({})", "");
        fold("({a:1})", "");
        fold("({a:foo()})", "foo()");
        fold("({'a':foo()})", "foo()");
        // Object-spread may trigger getters.
        fold_same("({...a})");
        fold_same("({...foo()})");

        fold("({ [bar()]: foo() })", "bar(), foo()");
        fold_same("({ ...baz, [bar()]: foo() })");
    }

    #[test]
    fn test_array_literal() {
        fold("([])", "");
        fold("([1])", "");
        fold("([a])", "");
        fold("([foo()])", "foo()");
        fold_same("baz.map((v) => [v])");
    }

    #[test]
    fn test_array_literal_containing_spread() {
        fold_same("([...c])");
        fold("([4, ...c, a])", "([...c])");
        fold("([foo(), ...c, bar()])", "(foo(), [...c], bar())");
        fold("([...a, b, ...c])", "([...a, ...c])");
        fold_same("([...b, ...c])"); // It would also be fine if the spreads were split apart.
    }

    #[test]
    fn test_fold_unary_expression_statement() {
        fold("typeof x", "");
        fold("typeof x?.y", "x?.y");
        fold("typeof x.y", "x.y");
        fold("typeof x.y.z()", "x.y.z()");
        fold("void x", "x");
        fold("void x?.y", "x?.y");
        fold("void x.y", "x.y");
        fold("void x.y.z()", "x.y.z()");

        // Removed in `MinimizeConditions`, to keep this pass idempotent for DCE.
        fold_same("!x");
        fold_same("!x?.y");
        fold_same("!x.y");
        fold_same("!x.y.z()");
        fold_same("-x.y.z()");

        fold_same("delete x");
        fold_same("delete x.y");
        fold_same("delete x.y.z()");
        fold_same("+0n"); // Uncaught TypeError: Cannot convert a BigInt value to a number
    }

    #[test]
    fn test_fold_sequence_expr() {
        fold("('foo', 'bar', 'baz')", "");
        fold("('foo', 'bar', baz())", "baz()");
        fold("('foo', bar(), baz())", "bar(), baz()");
        fold("(() => {}, bar(), baz())", "bar(), baz()");
        fold("(function k() {}, k(), baz())", "k(), baz()");
        fold_same("(0, o.f)();");
        fold("var obj = Object((null, 2, 3), 1, 2);", "var obj = Object(3, 1, 2);");
        fold_same("(0 instanceof 0, foo)");
        fold_same("(0 in 0, foo)");
        fold_same("React.useEffect(() => (isMountRef.current = false, () => { isMountRef.current = true; }), [])");
    }

    #[test]
    fn test_fold_try_statement() {
        fold_same("try { throw 0 } catch (e) { foo() }");
        fold_same("try {} catch (e) { var foo }");
        fold("try {} catch (e) { foo() }", "");
        fold("try {} catch (e) { foo() } finally {}", "");
        fold("try {} finally { foo() }", "{ foo() }");
        fold("try {} catch (e) { foo() } finally { bar() }", "{ bar() }");
        fold("try {} finally { var x = foo() }", "{ var x = foo() }");
        fold("try {} catch (e) { foo() } finally { var x = bar() }", "{ var x = bar() }");
        fold("try {} finally { let x = foo() }", "{ let x = foo() }");
        fold("try {} catch (e) { foo() } finally { let x = bar() }", "{ let x = bar();}");
        fold("try {} catch () { } finally {}", "");
    }

    #[test]
    fn test_fold_if_statement() {
        test("if (foo) {}", "foo");
        test("if (foo) {} else {}", "foo");
        test("if (false) {}", "");
        test("if (true) {}", "");
    }

    #[test]
    fn test_fold_iife() {
        fold_same("var k = () => {}");
        fold_same("var k = function () {}");
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
}
