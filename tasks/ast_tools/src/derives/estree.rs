use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use rustc_hash::FxHashMap;

use crate::{
    markers::ESTreeStructTagMode,
    schema::{
        serialize::{enum_variant_name, get_always_flatten_structs, get_type_tag},
        EnumDef, FieldDef, GetGenerics, GetIdent, Schema, StructDef, TypeDef,
    },
};

use super::{define_derive, Derive};

pub struct DeriveESTree;

define_derive!(DeriveESTree);

impl Derive for DeriveESTree {
    fn trait_name() -> &'static str {
        "ESTree"
    }

    fn snake_name() -> String {
        "estree".to_string()
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(unused_imports, unused_mut, clippy::match_same_arms)]

            ///@@line_break
            use serde::{Serialize, Serializer, ser::SerializeMap};
        }
    }

    fn derive(&mut self, def: &TypeDef, schema: &Schema) -> TokenStream {
        if let TypeDef::Struct(def) = def {
            if def
                .markers
                .estree
                .as_ref()
                .and_then(|e| e.tag_mode.as_ref())
                .is_some_and(|e| e == &ESTreeStructTagMode::CustomSerialize)
            {
                return TokenStream::new();
            }
        }

        let body = match def {
            TypeDef::Enum(def) => serialize_enum(def),
            TypeDef::Struct(def) => serialize_struct(def, schema),
        };
        let ident = def.ident();

        let lifetime = if def.has_lifetime() { quote!(<'_>) } else { TokenStream::new() };
        quote! {
            impl Serialize for #ident #lifetime {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    #body
                }
            }
        }
    }
}

fn serialize_struct(def: &StructDef, schema: &Schema) -> TokenStream {
    if let Some(via) = &def.markers.estree.as_ref().and_then(|e| e.via.as_ref()) {
        let via: TokenStream = via.parse().unwrap();
        return quote! {
            #via::from(self).serialize(serializer)
        };
    }

    let ident = def.ident();
    // If type_tag is Some, we serialize it manually. If None, either one of
    // the fields is named r#type, or the struct does not need a "type" field.
    let type_tag = get_type_tag(def);

    let mut fields = vec![];
    if let Some(ty) = &type_tag {
        fields.push(quote! { map.serialize_entry("type", #ty)?; });
    }

    let mut append_to: FxHashMap<String, &FieldDef> = FxHashMap::default();

    // Scan through to find all append_to fields
    for field in &def.fields {
        let Some(parent) = field.markers.derive_attributes.estree.append_to.as_ref() else {
            continue;
        };
        assert!(
            append_to.insert(parent.clone(), field).is_none(),
            "Duplicate append_to target (on {ident})"
        );
    }

    for field in &def.fields {
        if field.markers.derive_attributes.estree.skip
            || field.markers.derive_attributes.estree.append_to.is_some()
        {
            continue;
        }
        let ident = field.ident().unwrap();
        let name = match &field.markers.derive_attributes.estree.rename {
            Some(rename) => rename.to_string(),
            None => field.name.clone().unwrap().to_case(Case::Camel),
        };
        assert!(
            !(name == "type" && type_tag.is_some()),
            "Unexpected r#type field when #[estree(type = ...)] is specified (on {ident})"
        );

        let ident = field.ident().unwrap();
        let always_flatten = match field.typ.type_id() {
            Some(id) => get_always_flatten_structs(schema).contains(&id),
            None => false,
        };

        let append_after = append_to.get(&ident.to_string());

        if always_flatten || field.markers.derive_attributes.estree.flatten {
            assert!(
                append_after.is_none(),
                "Cannot flatten and append to the same field (on {ident})"
            );
            fields.push(quote! {
                self.#ident.serialize(
                    serde::__private::ser::FlatMapSerializer(&mut map)
                )?;
            });
        } else if let Some(append_after) = append_after {
            let after_ident = append_after.ident().unwrap();
            fields.push(quote! {
                map.serialize_entry(
                    #name,
                    &oxc_estree::ser::AppendTo {
                        array: &self.#ident,
                        after: &self.#after_ident
                    }
                )?;
            });
        } else if let Some(via) = &field.markers.derive_attributes.estree.via {
            let via_tokens: TokenStream = via.parse().unwrap();
            fields.push(quote! {
                map.serialize_entry(
                    #name,
                    &#via_tokens(&self.#ident)
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

    let is_untagged = def.all_variants().all(|var| var.fields.len() == 1);

    if is_untagged {
        let match_branches = def.all_variants().map(|var| {
            let var_ident = var.ident();
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
