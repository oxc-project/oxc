use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod class_properties;
mod class_static_block;
mod options;

use class_properties::ClassProperties;
pub use class_properties::ClassPropertiesOptions;
use class_static_block::ClassStaticBlock;
pub use options::ES2022Options;

pub struct ES2022<'a, 'ctx> {
    // Plugins
    class_static_block: Option<ClassStaticBlock>,
    class_properties: Option<ClassProperties<'a, 'ctx>>,
}

impl<'a, 'ctx> ES2022<'a, 'ctx> {
    pub fn new(options: ES2022Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        // Class properties transform performs the static block transform differently.
        // So only enable static block transform if class properties transform is disabled.
        let (class_static_block, class_properties) =
            if let Some(properties_options) = options.class_properties {
                let class_properties =
                    ClassProperties::new(properties_options, options.class_static_block, ctx);
                (None, Some(class_properties))
            } else {
                let class_static_block =
                    if options.class_static_block { Some(ClassStaticBlock::new()) } else { None };
                (class_static_block, None)
            };
        Self { class_static_block, class_properties }
    }
}

impl<'a> Traverse<'a> for ES2022<'a, '_> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_expression(expr, ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.exit_expression(expr, ctx);
        }
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_class_body(body, ctx);
        } else if let Some(class_static_block) = &mut self.class_static_block {
            class_static_block.enter_class_body(body, ctx);
        }
    }

    fn exit_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.exit_class(class, ctx);
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
}
