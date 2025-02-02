use proc_macro2::TokenStream;
use quote::quote;

use super::{extensions::layout::Layout, Def, Derives, FileId, Schema, TypeDef, TypeId};

/// Type definition for a primitive type.
///
/// Includes:
/// * Built-ins e.g. `u8`, `&str`.
/// * Special Oxc types e.g. `ScopeId`, `Atom`.
#[derive(Debug)]
pub struct PrimitiveDef {
    pub id: TypeId,
    pub name: &'static str,
    pub layout: Layout,
}

impl PrimitiveDef {
    /// Create new [`PrimitiveDef`].
    pub fn new(name: &'static str) -> Self {
        Self { id: TypeId::DUMMY, name, layout: Layout::default() }
    }
}

impl Def for PrimitiveDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        self.id
    }

    /// Get type name.
    fn name(&self) -> &str {
        self.name
    }

    /// Get [`FileId`] of file containing definition of this type.
    ///
    /// Primitives are not defined in a file, so returns `None`.
    fn file_id(&self) -> Option<FileId> {
        None
    }

    /// Get all traits which have derives generated for this type.
    ///
    /// Primitives never have any generated derives.
    fn generated_derives(&self) -> Derives {
        Derives::none()
    }

    /// Get if type has a lifetime.
    #[expect(unused_variables)]
    fn has_lifetime(&self, schema: &Schema) -> bool {
        self.name() == "&str" || self.name() == "Atom"
    }

    /// Get type signature (including lifetimes).
    /// Lifetime is anonymous (`'_`) if `anon` is true.
    #[expect(unused_variables)]
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        match self.name() {
            "&str" => {
                if anon {
                    quote!(&str)
                } else {
                    quote!(&'a str)
                }
            }
            "Atom" => {
                if anon {
                    quote!(Atom<'_>)
                } else {
                    quote!(Atom<'a>)
                }
            }
            _ => {
                let ident = self.ident();
                quote!( #ident )
            }
        }
    }

    /// Get inner type, if type has one.
    ///
    /// Primitives don't have an inner type, so returns `None`.
    #[expect(unused_variables)]
    fn maybe_inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        None
    }
}
