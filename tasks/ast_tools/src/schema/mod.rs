use std::{iter::FusedIterator, slice};

use rustc_hash::FxHashMap;
// Have to import this even though don't use it, due to a bug in `define_index_type!` macro
#[expect(unused_imports)]
use serde::Serialize;

use oxc_data_structures::slice_iter::SliceIter;
use oxc_index::{IndexVec, define_index_type};

mod defs;
mod derives;
mod file;
mod meta;
pub use defs::*;
pub use derives::Derives;
pub use file::File;
pub use meta::MetaType;

/// Extensions to schema for specific derives / generators
pub mod extensions {
    pub mod ast_builder;
    pub mod clone_in;
    pub mod content_eq;
    pub mod dummy;
    pub mod estree;
    pub mod kind;
    pub mod layout;
    pub mod span;
    pub mod visit;
}

define_index_type! {
    /// ID of type in the AST
    pub struct TypeId = u32;
}

impl TypeId {
    pub const DUMMY: Self = Self::from_raw_unchecked(0);
}

define_index_type! {
    /// ID of source file
    pub struct FileId = u32;
}

define_index_type! {
    /// ID of meta type
    pub struct MetaId = u32;
}

/// Schema of all AST types.
#[derive(Debug)]
pub struct Schema {
    /// Type definitions
    pub types: IndexVec<TypeId, TypeDef>,
    /// Mapping from type name to [`TypeId`]
    pub type_names: FxHashMap<String, TypeId>,
    /// Meta types
    pub metas: IndexVec<MetaId, MetaType>,
    /// Mapping from meta type name to [`MetaId`]
    pub meta_names: FxHashMap<String, MetaId>,
    /// Source files
    pub files: IndexVec<FileId, File>,
}

impl Schema {
    /// Get reference to [`TypeDef`] for a type name.
    ///
    /// # Panics
    /// Panics if no type with supplied name.
    pub fn type_by_name(&self, name: &str) -> &TypeDef {
        let type_id = self.type_names[name];
        &self.types[type_id]
    }

    /// Get mutable reference to [`TypeDef`] for a type name.
    ///
    /// # Panics
    /// Panics if no type with supplied name.
    pub fn type_by_name_mut(&mut self, name: &str) -> &mut TypeDef {
        let type_id = self.type_names[name];
        &mut self.types[type_id]
    }

    /// Get reference to [`MetaType`] for a meta type name.
    ///
    /// # Panics
    /// Panics if no type with supplied name.
    pub fn meta_by_name(&self, name: &str) -> &MetaType {
        let meta_id = self.meta_names[name];
        &self.metas[meta_id]
    }

    /// Get iterator over all structs and enums.
    #[expect(dead_code)]
    pub fn structs_and_enums(&self) -> StructsAndEnums<'_> {
        StructsAndEnums::new(self)
    }
}

/// Methods for getting a specific type def (e.g. [`StructDef`]) for a [`TypeId`].
///
/// These methods are useful in `Generator::prepare` / `Derive::prepare` where
/// you have to deal in [`TypeId`]s, to work around borrow-checker restrictions.
impl Schema {
    /// Get reference to [`StructDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a struct.
    pub fn struct_def(&self, type_id: TypeId) -> &StructDef {
        self.types[type_id].as_struct().unwrap()
    }

    /// Get mutable reference to [`StructDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a struct.
    pub fn struct_def_mut(&mut self, type_id: TypeId) -> &mut StructDef {
        self.types[type_id].as_struct_mut().unwrap()
    }

    /// Get reference to [`EnumDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not an enum.
    pub fn enum_def(&self, type_id: TypeId) -> &EnumDef {
        self.types[type_id].as_enum().unwrap()
    }

    /// Get mutable reference to [`EnumDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not an enum.
    pub fn enum_def_mut(&mut self, type_id: TypeId) -> &mut EnumDef {
        self.types[type_id].as_enum_mut().unwrap()
    }

    /// Get reference to [`PrimitiveDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a primitive.
    pub fn primitive_def(&self, type_id: TypeId) -> &PrimitiveDef {
        self.types[type_id].as_primitive().unwrap()
    }

    /// Get mutable reference to [`PrimitiveDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a primitive.
    pub fn primitive_def_mut(&mut self, type_id: TypeId) -> &mut PrimitiveDef {
        self.types[type_id].as_primitive_mut().unwrap()
    }

    /// Get reference to [`OptionDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not an `Option`.
    pub fn option_def(&self, type_id: TypeId) -> &OptionDef {
        self.types[type_id].as_option().unwrap()
    }

    /// Get mutable reference to [`OptionDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not an `Option`.
    pub fn option_def_mut(&mut self, type_id: TypeId) -> &mut OptionDef {
        self.types[type_id].as_option_mut().unwrap()
    }

    /// Get reference to [`BoxDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Box`.
    pub fn box_def(&self, type_id: TypeId) -> &BoxDef {
        self.types[type_id].as_box().unwrap()
    }

    /// Get mutable reference to [`BoxDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Box`.
    pub fn box_def_mut(&mut self, type_id: TypeId) -> &mut BoxDef {
        self.types[type_id].as_box_mut().unwrap()
    }

    /// Get reference to [`VecDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Vec`.
    pub fn vec_def(&self, type_id: TypeId) -> &VecDef {
        self.types[type_id].as_vec().unwrap()
    }

    /// Get mutable reference to [`VecDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Vec`.
    pub fn vec_def_mut(&mut self, type_id: TypeId) -> &mut VecDef {
        self.types[type_id].as_vec_mut().unwrap()
    }

    /// Get reference to [`CellDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Cell`.
    pub fn cell_def(&self, type_id: TypeId) -> &CellDef {
        self.types[type_id].as_cell().unwrap()
    }

    /// Get mutable reference to [`CellDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a `Cell`.
    pub fn cell_def_mut(&mut self, type_id: TypeId) -> &mut CellDef {
        self.types[type_id].as_cell_mut().unwrap()
    }

    /// Get reference to [`PointerDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a pointer.
    pub fn pointer_def(&self, type_id: TypeId) -> &PointerDef {
        self.types[type_id].as_pointer().unwrap()
    }

    /// Get mutable reference to [`PointerDef`] for a [`TypeId`].
    ///
    /// # Panics
    /// Panics if type [`TypeId`] refers to is not a pointer.
    pub fn pointer_def_mut(&mut self, type_id: TypeId) -> &mut PointerDef {
        self.types[type_id].as_pointer_mut().unwrap()
    }
}

/// Iterator over structs and enums.
pub struct StructsAndEnums<'s> {
    iter: slice::Iter<'s, TypeDef>,
}

impl<'s> StructsAndEnums<'s> {
    fn new(schema: &'s Schema) -> Self {
        Self { iter: schema.types.iter() }
    }
}

impl<'s> Iterator for StructsAndEnums<'s> {
    type Item = StructOrEnum<'s>;

    fn next(&mut self) -> Option<StructOrEnum<'s>> {
        if let Some(type_def) = self.iter.next() {
            match type_def {
                TypeDef::Struct(struct_def) => Some(StructOrEnum::Struct(struct_def)),
                TypeDef::Enum(enum_def) => Some(StructOrEnum::Enum(enum_def)),
                _ => {
                    // Structs and enums are always first in `Schema::types`,
                    // so if we encounter a different type, iteration is done.
                    self.iter.advance_to_end();
                    None
                }
            }
        } else {
            None
        }
    }
}

impl FusedIterator for StructsAndEnums<'_> {}
