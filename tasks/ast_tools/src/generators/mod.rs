use crate::{
    codegen::{CodegenBase, LateCtx},
    output::{Output, RawOutput},
    Result,
};

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

pub trait Generator: CodegenBase {
    // Methods defined by implementer

    fn generate(&mut self, ctx: &LateCtx) -> Output;

    // Standard methods

    fn output(&mut self, ctx: &LateCtx) -> Result<RawOutput> {
        let output = self.generate(ctx);
        Ok(output.output(Self::file_path()))
    }
}

macro_rules! define_generator {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::{CodegenBase, LateCtx, Runner},
                output::RawOutput,
                Result,
            };

            impl $($lifetime)? CodegenBase for $ident $($lifetime)? {
                fn file_path() -> &'static str {
                    file!()
                }
            }

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                type Context = LateCtx;
                type Output = RawOutput;

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn run(&mut self, ctx: &LateCtx) -> Result<RawOutput> {
                    self.output(ctx)
                }
            }
        };
    };
}
pub(crate) use define_generator;
