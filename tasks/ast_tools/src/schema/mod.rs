use quote::ToTokens;
use rustc_hash::FxHashSet;
use serde::Serialize;

use crate::{
    codegen,
    layout::KnownLayout,
    markers::{
        get_derive_attributes, get_estree_attribute, get_scope_attribute, get_scope_markers,
        get_visit_markers,
    },
    rust_ast as rust,
    util::{unexpanded_macro_err, TypeExt},
    Result, TypeId,
};

mod defs;
mod get_generics;
mod get_ident;
pub mod serialize;
mod to_type;
pub use defs::*;
pub use get_generics::GetGenerics;
pub use get_ident::GetIdent;
pub use to_type::ToType;

#[derive(Debug, Serialize)]
pub enum TypeName {
    Ident(String),
    Vec(Box<TypeName>),
    Box(Box<TypeName>),
    Opt(Box<TypeName>),
    Ref(Box<TypeName>),
    /// We bailed on detecting wrapper
    Complex(Box<TypeName>),
}

impl TypeName {
    pub fn inner_name(&self) -> &str {
        match self {
            Self::Ident(it) => it,
            Self::Complex(it) | Self::Vec(it) | Self::Box(it) | Self::Opt(it) | Self::Ref(it) => {
                it.inner_name()
            }
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        if let Self::Ident(it) = self {
            Some(it)
        } else {
            None
        }
    }

    /// First identifier of multi-part type
    ///
    /// `Adt<T>`
    ///  ^^^
    ///
    /// Panics
    ///
    /// When `self` is `TypeName::Ref` or `TypeName::Complex`.
    ///
    pub fn first_ident(&self) -> &str {
        match self {
            Self::Ident(it) => it.as_str(),
            Self::Vec(_) => "Vec",
            Self::Box(_) => "Box",
            Self::Opt(_) => "Option",
            Self::Ref(_) | Self::Complex(_) => panic!(),
        }
    }
}

impl<'a> From<crate::util::TypeIdentResult<'a>> for TypeName {
    fn from(it: crate::util::TypeIdentResult<'a>) -> Self {
        use crate::util::TypeIdentResult;
        match it {
            TypeIdentResult::Ident(it) => Self::Ident(it.to_string()),
            TypeIdentResult::Vec(it) => Self::Vec(Box::new(Self::from(*it))),
            TypeIdentResult::Box(it) => Self::Box(Box::new(Self::from(*it))),
            TypeIdentResult::Option(it) => Self::Opt(Box::new(Self::from(*it))),
            TypeIdentResult::Reference(it) => Self::Ref(Box::new(Self::from(*it))),
            TypeIdentResult::Complex(it) => Self::Complex(Box::new(Self::from(*it))),
        }
    }
}

#[derive(Debug, Default, serde::Serialize)]
pub struct Schema {
    pub defs: Vec<TypeDef>,
}

impl Schema {
    pub fn get(&self, id: TypeId) -> Option<&TypeDef> {
        self.defs.get(id)
    }
}

impl<'a> IntoIterator for &'a Schema {
    type IntoIter = std::slice::Iter<'a, TypeDef>;
    type Item = &'a TypeDef;

    fn into_iter(self) -> Self::IntoIter {
        self.defs.iter()
    }
}

fn parse_struct_outer_markers(attrs: &Vec<syn::Attribute>) -> Result<StructOuterMarkers> {
    Ok(StructOuterMarkers {
        scope: get_scope_attribute(attrs).transpose()?,
        estree: get_estree_attribute(attrs).transpose()?,
    })
}

fn parse_enum_outer_markers(attrs: &Vec<syn::Attribute>) -> Result<EnumOuterMarkers> {
    Ok(EnumOuterMarkers { estree: get_estree_attribute(attrs).transpose()?.unwrap_or_default() })
}

fn parse_inner_markers(attrs: &Vec<syn::Attribute>) -> Result<InnerMarkers> {
    Ok(InnerMarkers {
        span: attrs.iter().any(|a| a.path().is_ident("span")),
        visit: get_visit_markers(attrs)?,
        scope: get_scope_markers(attrs)?,
        derive_attributes: get_derive_attributes(attrs)?,
    })
}

// lower `AstType` to `TypeDef`.
pub fn lower_ast_types(ctx: &codegen::EarlyCtx) -> Schema {
    let defs = ctx
        .mods()
        .borrow()
        .iter()
        .flat_map(|it| &it.items)
        .map(|it| lower_ast_type(&it.borrow(), ctx))
        .collect();
    Schema { defs }
}

fn lower_ast_type(ty: &rust::AstType, ctx: &codegen::EarlyCtx) -> TypeDef {
    match ty {
        rust::AstType::Enum(it) => TypeDef::Enum(lower_ast_enum(it, ctx)),
        rust::AstType::Struct(it) => TypeDef::Struct(lower_ast_struct(it, ctx)),
        rust::AstType::Macro(it) => panic!("{}", unexpanded_macro_err(&it.item)),
    }
}

fn lower_ast_enum(it @ rust::Enum { item, meta }: &rust::Enum, ctx: &codegen::EarlyCtx) -> EnumDef {
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
    EnumDef {
        id: ctx.type_id(&it.ident().to_string()).unwrap(),
        name: it.ident().to_string(),
        visitable: meta.visitable,
        variants: item
            .variants
            .iter()
            .filter(|it| !it.attrs.iter().any(|it| it.path().is_ident("inherit")))
            .map(|var| lower_variant(var, || it.ident().to_string(), ctx))
            .collect(),
        inherits: meta.inherits.iter().map(|it| lower_inherit(it, ctx)).collect(),
        has_lifetime: item.generics.lifetimes().count() > 0,

        size_64,
        align_64,
        offsets_64,
        size_32,
        align_32,
        offsets_32,

        markers: parse_enum_outer_markers(&item.attrs).unwrap(),
        generated_derives: parse_generate_derive(&item.attrs),

        module_path: meta.module_path.clone(),
    }
}

fn lower_ast_struct(
    it @ rust::Struct { item, meta }: &rust::Struct,
    ctx: &codegen::EarlyCtx,
) -> StructDef {
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
    StructDef {
        id: ctx.type_id(&it.ident().to_string()).unwrap(),
        name: it.ident().to_string(),
        visitable: meta.visitable,
        fields: item.fields.iter().map(|fi| lower_field(fi, ctx)).collect(),
        has_lifetime: item.generics.lifetimes().count() > 0,

        size_64,
        align_64,
        offsets_64,
        size_32,
        align_32,
        offsets_32,

        markers: parse_struct_outer_markers(&item.attrs).unwrap(),
        generated_derives: parse_generate_derive(&item.attrs),

        module_path: meta.module_path.clone(),
    }
}

fn lower_variant<F>(variant: &syn::Variant, enum_dbg_name: F, ctx: &codegen::EarlyCtx) -> VariantDef
where
    F: Fn() -> String,
{
    VariantDef {
        name: variant.ident.to_string(),
        discriminant: variant.discriminant.as_ref().map_or_else(
            || panic!("expected explicit enum discriminants on {}", enum_dbg_name()),
            |(_, disc)| match disc {
                syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit), .. }) => {
                    lit.base10_parse().expect("invalid base10 enum discriminant")
                }
                _ => panic!("invalid enum discriminant {:?} on {}", disc, enum_dbg_name()),
            },
        ),
        fields: variant.fields.iter().map(|fi| lower_field(fi, ctx)).collect(),
        markers: parse_inner_markers(&variant.attrs).unwrap(),
    }
}

fn lower_inherit(inherit: &rust::Inherit, ctx: &codegen::EarlyCtx) -> InheritDef {
    match inherit {
        rust::Inherit::Linked { super_, variants } => InheritDef {
            super_: create_type_ref(super_, ctx),
            variants: variants
                .iter()
                .map(|var| lower_variant(var, || super_.get_ident().inner_ident().to_string(), ctx))
                .collect(),
        },
        rust::Inherit::Unlinked(_) => {
            panic!("`Unlinked` inherits can't be converted to a valid `InheritDef`!")
        }
    }
}

fn lower_field(field: &syn::Field, ctx: &codegen::EarlyCtx) -> FieldDef {
    FieldDef {
        name: field
            .ident
            .as_ref()
            .map(|ident| ident.to_string().trim_start_matches("r#").to_string()),
        vis: Visibility::from(&field.vis),
        typ: create_type_ref(&field.ty, ctx),
        markers: parse_inner_markers(&field.attrs).unwrap(),
        docs: get_docs(&field.attrs),
    }
}

fn create_type_ref(ty: &syn::Type, ctx: &codegen::EarlyCtx) -> TypeRef {
    let ident = ty.get_ident();
    let id = ident.as_ident().and_then(|id| ctx.type_id(&id.to_string()));
    let transparent_id = ctx.type_id(&ident.inner_ident().to_string());
    let raw = ty.to_token_stream().to_string().replace(' ', "");
    TypeRef {
        id,
        transparent_id,
        raw,
        name: TypeName::from(ty.get_ident()),
        analysis: ty.analyze(ctx),
    }
}

fn get_docs(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if let syn::Meta::NameValue(syn::MetaNameValue {
                path,
                value: syn::Expr::Lit(lit),
                ..
            }) = &attr.meta
            {
                if !path.is_ident("doc") {
                    return None;
                }
                match &lit.lit {
                    syn::Lit::Str(lit) => Some(lit.value().trim().to_string()),
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect()
}

fn parse_generate_derive(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut derives = FxHashSet::default();
    for attr in attrs {
        if !attr.path().is_ident("generate_derive") {
            continue;
        }

        let args: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> =
            attr.parse_args_with(syn::punctuated::Punctuated::parse_terminated).unwrap();

        for arg in args {
            derives.insert(arg.to_string());
        }
    }
    Vec::from_iter(derives)
}

macro_rules! with_either {
    ($def:expr, $it:ident => $body:expr) => {
        match $def {
            TypeDef::Struct($it) => $body,
            TypeDef::Enum($it) => $body,
        }
    };
}

use with_either;
