use super::Pass;

pub struct BuildSchema;

impl Pass for BuildSchema {
    fn once(&mut self, ctx: &crate::CodegenCtx) -> crate::Result<bool> {
        for m in ctx.mods.borrow().iter() {
            m.build_in(&mut ctx.schema.borrow_mut())?;
        }
        Ok(true)
    }
}
