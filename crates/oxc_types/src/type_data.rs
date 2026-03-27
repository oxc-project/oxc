use std::collections::HashMap;
use std::sync::Arc;

use oxc_span::CompactStr;
use smallvec::SmallVec;

use crate::TypeId;

/// Type-specific data for each kind of type.
///
/// This is the polymorphic payload stored alongside `TypeFlags` in the type arena.
/// The variant must be consistent with the `TypeFlags` on the same type.
///
/// Mirrors tsgo's `TypeData` interface and its implementing structs.
/// Go's struct embedding hierarchy is flattened into a Rust enum.
#[derive(Debug, Clone)]
pub enum TypeData {
    /// Intrinsic types: `any`, `unknown`, `string`, `number`, `boolean`,
    /// `bigint`, `symbol`, `void`, `undefined`, `null`, `never`, `object`.
    Intrinsic(IntrinsicType),

    /// Literal types: `"hello"`, `42`, `true`, `123n`.
    Literal(LiteralType),

    /// Unique ES symbol type: `unique symbol`.
    UniqueESSymbol(UniqueESSymbolType),

    /// Object type (anonymous object literals, computed results, etc.).
    Object(ObjectType),

    /// Type reference: instantiation of a generic type or interface.
    TypeReference(TypeReferenceType),

    /// Interface type (named interface or class).
    Interface(InterfaceType),

    /// Tuple type: `[string, number, ...boolean[]]`.
    Tuple(TupleType),

    /// Mapped type: `{ [K in keyof T]: V }`.
    Mapped(MappedType),

    /// Reverse mapped type (inferred from mapped type context).
    ReverseMapped(ReverseMappedType),

    /// Evolving array type (array literals whose element type widens).
    EvolvingArray(EvolvingArrayType),

    /// Instantiation expression type: `f<number>` as expression.
    InstantiationExpression(InstantiationExpressionType),

    /// Union type: `A | B | C`.
    Union(UnionType),

    /// Intersection type: `A & B & C`.
    Intersection(IntersectionType),

    /// Type parameter: `T` in `<T>`.
    TypeParameter(TypeParameterType),

    /// Index type: `keyof T`.
    Index(IndexType),

    /// Indexed access type: `T[K]`.
    IndexedAccess(IndexedAccessType),

    /// Template literal type: `` `hello ${string}` ``.
    TemplateLiteral(TemplateLiteralType),

    /// String mapping type: `Uppercase<T>`, `Lowercase<T>`, etc.
    StringMapping(StringMappingType),

    /// Substitution type (internal — represents a type parameter with a constraint).
    Substitution(SubstitutionType),

    /// Conditional type: `T extends U ? X : Y`.
    Conditional(ConditionalType),
}

// ---------------------------------------------------------------------------
// Struct definitions for each variant
// ---------------------------------------------------------------------------

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

/// A `unique symbol` type, produced by `const sym: unique symbol = Symbol()`.
///
/// Corresponds to tsgo's `UniqueESSymbolType`.
#[derive(Debug, Clone)]
pub struct UniqueESSymbolType {
    pub name: CompactStr,
}

/// An anonymous object type (object literals, computed results).
///
/// Corresponds to tsgo's `ObjectType`. In Go this is a base struct for
/// TypeReference, InterfaceType, MappedType, etc. via embedding.
/// Here we store the shared "structured" fields inline.
#[derive(Debug, Clone)]
pub struct ObjectType {
    /// Target type for instantiated generics.
    pub target: Option<TypeId>,
    /// Named properties: property name → property type.
    pub properties: HashMap<CompactStr, TypeId>,
    // TODO: call/construct signatures, index infos
}

/// A type reference — an instantiation of a generic type.
///
/// Corresponds to tsgo's `TypeReference` (embeds `ObjectType`).
#[derive(Debug, Clone)]
pub struct TypeReferenceType {
    /// Target type for instantiated generics.
    pub target: Option<TypeId>,
    /// Resolved type arguments for the instantiation.
    pub resolved_type_arguments: SmallVec<[TypeId; 4]>,
    // TODO: node reference
}

/// An interface or class type with declared members.
///
/// Corresponds to tsgo's `InterfaceType` (embeds `TypeReference`).
#[derive(Debug, Clone)]
pub struct InterfaceType {
    /// Target type for instantiated generics.
    pub target: Option<TypeId>,
    /// Resolved type arguments.
    pub resolved_type_arguments: SmallVec<[TypeId; 4]>,
    /// All type parameters (outer + local + thisType).
    pub all_type_parameters: SmallVec<[TypeId; 4]>,
    /// The `this` type, if any.
    pub this_type: Option<TypeId>,
    /// Resolved base types.
    pub resolved_base_types: SmallVec<[TypeId; 4]>,
    /// Declared properties: property name → property type.
    pub properties: HashMap<CompactStr, TypeId>,
    // TODO: call/construct signatures, index infos
}

/// A tuple type: `[string, number, ...boolean[]]`.
///
/// Corresponds to tsgo's `TupleType` (embeds `InterfaceType`).
#[derive(Debug, Clone)]
pub struct TupleType {
    /// Target type for instantiated generics.
    pub target: Option<TypeId>,
    /// Resolved type arguments (element types).
    pub resolved_type_arguments: SmallVec<[TypeId; 4]>,
    /// Per-element metadata.
    pub element_infos: Vec<TupleElementInfo>,
    /// Number of required or variadic elements.
    pub min_length: u32,
    /// Number of initial required or optional elements.
    pub fixed_length: u32,
    /// Combined element flags.
    pub combined_flags: ElementFlags,
    /// Whether the tuple is readonly.
    pub readonly: bool,
}

/// Metadata for a single tuple element.
#[derive(Debug, Clone)]
pub struct TupleElementInfo {
    pub element_type: TypeId,
    pub flags: ElementFlags,
    pub label_name: Option<CompactStr>,
}

bitflags::bitflags! {
    /// Flags for tuple elements.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ElementFlags: u32 {
        const Required  = 1 << 0;
        const Optional  = 1 << 1;
        const Rest      = 1 << 2;
        const Variadic  = 1 << 3;
    }
}

/// A mapped type: `{ [K in keyof T]: V }`.
///
/// Corresponds to tsgo's `MappedType` (embeds `ObjectType`).
#[derive(Debug, Clone)]
pub struct MappedType {
    pub type_parameter: TypeId,
    pub constraint_type: Option<TypeId>,
    pub name_type: Option<TypeId>,
    pub template_type: Option<TypeId>,
    pub modifiers_type: Option<TypeId>,
    pub resolved_apparent_type: Option<TypeId>,
    pub contains_error: bool,
    // TODO: declaration (AST node reference)
}

/// A reverse-mapped type (inferred from mapped type context).
///
/// Corresponds to tsgo's `ReverseMappedType` (embeds `ObjectType`).
#[derive(Debug, Clone)]
pub struct ReverseMappedType {
    pub source: TypeId,
    pub mapped_type: TypeId,
    pub constraint_type: TypeId,
}

/// An evolving array type — array literals whose element type widens as
/// elements are pushed.
///
/// Corresponds to tsgo's `EvolvingArrayType` (embeds `ObjectType`).
#[derive(Debug, Clone)]
pub struct EvolvingArrayType {
    pub element_type: TypeId,
    pub final_array_type: Option<TypeId>,
}

/// An instantiation expression type: `f<number>` used as a value.
///
/// Corresponds to tsgo's `InstantiationExpressionType` (embeds `ObjectType`).
#[derive(Debug, Clone)]
pub struct InstantiationExpressionType {
    // TODO: node (AST reference)
    pub _placeholder: (),
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
    pub types: Arc<SmallVec<[TypeId; 4]>>,
}

/// An intersection type (`A & B & C`).
///
/// Constituents are stored as an ordered list of `TypeId`s.
/// Invariants maintained by the checker:
/// - No nested intersections (flattened).
/// - No duplicate types.
#[derive(Debug, Clone)]
pub struct IntersectionType {
    pub types: SmallVec<[TypeId; 4]>,
}

/// A type parameter: `T` in `function foo<T>(x: T)`.
///
/// Corresponds to tsgo's `TypeParameter`.
#[derive(Debug, Clone)]
pub struct TypeParameterType {
    /// The constraint: `T extends Constraint`.
    pub constraint: Option<TypeId>,
    /// Target type parameter (for instantiated type parameters).
    pub target: Option<TypeId>,
    /// Whether this is the implicit `this` type parameter.
    pub is_this_type: bool,
    /// Resolved default type: `T = Default`.
    pub resolved_default_type: Option<TypeId>,
    // TODO: mapper
}

/// An index type: `keyof T`.
///
/// Corresponds to tsgo's `IndexType`.
#[derive(Debug, Clone)]
pub struct IndexType {
    /// The type being indexed: `T` in `keyof T`.
    pub target: TypeId,
    // TODO: index_flags
}

/// An indexed access type: `T[K]`.
///
/// Corresponds to tsgo's `IndexedAccessType`.
#[derive(Debug, Clone)]
pub struct IndexedAccessType {
    /// The object type: `T` in `T[K]`.
    pub object_type: TypeId,
    /// The index type: `K` in `T[K]`.
    pub index_type: TypeId,
    // TODO: access_flags
}

/// A template literal type: `` `hello ${string}` ``.
///
/// Corresponds to tsgo's `TemplateLiteralType`.
#[derive(Debug, Clone)]
pub struct TemplateLiteralType {
    /// Fixed text segments. Always one more element than `types`.
    pub texts: Vec<CompactStr>,
    /// Interpolated type segments.
    pub types: SmallVec<[TypeId; 4]>,
}

/// A string mapping type: `Uppercase<T>`, `Lowercase<T>`, etc.
///
/// Corresponds to tsgo's `StringMappingType`.
#[derive(Debug, Clone)]
pub struct StringMappingType {
    /// The type being mapped.
    pub target: TypeId,
}

/// A substitution type — represents a type variable with a known constraint.
/// Internal to the checker; never directly written by users.
///
/// Corresponds to tsgo's `SubstitutionType`.
#[derive(Debug, Clone)]
pub struct SubstitutionType {
    /// The base (target) type.
    pub base_type: TypeId,
    /// The constraint the type is known to satisfy.
    pub constraint: TypeId,
}

/// A conditional type: `T extends U ? X : Y`.
///
/// Corresponds to tsgo's `ConditionalType`.
#[derive(Debug, Clone)]
pub struct ConditionalType {
    pub check_type: TypeId,
    pub extends_type: TypeId,
    pub resolved_true_type: Option<TypeId>,
    pub resolved_false_type: Option<TypeId>,
    // TODO: root, mapper, combined_mapper, resolved_inferred_true_type, etc.
}
