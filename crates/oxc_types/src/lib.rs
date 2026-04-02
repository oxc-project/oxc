//! Type representation for oxc's TypeScript type checker.
//!
//! This crate provides the core type data structures: `TypeId`, `TypeFlags`,
//! `ObjectFlags`, `TypeData`, and `TypeArena`. It does not contain any
//! type-checking logic itself — that lives in `oxc_checker`.

mod object_flags;
mod type_arena;
mod type_data;
mod type_factory;
mod type_flags;
mod type_id;
mod type_mapper;

pub use object_flags::ObjectFlags;
pub use type_arena::TypeArena;
pub use type_data::*;
pub use type_factory::{
    TypeFactory, instantiate_type_common, instantiate_signature, instantiate_structured_type,
    signature_could_contain_type_variables, type_could_contain_type_variables,
};
pub use type_flags::TypeFlags;
pub use type_id::TypeId;
pub use type_mapper::TypeMapper;

#[cfg(test)]
mod tests;
