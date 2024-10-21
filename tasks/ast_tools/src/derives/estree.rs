use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

use super::{define_derive, Derive, DeriveOutput};
use crate::{
    codegen::LateCtx,
    markers::ESTreeStructAttribute,
    schema::{
        serialize::{enum_variant_name, get_type_tag},
        EnumDef, GetGenerics, GetIdent, StructDef, TypeDef, TypeName,
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
        let ts_type_def = match def {
            TypeDef::Enum(def) => typescript_enum(def),
            TypeDef::Struct(def) => Some(typescript_struct(def)),
        };
        let ts_type_def = if let Some(ts_type_def) = ts_type_def {
            quote! {
                #[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
                const TS_APPEND_CONTENT: &'static str = #ts_type_def;
            }
        } else {
            TokenStream::new()
        };

        if let TypeDef::Struct(def) = def {
            if def
                .markers
                .estree
                .as_ref()
                .is_some_and(|e| e == &ESTreeStructAttribute::CustomSerialize)
            {
                return ts_type_def;
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

            ///@@line_break
            #ts_type_def
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

// Untagged enums: "type Expression = BooleanLiteral | NullLiteral"
// Tagged enums: "type PropertyKind = 'init' | 'get' | 'set'"
fn typescript_enum(def: &EnumDef) -> Option<String> {
    if def.markers.estree.custom_ts_def {
        return None;
    }

    let union = if def.markers.estree.untagged {
        def.all_variants().map(|var| type_to_string(var.fields[0].typ.name())).join(" | ")
    } else {
        def.all_variants().map(|var| format!("'{}'", enum_variant_name(var, def))).join(" | ")
    };
    let ident = def.ident();
    Some(format!("export type {ident} = {union};"))
}

fn typescript_struct(def: &StructDef) -> String {
    let ident = def.ident();
    let mut fields = String::new();
    let mut extends = vec![];

    if let Some(type_tag) = get_type_tag(def) {
        fields.push_str(&format!("\n\ttype: '{type_tag}';"));
    }

    for field in &def.fields {
        if field.markers.derive_attributes.estree.skip {
            continue;
        }
        let ty = match &field.markers.derive_attributes.tsify_type {
            Some(ty) => ty.clone(),
            None => type_to_string(field.typ.name()),
        };

        if field.markers.derive_attributes.estree.flatten {
            extends.push(ty);
            continue;
        }

        let name = match &field.markers.derive_attributes.estree.rename {
            Some(rename) => rename.to_string(),
            None => field.name.clone().unwrap().to_case(Case::Camel),
        };

        fields.push_str(&format!("\n\t{name}: {ty};"));
    }
    let extends =
        if extends.is_empty() { String::new() } else { format!(" & {}", extends.join(" & ")) };
    format!("export type {ident} = ({{{fields}\n}}){extends};")
}

fn type_to_string(ty: &TypeName) -> String {
    match ty {
        TypeName::Ident(ident) => match ident.as_str() {
            "f64" | "f32" | "usize" | "u64" | "u32" | "u16" | "u8" | "i64" | "i32" | "i16"
            | "i8" => "number",
            "bool" => "boolean",
            "str" | "String" | "Atom" | "CompactStr" => "string",
            ty => ty,
        }
        .to_string(),
        TypeName::Vec(type_name) => format!("Array<{}>", type_to_string(type_name)),
        TypeName::Box(type_name) | TypeName::Ref(type_name) | TypeName::Complex(type_name) => {
            type_to_string(type_name)
        }
        TypeName::Opt(type_name) => format!("({}) | null", type_to_string(type_name)),
    }
}
