//! Loop-body scoping and decorator key names.

use std::mem;

use rustc_hash::FxHashMap;

use oxc_allocator::{Address, CloneIn, GetAddress, GetAllocator, TakeIn};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::{reference::ReferenceFlags, scope::ScopeFlags};
use oxc_traverse::Ancestor;

use crate::{context::TraverseCtx, state::TransformState};

use super::StandardDecoratorState;

impl<'a> StandardDecoratorState<'a> {
    /// Give an unbraced loop body a scope for per-evaluation decorator state.
    ///
    /// Runs on every statement, so the cheap checks are inlined into the caller.
    #[inline]
    pub(crate) fn wrap_decorated_loop_body(
        &mut self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.enabled
            && !matches!(statement, Statement::BlockStatement(_))
            && matches!(
                ctx.parent(),
                Ancestor::ForStatementBody(_)
                    | Ancestor::ForInStatementBody(_)
                    | Ancestor::ForOfStatementBody(_)
                    | Ancestor::WhileStatementBody(_)
                    | Ancestor::DoWhileStatementBody(_)
            )
        {
            self.wrap_loop_body_if_decorated(statement, ctx);
        }
    }

    #[inline(never)]
    fn wrap_loop_body_if_decorated(
        &mut self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        struct DecoratedClassFinder<'s, 'a> {
            found: bool,
            loop_classes: &'s mut FxHashMap<Address, bool>,
            state: &'s TransformState<'a>,
        }
        impl<'a> Visit<'a> for DecoratedClassFinder<'_, 'a> {
            fn visit_statement(&mut self, statement: &Statement<'a>) {
                let loop_address = matches!(
                    statement,
                    Statement::ForStatement(_)
                        | Statement::ForInStatement(_)
                        | Statement::ForOfStatement(_)
                        | Statement::WhileStatement(_)
                        | Statement::DoWhileStatement(_)
                )
                .then(|| statement.address());
                if let Some(address) = loop_address
                    && let Some(&found) = self.loop_classes.get(&address)
                {
                    self.found |= found;
                    return;
                }

                let found_before = mem::take(&mut self.found);
                oxc_ast_visit::walk::walk_statement(self, statement);
                if let Some(address) = loop_address {
                    // Reuse both positive and negative results for nested unbraced loops.
                    self.loop_classes.insert(address, self.found);
                }
                self.found |= found_before;
            }

            fn visit_class(&mut self, class: &Class<'a>) {
                self.found |=
                    self.state.class_lowering_before_entry(class).handles_standard_decorators();
                if !self.found {
                    oxc_ast_visit::walk::walk_class(self, class);
                }
            }
        }
        use oxc_ast_visit::Visit;

        let found = {
            let mut finder = DecoratedClassFinder {
                found: false,
                loop_classes: &mut self.loop_classes,
                state: &ctx.state,
            };
            finder.visit_statement(statement);
            finder.found
        };
        if found {
            let scope_id = ctx.insert_scope_below_statement(statement, ScopeFlags::empty());
            let body = statement.take_in(ctx);
            *statement = Statement::new_block_statement_with_scope_id(SPAN, [body], scope_id, ctx);
        }
    }

    /// Expression for element name passed to `__decorateElement`.
    pub(crate) fn decorator_key_expr(
        key: &PropertyKey<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match key {
            PropertyKey::PrivateIdentifier(ident) => {
                let name = Str::from_strs_array_in(["#", &ident.name], ctx);
                Expression::new_string_literal(SPAN, name, None, ctx)
            }
            PropertyKey::StaticIdentifier(ident) => {
                Expression::new_string_literal(SPAN, ident.name, None, ctx)
            }
            PropertyKey::Identifier(ident) => {
                let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id();
                ctx.create_ident_expr(SPAN, ident.name, symbol_id, ReferenceFlags::Read)
            }
            PropertyKey::AssignmentExpression(assign) => {
                let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left else {
                    unreachable!("memoized computed key is always assignment to temp var");
                };
                let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id();
                ctx.create_ident_expr(SPAN, ident.name, symbol_id, ReferenceFlags::Read)
            }
            PropertyKey::StringLiteral(lit) => {
                Expression::StringLiteral(lit.clone_in(ctx.ast.allocator()))
            }
            PropertyKey::NumericLiteral(lit) => {
                Expression::NumericLiteral(lit.clone_in(ctx.ast.allocator()))
            }
            key => key.to_expression().clone_in(ctx.ast.allocator()),
        }
    }
}
