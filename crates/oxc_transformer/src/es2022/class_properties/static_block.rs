//! ES2022: Class Properties
//! Transform of class static blocks.

use oxc_ast::ast::*;
use oxc_traverse::TraverseCtx;

use super::super::ClassStaticBlock;

use super::ClassProperties;

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Convert static block to `Expression`.
    ///
    /// `static { x = 1; }` -> `x = 1`
    /// `static { x = 1; y = 2; } -> `(() => { x = 1; y = 2; })()`
    ///
    /// TODO: Add tests for this if there aren't any already.
    /// Include tests for evaluation order inc that static block goes before class expression
    /// unless also static properties, or static block uses class name.
    pub(super) fn convert_static_block(
        &mut self,
        block: &mut StaticBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: Convert `this` and references to class name.
        // `x = class C { static { this.C = C; } }` -> `x = (_C = class C {}, _C.C = _C, _C)`
        // TODO: Scope of static block contents becomes outer scope, not scope of class.

        // If class expression, assignment in static block moves to a position where it's read from.
        // e.g.: `x` here now has read+write `ReferenceFlags`:
        // `C = class C { static { x = 1; } }` -> `C = (_C = class C {}, x = 1, _C)`
        let expr = ClassStaticBlock::convert_block_to_expression(block, ctx);
        self.insert_expr_after_class(expr, ctx);
    }
}
