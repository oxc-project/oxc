use super::PeepholeOptimizations;

use crate::TraverseCtx;
use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::IsLiteralValue;
use oxc_span::{GetSpan, SPAN};

impl<'a> PeepholeOptimizations {
    /// Simplifies destructuring assignments by transforming array patterns into a sequence of
    /// variable declarations, whenever possible. This function modifies the input declarations
    /// and returns whether any changes were made.
    pub fn try_minimize_variable_declarator(
        declarations: &mut ArenaVec<'a, VariableDeclarator<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut i = declarations.len();
        while i > 0 {
            i -= 1;

            let Some(last) = declarations.get_mut(i) else {
                continue;
            };
            let Some((new_id, new_init)) = Self::simplify_array_destruction_assignment(last, ctx)
            else {
                continue;
            };

            if new_init.is_some()
                && let Some(new_id) = new_id
            {
                let new_decl = VariableDeclarator::new(
                    new_id.span(),
                    new_id,
                    None,
                    new_init,
                    last.definite,
                    ctx,
                );
                if Self::is_empty_array_destruction_assignment(last) {
                    ctx.replace_variable_declarator(last, new_decl);
                    i += 1;
                } else {
                    declarations.insert(i, new_decl);
                    ctx.notice_change();
                    i += 2;
                }
            } else if Self::is_empty_array_destruction_assignment(last) {
                ctx.drop_variable_declarator(&declarations.remove(i));
            }
        }
    }

    /// Determines whether an array destruction assignment can be simplified.
    /// - `let [x, y] = [1, 2];` -> true
    /// - `let [x, y] = [...arr];` -> false
    fn can_simplify_array_to_array_destruction_assignment(
        id_kind: &ArrayPattern<'a>,
        init_expr: &ArrayExpression<'a>,
        ctx: &crate::traverse_context::MinifierTraverseCtx<'a>,
    ) -> bool {
        // if left side of assignment is empty do not process it
        if id_kind.is_empty() {
            return false;
        }

        let init_len = init_expr.elements.len();
        // [???] = [] or [...rest] = [??]
        if init_len == 0 || (id_kind.rest.is_some() && id_kind.elements.is_empty()) {
            return true;
        }

        let first_init = init_expr.elements.first();

        // check if the first init is not spread when rest is present without elements
        // [] = [...rest] | [a, ...rest] = [...rest]
        if first_init.is_some_and(ArrayExpressionElement::is_spread)
            && id_kind.rest.is_none()
            && !id_kind.elements.is_empty()
        {
            return false;
        }

        // check for `[a = b] = [c]`
        if init_len == 1 {
            if first_init.is_some_and(|expr| !expr.is_literal_value(false, ctx))
                && id_kind
                    .elements
                    .first()
                    .is_some_and(|e| e.as_ref().is_none_or(BindingPattern::is_assignment_pattern))
            {
                return false;
            }
        } else if !init_expr
            .elements
            .iter()
            .all(|expr| expr.is_spread() || expr.is_literal_value(false, ctx))
        {
            return false;
        }

        true
    }

    fn simplify_array_destruction_assignment(
        decl: &mut VariableDeclarator<'a>,
        ctx: &mut crate::traverse_context::MinifierTraverseCtx<'a>,
    ) -> Option<(Option<BindingPattern<'a>>, Option<Expression<'a>>)> {
        let BindingPattern::ArrayPattern(id_pattern) = &mut decl.id else {
            return None;
        };
        let Some(Expression::ArrayExpression(init_expr)) = &mut decl.init else {
            return None;
        };
        if !Self::can_simplify_array_to_array_destruction_assignment(id_pattern, init_expr, ctx) {
            return None;
        }

        if id_pattern.elements.is_empty() {
            let Some(mut rest) = id_pattern.rest.take() else {
                return None;
            };
            return Some((
                Some(rest.argument.take_in(ctx)),
                Some(Expression::ArrayExpression(init_expr.take_in_box(ctx))),
            ));
        }

        let init_item = match init_expr.elements.first() {
            None => Expression::new_void_0(SPAN, ctx),
            Some(ArrayExpressionElement::Elision(_)) => {
                init_expr.elements.remove(0);
                Expression::new_void_0(SPAN, ctx)
            }
            Some(ArrayExpressionElement::SpreadElement(_)) => return None,
            Some(_) => init_expr.elements.remove(0).into_expression(),
        };
        let id_item = id_pattern.elements.remove(0);

        match id_item {
            // `[a = b] = [??]`
            Some(BindingPattern::AssignmentPattern(mut pattern)) => {
                if init_item.is_literal_value(false, ctx) {
                    // if value is determined, `[a = b] = [c]` => `a = c` or `a = b`
                    if init_item.is_void_0() {
                        // `[a = b] = [undefined]` => `a = b`
                        ctx.drop_expression(&init_item);
                        Some((Some(pattern.left.take_in(ctx)), Some(pattern.right.take_in(ctx))))
                    } else {
                        // `[a = b] = [c]` => `a = c`
                        ctx.drop_expression(&pattern.right);
                        Some((Some(pattern.left.take_in(ctx)), Some(init_item)))
                    }
                } else {
                    // `[a = b] = [c]` where c is undetermined => `[a = b] = [c]`
                    Some((
                        Some(BindingPattern::new_array_pattern(
                            decl.span,
                            [Some(BindingPattern::AssignmentPattern(pattern))],
                            None,
                            ctx,
                        )),
                        Some(Expression::new_array_expression(
                            init_item.span(),
                            [ArrayExpressionElement::from(init_item)],
                            ctx,
                        )),
                    ))
                }
            }
            // `[a, b] = [c, d]` => `a = c, b = d`
            Some(id) => Some((Some(id), Some(init_item))),
            // `[] = [??]` => `[] = [??]`
            None => {
                if init_item.is_literal_value(false, ctx) {
                    ctx.drop_expression(&init_item);
                    Some((None, None))
                } else {
                    Some((
                        Some(BindingPattern::new_array_pattern(
                            decl.span,
                            ArenaVec::new_in(ctx),
                            None,
                            ctx,
                        )),
                        Some(Expression::new_array_expression(
                            init_item.span(),
                            [ArrayExpressionElement::from(init_item)],
                            ctx,
                        )),
                    ))
                }
            }
        }
    }

    fn is_empty_array_destruction_assignment(decl: &VariableDeclarator<'a>) -> bool {
        let BindingPattern::ArrayPattern(id_pattern) = &decl.id else {
            return false;
        };
        let Some(Expression::ArrayExpression(array_expr)) = &decl.init else {
            return false;
        };
        id_pattern.elements.is_empty()
            && id_pattern.rest.is_none()
            && array_expr.elements.is_empty()
    }
}
