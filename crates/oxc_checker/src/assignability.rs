use oxc_types::{
    LiteralType, ObjectFlags, PropertyInfo, Signature, StructuredType, StructuredTypeKind,
    TypeData, TypeFlags, TypeId, build_member_map,
};
use smallvec::SmallVec;

use crate::Checker;

impl<'a> Checker<'a> {
    /// Check if `source` type is assignable to `target` type.
    ///
    /// This implements the primitive subset of TypeScript's assignability relation,
    /// matching tsgo's `isSimpleTypeRelatedTo` in `relater.go`.
    ///
    /// Results are cached for non-trivial checks (union/intersection/structural)
    /// to avoid exponential blowup on deeply nested types.
    pub fn is_type_assignable_to(&mut self, source: TypeId, target: TypeId) -> bool {
        // Identity
        if source == target {
            return true;
        }

        let s = self.type_arena.get_flags(source);
        let t = self.type_arena.get_flags(target);

        // `any` is assignable to and from everything; `never` is assignable to everything
        if s.intersects(TypeFlags::Any)
            || t.intersects(TypeFlags::Any)
            || s.intersects(TypeFlags::Never)
        {
            return true;
        }

        // Everything is assignable to unknown
        if t.intersects(TypeFlags::Unknown) {
            return true;
        }

        // StringLike (String, StringLiteral, TemplateLiteral, StringMapping) → String
        if s.intersects(TypeFlags::StringLike) && t.intersects(TypeFlags::String) {
            return true;
        }

        // NumberLike (Number, NumberLiteral, Enum) → Number
        if s.intersects(TypeFlags::NumberLike) && t.intersects(TypeFlags::Number) {
            return true;
        }

        // BigIntLike → BigInt
        if s.intersects(TypeFlags::BigIntLike) && t.intersects(TypeFlags::BigInt) {
            return true;
        }

        // BooleanLike → Boolean
        if s.intersects(TypeFlags::BooleanLike) && t.intersects(TypeFlags::Boolean) {
            return true;
        }

        // ESSymbolLike → ESSymbol
        if s.intersects(TypeFlags::ESSymbolLike) && t.intersects(TypeFlags::ESSymbol) {
            return true;
        }

        // Undefined → Undefined | Void
        if s.intersects(TypeFlags::Undefined)
            && t.intersects(TypeFlags::Undefined | TypeFlags::Void)
        {
            return true;
        }

        // Null → Null
        if s.intersects(TypeFlags::Null) && t.intersects(TypeFlags::Null) {
            return true;
        }

        // Void → Void
        if s.intersects(TypeFlags::Void) && t.intersects(TypeFlags::Void) {
            return true;
        }

        // Same literal value check
        if s.intersects(TypeFlags::Literal) && t.intersects(TypeFlags::Literal) {
            return self.are_literals_equal(source, target);
        }

        // For non-trivial checks (unions, intersections, structural), use cache
        let cache_key = (source.index() as u64) << 32 | target.index() as u64;
        if let Some(&cached) = self.assignability_cache.get(&cache_key) {
            return cached;
        }

        // Cycle detection: if this pair is already being checked (e.g., circular
        // TypeParameter constraints T extends U, U extends T), return false to
        // break the cycle. Mirrors the `resolving_symbols` pattern.
        if !self.in_flight_assignability.insert(cache_key) {
            return false;
        }

        let result = self.is_type_assignable_to_slow(source, target, s, t);
        self.in_flight_assignability.remove(&cache_key);
        self.assignability_cache.insert(cache_key, result);
        result
    }

    /// Non-trivial assignability checks: unions, intersections, structural.
    /// Results are cached by the caller.
    fn is_type_assignable_to_slow(
        &mut self,
        source: TypeId,
        target: TypeId,
        s: TypeFlags,
        t: TypeFlags,
    ) -> bool {
        // Source is union → every constituent must be assignable to target.
        // Check this before target-union so that `"a" | 1` vs `string | number`
        // distributes correctly (each source constituent checked against full target).
        if s.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(source) {
                return u.types.iter().all(|&member| self.is_type_assignable_to(member, target));
            }
        }

        // Target is union → source assignable to any constituent
        if t.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(target) {
                return u.types.iter().any(|&member| self.is_type_assignable_to(source, member));
            }
        }

        // Target is intersection → source must be assignable to ALL constituents
        if t.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = self.type_arena.get_data(target) {
                let members: SmallVec<[TypeId; 4]> = i.types.clone();
                return members.iter().all(|&m| self.is_type_assignable_to(source, m));
            }
        }

        // Source is intersection → cheap check: if ANY constituent is assignable
        // to target, succeed without structural comparison.
        if s.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = self.type_arena.get_data(source) {
                let members: SmallVec<[TypeId; 4]> = i.types.clone();
                if members.iter().any(|&m| self.is_type_assignable_to(m, target)) {
                    return true;
                }
            }
        }

        // Structural assignability for object/interface types:
        // source must have all properties of target with compatible types.
        // Also handles source-intersection via structural fallback when
        // no single constituent matched above.
        if t.intersects(TypeFlags::Object) {
            if s.intersects(TypeFlags::Object) {
                return self.is_object_type_assignable_to(source, target);
            }
            if s.intersects(TypeFlags::Intersection) {
                let resolved = self.resolve_intersection_properties(source);
                return self.is_object_type_assignable_to(resolved, target);
            }
        }

        // Source is TypeParameter → check if its constraint is assignable to target.
        // e.g., `T extends number` should be assignable to `number | bigint`.
        // Unconstrained type parameters have no constraint, so they fall through to false.
        if s.intersects(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(source) {
                return self.is_type_assignable_to(constraint, target);
            }
        }

        false
    }

    /// Check if `source` is assignable to the type category described by `kind` flags.
    /// Fast path: check flags directly. Slow path: use full assignability.
    /// Mirrors tsgo's `isTypeAssignableToKind`.
    pub fn is_type_assignable_to_kind(&mut self, source: TypeId, kind: TypeFlags) -> bool {
        self.is_type_assignable_to_kind_ex(source, kind, false)
    }

    /// Strict variant that rejects any/unknown/void/undefined/null before the slow path.
    /// Used by the `+` operator where `any + any` should return `any` (handled separately),
    /// not match the number/bigint/string branches.
    /// Mirrors tsgo's `isTypeAssignableToKindEx`.
    pub fn is_type_assignable_to_kind_ex(
        &mut self,
        source: TypeId,
        kind: TypeFlags,
        strict: bool,
    ) -> bool {
        let flags = self.type_arena.get_flags(source);
        if flags.intersects(kind) {
            return true;
        }
        if strict
            && flags.intersects(
                TypeFlags::Any
                    | TypeFlags::Unknown
                    | TypeFlags::Void
                    | TypeFlags::Undefined
                    | TypeFlags::Null,
            )
        {
            return false;
        }
        (kind.intersects(TypeFlags::NumberLike)
            && self.is_type_assignable_to(source, self.number_type))
            || (kind.intersects(TypeFlags::BigIntLike)
                && self.is_type_assignable_to(source, self.bigint_type))
            || (kind.intersects(TypeFlags::StringLike)
                && self.is_type_assignable_to(source, self.string_type))
            || (kind.intersects(TypeFlags::BooleanLike)
                && self.is_type_assignable_to(source, self.boolean_type))
            || (kind.intersects(TypeFlags::ESSymbol)
                && self.is_type_assignable_to(source, self.es_symbol_type))
            || (kind.intersects(TypeFlags::Void)
                && self.is_type_assignable_to(source, self.void_type))
            || (kind.intersects(TypeFlags::Never)
                && self.is_type_assignable_to(source, self.never_type))
            || (kind.intersects(TypeFlags::Null)
                && self.is_type_assignable_to(source, self.null_type))
            || (kind.intersects(TypeFlags::Undefined)
                && self.is_type_assignable_to(source, self.undefined_type))
            || (kind.intersects(TypeFlags::NonPrimitive)
                && self.is_type_assignable_to(source, self.non_primitive_type))
    }

    /// Check if two literal types have the same value.
    fn are_literals_equal(&self, a: TypeId, b: TypeId) -> bool {
        match (self.type_arena.get_data(a), self.type_arena.get_data(b)) {
            (TypeData::Literal(la), TypeData::Literal(lb)) => match (la, lb) {
                (LiteralType::String(a), LiteralType::String(b)) => a == b,
                (LiteralType::String(_), _) => false,
                (LiteralType::Number(a), LiteralType::Number(b)) => a.to_bits() == b.to_bits(),
                (LiteralType::Number(_), _) => false,
                (LiteralType::BigInt(a), LiteralType::BigInt(b)) => a == b,
                (LiteralType::BigInt(_), _) => false,
                (LiteralType::Boolean(a), LiteralType::Boolean(b)) => a == b,
                (LiteralType::Boolean(_), _) => false,
            },
            _ => false,
        }
    }

    /// Structural assignability for object/interface types.
    ///
    /// For each property in the target, the source must have a property with
    /// the same name and a compatible type. If the target has call signatures,
    /// the source must also have compatible call signatures.
    fn is_object_type_assignable_to(&mut self, source: TypeId, target: TypeId) -> bool {
        // Variance shortcut: if both are TypeReferences with the same target,
        // compare type arguments directly (covariant for v1).
        if let Some(result) = self.check_type_reference_variance(source, target) {
            return result;
        }

        // Check call signature compatibility.
        // Access arena directly (not through &self helper) so the returned
        // references have lifetime 'a and don't conflict with &mut self.
        let target_sigs: &[Signature] = match self.type_arena.get_data(target) {
            TypeData::Function(f) => &f.signatures,
            TypeData::Structured(s) => &s.call_signatures,
            _ => &[],
        };
        if !target_sigs.is_empty() {
            let source_sigs: &[Signature] = match self.type_arena.get_data(source) {
                TypeData::Function(f) => &f.signatures,
                TypeData::Structured(s) => &s.call_signatures,
                _ => &[],
            };
            if source_sigs.is_empty() {
                return false;
            }
            // Each target signature must be matched by at least one source signature.
            for t_sig in target_sigs {
                let matched =
                    source_sigs.iter().any(|s_sig| self.is_signature_assignable_to(s_sig, t_sig));
                if !matched {
                    return false;
                }
            }
        }

        // Resolve TypeReferences so we can access their member_map
        let resolved_target = if let TypeData::TypeReference(_) = self.type_arena.get_data(target) {
            self.resolve_type_reference(target)
        } else {
            target
        };

        // Get target properties (iterate ordered Vec).
        // Direct arena access: reference has lifetime 'a, independent of &mut self.
        let target_props: &[oxc_types::PropertyInfo] =
            match self.type_arena.get_data(resolved_target) {
                TypeData::Structured(s) => &s.properties,
                _ => return true,
            };
        if target_props.is_empty() {
            return true;
        }

        // For each target property, O(1) lookup in source via get_property_of_type
        // (which uses member_map internally). This replaces the old O(P*Q) pattern.
        for prop in target_props {
            match self.get_property_of_type(source, &prop.name) {
                None => {
                    // Optional target properties don't need to exist on source
                    if !prop.optional {
                        return false;
                    }
                }
                Some(source_prop_type) => {
                    if !self.is_type_assignable_to(source_prop_type, prop.type_id) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if a source signature is assignable to a target signature.
    ///
    /// Parameters are checked contravariantly, return type covariantly.
    /// Source may have fewer required parameters than target (TypeScript
    /// allows this — callback compatibility).
    fn is_signature_assignable_to(&mut self, source: &Signature, target: &Signature) -> bool {
        // Source can have fewer parameters (callback compatibility)
        // but not more required parameters than target has total
        let target_max = target.parameters.len();
        if source.min_argument_count as usize > target_max {
            return false;
        }

        // Check parameter types (contravariant: target param assignable to source param)
        let check_count = source.parameters.len().min(target.parameters.len());
        for i in 0..check_count {
            let s_param = &source.parameters[i];
            let t_param = &target.parameters[i];
            // Skip if either is any
            if self.type_arena.get_flags(s_param.type_id).intersects(TypeFlags::Any)
                || self.type_arena.get_flags(t_param.type_id).intersects(TypeFlags::Any)
            {
                continue;
            }
            // Contravariant: target param must be assignable to source param
            // (but TypeScript uses bivariant checking for functions by default)
            // For simplicity, we use covariant checking like tsc's default
            if !self.is_type_assignable_to(s_param.type_id, t_param.type_id)
                && !self.is_type_assignable_to(t_param.type_id, s_param.type_id)
            {
                return false;
            }
        }

        // Check return type (covariant).
        // Void target return means "don't care about return value" — anything is assignable.
        let s_ret = source.return_type;
        let t_ret = target.return_type;
        let t_ret_flags = self.type_arena.get_flags(t_ret);
        if !t_ret_flags.intersects(TypeFlags::Void | TypeFlags::Any)
            && !self.type_arena.get_flags(s_ret).intersects(TypeFlags::Any)
        {
            if !self.is_type_assignable_to(s_ret, t_ret) {
                return false;
            }
        }

        true
    }

    /// Variance-based shortcut for TypeReference assignability.
    ///
    /// If both types are TypeReferences with the same target (e.g., both
    /// are `Array<...>`), we can compare type arguments directly instead
    /// of doing a full structural comparison.
    ///
    /// Returns `Some(bool)` if the shortcut applies, `None` if it doesn't
    /// (fall through to structural comparison).
    ///
    /// For v1, all type parameters are treated as covariant. This is correct
    /// for most readonly interfaces (Array, Promise, etc.) but overly
    /// permissive for mutable containers. Proper variance computation
    /// (from usage analysis) is a future improvement.
    fn check_type_reference_variance(&mut self, source: TypeId, target: TypeId) -> Option<bool> {
        let (source_target, source_args) = match self.type_arena.get_data(source) {
            TypeData::TypeReference(tr) => (tr.target?, &tr.resolved_type_arguments),
            _ => return None,
        };
        let (target_target, target_args) = match self.type_arena.get_data(target) {
            TypeData::TypeReference(tr) => (tr.target?, &tr.resolved_type_arguments),
            _ => return None,
        };

        // Only applies when both reference the same generic type
        if source_target != target_target {
            return None;
        }

        // Compare type arguments covariantly (v1 default)
        if source_args.len() != target_args.len() {
            return Some(false);
        }

        for (&s_arg, &t_arg) in source_args.iter().zip(target_args.iter()) {
            if !self.is_type_assignable_to(s_arg, t_arg) {
                return Some(false);
            }
        }

        Some(true)
    }

    /// Resolve an intersection type into a StructuredType with merged properties.
    ///
    /// For each property name found in ANY constituent, the resulting type is the
    /// intersection of that property's type across all constituents that have it.
    /// Results are cached via `intersection_resolved_cache`.
    fn resolve_intersection_properties(&mut self, intersection_id: TypeId) -> TypeId {
        if let Some(&cached) = self.intersection_resolved_cache.get(&intersection_id) {
            return cached;
        }

        let constituents: SmallVec<[TypeId; 4]> = match self.type_arena.get_data(intersection_id) {
            TypeData::Intersection(i) => i.types.clone(),
            _ => return intersection_id,
        };

        // Collect all unique property names from all constituents.
        let mut all_names: Vec<oxc_span::CompactStr> = Vec::new();
        let mut seen_names = rustc_hash::FxHashSet::default();
        for &constituent in &constituents {
            // Resolve TypeReference to access properties
            let resolved = if let TypeData::TypeReference(_) = self.type_arena.get_data(constituent)
            {
                self.resolve_type_reference(constituent)
            } else {
                constituent
            };
            if let TypeData::Structured(s) = self.type_arena.get_data(resolved) {
                for prop in &s.properties {
                    if seen_names.insert(prop.name.clone()) {
                        all_names.push(prop.name.clone());
                    }
                }
            }
        }

        // For each property, collect types from constituents and intersect.
        let mut properties = Vec::with_capacity(all_names.len());
        for name in &all_names {
            if let Some(prop_type) = self.get_property_of_type(intersection_id, name) {
                properties.push(PropertyInfo::new(name.clone(), prop_type));
            }
        }

        // Merge signatures and index types from all constituents.
        let mut call_signatures = Vec::new();
        let mut construct_signatures = Vec::new();
        let mut string_index_type: Option<TypeId> = None;
        let mut number_index_type: Option<TypeId> = None;
        for &constituent in &constituents {
            let resolved = if let TypeData::TypeReference(_) = self.type_arena.get_data(constituent)
            {
                self.resolve_type_reference(constituent)
            } else {
                constituent
            };
            if let TypeData::Structured(s) = self.type_arena.get_data(resolved) {
                call_signatures.extend(s.call_signatures.iter().cloned());
                construct_signatures.extend(s.construct_signatures.iter().cloned());
                if let Some(idx) = s.string_index_type {
                    string_index_type = Some(match string_index_type {
                        Some(existing) => self.get_or_create_intersection_type(vec![existing, idx]),
                        None => idx,
                    });
                }
                if let Some(idx) = s.number_index_type {
                    number_index_type = Some(match number_index_type {
                        Some(existing) => self.get_or_create_intersection_type(vec![existing, idx]),
                        None => idx,
                    });
                }
            }
        }

        let member_map = build_member_map(&properties);
        let resolved_id = self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous,
            TypeData::Structured(StructuredType {
                member_map,
                properties,
                string_index_type,
                number_index_type,
                call_signatures,
                construct_signatures,
                kind: StructuredTypeKind::Anonymous { target: Some(intersection_id) },
            }),
            None,
        );
        self.intersection_resolved_cache.insert(intersection_id, resolved_id);
        resolved_id
    }
}
