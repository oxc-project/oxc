mod class_static_block;
mod options;

use std::rc::Rc;

use oxc_ast::ast::ClassBody;

use crate::context::Ctx;

use self::class_static_block::ClassStaticBlock;
pub use self::options::Es2022Options;

#[allow(dead_code)]
pub struct Es2022<'a> {
    ctx: Ctx<'a>,
    options: Rc<Es2022Options>,

    // Plugins
    class_static_block: ClassStaticBlock<'a>,
}

impl<'a> Es2022<'a> {
    pub fn new(options: Es2022Options, ctx: &Ctx<'a>) -> Self {
        let options = Rc::new(options);

        Self {
            options: Rc::clone(&options),
            ctx: Rc::clone(ctx),
            class_static_block: ClassStaticBlock::new(ctx),
        }
    }

    pub fn transform_class_body(&mut self, body: &mut ClassBody<'a>) {
        self.class_static_block.transform_class_body(body);
    }
}
