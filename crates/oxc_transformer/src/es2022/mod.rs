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
    options: ES2022Options,
    // Plugins
    class_static_block: ClassStaticBlock,
    class_properties: Option<ClassProperties<'a, 'ctx>>,
}

impl<'a, 'ctx> ES2022<'a, 'ctx> {
    pub fn new(options: ES2022Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            options,
            class_static_block: ClassStaticBlock::new(),
            class_properties: options
                .class_properties
                .map(|options| ClassProperties::new(options, ctx)),
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ES2022<'a, 'ctx> {
    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.class_static_block {
            self.class_static_block.enter_class_body(body, ctx);
        }
        if let Some(class_properties) = &mut self.class_properties {
            class_properties.enter_class_body(body, ctx);
        }
    }
}
