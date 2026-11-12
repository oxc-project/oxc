use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_traverse::Traverse;

use crate::{
    common::class_transform::ClassTransform, context::TraverseCtx, decorator::DecoratorMode,
    state::TransformState,
};

mod class_properties;
mod class_static_block;
mod options;

use class_properties::ClassProperties;
pub use class_properties::ClassPropertiesOptions;
use class_static_block::ClassStaticBlock;
pub use options::ES2022Options;

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
        // Class properties transform performs the static block transform differently,
        // so the standalone static block transform runs only when class properties is disabled.
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
                    false,
                )),
            ),
            ClassTransform::StandardDecorators { transform_static_blocks } => (
                transform_static_blocks.then(|| ClassStaticBlock::new(true)),
                Some(ClassProperties::new(
                    ClassPropertiesOptions::default(),
                    transform_static_blocks,
                    remove_class_fields_without_initializer,
                    DecoratorMode::Standard,
                    true,
                )),
            ),
            ClassTransform::StaticBlocks => (Some(ClassStaticBlock::new(false)), None),
            ClassTransform::None => (None, None),
        };
        Self { top_level_await, class_static_block, class_properties }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ES2022<'a> {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
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

    #[inline]
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.exit_statement(stmt, ctx);
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
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_expression(expr, ctx);
        }
        if let Some(class_static_block) = &mut self.class_static_block {
            class_static_block.enter_expression(expr, ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !matches!(expr, Expression::ClassExpression(_)) {
            return;
        }
        // Both transforms are present only when class properties transform lowers decorated
        // classes, and static block transform handles the rest. Class properties transform runs
        // first. If it replaced the class, static block transform skipped that class.
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.exit_expression(expr, ctx);
        }
        if let Some(class_static_block) = &mut self.class_static_block {
            if matches!(expr, Expression::ClassExpression(_)) {
                class_static_block.exit_expression(expr, ctx);
            } else {
                class_static_block.exit_skipped_class_expression();
            }
        }
    }

    fn enter_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_object_property(prop, ctx);
        }
    }

    fn enter_decorator(&mut self, decorator: &mut Decorator<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_decorator(decorator, ctx);
        }
    }

    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_class(class, ctx);
        }
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        // Static block transform first, so it can skip decorated classes before their decorators
        // are lowered. Plain auto-accessors were already lowered on entering the class.
        if let Some(class_static_block) = &mut self.class_static_block {
            class_static_block.enter_class_body(body, ctx);
        }
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_class_body(body, ctx);
        }
    }

    fn exit_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.exit_class(class, ctx);
        }
        if let Some(class_static_block) = &mut self.class_static_block {
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
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_property_definition(prop, ctx);
        }
        if let Some(class_static_block) = &mut self.class_static_block {
            class_static_block.enter_property_definition(prop, ctx);
        }
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.exit_property_definition(prop, ctx);
        }
    }

    fn enter_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_static_block(block, ctx);
        }
    }

    fn exit_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
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

impl ES2022<'_> {
    fn is_top_level(ctx: &TraverseCtx) -> bool {
        ctx.current_hoist_scope_id() == ctx.scoping().root_scope_id()
    }
}
