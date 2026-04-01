//! Generic type argument inference.
//!
//! Implements the core algorithm for inferring type arguments at generic
//! function call sites. Given a generic signature like `<T>(x: T): T` and
//! an argument of type `string`, infers `T = string`.
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
        self.inferences
            .iter_mut()
            .find(|info| info.type_parameter == type_param)
    }
}

impl Checker<'_> {
    /// Recursively infer type arguments by structurally matching source against target.
    ///
    /// When target contains a type parameter that is being inferred, the
    /// corresponding source type is added as a candidate. For compound types
    /// (unions, type references), the algorithm recurses into constituents.
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

        // Quick reject: target can't contain type parameters
        if !target_flags
            .intersects(TypeFlags::TypeParameter | TypeFlags::Object | TypeFlags::Union | TypeFlags::Intersection)
        {
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

    /// Resolve all inferred types from collected candidates.
    ///
    /// For each type parameter:
    /// - No candidates → use constraint or `unknown`
    /// - One candidate → use it directly
    /// - Multiple candidates → union them
    pub(crate) fn get_inferred_types(
        &mut self,
        ctx: &mut InferenceContext,
    ) -> SmallVec<[TypeId; 4]> {
        let mut results = SmallVec::new();
        for info in &mut ctx.inferences {
            if let Some(cached) = info.inferred_type {
                results.push(cached);
                continue;
            }
            let result = match info.candidates.len() {
                0 => {
                    // No candidates: use constraint or unknown
                    self.get_constraint_of_type_parameter(info.type_parameter)
                        .unwrap_or(self.unknown_type)
                }
                1 => info.candidates[0],
                _ => {
                    // Multiple candidates: union them
                    let candidates = info.candidates.to_vec();
                    self.get_or_create_union_type(candidates)
                }
            };
            info.inferred_type = Some(result);
            results.push(result);
        }
        results
    }
}
