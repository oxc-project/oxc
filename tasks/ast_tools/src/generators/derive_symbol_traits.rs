use proc_macro2::TokenStream;

use crate::{
    codegen::LateCtx,
    markers::{SymbolBinding, SymbolMarkers},
    output,
    schema::{FieldDef, GetIdent, StructDef, TypeDef},
};
use quote::quote;

use super::{define_generator, Generator, GeneratorOutput};

define_generator! {
    pub struct DeriveSymbolTraitsGenerator;
}

impl Generator for DeriveSymbolTraitsGenerator {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let impls: Vec<_> = ctx
            .schema()
            .into_iter()
            .filter_map(TypeDef::as_struct)
            .filter_map(|struct_def| {
                struct_def.fields.iter().find_map(|field| {
                    field
                        .markers
                        .binding
                        .as_ref()
                        .and_then(SymbolMarkers::as_binding)
                        .map(|binding| (struct_def, field, binding))
                })
            })
            .map(generate_symbol_id_impl)
            .collect();

        let stream = quote! {
            use oxc_syntax::symbol::SymbolId;

            ///@@line_break
            #[allow(clippy::wildcard_imports)]
            use crate::ast::*;
            use crate::WithBindingIdentifier;
            use oxc_span::Atom;

            ///@@line_break
            #(#impls)*
        };

        GeneratorOutput::Stream((output(crate::AST_CRATE, "derive_symbol_traits.rs"), stream))
    }
}

fn generate_symbol_id_impl(
    (def, field, binding): (&StructDef, &FieldDef, &SymbolBinding),
) -> TokenStream {
    let struct_name = &def.ident();
    let field_name = field.ident().unwrap();
    let (symbol_id_impl, name_impl) = if binding.optional {
        if binding.recurse {
            (
                quote! { self.#field_name.as_ref().and_then(WithBindingIdentifier::symbol_id) },
                quote! { self.#field_name.as_ref().map(WithBindingIdentifier::name) },
            )
        } else {
            (
                quote! { self.#field_name.as_ref().and_then(|id| id.symbol_id.get()) },
                quote! { self.#field_name.as_ref().map(|id| id.name.clone()) },
            )
        }
    } else if binding.recurse {
        (quote! { self.#field_name.symbol_id() }, quote! { self.#field_name.name() })
    } else {
        (
            quote! { self.#field_name.symbol_id.get() },
            quote! { Some(self.#field_name.name.clone()) },
        )
    };

    quote! {
        ///@@line_break
        impl<'a> WithBindingIdentifier<'a> for #struct_name<'a> {
            #[inline]
            fn symbol_id(&self) -> Option<SymbolId> {
                #symbol_id_impl
            }

            ///@@line_break
            #[inline]
            fn name(&self) -> Option<Atom<'a>> {
                #name_impl
            }
        }
    }
}
