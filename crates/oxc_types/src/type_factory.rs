//! Shared type instantiation infrastructure.
//!
//! Provides `TypeFactory` (trait for creating composite types) and
//! `instantiate_type_common` (free function handling common instantiation
//! cases). Both `Checker` and `MergeContext` implement `TypeFactory`,
//! enabling type parameter remapping in both per-file checking and
//! cross-file declaration merging.
//!
//! The key architectural choice: `instantiate_type_common` takes `arena`
//! and `factory` as separate parameters. Arena references have lifetime `'a`
//! (tied to the arena, not to factory), so arena reads don't block factory
//! mutation. This avoids the borrow conflict that would arise if `factory`
//! owned the arena reference.

use smallvec::SmallVec;

use crate::{
    FunctionType, ObjectFlags, ParameterInfo, PropertyInfo, Signature, StructuredType,
    StructuredTypeKind, TypeArena, TypeData, TypeFlags, TypeId, TypeMapper, sort_properties,
};

/// Check if a type could contain type variables (type parameters or composite
/// types that transitively contain them). Pure arena read — no Checker needed.
pub fn type_could_contain_type_variables(arena: &TypeArena, type_id: TypeId) -> bool {
    let flags = arena.get_flags(type_id);
    flags.intersects(TypeFlags::Instantiable)
        || arena
            .get_object_flags(type_id)
            .intersects(ObjectFlags::CouldContainTypeVariables)
}

/// Check if a signature could contain type variables.
pub fn signature_could_contain_type_variables(arena: &TypeArena, sig: &Signature) -> bool {
    sig.parameters
        .iter()
        .any(|p| type_could_contain_type_variables(arena, p.type_id))
        || type_could_contain_type_variables(arena, sig.return_type)
}

/// Trait for creating composite types during instantiation.
///
/// Both `Checker` (full normalization, dedup caches) and `MergeContext`
/// (lightweight dedup) implement this. The `instantiate_type_recursive`
/// method controls recursion — override it to handle complex cases
/// (Conditional, Mapped, etc.) beyond what `instantiate_type_common` covers.
pub trait TypeFactory {
    fn never_type(&self) -> TypeId;

    fn create_union(&mut self, types: Vec<TypeId>) -> TypeId;
    fn create_intersection(&mut self, types: Vec<TypeId>) -> TypeId;
    fn create_type_reference(
        &mut self,
        target: TypeId,
        args: SmallVec<[TypeId; 4]>,
    ) -> TypeId;

    /// Instantiate a type by applying a mapper. Called recursively by
    /// `instantiate_type_common`. Override to handle complex cases.
    ///
    /// Default implementation handles common cases only (TypeParameter,
    /// Union, Intersection, TypeReference, Structured, Function).
    /// Returns type unchanged for Conditional, Mapped, Index, etc.
    fn instantiate_type_recursive(
        &mut self,
        arena: &TypeArena,
        type_id: TypeId,
        mapper: &TypeMapper,
    ) -> TypeId
    where
        Self: Sized,
    {
        instantiate_type_common(arena, self, type_id, mapper).unwrap_or(type_id)
    }
}

/// Handle common instantiation cases: TypeParameter, Union, Intersection,
/// Object (TypeReference + Structured + Function).
///
/// Returns `Some(result)` for handled cases, `None` for unhandled
/// (Conditional, Mapped, Tuple, Index, IndexedAccess). Callers that need
/// those cases (i.e., Checker) handle them in their `instantiate_type_recursive`
/// override.
pub fn instantiate_type_common(
    arena: &TypeArena,
    factory: &mut impl TypeFactory,
    type_id: TypeId,
    mapper: &TypeMapper,
) -> Option<TypeId> {
    let flags = arena.get_flags(type_id);

    // Quick reject: types that can never contain type parameters.
    if !flags.intersects(TypeFlags::StructuredOrInstantiable) {
        return Some(type_id);
    }

    // TypeParameter — base case: apply the mapper
    if flags.intersects(TypeFlags::TypeParameter) {
        return Some(mapper.map(type_id).unwrap_or(type_id));
    }

    // Union — instantiate each constituent
    if flags.intersects(TypeFlags::Union) {
        if let TypeData::Union(u) = arena.get_data(type_id) {
            let new_members: Vec<TypeId> = u
                .types
                .iter()
                .map(|&t| factory.instantiate_type_recursive(arena, t, mapper))
                .collect();
            return Some(factory.create_union(new_members));
        }
    }

    // Intersection — instantiate each constituent
    if flags.intersects(TypeFlags::Intersection) {
        if let TypeData::Intersection(i) = arena.get_data(type_id) {
            let new_members: Vec<TypeId> = i
                .types
                .iter()
                .map(|&t| factory.instantiate_type_recursive(arena, t, mapper))
                .collect();
            return Some(factory.create_intersection(new_members));
        }
    }

    // Object types: TypeReference, Structured, Function
    // (Mapped and Tuple are not handled — return None for those)
    if flags.intersects(TypeFlags::Object) {
        return instantiate_object_type_common(arena, factory, type_id, mapper);
    }

    // Index, IndexedAccess, Conditional — not handled by common code
    None
}

/// Handle common object type instantiation: TypeReference, Structured, Function.
/// Returns `None` for Mapped and Tuple (handled by Checker override).
fn instantiate_object_type_common(
    arena: &TypeArena,
    factory: &mut impl TypeFactory,
    type_id: TypeId,
    mapper: &TypeMapper,
) -> Option<TypeId> {
    match arena.get_data(type_id) {
        TypeData::TypeReference(tr) => {
            let target = tr.target;
            let new_args: SmallVec<[TypeId; 4]> = tr
                .resolved_type_arguments
                .iter()
                .map(|&t| factory.instantiate_type_recursive(arena, t, mapper))
                .collect();

            if new_args[..] == tr.resolved_type_arguments[..] {
                return Some(type_id); // no change
            }

            let Some(target) = target else {
                return Some(type_id);
            };
            Some(factory.create_type_reference(target, new_args))
        }

        TypeData::Structured(s) => Some(instantiate_structured_type(
            arena,
            factory,
            type_id,
            &s.properties,
            &s.call_signatures,
            &s.construct_signatures,
            &s.kind,
            s.string_index_type,
            s.number_index_type,
            mapper,
        )),

        TypeData::Function(func) => {
            let sigs: SmallVec<[Signature; 1]> = func
                .signatures
                .iter()
                .map(|sig| instantiate_signature(arena, factory, sig, mapper))
                .collect();
            // Check if anything actually changed
            let changed = sigs.iter().zip(func.signatures.iter()).any(|(new, old)| {
                new.return_type != old.return_type
                    || new.parameters.len() != old.parameters.len()
                    || new
                        .parameters
                        .iter()
                        .zip(old.parameters.iter())
                        .any(|(np, op)| np.type_id != op.type_id)
            });
            if !changed {
                return Some(type_id);
            }
            let mut obj_flags = ObjectFlags::Anonymous;
            if sigs
                .iter()
                .any(|s| signature_could_contain_type_variables(arena, s))
            {
                obj_flags |= ObjectFlags::CouldContainTypeVariables;
            }
            Some(arena.new_type(
                TypeFlags::Object,
                obj_flags,
                TypeData::Function(Box::new(FunctionType { signatures: sigs })),
                None,
            ))
        }

        // Mapped, Tuple — handled by Checker override
        _ => None,
    }
}

/// Instantiate a signature with a type mapper.
pub fn instantiate_signature(
    arena: &TypeArena,
    factory: &mut impl TypeFactory,
    sig: &Signature,
    mapper: &TypeMapper,
) -> Signature {
    let new_params: Vec<ParameterInfo> = sig
        .parameters
        .iter()
        .map(|p| ParameterInfo {
            name: p.name.clone(),
            type_id: factory.instantiate_type_recursive(arena, p.type_id, mapper),
            is_optional: p.is_optional,
            is_rest: p.is_rest,
        })
        .collect();
    let new_return = factory.instantiate_type_recursive(arena, sig.return_type, mapper);
    Signature {
        flags: sig.flags,
        min_argument_count: sig.min_argument_count,
        parameters: new_params,
        return_type: new_return,
        type_parameters: sig.type_parameters.clone(),
    }
}

/// Instantiate a structured type: properties, call/construct signatures,
/// and base types. Returns the original type_id if nothing changed.
/// Instantiate a structured type: properties, call/construct signatures,
/// index signatures, and base types. Returns the original type_id if
/// nothing changed. Preserves ObjectFlags from the original type.
#[allow(clippy::too_many_arguments)]
pub fn instantiate_structured_type(
    arena: &TypeArena,
    factory: &mut impl TypeFactory,
    type_id: TypeId,
    properties: &[PropertyInfo],
    call_signatures: &[Signature],
    construct_signatures: &[Signature],
    kind: &StructuredTypeKind,
    string_index_type: Option<TypeId>,
    number_index_type: Option<TypeId>,
    mapper: &TypeMapper,
) -> TypeId {
    let mut changed = false;

    let mut new_props: Vec<PropertyInfo> = properties
        .iter()
        .map(|p| {
            let new_type_id = factory.instantiate_type_recursive(arena, p.type_id, mapper);
            if new_type_id != p.type_id {
                changed = true;
            }
            PropertyInfo {
                name: p.name.clone(),
                type_id: new_type_id,
                optional: p.optional,
                readonly: p.readonly,
                decl_order: 0,
            }
        })
        .collect();

    let new_call_sigs: Vec<Signature> = call_signatures
        .iter()
        .map(|sig| {
            let new_sig = instantiate_signature(arena, factory, sig, mapper);
            if new_sig.return_type != sig.return_type
                || new_sig
                    .parameters
                    .iter()
                    .zip(sig.parameters.iter())
                    .any(|(n, o)| n.type_id != o.type_id)
            {
                changed = true;
            }
            new_sig
        })
        .collect();

    let new_construct_sigs: Vec<Signature> = construct_signatures
        .iter()
        .map(|sig| {
            let new_sig = instantiate_signature(arena, factory, sig, mapper);
            if new_sig.return_type != sig.return_type
                || new_sig
                    .parameters
                    .iter()
                    .zip(sig.parameters.iter())
                    .any(|(n, o)| n.type_id != o.type_id)
            {
                changed = true;
            }
            new_sig
        })
        .collect();

    // Instantiate resolved_base_types in the kind if present
    let instantiated_bases: Option<SmallVec<[TypeId; 4]>> =
        if let StructuredTypeKind::Interface { resolved_base_types, .. } = kind {
            if resolved_base_types.is_empty() {
                None
            } else {
                let mut bc = false;
                let bases: SmallVec<[TypeId; 4]> = resolved_base_types
                    .iter()
                    .map(|&bt| {
                        let new_bt = factory.instantiate_type_recursive(arena, bt, mapper);
                        if new_bt != bt {
                            bc = true;
                        }
                        new_bt
                    })
                    .collect();
                if bc {
                    changed = true;
                    Some(bases)
                } else {
                    None
                }
            }
        } else {
            None
        };

    if !changed {
        return type_id;
    }

    // Build the final kind — only clone when we need a new type
    let final_kind = if let Some(new_bases) = instantiated_bases {
        let mut cloned = kind.clone();
        if let StructuredTypeKind::Interface { resolved_base_types, .. } = &mut cloned {
            *resolved_base_types = new_bases;
        }
        cloned
    } else {
        kind.clone()
    };

    // Preserve original ObjectFlags (Interface, Class, Anonymous, etc.)
    // and propagate CouldContainTypeVariables if any member still has type vars.
    let mut obj_flags = arena.get_object_flags(type_id);
    let has_type_vars = new_props
        .iter()
        .any(|p| type_could_contain_type_variables(arena, p.type_id))
        || new_call_sigs
            .iter()
            .any(|s| signature_could_contain_type_variables(arena, s))
        || new_construct_sigs
            .iter()
            .any(|s| signature_could_contain_type_variables(arena, s));
    if has_type_vars {
        obj_flags |= ObjectFlags::CouldContainTypeVariables;
    } else {
        obj_flags -= ObjectFlags::CouldContainTypeVariables;
    }

    sort_properties(&mut new_props);
    arena.new_type(
        TypeFlags::Object,
        obj_flags,
        TypeData::Structured(Box::new(StructuredType {
            properties: new_props,
            string_index_type,
            number_index_type,
            call_signatures: new_call_sigs,
            construct_signatures: new_construct_sigs,
            kind: final_kind,
        })),
        None,
    )
}
