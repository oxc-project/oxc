use crate::{codegen::LateCtx, output::Output, Result};

mod assert_layouts;
mod ast_builder;
mod ast_kind;
mod typescript;
mod visit;

pub use assert_layouts::AssertLayouts;
pub use ast_builder::AstBuilderGenerator;
pub use ast_kind::AstKindGenerator;
pub use typescript::TypescriptGenerator;
pub use visit::{VisitGenerator, VisitMutGenerator};

pub trait Generator {
    // Methods defined by implementer

    fn generate(&mut self, ctx: &LateCtx) -> Output;

    // Standard methods

    fn output(&mut self, ctx: &LateCtx) -> Result<Vec<Output>> {
        Ok(vec![self.generate(ctx)])
    }
}

macro_rules! define_generator {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::{LateCtx, Runner},
                output::Output,
                Result,
            };

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                type Context = LateCtx;

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn file_path(&self) -> &'static str {
                    file!()
                }

                fn run(&mut self, ctx: &LateCtx) -> Result<Vec<Output>> {
                    self.output(ctx)
                }
            }
        };
    };
}
pub(crate) use define_generator;
