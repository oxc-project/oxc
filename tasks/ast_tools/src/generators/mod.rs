use std::path::PathBuf;

use proc_macro2::TokenStream;

use crate::codegen::LateCtx;

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
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput;
}

#[derive(Debug, Clone)]
pub enum GeneratorOutput {
    Rust { path: PathBuf, tokens: TokenStream },
    Text { path: PathBuf, content: String },
}

macro_rules! define_generator {
    ($vis:vis struct $ident:ident $($lifetime:lifetime)? $($rest:tt)*) => {
        $vis struct $ident $($lifetime)? $($rest)*
        impl $($lifetime)? $crate::codegen::Runner for $ident $($lifetime)? {
            type Context = $crate::codegen::LateCtx;
            type Output = $crate::GeneratorOutput;

            fn name(&self) -> &'static str {
                stringify!($ident)
            }

            fn run(&mut self, ctx: &$crate::codegen::LateCtx) -> $crate::Result<Self::Output> {
                Ok(self.generate(ctx))
            }
        }
    };
}
pub(crate) use define_generator;
