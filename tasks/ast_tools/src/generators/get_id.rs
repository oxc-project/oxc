//! Generator for ID getter/setter methods on all types with `scope_id`, `symbol_id`, `reference_id`
//! fields.
//!
//! e.g. Generates `scope_id` and `set_scope_id` methods on all types with a `scope_id` field.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    output::{output_path, Output},
    schema::{Schema, TypeDef},
    util::ToIdent,
    Generator,
};

use super::define_generator;

pub struct GetIdGenerator;

define_generator!(GetIdGenerator);

impl Generator for GetIdGenerator {
    fn generate(&mut self, schema: &Schema) -> Output {
        let impls = schema.defs.iter().filter_map(generate_for_type);

        let output = quote! {
            use oxc_syntax::{reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

            ///@@line_break
            use crate::ast::*;

            #(#impls)*
        };

        Output::Rust { path: output_path(crate::AST_CRATE, "get_id.rs"), tokens: output }
    }
}

fn generate_for_type(def: &TypeDef) -> Option<TokenStream> {
    let TypeDef::Struct(def) = def else { return None };

    let struct_name = def.name.as_str();

    let methods = def
        .fields
        .iter()
        .filter_map(|field| {
            let field_ident = field.ident().expect("expected named field");
            let field_name = field_ident.to_string();

            let type_name = match (field_name.as_str(), field.typ.raw()) {
                ("scope_id", "Cell<Option<ScopeId>>") => "ScopeId",
                ("symbol_id", "Cell<Option<SymbolId>>") => "SymbolId",
                ("reference_id", "Cell<Option<ReferenceId>>") => "ReferenceId",
                _ => return None,
            };
            let type_ident = type_name.to_ident();

            // Generate getter method
            let get_doc1 = format!(" Get [`{type_name}`] of [`{struct_name}`].");
            let get_doc2 = format!(" Only use this method on a post-semantic AST where [`{type_name}`]s are always defined.");
            let get_doc3 = format!(" Panics if `{field_name}` is [`None`].");

            let get_method = quote! {
                #[doc = #get_doc1]
                ///
                #[doc = #get_doc2]
                ///
                /// # Panics
                #[doc = #get_doc3]
                #[inline]
                pub fn #field_ident(&self) -> #type_ident {
                    self.#field_ident.get().unwrap()
                }
            };

            // Generate setter method
            let set_method_ident = format_ident!("set_{field_name}");
            let set_doc = format!(" Set [`{type_name}`] of [`{struct_name}`].");
            let set_method = quote! {
                #[doc = #set_doc]
                #[inline]
                pub fn #set_method_ident(&self, #field_ident: #type_ident) {
                    self.#field_ident.set(Some(#field_ident));
                }
            };

            Some(quote! {
                ///@@line_break
                #get_method

                ///@@line_break
                #set_method
            })
        })
        .collect::<Vec<_>>();

    if methods.is_empty() {
        return None;
    }

    let struct_name_ident = struct_name.to_ident();
    let lifetime = if def.has_lifetime { quote!(<'_>) } else { TokenStream::default() };

    Some(quote! {
        ///@@line_break
        impl #struct_name_ident #lifetime {
            #(#methods)*
        }
    })
}
