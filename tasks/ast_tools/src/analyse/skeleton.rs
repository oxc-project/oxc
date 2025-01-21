use syn::{ItemEnum, ItemStruct};

use super::schema::FileId;

#[derive(Debug)]
pub enum Skeleton {
    Struct(StructSkeleton),
    Enum(EnumSkeleton),
}

#[derive(Debug)]
pub struct StructSkeleton {
    pub name: String,
    pub item: ItemStruct,
    pub file_id: FileId,
}

#[derive(Debug)]
pub struct EnumSkeleton {
    pub name: String,
    pub item: ItemEnum,
    pub inherits: Vec<String>,
    pub file_id: FileId,
}
