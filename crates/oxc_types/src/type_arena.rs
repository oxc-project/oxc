use oxc_index::IndexVec;
use oxc_syntax::symbol::SymbolId;

use crate::{ObjectFlags, TypeData, TypeFlags, TypeId};

/// Arena-based storage for all types created during type checking.
///
/// Types are identified by `TypeId` and their data is stored in parallel vectors.
/// This follows oxc's struct-of-arrays pattern for cache-friendly access.
#[derive(Debug)]
pub struct TypeArena {
    /// Flags classifying each type's kind.
    flags: IndexVec<TypeId, TypeFlags>,
    /// Additional object-specific flags.
    object_flags: IndexVec<TypeId, ObjectFlags>,
    /// Type-specific data (variant payload).
    data: IndexVec<TypeId, TypeData>,
    /// Optional associated symbol (the declaration that produced this type).
    symbols: IndexVec<TypeId, Option<SymbolId>>,
}

impl TypeArena {
    /// Create a new empty type arena.
    pub fn new() -> Self {
        Self {
            flags: IndexVec::new(),
            object_flags: IndexVec::new(),
            data: IndexVec::new(),
            symbols: IndexVec::new(),
        }
    }

    /// Create a new type arena with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            flags: IndexVec::with_capacity(capacity),
            object_flags: IndexVec::with_capacity(capacity),
            data: IndexVec::with_capacity(capacity),
            symbols: IndexVec::with_capacity(capacity),
        }
    }

    /// Number of types in the arena.
    pub fn len(&self) -> usize {
        self.flags.len()
    }

    /// Whether the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    /// Create a new type and return its `TypeId`.
    pub fn new_type(
        &mut self,
        flags: TypeFlags,
        object_flags: ObjectFlags,
        data: TypeData,
        symbol: Option<SymbolId>,
    ) -> TypeId {
        let id = self.flags.push(flags);
        self.object_flags.push(object_flags);
        self.data.push(data);
        self.symbols.push(symbol);
        id
    }

    /// Get the `TypeFlags` for a type.
    #[inline]
    pub fn get_flags(&self, id: TypeId) -> TypeFlags {
        self.flags[id]
    }

    /// Get the `ObjectFlags` for a type.
    #[inline]
    pub fn get_object_flags(&self, id: TypeId) -> ObjectFlags {
        self.object_flags[id]
    }

    /// Get the `TypeData` for a type.
    #[inline]
    pub fn get_data(&self, id: TypeId) -> &TypeData {
        &self.data[id]
    }

    /// Get a mutable reference to the `TypeData` for a type.
    #[inline]
    pub fn get_data_mut(&mut self, id: TypeId) -> &mut TypeData {
        &mut self.data[id]
    }

    /// Get the associated symbol for a type, if any.
    #[inline]
    pub fn get_symbol(&self, id: TypeId) -> Option<SymbolId> {
        self.symbols[id]
    }

    /// Set the `ObjectFlags` for a type.
    #[inline]
    pub fn set_object_flags(&mut self, id: TypeId, flags: ObjectFlags) {
        self.object_flags[id] = flags;
    }
}

impl Default for TypeArena {
    fn default() -> Self {
        Self::new()
    }
}
