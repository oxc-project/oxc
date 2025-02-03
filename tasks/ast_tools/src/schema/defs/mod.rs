use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{codegen::DeriveId, utils::create_ident, Schema};

use super::{extensions, Derives, FileId, TypeId};

mod r#box;
mod cell;
mod r#enum;
mod option;
mod primitive;
mod r#struct;
mod r#type;
mod vec;
pub use cell::CellDef;
pub use option::OptionDef;
pub use primitive::PrimitiveDef;
pub use r#box::BoxDef;
pub use r#enum::{Discriminant, EnumDef, VariantDef};
pub use r#struct::{FieldDef, StructDef, Visibility};
pub use r#type::TypeDef;
pub use vec::VecDef;

/// Trait for type defs.
pub trait Def {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId;

    /// Get type name.
    fn name(&self) -> &str;

    /// Get [`FileId`] of file containing definition of this type.
    fn file_id(&self) -> Option<FileId>;

    /// Get all traits which have derives generated for this type.
    fn generated_derives(&self) -> Derives;

    /// Get whether a derive is generated for this type.
    fn generates_derive(&self, derive_id: DeriveId) -> bool {
        self.generated_derives().has(derive_id)
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool;

    /// Get type name in snake case.
    fn snake_name(&self) -> String {
        self.name().to_case(Case::Snake)
    }

    /// Get type name as an [`Ident`].
    ///
    /// [`Ident`]: struct@Ident
    fn ident(&self) -> Ident {
        create_ident(self.name())
    }

    /// Get type signature (including lifetimes).
    fn ty(&self, schema: &Schema) -> TokenStream {
        self.ty_with_lifetime(schema, false)
    }

    /// Get type signature (including anonymous lifetimes).
    fn ty_anon(&self, schema: &Schema) -> TokenStream {
        self.ty_with_lifetime(schema, true)
    }

    /// Get type signature (including lifetimes).
    /// Lifetimes are anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream;

    /// Get lifetime (if type has one).
    /// Lifetime is anonymous (`'_`) if `anon` is true.
    fn lifetime_maybe_anon(&self, schema: &Schema, anon: bool) -> TokenStream {
        if anon {
            self.lifetime_anon(schema)
        } else {
            self.lifetime(schema)
        }
    }

    /// Get lifetime (if type has one).
    fn lifetime(&self, schema: &Schema) -> TokenStream {
        if self.has_lifetime(schema) {
            quote!( <'a> )
        } else {
            quote!()
        }
    }

    /// Get anonymous lifetime (if type has one).
    fn lifetime_anon(&self, schema: &Schema) -> TokenStream {
        if self.has_lifetime(schema) {
            quote!( <'_> )
        } else {
            quote!()
        }
    }

    /// Get inner type, if type has one.
    ///
    /// This is the direct inner type e.g. `Cell<Option<ScopeId>>` -> `Option<ScopeId>`.
    /// Use [`innermost_type`] method if you want `ScopeId` in this example.
    ///
    /// Returns `None` for types which don't have a single inner type (structs, enums, and primitives).
    ///
    /// [`innermost_type`]: Def::innermost_type
    fn maybe_inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef>;

    /// Get innermost type.
    ///
    /// e.g. `ScopeId` in `Cell<Option<ScopeId>>`.
    ///
    /// Use [`inner_type`] method if you want the direct inner type (`Option<ScopeId>` in this example).
    ///
    /// [`inner_type`]: Def::innermost_type
    fn innermost_type<'s>(&self, schema: &'s Schema) -> &'s TypeDef {
        if let Some(inner_type) = self.maybe_inner_type(schema) {
            inner_type.innermost_type(schema)
        } else {
            &schema.types[self.id()]
        }
    }
}
