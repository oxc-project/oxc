use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

use super::{define_derive, Derive, DeriveOutput};
use crate::{
    codegen::LateCtx,
    markers::ESTreeStructAttribute,
    schema::{
        serialize::{enum_variant_name, get_type_tag},
        EnumDef, GetGenerics, GetIdent, StructDef, TypeDef,
    },
};

define_derive! {
    pub struct DeriveESTree;
}

impl Derive for DeriveESTree {
    fn trait_name() -> &'static str {
        "ESTree"
    }

    fn snake_name() -> String {
        "estree".to_string()
    }

    fn derive(&mut self, def: &TypeDef, _: &LateCtx) -> TokenStream {
        if let TypeDef::Struct(def) = def {
            if def
                .markers
                .estree
                .as_ref()
                .is_some_and(|e| e == &ESTreeStructAttribute::CustomSerialize)
            {
                return TokenStream::new();
            }
        }

        let body = match def {
            TypeDef::Enum(def) => serialize_enum(def),
            TypeDef::Struct(def) => serialize_struct(def),
        };
        let ident = def.ident();

        let lifetime = if def.has_lifetime() { quote!(<'a>) } else { TokenStream::new() };
        quote! {
            impl #lifetime Serialize for #ident #lifetime {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    #body
                }
            }
        }
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(unused_imports, unused_mut, clippy::match_same_arms)]

            ///@@line_break
            use serde::{Serialize, Serializer, ser::SerializeMap};
        }
    }
}

fn serialize_struct(def: &StructDef) -> TokenStream {
    let ident = def.ident();
    // If type_tag is Some, we serialize it manually. If None, either one of
    // the fields is named r#type, or the struct does not need a "type" field.
    let type_tag = get_type_tag(def);

    let mut fields = vec![];
    if let Some(ty) = &type_tag {
        fields.push(quote! { map.serialize_entry("type", #ty)?; });
    }
    for field in &def.fields {
        if field.markers.derive_attributes.estree.skip {
            continue;
        }
        let name = match &field.markers.derive_attributes.estree.rename {
            Some(rename) => rename.to_string(),
            None => field.name.clone().unwrap().to_case(Case::Camel),
        };
        assert!(
            !(name == "type" && type_tag.is_some()),
            "Unexpected r#type field when #[estree(type = ...)] is specified (on {ident})"
        );

        let ident = field.ident().unwrap();
        if field.markers.derive_attributes.estree.flatten {
            fields.push(quote! {
                self.#ident.serialize(
                    serde::__private::ser::FlatMapSerializer(&mut map)
                )?;
            });
        } else {
            fields.push(quote! {
                map.serialize_entry(#name, &self.#ident)?;
            });
        }
    }

    quote! {
        let mut map = serializer.serialize_map(None)?;
        #(#fields)*
        map.end()
    }
}

// 3 different kinds of AST enums:
//  1. Transparent enums, which would be #[serde(untagged)]. These take their
//     type tag from their children. Each of the variants is its own struct.
//  2. Type enums, which are not camelCased. These are for example the
//     r#type field of a Function, and are used instead of the struct name
//     as the type field on the JSON.
//  3. All other enums, which are camelCased.
fn serialize_enum(def: &EnumDef) -> TokenStream {
    let ident = def.ident();
    if def.markers.estree.untagged {
        let match_branches = def.all_variants().map(|var| {
            let var_ident = var.ident();
            assert!(var.fields.len() == 1, "Each variant of an untagged enum must have exactly one inner field (on {ident}::{var_ident})");
            quote! {
                #ident::#var_ident(x) => {
                    Serialize::serialize(x, serializer)
                }
            }
        });
        quote! {
            match self {
                #(#match_branches),*
            }
        }
    } else {
        let match_branches = def.all_variants().map(|var| {
            let var_ident = var.ident();
            let enum_name = ident.to_string();
            let discriminant = u32::from(var.discriminant);
            let serialized_to = enum_variant_name(var, def);
            assert!(
                var.fields.is_empty(),
                "Tagged enums must not have inner fields (on {ident}::{var_ident})"
            );
            quote! {
                #ident::#var_ident => {
                    serializer.serialize_unit_variant(#enum_name, #discriminant, #serialized_to)
                }
            }
        });
        quote! {
            match *self {
                #(#match_branches),*
            }
        }
    }
}
