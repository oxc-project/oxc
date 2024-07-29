mod build_schema;
mod calc_layout;
mod linker;

use std::collections::VecDeque;

use itertools::Itertools;

use crate::{schema::RType, CodegenCtx, Generator, GeneratorOutput, Result};

pub use build_schema::BuildSchema;
pub use calc_layout::CalcLayout;
pub use linker::Linker;

pub trait Pass {
    /// Returns false if can't resolve
    fn once(&mut self, _ctx: &CodegenCtx) -> Result<bool> {
        Ok(true)
    }

    /// Returns false if can't resolve
    fn each(&mut self, _ty: &mut RType, _ctx: &CodegenCtx) -> Result<bool> {
        Ok(true)
    }
}

pub struct PassGenerator(&'static str, Box<dyn Pass>);

impl PassGenerator {
    pub fn new<P: Pass + 'static>(name: &'static str, pass: P) -> Self {
        Self(name, Box::new(pass))
    }
}

impl Generator for PassGenerator {
    fn name(&self) -> &'static str {
        self.0
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        match self.call(ctx) {
            Ok(val) => GeneratorOutput::Info(vec![val.into()]),
            Err(err) => GeneratorOutput::Err(err),
        }
    }
}

impl PassGenerator {
    fn call(&mut self, ctx: &CodegenCtx) -> Result<bool> {
        // call once
        if !self.1.once(ctx)? {
            return Ok(false);
        }

        // call each
        // we sort by `TypeId` so we always have the same ordering as how it is written in the rust.
        let mut unresolved =
            ctx.ident_table.iter().sorted_by_key(|it| it.1).map(|it| it.0).collect::<VecDeque<_>>();

        while let Some(next) = unresolved.pop_back() {
            let next_id = *ctx.type_id(next).unwrap();

            let val = &mut ctx.ty_table[next_id].borrow_mut();

            if !self.1.each(val, ctx)? {
                unresolved.push_front(next);
            }
        }
        Ok(unresolved.is_empty())
    }
}
