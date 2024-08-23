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

use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, SymbolFlags};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

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
}

impl<'a> Traverse<'a> for NullishCoalescingOperator<'a> {
    fn enter_statements(&mut self, _stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.var_declarations.push(ctx.ast.vec());
    }

    fn exit_statements(
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

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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
        if ctx.is_static(&logical_expr.left) {
            *expr = Self::create_conditional_expression(
                Self::clone_expression(&logical_expr.left, ctx),
                logical_expr.left,
                logical_expr.right,
                ctx,
            );
            return;
        }

        // ctx.ancestor(1) is AssignmentPattern
        // ctx.ancestor(2) is BindingPattern;
        // ctx.ancestor(3) is FormalParameter
        let is_parent_formal_parameter = ctx
            .ancestor(3)
            .is_some_and(|ancestor| matches!(ancestor, Ancestor::FormalParameterPattern(_)));

        let current_scope_id = if is_parent_formal_parameter {
            ctx.create_child_scope_of_current(ScopeFlags::Arrow | ScopeFlags::Function)
        } else {
            ctx.current_scope_id()
        };

        let (id, ident) =
            Self::create_new_var_with_expression(&logical_expr.left, current_scope_id, ctx);

        let left =
            AssignmentTarget::from(ctx.ast.simple_assignment_target_from_identifier_reference(
                ctx.clone_identifier_reference(&ident, ReferenceFlags::Write),
            ));

        let reference = ctx.ast.expression_from_identifier_reference(ident);
        let assignment = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            left,
            logical_expr.left,
        );

        let mut new_expr =
            Self::create_conditional_expression(reference, assignment, logical_expr.right, ctx);

        if is_parent_formal_parameter {
            // Replace `function (a, x = a.b ?? c) {}` to `function (a, x = (() => a.b ?? c)() ){}`
            // so the temporary variable can be injected in correct scope
            let param = ctx.ast.formal_parameter(SPAN, ctx.ast.vec(), id, None, false, false);
            let params = ctx.ast.formal_parameters(
                SPAN,
                FormalParameterKind::ArrowFormalParameters,
                ctx.ast.vec1(param),
                None::<BindingRestElement>,
            );
            let body = ctx.ast.function_body(
                SPAN,
                ctx.ast.vec(),
                ctx.ast.vec1(ctx.ast.statement_expression(SPAN, new_expr)),
            );
            let type_parameters = None::<TSTypeParameterDeclaration>;
            let type_annotation = None::<TSTypeAnnotation>;
            let arrow_function = ctx.ast.arrow_function_expression(
                SPAN,
                true,
                false,
                type_parameters,
                params,
                type_annotation,
                body,
            );
            arrow_function.scope_id.set(Some(current_scope_id));
            let arrow_function = ctx.ast.expression_from_arrow_function(arrow_function);
            // `(x) => x;` -> `((x) => x)();`
            new_expr = ctx.ast.expression_call(
                SPAN,
                arrow_function,
                None::<TSTypeParameterInstantiation>,
                ctx.ast.vec(),
                false,
            );
        } else {
            let kind = VariableDeclarationKind::Var;
            self.var_declarations
                .last_mut()
                .unwrap()
                .push(ctx.ast.variable_declarator(SPAN, kind, id, None, false));
        }

        *expr = new_expr;
    }
}

impl<'a> NullishCoalescingOperator<'a> {
    fn clone_expression(expr: &Expression<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match expr {
            Expression::Identifier(ident) => ctx.ast.expression_from_identifier_reference(
                ctx.clone_identifier_reference(ident, ReferenceFlags::Read),
            ),
            _ => expr.clone_in(ctx.ast.allocator),
        }
    }

    fn create_new_var_with_expression(
        expr: &Expression<'a>,
        current_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> (BindingPattern<'a>, IdentifierReference<'a>) {
        // Add `var name` to scope
        let symbol_id = ctx.generate_uid_based_on_node(
            expr,
            current_scope_id,
            SymbolFlags::FunctionScopedVariable,
        );
        let symbol_name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));

        // var _name;
        let binding_identifier = BindingIdentifier {
            span: SPAN,
            name: symbol_name.clone(),
            symbol_id: Cell::new(Some(symbol_id)),
        };
        let id = ctx.ast.binding_pattern_kind_from_binding_identifier(binding_identifier);
        let id = ctx.ast.binding_pattern(id, None::<TSTypeAnnotation<'_>>, false);
        let reference =
            ctx.create_reference_id(SPAN, symbol_name, Some(symbol_id), ReferenceFlags::Read);

        (id, reference)
    }

    /// Create a conditional expression
    ///
    /// ```js
    /// // Input
    /// bar ?? "qux"
    ///
    /// // Output
    /// qux = bar !== null && bar !== void 0 ? bar : "qux"
    /// //    ^^^ assignment  ^^^ reference           ^^^ default
    /// ```
    ///
    /// reference and assignment are the same in this case, but they can be different
    fn create_conditional_expression(
        reference: Expression<'a>,
        assignment: Expression<'a>,
        default: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let op = BinaryOperator::StrictInequality;
        let null = ctx.ast.expression_null_literal(SPAN);
        let left = ctx.ast.expression_binary(SPAN, assignment, op, null);
        let right = ctx.ast.expression_binary(
            SPAN,
            Self::clone_expression(&reference, ctx),
            op,
            ctx.ast.void_0(),
        );
        let test = ctx.ast.expression_logical(SPAN, left, LogicalOperator::And, right);

        ctx.ast.expression_conditional(SPAN, test, reference, default)
    }
}
