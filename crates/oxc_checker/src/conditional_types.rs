//! Resolution of conditional types: `T extends U ? X : Y`.
//!
//! Conditional types test whether a type extends another and select one of
//! two branches. When the check type is a bare type parameter, the conditional
//! is *distributive*: `(A | B) extends U ? X : Y` distributes to
//! `(A extends U ? X : Y) | (B extends U ? X : Y)`.
//!
//! Each conditional type AST node creates a `ConditionalRoot` (shared across
//! instantiations) that stores the original types and any `infer` type
//! parameters from the extends clause.
//!
//! Resolution strategy:
//! - **Concrete**: both check and extends are non-generic → use `is_type_assignable_to`
//!   (with optional inference for `infer` params)
//! - **Deferred**: check type is generic → create a `ConditionalType` in the arena
//! - **Never**: `never extends U ? X : Y` → `never` (empty distribution)

use oxc_types::{ConditionalRoot, ConditionalRootId, ConditionalType, ObjectFlags, TypeData, TypeFlags, TypeId};
use smallvec::SmallVec;

use crate::Checker;
use crate::inference::InferenceContext;
use crate::instantiation::TypeMapper;

impl Checker<'_> {
    /// Create a `ConditionalRoot` and return its ID.
    ///
    /// Called once per `TSConditionalType` AST node. The root is shared
    /// by all instantiations of this conditional type.
    pub(crate) fn create_conditional_root(
        &mut self,
        check_type: TypeId,
        extends_type: TypeId,
        true_type: TypeId,
        false_type: TypeId,
        is_distributive: bool,
        infer_type_parameters: SmallVec<[TypeId; 2]>,
    ) -> ConditionalRootId {
        let id = ConditionalRootId::new(self.conditional_roots.len() as u32);
        self.conditional_roots.push(ConditionalRoot {
            check_type,
            extends_type,
            true_type,
            false_type,
            is_distributive,
            infer_type_parameters,
        });
        id
    }

    /// Resolve or defer a conditional type.
    ///
    /// If both check and extends are concrete, resolves immediately using
    /// assignability (running inference for `infer` params if present).
    /// Otherwise creates a deferred `ConditionalType`.
    ///
    /// Mirrors tsgo's `getConditionalType`.
    pub(crate) fn get_conditional_type(
        &mut self,
        root_id: ConditionalRootId,
        check_type: TypeId,
        extends_type: TypeId,
        true_type: TypeId,
        false_type: TypeId,
    ) -> TypeId {
        let check_flags = self.type_arena.get_flags(check_type);

        // never extends anything → never (empty distribution)
        if check_flags.intersects(TypeFlags::Never) {
            return self.never_type;
        }

        let check_is_generic = self.is_generic_type(check_type);

        // When check is concrete, try to resolve eagerly.
        // If we have infer params, the extends type will contain them as
        // TypeParameters — that's expected. Inference resolves them, so we
        // don't require extends to be non-generic in that case.
        if !check_is_generic {
            // The root may come from a different file (e.g., lib.d.ts utility
            // types like ReturnType<T>). If the root_id is out of range for
            // this checker's conditional_roots, fall through to the deferred
            // path. TODO: fix properly by moving roots to shared storage.
            let Some(root) = self.conditional_roots.get(root_id.index()) else {
                // Fall through to create a deferred ConditionalType below
                return self.type_arena.new_type(
                    TypeFlags::Conditional,
                    ObjectFlags::None,
                    TypeData::Conditional(ConditionalType {
                        root: root_id,
                        check_type,
                        extends_type,
                        true_type,
                        false_type,
                    }),
                    None,
                );
            };
            let infer_params = root.infer_type_parameters.clone();

            if !infer_params.is_empty() {
                // Concrete check + infer params → run inference to resolve
                return self.resolve_conditional_with_infer(
                    root_id, check_type, extends_type, true_type, false_type, &infer_params,
                );
            }

            // No infer params — extends must also be concrete to resolve
            if !self.is_generic_type(extends_type) {
                if self.is_type_assignable_to(check_type, extends_type) {
                    return true_type;
                }
                return false_type;
            }
        }

        // Generic check type (or generic extends without infer params) — defer
        self.type_arena.new_type(
            TypeFlags::Conditional,
            ObjectFlags::None,
            TypeData::Conditional(ConditionalType {
                root: root_id,
                check_type,
                extends_type,
                true_type,
                false_type,
            }),
            None,
        )
    }

    /// Resolve a concrete conditional type that has `infer` type parameters.
    ///
    /// Runs type inference to match check_type against extends_type, collecting
    /// candidates for each infer param. Then tests assignability with the
    /// inferred extends type and returns the appropriate branch with inferred
    /// types substituted.
    ///
    /// If the extends type is still generic after inference (because it contains
    /// outer type parameters that weren't resolved), falls back to creating a
    /// deferred `ConditionalType`.
    fn resolve_conditional_with_infer(
        &mut self,
        root_id: ConditionalRootId,
        check_type: TypeId,
        extends_type: TypeId,
        true_type: TypeId,
        false_type: TypeId,
        infer_params: &[TypeId],
    ) -> TypeId {
        // Create inference context and run inference
        let mut ctx = InferenceContext::new(infer_params);
        self.infer_from_types(&mut ctx, check_type, extends_type);
        let inferred = self.get_inferred_types(&mut ctx);

        // Build mapper: infer params → inferred types
        let Some(mapper) = TypeMapper::from_type_parameters(infer_params, &inferred) else {
            // Shouldn't happen since infer_params is non-empty
            if self.is_type_assignable_to(check_type, extends_type) {
                return true_type;
            }
            return false_type;
        };

        // Instantiate extends type with inferred values to get concrete extends
        let resolved_extends = self.instantiate_type(extends_type, &mapper);

        // If extends is still generic after inference (e.g., it contains outer
        // type params that aren't infer params), we can't resolve eagerly.
        // Fall back to a deferred conditional with the inferred substitutions applied.
        if self.is_generic_type(resolved_extends) {
            let resolved_true = self.instantiate_type(true_type, &mapper);
            return self.type_arena.new_type(
                TypeFlags::Conditional,
                ObjectFlags::None,
                TypeData::Conditional(ConditionalType {
                    root: root_id,
                    check_type,
                    extends_type: resolved_extends,
                    true_type: resolved_true,
                    false_type,
                }),
                None,
            );
        }

        // Test assignability with the inferred extends type
        if self.is_type_assignable_to(check_type, resolved_extends) {
            // True branch: substitute infer params with inferred types
            return self.instantiate_type(true_type, &mapper);
        }

        // False branch: no infer substitution needed (infer params not in scope)
        false_type
    }

    /// Check if a type is generic (contains type parameters or other
    /// instantiable types that prevent eager resolution).
    ///
    /// Uses `TypeFlags::Instantiable` for direct checks and
    /// `ObjectFlags::CouldContainTypeVariables` (propagated at creation
    /// time) for composite types — O(1), no tree walk.
    ///
    /// The flag is propagated for unions, intersections, function types,
    /// constructor types, and tuple types.
    pub(crate) fn is_generic_type(&self, type_id: TypeId) -> bool {
        let flags = self.type_arena.get_flags(type_id);
        if flags.intersects(TypeFlags::Instantiable) {
            return true;
        }
        // Check CouldContainTypeVariables for Object types (function, tuple,
        // structured) and union/intersection types.
        if flags.intersects(TypeFlags::Object | TypeFlags::UnionOrIntersection) {
            return self.type_arena.get_object_flags(type_id)
                .intersects(oxc_types::ObjectFlags::CouldContainTypeVariables);
        }
        false
    }
}
