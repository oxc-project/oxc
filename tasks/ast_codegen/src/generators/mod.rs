use std::path::PathBuf;

use proc_macro2::TokenStream;

use crate::codegen::LateCtx;

mod assert_layouts;
mod ast_builder;
mod ast_kind;
mod derive_clone_in;
mod derive_get_span;
mod visit;

pub use assert_layouts::AssertLayouts;
pub use ast_builder::AstBuilderGenerator;
pub use ast_kind::AstKindGenerator;
pub use derive_clone_in::DeriveCloneIn;
pub use derive_get_span::{DeriveGetSpan, DeriveGetSpanMut};
pub use visit::{VisitGenerator, VisitMutGenerator};

/// Inserts a newline in the `TokenStream`.
#[allow(unused)]
macro_rules! endl {
    () => {
        /* only works in the context of `quote` macro family! */
    };
}

pub trait Generator {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput;
}

pub type GeneratedTokenStream = (/* output path */ PathBuf, TokenStream);
pub type GeneratedDataStream = (/* output path */ PathBuf, Vec<u8>);

// TODO: remove me
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum GeneratorOutput {
    None,
    Info(Vec<u8>),
    Data(GeneratedDataStream),
    Stream(GeneratedTokenStream),
}

// TODO: remove me
#[allow(dead_code)]
impl GeneratorOutput {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn expect_none(&self) {
        assert!(self.is_none());
    }

    pub fn to_info(&self) -> &[u8] {
        if let Self::Info(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn to_data(&self) -> &GeneratedDataStream {
        if let Self::Data(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn to_stream(&self) -> &GeneratedTokenStream {
        if let Self::Stream(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn into_info(self) -> Vec<u8> {
        if let Self::Info(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn into_data(self) -> GeneratedDataStream {
        if let Self::Data(it) = self {
            it
        } else {
            panic!();
        }
    }

    pub fn into_stream(self) -> GeneratedTokenStream {
        if let Self::Stream(it) = self {
            it
        } else {
            panic!();
        }
    }
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
#[allow(unused)]
macro_rules! insert {
    ($fmt:literal $(, $args:expr)*) => {{
        let txt = format!($fmt, $($args)*);
        format!(r#"insert!("{}");"#, txt).parse::<proc_macro2::TokenStream>().unwrap()
    }};
}
#[allow(unused_imports)]
pub(crate) use insert;

/// Creates a generated file warning + required information for a generated file.
macro_rules! generated_header {
    () => {{
        let file = file!().replace("\\", "/");
        // TODO add generation date, AST source hash, etc here.
        let edit_comment = format!("@ To edit this generated file you have to edit `{file}`");
        quote::quote! {
            //!@ Auto-generated code, DO NOT EDIT DIRECTLY!
            #![doc = #edit_comment]
            //!@@line_break
        }
    }};
}

pub(crate) use generated_header;
