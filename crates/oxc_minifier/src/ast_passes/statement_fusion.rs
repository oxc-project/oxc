use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::CompressorPass;

/// Statement Fusion
///
/// Tries to fuse all the statements in a block into a one statement by using COMMAs or statements.
///
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/StatementFusion.java>
pub struct StatementFusion {
    changed: bool,
}

impl<'a> CompressorPass<'a> for StatementFusion {
    fn changed(&self) -> bool {
        self.changed
    }

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = false;
        oxc_traverse::walk_program(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for StatementFusion {
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fuse_statements(&mut program.body, ctx);
    }

    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fuse_statements(&mut body.statements, ctx);
    }

    fn exit_block_statement(&mut self, block: &mut BlockStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fuse_statements(&mut block.body, ctx);
    }
}

impl<'a> StatementFusion {
    pub fn new() -> Self {
        Self { changed: false }
    }

    fn fuse_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if Self::can_fuse_into_one_statement(stmts) {
            self.fuse_into_one_statement(stmts, ctx);
        }
    }

    fn can_fuse_into_one_statement(stmts: &[Statement<'a>]) -> bool {
        let len = stmts.len();
        if len <= 1 {
            return false;
        }
        if stmts[0..len - 1].iter().any(|s| !matches!(s, Statement::ExpressionStatement(_))) {
            return false;
        }
        Self::is_fusable_control_statement(&stmts[len - 1])
    }

    fn is_fusable_control_statement(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::ExpressionStatement(_)
            | Statement::IfStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::SwitchStatement(_) => true,
            Statement::ReturnStatement(return_stmt) => return_stmt.argument.is_some(),
            Statement::ForStatement(for_stmt) => {
                // Avoid cases where we have for(var x;_;_) { ....
                for_stmt.init.is_none()
                    || for_stmt.init.as_ref().is_some_and(ForStatementInit::is_expression)
            }
            Statement::ForInStatement(for_in_stmt) => !for_in_stmt.left.may_have_side_effects(),
            Statement::LabeledStatement(labeled_stmt) => {
                Self::is_fusable_control_statement(&labeled_stmt.body)
            }
            Statement::BlockStatement(block) => {
                can_merge_block_stmt(block)
                    && block.body.first().map_or(false, Self::is_fusable_control_statement)
            }
            _ => false,
        }
    }

    fn fuse_into_one_statement(
        &mut self,
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let len = stmts.len();
        let mut expressions = ctx.ast.vec();

        for stmt in stmts.iter_mut().take(len - 1) {
            match stmt {
                Statement::ExpressionStatement(expr_stmt) => {
                    if let Expression::SequenceExpression(sequence_expr) = &mut expr_stmt.expression
                    {
                        expressions.extend(
                            sequence_expr
                                .expressions
                                .iter_mut()
                                .map(|e| ctx.ast.move_expression(e)),
                        );
                    } else {
                        expressions.push(ctx.ast.move_expression(&mut expr_stmt.expression));
                    }
                    *stmt = ctx.ast.statement_empty(SPAN);
                }
                _ => unreachable!(),
            }
        }

        let last = stmts.last_mut().unwrap();
        Self::fuse_expression_into_control_flow_statement(last, expressions, ctx);

        *stmts = ctx.ast.vec1(ctx.ast.move_statement(last));
        self.changed = true;
    }

    fn fuse_expression_into_control_flow_statement(
        stmt: &mut Statement<'a>,
        exprs: Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut exprs = exprs;
        let expr = match stmt {
            Statement::ExpressionStatement(expr_stmt) => &mut expr_stmt.expression,
            Statement::IfStatement(if_stmt) => &mut if_stmt.test,
            Statement::ThrowStatement(throw_stmt) => &mut throw_stmt.argument,
            Statement::SwitchStatement(switch_stmt) => &mut switch_stmt.discriminant,
            Statement::ReturnStatement(return_stmt) => return_stmt.argument.as_mut().unwrap(),
            Statement::ForStatement(for_stmt) => {
                if let Some(init) = for_stmt.init.as_mut() {
                    init.as_expression_mut().unwrap()
                } else {
                    for_stmt.init =
                        Some(ctx.ast.for_statement_init_expression(
                            ctx.ast.expression_sequence(SPAN, exprs),
                        ));
                    return;
                }
            }
            Statement::ForInStatement(for_stmt) => &mut for_stmt.right,
            Statement::LabeledStatement(labeled_stmt) => {
                Self::fuse_expression_into_control_flow_statement(
                    &mut labeled_stmt.body,
                    exprs,
                    ctx,
                );
                return;
            }
            Statement::BlockStatement(block) => {
                Self::fuse_expression_into_control_flow_statement(
                    block.body.first_mut().unwrap(),
                    exprs,
                    ctx,
                );
                return;
            }
            _ => {
                unreachable!("must match with `Self::is_fusable_control_statement`");
            }
        };
        exprs.push(ctx.ast.move_expression(expr));
        *expr = ctx.ast.expression_sequence(SPAN, exprs);
    }
}

fn can_merge_block_stmt(node: &BlockStatement) -> bool {
    return node.body.iter().all(can_merge_block_stmt_member);
}

fn can_merge_block_stmt_member(node: &Statement) -> bool {
    match node {
        Statement::LabeledStatement(label) => can_merge_block_stmt_member(&label.body),
        Statement::VariableDeclaration(var_decl) => {
            !matches!(var_decl.kind, VariableDeclarationKind::Const | VariableDeclarationKind::Let)
        }
        Statement::ClassDeclaration(_) | Statement::FunctionDeclaration(_) => false,
        _ => true,
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::StatementFusion::new();
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    fn fuse(before: &str, after: &str) {
        test(
            &("function F(){if(CONDITION){".to_string() + before + "}}"),
            &("function F(){if(CONDITION){".to_string() + after + "}}"),
        );
    }

    fn fuse_same(code: &str) {
        test_same(&("function F(){if(CONDITION){".to_string() + code + "}}"));
    }

    #[test]
    fn nothing_to_do() {
        fuse_same("");
        fuse_same("a");
        fuse_same("a()");
        fuse_same("if(a()){}");
    }

    #[test]
    fn fold_block_with_statements() {
        fuse("a;b;c", "a,b,c");
        fuse("a();b();c();", "a(),b(),c()");
        fuse("a(),b();c(),d()", "a(),b(),c(),d()");
        fuse("a();b(),c(),d()", "a(),b(),c(),d()");
        fuse("a(),b(),c();d()", "a(),b(),c(),d()");
    }

    #[test]
    fn fold_block_into_if() {
        fuse("a;b;c;if(x){}", "if(a,b,c,x){}");
        fuse("a;b;c;if(x,y){}else{}", "if(a,b,c,x,y){}else{}");
        fuse("a;b;c;if(x,y){}", "if(a,b,c,x,y){}");
        fuse("a;b;c;if(x,y,z){}", "if(a,b,c,x,y,z){}");

        // Can't fuse if there are statements after the IF.
        fuse_same("a();if(a()){}a()");
    }

    #[test]
    fn fold_block_return() {
        fuse("a;b;c;return x", "return a,b,c,x");
        fuse("a;b;c;return x+y", "return a,b,c,x+y");

        // DeadAssignmentElimination would have cleaned it up anyways.
        fuse_same("a;b;c;return x;a;b;c");
    }

    #[test]
    fn fold_block_throw() {
        fuse("a;b;c;throw x", "throw a,b,c,x");
        fuse("a;b;c;throw x+y", "throw a,b,c,x+y");
        fuse_same("a;b;c;throw x;a;b;c");
    }

    #[test]
    fn fold_switch() {
        fuse("a;b;c;switch(x){}", "switch(a,b,c,x){}");
    }

    #[test]
    fn fuse_into_for_in1() {
        fuse("a;b;c;for(x in y){}", "for(x in a,b,c,y){}");
    }

    #[test]
    fn fuse_into_for_in2() {
        // This test case causes a parse warning in ES5 strict out, but is a parse error in ES6+ out.
        // setAcceptedLanguage(CompilerOptions.LanguageMode.ECMASCRIPT5_STRICT);
        // set_expect_parse_warnings_in_this_test();
        fuse_same("a();for(var x = b() in y){}");
    }

    #[test]
    fn fuse_into_vanilla_for1() {
        fuse("a;b;c;for(;g;){}", "for(a,b,c;g;){}");
        fuse("a;b;c;for(d;g;){}", "for(a,b,c,d;g;){}");
        fuse("a;b;c;for(d,e;g;){}", "for(a,b,c,d,e;g;){}");
        fuse_same("a();for(var x;g;){}");
    }

    #[test]
    fn fuse_into_vanilla_for2() {
        fuse_same("a;b;c;for(var d;g;){}");
        fuse_same("a;b;c;for(let d;g;){}");
        fuse_same("a;b;c;for(const d = 5;g;){}");
    }

    #[test]
    fn fuse_into_label() {
        // fuse("a;b;c;label:for(x in y){}", "label:for(x in a,b,c,y){}");
        // fuse("a;b;c;label:for(;g;){}", "label:for(a,b,c;g;){}");
        // fuse("a;b;c;l1:l2:l3:for(;g;){}", "l1:l2:l3:for(a,b,c;g;){}");
        fuse_same("a;b;c;label:while(true){}");
    }

    #[test]
    fn fuse_into_block() {
        fuse("a;b;c;{d;e;f}", "{a,b,c,d,e,f}");
        fuse(
            "a;b; label: { if(q) break label; bar(); }",
            "label: { if(a,b,q) break label; bar(); }",
        );
        fuse_same("a;b;c;{var x;d;e;}");
        fuse_same("a;b;c;label:{break label;d;e;}");
    }

    #[test]
    fn no_fuse_into_while() {
        fuse_same("a;b;c;while(x){}");
    }

    #[test]
    fn no_fuse_into_do() {
        fuse_same("a;b;c;do{}while(x)");
    }

    #[test]
    fn no_fuse_into_block() {
        // Never fuse a statement into a block that contains let/const/class declarations, or you risk
        // colliding variable names. (unless the AST is normalized).
        fuse("a; {b;}", "{a,b;}");
        fuse("a; {b; var a = 1;}", "{a,b; var a = 1;}");
        fuse_same("a; { b; let a = 1; }");
        fuse_same("a; { b; const a = 1; }");
        fuse_same("a; { b; class a {} }");
        fuse_same("a; { b; function a() {} }");
        fuse_same("a; { b; const otherVariable = 1; }");

        // enable_normalize();
        // test(
        //     "function f(a) { if (COND) { a; { b; let a = 1; } } }",
        //     "function f(a) { if (COND) { { a,b; let a$jscomp$1 = 1; } } }",
        // );
        // test(
        //     "function f(a) { if (COND) { a; { b; let otherVariable = 1; } } }",
        //     "function f(a) { if (COND) {  { a,b; let otherVariable = 1; } } }",
        // );
    }

    #[test]
    fn no_global_scope_changes() {
        test_same("a,b,c");
    }

    #[test]
    fn no_function_block_changes() {
        test_same("function foo() { a,b,c }");
    }
}
