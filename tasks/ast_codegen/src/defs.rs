use super::{REnum, RStruct, RType};
use crate::{layout::KnownLayout, schema::Inherit, util::TypeExt, TypeName};
use quote::ToTokens;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

impl TypeDef {
    pub fn name(&self) -> &String {
        match self {
            Self::Struct(it) => &it.name,
            Self::Enum(it) => &it.name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StructDef {
    pub name: TypeName,
    pub fields: Vec<FieldDef>,
    pub has_lifetime: bool,
    pub size_64: usize,
    pub align_64: usize,
    pub offsets_64: Option<Vec<usize>>,
    pub size_32: usize,
    pub align_32: usize,
    pub offsets_32: Option<Vec<usize>>,
}

#[derive(Debug, Serialize)]
pub struct EnumDef {
    pub name: TypeName,
    pub variants: Vec<EnumVariantDef>,
    /// For `@inherits` inherited enum variants
    pub inherits: Vec<EnumInheritDef>,
    pub has_lifetime: bool,
    pub size_64: usize,
    pub align_64: usize,
    pub offsets_64: Option<Vec<usize>>,
    pub size_32: usize,
    pub align_32: usize,
    pub offsets_32: Option<Vec<usize>>,
}

#[derive(Debug, Serialize)]
pub struct EnumVariantDef {
    pub name: TypeName,
    pub fields: Vec<FieldDef>,
    pub discriminant: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct EnumInheritDef {
    pub super_name: String,
    pub variants: Vec<EnumVariantDef>,
}

#[derive(Debug, Serialize)]
pub struct FieldDef {
    /// `None` if unnamed
    pub name: Option<String>,
    pub r#type: TypeName,
}

impl From<&RType> for Option<TypeDef> {
    fn from(rtype: &RType) -> Self {
        match rtype {
            RType::Enum(it) => Some(TypeDef::Enum(it.into())),
            RType::Struct(it) => Some(TypeDef::Struct(it.into())),
            _ => None,
        }
    }
}

impl From<&REnum> for EnumDef {
    fn from(it @ REnum { item, meta }: &REnum) -> Self {
        let (size_64, align_64, offsets_64) = meta
            .layout_64
            .clone()
            .layout()
            .map_or_else(|| panic!("Uncalculated layout on {}!", item.ident), KnownLayout::unpack);
        let (size_32, align_32, offsets_32) = meta
            .layout_32
            .clone()
            .layout()
            .map_or_else(|| panic!("Uncalculated layout on {}!", item.ident), KnownLayout::unpack);
        Self {
            name: it.ident().to_string(),
            variants: item.variants.iter().map(Into::into).collect(),
            inherits: meta.inherits.iter().map(Into::into).collect(),
            has_lifetime: item.generics.lifetimes().count() > 0,

            size_64,
            align_64,
            offsets_64,
            size_32,
            align_32,
            offsets_32,
        }
    }
}

impl From<&RStruct> for StructDef {
    fn from(it @ RStruct { item, meta }: &RStruct) -> Self {
        let (size_64, align_64, offsets_64) = meta
            .layout_64
            .clone()
            .layout()
            .map_or_else(|| panic!("Uncalculated layout on {}!", item.ident), KnownLayout::unpack);
        let (size_32, align_32, offsets_32) = meta
            .layout_32
            .clone()
            .layout()
            .map_or_else(|| panic!("Uncalculated layout on {}!", item.ident), KnownLayout::unpack);
        Self {
            name: it.ident().to_string(),
            fields: item.fields.iter().map(Into::into).collect(),
            has_lifetime: item.generics.lifetimes().count() > 0,

            size_64,
            align_64,
            offsets_64,
            size_32,
            align_32,
            offsets_32,
        }
    }
}

impl From<&syn::Variant> for EnumVariantDef {
    fn from(variant: &syn::Variant) -> Self {
        Self {
            name: variant.ident.to_string(),
            discriminant: variant.discriminant.as_ref().map(|(_, disc)| match disc {
                syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit), .. }) => {
                    lit.base10_parse().expect("invalid base10 enum discriminant")
                }
                _ => panic!("invalid enum discriminant"),
            }),
            fields: variant.fields.iter().map(Into::into).collect(),
        }
    }
}

impl From<&Inherit> for EnumInheritDef {
    fn from(inherit: &Inherit) -> Self {
        match inherit {
            Inherit::Linked { super_, variants } => Self {
                super_name: super_.get_ident().as_ident().unwrap().to_string(),
                variants: variants.iter().map(Into::into).collect(),
            },
            Inherit::Unlinked(_) => {
                panic!("`Unlinked` inherits can't be converted to a valid `EnumInheritDef`!")
            }
        }
    }
}

impl From<&syn::Field> for FieldDef {
    fn from(field: &syn::Field) -> Self {
        Self {
            name: field.ident.as_ref().map(ToString::to_string),
            r#type: field.ty.to_token_stream().to_string().replace(' ', ""),
        }
    }
}
