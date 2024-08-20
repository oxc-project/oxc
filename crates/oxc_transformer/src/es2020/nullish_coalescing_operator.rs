//! ES2020: Nullish Coalescing Operator
//!
//! This plugin transforms nullish coalescing operators (`??`) to a series of ternary expressions.
//!
//! > This plugin is included in `preset-env`, in ES2020
//!
//! ## Example
//!
//! Input:
//! ```js
//! var foo = object.foo ?? "default";
//! ```
//!
//! Output:
//! ```js
//! var _object$foo;
//! var foo =
//! (_object$foo = object.foo) !== null && _object$foo !== void 0
//!   ? _object$foo
//!   : "default";
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-nullish-coalescing-operator](https://babeljs.io/docs/babel-plugin-transform-nullish-coalescing-operator).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-nullish-coalescing-operator>
//! * Nullish coalescing TC39 proposal: <https://github.com/tc39-transfer/proposal-nullish-coalescing>

use std::cell::Cell;

use oxc_semantic::{ReferenceFlag, SymbolFlags};
use oxc_traverse::TraverseCtx;

use oxc_allocator::{CloneIn, Vec};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator};

use crate::context::Ctx;

pub struct NullishCoalescingOperator<'a> {
    _ctx: Ctx<'a>,
    var_declarations: std::vec::Vec<Vec<'a, VariableDeclarator<'a>>>,
}

impl<'a> NullishCoalescingOperator<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { _ctx: ctx, var_declarations: vec![] }
    }

    fn clone_identifier_reference(
        ident: &IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        let reference = ctx.symbols().get_reference(ident.reference_id.get().unwrap());
        let symbol_id = reference.symbol_id();
        let flag = reference.flag();
        ctx.create_reference_id(ident.span, ident.name.clone(), symbol_id, *flag)
    }

    fn clone_expression(expr: &Expression<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match expr {
            Expression::Identifier(ident) => ctx
                .ast
                .expression_from_identifier_reference(Self::clone_identifier_reference(ident, ctx)),
            _ => expr.clone_in(ctx.ast.allocator),
        }
    }

    pub fn transform_statements(
        &mut self,
        _statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.var_declarations.push(ctx.ast.vec());
    }

    pub fn transform_statements_on_exit(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(declarations) = self.var_declarations.pop() {
            if declarations.is_empty() {
                return;
            }
            let variable = ctx.ast.alloc_variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                declarations,
                false,
            );
            statements.insert(0, Statement::VariableDeclaration(variable));
        }
    }

    fn create_new_var_with_expression(
        &mut self,
        expr: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        // Add `var name` to scope
        let name = match expr {
            Expression::Identifier(ident) => ident.name.as_str(),
            // TODO: needs to port generateUidIdentifierBasedOnNode
            // https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L543-L545
            _ => "nullish_coalescing_operator",
        };
        let symbol_id =
            ctx.generate_uid_in_current_scope(name, SymbolFlags::FunctionScopedVariable);
        let symbol_name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));

        {
            // var _name;
            let binding_identifier = BindingIdentifier {
                span: SPAN,
                name: symbol_name.clone(),
                symbol_id: Cell::new(Some(symbol_id)),
            };
            let kind = VariableDeclarationKind::Var;
            let id = ctx.ast.binding_pattern_kind_from_binding_identifier(binding_identifier);
            let id = ctx.ast.binding_pattern(id, None::<TSTypeAnnotation<'_>>, false);
            self.var_declarations
                .last_mut()
                .unwrap()
                .push(ctx.ast.variable_declarator(SPAN, kind, id, None, false));
        };

        ctx.create_reference_id(SPAN, symbol_name, Some(symbol_id), ReferenceFlag::Read)
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // left ?? right
        if !matches!(expr, Expression::LogicalExpression(logical_expr) if logical_expr.operator == LogicalOperator::Coalesce)
        {
            return;
        }

        // Take ownership of the `LogicalExpression`
        let logical_expr = match ctx.ast.move_expression(expr) {
            Expression::LogicalExpression(logical_expr) => logical_expr.unbox(),
            _ => unreachable!(),
        };

        // skip creating extra reference when `left` is static
        let (reference, assignment) = if ctx.symbols().is_static(&logical_expr.left) {
            (Self::clone_expression(&logical_expr.left, ctx), logical_expr.left)
        } else {
            let ident = self.create_new_var_with_expression(&logical_expr.left, ctx);
            let left =
                AssignmentTarget::from(ctx.ast.simple_assignment_target_from_identifier_reference(
                    Self::clone_identifier_reference(&ident, ctx),
                ));
            let right = logical_expr.left;
            (
                ctx.ast.expression_from_identifier_reference(ident),
                ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, left, right),
            )
        };

        let op = BinaryOperator::StrictInequality;
        let null = ctx.ast.expression_null_literal(SPAN);
        let left = ctx.ast.expression_binary(SPAN, assignment, op, null);
        let right = ctx.ast.expression_binary(
            SPAN,
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { ctx.ast.copy(&reference) },
            op,
            ctx.ast.void_0(),
        );
        let test = ctx.ast.expression_logical(SPAN, left, LogicalOperator::And, right);

        *expr = ctx.ast.expression_conditional(SPAN, test, reference, logical_expr.right);
    }
}
