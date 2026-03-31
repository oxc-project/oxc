use oxc_types::{InterfaceType, ObjectFlags, ObjectType, PropertyInfo, TypeData, TypeFlags, TypeId, build_member_map};
use smallvec::SmallVec;

use crate::Checker;

/// A type mapper: maps type parameters to concrete types.
///
/// Mirrors tsgo's `TypeMapper` (mapper.go). We use a Rust enum
/// instead of Go's interface dispatch (see checker_architecture.md §7).
///
/// All variants are flat (no Box/heap indirection). Clone is a cheap
/// SmallVec stack copy for ≤4 type parameters (the common case).
#[derive(Debug, Clone)]
pub enum TypeMapper {
    /// Single substitution: one type parameter → one type argument.
    /// Covers: `Array<string>`, `Promise<number>`.
    Simple { source: TypeId, target: TypeId },

    /// Multiple substitutions: N type parameters → N type arguments.
    /// Covers: `Map<string, number>`, `Record<K, V>`, and composed mappers.
    Array {
        sources: SmallVec<[TypeId; 4]>,
        targets: SmallVec<[TypeId; 4]>,
    },
}

impl TypeMapper {
    /// Create a mapper from type parameter list and type argument list.
    pub fn from_type_parameters(
        type_params: &[TypeId],
        type_args: &[TypeId],
    ) -> Option<Self> {
        if type_params.is_empty() || type_params.len() != type_args.len() {
            return None;
        }

        if type_params.len() == 1 {
            Some(Self::Simple {
                source: type_params[0],
                target: type_args[0],
            })
        } else {
            Some(Self::Array {
                sources: SmallVec::from_slice(type_params),
                targets: SmallVec::from_slice(type_args),
            })
        }
    }

    /// Map a type through this mapper. Returns `None` if the type
    /// is not a type parameter in this mapper (meaning "leave it alone").
    pub fn map(&self, t: TypeId) -> Option<TypeId> {
        match self {
            Self::Simple { source, target } => {
                if t == *source {
                    Some(*target)
                } else {
                    None
                }
            }
            Self::Array { sources, targets } => sources
                .iter()
                .position(|s| *s == t)
                .map(|i| targets[i]),
        }
    }

    /// Return a new mapper with an additional or overridden mapping.
    /// If `source` already exists in the mapper, its target is replaced.
    /// Otherwise the mapping is appended.
    ///
    /// Clone is cheap (SmallVec stack copy for ≤4 params). No heap allocation.
    pub fn with_mapping(self, source: TypeId, target: TypeId) -> Self {
        match self {
            Self::Simple { source: s, target: t } => {
                if s == source {
                    Self::Simple { source, target }
                } else {
                    Self::Array {
                        sources: smallvec::smallvec![s, source],
                        targets: smallvec::smallvec![t, target],
                    }
                }
            }
            Self::Array { mut sources, mut targets } => {
                if let Some(pos) = sources.iter().position(|&s| s == source) {
                    targets[pos] = target;
                } else {
                    sources.push(source);
                    targets.push(target);
                }
                Self::Array { sources, targets }
            }
        }
    }
}

impl<'a> Checker<'a> {
    /// Resolve a TypeReference to a fully-instantiated type with populated
    /// member_map. Results are cached in `instantiation_cache`.
    ///
    /// For `Array<string>`, this looks up `Array<T>`'s declared members,
    /// substitutes T→string, creates a new InterfaceType in the arena with
    /// the instantiated properties + member_map, and caches the result.
    /// Subsequent calls for the same TypeReference return the cached type.
    ///
    /// Mirrors tsgo's lazy `resolveStructuredTypeMembers` +
    /// `ObjectType.instantiations` cache.
    pub(crate) fn resolve_type_reference(&mut self, type_ref_id: TypeId) -> TypeId {
        if let Some(&cached) = self.instantiation_cache.get(&type_ref_id) {
            return cached;
        }

        let TypeData::TypeReference(tr) = self.type_arena.get_data(type_ref_id) else {
            return type_ref_id;
        };
        let Some(target) = tr.target else { return type_ref_id; };
        let type_args = &tr.resolved_type_arguments;

        // Dispatch based on target type kind
        match self.type_arena.get_data(target) {
            TypeData::Interface(iface) => {
                let Some(mapper) = TypeMapper::from_type_parameters(
                    &iface.all_type_parameters,
                    type_args,
                ) else {
                    self.instantiation_cache.insert(type_ref_id, target);
                    return target;
                };

                // Instantiate properties. Arena references are stable (AppendOnlyVec).
                let instantiated_props: Vec<PropertyInfo> = iface.properties
                    .iter()
                    .map(|p| {
                        PropertyInfo {
                            name: p.name.clone(),
                            type_id: self.instantiate_type(p.type_id, &mapper),
                            optional: p.optional,
                            readonly: p.readonly,
                        }
                    })
                    .collect();
                let member_map = build_member_map(&instantiated_props);

                let resolved_id = self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Interface,
                    TypeData::Interface(InterfaceType {
                        target: Some(target),
                        resolved_type_arguments: type_args.clone(),
                        all_type_parameters: SmallVec::new(),
                        this_type: None,
                        resolved_base_types: SmallVec::new(),
                        properties: instantiated_props,
                        member_map,
                        call_signatures: Vec::new(),
                        construct_signatures: Vec::new(),
                    }),
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
    /// Recursively walks the type structure, substituting type parameters
    /// according to the mapper. Uses flag-based quick reject to skip
    /// types that can't contain type parameters.
    ///
    /// Results are cached in `instantiation_cache` so each unique
    /// (type, mapper-target-list) combination is only computed once.
    pub fn instantiate_type(&mut self, type_id: TypeId, mapper: &TypeMapper) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);

        // Quick reject: types that can never contain type parameters.
        // StructuredOrInstantiable covers Object, Union, Intersection,
        // TypeParameter, IndexedAccess, Conditional, Substitution,
        // Index, TemplateLiteral, StringMapping.
        if !flags.intersects(TypeFlags::StructuredOrInstantiable) {
            return type_id;
        }

        // Type parameter — base case: apply the mapper
        if flags.intersects(TypeFlags::TypeParameter) {
            return mapper.map(type_id).unwrap_or(type_id);
        }

        // Union — instantiate each constituent
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let new_members: Vec<TypeId> = u.types
                    .iter()
                    .map(|&t| self.instantiate_type(t, mapper))
                    .collect();
                return self.get_or_create_union_type(new_members);
            }
        }

        // Intersection — instantiate each constituent
        if flags.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = self.type_arena.get_data(type_id) {
                let new_members: Vec<TypeId> = i.types
                    .iter()
                    .map(|&t| self.instantiate_type(t, mapper))
                    .collect();
                return self.get_or_create_intersection_type(new_members);
            }
        }

        // Object types: TypeReference or Interface/Object with properties
        if flags.intersects(TypeFlags::Object) {
            return self.instantiate_object_type(type_id, mapper);
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

        // Conditional: T extends U ? X : Y — instantiate all parts, then resolve
        if flags.intersects(TypeFlags::Conditional) {
            if let TypeData::Conditional(cond) = self.type_arena.get_data(type_id) {
                let is_distributive = cond.is_distributive;
                let orig_check = cond.check_type;
                let orig_extends = cond.extends_type;
                let orig_true = cond.true_type;
                let orig_false = cond.false_type;

                let new_check = self.instantiate_type(orig_check, mapper);

                // Distribution: if distributive and check resolved to a union,
                // distribute over each member. Each member gets its own mapper
                // where the check type parameter maps to that specific member
                // (overriding the union mapping), so true/false branches see
                // the individual member, not the whole union.
                if is_distributive {
                    let check_flags = self.type_arena.get_flags(new_check);
                    if check_flags.intersects(TypeFlags::Union) {
                        if let TypeData::Union(u) = self.type_arena.get_data(new_check) {
                            let members: Vec<TypeId> = u.types.iter().copied().collect();
                            let results: Vec<TypeId> = members
                                .iter()
                                .map(|&member| {
                                    // Override: check_param → member
                                    let per_member = mapper.clone().with_mapping(orig_check, member);
                                    let ext = self.instantiate_type(orig_extends, &per_member);
                                    let tru = self.instantiate_type(orig_true, &per_member);
                                    let fal = self.instantiate_type(orig_false, &per_member);
                                    self.get_conditional_type(member, ext, tru, fal, false)
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
                    new_check, new_extends, new_true, new_false, is_distributive,
                );
            }
        }

        // TODO: TemplateLiteral, StringMapping

        type_id
    }

    /// Instantiate an object type (interface, type reference, or anonymous object).
    fn instantiate_object_type(&mut self, type_id: TypeId, mapper: &TypeMapper) -> TypeId {
        match self.type_arena.get_data(type_id) {
            TypeData::TypeReference(tr) => {
                // Instantiate the type arguments, keep the same target.
                // e.g., if we have Wrapper<T> where T→string, and a property
                // typed as Array<T>, we instantiate to Array<string>.
                let target = tr.target;
                let new_args: SmallVec<[TypeId; 4]> = tr.resolved_type_arguments
                    .iter()
                    .map(|&t| self.instantiate_type(t, mapper))
                    .collect();

                if new_args[..] == tr.resolved_type_arguments[..] {
                    return type_id; // no change
                }

                self.type_arena.new_type(
                    TypeFlags::Object,
                    oxc_types::ObjectFlags::None,
                    TypeData::TypeReference(oxc_types::TypeReferenceType {
                        target,
                        resolved_type_arguments: new_args,
                    }),
                    None,
                )
            }

            TypeData::Interface(iface) => {
                let properties = &iface.properties;
                self.instantiate_properties(type_id, properties, mapper)
            }

            TypeData::Object(obj) => {
                let properties = &obj.properties;
                self.instantiate_properties(type_id, properties, mapper)
            }

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
                        type_id, concrete, template, type_param,
                        optional_mod, readonly_mod, name_type, mapper,
                    );
                }

                // Non-homomorphic path (Record<K, V>, etc.): resolve constraint
                // to concrete keys, instantiate template per key.
                // Not cached — the result depends on both constraint and mapper,
                // and the non-homomorphic path is infrequent.
                let new_constraint = self.instantiate_type(constraint, mapper);

                let Some(properties) = self.resolve_mapped_type_to_properties(
                    new_constraint, template, type_param,
                    optional_mod, readonly_mod,
                    Some(mapper), None, name_type,
                ) else {
                    return type_id; // can't resolve — return as-is
                };

                self.build_mapped_object_type(type_id, properties)
            }

            _ => type_id,
        }
    }

    /// Instantiate properties of an Object or Interface type with a mapper.
    /// Returns the original type_id if no properties changed.
    fn instantiate_properties(
        &mut self,
        type_id: TypeId,
        properties: &[PropertyInfo],
        mapper: &TypeMapper,
    ) -> TypeId {
        let mut new_props = Vec::new();
        let mut changed = false;
        for p in properties {
            let new_type_id = self.instantiate_type(p.type_id, mapper);
            if new_type_id != p.type_id {
                changed = true;
            }
            new_props.push(PropertyInfo {
                name: p.name.clone(),
                type_id: new_type_id,
                optional: p.optional,
                readonly: p.readonly,
            });
        }

        if !changed {
            return type_id;
        }

        self.type_arena.new_type(
            TypeFlags::Object,
            oxc_types::ObjectFlags::None,
            TypeData::Object(ObjectType {
                target: Some(type_id),
                member_map: build_member_map(&new_props),
                properties: new_props,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
            }),
            None,
        )
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
        if target_flags.intersects(TypeFlags::TypeParameter) {
            Some(idx.target)
        } else {
            None
        }
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
            TypeFlags::String | TypeFlags::Number | TypeFlags::Boolean
            | TypeFlags::BigInt | TypeFlags::ESSymbol
            | TypeFlags::StringLiteral | TypeFlags::NumberLiteral
            | TypeFlags::BooleanLiteral | TypeFlags::BigIntLiteral
            | TypeFlags::Void | TypeFlags::Undefined | TypeFlags::Null
            | TypeFlags::Never
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
                            mapped_type_id, member, template, type_param,
                            optional_mod, readonly_mod, name_type, outer_mapper,
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
                        elem_type, template, type_param, optional_mod, outer_mapper,
                    );
                    let result = self.type_arena.new_type(
                        TypeFlags::Object,
                        ObjectFlags::Reference,
                        TypeData::TypeReference(oxc_types::TypeReferenceType {
                            target: Some(self.array_type),
                            resolved_type_arguments: smallvec::smallvec![mapped_elem],
                        }),
                        None,
                    );
                    self.mapped_type_cache.insert(cache_key, result);
                    return result;
                }
            }

            // Tuples: map each element type, preserve tuple structure.
            // Partial<[string, number]> → [string?, number?]
            if let TypeData::Tuple(tuple) = self.type_arena.get_data(concrete) {
                let element_infos = tuple.element_infos.clone();
                let readonly = tuple.readonly;
                let new_elements: Vec<oxc_types::TupleElementInfo> = element_infos
                    .iter()
                    .map(|info| {
                        let mapped_elem = self.instantiate_mapped_element_type(
                            info.element_type, template, type_param,
                            optional_mod, outer_mapper,
                        );
                        let new_flags = match optional_mod {
                            oxc_types::MappedTypeModifier::Add => {
                                (info.flags - oxc_types::ElementFlags::Required)
                                    | oxc_types::ElementFlags::Optional
                            }
                            oxc_types::MappedTypeModifier::Remove => {
                                (info.flags - oxc_types::ElementFlags::Optional)
                                    | oxc_types::ElementFlags::Required
                            }
                            oxc_types::MappedTypeModifier::None => info.flags,
                        };
                        oxc_types::TupleElementInfo {
                            element_type: mapped_elem,
                            flags: new_flags,
                            label_name: info.label_name.clone(),
                        }
                    })
                    .collect();

                let min_length = new_elements
                    .iter()
                    .filter(|e| e.flags.contains(oxc_types::ElementFlags::Required))
                    .count() as u32;
                let has_rest = new_elements
                    .iter()
                    .any(|e| e.flags.contains(oxc_types::ElementFlags::Rest));
                let fixed_length = if has_rest {
                    new_elements.len() as u32 - 1
                } else {
                    new_elements.len() as u32
                };
                let combined_flags = new_elements
                    .iter()
                    .fold(oxc_types::ElementFlags::empty(), |acc, e| acc | e.flags);
                let type_arguments: SmallVec<[TypeId; 4]> = new_elements
                    .iter()
                    .map(|e| e.element_type)
                    .collect();

                let new_readonly = match readonly_mod {
                    oxc_types::MappedTypeModifier::Add => true,
                    oxc_types::MappedTypeModifier::Remove => false,
                    oxc_types::MappedTypeModifier::None => readonly,
                };

                let result = self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Tuple,
                    TypeData::Tuple(oxc_types::TupleType {
                        target: None,
                        resolved_type_arguments: type_arguments,
                        element_infos: new_elements,
                        min_length,
                        fixed_length,
                        combined_flags,
                        readonly: new_readonly,
                    }),
                    None,
                );
                self.mapped_type_cache.insert(cache_key, result);
                return result;
            }
        }

        // Objects/Interfaces: resolve keyof to get keys, instantiate template
        let keyof_concrete = self.get_index_type(concrete);
        let Some(properties) = self.resolve_mapped_type_to_properties(
            keyof_concrete, template, type_param,
            optional_mod, readonly_mod,
            Some(outer_mapper), Some(concrete), name_type,
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
