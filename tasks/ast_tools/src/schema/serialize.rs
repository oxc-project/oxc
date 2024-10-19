use super::{EnumDef, StructDef, VariantDef};
use crate::{markers::ESTreeStructAttribute, schema::GetIdent};
use convert_case::{Case, Casing};

pub fn enum_variant_name(var: &VariantDef, enm: &EnumDef) -> String {
    match var.markers.derive_attributes.estree.rename.as_ref() {
        Some(rename) => rename.to_string(),
        None => match enm.markers.estree.rename_all.as_deref() {
            Some("camelCase") => var.ident().to_string().to_case(Case::Camel),
            Some(case) => {
                panic!("Unsupported rename_all: {case} (on {})", enm.ident())
            }
            None => var.ident().to_string(),
        },
    }
}

pub fn get_type_tag(def: &StructDef) -> Option<String> {
    match &def.markers.estree {
        Some(ESTreeStructAttribute::NoType) => None,
        Some(ESTreeStructAttribute::Type(type_name)) => Some(type_name.clone()),
        Some(ESTreeStructAttribute::CustomSerialize) | None => {
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
