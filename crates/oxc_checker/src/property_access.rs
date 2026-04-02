use oxc_ast::ast::{ComputedMemberExpression, Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::GetSpan;
use oxc_types::{TypeData, TypeFlags, TypeId};
use smallvec::SmallVec;

use crate::instantiation::TypeMapper;
use crate::Checker;

impl Checker<'_> {
    /// Resolve a static member expression (`obj.prop`) given a pre-resolved object type.
    /// Looks up the property by name and reports TS2339 if not found.
    /// Null checking is the caller's responsibility (see dispatch site convention).
    pub(crate) fn resolve_static_member_type(
        &mut self,
        object_type: TypeId,
        expr: &oxc_ast::ast::StaticMemberExpression<'_>,
    ) -> TypeId {
        let prop_name = expr.property.name.as_str();
        let result = self.get_property_of_type(object_type, prop_name);
        if result.is_none() {
            let type_str = self.type_to_string(object_type);
            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Property '{prop_name}' does not exist on type '{type_str}'."
                ))
                .with_error_code("ts", "2339")
                .with_label(expr.property.span),
            );
        }
        // Freshen literal results so they widen correctly for mutable bindings
        // (e.g., `var x = Colors.Cornflower` should widen from `0` to `number`).
        let t = result.unwrap_or(self.any_type);
        self.get_fresh_type_of_literal(t)
    }

    /// Get the apparent type of a type.
    ///
    /// Maps primitive types to their corresponding global wrapper interfaces
    /// from lib.d.ts (e.g., `string` → `String`, `number` → `Number`).
    /// This enables property access on primitives — `"hello".charAt(0)` works
    /// because `string`'s apparent type is the `String` interface which has
    /// a `charAt` method.
    ///
    /// Mirrors tsgo's `getApparentType` (checker.go).
    pub(crate) fn get_apparent_type(&self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);

        if flags.intersects(TypeFlags::StringLike) {
            if let Some(t) = self.global_string_type {
                return t;
            }
        } else if flags.intersects(TypeFlags::NumberLike) {
            if let Some(t) = self.global_number_type {
                return t;
            }
        } else if flags.intersects(TypeFlags::BigIntLike) {
            if let Some(t) = self.global_bigint_type {
                return t;
            }
        } else if flags.intersects(TypeFlags::BooleanLike) {
            if let Some(t) = self.global_boolean_type {
                return t;
            }
        } else if flags.intersects(TypeFlags::ESSymbolLike) {
            if let Some(t) = self.global_es_symbol_type {
                return t;
            }
        }

        type_id
    }

    /// Look up a property by name on a type. O(1) via HashMap.
    ///
    /// Handles Object, Interface, TypeReference, Union, and primitive types.
    /// Primitive types are resolved to their apparent type (wrapper interface)
    /// before property lookup.
    /// For TypeReferences, resolves to the instantiated type first
    /// (cached via `instantiation_cache`).
    /// Returns `None` if the property is not found or the type
    /// doesn't support property access.
    ///
    /// NOTE: This function always applies apparent type resolution (matching
    /// tsgo's `getPropertyOfTypeEx`). If a raw property lookup without
    /// apparent type mapping is ever needed, split this into a raw version
    /// and an `_ex` variant, or add a `skip_apparent_type` parameter.
    pub(crate) fn get_property_of_type(&mut self, type_id: TypeId, name: &str) -> Option<TypeId> {
        let flags = self.type_arena.get_flags(type_id);

        // any.prop → any
        if flags.intersects(TypeFlags::Any) {
            return Some(self.any_type);
        }

        // Apparent type: map primitives to their wrapper interfaces.
        // e.g., string → String, number → Number.
        let type_id = self.get_apparent_type(type_id);
        let flags = self.type_arena.get_flags(type_id);

        // Union type: look up property on each constituent, union the results
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let mut concrete = Vec::with_capacity(u.types.len());
                for &m in u.types.iter() {
                    match self.get_property_of_type(m, name) {
                        None => return None,
                        Some(t) => concrete.push(t),
                    }
                }
                return Some(self.get_or_create_union_type(concrete));
            }
        }

        // Intersection type: look up property on each constituent, intersect results.
        // Property exists if found in ANY constituent (opposite of unions).
        if flags.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = self.type_arena.get_data(type_id) {
                let mut prop_types = Vec::new();
                for &member in i.types.iter() {
                    if let Some(prop) = self.get_property_of_type(member, name) {
                        if prop != self.any_type
                            || self.type_arena.get_flags(member).intersects(TypeFlags::Any)
                        {
                            prop_types.push(prop);
                        }
                    }
                }
                if prop_types.is_empty() {
                    return None;
                }
                if prop_types.len() == 1 {
                    return Some(prop_types[0]);
                }
                return Some(self.get_or_create_intersection_type(prop_types));
            }
        }

        // TypeReference: lazy per-property resolution (avoids materializing all properties).
        // Check instantiation_cache first — if another path (assignability, keyof) already
        // resolved this TypeReference eagerly, reuse that for O(1) lookup.
        if let TypeData::TypeReference(_) = self.type_arena.get_data(type_id) {
            if let Some(&cached) = self.instantiation_cache.get(&type_id) {
                return self.get_property_of_structured(cached, name);
            }
            return self.get_property_of_type_reference(type_id, name);
        }

        // Direct property lookup on StructuredType (object literals, resolved interfaces, etc.)
        self.get_property_of_structured(type_id, name)
    }

    /// Property lookup on a concrete StructuredType (not a TypeReference).
    /// Checks own properties, walks base types, falls back to index signature.
    fn get_property_of_structured(&mut self, type_id: TypeId, name: &str) -> Option<TypeId> {
        let TypeData::Structured(s) = self.type_arena.get_data(type_id) else {
            return None;
        };
        if let Some(prop) = s.find_property(name) {
            return Some(prop.type_id);
        }
        if let oxc_types::StructuredTypeKind::Interface { resolved_base_types, .. } = &s.kind {
            for base in resolved_base_types.iter() {
                if let Some(prop) = self.get_property_of_type(*base, name) {
                    return Some(prop);
                }
            }
        }
        if let Some(idx_type) = s.string_index_type {
            return Some(idx_type);
        }
        None
    }

    /// Lazy per-property resolution for TypeReferences.
    ///
    /// Instead of materializing all properties via `resolve_type_reference`,
    /// looks up the single requested property on the uninstantiated target
    /// and instantiates only that property's type through the mapper.
    ///
    /// Falls back to `resolve_type_reference` for non-Structured targets.
    ///
    /// Note: this path only handles property lookups. Call/construct signatures
    /// on TypeReferences still require `resolve_type_reference` (accessed through
    /// separate code paths in assignability and inference).
    fn get_property_of_type_reference(
        &mut self,
        type_ref_id: TypeId,
        name: &str,
    ) -> Option<TypeId> {
        let TypeData::TypeReference(tr) = self.type_arena.get_data(type_ref_id) else {
            return None;
        };
        let target = tr.target?;
        let type_args: SmallVec<[TypeId; 4]> = tr.resolved_type_arguments.clone();

        let TypeData::Structured(s) = self.type_arena.get_data(target) else {
            // Non-structured target — fall back to eager resolve
            let resolved = self.resolve_type_reference(type_ref_id);
            return self.get_property_of_type(resolved, name);
        };

        // Extract everything from arena in one pass before calling &mut self methods.
        let prop_type = s.find_property(name).map(|p| p.type_id);
        let string_idx = s.string_index_type;
        let (type_params, base_types) = match &s.kind {
            oxc_types::StructuredTypeKind::Interface { all_type_parameters, resolved_base_types, .. } => {
                (all_type_parameters.clone(), resolved_base_types.clone())
            }
            oxc_types::StructuredTypeKind::Anonymous { .. } => {
                // No type params — direct lookup, no mapper needed
                return prop_type.or(string_idx);
            }
        };

        let mapper = TypeMapper::from_type_parameters(&type_params, &type_args);

        // Own property
        if let Some(pt) = prop_type {
            return Some(match &mapper {
                Some(m) => self.instantiate_type(pt, m),
                None => pt,
            });
        }

        // Base types
        for &base in &base_types {
            let instantiated_base = match &mapper {
                Some(m) => self.instantiate_type(base, m),
                None => base,
            };
            if let Some(prop) = self.get_property_of_type(instantiated_base, name) {
                return Some(prop);
            }
        }

        // Index signature fallback
        string_idx.map(|idx| match &mapper {
            Some(m) => self.instantiate_type(idx, m),
            None => idx,
        })
    }

    /// Resolve a computed member expression (`obj["key"]`, `obj[0]`) given a pre-resolved
    /// object type. For string literal keys, performs a property lookup.
    /// Null checking is the caller's responsibility (see dispatch site convention).
    pub(crate) fn resolve_computed_member_type(
        &mut self,
        object_type: TypeId,
        expr: &ComputedMemberExpression<'_>,
    ) -> TypeId {
        // String literal index → property lookup
        if let Expression::StringLiteral(lit) = &expr.expression {
            let result = self.get_property_of_type(object_type, &lit.value);
            if result.is_none() {
                let type_str = self.type_to_string(object_type);
                let prop_name = &lit.value;
                self.diagnostics.push(
                    OxcDiagnostic::error(format!(
                        "Property '{prop_name}' does not exist on type '{type_str}'."
                    ))
                    .with_error_code("ts", "2339")
                    .with_label(lit.span()),
                );
            }
            return result.unwrap_or(self.any_type);
        }
        // TODO: numeric index on arrays/tuples, keyof, index signatures
        self.any_type
    }
}
