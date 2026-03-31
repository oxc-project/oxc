use oxc_types::{LiteralType, Signature, TypeData, TypeFlags, TypeId};

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
        if s.intersects(TypeFlags::Any) || t.intersects(TypeFlags::Any) || s.intersects(TypeFlags::Never) {
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

        let result = self.is_type_assignable_to_slow(source, target, s, t);
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
                return u.types
                    .iter()
                    .all(|&member| self.is_type_assignable_to(member, target));
            }
        }

        // Target is union → source assignable to any constituent
        if t.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(target) {
                return u.types
                    .iter()
                    .any(|&member| self.is_type_assignable_to(source, member));
            }
        }

        // Structural assignability for object/interface types:
        // source must have all properties of target with compatible types.
        if t.intersects(TypeFlags::Object) && s.intersects(TypeFlags::Object) {
            return self.is_object_type_assignable_to(source, target);
        }

        false
    }

    /// Check if two literal types have the same value.
    fn are_literals_equal(&self, a: TypeId, b: TypeId) -> bool {
        match (
            self.type_arena.get_data(a),
            self.type_arena.get_data(b),
        ) {
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

        // Check call signature compatibility
        let target_sigs = self.get_call_signatures_of_type(target);
        if !target_sigs.is_empty() {
            let source_sigs = self.get_call_signatures_of_type(source);
            if source_sigs.is_empty() {
                return false;
            }
            // Each target signature must be matched by at least one source signature
            for t_sig in &target_sigs {
                let matched = source_sigs.iter().any(|s_sig| {
                    self.is_signature_assignable_to(s_sig, t_sig)
                });
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

        // Get target properties (iterate ordered Vec)
        let target_props: &[oxc_types::PropertyInfo] = match self.type_arena.get_data(resolved_target) {
            TypeData::Object(obj) => &obj.properties,
            TypeData::Interface(iface) => &iface.properties,
            _ => return true,
        };
        if target_props.is_empty() {
            return true;
        }

        // For each target property, O(1) lookup in source via get_property_of_type
        // (which uses member_map internally). This replaces the old O(P*Q) pattern.
        for prop in target_props {
            let source_prop_type = self.get_property_of_type(source, &prop.name);
            // If source doesn't have this property (returned any_type but source isn't any)
            if source_prop_type == self.any_type {
                let source_flags = self.type_arena.get_flags(source);
                if !source_flags.intersects(TypeFlags::Any) {
                    return false;
                }
            }
            if !self.is_type_assignable_to(source_prop_type, prop.type_id) {
                return false;
            }
        }

        true
    }

    /// Get call signatures from a type (Function, Interface, or Object with call sigs).
    fn get_call_signatures_of_type(&self, type_id: TypeId) -> Vec<Signature> {
        match self.type_arena.get_data(type_id) {
            TypeData::Function(f) => f.signatures.to_vec(),
            TypeData::Interface(i) => i.call_signatures.clone(),
            TypeData::Object(o) => o.call_signatures.clone(),
            _ => Vec::new(),
        }
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

        // Check return type (covariant)
        let s_ret = source.return_type;
        let t_ret = target.return_type;
        if !self.type_arena.get_flags(s_ret).intersects(TypeFlags::Any)
            && !self.type_arena.get_flags(t_ret).intersects(TypeFlags::Any)
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
    fn check_type_reference_variance(
        &mut self,
        source: TypeId,
        target: TypeId,
    ) -> Option<bool> {
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

}
