//! Resolution of conditional types: `T extends U ? X : Y`.
//!
//! Conditional types test whether a type extends another and select one of
//! two branches. When the check type is a bare type parameter, the conditional
//! is *distributive*: `(A | B) extends U ? X : Y` distributes to
//! `(A extends U ? X : Y) | (B extends U ? X : Y)`.
//!
//! Resolution strategy:
//! - **Concrete**: both check and extends are non-generic → use `is_type_assignable_to`
//! - **Deferred**: check type is generic → create a `ConditionalType` in the arena
//! - **Never**: `never extends U ? X : Y` → `never` (empty distribution)

use oxc_types::{ConditionalType, ObjectFlags, TypeData, TypeFlags, TypeId};

use crate::Checker;

impl Checker<'_> {
    /// Resolve or defer a conditional type.
    ///
    /// If both check and extends are concrete, resolves immediately using
    /// assignability. Otherwise creates a deferred `ConditionalType`.
    ///
    /// Mirrors tsgo's `getConditionalType`.
    pub(crate) fn get_conditional_type(
        &mut self,
        check_type: TypeId,
        extends_type: TypeId,
        true_type: TypeId,
        false_type: TypeId,
        is_distributive: bool,
    ) -> TypeId {
        let check_flags = self.type_arena.get_flags(check_type);

        // never extends anything → never (empty distribution)
        if check_flags.intersects(TypeFlags::Never) {
            return self.never_type;
        }

        // If neither side contains type parameters, resolve immediately
        if !self.is_generic_type(check_type) && !self.is_generic_type(extends_type) {
            if self.is_type_assignable_to(check_type, extends_type) {
                return true_type;
            }
            return false_type;
        }

        // Generic — create deferred conditional type
        self.type_arena.new_type(
            TypeFlags::Conditional,
            ObjectFlags::None,
            TypeData::Conditional(ConditionalType {
                check_type,
                extends_type,
                true_type,
                false_type,
                is_distributive,
            }),
            None,
        )
    }

    /// Check if a type is generic (contains type parameters or other
    /// instantiable types that prevent eager resolution).
    ///
    /// Uses `TypeFlags::Instantiable` for direct checks and
    /// `ObjectFlags::CouldContainTypeVariables` (propagated at union/
    /// intersection creation time) for composite types — O(1), no tree walk.
    pub(crate) fn is_generic_type(&self, type_id: TypeId) -> bool {
        let flags = self.type_arena.get_flags(type_id);
        if flags.intersects(TypeFlags::Instantiable) {
            return true;
        }
        // Unions/intersections propagate CouldContainTypeVariables at creation
        if flags.intersects(TypeFlags::UnionOrIntersection) {
            return self.type_arena.get_object_flags(type_id)
                .intersects(oxc_types::ObjectFlags::CouldContainTypeVariables);
        }
        false
    }
}
