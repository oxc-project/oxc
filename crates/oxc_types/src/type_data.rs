use oxc_span::CompactStr;

use crate::TypeId;

/// Type-specific data for each kind of type.
///
/// This is the polymorphic payload stored alongside `TypeFlags` in the type arena.
/// The variant must be consistent with the `TypeFlags` on the same type.
#[derive(Debug, Clone)]
pub enum TypeData {
    /// Intrinsic types: `any`, `unknown`, `string`, `number`, `boolean`,
    /// `bigint`, `symbol`, `void`, `undefined`, `null`, `never`, `object`.
    Intrinsic(IntrinsicType),

    /// Literal types: `"hello"`, `42`, `true`, `123n`.
    Literal(LiteralType),

    /// Union type: `A | B | C`.
    Union(UnionType),

    /// Intersection type: `A & B & C`.
    Intersection(IntersectionType),
}

/// An intrinsic (built-in primitive) type, identified by name.
///
/// Corresponds to tsgo's `IntrinsicType`.
#[derive(Debug, Clone)]
pub struct IntrinsicType {
    /// The debug/display name: `"string"`, `"number"`, `"any"`, etc.
    pub intrinsic_name: &'static str,
}

/// A literal type with a specific value.
///
/// Corresponds to tsgo's `LiteralType`.
#[derive(Debug, Clone)]
pub enum LiteralType {
    String(CompactStr),
    Number(f64),
    BigInt(CompactStr),
    Boolean(bool),
}

/// A union type (`A | B | C`).
///
/// Constituents are stored as an ordered list of `TypeId`s.
/// Invariants maintained by the checker:
/// - No nested unions (flattened).
/// - No duplicate types.
/// - `never` types are removed.
#[derive(Debug, Clone)]
pub struct UnionType {
    pub types: Vec<TypeId>,
}

/// An intersection type (`A & B & C`).
///
/// Constituents are stored as an ordered list of `TypeId`s.
/// Invariants maintained by the checker:
/// - No nested intersections (flattened).
/// - No duplicate types.
#[derive(Debug, Clone)]
pub struct IntersectionType {
    pub types: Vec<TypeId>,
}
