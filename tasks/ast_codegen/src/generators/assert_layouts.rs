use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

use crate::{
    output,
    schema::{FieldDef, ToType, TypeDef},
    Generator, GeneratorOutput, LateCtx,
};

use super::{define_generator, generated_header};

define_generator! {
    pub struct AssertLayouts;
}

impl Generator for AssertLayouts {
    fn name(&self) -> &'static str {
        stringify!(AssertLayouts)
    }

    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let (assertions_64, assertions_32) = ctx
            .schema
            .definitions
            .iter()
            .map(|def| {
                let typ = def.to_type_elide();
                assert_type(&typ, def)
            })
            .collect::<(Vec<TokenStream>, Vec<TokenStream>)>();

        let header = generated_header!();

        GeneratorOutput::Stream((
            output(crate::AST_CRATE, "assert_layouts.rs"),
            quote! {
                #header

                use std::mem::{align_of, offset_of, size_of};

                endl!();

                use crate::ast::*;
                use oxc_span::*;
                use oxc_syntax::{number::*, operator::*};


                endl!();

                #[cfg(target_pointer_width = "64")]
                const _: () = { #(#assertions_64)* };
                endl!();

                #[cfg(target_pointer_width = "32")]
                const _: () = { #(#assertions_32)* };
                endl!();

                #[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
                const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
            },
        ))
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
            let field = field.name.as_ref().map(|it| format_ident!("{it}"));
            quote! {
                assert!(offset_of!(#ty, #field) == #offset);
            }
        },
    );
    tk.extend(assertions);
    tk
}
