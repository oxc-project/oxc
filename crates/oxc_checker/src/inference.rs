//! Generic type argument inference.
//!
//! Implements the core algorithm for inferring type arguments at generic
//! function call sites and for `infer` in conditional types. Given a
//! generic signature like `<T>(x: T): T` and an argument of type `string`,
//! infers `T = string`.
//!
//! Mirrors tsgo's `inferTypes` / `inferFromTypes` in `inference.go`.

use oxc_types::{TypeData, TypeFlags, TypeId};
use smallvec::SmallVec;

use crate::Checker;

/// Per-type-parameter inference state.
///
/// Collects candidate types during inference, resolves lazily.
/// Mirrors tsgo's `InferenceInfo`.
pub(crate) struct InferenceInfo {
    /// The type parameter being inferred (e.g., T).
    pub type_parameter: TypeId,
    /// Types inferred in covariant positions.
    pub candidates: SmallVec<[TypeId; 4]>,
    /// Cached resolved type (invalidated when candidates change).
    pub inferred_type: Option<TypeId>,
}

/// Inference context for a generic function call.
///
/// One `InferenceInfo` per type parameter in the generic signature.
/// Mirrors tsgo's `InferenceContext`.
pub(crate) struct InferenceContext {
    pub inferences: SmallVec<[InferenceInfo; 4]>,
}

impl InferenceContext {
    /// Create a new inference context for a set of type parameters.
    pub fn new(type_parameters: &[TypeId]) -> Self {
        Self {
            inferences: type_parameters
                .iter()
                .map(|&tp| InferenceInfo {
                    type_parameter: tp,
                    candidates: SmallVec::new(),
                    inferred_type: None,
                })
                .collect(),
        }
    }

    /// Find the InferenceInfo for a type parameter, if it's being inferred.
    pub fn get_inference_info_mut(&mut self, type_param: TypeId) -> Option<&mut InferenceInfo> {
        self.inferences.iter_mut().find(|info| info.type_parameter == type_param)
    }
}

impl Checker<'_> {
    /// Recursively infer type arguments by structurally matching source against target.
    ///
    /// When target contains a type parameter that is being inferred, the
    /// corresponding source type is added as a candidate. For compound types
    /// (unions, type references, functions, objects, tuples), the algorithm
    /// recurses into constituents.
    pub(crate) fn infer_from_types(
        &mut self,
        ctx: &mut InferenceContext,
        source: TypeId,
        target: TypeId,
    ) {
        // Identity — nothing to infer
        if source == target {
            return;
        }

        let target_flags = self.type_arena.get_flags(target);

        // Quick reject: target can't contain type parameters.
        // Object covers TypeReference, Structured, Function, and Tuple types.
        if !target_flags.intersects(
            TypeFlags::TypeParameter
                | TypeFlags::Object
                | TypeFlags::Union
                | TypeFlags::Intersection,
        ) {
            return;
        }

        // Base case: target is a type parameter → collect candidate
        if target_flags.contains(TypeFlags::TypeParameter) {
            if let Some(info) = ctx.get_inference_info_mut(target) {
                info.candidates.push(source);
                info.inferred_type = None; // invalidate cache
            }
            return;
        }

        // TypeReference: if source is also a TypeReference with same target,
        // infer pairwise from type arguments.
        // (handles Array<T> matched against Array<string> → T = string)
        let target_data = self.type_arena.get_data(target);
        let source_data = self.type_arena.get_data(source);
        if let (TypeData::TypeReference(s_ref), TypeData::TypeReference(t_ref)) =
            (source_data, target_data)
        {
            if s_ref.target == t_ref.target && s_ref.target.is_some() {
                let pairs: SmallVec<[(TypeId, TypeId); 4]> = s_ref
                    .resolved_type_arguments
                    .iter()
                    .zip(t_ref.resolved_type_arguments.iter())
                    .map(|(&s, &t)| (s, t))
                    .collect();
                for (s_arg, t_arg) in pairs {
                    self.infer_from_types(ctx, s_arg, t_arg);
                }
                return;
            }
        }

        // Function types: match signatures (parameters + return type).
        // Handles: T extends (...args: infer P) => infer R ? ...
        if let (TypeData::Function(s_func), TypeData::Function(t_func)) = (source_data, target_data)
        {
            if let (Some(s_sig), Some(t_sig)) =
                (s_func.signatures.first(), t_func.signatures.first())
            {
                // Collect pairs to avoid borrow conflict
                let pairs = Self::collect_signature_inference_pairs(s_sig, t_sig);
                for (s_type, t_type) in pairs {
                    self.infer_from_types(ctx, s_type, t_type);
                }
                return;
            }
        }

        // Structured types with call/construct signatures: infer from signatures
        // and properties. Handles object types that have call signatures
        // (e.g., { (): R }) matched against function-like targets, and
        // property-by-property inference (e.g., { a: infer U }).
        if let (TypeData::Structured(s_struct), TypeData::Structured(t_struct)) =
            (source_data, target_data)
        {
            // Infer from call signatures
            if let (Some(s_sig), Some(t_sig)) =
                (s_struct.call_signatures.first(), t_struct.call_signatures.first())
            {
                let pairs = Self::collect_signature_inference_pairs(s_sig, t_sig);
                for (s_type, t_type) in pairs {
                    self.infer_from_types(ctx, s_type, t_type);
                }
            }

            // Infer from construct signatures
            if let (Some(s_sig), Some(t_sig)) =
                (s_struct.construct_signatures.first(), t_struct.construct_signatures.first())
            {
                let pairs = Self::collect_signature_inference_pairs(s_sig, t_sig);
                for (s_type, t_type) in pairs {
                    self.infer_from_types(ctx, s_type, t_type);
                }
            }

            // Infer from properties: for each target property, find matching
            // source property by name and infer from their types.
            let prop_pairs: SmallVec<[(TypeId, TypeId); 4]> = t_struct
                .properties
                .iter()
                .filter_map(|t_prop| {
                    s_struct.member_map.get(&t_prop.name).map(|&s_type| (s_type, t_prop.type_id))
                })
                .collect();
            for (s_type, t_type) in prop_pairs {
                self.infer_from_types(ctx, s_type, t_type);
            }
            return;
        }

        // Cross-kind: source is Function, target is Structured with call signatures
        // (or vice versa). Handles matching a function against { (): infer R }.
        if let (TypeData::Function(s_func), TypeData::Structured(t_struct)) =
            (source_data, target_data)
        {
            if let (Some(s_sig), Some(t_sig)) =
                (s_func.signatures.first(), t_struct.call_signatures.first())
            {
                let pairs = Self::collect_signature_inference_pairs(s_sig, t_sig);
                for (s_type, t_type) in pairs {
                    self.infer_from_types(ctx, s_type, t_type);
                }
                return;
            }
        }
        if let (TypeData::Structured(s_struct), TypeData::Function(t_func)) =
            (source_data, target_data)
        {
            if let (Some(s_sig), Some(t_sig)) =
                (s_struct.call_signatures.first(), t_func.signatures.first())
            {
                let pairs = Self::collect_signature_inference_pairs(s_sig, t_sig);
                for (s_type, t_type) in pairs {
                    self.infer_from_types(ctx, s_type, t_type);
                }
                return;
            }
        }

        // Tuple types: infer element-by-element from matching positions.
        if let (TypeData::Tuple(s_tuple), TypeData::Tuple(t_tuple)) = (source_data, target_data) {
            let pairs: SmallVec<[(TypeId, TypeId); 4]> = s_tuple
                .element_infos
                .iter()
                .zip(t_tuple.element_infos.iter())
                .map(|(s, t)| (s.element_type, t.element_type))
                .collect();
            for (s_type, t_type) in pairs {
                self.infer_from_types(ctx, s_type, t_type);
            }
            return;
        }

        // Union source: infer from each constituent
        let source_flags = self.type_arena.get_flags(source);
        if source_flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(source) {
                let members: SmallVec<[TypeId; 4]> = u.types.iter().copied().collect();
                for member in members {
                    self.infer_from_types(ctx, member, target);
                }
                return;
            }
        }

        // Union target: infer into each constituent
        if target_flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(target) {
                let members: SmallVec<[TypeId; 4]> = u.types.iter().copied().collect();
                for member in members {
                    self.infer_from_types(ctx, source, member);
                }
                return;
            }
        }
    }

    /// Collect (source_type, target_type) pairs for signature inference.
    ///
    /// Matches parameters pairwise (by position) and pairs the return types.
    /// Handles arity differences: if one signature has fewer parameters, only
    /// the overlapping positions are matched. This is a pure data extraction
    /// (no &mut self) to avoid borrow conflicts with recursive inference.
    fn collect_signature_inference_pairs(
        s_sig: &oxc_types::Signature,
        t_sig: &oxc_types::Signature,
    ) -> SmallVec<[(TypeId, TypeId); 8]> {
        let mut pairs = SmallVec::new();

        // Parameters: match by position (min of both arities).
        // Tsgo matches parameters contravariantly, but for basic inference
        // covariant matching is sufficient — collecting candidates works
        // the same way regardless of variance direction.
        let param_count = s_sig.parameters.len().min(t_sig.parameters.len());
        for i in 0..param_count {
            pairs.push((s_sig.parameters[i].type_id, t_sig.parameters[i].type_id));
        }

        // Return type
        pairs.push((s_sig.return_type, t_sig.return_type));

        pairs
    }

    /// Resolve all inferred types from collected candidates.
    ///
    /// For each type parameter:
    /// - No candidates → use constraint or `unknown`
    /// - One candidate → use it directly
    /// - Multiple candidates → union them
    ///
    /// If the type parameter has an explicit constraint (`infer T extends U`),
    /// the inferred type is narrowed: union members that don't satisfy the
    /// constraint are filtered out. If nothing survives, the constraint itself
    /// is used as the inferred type.
    pub(crate) fn get_inferred_types(
        &mut self,
        ctx: &mut InferenceContext,
    ) -> SmallVec<[TypeId; 4]> {
        let mut results = SmallVec::new();
        for i in 0..ctx.inferences.len() {
            let info = &ctx.inferences[i];
            if let Some(cached) = info.inferred_type {
                results.push(cached);
                continue;
            }

            let type_parameter = info.type_parameter;
            let result = match info.candidates.len() {
                0 => {
                    // No candidates: use constraint or unknown
                    self.get_constraint_of_type_parameter(type_parameter)
                        .unwrap_or(self.unknown_type)
                }
                1 => info.candidates[0],
                _ => {
                    // Multiple candidates: union them
                    let candidates = info.candidates.to_vec();
                    self.get_or_create_union_type(candidates)
                }
            };

            // If the type parameter has a constraint (e.g., `infer T extends string`),
            // narrow the inferred type to only include members that satisfy it.
            let result = self.apply_infer_constraint(type_parameter, result);

            ctx.inferences[i].inferred_type = Some(result);
            results.push(result);
        }
        results
    }

    /// If a type parameter has an explicit constraint, narrow the inferred type
    /// by filtering out union members that don't satisfy it. If the inferred
    /// type is not a union, check it directly. If nothing satisfies the
    /// constraint, use the constraint as the result.
    ///
    /// Mirrors tsgo's constraint enforcement in `getInferredType`.
    fn apply_infer_constraint(&mut self, type_parameter: TypeId, inferred: TypeId) -> TypeId {
        let Some(constraint) = self.get_constraint_of_type_parameter(type_parameter) else {
            return inferred;
        };

        // If the inferred type satisfies the constraint directly, keep it
        if self.is_type_assignable_to(inferred, constraint) {
            return inferred;
        }

        // If inferred is a union, filter to members that satisfy the constraint
        if self.type_arena.get_flags(inferred).intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(inferred) {
                let members: SmallVec<[TypeId; 4]> = u.types.iter().copied().collect();
                let filtered: Vec<TypeId> = members
                    .iter()
                    .filter(|&&m| self.is_type_assignable_to(m, constraint))
                    .copied()
                    .collect();
                if !filtered.is_empty() {
                    return self.get_or_create_union_type(filtered);
                }
            }
        }

        // Nothing satisfies the constraint — use constraint as fallback
        constraint
    }
}
