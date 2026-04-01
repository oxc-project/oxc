//! Resolution of mapped types: `{ [P in keyof T]: T[P] }`.
//!
//! A mapped type iterates over a set of keys (usually `keyof T`) and produces
//! a new object type with each key mapped through a template. This is how
//! utility types like `Partial<T>`, `Required<T>`, `Pick<T, K>`, and
//! `Record<K, V>` are defined in lib.d.ts.
//!
//! Resolution happens when a mapped type is instantiated with concrete type
//! arguments (e.g., `Partial<{a: string}>`) and its properties are accessed.
//! The deferred MappedType stores the constraint, template, and modifiers;
//! this module resolves them into a concrete StructuredType.

use oxc_span::CompactStr;
use oxc_types::{
    MappedTypeModifier, ObjectFlags, PropertyInfo, StructuredType, StructuredTypeKind, TypeData,
    TypeFlags, TypeId, sort_properties,
};

use crate::Checker;
use crate::instantiation::TypeMapper;

impl Checker<'_> {
    /// Resolve a mapped type's constraint and template into concrete properties.
    ///
    /// Given a constraint (set of keys) and template, iterates each key and
    /// instantiates the template with the type parameter bound to that key.
    /// Returns `None` if the constraint can't be enumerated (generic).
    ///
    /// Used by `instantiate_object_type` for the `Mapped` variant.
    pub(crate) fn resolve_mapped_type_to_properties(
        &mut self,
        constraint: TypeId,
        template: Option<TypeId>,
        type_param: TypeId,
        optional_mod: MappedTypeModifier,
        readonly_mod: MappedTypeModifier,
        outer_mapper: Option<&TypeMapper>,
        modifiers_type: Option<TypeId>,
        name_type: Option<TypeId>,
    ) -> Option<Vec<PropertyInfo>> {
        let keys = self.resolve_mapped_type_keys(constraint)?;

        let mut properties = Vec::new();
        for key_type_id in &keys {
            let key_name = self.get_string_from_literal_type(*key_type_id)?;

            // Build per-key mapper once — used for both name_type and template.
            let key_mapper = match outer_mapper {
                Some(outer) => outer.clone().with_mapping(type_param, *key_type_id),
                None => TypeMapper::Simple { source: type_param, target: *key_type_id },
            };

            // Apply name remapping (`as` clause) if present.
            // If name_type evaluates to `never`, skip this property (filtering).
            // If it evaluates to a string literal, use that as the new key name.
            let mapped_key_name = if let Some(name_type_id) = name_type {
                let resolved_name = self.instantiate_type(name_type_id, &key_mapper);
                let name_flags = self.type_arena.get_flags(resolved_name);
                if name_flags.intersects(TypeFlags::Never) {
                    continue; // filtered out
                }
                self.get_string_from_literal_type(resolved_name).unwrap_or(key_name.clone())
            } else {
                key_name.clone()
            };

            let prop_type = if let Some(template_type) = template {
                self.instantiate_type(template_type, &key_mapper)
            } else {
                self.any_type
            };

            // Look up source property flags for modifier inheritance
            let source_prop =
                modifiers_type.and_then(|mt| self.get_property_info_of_type(mt, &key_name));
            let source_optional = source_prop.map_or(false, |(_, opt, _)| opt);
            let source_readonly = source_prop.map_or(false, |(_, _, ro)| ro);

            // Apply optional modifier
            let (final_type, is_optional) = match optional_mod {
                MappedTypeModifier::Add => {
                    let t = self.get_or_create_union_type(vec![prop_type, self.undefined_type]);
                    (t, true)
                }
                MappedTypeModifier::Remove => {
                    let t = self.remove_undefined_from_type(prop_type);
                    (t, false)
                }
                MappedTypeModifier::None => {
                    // Preserve source optionality
                    if source_optional {
                        let t = self.get_or_create_union_type(vec![prop_type, self.undefined_type]);
                        (t, true)
                    } else {
                        (prop_type, false)
                    }
                }
            };

            // Apply readonly modifier
            let is_readonly = match readonly_mod {
                MappedTypeModifier::Add => true,
                MappedTypeModifier::Remove => false,
                MappedTypeModifier::None => source_readonly,
            };

            properties.push(PropertyInfo {
                name: mapped_key_name,
                type_id: final_type,
                optional: is_optional,
                readonly: is_readonly,
                decl_order: 0,
            });
        }

        Some(properties)
    }

    /// Look up a property's PropertyInfo from a concrete type by name.
    /// Uses binary search for O(log N) name lookup.
    fn get_property_info_of_type(
        &self,
        type_id: TypeId,
        name: &str,
    ) -> Option<(TypeId, bool, bool)> {
        match self.type_arena.get_data(type_id) {
            TypeData::Structured(s) => {
                s.find_property(name).map(|p| (p.type_id, p.optional, p.readonly))
            }
            _ => None,
        }
    }

    /// Remove `undefined` from a union type. Used by `-?` mapped type modifier.
    /// If the type is not a union, returns it unchanged.
    fn remove_undefined_from_type(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);
        if !flags.intersects(TypeFlags::Union) {
            return type_id;
        }
        if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
            let filtered: Vec<TypeId> = u
                .types
                .iter()
                .copied()
                .filter(|&t| !self.type_arena.get_flags(t).intersects(TypeFlags::Undefined))
                .collect();
            if filtered.len() == u.types.len() {
                return type_id; // no undefined to remove
            }
            return self.get_or_create_union_type(filtered);
        }
        type_id
    }

    /// Build a StructuredType from resolved mapped type properties.
    pub(crate) fn build_mapped_object_type(
        &mut self,
        target: TypeId,
        mut properties: Vec<PropertyInfo>,
    ) -> TypeId {
        sort_properties(&mut properties);
        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous | ObjectFlags::Mapped,
            TypeData::Structured(Box::new(StructuredType {
                properties,
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: Some(target) },
            })),
            None,
        )
    }

    /// Resolve a mapped type constraint to a list of concrete key types.
    ///
    /// For `keyof T` where T is concrete: returns the property name literals.
    /// For a union of string literals: returns the union members.
    /// For generic constraints: returns None (can't enumerate).
    pub(crate) fn resolve_mapped_type_keys(&mut self, constraint: TypeId) -> Option<Vec<TypeId>> {
        let flags = self.type_arena.get_flags(constraint);

        // String literal: single key
        if flags.intersects(TypeFlags::StringLiteral) {
            return Some(vec![constraint]);
        }

        // Union of literals: return all members
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(constraint) {
                let members: Vec<TypeId> = u.types.iter().copied().collect();
                // Check all members are string literals
                let all_literals = members
                    .iter()
                    .all(|&m| self.type_arena.get_flags(m).intersects(TypeFlags::StringLiteral));
                if all_literals {
                    return Some(members);
                }
            }
        }

        // Index type (keyof T): resolve keyof, then extract keys
        if flags.intersects(TypeFlags::Index) {
            if let TypeData::Index(idx) = self.type_arena.get_data(constraint) {
                let target = idx.target;
                let target_flags = self.type_arena.get_flags(target);
                // keyof TypeParameter — can't enumerate keys of a generic type
                if target_flags.intersects(TypeFlags::TypeParameter) {
                    return None;
                }
                let resolved_keyof = self.get_index_type(target);
                // Recurse — the resolved keyof should be a union of literals
                return self.resolve_mapped_type_keys(resolved_keyof);
            }
        }

        // Type parameter or other generic: can't enumerate
        if flags.intersects(TypeFlags::TypeParameter) {
            return None;
        }

        // Intersection or other complex type: try to resolve
        None
    }

    /// Extract the string value from a string literal type.
    pub(crate) fn get_string_from_literal_type(&self, type_id: TypeId) -> Option<CompactStr> {
        if let TypeData::Literal(oxc_types::LiteralType::String(s)) =
            self.type_arena.get_data(type_id)
        {
            Some(s.clone())
        } else {
            None
        }
    }
}
