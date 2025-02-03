use proc_macro2::TokenStream;
use quote::quote;

use super::{extensions::layout::Layout, Def, Derives, FileId, Schema, TypeDef, TypeId};

/// Type definition for an `Option`.
#[derive(Debug)]
pub struct OptionDef {
    pub id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
    pub layout: Layout,
}

impl OptionDef {
    /// Create new [`OptionDef`].
    pub fn new(name: String, inner_type_id: TypeId) -> Self {
        Self { id: TypeId::DUMMY, name, inner_type_id, layout: Layout::default() }
    }

    /// Get inner type.
    ///
    /// This is the direct inner type e.g. `Option<Box<FunctionBody>>` -> `Box<FunctionBody>`.
    /// Use [`innermost_type`] method if you want `FunctionBody` in this example.
    ///
    /// [`innermost_type`]: Self::innermost_type
    pub fn inner_type<'s>(&self, schema: &'s Schema) -> &'s TypeDef {
        &schema.types[self.inner_type_id]
    }
}

impl Def for OptionDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        self.id
    }

    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get [`FileId`] of file containing definition of this type.
    ///
    /// `Options`s are not defined in a file, so returns `None`.
    fn file_id(&self) -> Option<FileId> {
        None
    }

    /// Get all traits which have derives generated for this type.
    ///
    /// `Option`s never have any generated derives.
    fn generated_derives(&self) -> Derives {
        Derives::none()
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool {
        self.inner_type(schema).has_lifetime(schema)
    }

    /// Get type signature (including lifetimes).
    /// Lifetimes are anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let inner_ty = self.inner_type(schema).ty_with_lifetime(schema, anon);
        quote!( Option<#inner_ty> )
    }

    /// Get inner type, if type has one.
    ///
    /// All `Option`s have an inner type, so better to use [`inner_type`] or [`innermost_type`] methods,
    /// which don't return an `Option`.
    ///
    /// [`inner_type`]: Self::inner_type
    /// [`innermost_type`]: Self::innermost_type
    fn maybe_inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        Some(self.inner_type(schema))
    }
}
