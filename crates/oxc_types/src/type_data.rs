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

    /// Structured type: object literal, interface, or class with properties.
    /// Unifies the old Object and Interface variants. The `kind` field on
    /// `StructuredType` distinguishes anonymous objects from named interfaces.
    ///
    /// Boxed to keep the `TypeData` enum compact (~56 bytes instead of 208).
    /// `StructuredType` is the largest variant (208 bytes) due to its Vec/HashMap
    /// fields, but represents only ~15% of types in a typical codebase. Boxing
    /// avoids penalizing the 85% of types that are small (intrinsics, literals,
    /// unions, type parameters, etc.).
    Structured(Box<StructuredType>),

    /// Type reference: instantiation of a generic type or interface.
    TypeReference(TypeReferenceType),

    /// Tuple type: `[string, number, ...boolean[]]`.
    ///
    /// Boxed because `TupleType` (72 bytes) would otherwise dominate the enum
    /// size. Tuples already heap-allocate their element_infos Vec, so the extra
    /// Box indirection is negligible.
    Tuple(Box<TupleType>),

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

    /// Function type (arrow function, function expression, function declaration).
    /// Most functions have exactly 1 call signature.
    ///
    /// Boxed because `FunctionType` (72 bytes) would otherwise be the largest
    /// unboxed variant. Functions with 1 signature already heap-allocate their
    /// parameters Vec, so the extra Box is negligible.
    Function(Box<FunctionType>),
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

/// A named property on a structured type (object, interface, class).
///
/// Corresponds to tsgo's `Symbol` with `SymbolFlagsProperty`. Stores the
/// property name, type, and modifier flags (optional, readonly) needed for
/// mapped type modifier inheritance (`-?`, `-readonly`).
///
/// Properties in a `StructuredType` are sorted by name for O(log N) binary
/// search lookup. The `decl_order` field preserves the original declaration
/// order for display purposes (error messages, type-to-string conversion).
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: CompactStr,
    pub type_id: TypeId,
    pub optional: bool,
    pub readonly: bool,
    /// Original declaration position (0-based). Used to reconstruct source
    /// order when displaying the type. Fits in existing struct padding, so
    /// `PropertyInfo` remains 32 bytes.
    pub decl_order: u16,
}

impl PropertyInfo {
    /// Create a required, non-readonly property.
    pub fn new(name: CompactStr, type_id: TypeId) -> Self {
        Self { name, type_id, optional: false, readonly: false, decl_order: 0 }
    }
}

/// A structured type: object literal, interface, or class with properties.
///
/// Unifies tsgo's `ObjectType` and `InterfaceType`. Shared fields (properties,
/// signatures, index types) live directly on this struct. The `kind` field
/// carries variant-specific data (interface type parameters, base types, etc.).
/// This eliminates duplicate match arms throughout the checker — code that
/// just needs properties works on `&StructuredType` without caring whether
/// it's an object or interface.
///
/// Properties are sorted by name for O(log N) binary search lookup, replacing
/// the previous `FxHashMap<CompactStr, TypeId>` member map. This eliminates
/// per-property name duplication and reduces `StructuredType` size by 48 bytes.
/// Original declaration order is preserved via `PropertyInfo::decl_order`.
///
/// Follows the same pattern as `FlowEntry { kind: FlowNodeKind }`.
#[derive(Debug, Clone)]
pub struct StructuredType {
    /// Named properties, **sorted by name** for binary search lookup.
    /// Use [`StructuredType::find_property`] for O(log N) name lookup.
    ///
    /// **Warning**: Iterating this field directly yields alphabetical order,
    /// NOT declaration order. When building a new type from these properties
    /// or displaying them, use [`StructuredType::properties_in_decl_order`]
    /// to get the original source ordering.
    pub properties: Vec<PropertyInfo>,
    /// String index signature value type: `{ [key: string]: T }` → T.
    pub string_index_type: Option<TypeId>,
    /// Number index signature value type: `{ [idx: number]: T }` → T.
    pub number_index_type: Option<TypeId>,
    /// Call signatures.
    pub call_signatures: Vec<Signature>,
    /// Construct signatures.
    pub construct_signatures: Vec<Signature>,
    /// Variant-specific data.
    pub kind: StructuredTypeKind,
}

/// Variant-specific data for structured types.
#[derive(Debug, Clone)]
pub enum StructuredTypeKind {
    /// Anonymous object (object literal, resolved mapped type, instantiated properties).
    Anonymous {
        target: Option<TypeId>,
    },
    /// Named interface or class with generics and inheritance.
    Interface {
        target: Option<TypeId>,
        resolved_type_arguments: SmallVec<[TypeId; 4]>,
        all_type_parameters: SmallVec<[TypeId; 4]>,
        this_type: Option<TypeId>,
        resolved_base_types: SmallVec<[TypeId; 4]>,
    },
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
    /// The iteration type parameter: `P` in `[P in keyof T]`.
    pub type_parameter: TypeId,
    /// The constraint on the iteration: `keyof T` in `[P in keyof T]`.
    pub constraint_type: Option<TypeId>,
    /// Key remapping: the `X` in `[P in keyof T as X]`.
    pub name_type: Option<TypeId>,
    /// The template value type: `T[P]` in `{ [P in keyof T]: T[P] }`.
    pub template_type: Option<TypeId>,
    /// Optional modifier: None (preserve), Some(Add), Some(Remove).
    /// `?` or `+?` = Add, `-?` = Remove.
    pub optional_modifier: MappedTypeModifier,
    /// Readonly modifier: None (preserve), Some(Add), Some(Remove).
    /// `readonly` or `+readonly` = Add, `-readonly` = Remove.
    pub readonly_modifier: MappedTypeModifier,
}

/// Modifier operation in a mapped type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedTypeModifier {
    /// No modifier — preserve from source type.
    None,
    /// `+?` or `?` / `+readonly` or `readonly` — add the modifier.
    Add,
    /// `-?` / `-readonly` — remove the modifier.
    Remove,
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
    /// The name of the type parameter (e.g., `"T"`).
    pub name: Option<CompactStr>,
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
/// Corresponds to tsgo's `ConditionalType`. Created as a deferred type when
/// the check type is generic. Resolved eagerly when both check and extends
/// are concrete.
///
/// Root fields (`is_distributive`, `infer_type_parameters`) are stored inline
/// rather than behind an indirection ID. This avoids cross-file indexing
/// issues when conditional types from lib.d.ts are instantiated by per-file
/// checkers. The memory overhead (~24 bytes per instantiation) is negligible.
#[derive(Debug, Clone)]
pub struct ConditionalType {
    /// The (possibly instantiated) check type.
    pub check_type: TypeId,
    /// The (possibly instantiated) extends type.
    pub extends_type: TypeId,
    /// The (possibly instantiated) true branch type.
    /// Infer type parameters remain as TypeParameters here until inference
    /// resolves them during eager conditional type resolution.
    pub true_type: TypeId,
    /// The (possibly instantiated) false branch type.
    pub false_type: TypeId,
    /// Whether the conditional distributes over unions.
    /// True when the original check type was a bare type parameter.
    pub is_distributive: bool,
    /// Type parameters declared with `infer` in the extends clause.
    /// E.g., for `T extends Array<infer U> ? U : never`, this contains `[U]`.
    /// Empty when the conditional has no `infer` declarations.
    pub infer_type_parameters: SmallVec<[TypeId; 2]>,
}

// ---------------------------------------------------------------------------
// Function types and signatures
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    /// Flags for function/method signatures.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SignatureFlags: u8 {
        const None             = 0;
        const HasRestParameter = 1 << 0;
        const Construct        = 1 << 1;
        const Abstract         = 1 << 2;
    }
}

/// A single function/method signature.
///
/// Corresponds to tsgo's `Signature`.
#[derive(Debug, Clone)]
pub struct Signature {
    pub flags: SignatureFlags,
    /// Minimum number of required arguments (excludes optional/rest).
    pub min_argument_count: u32,
    /// Parameter names and types, in order.
    pub parameters: Vec<ParameterInfo>,
    /// The return type of the signature.
    pub return_type: TypeId,
    /// Type parameters for generic signatures (e.g., `<T>` in `function id<T>(x: T): T`).
    pub type_parameters: SmallVec<[TypeId; 4]>,
    // Deferred: this_parameter, mapper, type_predicate
}

/// Info about a single parameter in a signature.
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: CompactStr,
    pub type_id: TypeId,
    pub is_optional: bool,
    pub is_rest: bool,
}

/// A function type (arrow function, function expression, function declaration).
/// Most functions have exactly 1 call signature.
///
/// Unlike tsc which models function types as anonymous object types with a
/// single call signature, we use a dedicated variant to avoid bloating the
/// common case with object-related fields.
#[derive(Debug, Clone)]
pub struct FunctionType {
    pub signatures: SmallVec<[Signature; 1]>,
}

/// Sort a property list by name for binary search lookup.
///
/// Assigns `decl_order` to each property based on its current position
/// (preserving the original declaration order), then sorts by name.
/// Called at type creation time before storing in `StructuredType`.
///
/// Replaces the previous `build_member_map` which created a separate
/// `FxHashMap<CompactStr, TypeId>` — this eliminates per-property name
/// duplication and the 48-byte HashMap overhead per type.
pub fn sort_properties(properties: &mut [PropertyInfo]) {
    for (i, p) in properties.iter_mut().enumerate() {
        p.decl_order = i as u16;
    }
    properties.sort_unstable_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
}

impl StructuredType {
    /// Look up a property by name using binary search. O(log N).
    pub fn find_property(&self, name: &str) -> Option<&PropertyInfo> {
        self.properties
            .binary_search_by(|p| p.name.as_str().cmp(name))
            .ok()
            .map(|idx| &self.properties[idx])
    }

    /// Check if a property with the given name exists. O(log N).
    pub fn has_property(&self, name: &str) -> bool {
        self.find_property(name).is_some()
    }

    /// Iterate properties in original declaration order (for display).
    ///
    /// Allocates a temporary Vec and sorts it — O(N log N). Use only for
    /// display/diagnostics, not in hot paths like assignability or inference.
    pub fn properties_in_decl_order(&self) -> Vec<&PropertyInfo> {
        let mut ordered: Vec<&PropertyInfo> = self.properties.iter().collect();
        ordered.sort_unstable_by_key(|p| p.decl_order);
        ordered
    }
}
