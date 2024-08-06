use serde::Serialize;

use super::{with_either, TypeName};
use crate::{
    markers::{ScopeAttr, ScopeMarkers, VisitMarkers},
    util::{ToIdent, TypeAnalysis, TypeWrapper},
    TypeId,
};

#[derive(Debug, Serialize)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

impl TypeDef {
    pub fn id(&self) -> TypeId {
        with_either!(self, it => it.id)
    }

    pub fn name(&self) -> &String {
        with_either!(self, it => &it.name)
    }

    pub fn visitable(&self) -> bool {
        with_either!(self, it => it.visitable)
    }
}

#[derive(Debug, Serialize)]
pub struct StructDef {
    pub id: TypeId,
    pub name: String,
    pub visitable: bool,
    pub fields: Vec<FieldDef>,
    pub has_lifetime: bool,
    pub size_64: usize,
    pub align_64: usize,
    pub offsets_64: Option<Vec<usize>>,
    pub size_32: usize,
    pub align_32: usize,
    pub offsets_32: Option<Vec<usize>>,
    #[serde(skip)]
    pub markers: OuterMarkers,
}

#[derive(Debug, Serialize)]
pub struct EnumDef {
    pub id: TypeId,
    pub name: String,
    pub visitable: bool,
    pub variants: Vec<VariantDef>,
    /// For `@inherits` inherited enum variants
    pub inherits: Vec<InheritDef>,
    pub has_lifetime: bool,
    pub size_64: usize,
    pub align_64: usize,
    pub offsets_64: Option<Vec<usize>>,
    pub size_32: usize,
    pub align_32: usize,
    pub offsets_32: Option<Vec<usize>>,
}

impl EnumDef {
    /// Returns an iterator that would first walk all "real" variants and moves onto inherited ones
    /// based on the inheritance order.
    pub fn all_variants(&self) -> impl Iterator<Item = &VariantDef> {
        self.variants.iter().chain(self.inherits.iter().flat_map(|it| it.variants.iter()))
    }
}

#[derive(Debug, Serialize)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub discriminant: u8,
    pub markers: InnerMarkers,
}

impl VariantDef {
    pub fn ident(&self) -> syn::Ident {
        self.name.to_ident()
    }
}

#[derive(Debug, Serialize)]
pub struct InheritDef {
    pub super_: TypeRef,
    pub variants: Vec<VariantDef>,
}

#[derive(Debug, Serialize)]
pub struct FieldDef {
    /// `None` if unnamed
    pub name: Option<String>,
    pub typ: TypeRef,
    pub markers: InnerMarkers,
    pub docs: Vec<String>,
}

impl FieldDef {
    pub fn ident(&self) -> Option<syn::Ident> {
        self.name.as_ref().map(ToIdent::to_ident)
    }
}

#[derive(Debug, Serialize)]
pub struct TypeRef {
    pub(super) id: Option<TypeId>,
    pub(super) name: TypeName,

    #[serde(skip)]
    pub(super) transparent_id: Option<TypeId>,

    #[serde(skip)]
    pub(super) raw: String,
    #[serde(skip)]
    pub(super) analysis: TypeAnalysis,
}

impl TypeRef {
    /// It is `None` for foreign types.
    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        self.id
    }

    /// Reflects the inner most type id of `Adt1<Adt2<...AdtN<T>>>`
    #[inline]
    pub fn transparent_type_id(&self) -> Option<TypeId> {
        self.transparent_id
    }

    /// Reflects the inner type id of `Box<T>`
    #[inline]
    pub fn name(&self) -> &TypeName {
        &self.name
    }

    #[inline]
    pub fn analysis(&self) -> &TypeAnalysis {
        &self.analysis
    }

    #[inline]
    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn is_str_slice(&self) -> bool {
        matches!(self.analysis().wrapper, TypeWrapper::Ref if self.name.inner_name() == "str")
    }
}

#[derive(Debug)]
pub struct OuterMarkers {
    pub scope: Option<ScopeAttr>,
}

#[derive(Debug, Serialize)]
pub struct InnerMarkers {
    /// marker that hints to fold span in here
    pub span: bool,
    #[serde(skip)]
    pub visit: Option<VisitMarkers>,
    #[serde(skip)]
    pub scope: Option<ScopeMarkers>,
}
