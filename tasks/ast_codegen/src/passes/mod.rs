mod build_schema;
mod calc_layout;
mod linker;

use std::collections::VecDeque;

use itertools::Itertools;

use crate::{schema::RType, CodegenCtx, Result};

pub use build_schema::BuildSchema;
pub use calc_layout::CalcLayout;
pub use linker::Linker;

pub trait Pass {
    fn name(&self) -> &'static str;

    /// Returns false if can't resolve
    fn once(&mut self, _ctx: &CodegenCtx) -> Result<bool> {
        Ok(true)
    }

    /// Returns false if can't resolve
    fn each(&mut self, _ty: &mut RType, _ctx: &CodegenCtx) -> Result<bool> {
        Ok(true)
    }

    fn call(&mut self, ctx: &CodegenCtx) -> Result<bool> {
        // call once
        if !self.once(ctx)? {
            return Ok(false);
        }

        // call each
        // we sort by `TypeId` so we always have the same ordering as how it is written in the rust.
        let mut unresolved =
            ctx.ident_table.iter().sorted_by_key(|it| it.1).map(|it| it.0).collect::<VecDeque<_>>();

        while let Some(next) = unresolved.pop_back() {
            let next_id = *ctx.type_id(next).unwrap();

            let val = &mut ctx.ty_table[next_id].borrow_mut();

            if !self.each(val, ctx)? {
                unresolved.push_front(next);
            }
        }
        Ok(unresolved.is_empty())
    }
}

macro_rules! define_pass {
    ($vis:vis struct $ident:ident $($lifetime:lifetime)? $($rest:tt)*) => {
        $vis struct $ident $($lifetime)? $($rest)*
        impl $($lifetime)? $crate::Runner for $ident $($lifetime)? {
            fn name(&self) -> &'static str {
                $crate::Pass::name(self)
            }

            fn run(&mut self, ctx: &$crate::CodegenCtx) -> $crate::Result<$crate::GeneratorOutput> {
                self.call(ctx)?;
                Ok($crate::GeneratorOutput::None)
            }
        }
    };
}

pub(crate) use define_pass;
