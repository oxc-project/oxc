use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

mod class_static_block;
mod options;

use class_static_block::ClassStaticBlock;

pub use options::ES2022Options;

pub struct ES2022 {
    options: ES2022Options,
    // Plugins
    class_static_block: ClassStaticBlock,
}

impl ES2022 {
    pub fn new(options: ES2022Options) -> Self {
        Self { options, class_static_block: ClassStaticBlock::new() }
    }
}

impl<'a> Traverse<'a> for ES2022 {
    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.class_static_block {
            self.class_static_block.enter_class_body(body, ctx);
        }
    }
}
