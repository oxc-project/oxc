//! Cross-file interface type merging.
//!
//! When multiple ambient files (e.g., lib.es5.d.ts + lib.es2015.core.d.ts)
//! declare the same interface (`interface Array<T>`), this module merges
//! them into a single StructuredType with combined properties, call
//! signatures, index signatures, and base types.
//!
//! Type parameter remapping is handled via `TypeFactory` + `instantiate_type_common`:
//! each file's checker creates its own TypeParameter TypeIds for `T`, so the
//! extension file's property types must be instantiated through a mapper
//! that maps the extension's `T` to the base's `T`.

use std::sync::Arc;

use oxc_checker_host::IntrinsicIds;
use oxc_types::{
    ObjectFlags, PropertyInfo, StructuredType, StructuredTypeKind, TypeArena, TypeData, TypeFactory,
    TypeFlags, TypeId, TypeMapper, TypeReferenceType, UnionType, sort_properties,
};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

/// Lightweight `TypeFactory` for merge operations.
///
/// Owns its own dedup caches for unions, intersections, and type references.
/// No Checker or Semantic needed — only arena access.
pub(crate) struct MergeContext<'a> {
    arena: &'a TypeArena,
    intrinsics: IntrinsicIds,
    union_cache: FxHashMap<Arc<SmallVec<[TypeId; 4]>>, TypeId>,
    intersection_cache: FxHashMap<SmallVec<[TypeId; 4]>, TypeId>,
    type_ref_cache: FxHashMap<(TypeId, SmallVec<[TypeId; 4]>), TypeId>,
}

impl<'a> MergeContext<'a> {
    pub(crate) fn new(arena: &'a TypeArena, intrinsics: IntrinsicIds) -> Self {
        Self {
            arena,
            intrinsics,
            union_cache: FxHashMap::default(),
            intersection_cache: FxHashMap::default(),
            type_ref_cache: FxHashMap::default(),
        }
    }
}

impl TypeFactory for MergeContext<'_> {
    fn never_type(&self) -> TypeId {
        self.intrinsics.never_type
    }

    fn create_union(&mut self, mut types: Vec<TypeId>) -> TypeId {
        // Simplified normalization: filter never, sort, dedup.
        // Sufficient for type parameter remapping where inputs are
        // already normalized by the per-file Checker.
        let never = self.intrinsics.never_type;
        types.retain(|&t| t != never);

        if types.is_empty() {
            return never;
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
        let arena = self.arena;
        *self.union_cache.entry(key.clone()).or_insert_with(|| {
            arena.new_type(
                TypeFlags::Union,
                ObjectFlags::None,
                TypeData::Union(UnionType { types: key }),
                None,
            )
        })
    }

    fn create_intersection(&mut self, mut types: Vec<TypeId>) -> TypeId {
        // Simplified: dedup preserving order, no complex normalization.
        let never = self.intrinsics.never_type;
        if types.iter().any(|&t| t == never) {
            return never;
        }

        // Order-preserving dedup
        let mut seen = rustc_hash::FxHashSet::default();
        types.retain(|t| seen.insert(*t));

        if types.is_empty() {
            return self.intrinsics.unknown_type;
        }
        if types.len() == 1 {
            return types[0];
        }

        let key: SmallVec<[TypeId; 4]> = SmallVec::from_vec(types);
        let arena = self.arena;
        *self.intersection_cache.entry(key.clone()).or_insert_with(|| {
            arena.new_type(
                TypeFlags::Intersection,
                ObjectFlags::None,
                TypeData::Intersection(oxc_types::IntersectionType { types: key }),
                None,
            )
        })
    }

    fn create_type_reference(
        &mut self,
        target: TypeId,
        args: SmallVec<[TypeId; 4]>,
    ) -> TypeId {
        let key = (target, args.clone());
        let arena = self.arena;
        *self.type_ref_cache.entry(key).or_insert_with(|| {
            let mut obj_flags = ObjectFlags::Reference;
            if args.iter().any(|&t| {
                let f = arena.get_flags(t);
                f.intersects(TypeFlags::Instantiable)
                    || arena
                        .get_object_flags(t)
                        .intersects(ObjectFlags::CouldContainTypeVariables)
            }) {
                obj_flags |= ObjectFlags::CouldContainTypeVariables;
            }
            arena.new_type(
                TypeFlags::Object,
                obj_flags,
                TypeData::TypeReference(TypeReferenceType {
                    target: Some(target),
                    resolved_type_arguments: args,
                }),
                None,
            )
        })
    }
}

/// Merge two interface TypeIds from different files into one.
///
/// `base` is the type from the earlier file (e.g., lib.es5.d.ts).
/// `extension` is the type from the later file (e.g., lib.es2015.core.d.ts).
///
/// The merged type uses the base's type parameters as canonical.
/// Extension properties are instantiated through a mapper that remaps
/// the extension's type parameters to the base's.
///
/// Call signatures follow TypeScript's merge ordering: extension's
/// signatures come first (later declarations take priority in overload
/// resolution).
pub(crate) fn merge_interface_types(
    ctx: &mut MergeContext<'_>,
    base: TypeId,
    extension: TypeId,
) -> TypeId {
    let arena = ctx.arena;

    // Both must be Structured/Interface types
    let TypeData::Structured(base_s) = arena.get_data(base) else {
        return base;
    };
    let TypeData::Structured(ext_s) = arena.get_data(extension) else {
        return base;
    };

    // Extract type parameters
    let (base_type_params, base_base_types) = match &base_s.kind {
        StructuredTypeKind::Interface {
            all_type_parameters,
            resolved_base_types,
            ..
        } => (all_type_parameters.clone(), resolved_base_types.clone()),
        _ => return base,
    };

    let ext_type_params = match &ext_s.kind {
        StructuredTypeKind::Interface {
            all_type_parameters,
            ..
        } => all_type_parameters.clone(),
        _ => return base,
    };

    // Validate type parameter arity matches
    if base_type_params.len() != ext_type_params.len() {
        return base; // can't merge — arity mismatch
    }

    // Build mapper: extension's type params → base's type params
    let mapper = if !base_type_params.is_empty() {
        TypeMapper::from_type_parameters(&ext_type_params, &base_type_params)
    } else {
        None
    };

    // Re-read extension data (arena references are stable, but we need
    // fresh borrows after the clones above released them)
    let TypeData::Structured(ext_s) = arena.get_data(extension) else {
        unreachable!()
    };

    // Helper: remap a type through the mapper via the trait method.
    // Uses instantiate_type_recursive so the factory controls recursion.
    let remap = |ctx: &mut MergeContext<'_>, t: TypeId| -> TypeId {
        if let Some(ref mapper) = mapper {
            ctx.instantiate_type_recursive(arena, t, mapper)
        } else {
            t
        }
    };

    // Instantiate extension's properties through the mapper
    let remapped_ext_props: Vec<PropertyInfo> = ext_s
        .properties
        .iter()
        .map(|p| PropertyInfo {
            name: p.name.clone(),
            type_id: remap(ctx, p.type_id),
            optional: p.optional,
            readonly: p.readonly,
            decl_order: 0,
        })
        .collect();

    // Instantiate extension's call signatures through the mapper
    let remapped_ext_call_sigs: Vec<oxc_types::Signature> = ext_s
        .call_signatures
        .iter()
        .map(|sig| {
            if let Some(ref mapper) = mapper {
                oxc_types::instantiate_signature(arena, ctx, sig, mapper)
            } else {
                sig.clone()
            }
        })
        .collect();

    // Instantiate extension's construct signatures
    let remapped_ext_construct_sigs: Vec<oxc_types::Signature> = ext_s
        .construct_signatures
        .iter()
        .map(|sig| {
            if let Some(ref mapper) = mapper {
                oxc_types::instantiate_signature(arena, ctx, sig, mapper)
            } else {
                sig.clone()
            }
        })
        .collect();

    // Extension's index signatures (remapped)
    let ext_string_index = ext_s.string_index_type.map(|t| remap(ctx, t));
    let ext_number_index = ext_s.number_index_type.map(|t| remap(ctx, t));

    // Extension's base types (remapped)
    let ext_base_types: SmallVec<[TypeId; 4]> = match &ext_s.kind {
        StructuredTypeKind::Interface {
            resolved_base_types,
            ..
        } => resolved_base_types
            .iter()
            .map(|&bt| remap(ctx, bt))
            .collect(),
        _ => SmallVec::new(),
    };

    // Re-read base data for combining
    let TypeData::Structured(base_s) = arena.get_data(base) else {
        unreachable!()
    };

    // Combine properties: base first, then extension (additive).
    // In TypeScript, duplicate property names across merged declarations must
    // have identical types (TS2717). For lib files they're always additive
    // (no conflicts), so we keep the first occurrence of each name.
    let mut prop_map: FxHashMap<&str, PropertyInfo> = FxHashMap::default();
    for p in &base_s.properties {
        prop_map.entry(p.name.as_str()).or_insert_with(|| p.clone());
    }
    for p in &remapped_ext_props {
        prop_map.entry(p.name.as_str()).or_insert_with(|| p.clone());
    }
    let mut merged_props: Vec<PropertyInfo> = prop_map.into_values().collect();
    sort_properties(&mut merged_props);

    // Combine call signatures: extension first (TypeScript overload ordering)
    let mut merged_call_sigs = remapped_ext_call_sigs;
    merged_call_sigs.extend(base_s.call_signatures.iter().cloned());

    // Combine construct signatures: extension first (same ordering as call sigs)
    let mut merged_construct_sigs = remapped_ext_construct_sigs;
    merged_construct_sigs.extend(base_s.construct_signatures.iter().cloned());

    // Index signatures: extension overrides base
    let merged_string_index = ext_string_index.or(base_s.string_index_type);
    let merged_number_index = ext_number_index.or(base_s.number_index_type);

    // Combine base types: union of both (deduped)
    let mut merged_base_types = base_base_types;
    for &bt in &ext_base_types {
        if !merged_base_types.contains(&bt) {
            merged_base_types.push(bt);
        }
    }

    // Preserve the base type's symbol for display
    let symbol = arena.get_symbol(base);

    arena.new_type(
        TypeFlags::Object,
        ObjectFlags::Interface,
        TypeData::Structured(Box::new(StructuredType {
            properties: merged_props,
            string_index_type: merged_string_index,
            number_index_type: merged_number_index,
            call_signatures: merged_call_sigs,
            construct_signatures: merged_construct_sigs,
            kind: StructuredTypeKind::Interface {
                target: None,
                resolved_type_arguments: SmallVec::new(),
                all_type_parameters: base_type_params,
                this_type: None,
                resolved_base_types: merged_base_types,
            },
        })),
        symbol,
    )
}
