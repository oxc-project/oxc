use oxc_index::{define_index_type, IndexVec};
use rustc_hash::FxHashMap;
// Have to import this even though don't use it, due to a bug in `define_index_type!` macro
#[expect(unused_imports)]
use serde::Serialize;

mod defs;
mod derives;
mod file;
pub use defs::*;
pub use derives::Derives;
pub use file::File;

/// Extensions to schema for specific derives / generators
pub mod extensions {
    pub mod clone_in;
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

/// Schema of all AST types.
#[derive(Debug)]
pub struct Schema {
    /// Type definitions
    pub types: IndexVec<TypeId, TypeDef>,
    /// Mapping from type name to [`TypeId`]
    pub type_names: FxHashMap<String, TypeId>,
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
}
