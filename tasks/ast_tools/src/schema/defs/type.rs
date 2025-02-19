use proc_macro2::TokenStream;

use super::{
    BoxDef, CellDef, Def, Derives, EnumDef, OptionDef, PrimitiveDef, Schema, StructDef, TypeId,
    VecDef,
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
            Self::Struct(def) => def.id(),
            Self::Enum(def) => def.id(),
            Self::Primitive(def) => def.id(),
            Self::Option(def) => def.id(),
            Self::Box(def) => def.id(),
            Self::Vec(def) => def.id(),
            Self::Cell(def) => def.id(),
        }
    }

    /// Get type name.
    fn name(&self) -> &str {
        match self {
            Self::Struct(def) => def.name(),
            Self::Enum(def) => def.name(),
            Self::Primitive(def) => def.name(),
            Self::Option(def) => def.name(),
            Self::Box(def) => def.name(),
            Self::Vec(def) => def.name(),
            Self::Cell(def) => def.name(),
        }
    }

    /// Get all traits which have derives generated for this type.
    fn generated_derives(&self) -> Derives {
        match self {
            Self::Struct(def) => def.generated_derives(),
            Self::Enum(def) => def.generated_derives(),
            Self::Primitive(def) => def.generated_derives(),
            Self::Option(def) => def.generated_derives(),
            Self::Box(def) => def.generated_derives(),
            Self::Vec(def) => def.generated_derives(),
            Self::Cell(def) => def.generated_derives(),
        }
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, schema: &Schema) -> bool {
        match self {
            Self::Struct(def) => def.has_lifetime(schema),
            Self::Enum(def) => def.has_lifetime(schema),
            Self::Primitive(def) => def.has_lifetime(schema),
            Self::Option(def) => def.has_lifetime(schema),
            Self::Box(def) => def.has_lifetime(schema),
            Self::Vec(def) => def.has_lifetime(schema),
            Self::Cell(def) => def.has_lifetime(schema),
        }
    }

    /// Get type signature (including anonymous lifetimes).
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        match self {
            Self::Struct(def) => def.ty_with_lifetime(schema, anon),
            Self::Enum(def) => def.ty_with_lifetime(schema, anon),
            Self::Primitive(def) => def.ty_with_lifetime(schema, anon),
            Self::Option(def) => def.ty_with_lifetime(schema, anon),
            Self::Box(def) => def.ty_with_lifetime(schema, anon),
            Self::Vec(def) => def.ty_with_lifetime(schema, anon),
            Self::Cell(def) => def.ty_with_lifetime(schema, anon),
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
            Self::Struct(def) => def.maybe_inner_type(schema),
            Self::Enum(def) => def.maybe_inner_type(schema),
            Self::Primitive(def) => def.maybe_inner_type(schema),
            Self::Option(def) => def.maybe_inner_type(schema),
            Self::Box(def) => def.maybe_inner_type(schema),
            Self::Vec(def) => def.maybe_inner_type(schema),
            Self::Cell(def) => def.maybe_inner_type(schema),
        }
    }
}

/// `is_*` / `as_*` / `as_*_mut` methods.
impl TypeDef {
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
