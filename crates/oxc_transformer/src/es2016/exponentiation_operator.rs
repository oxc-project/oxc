use std::cell::Cell;

use oxc_allocator::{CloneIn, Vec};
use oxc_ast::ast::*;
use oxc_semantic::{ReferenceFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator};
use oxc_traverse::TraverseCtx;

use crate::context::Ctx;

/// ES2016: Exponentiation Operator
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-exponentiation-operator>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-exponentiation-operator>
/// * <https://github.com/babel/babel/blob/main/packages/babel-helper-builder-binary-assignment-operator-visitor>
pub struct ExponentiationOperator<'a> {
    _ctx: Ctx<'a>,
    var_declarations: std::vec::Vec<Vec<'a, VariableDeclarator<'a>>>,
}

#[derive(Debug)]
struct Exploded<'a> {
    reference: AssignmentTarget<'a>,
    uid: Expression<'a>,
}

impl<'a> ExponentiationOperator<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { _ctx: ctx, var_declarations: vec![] }
    }

    fn clone_identifier_reference(
        ident: &IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        let reference = ctx.symbols().get_reference(ident.reference_id.get().unwrap());
        let symbol_id = reference.symbol_id();
        let flags = reference.flags();
        ctx.create_reference_id(ident.span, ident.name.clone(), symbol_id, flags)
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

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // left ** right
        if let Expression::BinaryExpression(binary_expr) = expr {
            if binary_expr.operator == BinaryOperator::Exponential {
                let left = ctx.ast.move_expression(&mut binary_expr.left);
                let right = ctx.ast.move_expression(&mut binary_expr.right);
                *expr = Self::math_pow(left, right, ctx);
            }
        }

        // left **= right
        if let Expression::AssignmentExpression(assign_expr) = expr {
            if assign_expr.operator == AssignmentOperator::Exponential {
                let mut nodes = ctx.ast.vec();
                let Some(Exploded { reference, uid }) =
                    self.explode(&mut assign_expr.left, &mut nodes, ctx)
                else {
                    return;
                };
                let right = ctx.ast.move_expression(&mut assign_expr.right);
                let right = Self::math_pow(uid, right, ctx);
                let assign_expr = ctx.ast.expression_assignment(
                    SPAN,
                    AssignmentOperator::Assign,
                    reference,
                    right,
                );
                nodes.push(assign_expr);
                *expr = ctx.ast.expression_sequence(SPAN, nodes);
            }
        }
    }

    /// `left ** right` -> `Math.pow(left, right)`
    fn math_pow(
        left: Expression<'a>,
        right: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let ident_math =
            ctx.create_reference_id(SPAN, ctx.ast.atom("Math"), None, ReferenceFlags::Read);
        let object = ctx.ast.expression_from_identifier_reference(ident_math);
        let property = ctx.ast.identifier_name(SPAN, "pow");
        let callee =
            Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false));
        let mut arguments = ctx.ast.vec_with_capacity(2);
        arguments.push(Argument::from(left));
        arguments.push(Argument::from(right));
        ctx.ast.expression_call(
            SPAN,
            callee,
            None::<TSTypeParameterInstantiation<'_>>,
            arguments,
            false,
        )
    }

    /// Change `lhs **= 2` to `var temp; temp = lhs, lhs = Math.pow(temp, 2);`.
    /// If the lhs is a member expression `obj.ref` or `obj[ref]`, assign them to a temporary variable so side-effects are not computed twice.
    /// For `obj.ref`, change it to `var _obj; _obj = obj, _obj["ref"] = Math.pow(_obj["ref"], 2)`.
    /// For `obj[ref]`, change it to `var _obj, _ref; _obj = obj, _ref = ref, _obj[_ref] = Math.pow(_obj[_ref], 2);`.
    fn explode(
        &mut self,
        node: &mut AssignmentTarget<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Exploded<'a>> {
        let node = node.as_simple_assignment_target_mut()?;
        let obj = self.get_obj_ref(node, nodes, ctx)?;
        let (reference, uid) = match node {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let reference = AssignmentTarget::AssignmentTargetIdentifier(
                    ctx.ast.alloc(Self::clone_identifier_reference(ident.as_ref(), ctx)),
                );
                (reference, obj)
            }
            match_member_expression!(SimpleAssignmentTarget) => {
                let member_expr = node.to_member_expression_mut();
                let computed = member_expr.is_computed();
                let prop = self.get_prop_ref(member_expr, nodes, ctx)?;
                let optional = false;
                let obj_clone = Self::clone_expression(&obj, ctx);
                let (reference, uid) = match &prop {
                    Expression::Identifier(ident) if !computed => {
                        let ident = IdentifierName::new(SPAN, ident.name.clone());
                        (
                            // TODO:
                            // Both of these are the same, but it's in order to avoid after cloning without reference_id.
                            // Related: https://github.com/oxc-project/oxc/issues/4804
                            ctx.ast.member_expression_static(
                                SPAN,
                                obj_clone,
                                ident.clone(),
                                optional,
                            ),
                            ctx.ast.member_expression_static(SPAN, obj, ident, optional),
                        )
                    }
                    _ => {
                        let prop_clone = Self::clone_expression(&prop, ctx);
                        (
                            ctx.ast
                                .member_expression_computed(SPAN, obj_clone, prop_clone, optional),
                            ctx.ast.member_expression_computed(SPAN, obj, prop, optional),
                        )
                    }
                };
                (
                    AssignmentTarget::from(
                        ctx.ast.simple_assignment_target_member_expression(reference),
                    ),
                    Expression::from(uid),
                )
            }
            _ => return None,
        };
        Some(Exploded { reference, uid })
    }

    /// Make sure side-effects of evaluating `obj` of `obj.ref` and `obj[ref]` only happen once.
    fn get_obj_ref(
        &mut self,
        node: &mut SimpleAssignmentTarget<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let reference = match node {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                if ident
                    .reference_id
                    .get()
                    .is_some_and(|reference_id| ctx.symbols().has_binding(reference_id))
                {
                    // this variable is declared in scope so we can be 100% sure
                    // that evaluating it multiple times won't trigger a getter
                    // or something else
                    return Some(ctx.ast.expression_from_identifier_reference(
                        Self::clone_identifier_reference(ident, ctx),
                    ));
                }
                // could possibly trigger a getter so we need to only evaluate it once
                ctx.ast.expression_from_identifier_reference(Self::clone_identifier_reference(
                    ident, ctx,
                ))
            }
            match_member_expression!(SimpleAssignmentTarget) => {
                let expr = match node {
                    SimpleAssignmentTarget::ComputedMemberExpression(e) => &mut e.object,
                    SimpleAssignmentTarget::StaticMemberExpression(e) => &mut e.object,
                    SimpleAssignmentTarget::PrivateFieldExpression(e) => &mut e.object,
                    _ => unreachable!(),
                };
                let expr = ctx.ast.move_expression(expr);
                // the object reference that we need to save is locally declared
                // so as per the previous comment we can be 100% sure evaluating
                // it multiple times will be safe
                // Super cannot be directly assigned so lets return it also
                if matches!(expr, Expression::Super(_))
                    || matches!(&expr, Expression::Identifier(ident) if ident
                        .reference_id
                        .get()
                        .is_some_and(|reference_id| ctx.symbols().has_binding(reference_id)))
                {
                    return Some(expr);
                }

                expr
            }
            _ => return None,
        };
        Some(self.add_new_reference(reference, nodes, ctx))
    }

    /// Make sure side-effects of evaluating `ref` of `obj.ref` and `obj[ref]` only happen once.
    fn get_prop_ref(
        &mut self,
        node: &mut MemberExpression<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let expr = match node {
            MemberExpression::ComputedMemberExpression(expr) => {
                let expr = ctx.ast.move_expression(&mut expr.expression);
                if expr.is_literal() {
                    return Some(expr);
                }
                expr
            }
            MemberExpression::StaticMemberExpression(expr) => {
                return Some(ctx.ast.expression_string_literal(SPAN, expr.property.name.clone()));
            }
            MemberExpression::PrivateFieldExpression(_) => {
                // From babel: "We can't generate property ref for private name, please install `@babel/plugin-transform-class-properties`"
                return None;
            }
        };
        Some(self.add_new_reference(expr, nodes, ctx))
    }

    fn add_new_reference(
        &mut self,
        expr: Expression<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let name = match expr {
            Expression::Identifier(ref ident) => ident.name.clone().as_str(),
            _ => "ref",
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
        }

        let ident =
            ctx.create_reference_id(SPAN, symbol_name, Some(symbol_id), ReferenceFlags::Read);

        // let ident = self.create_new_var_with_expression(&expr);
        // Add new reference `_name = name` to nodes
        let left = ctx.ast.simple_assignment_target_from_identifier_reference(
            Self::clone_identifier_reference(&ident, ctx),
        );
        let op = AssignmentOperator::Assign;
        nodes.push(ctx.ast.expression_assignment(SPAN, op, AssignmentTarget::from(left), expr));
        ctx.ast.expression_from_identifier_reference(ident)
    }
}
