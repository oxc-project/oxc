mod assert_layouts;
mod ast_builder;
mod ast_kind;
mod derive_clone_in;
mod derive_get_span;
mod visit;

/// Inserts a newline in the `TokenStream`.
#[allow(unused)]
macro_rules! endl {
    () => {
        /* only works in the context of `quote` macro family! */
    };
}

/// Similar to how `insert` macro works in the context of `quote` macro family, But this one can be
/// used outside and accepts expressions.
/// Wraps the result of the given expression in `insert!({value here});` and outputs it as `TokenStream`.
macro_rules! insert {
    ($fmt:literal $(, $args:expr)*) => {{
        let txt = format!($fmt, $($args)*);
        format!(r#"insert!("{}");"#, txt).parse::<proc_macro2::TokenStream>().unwrap()
    }};
}

/// Creates a generated file warning + required information for a generated file.
macro_rules! generated_header {
    () => {{
        let file = file!().replace("\\", "/");
        let edit_comment =
            $crate::generators::insert!("// To edit this generated file you have to edit `{file}`");
        // TODO add generation date, AST source hash, etc here.
        quote::quote! {
            insert!("// Auto-generated code, DO NOT EDIT DIRECTLY!");
            #edit_comment
            endl!();
        }
    }};
}

pub(crate) use generated_header;
pub(crate) use insert;

pub use assert_layouts::AssertLayouts;
pub use ast_builder::AstBuilderGenerator;
pub use ast_kind::AstKindGenerator;
pub use derive_clone_in::DeriveCloneIn;
pub use derive_get_span::{DeriveGetSpan, DeriveGetSpanMut};
pub use visit::{VisitGenerator, VisitMutGenerator};

use crate::{GeneratorOutput, LateCtx};

pub trait Generator {
    fn name(&self) -> &'static str;
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput;
}

macro_rules! define_generator {
    ($vis:vis struct $ident:ident $($lifetime:lifetime)? $($rest:tt)*) => {
        $vis struct $ident $($lifetime)? $($rest)*
        impl $($lifetime)? $crate::Runner for $ident $($lifetime)? {
            type Context = $crate::LateCtx;
            type Output = $crate::GeneratorOutput;

            fn name(&self) -> &'static str {
                $crate::Generator::name(self)
            }

            fn run(&mut self, ctx: &$crate::LateCtx) -> $crate::Result<Self::Output> {
                Ok(self.generate(ctx))
            }
        }
    };
}

pub(crate) use define_generator;
