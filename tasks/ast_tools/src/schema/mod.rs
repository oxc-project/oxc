use std::fmt;

use quote::ToTokens;
use rustc_hash::FxHashSet;
use serde::Serialize;
use syn::{
    punctuated::Punctuated, Attribute, Expr, ExprLit, Field, Ident, Lit, Meta, MetaNameValue,
    Token, Type, Variant,
};

use crate::{
    codegen::EarlyCtx,
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
impl fmt::Display for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(it) => write!(f, "{it}"),
            Self::Vec(it) => write!(f, "Vec<{it}>"),
            Self::Box(it) => write!(f, "Box<{it}>"),
            Self::Opt(it) => write!(f, "Option<{it}>"),
            Self::Ref(it) => write!(f, "&{it}"),
            Self::Complex(it) => write!(f, "{it}"),
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

fn parse_struct_outer_markers(attrs: &Vec<Attribute>) -> Result<StructOuterMarkers> {
    Ok(StructOuterMarkers {
        scope: get_scope_attribute(attrs).transpose()?,
        estree: get_estree_attribute(attrs).transpose()?,
    })
}

fn parse_enum_outer_markers(attrs: &Vec<Attribute>) -> Result<EnumOuterMarkers> {
    Ok(EnumOuterMarkers { estree: get_estree_attribute(attrs).transpose()?.unwrap_or_default() })
}

fn parse_inner_markers(attrs: &Vec<Attribute>) -> Result<InnerMarkers> {
    Ok(InnerMarkers {
        span: attrs.iter().any(|a| a.path().is_ident("span")),
        visit: get_visit_markers(attrs)?,
        scope: get_scope_markers(attrs)?,
        derive_attributes: get_derive_attributes(attrs)?,
    })
}

// lower `AstType` to `TypeDef`.
pub fn lower_ast_types(ctx: &EarlyCtx) -> Schema {
    let defs = ctx
        .mods()
        .borrow()
        .iter()
        .flat_map(|it| &it.items)
        .map(|it| lower_ast_type(&it.borrow(), ctx))
        .collect();
    Schema { defs }
}

fn lower_ast_type(ty: &rust::AstType, ctx: &EarlyCtx) -> TypeDef {
    match ty {
        rust::AstType::Enum(it) => TypeDef::Enum(lower_ast_enum(it, ctx)),
        rust::AstType::Struct(it) => TypeDef::Struct(lower_ast_struct(it, ctx)),
        rust::AstType::Macro(it) => panic!("{}", unexpanded_macro_err(&it.item)),
    }
}

fn lower_ast_enum(it @ rust::Enum { item, meta }: &rust::Enum, ctx: &EarlyCtx) -> EnumDef {
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
        is_visitable: meta.is_visitable,
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

fn lower_ast_struct(it @ rust::Struct { item, meta }: &rust::Struct, ctx: &EarlyCtx) -> StructDef {
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
        is_visitable: meta.is_visitable,
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

fn lower_variant<F>(variant: &Variant, enum_dbg_name: F, ctx: &EarlyCtx) -> VariantDef
where
    F: Fn() -> String,
{
    VariantDef {
        name: variant.ident.to_string(),
        discriminant: variant.discriminant.as_ref().map_or_else(
            || panic!("expected explicit enum discriminants on {}", enum_dbg_name()),
            |(_, disc)| match disc {
                Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) => {
                    lit.base10_parse().expect("invalid base10 enum discriminant")
                }
                _ => panic!("invalid enum discriminant {:?} on {}", disc, enum_dbg_name()),
            },
        ),
        fields: variant.fields.iter().map(|fi| lower_field(fi, ctx)).collect(),
        markers: parse_inner_markers(&variant.attrs).unwrap(),
    }
}

fn lower_inherit(inherit: &rust::Inherit, ctx: &EarlyCtx) -> InheritDef {
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

fn lower_field(field: &Field, ctx: &EarlyCtx) -> FieldDef {
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

fn create_type_ref(ty: &Type, ctx: &EarlyCtx) -> TypeRef {
    let ident = ty.get_ident();
    let id = ident.as_ident().and_then(|id| ctx.type_id(&id.to_string()));
    let transparent_id = ctx.type_id(&ident.inner_ident().to_string());
    #[expect(clippy::disallowed_methods)]
    let raw = ty.to_token_stream().to_string().replace(' ', "");
    TypeRef {
        id,
        transparent_id,
        raw,
        name: TypeName::from(ty.get_ident()),
        analysis: ty.analyze(ctx),
    }
}

fn get_docs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if let Meta::NameValue(MetaNameValue { path, value: Expr::Lit(lit), .. }) = &attr.meta {
                if !path.is_ident("doc") {
                    return None;
                }
                match &lit.lit {
                    Lit::Str(lit) => Some(lit.value().trim().to_string()),
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect()
}

fn parse_generate_derive(attrs: &[Attribute]) -> Vec<String> {
    let mut derives = FxHashSet::default();
    for attr in attrs {
        if !attr.path().is_ident("generate_derive") {
            continue;
        }

        let args: Punctuated<Ident, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated).unwrap();

        for arg in args {
            derives.insert(arg.to_string());
        }
    }
    Vec::from_iter(derives)
}
