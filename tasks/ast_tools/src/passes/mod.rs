use std::collections::VecDeque;

use crate::{codegen::EarlyCtx, output::Output, rust_ast::AstType, Result};

mod calc_layout;
mod linker;
pub use calc_layout::CalcLayout;
pub use linker::Linker;

pub trait Pass {
    // Methods defined by implementer

    /// Run on each type.
    /// Returns `false` if can't resolve.
    fn each(&mut self, ty: &mut AstType, ctx: &EarlyCtx) -> Result<bool>;

    // Standard methods

    /// Run pass.
    fn output(&mut self, ctx: &EarlyCtx) -> Result<Vec<Output>> {
        // We sort by `TypeId`, so we have the same ordering as it's written in Rust source
        let mut unresolved = ctx.chronological_idents().collect::<VecDeque<_>>();

        while let Some(next) = unresolved.pop_back() {
            let next_id = ctx.type_id(next).unwrap();

            let ast_ref = ctx.ast_ref(next_id);
            let val = &mut ast_ref.borrow_mut();

            if !self.each(val, ctx)? {
                unresolved.push_front(next);
            }
        }
        Ok(vec![])
    }
}

macro_rules! define_pass {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::{EarlyCtx, Runner},
                output::Output,
                Result,
            };

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                type Context = EarlyCtx;

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn file_path(&self) -> &'static str {
                    file!()
                }

                fn run(&mut self, ctx: &Self::Context) -> Result<Vec<Output>> {
                    self.output(ctx)
                }
            }
        };
    };
}

pub(crate) use define_pass;
