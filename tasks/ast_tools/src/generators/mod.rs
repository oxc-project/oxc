use std::path::PathBuf;

use proc_macro2::TokenStream;

use crate::codegen::LateCtx;

mod assert_layouts;
mod ast_builder;
mod ast_kind;
mod visit;

pub use assert_layouts::AssertLayouts;
pub use ast_builder::AstBuilderGenerator;
pub use ast_kind::AstKindGenerator;
pub use visit::{VisitGenerator, VisitMutGenerator};

/// Inserts a newline in the `TokenStream`.
#[expect(unused)]
macro_rules! endl {
    () => {
        /* only works in the context of `quote` macro family! */
    };
}

pub trait Generator {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput;
}

#[derive(Debug, Clone)]
pub enum GeneratorOutput {
    Rust {
        path: PathBuf,
        tokens: TokenStream,
    },
    #[expect(dead_code)]
    Text {
        path: PathBuf,
        content: String,
    },
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

/// Similar to how `insert` macro works in the context of `quote` macro family, But this one can be
/// used outside and accepts expressions.
/// Wraps the result of the given expression in `insert!({value here});` and outputs it as `TokenStream`.
#[expect(unused)]
macro_rules! insert {
    ($fmt:literal $(, $args:expr)*) => {{
        let txt = format!($fmt, $($args)*);
        format!(r#"insert!("{}");"#, txt).parse::<proc_macro2::TokenStream>().unwrap()
    }};
}
#[expect(unused_imports)]
pub(crate) use insert;
