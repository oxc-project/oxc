//! The owned, `Send + Sync` type representation shared across checking threads.
//!
//! The oxc AST cannot serve as the cross-thread environment: nodes contain
//! `Cell` fields and are `!Sync`. Surfaces are lowered into these owned types
//! once, frozen at link time, and then only read.

use oxc_span::Span;

/// Index of a source file in the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub u32);

impl FileId {
    /// The file index as `usize`.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Index of a symbol in [`crate::env::ProgramEnv::symbols`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

impl SymbolId {
    /// The symbol index as `usize`.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Index of a type, either into the frozen global [`TypeTable`] or into a
/// per-file scratch table layered on top of it (ids `>= TypeTable::len`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

impl TypeId {
    /// The type index as `usize`.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// One declared enum member.
#[derive(Debug, Clone)]
pub struct EnumMemberInfo {
    /// Member name.
    pub name: Box<str>,
    /// Whether the member's value is a string (string enum member).
    pub is_string: bool,
}

/// A declared type parameter of a generic alias/interface.
#[derive(Debug, Clone)]
pub struct TypeParamInfo {
    /// Parameter name (`T`).
    pub name: Box<str>,
    /// `extends` constraint, if any.
    pub constraint: Option<TypeId>,
    /// Default type, if any.
    pub default: Option<TypeId>,
}

/// Target of a [`Type::Ref`].
///
/// `Pending*` variants exist only between surface lowering and linking; the
/// linker rewrites them to [`RefTarget::Symbol`] or [`RefTarget::Unresolved`].
#[derive(Debug, Clone)]
pub enum RefTarget {
    /// Resolved to a program symbol.
    Symbol(SymbolId),
    /// A checker-local generic declaration (pass B only; index into the
    /// checker's local-generics registry).
    Local(u32),
    /// Resolved to a declaration in the same file's surface (by declaration
    /// index); rewritten to [`RefTarget::Symbol`] at link time.
    PendingLocal(u32),
    /// Resolved to an imported name; rewritten at link time by chasing the
    /// import edge to the target file's exports.
    PendingImport {
        /// Index into the surface's import list.
        import: u32,
        /// Exported name in the target module. `None` means the default export.
        member: Option<Box<str>>,
    },
    /// Could not be resolved (unknown global, unmodeled construct). Relations
    /// involving it return `Unknown` and stay silent.
    Unresolved,
}

/// A member of an object/interface shape.
#[derive(Debug, Clone)]
pub struct Member {
    /// Property name.
    pub name: Box<str>,
    /// Declared or inferred property type.
    pub ty: TypeId,
    /// `?` modifier.
    pub optional: bool,
}

/// An object/interface structure.
#[derive(Debug, Clone)]
pub struct ObjectShape {
    /// Named members. Call/construct/index signatures are not modeled in v0.
    pub members: Box<[Member]>,
    /// `true` when the shape is known to be incomplete (interface `extends`,
    /// object spread, index signatures, ...). A member missing from an inexact
    /// shape yields `Unknown` rather than `False`.
    pub inexact: bool,
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct Param {
    /// Parameter name (for diagnostics only).
    pub name: Box<str>,
    /// Declared parameter type ([`TypeTable::ANY`] when unannotated).
    pub ty: TypeId,
    /// Optional (`?` or has a default).
    pub optional: bool,
}

/// A function/method structure. v0 stores it but relates functions as `Unknown`.
#[derive(Debug, Clone)]
pub struct FunctionShape {
    /// Parameters in declaration order.
    pub params: Box<[Param]>,
    /// Return type ([`TypeTable::ANY`] when unannotated).
    pub ret: TypeId,
}

/// The type representation.
///
/// Anything v0 does not model lowers to [`Type::Unsupported`], which relates
/// as `Unknown` (silent) rather than producing wrong errors.
#[derive(Debug, Clone)]
pub enum Type {
    /// `any`
    Any,
    /// `unknown`
    Unknown,
    /// `never`
    Never,
    /// `void`
    Void,
    /// `undefined`
    Undefined,
    /// `null`
    Null,
    /// `string`
    String,
    /// `number`
    Number,
    /// `boolean`
    Boolean,
    /// `bigint`
    BigInt,
    /// `symbol`
    Symbol,
    /// `object`
    ObjectKeyword,
    /// A string literal type, e.g. `"on"`.
    StringLiteral(Box<str>),
    /// A number literal type, e.g. `42`.
    NumberLiteral(f64),
    /// A boolean literal type: `true` or `false`.
    BooleanLiteral(bool),
    /// A bigint literal type, stored as source text without the `n` suffix.
    BigIntLiteral(Box<str>),
    /// A union type.
    Union(Box<[TypeId]>),
    /// An intersection type. v0 relates it as `Unknown`.
    Intersection(Box<[TypeId]>),
    /// `T[]` / `Array<T>`.
    Array(TypeId),
    /// `[A, B, ...]`. Optional/rest elements lower to [`Type::Unsupported`].
    Tuple(Box<[TypeId]>),
    /// An object type literal or interface body.
    Object(ObjectShape),
    /// A function type.
    Function(Box<FunctionShape>),
    /// A reference to a named type (alias/interface/class/enum/...).
    Ref {
        /// What the name resolved to.
        target: RefTarget,
        /// Type arguments. Non-empty arguments make relations return `Unknown`
        /// in v0 (no instantiation yet).
        args: Box<[TypeId]>,
    },
    /// A reference to a type parameter of the enclosing generic declaration,
    /// replaced by `substitute` at instantiation time.
    /// ("TypeParam" deliberately mirrors tsc's TypeParameter naming.)
    #[expect(clippy::enum_variant_names)]
    TypeParam {
        /// Parameter position.
        index: u16,
        /// Parameter name (for printing uninstantiated forms).
        name: Box<str>,
    },
    /// A construct v0 does not model (conditional/mapped/indexed/typeof/...,
    /// namespaces). Relates as `Unknown`.
    Unsupported,
    /// A *fresh* literal or object-literal type — produced directly by an
    /// expression (tsc's freshness). Stripped to the inner type for relations
    /// and printing; gates contextual widening and excess-property checks.
    Fresh(TypeId),
    /// A name wrapper for local (non-exported) type aliases so diagnostics
    /// print the alias name. Transparent to relations.
    Named {
        /// The alias name.
        name: Box<str>,
        /// The aliased type.
        ty: TypeId,
    },
    /// `readonly T[]` / `ReadonlyArray<T>` / `readonly [..]`. Assigning a
    /// readonly array/tuple to a mutable one is TS4104.
    Readonly(TypeId),
    /// The type of one enum member (`Status.Active`). Nominal.
    EnumMember {
        /// The declaring enum symbol.
        symbol: SymbolId,
        /// Member index within the enum.
        index: u32,
    },
    /// The value side of an enum declaration (`typeof Status`); member access
    /// yields [`Type::EnumMember`].
    EnumValue(SymbolId),
    /// The value side of a class declaration (`typeof Circle`); `new` yields
    /// the instance type.
    ClassValue(SymbolId),
    /// An optional tuple element (`[number, string?]`) — only valid inside
    /// [`Type::Tuple`] members.
    OptionalElem(TypeId),
    /// A rest tuple element (`[string, ...number[]]`), wrapping the array
    /// type — only valid as the last [`Type::Tuple`] member.
    RestElem(TypeId),
}

/// Append-only type storage. The global table is built single-threaded at link
/// time and frozen; checking threads extend it logically via a local overlay
/// (see [`crate::check`]).
#[derive(Debug, Default)]
pub struct TypeTable {
    types: Vec<Type>,
}

macro_rules! intrinsics {
    ($(($const_name:ident, $idx:literal, $variant:ident)),* $(,)?) => {
        impl TypeTable {
            $(
                #[doc = concat!("Fixed id of the `", stringify!($variant), "` intrinsic.")]
                pub const $const_name: TypeId = TypeId($idx);
            )*

            /// Create a table pre-seeded with the intrinsic types at their
            /// fixed ids.
            pub fn new() -> Self {
                Self { types: vec![$(Type::$variant),*] }
            }
        }
    };
}

intrinsics!(
    (ANY, 0, Any),
    (UNKNOWN, 1, Unknown),
    (NEVER, 2, Never),
    (VOID, 3, Void),
    (UNDEFINED, 4, Undefined),
    (NULL, 5, Null),
    (STRING, 6, String),
    (NUMBER, 7, Number),
    (BOOLEAN, 8, Boolean),
    (BIGINT, 9, BigInt),
    (SYMBOL, 10, Symbol),
    (OBJECT_KEYWORD, 11, ObjectKeyword),
    (UNSUPPORTED, 12, Unsupported),
);

/// Number of intrinsic types pre-seeded in every [`TypeTable`]. Type ids below
/// this value always denote intrinsics, in any table or sink.
pub const INTRINSIC_COUNT: u32 = 13;

impl TypeTable {
    /// Number of types in the table.
    pub fn len(&self) -> u32 {
        u32::try_from(self.types.len()).expect("type table exceeds u32")
    }

    /// Whether the table is empty (it never is once constructed via `new`).
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Append a type, returning its id.
    pub fn push(&mut self, ty: Type) -> TypeId {
        let id = TypeId(self.len());
        self.types.push(ty);
        id
    }

    /// Look up a type by id. Panics on local-overlay ids.
    pub fn get(&self, id: TypeId) -> &Type {
        &self.types[id.index()]
    }

    /// Mutable lookup, used by the linker to rewrite pending refs.
    pub fn get_mut(&mut self, id: TypeId) -> &mut Type {
        &mut self.types[id.index()]
    }
}

/// What a symbol means. A single symbol can be referenced from type position,
/// value position, or both depending on its kind.
#[derive(Debug)]
pub enum SymbolKind {
    /// `const` / `let` / `var` / function declaration — a value with a type.
    Value {
        /// The declared (or isolated-declarations-inferred) type.
        ty: TypeId,
    },
    /// `type X<...> = ...`
    TypeAlias {
        /// The aliased type (containing [`Type::TypeParam`] markers when
        /// generic).
        ty: TypeId,
        /// Declared type parameters.
        params: Box<[TypeParamInfo]>,
    },
    /// `interface X<...> { ... }`
    Interface {
        /// The body shape (a [`Type::Object`]).
        ty: TypeId,
        /// Declared type parameters.
        params: Box<[TypeParamInfo]>,
    },
    /// `class X { ... }` — structurally typed via its instance shape, as in
    /// tsc (public members only; private members make the shape inexact).
    Class {
        /// The instance type (a [`Type::Object`] shape).
        instance: TypeId,
    },
    /// `enum X { ... }` — nominal; members carry their value kind for the
    /// enum-to-primitive assignability rules.
    Enum {
        /// Declared members in order.
        members: Box<[EnumMemberInfo]>,
    },
    /// `namespace X { ... }` / `export * as ns` — opaque in v0.
    Namespace,
}

impl SymbolKind {
    /// Whether the symbol can be used in type position.
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            SymbolKind::TypeAlias { .. }
                | SymbolKind::Interface { .. }
                | SymbolKind::Class { .. }
                | SymbolKind::Enum { .. }
        )
    }

    /// Whether the symbol can be used in value position.
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            SymbolKind::Value { .. }
                | SymbolKind::Class { .. }
                | SymbolKind::Enum { .. }
                | SymbolKind::Namespace
        )
    }
}

/// A program symbol: one exported (or export-referenced) declaration.
#[derive(Debug)]
pub struct SymbolData {
    /// Declared name.
    pub name: Box<str>,
    /// Declaring file.
    pub file: FileId,
    /// Span of the declaration name in the declaring file.
    pub span: Span,
    /// What the symbol is.
    pub kind: SymbolKind,
}
