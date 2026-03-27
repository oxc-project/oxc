use oxc_types::{LiteralType, TypeData, TypeFlags, TypeId};

use crate::Checker;

impl Checker<'_> {
    /// Check if `source` type is assignable to `target` type.
    ///
    /// This implements the primitive subset of TypeScript's assignability relation,
    /// matching tsgo's `isSimpleTypeRelatedTo` in `relater.go`.
    pub fn is_type_assignable_to(&self, source: TypeId, target: TypeId) -> bool {
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

        // Source is union → every constituent must be assignable to target.
        // Check this before target-union so that `"a" | 1` vs `string | number`
        // distributes correctly (each source constituent checked against full target).
        if s.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(source) {
                let members = u.types.clone();
                return members
                    .iter()
                    .all(|&member| self.is_type_assignable_to(member, target));
            }
        }

        // Target is union → source assignable to any constituent
        if t.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(target) {
                let members = u.types.clone();
                return members
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
    /// the same name and a compatible type.
    fn is_object_type_assignable_to(&self, source: TypeId, target: TypeId) -> bool {
        let target_props = self.get_properties_of_type(target);
        let source_props = self.get_properties_of_type(source);

        // If target has no properties, any object is assignable
        let Some(target_props) = target_props else {
            return true;
        };
        let Some(source_props) = source_props else {
            // Source has no properties but target does
            return target_props.is_empty();
        };

        // Every target property must exist in source with compatible type
        for (name, &target_prop_type) in target_props {
            match source_props.get(name) {
                Some(&source_prop_type) => {
                    if !self.is_type_assignable_to(source_prop_type, target_prop_type) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    /// Get the properties of a type, if it's an object or interface type.
    fn get_properties_of_type(
        &self,
        type_id: TypeId,
    ) -> Option<&std::collections::HashMap<oxc_span::CompactStr, TypeId>> {
        match self.type_arena.get_data(type_id) {
            TypeData::Object(obj) => Some(&obj.properties),
            TypeData::Interface(iface) => Some(&iface.properties),
            _ => None,
        }
    }
}
