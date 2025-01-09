use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_span::SPAN;
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::CompressorPass;

/// Statement Fusion
///
/// Tries to fuse all the statements in a block into a one statement by using COMMAs or statements.
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/StatementFusion.java>
pub struct StatementFusion {
    pub(crate) changed: bool,
}

impl<'a> CompressorPass<'a> for StatementFusion {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        self.changed = false;
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for StatementFusion {
    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.fuse_statements(stmts, ctx);
    }
}

impl<'a> StatementFusion {
    pub fn new() -> Self {
        Self { changed: false }
    }

    fn fuse_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let len = stmts.len();

        if len <= 1 {
            return;
        }

        let mut end = None;

        // TODO: make this cleaner and faster. Find the groups of expressions i..j and fusable j+1
        // statement.
        for i in (0..stmts.len()).rev() {
            match end {
                None => {
                    if Self::is_fusable_control_statement(&stmts[i]) {
                        end = Some(i);
                    }
                }
                Some(j) => {
                    let is_expr_stmt = matches!(&stmts[i], Statement::ExpressionStatement(_));
                    if i == 0 && is_expr_stmt {
                        Self::fuse_into_one_statement(&mut stmts[0..=j], ctx);
                        self.changed = true;
                    } else if !is_expr_stmt {
                        if j - i > 1 {
                            Self::fuse_into_one_statement(&mut stmts[i + 1..=j], ctx);
                            self.changed = true;
                        }
                        if Self::is_fusable_control_statement(&stmts[i]) {
                            end = Some(i);
                        } else {
                            end = None;
                        }
                    }
                }
            }
        }

        if self.changed {
            stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        }
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

    fn fuse_into_one_statement(stmts: &mut [Statement<'a>], ctx: &mut TraverseCtx<'a>) {
        let mut exprs = ctx.ast.vec();

        let len = stmts.len();

        for stmt in &mut stmts[0..len - 1] {
            if let Statement::ExpressionStatement(expr_stmt) = stmt {
                if let Expression::SequenceExpression(sequence_expr) = &mut expr_stmt.expression {
                    exprs.extend(
                        sequence_expr.expressions.iter_mut().map(|e| ctx.ast.move_expression(e)),
                    );
                } else {
                    exprs.push(ctx.ast.move_expression(&mut expr_stmt.expression));
                }
                *stmt = ctx.ast.statement_empty(SPAN);
            } else {
                break;
            }
        }

        let last = &mut stmts[len - 1];
        Self::fuse_expression_into_control_flow_statement(last, exprs, ctx);
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
                        Some(ForStatementInit::from(ctx.ast.expression_sequence(SPAN, exprs)));
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
    node.body.iter().all(can_merge_block_stmt_member)
}

fn can_merge_block_stmt_member(node: &Statement) -> bool {
    match node {
        Statement::LabeledStatement(label) => can_merge_block_stmt_member(&label.body),
        Statement::VariableDeclaration(var_decl) => var_decl.kind.is_var(),
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
        fuse("a();if(a()){}a()", "if(a(), a()){}a()");
    }

    #[test]
    fn fold_block_return() {
        fuse("a;b;c;return x", "return a,b,c,x");
        fuse("a;b;c;return x+y", "return a,b,c,x+y");
        fuse("a;b;c;return x;a;b;c", "return a,b,c,x;a,b,c");
    }

    #[test]
    fn fold_block_throw() {
        fuse("a;b;c;throw x", "throw a,b,c,x");
        fuse("a;b;c;throw x+y", "throw a,b,c,x+y");
        fuse("a;b;c;throw x;a;b;c", "throw a,b,c,x;a,b,c");
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
        fuse("a;b;c;for(var d;g;){}", "a,b,c;for(var d;g;){}");
        fuse("a;b;c;for(let d;g;){}", "a,b,c;for(let d;g;){}");
        fuse("a;b;c;for(const d = 5;g;){}", "a,b,c;for(const d = 5;g;){}");
    }

    #[test]
    fn fuse_into_label() {
        fuse("a;b;c;label:for(x in y){}", "label:for(x in a,b,c,y){}");
        fuse("a;b;c;label:for(;g;){}", "label:for(a,b,c;g;){}");
        fuse("a;b;c;l1:l2:l3:for(;g;){}", "l1:l2:l3:for(a,b,c;g;){}");
        fuse("a;b;c;label:while(true){}", "label:for(a,b,c;true;){}");
    }

    #[test]
    fn fuse_into_block() {
        fuse("a;b;c;{d;e;f}", "{a,b,c,d,e,f}");
        fuse(
            "a;b; label: { if(q) break label; bar(); }",
            "label: { if(a,b,q) break label; bar(); }",
        );
        fuse("a;b;c;{var x;d;e;}", "a,b,c;{var x;d,e;}");
        fuse("a;b;c;label:{break label;d;e;}", "a,b,c;label:{break label;d,e;}");
    }

    #[test]
    fn fuse_into_switch_cases() {
        fuse("switch (_) { case _: a; return b }", "switch (_) { case _: return a, b }");
    }

    #[test]
    fn no_fuse_into_while() {
        fuse("a;b;c;while(x){}", "for(a,b,c;x;){}");
    }

    #[test]
    fn no_fuse_into_do() {
        fuse("a;b;c;do{}while(x)", "a,b,c;do{}while(x)");
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

        // test(
        // "function f(a) { if (COND) { a; { b; let a = 1; } } }",
        // "function f(a) { if (COND) { { a,b; let a$jscomp$1 = 1; } } }",
        // );
        // test(
        // "function f(a) { if (COND) { a; { b; let otherVariable = 1; } } }",
        // "function f(a) { if (COND) {  { a,b; let otherVariable = 1; } } }",
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
