use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::{generate_header, CodegenBase, LateCtx},
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

#[derive(Debug, Clone)]
pub enum GeneratorOutput {
    Rust { path: PathBuf, tokens: TokenStream },
    Text { path: PathBuf, content: String },
}

pub trait Generator: CodegenBase {
    // Methods defined by implementer

    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput;

    // Standard methods

    fn output(&mut self, ctx: &LateCtx) -> Result<GeneratorOutput> {
        let mut output = self.generate(ctx);

        if let GeneratorOutput::Rust { tokens, .. } = &mut output {
            let header = generate_header(Self::file_path());
            *tokens = quote! {
                #header
                #tokens
            };
        }

        Ok(output)
    }
}

macro_rules! define_generator {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::{CodegenBase, LateCtx, Runner},
                generators::GeneratorOutput,
                Result,
            };

            impl $($lifetime)? CodegenBase for $ident $($lifetime)? {
                fn file_path() -> &'static str {
                    file!()
                }
            }

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                type Context = LateCtx;
                type Output = GeneratorOutput;

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn run(&mut self, ctx: &LateCtx) -> Result<GeneratorOutput> {
                    self.output(ctx)
                }
            }
        };
    };
}
pub(crate) use define_generator;
