use std::{iter::FusedIterator, ops::Range, slice};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::utils::{create_ident, pluralize, snake_case};

use super::{
    Containers, Def, Derives, File, FileId, Schema, TypeDef, TypeId, Visibility,
    extensions::{
        ast_builder::AstBuilderType,
        clone_in::CloneInType,
        content_eq::ContentEqType,
        dummy::DummyEnum,
        estree::{ESTreeEnum, ESTreeEnumVariant},
        layout::{GetLayout, Layout},
        visit::{VisitEnum, VisitFieldOrVariant},
    },
};

pub type Discriminant = u8;

/// Type definition for an enum.
#[derive(Debug)]
pub struct EnumDef {
    pub id: TypeId,
    pub name: String,
    pub plural_name: Option<String>,
    pub has_lifetime: bool,
    #[expect(unused)]
    pub is_foreign: bool,
    pub file_id: FileId,
    pub containers: Containers,
    #[expect(unused)]
    pub visibility: Visibility,
    // For `#[derive(...)]` attributes.
    pub derives: Vec<String>,
    pub generated_derives: Derives,
    pub variants: Vec<VariantDef>,
    /// For `@inherits` inherited enum variants
    pub inherits: Vec<TypeId>,
    pub builder: AstBuilderType,
    pub visit: VisitEnum,
    pub layout: Layout,
    pub clone_in: CloneInType,
    pub dummy: DummyEnum,
    pub content_eq: ContentEqType,
    pub estree: ESTreeEnum,
}

impl EnumDef {
    /// Create new [`EnumDef`].
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
        variants: Vec<VariantDef>,
        inherits: Vec<TypeId>,
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
            variants,
            inherits,
            builder: AstBuilderType::default(),
            visit: VisitEnum::default(),
            layout: Layout::default(),
            clone_in: CloneInType::default(),
            dummy: DummyEnum::default(),
            content_eq: ContentEqType::default(),
            estree: ESTreeEnum::default(),
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

    /// Get iterator over all enum's variants (including inherited)
    pub fn all_variants<'s>(&'s self, schema: &'s Schema) -> AllVariantsIter<'s> {
        AllVariantsIter::new(self, schema)
    }

    /// Get iterator over enums this enum inherits from directly.
    ///
    /// To get all enums this enum inherits from, including transitively, use [`all_inherits`].
    ///
    /// [`all_inherits`]: Self::all_inherits
    pub fn inherits_enums<'s>(&'s self, schema: &'s Schema) -> impl Iterator<Item = &'s EnumDef> {
        self.inherits.iter().map(|&type_id| schema.enum_def(type_id))
    }

    /// Get iterator over all enums this enum inherits variants from, transitively, in depth-first pre-order.
    ///
    /// e.g.:
    ///
    /// * `AssignmentTarget` inherits `SimpleAssignmentTarget` and `AssignmentTargetPattern`.
    /// * `SimpleAssignmentTarget` inherits `MemberExpression`.
    ///
    /// The iterator yields: `SimpleAssignmentTarget`, `MemberExpression`, `AssignmentTargetPattern`.
    ///
    /// Unlike [`inherits_enums`], which yields only the directly-inherited enums.
    ///
    /// [`inherits_enums`]: Self::inherits_enums
    pub fn all_inherits<'s>(&'s self, schema: &'s Schema) -> AllInheritsIter<'s> {
        AllInheritsIter::new(self, schema)
    }

    /// Get whether all variants are fieldless.
    pub fn is_fieldless(&self) -> bool {
        // All AST enums are `#[repr(C, u8)]` or `#[repr(u8)]`.
        // Such enums must have at least 1 variant, so only way can have size 1
        // is if all variants are fieldless.
        self.layout_64().size == 1
    }

    /// Get the [`File`] which this struct is defined in.
    pub fn file<'s>(&self, schema: &'s Schema) -> &'s File {
        &schema.files[self.file_id]
    }

    /// Get iterator over variant indexes.
    ///
    /// Only includes own variant, not inherited.
    pub fn variant_indices(&self) -> Range<usize> {
        0..self.variants.len()
    }

    /// Get iterator over inherits indexes.
    pub fn inherits_indices(&self) -> Range<usize> {
        0..self.inherits.len()
    }
}

impl Def for EnumDef {
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
    /// Enums don't have a single inner type, so returns `None`.
    #[expect(unused_variables)]
    fn maybe_inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        None
    }
}

#[derive(Debug)]
pub struct VariantDef {
    pub name: String,
    pub field_type_id: Option<TypeId>,
    pub discriminant: Discriminant,
    pub visit: VisitFieldOrVariant,
    pub estree: ESTreeEnumVariant,
}

impl VariantDef {
    /// Create new [`VariantDef`].
    pub fn new(name: String, field_type_id: Option<TypeId>, discriminant: Discriminant) -> Self {
        Self {
            name,
            field_type_id,
            discriminant,
            visit: VisitFieldOrVariant::default(),
            estree: ESTreeEnumVariant::default(),
        }
    }

    /// Get variant name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get variant name in snake case.
    pub fn snake_name(&self) -> String {
        snake_case(self.name())
    }

    /// Get variant name in camel case.
    pub fn camel_name(&self) -> String {
        self.name().to_case(Case::Camel)
    }

    /// Get variant name as an [`Ident`].
    ///
    /// [`Ident`]: struct@Ident
    pub fn ident(&self) -> Ident {
        create_ident(self.name())
    }

    /// Get variant's field type.
    ///
    /// Returns `None` if variant is fieldless.
    pub fn field_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        self.field_type_id.map(|type_id| &schema.types[type_id])
    }

    /// Returns `true` if variant is fieldless.
    ///
    /// e.g. `enum Foo { Bar, Qux(u64) }`
    /// `Bar` variant is fieldless, `Qux` variant is not.
    pub fn is_fieldless(&self) -> bool {
        self.field_type_id.is_none()
    }
}

/// Iterator over all variants of an enum (including inherited).
///
/// Yields the enum's own variants first, then the own variants of each inherited enum in turn
/// (in the order produced by [`AllInheritsIter`]).
pub struct AllVariantsIter<'s> {
    /// Iterator over the current enum's own variants.
    /// Starts as the enum's own variants, then advances to each inherited enum's own variants.
    variants_iter: slice::Iter<'s, VariantDef>,
    /// Iterator over all inherited enums.
    inherits_iter: AllInheritsIter<'s>,
}

impl<'s> AllVariantsIter<'s> {
    /// Create new [`AllVariantsIter`].
    fn new(enum_def: &'s EnumDef, schema: &'s Schema) -> Self {
        Self {
            variants_iter: enum_def.variants.iter(),
            inherits_iter: enum_def.all_inherits(schema),
        }
    }
}

impl<'s> Iterator for AllVariantsIter<'s> {
    type Item = &'s VariantDef;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Yield from current enum's own variants
            if let Some(variant) = self.variants_iter.next() {
                return Some(variant);
            }
            // Current enum's variants exhausted - move on to next inherited enum's own variants
            self.variants_iter = self.inherits_iter.next()?.variants.iter();
        }
    }
}

impl FusedIterator for AllVariantsIter<'_> {}

/// Iterator over all enums this enum inherits variants from, transitively.
///
/// Yields in depth-first pre-order.
///
/// No enum is yielded more than once - the inheritance graph is a tree (no diamonds),
/// so the same enum cannot be reached via 2 different paths.
///
/// Created by [`EnumDef::all_inherits`].
pub struct AllInheritsIter<'s> {
    schema: &'s Schema,
    /// Stack of iterators over `inherits` lists, one per level of the depth-first traversal.
    stack: Vec<slice::Iter<'s, TypeId>>,
}

impl<'s> AllInheritsIter<'s> {
    /// Create new [`AllInheritsIter`].
    fn new(enum_def: &'s EnumDef, schema: &'s Schema) -> Self {
        Self { schema, stack: vec![enum_def.inherits.iter()] }
    }
}

impl<'s> Iterator for AllInheritsIter<'s> {
    type Item = &'s EnumDef;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next inherited `TypeId` from top of stack, popping iterators which are exhausted.
        // Returns `None` when the stack is empty (traversal complete).
        let type_id = loop {
            let iter = self.stack.last_mut()?;
            if let Some(&type_id) = iter.next() {
                break type_id;
            }
            self.stack.pop();
        };

        // Descend into this enum's own inherits next, for depth-first pre-order
        let enum_def = self.schema.enum_def(type_id);
        self.stack.push(enum_def.inherits.iter());
        Some(enum_def)
    }
}

impl FusedIterator for AllInheritsIter<'_> {}
