use proc_macro2::TokenStream;
use quote::quote;

use super::{Containers, Def, Derives, Schema, TypeDef, TypeId, extensions::layout::Layout};

/// Type definition for a pointer.
///
/// NOTE: Support for pointers is incomplete.
/// `*const` and `*mut` are not implemented yet.
#[derive(Debug)]
pub struct PointerDef {
    pub id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
    pub kind: PointerKind,
    pub containers: Containers,
    pub layout: Layout,
}

/// Pointer kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerKind {
    NonNull,
    #[expect(dead_code)]
    Const,
    #[expect(dead_code)]
    Mut,
}

impl PointerDef {
    /// Create new [`PointerDef`].
    pub fn new(name: String, inner_type_id: TypeId, kind: PointerKind) -> Self {
        Self {
            id: TypeId::DUMMY,
            name,
            inner_type_id,
            kind,
            containers: Containers::default(),
            layout: Layout::default(),
        }
    }

    /// Get inner type.
    ///
    /// This is the direct inner type e.g. `NonNull<Option<u64>>` -> `Option<u64>`.
    /// Use [`innermost_type`] method if you want `u64` in this example.
    ///
    /// [`innermost_type`]: Self::innermost_type
    pub fn inner_type<'s>(&self, schema: &'s Schema) -> &'s TypeDef {
        &schema.types[self.inner_type_id]
    }
}

impl Def for PointerDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        self.id
    }

    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get all traits which have derives generated for this type.
    ///
    /// Pointers never have any generated derives.
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
        #[expect(clippy::unimplemented)]
        if self.kind != PointerKind::NonNull {
            unimplemented!();
        }

        let inner_ty = self.inner_type(schema).ty_with_lifetime(schema, anon);
        quote!( NonNull<#inner_ty> )
    }

    /// Get inner type, if type has one.
    ///
    /// All pointers have an inner type, so better to use [`inner_type`] or [`innermost_type`] methods,
    /// which don't return an `Option`.
    ///
    /// [`inner_type`]: Self::inner_type
    /// [`innermost_type`]: Self::innermost_type
    fn maybe_inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        Some(self.inner_type(schema))
    }
}
