use std::ops::Range;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::{create_ident_tokens, pluralize};

use super::{
    Containers, Def, Derives, File, FileId, Schema, TypeDef, TypeId, Visibility,
    extensions::{
        ast_builder::{AstBuilderStructField, AstBuilderType},
        clone_in::{CloneInStructField, CloneInType},
        content_eq::{ContentEqStructField, ContentEqType},
        dummy::DummyStruct,
        estree::{ESTreeStruct, ESTreeStructField},
        kind::Kind,
        layout::{Layout, Offset},
        span::SpanStruct,
        visit::{VisitFieldOrVariant, VisitStruct},
    },
};

/// Type definition for a struct.
#[derive(Debug)]
pub struct StructDef {
    pub id: TypeId,
    pub name: String,
    pub plural_name: Option<String>,
    pub has_lifetime: bool,
    pub is_foreign: bool,
    pub file_id: FileId,
    pub containers: Containers,
    #[expect(unused)]
    pub visibility: Visibility,
    /// For `#[derive(...)]` attributes.
    pub derives: Vec<String>,
    pub generated_derives: Derives,
    pub fields: Vec<FieldDef>,
    pub builder: AstBuilderType,
    pub visit: VisitStruct,
    pub kind: Kind,
    pub layout: Layout,
    pub span: SpanStruct,
    pub clone_in: CloneInType,
    pub dummy: DummyStruct,
    pub content_eq: ContentEqType,
    pub estree: ESTreeStruct,
}

impl StructDef {
    /// Create new [`StructDef`].
    pub fn new(
        id: TypeId,
        name: String,
        plural_name: Option<String>,
        has_lifetime: bool,
        is_foreign: bool,
        file_id: FileId,
        visibility: Visibility,
        derives: Vec<String>,
        generated_derives: Derives,
        fields: Vec<FieldDef>,
    ) -> Self {
        Self {
            id,
            name,
            plural_name,
            has_lifetime,
            is_foreign,
            file_id,
            containers: Containers::default(),
            visibility,
            derives,
            generated_derives,
            fields,
            builder: AstBuilderType::default(),
            visit: VisitStruct::default(),
            kind: Kind::default(),
            layout: Layout::default(),
            span: SpanStruct::default(),
            clone_in: CloneInType::default(),
            dummy: DummyStruct::default(),
            content_eq: ContentEqType::default(),
            estree: ESTreeStruct::default(),
        }
    }

    /// Get plural type name.
    pub fn plural_name(&self) -> String {
        self.plural_name.clone().unwrap_or_else(|| pluralize(self.name()))
    }

    /// Get plural type name in snake case.
    pub fn plural_snake_name(&self) -> String {
        self.plural_name().to_case(Case::Snake)
    }

    /// Get the [`File`] which this struct is defined in.
    pub fn file<'s>(&self, schema: &'s Schema) -> &'s File {
        &schema.files[self.file_id]
    }

    /// Get iterator over field indexes.
    pub fn field_indices(&self) -> Range<usize> {
        0..self.fields.len()
    }

    /// Get reference to [`FieldDef`] for field called `name`.
    ///
    /// # Panics
    /// Panics if struct does not have a field called `name`.
    pub fn field_by_name(&self, name: &str) -> &FieldDef {
        self.fields.iter().find(|field| field.name() == name).unwrap()
    }
}

impl Def for StructDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        self.id
    }

    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get all traits which have derives generated for this type.
    fn generated_derives(&self) -> Derives {
        self.generated_derives
    }

    /// Get if type has a lifetime.
    #[expect(unused_variables)]
    fn has_lifetime(&self, schema: &Schema) -> bool {
        self.has_lifetime
    }

    /// Get type signature (including lifetime).
    /// Lifetime is anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let ident = self.ident();
        let lifetime = self.lifetime_maybe_anon(schema, anon);
        quote!( #ident #lifetime )
    }

    /// Get inner type, if type has one.
    ///
    /// Structs don't have a single inner type, so returns `None`.
    #[expect(unused_variables)]
    fn maybe_inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        None
    }
}

#[derive(Debug)]
pub struct FieldDef {
    pub name: String,
    pub type_id: TypeId,
    pub visibility: Visibility,
    pub doc_comment: Option<String>,
    pub builder: AstBuilderStructField,
    pub visit: VisitFieldOrVariant,
    pub offset: Offset,
    pub clone_in: CloneInStructField,
    pub content_eq: ContentEqStructField,
    pub estree: ESTreeStructField,
}

impl FieldDef {
    /// Create new [`FieldDef`].
    pub fn new(
        name: String,
        type_id: TypeId,
        visibility: Visibility,
        doc_comment: Option<String>,
    ) -> Self {
        Self {
            name,
            type_id,
            visibility,
            doc_comment,
            builder: AstBuilderStructField::default(),
            visit: VisitFieldOrVariant::default(),
            offset: Offset::default(),
            clone_in: CloneInStructField::default(),
            content_eq: ContentEqStructField::default(),
            estree: ESTreeStructField::default(),
        }
    }

    /// Get field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get field name in camel case.
    pub fn camel_name(&self) -> String {
        self.name().to_case(Case::Camel)
    }

    /// Get field name as an identifier.
    ///
    /// This is a [`TokenStream`] not `Ident`, to handle unnamed fields where field name is e.g. `0`.
    pub fn ident(&self) -> TokenStream {
        create_ident_tokens(self.name())
    }

    /// Get field type.
    pub fn type_def<'s>(&self, schema: &'s Schema) -> &'s TypeDef {
        &schema.types[self.type_id]
    }
}
