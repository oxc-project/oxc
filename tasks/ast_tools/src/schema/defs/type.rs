use proc_macro2::TokenStream;

use super::{
    BoxDef, CellDef, Def, Derives, EnumDef, FileId, OptionDef, PrimitiveDef, Schema, StructDef,
    TypeId, VecDef,
};

/// Type definition for a type.
#[derive(Debug)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Primitive(PrimitiveDef),
    Option(OptionDef),
    Box(BoxDef),
    Vec(VecDef),
    Cell(CellDef),
}

impl Def for TypeDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        match self {
            TypeDef::Struct(def) => def.id(),
            TypeDef::Enum(def) => def.id(),
            TypeDef::Primitive(def) => def.id(),
            TypeDef::Option(def) => def.id(),
            TypeDef::Box(def) => def.id(),
            TypeDef::Vec(def) => def.id(),
            TypeDef::Cell(def) => def.id(),
        }
    }

    /// Get type name.
    fn name(&self) -> &str {
        match self {
            TypeDef::Struct(def) => def.name(),
            TypeDef::Enum(def) => def.name(),
            TypeDef::Primitive(def) => def.name(),
            TypeDef::Option(def) => def.name(),
            TypeDef::Box(def) => def.name(),
            TypeDef::Vec(def) => def.name(),
            TypeDef::Cell(def) => def.name(),
        }
    }

    /// Get [`FileId`] of file containing definition of this type.
    ///
    /// Returns `None` if type is not defined in a file (e.g. primitives).
    fn file_id(&self) -> Option<FileId> {
        match self {
            TypeDef::Struct(def) => def.file_id(),
            TypeDef::Enum(def) => def.file_id(),
            TypeDef::Primitive(def) => def.file_id(),
            TypeDef::Option(def) => def.file_id(),
            TypeDef::Box(def) => def.file_id(),
            TypeDef::Vec(def) => def.file_id(),
            TypeDef::Cell(def) => def.file_id(),
        }
    }

    /// Get all traits which have derives generated for this type.
    fn generated_derives(&self) -> Derives {
        match self {
            TypeDef::Struct(def) => def.generated_derives(),
            TypeDef::Enum(def) => def.generated_derives(),
            TypeDef::Primitive(def) => def.generated_derives(),
            TypeDef::Option(def) => def.generated_derives(),
            TypeDef::Box(def) => def.generated_derives(),
            TypeDef::Vec(def) => def.generated_derives(),
            TypeDef::Cell(def) => def.generated_derives(),
        }
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool {
        match self {
            TypeDef::Struct(def) => def.has_lifetime(schema),
            TypeDef::Enum(def) => def.has_lifetime(schema),
            TypeDef::Primitive(def) => def.has_lifetime(schema),
            TypeDef::Option(def) => def.has_lifetime(schema),
            TypeDef::Box(def) => def.has_lifetime(schema),
            TypeDef::Vec(def) => def.has_lifetime(schema),
            TypeDef::Cell(def) => def.has_lifetime(schema),
        }
    }

    /// Get type signature (including anonymous lifetimes).
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        match self {
            TypeDef::Struct(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Enum(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Primitive(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Option(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Box(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Vec(def) => def.ty_with_lifetime(schema, anon),
            TypeDef::Cell(def) => def.ty_with_lifetime(schema, anon),
        }
    }

    /// Get inner type, if type has one.
    ///
    /// This is the direct inner type e.g. `Cell<Option<ScopeId>>` -> `Option<ScopeId>`.
    /// Use [`innermost_type`] method if you want `ScopeId` in this example.
    ///
    /// Returns `None` for types which don't have a single inner type (structs, enums, and primitives).
    ///
    /// [`innermost_type`]: Self::innermost_type
    fn maybe_inner_type<'s>(&self, schema: &'s Schema) -> Option<&'s TypeDef> {
        match self {
            TypeDef::Struct(def) => def.maybe_inner_type(schema),
            TypeDef::Enum(def) => def.maybe_inner_type(schema),
            TypeDef::Primitive(def) => def.maybe_inner_type(schema),
            TypeDef::Option(def) => def.maybe_inner_type(schema),
            TypeDef::Box(def) => def.maybe_inner_type(schema),
            TypeDef::Vec(def) => def.maybe_inner_type(schema),
            TypeDef::Cell(def) => def.maybe_inner_type(schema),
        }
    }
}

/// `is_*` / `as_*` / `as_*_mut` methods.
impl TypeDef {
    #[expect(dead_code)]
    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    pub fn as_struct(&self) -> Option<&StructDef> {
        match self {
            Self::Struct(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_struct_mut(&mut self) -> Option<&mut StructDef> {
        match self {
            Self::Struct(def) => Some(def),
            _ => None,
        }
    }

    #[expect(dead_code)]
    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    pub fn as_enum(&self) -> Option<&EnumDef> {
        match self {
            Self::Enum(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_enum_mut(&mut self) -> Option<&mut EnumDef> {
        match self {
            Self::Enum(def) => Some(def),
            _ => None,
        }
    }

    #[expect(dead_code)]
    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    pub fn as_primitive(&self) -> Option<&PrimitiveDef> {
        match self {
            Self::Primitive(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_primitive_mut(&mut self) -> Option<&mut PrimitiveDef> {
        match self {
            Self::Primitive(def) => Some(def),
            _ => None,
        }
    }

    #[expect(dead_code)]
    pub fn is_option(&self) -> bool {
        matches!(self, Self::Option(_))
    }

    pub fn as_option(&self) -> Option<&OptionDef> {
        match self {
            Self::Option(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_option_mut(&mut self) -> Option<&mut OptionDef> {
        match self {
            Self::Option(def) => Some(def),
            _ => None,
        }
    }

    pub fn is_box(&self) -> bool {
        matches!(self, Self::Box(_))
    }

    pub fn as_box(&self) -> Option<&BoxDef> {
        match self {
            Self::Box(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_box_mut(&mut self) -> Option<&mut BoxDef> {
        match self {
            Self::Box(def) => Some(def),
            _ => None,
        }
    }

    #[expect(dead_code)]
    pub fn is_vec(&self) -> bool {
        matches!(self, Self::Vec(_))
    }

    pub fn as_vec(&self) -> Option<&VecDef> {
        match self {
            Self::Vec(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_vec_mut(&mut self) -> Option<&mut VecDef> {
        match self {
            Self::Vec(def) => Some(def),
            _ => None,
        }
    }

    #[expect(dead_code)]
    pub fn is_cell(&self) -> bool {
        matches!(self, Self::Cell(_))
    }

    pub fn as_cell(&self) -> Option<&CellDef> {
        match self {
            Self::Cell(def) => Some(def),
            _ => None,
        }
    }

    pub fn as_cell_mut(&mut self) -> Option<&mut CellDef> {
        match self {
            Self::Cell(def) => Some(def),
            _ => None,
        }
    }
}
