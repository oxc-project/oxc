use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, PathArguments, Type};

use crate::{
    defs::{EnumDef, StructDef, TypeDef},
    output, CodegenCtx, Generator, GeneratorOutput,
};

use super::generated_header;

pub struct AssertLayouts(pub &'static str);

impl Generator for AssertLayouts {
    fn name(&self) -> &'static str {
        self.0
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        let (assertions_64, assertions_32) = ctx
            .schema
            .borrow()
            .definitions
            .iter()
            .map(|def| {
                let typ =
                    ctx.find(def.name()).and_then(|ty| ty.borrow().as_type()).map(|mut ty| {
                        if let Type::Path(ty) = &mut ty {
                            if let Some(seg) = ty.path.segments.first_mut() {
                                if let PathArguments::AngleBracketed(args) = &mut seg.arguments {
                                    *args = parse_quote!(<'static>);
                                }
                            }
                        }
                        ty
                    });
                match def {
                    TypeDef::Struct(StructDef { size_64, align_64, size_32, align_32, .. })
                    | TypeDef::Enum(EnumDef { size_64, align_64, size_32, align_32, .. }) => (
                        quote! {
                            assert!(size_of::<#typ>() == #size_64);
                            assert!(align_of::<#typ>() == #align_64);
                        },
                        quote! {
                            assert!(size_of::<#typ>() == #size_32);
                            assert!(align_of::<#typ>() == #align_32);
                        },
                    ),
                }
            })
            .collect::<(Vec<TokenStream>, Vec<TokenStream>)>();

        let header = generated_header!();

        GeneratorOutput::Stream((
            output(crate::AST_CRATE, self.0),
            quote! {
                #header

                use crate::ast::*;

                endl!();

                #[cfg(target_pointer_width = "64")]
                const _: () = { #(#assertions_64)* };
                #[cfg(target_pointer_width = "32")]
                const _: () = { #(#assertions_32)* };
                #[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
                const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
            },
        ))
    }
}
