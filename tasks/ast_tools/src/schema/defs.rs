use serde::Serialize;

use super::{with_either, TypeName};
use crate::{
    markers::{
        DeriveAttributes, ESTreeEnumAttribute, ESTreeStructAttribute, ScopeAttribute, ScopeMarkers,
        VisitMarkers,
    },
    util::{ToIdent, TypeAnalysis, TypeWrapper},
    TypeId,
};

#[derive(Debug, Serialize)]
#[serde(untagged)]
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

    pub fn generated_derives(&self) -> &Vec<String> {
        with_either!(self, it => &it.generated_derives)
    }

    pub fn generates_derive(&self, derive: &str) -> bool {
        let generated_derives = self.generated_derives();
        generated_derives.iter().any(|it| it == derive)
    }

    pub fn module_path(&self) -> &str {
        with_either!(self, it => &it.module_path)
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename = "struct", rename_all = "camelCase")]
pub struct StructDef {
    pub id: TypeId,
    pub name: String,
    #[serde(skip)]
    pub visitable: bool,
    pub fields: Vec<FieldDef>,
    #[serde(skip)]
    pub has_lifetime: bool,
    pub size_64: usize,
    pub align_64: usize,
    pub offsets_64: Option<Vec<usize>>,
    pub size_32: usize,
    pub align_32: usize,
    pub offsets_32: Option<Vec<usize>>,
    #[serde(skip)]
    pub generated_derives: Vec<String>,
    #[serde(skip)]
    pub markers: StructOuterMarkers,
    #[serde(skip)]
    pub module_path: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename = "enum", rename_all = "camelCase")]
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
    pub generated_derives: Vec<String>,
    #[serde(skip)]
    pub module_path: String,
    #[serde(skip)]
    pub markers: EnumOuterMarkers,
}

impl EnumDef {
    /// Returns an iterator that would first walk all "real" variants and moves onto inherited ones
    /// based on the inheritance order.
    pub fn all_variants(&self) -> impl Iterator<Item = &VariantDef> {
        self.variants.iter().chain(self.inherits.iter().flat_map(|it| it.variants.iter()))
    }

    /// Are all the variants in this enum unit?
    /// Example:
    /// ```
    /// enum E { A, B, C, D }
    ///
    /// ```
    ///
    pub fn is_unit(&self) -> bool {
        self.all_variants().all(VariantDef::is_unit)
    }
}

#[derive(Debug, Serialize)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub discriminant: u8,
    #[serde(skip)]
    pub markers: InnerMarkers,
}

impl VariantDef {
    pub fn ident(&self) -> syn::Ident {
        self.name.to_ident()
    }

    pub fn is_unit(&self) -> bool {
        self.fields.is_empty()
    }
}

#[derive(Debug, Serialize)]
pub struct InheritDef {
    #[serde(rename = "super")]
    pub super_: TypeRef,
    pub variants: Vec<VariantDef>,
}

#[derive(Debug, Serialize)]
pub struct FieldDef {
    /// `None` if unnamed
    pub name: Option<String>,
    #[serde(skip)]
    pub vis: Visibility,
    #[serde(rename = "type")]
    pub typ: TypeRef,
    #[serde(skip)]
    pub markers: InnerMarkers,
    #[serde(skip)]
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    None,
    Pub,
    /// rest of the restricted visibilities
    Rest,
}

impl Visibility {
    pub fn is_pub(&self) -> bool {
        matches!(self, Self::Pub)
    }
}

impl From<&syn::Visibility> for Visibility {
    fn from(vis: &syn::Visibility) -> Self {
        match vis {
            syn::Visibility::Public(_) => Self::Pub,
            syn::Visibility::Inherited => Self::None,
            syn::Visibility::Restricted(_) => Self::Rest,
        }
    }
}

impl FieldDef {
    pub fn ident(&self) -> Option<syn::Ident> {
        self.name.as_ref().map(ToIdent::to_ident)
    }
}

#[derive(Debug, Serialize)]
pub struct TypeRef {
    #[serde(skip)]
    pub(super) id: Option<TypeId>,
    pub(super) name: TypeName,

    #[serde(rename = "id")]
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
pub struct StructOuterMarkers {
    pub scope: Option<ScopeAttribute>,
    pub estree: Option<ESTreeStructAttribute>,
}

#[derive(Debug)]
pub struct EnumOuterMarkers {
    pub estree: ESTreeEnumAttribute,
}

#[derive(Debug, Serialize)]
pub struct InnerMarkers {
    /// marker that hints to fold span in here
    pub span: bool,
    pub derive_attributes: DeriveAttributes,
    #[serde(skip)]
    pub visit: VisitMarkers,
    #[serde(skip)]
    pub scope: ScopeMarkers,
}
