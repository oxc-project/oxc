use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::{BoundIdentifier, TraverseCtx};

use super::TypeScript;

impl<'a> TypeScript<'a, '_> {
    /// Transform class elements and convert constructor parameters to `this` assignments.
    ///
    /// Input:
    /// ```ts
    /// class C {
    ///   constructor(public x, private y) {}
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// class C {
    ///  constructor(x, y) {
    ///   this.x = x;
    ///   this.y = y;
    /// }
    /// ```
    pub(super) fn transform_class(class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut constructor = None;
        for element in &mut class.body.body {
            if let ClassElement::MethodDefinition(method) = element {
                if method.kind == MethodDefinitionKind::Constructor {
                    constructor = Some(&mut method.value);
                    break;
                }
            }
        }

        if let Some(constructor) = constructor {
            let params = &constructor.params.items;

            // Transform constructor parameters that include modifier to `this` assignments.
            let assignments = params
                .iter()
                .filter_map(|param| {
                    if param.has_modifier() {
                        param.pattern.get_binding_identifier().map(|id| {
                            Self::create_this_property_assignment(
                                id.span,
                                id.name,
                                BoundIdentifier::from_binding_ident(id).create_read_expression(ctx),
                                ctx,
                            )
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if let Some(constructor_body) = constructor.body.as_mut() {
                // Find the position of the super call in the constructor body,
                // don't need to care about nested super call because `TypeScript`
                // doesn't allow it.
                let super_call_position = constructor_body
                    .statements
                    .iter()
                    .position(|stmt| {
                        matches!(stmt, Statement::ExpressionStatement(stmt)
                        if stmt.expression.is_super_call_expression())
                    })
                    .map_or(0, |pos| pos + 1);

                // Insert the assignments after the super call
                constructor_body
                    .statements
                    .splice(super_call_position..super_call_position, assignments);
            }
        }
    }

    // Creates `this.name = name`
    fn create_this_property_assignment(
        span: Span,
        name: Atom<'a>,
        value: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        ctx.ast.statement_expression(
            SPAN,
            ctx.ast.expression_assignment(
                SPAN,
                AssignmentOperator::Assign,
                AssignmentTarget::StaticMemberExpression(ctx.ast.alloc_static_member_expression(
                    SPAN,
                    ctx.ast.expression_this(SPAN),
                    ctx.ast.identifier_name(span, name),
                    false,
                )),
                value,
            ),
        )
    }
}
