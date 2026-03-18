//! Type representation for oxc's TypeScript type checker.
//!
//! This crate provides the core type data structures: `TypeId`, `TypeFlags`,
//! `ObjectFlags`, `TypeData`, and `TypeArena`. It does not contain any
//! type-checking logic itself — that lives in `oxc_checker`.

mod object_flags;
mod type_arena;
mod type_data;
mod type_flags;
mod type_id;

pub use object_flags::ObjectFlags;
pub use type_arena::TypeArena;
pub use type_data::{IntersectionType, IntrinsicType, LiteralType, TypeData, UnionType};
pub use type_flags::TypeFlags;
pub use type_id::TypeId;

#[cfg(test)]
mod tests;
