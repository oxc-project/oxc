use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_syntax::scope::ScopeId;
use oxc_traverse::{Ancestor, Traverse};

use crate::{
    common::{class_lowering::ClassLowering, class_transform::ClassTransform},
    context::TraverseCtx,
    decorator::DecoratorMode,
    state::TransformState,
};

mod class_properties;
mod class_static_block;
mod options;

use class_properties::ClassProperties;
pub use class_properties::ClassPropertiesOptions;
use class_static_block::ClassStaticBlock;
pub use options::ES2022Options;

/// The transform which lowers a class, decided by its [`ClassLowering`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClassOwner {
    /// Class-properties lowers the class's fields, and its static blocks with them.
    ClassProperties,
    /// The static block transform lowers the class's static blocks.
    StaticBlocks,
    /// Neither transform lowers the class.
    None,
}

/// Each class is handled by one transform, its [`ClassOwner`]. For the classes it does not own,
/// class-properties still keeps the state lowered classes and standard decorators read (see
/// `class_properties::unlowered_class`).
pub struct ES2022<'a> {
    top_level_await: bool,

    // Plugins
    class_static_block: Option<ClassStaticBlock<'a>>,
    class_properties: Option<ClassProperties<'a>>,
}

impl ES2022<'_> {
    pub fn new(
        top_level_await: bool,
        class_transform: ClassTransform,
        remove_class_fields_without_initializer: bool,
    ) -> Self {
        // Class-properties owns static blocks in classes it lowers. In decorators-only mode,
        // the standalone static-block transform handles the remaining classes.
        let (class_static_block, class_properties) = match class_transform {
            ClassTransform::ClassProperties {
                options,
                transform_static_blocks,
                decorator_mode,
            } => (
                None,
                Some(ClassProperties::new(
                    options,
                    transform_static_blocks,
                    remove_class_fields_without_initializer,
                    decorator_mode,
                )),
            ),
            ClassTransform::StandardDecorators { transform_static_blocks } => (
                transform_static_blocks.then(ClassStaticBlock::new),
                Some(ClassProperties::new(
                    ClassPropertiesOptions::default(),
                    transform_static_blocks,
                    remove_class_fields_without_initializer,
                    DecoratorMode::Standard,
                )),
            ),
            ClassTransform::StaticBlocks => (Some(ClassStaticBlock::new()), None),
            ClassTransform::None => (None, None),
        };
        Self { top_level_await, class_static_block, class_properties }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ES2022<'a> {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Class-properties also scopes loop bodies for standard decorators, so it sees every
        // statement. The static block transform only needs class and export statements.
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_statement(stmt, ctx);
        }
        if !matches!(
            stmt,
            Statement::ClassDeclaration(_)
                | Statement::ExportDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
        ) {
            return;
        }
        if let Some(class_static_block) = &mut self.class_static_block {
            class_static_block.enter_statement(stmt, ctx);
        }
    }

    #[inline] // Because this is a no-op in release mode
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.exit_program(program, ctx);
        }
    }

    #[inline]
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // Keep the `else if` shape for the common single-transform case. Visiting both transforms
        // here made the inlined expression walker slower for every expression.
        if let Some(class_properties) = &mut self.class_properties {
            if self.class_static_block.is_some() {
                self.enter_expression_both_class_transforms(expr, ctx);
            } else if class_properties.lowers_standard_decorators()
                && matches!(expr, Expression::ClassExpression(_))
            {
                self.enter_class_expression(expr, ctx);
            } else {
                class_properties.enter_expression(expr, ctx);
            }
        } else if let Some(class_static_block) = &mut self.class_static_block {
            class_static_block.enter_expression(expr, ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ClassExpression(class) = expr else { return };
        match self.entered_class_owner(class.scope_id(), ctx) {
            ClassOwner::ClassProperties => {
                if let Some(class_properties) = &mut self.class_properties {
                    class_properties.exit_expression(expr, ctx);
                }
            }
            ClassOwner::StaticBlocks => {
                if let Some(class_static_block) = &mut self.class_static_block {
                    class_static_block.exit_expression(expr, ctx);
                }
            }
            ClassOwner::None => {}
        }
    }

    fn enter_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_object_property(prop, ctx);
        }
    }

    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        // Neither transform has work on entering a class it owns. Lower the native
        // auto-accessors of the others.
        if self.entered_class_owner(class.scope_id(), ctx) != ClassOwner::ClassProperties
            && let Some(class_properties) = &mut self.class_properties
        {
            class_properties.enter_unlowered_class(class, ctx);
        }
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        let Ancestor::ClassBody(class) = ctx.parent() else { unreachable!() };
        let owner = self.entered_class_owner(class.scope_id().get().unwrap(), ctx);
        if owner == ClassOwner::ClassProperties {
            // The static block transform's parent hooks may have prepared this class before its
            // owner was known.
            if let Some(class_static_block) = &mut self.class_static_block {
                class_static_block.discard_pending_class(body);
            }
            if let Some(class_properties) = &mut self.class_properties {
                class_properties.enter_class_body(body, ctx);
            }
            return;
        }
        // Plain auto-accessors were already lowered on entering the class, so their generated
        // static block is lowered here. Capture names after the blocks are folded into fields.
        if owner == ClassOwner::StaticBlocks
            && let Some(class_static_block) = &mut self.class_static_block
        {
            class_static_block.enter_class_body(body, ctx);
        }
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_unlowered_class_body(body, ctx);
        }
    }

    fn exit_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let owner = self.entered_class_owner(class.scope_id(), ctx);
        if owner == ClassOwner::ClassProperties {
            if let Some(class_properties) = &mut self.class_properties {
                class_properties.exit_class(class, ctx);
            }
            return;
        }
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.exit_unlowered_class(class);
        }
        if owner == ClassOwner::StaticBlocks
            && let Some(class_static_block) = &mut self.class_static_block
        {
            class_static_block.exit_class(class, ctx);
        }
    }

    fn enter_assignment_target(
        &mut self,
        target: &mut AssignmentTarget<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_assignment_target(target, ctx);
        }
    }

    fn enter_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.current_class_owner(ctx) == ClassOwner::ClassProperties
            && let Some(class_properties) = &mut self.class_properties
        {
            class_properties.enter_property_definition(prop, ctx);
        }
        // Like the statement hook, this prepares the class in the initializer, whichever
        // transform owns the class the property belongs to.
        if let Some(class_static_block) = &mut self.class_static_block {
            class_static_block.enter_property_definition(prop, ctx);
        }
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.current_class_owner(ctx) == ClassOwner::ClassProperties
            && let Some(class_properties) = &mut self.class_properties
        {
            class_properties.exit_property_definition(prop, ctx);
        }
    }

    fn enter_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.current_class_owner(ctx) == ClassOwner::ClassProperties
            && let Some(class_properties) = &mut self.class_properties
        {
            class_properties.enter_static_block(block, ctx);
        }
    }

    fn exit_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.current_class_owner(ctx) == ClassOwner::ClassProperties
            && let Some(class_properties) = &mut self.class_properties
        {
            class_properties.exit_static_block(block, ctx);
        }
    }

    fn enter_await_expression(
        &mut self,
        node: &mut AwaitExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.top_level_await && Self::is_top_level(ctx) {
            let warning = OxcDiagnostic::warn(
                "Top-level await is not available in the configured target environment.",
            )
            .with_label(node.span);
            ctx.state.error(warning);
        }
    }
}

impl<'a> ES2022<'a> {
    /// The owner of a class with the given lowering.
    fn class_owner(&self, lowering: ClassLowering) -> ClassOwner {
        if lowering.lowers_fields() {
            debug_assert!(self.class_properties.is_some());
            ClassOwner::ClassProperties
        } else if self.class_static_block.is_some() {
            debug_assert!(ClassStaticBlock::owns_class(lowering));
            ClassOwner::StaticBlocks
        } else {
            ClassOwner::None
        }
    }

    /// The owner of a class which has been entered.
    fn entered_class_owner(&self, scope_id: ScopeId, ctx: &TraverseCtx<'a>) -> ClassOwner {
        // Without class-properties, every class keeps its fields.
        if self.class_properties.is_none() {
            return if self.class_static_block.is_some() {
                ClassOwner::StaticBlocks
            } else {
                ClassOwner::None
            };
        }
        self.class_owner(ctx.state.class_lowering_of(scope_id))
    }

    /// The owner of the class whose element is being visited.
    fn current_class_owner(&self, ctx: &TraverseCtx<'a>) -> ClassOwner {
        // Class elements are visited in the class scope.
        self.entered_class_owner(ctx.current_scope_id(), ctx)
    }

    /// Enter a class expression when class-properties lowers standard decorators, but static
    /// blocks are not lowered. Out of line to keep it out of the expression walker.
    #[inline(never)]
    fn enter_class_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ClassExpression(class) = expr else { unreachable!() };
        let lowering = ctx.state.class_lowering_before_entry(class);
        let owner = self.class_owner(lowering);
        let Some(class_properties) = &mut self.class_properties else { unreachable!() };
        if owner == ClassOwner::ClassProperties {
            class_properties.enter_class_expression(expr, lowering, ctx);
        } else {
            class_properties.enter_unlowered_class_expression(expr, lowering, ctx);
        }
    }

    /// Visit an expression when both transforms are present, which happens only when standard
    /// decorators and static blocks are lowered. Out of line to keep it out of the expression
    /// walker.
    #[inline(never)]
    fn enter_expression_both_class_transforms(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Class-properties enters only a bare class expression. The static block transform also
        // enters one wrapped in parentheses or TypeScript syntax.
        let class_lowering = match expr.get_inner_expression() {
            Expression::ClassExpression(class) => {
                Some(ctx.state.class_lowering_before_entry(class))
            }
            _ => None,
        };
        let is_class_expression = matches!(expr, Expression::ClassExpression(_));
        let (Some(class_properties), Some(class_static_block)) =
            (&mut self.class_properties, &mut self.class_static_block)
        else {
            unreachable!()
        };
        let Some(lowering) = class_lowering else {
            class_properties.enter_expression(expr, ctx);
            class_static_block.enter_expression(expr, ctx);
            return;
        };
        if lowering.lowers_fields() {
            if is_class_expression {
                class_properties.enter_class_expression(expr, lowering, ctx);
            }
        } else {
            if is_class_expression {
                class_properties.enter_unlowered_class_expression(expr, lowering, ctx);
            }
            class_static_block.enter_expression(expr, ctx);
        }
    }

    fn is_top_level(ctx: &TraverseCtx) -> bool {
        ctx.current_hoist_scope_id() == ctx.scoping().root_scope_id()
    }
}
