use oxc_span::Span;
use oxc_types::{ObjectFlags, StructuredTypeKind, TypeData, TypeFlags, TypeId};
use smallvec::SmallVec;

use crate::Checker;

/// Result of a type relation check that carries richer information than a bool.
///
/// Call sites use the variant to determine which error message to produce:
/// - `Assignable`: no error
/// - `NotAssignable`: standard TS2322/TS2345 "Type X is not assignable to type Y"
/// - `ExcessProperty`: TS2353 "Object literal may only specify known properties..."
#[derive(Debug, Clone)]
pub enum RelationResult {
    Assignable,
    NotAssignable,
    ExcessProperty {
        /// Name of the excess property found on the source.
        property_name: String,
        /// The target type that the literal was being assigned to.
        target_type: TypeId,
    },
}

impl RelationResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, RelationResult::Assignable)
    }
}

/// Relater performs assignability checking with error-reporting context.
///
/// Unlike `Checker::is_type_assignable_to` which returns a bare `bool`,
/// the Relater tracks enough context to produce accurate diagnostics,
/// including excess property errors for fresh object literals.
///
/// Methods take `c: &mut Checker` as a parameter rather than holding
/// a `&mut Checker` reference, avoiding borrow conflicts.
///
/// Mirrors tsgo's `Relater` struct (simplified for Phase 1).
pub struct Relater {
    /// The span to attach errors to (the expression being checked).
    #[allow(dead_code)]
    pub error_span: Span,
}

impl Relater {
    pub fn new(error_span: Span) -> Self {
        Self { error_span }
    }

    /// Check if `source` is assignable to `target`, with excess property checking.
    ///
    /// Performs excess property checking for fresh object literals before
    /// delegating to the standard assignability check.
    pub fn check_type_assignable_to(
        &self,
        c: &mut Checker,
        source: TypeId,
        target: TypeId,
    ) -> RelationResult {
        // If source is a fresh object literal and target is a suitable
        // excess property check target, check for excess properties first.
        if self.should_check_excess_properties(c, source, target) {
            if let Some(excess) = self.has_excess_properties(c, source, target) {
                return excess;
            }
        }

        // Delegate structural/primitive checking to existing implementation.
        if c.is_type_assignable_to(source, target) {
            RelationResult::Assignable
        } else {
            RelationResult::NotAssignable
        }
    }

    /// Determine if we should perform excess property checking.
    ///
    /// Returns true when source is a fresh object literal and target
    /// is a suitable excess property check target.
    fn should_check_excess_properties(&self, c: &Checker, source: TypeId, target: TypeId) -> bool {
        let source_flags = c.type_arena.get_flags(source);
        let source_obj_flags = c.type_arena.get_object_flags(source);

        // Source must be a fresh object literal
        if !source_flags.intersects(TypeFlags::Object) {
            return false;
        }
        if !source_obj_flags.contains(ObjectFlags::FreshLiteral | ObjectFlags::ObjectLiteral) {
            return false;
        }

        // Target must be a suitable check target
        Self::is_excess_property_check_target(c, target)
    }

    /// Check if a target type is suitable for excess property checking.
    ///
    /// Suitable targets: object types (with properties, signatures, or
    /// index signatures), union types (if any constituent is suitable),
    /// intersection types (if all constituents are suitable).
    ///
    /// Mirrors tsgo's `isExcessPropertyCheckTarget`.
    fn is_excess_property_check_target(c: &Checker, target: TypeId) -> bool {
        let flags = c.type_arena.get_flags(target);
        let obj_flags = c.type_arena.get_object_flags(target);

        if flags.intersects(TypeFlags::Object) {
            // Exclude object literal patterns with computed properties
            if obj_flags.intersects(ObjectFlags::ObjectLiteralPatternWithComputedProperties) {
                return false;
            }
            return true;
        }

        // Non-primitive type (`object`)
        if flags.intersects(TypeFlags::NonPrimitive) {
            return true;
        }

        // Union: at least one constituent must be a valid target
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = c.type_arena.get_data(target) {
                return u.types.iter().any(|&m| Self::is_excess_property_check_target(c, m));
            }
        }

        // Intersection: all constituents must be valid targets
        if flags.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = c.type_arena.get_data(target) {
                return i.types.iter().all(|&m| Self::is_excess_property_check_target(c, m));
            }
        }

        false
    }

    /// Check for excess properties on a fresh object literal.
    ///
    /// Iterates each property of the source object literal and checks
    /// whether it is a "known property" of the target type. Returns
    /// `Some(ExcessProperty)` on the first unknown property found,
    /// or `None` if all properties are known.
    fn has_excess_properties(
        &self,
        c: &mut Checker,
        source: TypeId,
        target: TypeId,
    ) -> Option<RelationResult> {
        // Skip if target is an empty object type {} — it accepts anything
        // structurally, so excess property check is meaningless.
        if self.is_empty_object_type(c, target) {
            return None;
        }

        // Arena references have lifetime 'a (AppendOnlyVec gives stable refs),
        // independent of &mut Checker, so we can iterate source_props while
        // calling is_known_property(&mut Checker).
        let source_props = match c.type_arena.get_data(source) {
            TypeData::Structured(s) => &s.properties,
            _ => return None,
        };

        for prop in source_props {
            if !Self::is_known_property(c, target, &prop.name) {
                return Some(RelationResult::ExcessProperty {
                    property_name: prop.name.to_string(),
                    target_type: target,
                });
            }
        }

        None
    }

    /// Check if a target type is an empty object type (no properties,
    /// no signatures, no index signatures).
    fn is_empty_object_type(&self, c: &Checker, target: TypeId) -> bool {
        let flags = c.type_arena.get_flags(target);
        if !flags.intersects(TypeFlags::Object) {
            return false;
        }
        match c.type_arena.get_data(target) {
            TypeData::Structured(s) => {
                s.properties.is_empty()
                    && s.call_signatures.is_empty()
                    && s.construct_signatures.is_empty()
                    && s.string_index_type.is_none()
                    && s.number_index_type.is_none()
            }
            _ => false,
        }
    }

    /// Check if a property name is "known" on a target type.
    ///
    /// A property is known if:
    /// 1. The target has a direct property with that name, OR
    /// 2. The target has a string index signature, OR
    /// 3. A base type has the property (interface inheritance), OR
    /// 4. For union targets: known in ANY constituent, OR
    /// 5. For intersection targets: known in ANY constituent
    ///
    /// Mirrors tsgo's `isKnownProperty`.
    fn is_known_property(c: &mut Checker, target: TypeId, name: &str) -> bool {
        let flags = c.type_arena.get_flags(target);

        // Union: known if known in ANY constituent
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = c.type_arena.get_data(target) {
                let types: SmallVec<[TypeId; 4]> = u.types.iter().copied().collect();
                return types.iter().any(|&m| Self::is_known_property(c, m, name));
            }
        }

        // Intersection: known if known in ANY constituent
        if flags.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = c.type_arena.get_data(target) {
                let types: SmallVec<[TypeId; 4]> = i.types.iter().copied().collect();
                return types.iter().any(|&m| Self::is_known_property(c, m, name));
            }
        }

        // Resolve TypeReference to access properties
        let resolved = if let TypeData::TypeReference(_) = c.type_arena.get_data(target) {
            c.resolve_type_reference(target)
        } else {
            target
        };

        match c.type_arena.get_data(resolved) {
            TypeData::Structured(s) => {
                // Direct property lookup
                if s.member_map.contains_key(name) {
                    return true;
                }
                // String index signature accepts any string key
                if s.string_index_type.is_some() {
                    return true;
                }
                // Walk base types (interface inheritance)
                if let StructuredTypeKind::Interface { resolved_base_types, .. } = &s.kind {
                    let bases: SmallVec<[TypeId; 4]> =
                        resolved_base_types.iter().copied().collect();
                    for base in &bases {
                        if Self::is_known_property(c, *base, name) {
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }
}
