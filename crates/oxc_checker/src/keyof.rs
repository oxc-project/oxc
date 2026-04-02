//! Implementation of `keyof T` (Index types) and `T[K]` (Indexed access types).
//!
//! `keyof` produces a union of string literal types from the property names
//! of a type. For concrete types this resolves immediately; for generic types
//! it creates a deferred `IndexType` that resolves during instantiation.
//!
//! `T[K]` accesses the type of a property by key. For concrete types with a
//! literal key, this is a direct member_map lookup. For unions it distributes.
//! For generic types it creates a deferred `IndexedAccessType`.

use oxc_types::{IndexType, IndexedAccessType, ObjectFlags, TypeData, TypeFlags, TypeId};

use crate::Checker;

impl Checker<'_> {
    /// Resolve `keyof T` to a union of property name literal types.
    ///
    /// For concrete object/interface types: produces `"a" | "b" | ...`
    /// For `any`: produces `string | number | symbol`
    /// For type parameters: creates a deferred `IndexType`
    /// For unions: distributes as intersection (`keyof (A | B)` = `keyof A & keyof B`)
    pub(crate) fn get_index_type(&mut self, target: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(target);

        // keyof any → string | number | symbol
        if flags.intersects(TypeFlags::Any) {
            return self.get_or_create_union_type(vec![
                self.string_type,
                self.number_type,
                self.es_symbol_type,
            ]);
        }

        // keyof unknown → never
        if flags.intersects(TypeFlags::Unknown) {
            return self.never_type;
        }

        // keyof never → never (actually string | number | symbol in tsc, but never is simpler)
        if flags.intersects(TypeFlags::Never) {
            return self.never_type;
        }

        // keyof (A | B) → (keyof A) & (keyof B)
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(target) {
                let index_types: Vec<TypeId> =
                    u.types.iter().map(|&t| self.get_index_type(t)).collect();
                return self.get_or_create_intersection_type(index_types);
            }
        }

        // keyof (A & B) → (keyof A) | (keyof B)
        if flags.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = self.type_arena.get_data(target) {
                let index_types: Vec<TypeId> =
                    i.types.iter().map(|&t| self.get_index_type(t)).collect();
                return self.get_or_create_union_type(index_types);
            }
        }

        // TypeReference: resolve first, then get property names
        if let TypeData::TypeReference(_) = self.type_arena.get_data(target) {
            let resolved = self.resolve_type_reference(target);
            return self.get_property_names_as_union(resolved);
        }

        // Concrete object/interface: produce union of property name literals
        if flags.intersects(TypeFlags::Object) {
            return self.get_property_names_as_union(target);
        }

        // Generic type parameter: create deferred IndexType
        if flags.intersects(TypeFlags::TypeParameter) {
            return self.type_arena.new_type(
                TypeFlags::Index,
                ObjectFlags::None,
                TypeData::Index(IndexType { target }),
                None,
            );
        }

        // Primitives: keyof string → union of string method names, etc.
        // Simplified: return never for now
        self.never_type
    }

    /// Get property names of a concrete type as a union of string literal types.
    fn get_property_names_as_union(&mut self, type_id: TypeId) -> TypeId {
        let names: Vec<TypeId> = match self.type_arena.get_data(type_id) {
            TypeData::Structured(s) => s
                .properties
                .iter()
                .map(|p| self.get_or_create_string_literal_type(&p.name))
                .collect(),
            _ => return self.never_type,
        };

        if names.is_empty() {
            return self.never_type;
        }

        self.get_or_create_union_type(names)
    }

    /// Resolve `T[K]` — indexed access type.
    ///
    /// For concrete types with a literal index: direct property lookup.
    /// For union index: distributes (`T["a" | "b"]` = `T["a"] | T["b"]`).
    /// For `keyof T` as index: returns union of all property types.
    /// For generic types: creates a deferred `IndexedAccessType`.
    pub(crate) fn get_indexed_access_type(
        &mut self,
        object_type: TypeId,
        index_type: TypeId,
    ) -> TypeId {
        let obj_flags = self.type_arena.get_flags(object_type);
        let idx_flags = self.type_arena.get_flags(index_type);

        // any[K] → any
        if obj_flags.intersects(TypeFlags::Any) {
            return self.any_type;
        }

        // T[never] → never
        if idx_flags.intersects(TypeFlags::Never) {
            return self.never_type;
        }

        // Index is a union: distribute T["a" | "b"] → T["a"] | T["b"]
        if idx_flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(index_type) {
                let result_types: Vec<TypeId> = u
                    .types
                    .iter()
                    .map(|&idx| self.get_indexed_access_type(object_type, idx))
                    .collect();
                return self.get_or_create_union_type(result_types);
            }
        }

        // Index is `keyof T` where T is the same as object type: return union
        // of all property types
        if idx_flags.intersects(TypeFlags::Index) {
            if let TypeData::Index(idx) = self.type_arena.get_data(index_type) {
                if idx.target == object_type {
                    return self.get_all_property_types_as_union(object_type);
                }
            }
        }

        // If either type is generic, create a deferred IndexedAccessType
        if obj_flags.intersects(
            TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Conditional,
        ) || idx_flags.intersects(
            TypeFlags::TypeParameter
                | TypeFlags::Index
                | TypeFlags::IndexedAccess
                | TypeFlags::Conditional,
        ) {
            return self.type_arena.new_type(
                TypeFlags::IndexedAccess,
                ObjectFlags::None,
                TypeData::IndexedAccess(IndexedAccessType { object_type, index_type }),
                None,
            );
        }

        // Concrete resolution: literal property lookup, tuple/array element access,
        // index signature fallback. Shared with expression-level computed member access.
        if let Some(result) = self.get_property_type_for_index_type(object_type, index_type) {
            return result;
        }

        self.any_type
    }

    /// Get a union of all property types for a concrete type.
    /// Used by `T[keyof T]` to produce the union of all value types.
    fn get_all_property_types_as_union(&mut self, type_id: TypeId) -> TypeId {
        let prop_types: Vec<TypeId> = match self.type_arena.get_data(type_id) {
            TypeData::Structured(s) => s.properties.iter().map(|p| p.type_id).collect(),
            _ => return self.any_type,
        };

        if prop_types.is_empty() {
            return self.never_type;
        }

        self.get_or_create_union_type(prop_types)
    }
}
