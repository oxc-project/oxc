use oxc_allocator::Allocator;
use oxc_ast::ast::{Function, Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::{Ancestor, Traverse, TraverseCtx, traverse_mut};

#[test]
fn annex_b_if_branch_callback_scopes() {
    #[derive(Default)]
    struct CheckScopes {
        branch_callbacks: usize,
        enclosing_callbacks: usize,
    }

    impl CheckScopes {
        fn check_function(&mut self, function: &Function<'_>, ctx: &TraverseCtx<'_, ()>) {
            if !matches!(
                ctx.parent(),
                Ancestor::IfStatementConsequent(_) | Ancestor::IfStatementAlternate(_)
            ) {
                return;
            }
            let body_scope_id = function.scope_id.get().unwrap();
            let implicit_scope_id = ctx.scoping().scope_parent_id(body_scope_id).unwrap();
            assert_eq!(ctx.current_scope_id(), implicit_scope_id);
            assert_eq!(ctx.current_block_scope_id(), implicit_scope_id);
            let id = function.id.as_ref().unwrap();
            assert_eq!(ctx.scoping().get_binding(implicit_scope_id, id.name), id.symbol_id.get());
            self.branch_callbacks += 1;
        }

        fn check_statement(&mut self, stmt: &Statement<'_>, ctx: &TraverseCtx<'_, ()>) {
            if let Statement::FunctionDeclaration(function) = stmt {
                self.check_function(function, ctx);
            } else if let Statement::ReturnStatement(_) = stmt {
                assert!(ctx.current_scope_flags().is_function());
                assert_eq!(ctx.current_scope_id(), ctx.current_hoist_scope_id());
                self.enclosing_callbacks += 1;
            }
        }
    }

    impl<'a> Traverse<'a, ()> for CheckScopes {
        fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
            self.check_statement(stmt, ctx);
        }

        fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
            self.check_statement(stmt, ctx);
        }

        fn enter_function(&mut self, function: &mut Function<'a>, ctx: &mut TraverseCtx<'a, ()>) {
            self.check_function(function, ctx);
        }

        fn exit_function(&mut self, function: &mut Function<'a>, ctx: &mut TraverseCtx<'a, ()>) {
            self.check_function(function, ctx);
        }
    }

    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "function outer(flag) { let f; if (flag) function f() {} else function f() {} return f; }",
        SourceType::script(),
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    let mut program = parsed.program;
    let built = SemanticBuilder::new().build(&program);
    assert!(built.diagnostics.is_empty());
    let scoping = built.semantic.into_scoping();
    let mut visitor = CheckScopes::default();
    traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
    assert_eq!(visitor.branch_callbacks, 8);
    assert_eq!(visitor.enclosing_callbacks, 2);
}
