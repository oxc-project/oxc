use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

use super::define_generator;
use crate::{
    codegen::{generated_header, LateCtx},
    output,
    schema::{FieldDef, ToType, TypeDef},
    util::ToIdent,
    Generator, GeneratorOutput,
};

define_generator! {
    pub struct AssertLayouts;
}

impl Generator for AssertLayouts {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let (assertions_64, assertions_32) = ctx
            .schema()
            .into_iter()
            .map(|def| {
                let typ = def.to_type_elide();
                assert_type(&typ, def)
            })
            .collect::<(Vec<TokenStream>, Vec<TokenStream>)>();

        let header = generated_header!();

        GeneratorOutput::Rust {
            path: output(crate::AST_CRATE, "assert_layouts.rs"),
            tokens: quote! {
                #header

                use std::mem::{align_of, offset_of, size_of};

                ///@@line_break
                #[allow(clippy::wildcard_imports)]
                use crate::ast::*;

                ///@@line_break
                #[allow(clippy::wildcard_imports)]
                use oxc_regular_expression::ast::*;

                ///@@line_break
                #[cfg(target_pointer_width = "64")]
                const _: () = { #(#assertions_64)* };

                ///@@line_break
                #[cfg(target_pointer_width = "32")]
                const _: () = { #(#assertions_32)* };

                ///@@line_break
                #[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
                const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
            },
        }
    }
}

fn assert_type(ty: &Type, def: &TypeDef) -> (TokenStream, TokenStream) {
    match def {
        TypeDef::Struct(def) => (
            with_offsets_assertion(
                assert_size_align(ty, def.size_64, def.align_64),
                ty,
                &def.fields,
                def.offsets_64.as_deref(),
            ),
            with_offsets_assertion(
                assert_size_align(ty, def.size_32, def.align_32),
                ty,
                &def.fields,
                def.offsets_32.as_deref(),
            ),
        ),
        TypeDef::Enum(def) => (
            assert_size_align(ty, def.size_64, def.align_64),
            assert_size_align(ty, def.size_32, def.align_32),
        ),
    }
}

fn assert_size_align(ty: &Type, size: usize, align: usize) -> TokenStream {
    quote! {
        ///@@line_break
        assert!(size_of::<#ty>() == #size);
        assert!(align_of::<#ty>() == #align);
    }
}

fn with_offsets_assertion(
    mut tk: TokenStream,
    ty: &Type,
    fields: &[FieldDef],
    offsets: Option<&[usize]>,
) -> TokenStream {
    let Some(offsets) = offsets else { return tk };

    let assertions = fields.iter().zip(offsets).filter(|(field, _)| field.vis.is_pub()).map(
        |(field, offset)| {
            let field = field.name.as_ref().map(ToIdent::to_ident);
            quote! {
                assert!(offset_of!(#ty, #field) == #offset);
            }
        },
    );
    tk.extend(assertions);
    tk
}
