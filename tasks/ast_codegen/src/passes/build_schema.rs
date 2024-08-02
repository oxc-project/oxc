use super::{define_pass, Pass};

define_pass! {
    pub struct BuildSchema;
}

impl Pass for BuildSchema {
    fn name(&self) -> &'static str {
        stringify!(BuildSchema)
    }

    fn once(&mut self, ctx: &crate::CodegenCtx) -> crate::Result<bool> {
        for m in ctx.mods.borrow_mut().iter_mut() {
            m.build_in(&mut ctx.schema.borrow_mut())?;
        }
        Ok(true)
    }
}
