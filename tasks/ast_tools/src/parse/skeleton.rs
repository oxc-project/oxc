use syn::{ItemEnum, ItemStruct};

use crate::schema::FileId;

/// "Skeleton" parsed from type definition in source file.
///
/// Contains only very basic information - type name, [`syn`]'s parsed AST for the type,
/// [`FileId`] of the file this type is defined in, and names of any enums this enum inherits.
///
/// [`Skeleton`]s are created in first parsing pass, is contains the bare minimum required
/// to be able to link up the types in the 2nd pass.
#[derive(Debug)]
pub enum Skeleton {
    Struct(StructSkeleton),
    Enum(EnumSkeleton),
}

impl Skeleton {
    pub fn name(&self) -> &str {
        match self {
            Self::Struct(s) => &s.name,
            Self::Enum(e) => &e.name,
        }
    }

    pub fn is_meta(&self) -> bool {
        match self {
            Self::Struct(s) => s.is_meta,
            Self::Enum(e) => e.is_meta,
        }
    }
}

// These 2 structs are both `#[repr(C)]` so that `name` and `is_meta` are in same position in both,
// making the 2 methods above cheaper

#[repr(C)]
#[derive(Debug)]
pub struct StructSkeleton {
    pub name: String,
    pub file_id: FileId,
    pub is_foreign: bool,
    pub is_meta: bool,
    pub item: ItemStruct,
}

#[repr(C)]
#[derive(Debug)]
pub struct EnumSkeleton {
    pub name: String,
    pub file_id: FileId,
    pub is_foreign: bool,
    pub is_meta: bool,
    pub item: ItemEnum,
    pub inherits: Vec<String>,
}
