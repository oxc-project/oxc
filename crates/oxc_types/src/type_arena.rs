use append_only_vec::AppendOnlyVec;
use oxc_syntax::symbol::SymbolId;

use crate::{ObjectFlags, TypeData, TypeFlags, TypeId};

/// Arena-based storage for all types created during type checking.
///
/// Types are identified by `TypeId` and their data is stored in parallel
/// columns. This follows oxc's struct-of-arrays (SoA) pattern.
///
/// # Stable references and `&self` allocation
///
/// The arena uses [`AppendOnlyVec`] as its backing store instead of `Vec`
/// or `IndexVec`. This is the critical design choice for the type checker:
///
/// **The problem**: Type checking uniquely requires *interleaved reading and
/// creating* of types. For example, checking `Array<string>.push(42)` must:
/// 1. Read `Array<T>`'s declared type (existing type)
/// 2. Instantiate with `T=string` (creates new type)
/// 3. Read the instantiated `push` method signature (reads the new type)
/// 4. Check argument types (may create more types)
///
/// With a standard `Vec`/`IndexVec`, `push()` takes `&mut self` because
/// it may reallocate, invalidating all existing `&T` references. This
/// forces `TypeArena::new_type` to take `&mut self`, which propagates to
/// `Checker.type_arena: &'a mut TypeArena`, causing every
/// `self.type_arena.get_data()` call to conflict with subsequent `&mut self`
/// method calls. The workaround was `.clone()` everywhere (~17 sites).
///
/// No other oxc crate faces this — they separate build and read phases
/// (e.g., `SemanticBuilder` builds, then `Semantic` is read-only). The
/// type checker is unique in needing both simultaneously.
///
/// **The solution**: `AppendOnlyVec` uses chunked storage that never
/// reallocates existing data. New elements go into new chunks; old chunks
/// are untouched. Both `push(&self)` and index access (`&self[i]`) take
/// shared references, and returned `&T` references are stable for the
/// arena's lifetime. This follows the same design principle as oxc's
/// `Bump` allocator (`oxc_allocator`), which also provides `&self`
/// allocation via multi-chunk storage with `Cell`-based interior mutability.
///
/// With this, `new_type(&self)` and `get_data(&self) -> &TypeData` coexist
/// without borrow conflicts. The `Checker` stores `&'a TypeArena` (shared
/// reference), and all clone workarounds are eliminated.
///
/// # Future: Cell-based alternative
///
/// `AppendOnlyVec` uses atomics internally (for `Send + Sync` support).
/// On x86, atomic loads compile to plain `mov` instructions (zero overhead).
/// On ARM, `Acquire` loads use `ldar` (slightly higher latency).
///
/// If atomics ever show up in profiles, a `Cell`-based implementation
/// (like `Bump`) would eliminate all synchronization overhead at the cost
/// of being `!Sync`. This would be appropriate if the multi-file
/// architecture uses per-thread arenas (no concurrent access needed).
/// The `TypeArena` API would be unchanged — only the backing store
/// implementation would differ.
///
/// # Multi-file parallelism
///
/// The `&self` API is compatible with both multi-file strategies:
/// - **Shared arena**: Multiple threads push to one `AppendOnlyVec`
///   (lock-free via atomics). Types are globally deduplicated.
/// - **Per-thread arenas**: Each thread has its own `TypeArena`.
///   The atomics are unused but harmless on x86.
///
/// See `multi_file_strategies.md` for the full design space analysis.
pub struct TypeArena {
    /// Flags classifying each type's kind.
    flags: AppendOnlyVec<TypeFlags>,
    /// Additional object-specific flags.
    object_flags: AppendOnlyVec<ObjectFlags>,
    /// Type-specific data (variant payload).
    data: AppendOnlyVec<TypeData>,
    /// Optional associated symbol (the declaration that produced this type).
    /// The `u16` is the file index — identifies which file's Semantic the
    /// SymbolId indexes into.
    symbols: AppendOnlyVec<Option<(u16, SymbolId)>>,
}

impl std::fmt::Debug for TypeArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeArena")
            .field("len", &self.flags.len())
            .finish()
    }
}

impl TypeArena {
    /// Create a new empty type arena.
    pub fn new() -> Self {
        Self {
            flags: AppendOnlyVec::new(),
            object_flags: AppendOnlyVec::new(),
            data: AppendOnlyVec::new(),
            symbols: AppendOnlyVec::new(),
        }
    }

    /// Create a new type arena.
    ///
    /// Note: `AppendOnlyVec` does not support pre-allocation, so the
    /// capacity hint is accepted for API compatibility but not used.
    /// The arena grows efficiently via exponential chunking.
    pub fn with_capacity(_capacity: usize) -> Self {
        Self::new()
    }

    /// Number of types in the arena.
    pub fn len(&self) -> usize {
        self.flags.len()
    }

    /// Whether the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.flags.len() == 0
    }

    /// Create a new type and return its `TypeId`.
    ///
    /// Takes `&self` (not `&mut self`) — this is the key property that
    /// enables the checker to read existing types and create new ones
    /// without borrow conflicts. See the struct-level documentation.
    pub fn new_type(
        &self,
        flags: TypeFlags,
        object_flags: ObjectFlags,
        data: TypeData,
        symbol: Option<(u16, SymbolId)>,
    ) -> TypeId {
        let idx = self.flags.push(flags);
        self.object_flags.push(object_flags);
        self.data.push(data);
        self.symbols.push(symbol);
        TypeId::from_usize(idx)
    }

    /// Get the `TypeFlags` for a type.
    #[inline]
    pub fn get_flags(&self, id: TypeId) -> TypeFlags {
        self.flags[id.index()]
    }

    /// Get the `ObjectFlags` for a type.
    #[inline]
    pub fn get_object_flags(&self, id: TypeId) -> ObjectFlags {
        self.object_flags[id.index()]
    }

    /// Get the `TypeData` for a type.
    ///
    /// The returned reference is stable — it will not be invalidated by
    /// subsequent `new_type` calls. This is the property that eliminates
    /// clone workarounds in the checker.
    #[inline]
    pub fn get_data(&self, id: TypeId) -> &TypeData {
        &self.data[id.index()]
    }

    /// Get the associated symbol for a type, if any.
    /// Returns `(file_idx, symbol_id)` — the file index identifies which
    /// file's Semantic the SymbolId indexes into.
    #[inline]
    pub fn get_symbol(&self, id: TypeId) -> Option<(u16, SymbolId)> {
        self.symbols[id.index()]
    }

    /// Create a copy of an existing type with a different symbol association.
    /// Used to attach a type alias name to a type after creation.
    #[inline]
    pub fn clone_type_with_symbol(
        &self,
        id: TypeId,
        symbol: Option<(u16, SymbolId)>,
    ) -> TypeId {
        let flags = self.flags[id.index()];
        let object_flags = self.object_flags[id.index()];
        let data = self.data[id.index()].clone();
        self.new_type(flags, object_flags, data, symbol)
    }
}

impl Default for TypeArena {
    fn default() -> Self {
        Self::new()
    }
}
