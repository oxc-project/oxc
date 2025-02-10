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

#[derive(Debug)]
pub struct StructSkeleton {
    pub name: String,
    pub file_id: FileId,
    pub item: ItemStruct,
}

#[derive(Debug)]
pub struct EnumSkeleton {
    pub name: String,
    pub file_id: FileId,
    pub item: ItemEnum,
    pub inherits: Vec<String>,
}
