use std::sync::Arc;

use oxc_span::CompactStr;
use oxc_types::{
    FunctionType, ObjectFlags, PropertyInfo, Signature, StructuredType, StructuredTypeKind,
    TypeArena, TypeData, TypeFactory, TypeFlags, TypeId, TypeMapper, TypeReferenceType, UnionType,
    instantiate_type_common, sort_properties,
};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

use crate::Checker;

impl Checker<'_> {
    /// Get or create a deduplicated union type from a list of constituent type IDs.
    ///
    /// Handles normalization: filters `never`, deduplicates, sorts.
    /// Returns `never` for empty, unwraps single-element unions.
    pub fn get_or_create_union_type(&mut self, mut types: Vec<TypeId>) -> TypeId {
        // Filter out `never` types
        types.retain(|&t| !self.type_arena.get_flags(t).intersects(TypeFlags::Never));

        if types.is_empty() {
            return self.never_type;
        }
        if types.len() == 1 {
            return types[0];
        }

        types.sort();
        types.dedup();

        if types.len() == 1 {
            return types[0];
        }

        let key: Arc<SmallVec<[TypeId; 4]>> = SmallVec::from_vec(types).into();

        let type_id = self.union_types.entry(key.clone()).or_insert_with_key(|key| {
            // Propagate CouldContainTypeVariables so is_generic_type
            // can check a single flag instead of walking constituents.
            let has_instantiable = key.iter().any(|&t| {
                let f = self.type_arena.get_flags(t);
                f.intersects(TypeFlags::Instantiable)
                    || self
                        .type_arena
                        .get_object_flags(t)
                        .intersects(ObjectFlags::CouldContainTypeVariables)
            });
            let obj_flags = if has_instantiable {
                ObjectFlags::CouldContainTypeVariables
            } else {
                ObjectFlags::None
            };
            self.type_arena.new_type(
                TypeFlags::Union,
                obj_flags,
                TypeData::Union(UnionType { types: key.clone() }),
                None,
            )
        });

        *type_id
    }

    /// Create a deduplicated intersection type from a list of constituent type IDs.
    ///
    /// Handles normalization: deduplicates while preserving constituent order
    /// (unlike unions which are sorted), matching tsgo's approach.
    /// Returns `unknown` for empty, unwraps single-element intersections.
    pub fn get_or_create_intersection_type(&mut self, mut types: Vec<TypeId>) -> TypeId {
        // 1. Flatten nested intersections: (A & B) & C → [A, B, C]
        let mut i = 0;
        while i < types.len() {
            if self.type_arena.get_flags(types[i]).intersects(TypeFlags::Intersection) {
                if let TypeData::Intersection(inter) = self.type_arena.get_data(types[i]) {
                    let children = &inter.types;
                    types.remove(i);
                    for (j, &child) in children.iter().enumerate() {
                        types.insert(i + j, child);
                    }
                    continue; // re-check at same index
                }
            }
            i += 1;
        }

        // 2. Never propagation: A & never → never
        if types.iter().any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::Never)) {
            return self.never_type;
        }

        // 3. Contradictory primitive reduction: string & number → never
        // Collect which disjoint primitive groups are present.
        let mut groups = 0u8;
        for &t in &types {
            let f = self.type_arena.get_flags(t);
            if f.intersects(TypeFlags::StringLike) {
                groups |= 1;
            }
            if f.intersects(TypeFlags::NumberLike) {
                groups |= 2;
            }
            if f.intersects(TypeFlags::BigIntLike) {
                groups |= 4;
            }
            if f.intersects(TypeFlags::BooleanLike) {
                groups |= 8;
            }
            if f.intersects(TypeFlags::ESSymbolLike) {
                groups |= 16;
            }
            if f.intersects(TypeFlags::Void) {
                groups |= 32;
            }
            if f.intersects(TypeFlags::Undefined) {
                groups |= 64;
            }
            if f.intersects(TypeFlags::Null) {
                groups |= 128;
            }
        }
        if groups.count_ones() > 1 {
            return self.never_type;
        }

        // 4. Supertype removal: string & "hello" → "hello"
        {
            let has_string_literal = types
                .iter()
                .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::StringLiteral));
            let has_number_literal = types
                .iter()
                .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::NumberLiteral));
            let has_boolean_literal = types
                .iter()
                .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::BooleanLiteral));
            let has_bigint_literal = types
                .iter()
                .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::BigIntLiteral));
            types.retain(|&t| {
                let f = self.type_arena.get_flags(t);
                !(has_string_literal && f.intersects(TypeFlags::String))
                    && !(has_number_literal && f.intersects(TypeFlags::Number))
                    && !(has_boolean_literal && f.intersects(TypeFlags::Boolean))
                    && !(has_bigint_literal && f.intersects(TypeFlags::BigInt))
            });
        }

        // Order-preserving dedup: retain first occurrence of each type.
        let mut seen = FxHashSet::default();
        types.retain(|t| seen.insert(*t));

        if types.is_empty() {
            return self.unknown_type;
        }
        if types.len() == 1 {
            return types[0];
        }

        let key: SmallVec<[TypeId; 4]> = SmallVec::from_vec(types);

        let type_id = self.intersection_types.entry(key).or_insert_with_key(|key| {
            let has_instantiable = key.iter().any(|&t| {
                let f = self.type_arena.get_flags(t);
                f.intersects(TypeFlags::Instantiable)
                    || self
                        .type_arena
                        .get_object_flags(t)
                        .intersects(ObjectFlags::CouldContainTypeVariables)
            });
            let obj_flags = if has_instantiable {
                ObjectFlags::CouldContainTypeVariables
            } else {
                ObjectFlags::None
            };
            self.type_arena.new_type(
                TypeFlags::Intersection,
                obj_flags,
                TypeData::Intersection(oxc_types::IntersectionType { types: key.clone() }),
                None,
            )
        });
        *type_id
    }

    /// Create a deduplicated TypeReference for a generic instantiation.
    ///
    /// If a TypeReference with the same `(target, type_args)` already exists,
    /// returns the existing TypeId. Otherwise creates a new one.
    /// Follows the same dedup pattern as `get_or_create_union_type`.
    pub fn get_or_create_type_reference(
        &mut self,
        target: TypeId,
        type_args: SmallVec<[TypeId; 4]>,
    ) -> TypeId {
        let key = (target, type_args);
        if let Some(&existing) = self.type_reference_types.get(&key) {
            return existing;
        }

        let has_instantiable = key.1.iter().any(|&t| {
            let f = self.type_arena.get_flags(t);
            f.intersects(TypeFlags::Instantiable)
                || self
                    .type_arena
                    .get_object_flags(t)
                    .intersects(ObjectFlags::CouldContainTypeVariables)
        });
        let obj_flags = ObjectFlags::Reference
            | if has_instantiable {
                ObjectFlags::CouldContainTypeVariables
            } else {
                ObjectFlags::None
            };

        let type_id = self.type_arena.new_type(
            TypeFlags::Object,
            obj_flags,
            TypeData::TypeReference(TypeReferenceType {
                target: Some(target),
                resolved_type_arguments: key.1.clone(),
            }),
            None,
        );
        self.type_reference_types.insert(key, type_id);
        type_id
    }

    /// Get or create a deduplicated string literal type.
    pub fn get_or_create_string_literal_type(&mut self, value: &str) -> TypeId {
        let key = CompactStr::new(value);
        let type_id = self.string_literal_types.entry(key).or_insert_with_key(|key| {
            self.type_arena.new_type(
                TypeFlags::StringLiteral,
                ObjectFlags::None,
                TypeData::Literal(oxc_types::LiteralType::String(key.clone())),
                None,
            )
        });
        *type_id
    }

    /// Get or create a deduplicated number literal type.
    pub fn get_or_create_number_literal_type(&mut self, value: f64) -> TypeId {
        let key = value.to_bits();
        let type_id = self.number_literal_types.entry(key).or_insert_with(|| {
            self.type_arena.new_type(
                TypeFlags::NumberLiteral,
                ObjectFlags::None,
                TypeData::Literal(oxc_types::LiteralType::Number(value)),
                None,
            )
        });
        *type_id
    }

    /// Get or create a deduplicated bigint literal type.
    pub fn get_or_create_bigint_literal_type(&mut self, value: &str) -> TypeId {
        let key = CompactStr::new(value);
        let type_id = self.bigint_literal_types.entry(key).or_insert_with_key(|key| {
            self.type_arena.new_type(
                TypeFlags::BigIntLiteral,
                ObjectFlags::None,
                TypeData::Literal(oxc_types::LiteralType::BigInt(key.clone())),
                None,
            )
        });
        *type_id
    }

    /// Get the fresh version of a literal type.
    ///
    /// Fresh literals are created from source-code literal expressions and will
    /// widen to their base type for mutable bindings. Non-fresh literals (from
    /// type annotations, narrowing, etc.) do not widen.
    ///
    /// Mirrors tsgo's `getFreshTypeOfLiteralType`.
    pub fn get_fresh_type_of_literal(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);
        if !flags.intersects(TypeFlags::Freshable) {
            return type_id;
        }
        // Already fresh — return as-is. Without this, passing a fresh TypeId
        // would miss the map lookup (keyed by regular ids) and create a
        // spurious fresh-of-fresh duplicate in the arena.
        if self.type_arena.get_object_flags(type_id).intersects(ObjectFlags::FreshLiteral) {
            return type_id;
        }
        if let Some(&fresh) = self.fresh_literal_map.get(&type_id) {
            return fresh;
        }
        let data = self.type_arena.get_data(type_id).clone();
        let fresh_id = self.type_arena.new_type(
            flags,
            ObjectFlags::FreshLiteral,
            data,
            self.type_arena.get_symbol(type_id),
        );
        self.fresh_literal_map.insert(type_id, fresh_id);
        self.regular_literal_map.insert(fresh_id, type_id);
        fresh_id
    }

    /// Get the regular (non-fresh) version of a literal type.
    ///
    /// Mirrors tsgo's `getRegularTypeOfLiteralType`.
    pub fn get_regular_type_of_literal(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);
        if flags.intersects(TypeFlags::Freshable) {
            let obj_flags = self.type_arena.get_object_flags(type_id);
            if obj_flags.intersects(ObjectFlags::FreshLiteral) {
                if let Some(&regular) = self.regular_literal_map.get(&type_id) {
                    return regular;
                }
            }
            return type_id;
        }
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let types = u.types.clone();
                let regular: Vec<TypeId> =
                    types.iter().map(|&m| self.get_regular_type_of_literal(m)).collect();
                return self.get_or_create_union_type(regular);
            }
        }
        type_id
    }

    /// Widen a fresh literal type to its base type.
    ///
    /// Only widens types marked with `ObjectFlags::FreshLiteral` (from source-code
    /// literal expressions). Non-fresh literals are returned unchanged.
    ///
    /// Mirrors tsgo's `getWidenedLiteralType`.
    pub fn get_widened_literal_type(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);
        if flags.intersects(TypeFlags::Freshable) {
            let obj_flags = self.type_arena.get_object_flags(type_id);
            if !obj_flags.intersects(ObjectFlags::FreshLiteral) {
                return type_id;
            }
            if flags.intersects(TypeFlags::StringLiteral) {
                return self.string_type;
            } else if flags.intersects(TypeFlags::NumberLiteral) {
                return self.number_type;
            } else if flags.intersects(TypeFlags::BigIntLiteral) {
                return self.bigint_type;
            } else if flags.intersects(TypeFlags::BooleanLiteral) {
                return self.boolean_type;
            }
        }
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let widened: Vec<TypeId> =
                    u.types.iter().map(|&m| self.get_widened_literal_type(m)).collect();
                return self.get_or_create_union_type(widened);
            }
        }
        type_id
    }

    /// Widen the inferred type for a variable-like declaration's initializer.
    ///
    /// Unified decision point for literal widening on variables, parameters,
    /// and class/interface properties. Mirrors tsgo's
    /// `getWidenedLiteralTypeForInitializer` + the widening portion of
    /// `widenTypeForVariableLikeDeclaration`.
    ///
    /// `is_const_like` should be `true` for `const` variables, `readonly`
    /// properties, and enum members — these preserve literal types. All other
    /// mutable declarations widen fresh literals to their base type and apply
    /// null/undefined → any widening.
    pub fn get_widened_type_for_initializer(
        &mut self,
        init_type: TypeId,
        is_const_like: bool,
    ) -> TypeId {
        if is_const_like {
            return init_type;
        }
        let widened = self.get_widened_literal_type(init_type);
        self.get_widened_type(widened)
    }

    // ── Type queries ──────────────────────────────────────────────────

    /// Check if a type is a unit type (single literal, enum, unique symbol, or nullable).
    ///
    /// Mirrors tsgo's `isUnitType`.
    pub fn is_unit_type(&self, type_id: TypeId) -> bool {
        self.type_arena.get_flags(type_id).intersects(TypeFlags::Unit)
    }

    /// Check if a type could contain a constituent matching the given flags.
    /// For union types, checks each member. Otherwise checks the type's own flags.
    ///
    /// Mirrors tsgo's `maybeTypeOfKind`.
    pub fn maybe_type_of_kind(&self, type_id: TypeId, kind: TypeFlags) -> bool {
        let flags = self.type_arena.get_flags(type_id);
        if flags.intersects(kind) {
            return true;
        }
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                return u.types.iter().any(|&t| self.type_arena.get_flags(t).intersects(kind));
            }
        }
        false
    }

    // ── Contextual literal widening ───────────────────────────────────

    /// Check if a literal type should be preserved (not widened) because the
    /// contextual type expects a literal of the same kind.
    ///
    /// For example, if the contextual type is `"hello" | "world"` and the candidate
    /// is `"hello"`, the literal is preserved. If there's no contextual type,
    /// the literal is always widened.
    ///
    /// Mirrors tsgo's `isLiteralOfContextualType`.
    fn is_literal_of_contextual_type(
        &mut self,
        candidate: TypeId,
        contextual: Option<TypeId>,
    ) -> bool {
        let Some(ctx) = contextual else { return false };
        let ctx_flags = self.type_arena.get_flags(ctx);

        // Union/intersection: any constituent preserves → preserve
        if ctx_flags.intersects(TypeFlags::UnionOrIntersection) {
            return match self.type_arena.get_data(ctx) {
                TypeData::Union(u) => {
                    u.types.iter().any(|&t| self.is_literal_of_contextual_type(candidate, Some(t)))
                }
                TypeData::Intersection(i) => {
                    i.types.iter().any(|&t| self.is_literal_of_contextual_type(candidate, Some(t)))
                }
                _ => false,
            };
        }

        // Type parameter constrained to a primitive: check constraint.
        // Mirrors tsgo: if getBaseConstraintOfType returns nil, use unknownType.
        if ctx_flags.intersects(TypeFlags::InstantiableNonPrimitive) {
            let constraint = self
                .get_base_constraint_of_type(ctx)
                .unwrap_or(self.unknown_type);
            return (self.maybe_type_of_kind(constraint, TypeFlags::String)
                && self.maybe_type_of_kind(candidate, TypeFlags::StringLiteral))
                || (self.maybe_type_of_kind(constraint, TypeFlags::Number)
                    && self.maybe_type_of_kind(candidate, TypeFlags::NumberLiteral))
                || (self.maybe_type_of_kind(constraint, TypeFlags::BigInt)
                    && self.maybe_type_of_kind(candidate, TypeFlags::BigIntLiteral))
                || (self.maybe_type_of_kind(constraint, TypeFlags::ESSymbol)
                    && self.maybe_type_of_kind(candidate, TypeFlags::UniqueESSymbol))
                || self.is_literal_of_contextual_type(candidate, Some(constraint));
        }

        // Direct literal matching: contextual type is a literal of the same kind
        let cand_flags = self.type_arena.get_flags(candidate);
        (ctx_flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::Index
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping,
        ) && cand_flags.intersects(TypeFlags::StringLiteral))
            || (ctx_flags.intersects(TypeFlags::NumberLiteral)
                && cand_flags.intersects(TypeFlags::NumberLiteral))
            || (ctx_flags.intersects(TypeFlags::BigIntLiteral)
                && cand_flags.intersects(TypeFlags::BigIntLiteral))
            || (ctx_flags.intersects(TypeFlags::BooleanLiteral)
                && cand_flags.intersects(TypeFlags::BooleanLiteral))
            || (ctx_flags.intersects(TypeFlags::UniqueESSymbol)
                && cand_flags.intersects(TypeFlags::UniqueESSymbol))
    }

    /// Widen a literal type unless the contextual type preserves it.
    ///
    /// Mirrors tsgo's `getWidenedLiteralLikeTypeForContextualType`.
    pub fn get_widened_literal_like_type_for_contextual_type(
        &mut self,
        type_id: TypeId,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        if !self.is_literal_of_contextual_type(type_id, contextual_type) {
            let widened = self.get_widened_literal_type(type_id);
            // TODO: get_widened_unique_es_symbol_type(widened)
            return self.get_regular_type_of_literal(widened);
        }
        self.get_regular_type_of_literal(type_id)
    }

    /// Apply TypeScript's return-type widening pipeline.
    ///
    /// Step 1: Contextual literal widening — only for unit types.
    /// Step 2: Object literal widening (TODO: `get_widened_type`).
    ///
    /// Mirrors the widening steps in tsgo's `getReturnTypeFromBody`.
    pub fn widen_return_type(
        &mut self,
        return_type: TypeId,
        contextual_return_type: Option<TypeId>,
    ) -> TypeId {
        let mut result = return_type;

        // Step 1: Contextual literal widening — only for unit types
        if self.is_unit_type(result) {
            result = self.get_widened_literal_like_type_for_contextual_type(
                result,
                contextual_return_type,
            );
        }

        // Step 2: Object literal / null/undefined widening
        result = self.get_widened_type(result);

        result
    }

    // ── Type widening (null/undefined → any) ─────────────────────────────

    /// Widen a type: null/undefined with widening marks → any,
    /// object literals → recursively widened properties.
    ///
    /// Fast-path: returns immediately if the type has no `RequiresWidening` flags.
    /// Mirrors tsgo's `getWidenedType` / `getWidenedTypeWithContext` (without
    /// the `WideningContext` parameter, which handles advanced union-property
    /// widening and can be added later).
    pub fn get_widened_type(&mut self, type_id: TypeId) -> TypeId {
        let obj_flags = self.type_arena.get_object_flags(type_id);
        if !obj_flags.intersects(ObjectFlags::RequiresWidening) {
            return type_id;
        }
        if let Some(&cached) = self.widened_type_cache.get(&type_id) {
            return cached;
        }
        let result = self.get_widened_type_worker(type_id);
        self.widened_type_cache.insert(type_id, result);
        result
    }

    /// Inner widening dispatch — called only when `RequiresWidening` is set.
    fn get_widened_type_worker(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);
        let obj_flags = self.type_arena.get_object_flags(type_id);

        // null/undefined with widening mark → any
        if flags.intersects(TypeFlags::Nullable)
            && obj_flags.intersects(ObjectFlags::ContainsWideningType)
        {
            return self.any_type;
        }

        // Object literal → recursively widen properties
        if flags.intersects(TypeFlags::Object)
            && obj_flags.intersects(ObjectFlags::ObjectLiteral)
        {
            return self.get_widened_type_of_object_literal(type_id);
        }

        // TODO: Union → map over constituents, filter widening nullable
        // TODO: Intersection → map over constituents
        // TODO: Array/Tuple TypeReference → widen type arguments

        type_id
    }

    /// Widen an object literal type by recursively widening each property.
    ///
    /// Creates a new `StructuredType` with property types replaced by their
    /// widened versions. The result has `ObjectLiteral` but not `FreshLiteral`
    /// or `RequiresWidening` — it is the resolved form.
    fn get_widened_type_of_object_literal(&mut self, type_id: TypeId) -> TypeId {
        // Arena references have lifetime 'a (tied to the arena, not &self),
        // so we can hold them while calling &mut self methods like get_widened_type.
        let TypeData::Structured(s) = self.type_arena.get_data(type_id) else {
            return type_id;
        };

        let mut any_changed = false;
        let mut widened_props: Vec<PropertyInfo> = Vec::with_capacity(s.properties.len());
        for p in &s.properties {
            let widened_type = self.get_widened_type(p.type_id);
            if widened_type != p.type_id {
                any_changed = true;
            }
            widened_props.push(PropertyInfo {
                name: p.name.clone(),
                type_id: widened_type,
                optional: p.optional,
                readonly: p.readonly,
                decl_order: p.decl_order,
            });
        }

        // If no property changed, return the original type to avoid
        // allocating a duplicate in the arena.
        if !any_changed {
            return type_id;
        }

        // Widen index signature value types too (tsgo does this).
        let string_index_type = s.string_index_type.map(|t| self.get_widened_type(t));
        let number_index_type = s.number_index_type.map(|t| self.get_widened_type(t));

        // Properties are already sorted (inherited from the original).
        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous | ObjectFlags::ObjectLiteral,
            TypeData::Structured(Box::new(StructuredType {
                properties: widened_props,
                string_index_type,
                number_index_type,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: None },
            })),
            None,
        )
    }

    // ── Flag propagation ────────────────────────────────────────────────

    /// Collect propagating `ObjectFlags` from a slice of types.
    ///
    /// ORs `get_object_flags(t)` for each type whose `TypeFlags` don't intersect
    /// `exclude_kind`, then masks the result with `ObjectFlags::PropagatingFlags`.
    /// Mirrors tsgo's `getPropagatingFlagsOfTypes`.
    #[allow(dead_code)] // Used when union/intersection widening is added
    pub(crate) fn get_propagating_flags_of_types(
        &self,
        types: &[TypeId],
        exclude_kind: TypeFlags,
    ) -> ObjectFlags {
        let mut result = ObjectFlags::None;
        for &t in types {
            if !exclude_kind.is_empty()
                && self.type_arena.get_flags(t).intersects(exclude_kind)
            {
                continue;
            }
            result |= self.type_arena.get_object_flags(t);
        }
        result & ObjectFlags::PropagatingFlags
    }

    // ── Spread type validation ─────────────────────────────────────────

    /// Check whether a type is valid as the argument of an object spread (`{ ...x }`).
    ///
    /// Valid spread types are: `any`, `object`, object types, instantiable
    /// non-primitive types (type parameters, conditionals, substitutions),
    /// and unions/intersections where every constituent is itself a valid spread type.
    ///
    /// Mirrors TypeScript's `isValidSpreadType`.
    pub(crate) fn is_valid_spread_type(&mut self, type_id: TypeId) -> bool {
        let resolved = self.get_base_constraint_or_type(type_id);
        let filtered = self.remove_definitely_falsy_types(resolved);
        let flags = self.type_arena.get_flags(filtered);

        if flags.intersects(
            TypeFlags::Any
                | TypeFlags::NonPrimitive
                | TypeFlags::Object
                | TypeFlags::InstantiableNonPrimitive,
        ) {
            return true;
        }

        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(filtered) {
                return u.types.iter().all(|&t| self.is_valid_spread_type(t));
            }
        }
        if flags.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = self.type_arena.get_data(filtered) {
                return i.types.iter().all(|&t| self.is_valid_spread_type(t));
            }
        }

        // Never is valid (it's a bottom type — spreading it produces nothing)
        flags.intersects(TypeFlags::Never)
    }

    /// Resolve the base constraint of an instantiable type (e.g. type parameter).
    /// Returns `None` if no constraint exists or the type is not instantiable.
    ///
    /// Mirrors TypeScript's `getBaseConstraintOfType`.
    fn get_base_constraint_of_type(&mut self, type_id: TypeId) -> Option<TypeId> {
        let flags = self.type_arena.get_flags(type_id);

        if flags.intersects(TypeFlags::TypeParameter) {
            return self.get_constraint_of_type_parameter(type_id);
        }
        // TODO: handle other instantiable types (conditional, substitution,
        // indexed access, index, template literal, string mapping)
        // and unions/intersections of such types.
        None
    }

    /// If `type_id` is an instantiable type (e.g. type parameter), resolve its
    /// base constraint; otherwise return the type unchanged.
    ///
    /// Mirrors TypeScript's `getBaseConstraintOrType`.
    fn get_base_constraint_or_type(&mut self, type_id: TypeId) -> TypeId {
        self.get_base_constraint_of_type(type_id).unwrap_or(type_id)
    }

    /// Filter out definitely-falsy constituents (null, undefined, void,
    /// false literal, 0 literal, "" literal) from a type.
    ///
    /// Mirrors TypeScript's `removeDefinitelyFalsyTypes`.
    fn remove_definitely_falsy_types(&mut self, type_id: TypeId) -> TypeId {
        self.narrow_type_by_predicate(type_id, |checker, t| !checker.is_falsy_type(t))
    }

    // ── Spread type merging ───────────────────────────────────────────

    /// Merge two types in a spread operation (left-fold).
    ///
    /// Called repeatedly as `spread = get_spread_type(spread, next_element)`
    /// while iterating through an object literal's properties and spreads.
    ///
    /// Mirrors TypeScript's `getSpreadType`.
    pub(crate) fn get_spread_type(&mut self, left: TypeId, right: TypeId) -> TypeId {
        let left_flags = self.type_arena.get_flags(left);
        let right_flags = self.type_arena.get_flags(right);

        // Any absorbs everything
        if left_flags.intersects(TypeFlags::Any) || right_flags.intersects(TypeFlags::Any) {
            return self.any_type;
        }
        // Unknown absorbs everything
        if left_flags.intersects(TypeFlags::Unknown) || right_flags.intersects(TypeFlags::Unknown) {
            return self.unknown_type;
        }
        // Never is identity
        if left_flags.intersects(TypeFlags::Never) {
            return right;
        }
        if right_flags.intersects(TypeFlags::Never) {
            return left;
        }

        // TODO (Tier 3): Union distribution
        // if left is union → mapType(left, |t| get_spread_type(t, right))
        // if right is union → mapType(right, |t| get_spread_type(left, t))

        // Primitive on right → return left unchanged (spreading a primitive is a no-op)
        if right_flags.intersects(
            TypeFlags::BooleanLike
                | TypeFlags::NumberLike
                | TypeFlags::BigIntLike
                | TypeFlags::StringLike
                | TypeFlags::EnumLike
                | TypeFlags::NonPrimitive
                | TypeFlags::Index,
        ) {
            return left;
        }

        // TODO (Tier 4): Generic object types → create intersection
        // if is_generic_object_type(left) || is_generic_object_type(right) {
        //     return get_intersection_type([left, right])
        // }

        // Concrete object merge
        self.merge_spread_types(left, right)
    }

    /// Merge two concrete object types for a spread operation.
    ///
    /// Right properties override left properties. When both sides have the
    /// same property and the right's is optional, a union of both types is
    /// created with the optionality of the left property preserved.
    fn merge_spread_types(&mut self, left: TypeId, right: TypeId) -> TypeId {
        // Phase 1: Read — extract property data from the arena as Copy/cheap types.
        // Arena references are stable (AppendOnlyVec), but we still need to release
        // the borrows before calling &mut self methods like get_or_create_union_type.
        // We extract only the data we need: names (CompactStr is cheap to clone),
        // TypeIds (Copy), and flags (Copy).

        let right_entries: Vec<(CompactStr, TypeId, bool, bool)> =
            if let TypeData::Structured(s) = self.type_arena.get_data(right) {
                s.properties_in_decl_order()
                    .iter()
                    .map(|p| (p.name.clone(), p.type_id, p.optional, p.readonly))
                    .collect()
            } else {
                Vec::new()
            };

        let left_entries: Vec<(CompactStr, TypeId, bool, bool)> =
            if let TypeData::Structured(s) = self.type_arena.get_data(left) {
                s.properties_in_decl_order()
                    .iter()
                    .map(|p| (p.name.clone(), p.type_id, p.optional, p.readonly))
                    .collect()
            } else {
                Vec::new()
            };

        let (l_str_idx, l_num_idx) = if let TypeData::Structured(s) = self.type_arena.get_data(left)
        {
            (s.string_index_type, s.number_index_type)
        } else {
            (None, None)
        };
        let (r_str_idx, r_num_idx) =
            if let TypeData::Structured(s) = self.type_arena.get_data(right) {
                (s.string_index_type, s.number_index_type)
            } else {
                (None, None)
            };

        // Phase 2: Merge — build result properties, calling &mut self as needed.

        // Start with right properties, tracking positions for O(1) overlap updates
        let mut result_props: Vec<PropertyInfo> = right_entries
            .iter()
            .map(|(name, type_id, optional, readonly)| {
                let mut p = PropertyInfo::new(name.clone(), *type_id);
                p.optional = *optional;
                p.readonly = *readonly;
                p
            })
            .collect();
        let right_index: FxHashMap<&CompactStr, usize> =
            right_entries.iter().enumerate().map(|(i, (name, ..))| (name, i)).collect();

        for (name, left_type, left_optional, left_readonly) in &left_entries {
            if let Some(&idx) = right_index.get(name) {
                // Both sides have this property
                if result_props[idx].optional {
                    // Right is optional: union the types, keep left's optionality
                    let union_type =
                        self.get_or_create_union_type(vec![*left_type, result_props[idx].type_id]);
                    result_props[idx].type_id = union_type;
                    result_props[idx].optional = *left_optional;
                }
                // If right is required, it wins — already in result_props
            } else {
                let mut p = PropertyInfo::new(name.clone(), *left_type);
                p.optional = *left_optional;
                p.readonly = *left_readonly;
                result_props.push(p);
            }
        }

        // Merge index signatures
        let string_index = if left == self.empty_object_type {
            r_str_idx
        } else {
            match (l_str_idx, r_str_idx) {
                (Some(l), Some(r)) => Some(self.get_or_create_union_type(vec![l, r])),
                (some, None) | (None, some) => some,
            }
        };
        let number_index = if left == self.empty_object_type {
            r_num_idx
        } else {
            match (l_num_idx, r_num_idx) {
                (Some(l), Some(r)) => Some(self.get_or_create_union_type(vec![l, r])),
                (some, None) | (None, some) => some,
            }
        };

        sort_properties(&mut result_props);
        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous
                | ObjectFlags::ObjectLiteral
                | ObjectFlags::ContainsObjectOrArrayLiteral
                | ObjectFlags::ContainsSpread,
            TypeData::Structured(Box::new(StructuredType {
                properties: result_props,
                string_index_type: string_index,
                number_index_type: number_index,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: None },
            })),
            None,
        )
    }

    /// Check if a type could contain type variables (type parameters or
    /// composite types that transitively contain them).
    ///
    /// This is the single-level check used at type creation time to propagate
    /// `CouldContainTypeVariables`. It works because the flag is set
    /// transitively: if a child has it, the parent gets it too.
    pub(crate) fn type_could_contain_type_variables(&self, type_id: TypeId) -> bool {
        let flags = self.type_arena.get_flags(type_id);
        flags.intersects(TypeFlags::Instantiable)
            || self
                .type_arena
                .get_object_flags(type_id)
                .intersects(ObjectFlags::CouldContainTypeVariables)
    }

    /// Check if a signature could contain type variables in its parameter
    /// types or return type.
    pub(crate) fn signature_could_contain_type_variables(&self, sig: &Signature) -> bool {
        sig.parameters.iter().any(|p| self.type_could_contain_type_variables(p.type_id))
            || self.type_could_contain_type_variables(sig.return_type)
    }

    /// Create a function type from a single signature.
    pub fn create_function_type(&mut self, signature: Signature) -> TypeId {
        let mut obj_flags = ObjectFlags::Anonymous;
        if self.signature_could_contain_type_variables(&signature) {
            obj_flags |= ObjectFlags::CouldContainTypeVariables;
        }
        self.type_arena.new_type(
            TypeFlags::Object,
            obj_flags,
            TypeData::Function(Box::new(FunctionType { signatures: smallvec::smallvec![signature] })),
            None,
        )
    }

    /// Create a function type from multiple signatures (overloaded functions).
    pub fn create_function_type_from_signatures(
        &mut self,
        signatures: smallvec::SmallVec<[Signature; 1]>,
    ) -> TypeId {
        let mut obj_flags = ObjectFlags::Anonymous;
        if signatures.iter().any(|s| self.signature_could_contain_type_variables(s)) {
            obj_flags |= ObjectFlags::CouldContainTypeVariables;
        }
        self.type_arena.new_type(
            TypeFlags::Object,
            obj_flags,
            TypeData::Function(Box::new(FunctionType { signatures })),
            None,
        )
    }

    /// Create a constructor type from a single construct signature.
    pub fn create_constructor_type(&mut self, signature: Signature) -> TypeId {
        let mut obj_flags = ObjectFlags::Anonymous;
        if self.signature_could_contain_type_variables(&signature) {
            obj_flags |= ObjectFlags::CouldContainTypeVariables;
        }
        self.type_arena.new_type(
            TypeFlags::Object,
            obj_flags,
            TypeData::Structured(Box::new(StructuredType {
                properties: Vec::new(),
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: vec![signature],
                kind: StructuredTypeKind::Anonymous { target: None },
            })),
            None,
        )
    }
}

impl TypeFactory for Checker<'_> {
    fn never_type(&self) -> TypeId {
        self.never_type
    }

    fn instantiate_type_recursive(
        &mut self,
        arena: &TypeArena,
        type_id: TypeId,
        mapper: &TypeMapper,
    ) -> TypeId {
        // Try common cases (TypeParameter, Union, Intersection, TypeRef, Structured, Function)
        if let Some(result) = instantiate_type_common(arena, self, type_id, mapper) {
            return result;
        }

        // Complex cases not handled by common code
        self.instantiate_type_complex(type_id, mapper)
    }

    fn create_union(&mut self, types: Vec<TypeId>) -> TypeId {
        self.get_or_create_union_type(types)
    }

    fn create_intersection(&mut self, types: Vec<TypeId>) -> TypeId {
        self.get_or_create_intersection_type(types)
    }

    fn create_type_reference(
        &mut self,
        target: TypeId,
        args: SmallVec<[TypeId; 4]>,
    ) -> TypeId {
        self.get_or_create_type_reference(target, args)
    }
}
