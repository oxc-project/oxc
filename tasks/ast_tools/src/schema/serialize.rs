use convert_case::{Case, Casing};
use rustc_hash::FxHashSet;

use super::{EnumDef, StructDef, TypeDef, VariantDef};
use crate::{markers::ESTreeStructTagMode, schema::GetIdent, Schema, TypeId};

pub fn enum_variant_name(var: &VariantDef, enm: &EnumDef) -> String {
    match var.markers.derive_attributes.estree.rename.as_ref() {
        Some(rename) => rename.to_string(),
        None => {
            if enm.markers.estree.no_rename_variants {
                var.ident().to_string()
            } else {
                var.ident().to_string().to_case(Case::Camel)
            }
        }
    }
}

pub fn get_type_tag(def: &StructDef) -> Option<String> {
    let tag_mode = def.markers.estree.as_ref().and_then(|e| e.tag_mode.as_ref());
    match tag_mode {
        Some(ESTreeStructTagMode::NoType) => None,
        Some(ESTreeStructTagMode::Type(type_name)) => Some(type_name.clone()),
        Some(ESTreeStructTagMode::CustomSerialize) | None => {
            let has_type_field =
                def.fields.iter().any(|f| matches!(f.name.as_deref(), Some("type")));
            if has_type_field {
                None
            } else {
                Some(def.ident().to_string())
            }
        }
    }
}

/// Returns a HashSet of structs that have the #[estree(always_flatten)] attribute.
pub fn get_always_flatten_structs(schema: &Schema) -> FxHashSet<TypeId> {
    let mut set = FxHashSet::default();
    for def in &schema.defs {
        if let TypeDef::Struct(def) = def {
            if def.markers.estree.as_ref().is_some_and(|e| e.always_flatten) {
                set.insert(def.id);
            }
        }
    }
    set
}
