use oxc_types::{
    ObjectFlags, PropertyInfo, StructuredType, StructuredTypeKind, TypeData, TypeFactory,
    TypeFlags, TypeId, TypeMapper, sort_properties,
};
use smallvec::SmallVec;

use crate::Checker;

impl<'a> Checker<'a> {
    /// Resolve a TypeReference to a fully-instantiated type with populated
    /// member_map. Results are cached in `instantiation_cache`.
    ///
    /// For `Array<string>`, this looks up `Array<T>`'s declared members,
    /// substitutes T→string, creates a new StructuredType in the arena with
    /// the instantiated properties + member_map, and caches the result.
    /// Subsequent calls for the same TypeReference return the cached type.
    ///
    /// Mirrors tsgo's lazy `resolveStructuredTypeMembers` +
    /// instantiation cache.
    pub(crate) fn resolve_type_reference(&mut self, type_ref_id: TypeId) -> TypeId {
        if let Some(&cached) = self.instantiation_cache.get(&type_ref_id) {
            return cached;
        }

        let TypeData::TypeReference(tr) = self.type_arena.get_data(type_ref_id) else {
            return type_ref_id;
        };
        let Some(target) = tr.target else {
            return type_ref_id;
        };
        let type_args = &tr.resolved_type_arguments;

        // Dispatch based on target type kind
        match self.type_arena.get_data(target) {
            TypeData::Structured(s) => {
                let StructuredTypeKind::Interface { all_type_parameters, .. } = &s.kind else {
                    self.instantiation_cache.insert(type_ref_id, target);
                    return target;
                };
                let Some(mapper) = TypeMapper::from_type_parameters(all_type_parameters, type_args)
                else {
                    self.instantiation_cache.insert(type_ref_id, target);
                    return target;
                };

                // Instantiate all members. Arena references are stable (AppendOnlyVec).
                let TypeData::Structured(s) = self.type_arena.get_data(target) else {
                    unreachable!()
                };
                let mut instantiated_props: Vec<PropertyInfo> = s
                    .properties
                    .iter()
                    .map(|p| PropertyInfo {
                        name: p.name.clone(),
                        type_id: self.instantiate_type(p.type_id, &mapper),
                        optional: p.optional,
                        readonly: p.readonly,
                        decl_order: 0,
                    })
                    .collect();
                sort_properties(&mut instantiated_props);

                // Instantiate call and construct signatures
                let instantiated_call_sigs: Vec<oxc_types::Signature> = s
                    .call_signatures
                    .iter()
                    .map(|sig| oxc_types::instantiate_signature(self.type_arena, self, sig, &mapper))
                    .collect();
                let instantiated_construct_sigs: Vec<oxc_types::Signature> = s
                    .construct_signatures
                    .iter()
                    .map(|sig| oxc_types::instantiate_signature(self.type_arena, self, sig, &mapper))
                    .collect();

                // Instantiate index signatures
                let instantiated_string_index = s
                    .string_index_type
                    .map(|t| self.instantiate_type(t, &mapper));
                let instantiated_number_index = s
                    .number_index_type
                    .map(|t| self.instantiate_type(t, &mapper));

                // Instantiate base types through the mapper so that
                // `interface Child<T> extends Base<T>` instantiated as
                // `Child<string>` gets `Base<string>` as its base type.
                let instantiated_base_types: SmallVec<[TypeId; 4]> =
                    if let StructuredTypeKind::Interface { resolved_base_types, .. } = &s.kind {
                        resolved_base_types
                            .iter()
                            .map(|&bt| self.instantiate_type(bt, &mapper))
                            .collect()
                    } else {
                        SmallVec::new()
                    };

                let resolved_id = self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Interface,
                    TypeData::Structured(Box::new(StructuredType {
                        properties: instantiated_props,
                        string_index_type: instantiated_string_index,
                        number_index_type: instantiated_number_index,
                        call_signatures: instantiated_call_sigs,
                        construct_signatures: instantiated_construct_sigs,
                        kind: StructuredTypeKind::Interface {
                            target: Some(target),
                            resolved_type_arguments: type_args.clone(),
                            all_type_parameters: SmallVec::new(),
                            this_type: None,
                            resolved_base_types: instantiated_base_types,
                        },
                    })),
                    None,
                );

                self.instantiation_cache.insert(type_ref_id, resolved_id);
                resolved_id
            }

            _ => {
                self.instantiation_cache.insert(type_ref_id, type_ref_id);
                type_ref_id
            }
        }
    }
}

impl Checker<'_> {
    /// Instantiate a type by applying a type mapper.
    ///
    /// Convenience wrapper that calls through the `TypeFactory` trait,
    /// which delegates common cases to `instantiate_type_common` and
    /// handles complex cases (Conditional, Mapped, Tuple, Index,
    /// IndexedAccess) in `instantiate_type_complex`.
    pub fn instantiate_type(&mut self, type_id: TypeId, mapper: &TypeMapper) -> TypeId {
        let arena = self.type_arena;
        self.instantiate_type_recursive(arena, type_id, mapper)
    }

    /// Handle complex instantiation cases not covered by `instantiate_type_common`:
    /// Mapped, Tuple, Index, IndexedAccess, Conditional.
    pub(crate) fn instantiate_type_complex(
        &mut self,
        type_id: TypeId,
        mapper: &TypeMapper,
    ) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);

        // Complex object types: Mapped, Tuple
        if flags.intersects(TypeFlags::Object) {
            return self.instantiate_object_type_complex(type_id, mapper);
        }

        // keyof T — instantiate target, then resolve
        if flags.intersects(TypeFlags::Index) {
            if let TypeData::Index(idx) = self.type_arena.get_data(type_id) {
                let new_target = self.instantiate_type(idx.target, mapper);
                if new_target == idx.target {
                    return type_id;
                }
                return self.get_index_type(new_target);
            }
        }

        // T[K] — instantiate both, then resolve
        if flags.intersects(TypeFlags::IndexedAccess) {
            if let TypeData::IndexedAccess(ia) = self.type_arena.get_data(type_id) {
                let new_obj = self.instantiate_type(ia.object_type, mapper);
                let new_idx = self.instantiate_type(ia.index_type, mapper);
                if new_obj == ia.object_type && new_idx == ia.index_type {
                    return type_id;
                }
                return self.get_indexed_access_type(new_obj, new_idx);
            }
        }

        // Conditional: T extends U ? X : Y
        if flags.intersects(TypeFlags::Conditional) {
            if let TypeData::Conditional(cond) = self.type_arena.get_data(type_id) {
                let orig_check = cond.check_type;
                let orig_extends = cond.extends_type;
                let orig_true = cond.true_type;
                let orig_false = cond.false_type;
                let is_distributive = cond.is_distributive;
                let infer_type_parameters = cond.infer_type_parameters.clone();

                let new_check = self.instantiate_type(orig_check, mapper);

                // Distribution: if distributive and check resolved to a union,
                // distribute over each member.
                if is_distributive {
                    let check_flags = self.type_arena.get_flags(new_check);
                    if check_flags.intersects(TypeFlags::Union) {
                        if let TypeData::Union(u) = self.type_arena.get_data(new_check) {
                            let members: Vec<TypeId> = u.types.iter().copied().collect();
                            let infer_params = infer_type_parameters.clone();
                            let results: Vec<TypeId> = members
                                .iter()
                                .map(|&member| {
                                    let per_member =
                                        mapper.clone().with_mapping(orig_check, member);
                                    let ext = self.instantiate_type(orig_extends, &per_member);
                                    let tru = self.instantiate_type(orig_true, &per_member);
                                    let fal = self.instantiate_type(orig_false, &per_member);
                                    self.get_conditional_type(
                                        member,
                                        ext,
                                        tru,
                                        fal,
                                        is_distributive,
                                        infer_params.clone(),
                                    )
                                })
                                .collect();
                            return self.get_or_create_union_type(results);
                        }
                    }
                }

                let new_extends = self.instantiate_type(orig_extends, mapper);
                let new_true = self.instantiate_type(orig_true, mapper);
                let new_false = self.instantiate_type(orig_false, mapper);

                return self.get_conditional_type(
                    new_check,
                    new_extends,
                    new_true,
                    new_false,
                    is_distributive,
                    infer_type_parameters,
                );
            }
        }

        // TODO: TemplateLiteral, StringMapping

        type_id
    }

    /// Handle complex object type instantiation: Mapped and Tuple.
    /// (TypeReference, Structured, Function are handled by common code.)
    fn instantiate_object_type_complex(
        &mut self,
        type_id: TypeId,
        mapper: &TypeMapper,
    ) -> TypeId {
        match self.type_arena.get_data(type_id) {
            TypeData::Mapped(mapped) => {
                let constraint = mapped.constraint_type.unwrap_or(self.never_type);
                let template = mapped.template_type;
                let type_param = mapped.type_parameter;
                let optional_mod = mapped.optional_modifier;
                let readonly_mod = mapped.readonly_modifier;
                let name_type = mapped.name_type;

                // Homomorphic detection: if constraint is `keyof T` where T is a
                // type parameter, check what T resolves to and dispatch.
                // This handles Partial<T>, Required<T>, Readonly<T>, etc.
                let homomorphic_var = self.get_homomorphic_type_variable(constraint);
                if let Some(type_variable) = homomorphic_var {
                    let concrete = mapper.map(type_variable).unwrap_or(type_variable);
                    return self.instantiate_mapped_type_homomorphic(
                        type_id,
                        concrete,
                        template,
                        type_param,
                        optional_mod,
                        readonly_mod,
                        name_type,
                        mapper,
                    );
                }

                // Non-homomorphic path (Record<K, V>, etc.): resolve constraint
                // to concrete keys, instantiate template per key.
                // Not cached — the result depends on both constraint and mapper,
                // and the non-homomorphic path is infrequent.
                let new_constraint = self.instantiate_type(constraint, mapper);

                let Some(properties) = self.resolve_mapped_type_to_properties(
                    new_constraint,
                    template,
                    type_param,
                    optional_mod,
                    readonly_mod,
                    Some(mapper),
                    None,
                    name_type,
                ) else {
                    return type_id; // can't resolve — return as-is
                };

                self.build_mapped_object_type(type_id, properties)
            }

            TypeData::Tuple(tuple) => {
                let new_elements: Vec<oxc_types::TupleElementInfo> = tuple
                    .element_infos
                    .iter()
                    .map(|info| oxc_types::TupleElementInfo {
                        element_type: self.instantiate_type(info.element_type, mapper),
                        flags: info.flags,
                        label_name: info.label_name.clone(),
                    })
                    .collect();
                let changed = new_elements
                    .iter()
                    .zip(tuple.element_infos.iter())
                    .any(|(new, old)| new.element_type != old.element_type);
                if !changed {
                    return type_id;
                }
                let type_arguments: SmallVec<[TypeId; 4]> =
                    new_elements.iter().map(|e| e.element_type).collect();
                let mut obj_flags = oxc_types::ObjectFlags::Tuple;
                if new_elements
                    .iter()
                    .any(|e| self.type_could_contain_type_variables(e.element_type))
                {
                    obj_flags |= oxc_types::ObjectFlags::CouldContainTypeVariables;
                }
                self.type_arena.new_type(
                    TypeFlags::Object,
                    obj_flags,
                    TypeData::Tuple(Box::new(oxc_types::TupleType {
                        target: tuple.target,
                        resolved_type_arguments: type_arguments,
                        min_length: tuple.min_length,
                        fixed_length: tuple.fixed_length,
                        combined_flags: tuple.combined_flags,
                        readonly: tuple.readonly,
                        element_infos: new_elements,
                    })),
                    None,
                )
            }

            _ => type_id,
        }
    }

    /// Detect if a mapped type is homomorphic: constraint is `keyof T` where
    /// T is a type parameter. Returns the type variable T if so.
    /// Mirrors tsgo's `getHomomorphicTypeVariable`.
    fn get_homomorphic_type_variable(&self, constraint: TypeId) -> Option<TypeId> {
        let flags = self.type_arena.get_flags(constraint);
        if !flags.intersects(TypeFlags::Index) {
            return None;
        }
        let TypeData::Index(idx) = self.type_arena.get_data(constraint) else {
            return None;
        };
        let target_flags = self.type_arena.get_flags(idx.target);
        if target_flags.intersects(TypeFlags::TypeParameter) { Some(idx.target) } else { None }
    }

    /// Instantiate a homomorphic mapped type (e.g., `Partial<T>` where T has
    /// been resolved to a concrete type).
    ///
    /// Dispatches on the concrete type:
    /// - Primitives: return unchanged (Partial<string> = string)
    /// - Unions: distribute (Partial<A | B> = Partial<A> | Partial<B>)
    /// - Arrays: map element type, preserve array wrapper
    /// - Tuples: map each element type, preserve tuple structure
    /// - Objects/Interfaces: enumerate keys, instantiate template per key
    ///
    /// Mirrors tsgo's `instantiateMappedType`.
    #[allow(clippy::too_many_arguments)]
    fn instantiate_mapped_type_homomorphic(
        &mut self,
        mapped_type_id: TypeId,
        concrete: TypeId,
        template: Option<TypeId>,
        type_param: TypeId,
        optional_mod: oxc_types::MappedTypeModifier,
        readonly_mod: oxc_types::MappedTypeModifier,
        name_type: Option<TypeId>,
        outer_mapper: &TypeMapper,
    ) -> TypeId {
        let concrete_flags = self.type_arena.get_flags(concrete);

        // Cache check
        let cache_key = (mapped_type_id.index() as u64) << 32 | concrete.index() as u64;
        if let Some(&cached) = self.mapped_type_cache.get(&cache_key) {
            return cached;
        }

        // Primitives pass through unchanged: Partial<string> = string
        if concrete_flags.intersects(
            TypeFlags::String
                | TypeFlags::Number
                | TypeFlags::Boolean
                | TypeFlags::BigInt
                | TypeFlags::ESSymbol
                | TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BooleanLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::Void
                | TypeFlags::Undefined
                | TypeFlags::Null
                | TypeFlags::Never,
        ) {
            self.mapped_type_cache.insert(cache_key, concrete);
            return concrete;
        }

        // Unions: distribute Partial<A | B> = Partial<A> | Partial<B>
        if concrete_flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(concrete) {
                let members: Vec<TypeId> = u.types.iter().copied().collect();
                let results: Vec<TypeId> = members
                    .iter()
                    .map(|&member| {
                        self.instantiate_mapped_type_homomorphic(
                            mapped_type_id,
                            member,
                            template,
                            type_param,
                            optional_mod,
                            readonly_mod,
                            name_type,
                            outer_mapper,
                        )
                    })
                    .collect();
                let result = self.get_or_create_union_type(results);
                self.mapped_type_cache.insert(cache_key, result);
                return result;
            }
        }

        // Arrays: map element type, preserve array wrapper.
        // Partial<string[]> → (string | undefined)[]
        if concrete_flags.intersects(TypeFlags::Object) {
            if let TypeData::TypeReference(tr) = self.type_arena.get_data(concrete) {
                if tr.target == Some(self.array_type) && !tr.resolved_type_arguments.is_empty() {
                    let elem_type = tr.resolved_type_arguments[0];
                    let mapped_elem = self.instantiate_mapped_element_type(
                        elem_type,
                        template,
                        type_param,
                        optional_mod,
                        outer_mapper,
                    );
                    let result = self.get_or_create_type_reference(
                        self.array_type,
                        smallvec::smallvec![mapped_elem],
                    );
                    self.mapped_type_cache.insert(cache_key, result);
                    return result;
                }
            }

            // Tuples: map each element type, preserve tuple structure.
            // Partial<[string, number]> → [string?, number?]
            if let TypeData::Tuple(tuple) = self.type_arena.get_data(concrete) {
                let readonly = tuple.readonly;
                let num_elements = tuple.element_infos.len();
                let mut new_elements = Vec::with_capacity(num_elements);
                for i in 0..num_elements {
                    // Re-access tuple from arena each iteration. The arena is
                    // append-only so the reference is stable, but the for loop
                    // avoids holding a closure that borrows both the arena ref
                    // and &mut self simultaneously.
                    let TypeData::Tuple(tuple) = self.type_arena.get_data(concrete) else {
                        unreachable!()
                    };
                    let info = &tuple.element_infos[i];
                    let elem_type = info.element_type;
                    let flags = info.flags;
                    let label_name = info.label_name.clone();

                    let mapped_elem = self.instantiate_mapped_element_type(
                        elem_type,
                        template,
                        type_param,
                        optional_mod,
                        outer_mapper,
                    );
                    let new_flags = match optional_mod {
                        oxc_types::MappedTypeModifier::Add => {
                            (flags - oxc_types::ElementFlags::Required)
                                | oxc_types::ElementFlags::Optional
                        }
                        oxc_types::MappedTypeModifier::Remove => {
                            (flags - oxc_types::ElementFlags::Optional)
                                | oxc_types::ElementFlags::Required
                        }
                        oxc_types::MappedTypeModifier::None => flags,
                    };
                    new_elements.push(oxc_types::TupleElementInfo {
                        element_type: mapped_elem,
                        flags: new_flags,
                        label_name,
                    });
                }

                let min_length = new_elements
                    .iter()
                    .filter(|e| e.flags.contains(oxc_types::ElementFlags::Required))
                    .count() as u32;
                let has_rest =
                    new_elements.iter().any(|e| e.flags.contains(oxc_types::ElementFlags::Rest));
                let fixed_length = if has_rest {
                    new_elements.len() as u32 - 1
                } else {
                    new_elements.len() as u32
                };
                let combined_flags = new_elements
                    .iter()
                    .fold(oxc_types::ElementFlags::empty(), |acc, e| acc | e.flags);
                let type_arguments: SmallVec<[TypeId; 4]> =
                    new_elements.iter().map(|e| e.element_type).collect();

                let new_readonly = match readonly_mod {
                    oxc_types::MappedTypeModifier::Add => true,
                    oxc_types::MappedTypeModifier::Remove => false,
                    oxc_types::MappedTypeModifier::None => readonly,
                };

                let result = self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Tuple,
                    TypeData::Tuple(Box::new(oxc_types::TupleType {
                        target: None,
                        resolved_type_arguments: type_arguments,
                        element_infos: new_elements,
                        min_length,
                        fixed_length,
                        combined_flags,
                        readonly: new_readonly,
                    })),
                    None,
                );
                self.mapped_type_cache.insert(cache_key, result);
                return result;
            }
        }

        // Objects/Interfaces: resolve keyof to get keys, instantiate template
        let keyof_concrete = self.get_index_type(concrete);
        let Some(properties) = self.resolve_mapped_type_to_properties(
            keyof_concrete,
            template,
            type_param,
            optional_mod,
            readonly_mod,
            Some(outer_mapper),
            Some(concrete),
            name_type,
        ) else {
            return mapped_type_id;
        };

        let result = self.build_mapped_object_type(mapped_type_id, properties);
        self.mapped_type_cache.insert(cache_key, result);
        result
    }

    /// Instantiate a single element type through a mapped type template.
    /// Used for array and tuple element mapping.
    fn instantiate_mapped_element_type(
        &mut self,
        element_type: TypeId,
        template: Option<TypeId>,
        type_param: TypeId,
        optional_mod: oxc_types::MappedTypeModifier,
        outer_mapper: &TypeMapper,
    ) -> TypeId {
        let Some(template_type) = template else {
            return self.any_type;
        };

        // For arrays/tuples, the "key" is the element type itself.
        // The template is instantiated with T bound to the element type.
        let mapper = outer_mapper.clone().with_mapping(type_param, element_type);
        let result = self.instantiate_type(template_type, &mapper);

        match optional_mod {
            oxc_types::MappedTypeModifier::Add => {
                self.get_or_create_union_type(vec![result, self.undefined_type])
            }
            _ => result,
        }
    }
}
