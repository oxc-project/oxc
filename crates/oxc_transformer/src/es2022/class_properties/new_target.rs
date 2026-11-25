//! ES2022: Class Properties
//! Replace `new.target` in class field initializers and static blocks.
//!
//! `new.target` is `undefined` in a field initializer or static block. Once the transform moves the
//! code out of the class, `new.target` would instead be the constructor (instance fields) or the
//! enclosing function's `new.target` (static fields and blocks). So replace it with `void 0` first.

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_syntax::scope::ScopeFlags;

use crate::context::TraverseCtx;

/// Replace `new.target` in a field initializer with `void 0`.
pub(super) fn replace_new_target_in_expression<'a>(
    expr: &mut Expression<'a>,
    ctx: &TraverseCtx<'a>,
) {
    if has_new_target(ctx) {
        NewTargetReplacer { ctx }.visit_expression(expr);
    }
}

/// Replace `new.target` in the body of a static block with `void 0`.
pub(super) fn replace_new_target_in_statements<'a>(
    stmts: &mut ArenaVec<'a, Statement<'a>>,
    ctx: &TraverseCtx<'a>,
) {
    if has_new_target(ctx) {
        NewTargetReplacer { ctx }.visit_statements(stmts);
    }
}

/// Semantic records whether the program contains `new.target`, and transforms never add one.
/// Most programs have none, so this skips walking every initializer.
#[inline]
fn has_new_target(ctx: &TraverseCtx<'_>) -> bool {
    ctx.scoping().has_new_target()
}

/// Arrow functions inherit `new.target`; other functions, field initializers and static blocks
/// have their own.
struct NewTargetReplacer<'c, 'a> {
    ctx: &'c TraverseCtx<'a>,
}

impl<'a> VisitMut<'a> for NewTargetReplacer<'_, 'a> {
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        if let Expression::NewTarget(new_target) = expr {
            *expr = Expression::new_void_0(new_target.span, self.ctx);
        } else {
            walk_mut::walk_expression(self, expr);
        }
    }

    fn visit_function(&mut self, _function: &mut Function<'a>, _flags: ScopeFlags) {}

    fn visit_class(&mut self, class: &mut Class<'a>) {
        // Decorators, the heritage and computed keys are evaluated in the enclosing context.
        // Method bodies, field initializers and static blocks are not.
        self.visit_decorators(&mut class.decorators);
        if let Some(heritage) = &mut class.heritage {
            self.visit_expression(&mut heritage.expression);
        }
        for element in &mut class.body.body {
            let (decorators, key) = match element {
                ClassElement::MethodDefinition(method) => {
                    let method = &mut **method;
                    (&mut method.decorators, &mut method.key)
                }
                ClassElement::PropertyDefinition(prop) => {
                    let prop = &mut **prop;
                    (&mut prop.decorators, &mut prop.key)
                }
                ClassElement::AccessorProperty(prop) => {
                    let prop = &mut **prop;
                    (&mut prop.decorators, &mut prop.key)
                }
                ClassElement::StaticBlock(_) | ClassElement::TSIndexSignature(_) => continue,
            };
            self.visit_decorators(decorators);
            if key.is_computed() {
                self.visit_property_key(key);
            }
        }
    }
}
