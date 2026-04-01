use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use oxc_types::{TypeData, TypeFlags, TypeId};

use crate::Checker;

impl Checker<'_> {
    /// Get the "awaited type" of a type — the result of `await`ing it.
    ///
    /// If the type is a `Promise<T>`, recursively unwraps to `T`.
    /// If the type is not a promise, returns the type unchanged.
    /// For unions, unwraps each constituent.
    ///
    /// `error_span` is the span of the `await` expression, used to attach
    /// diagnostics for circular self-referencing promises.
    ///
    /// Mirrors tsgo's `getAwaitedType` / `getAwaitedTypeNoAliasEx`.
    /// Currently implements the concrete-type fast path only (no `Awaited<T>`
    /// wrapping for generic types — that requires conditional type support).
    pub(crate) fn get_awaited_type(&mut self, type_id: TypeId, error_span: Span) -> TypeId {
        self.get_awaited_type_worker(type_id, error_span)
    }

    fn get_awaited_type_worker(
        &mut self,
        type_id: TypeId,
        error_span: Span,
    ) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);

        // `any` passes through unchanged
        if flags.intersects(TypeFlags::Any) {
            return type_id;
        }

        // Check cache
        if let Some(&cached) = self.awaited_type_cache.get(&type_id) {
            return cached;
        }

        // Union: map over constituents
        if flags.intersects(TypeFlags::Union) {
            if self.awaited_type_stack.contains(&type_id) {
                self.diagnostics.push(
                    OxcDiagnostic::error(
                        "Type is referenced directly or indirectly in the fulfillment callback of its own 'then' method."
                    )
                    .with_error_code("ts", "1062")
                    .with_label(error_span),
                );
                return self.any_type;
            }
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let members: Vec<TypeId> = u.types.iter().copied().collect();
                self.awaited_type_stack.push(type_id);
                let awaited_members: Vec<TypeId> = members
                    .iter()
                    .map(|&m| self.get_awaited_type_worker(m, error_span))
                    .collect();
                self.awaited_type_stack.pop();
                let result = self.get_or_create_union_type(awaited_members);
                self.awaited_type_cache.insert(type_id, result);
                return result;
            }
        }

        // Try to extract the promised type from a Promise-like type
        if let Some(promised_type) = self.get_promised_type_of_promise(type_id) {
            // Circular reference detection
            if type_id == promised_type || self.awaited_type_stack.contains(&promised_type) {
                // Self-referencing promise — would never resolve at runtime
                self.diagnostics.push(
                    OxcDiagnostic::error(
                        "Type is referenced directly or indirectly in the fulfillment callback of its own 'then' method."
                    )
                    .with_error_code("ts", "1062")
                    .with_label(error_span),
                );
                return self.any_type;
            }

            // Recursively unwrap (Promise<Promise<T>> → T)
            self.awaited_type_stack.push(type_id);
            let awaited = self.get_awaited_type_worker(promised_type, error_span);
            self.awaited_type_stack.pop();
            self.awaited_type_cache.insert(type_id, awaited);
            return awaited;
        }

        // Not a promise — return the type unchanged
        self.awaited_type_cache.insert(type_id, type_id);
        type_id
    }

    /// Extract the promised type `T` from a `Promise<T>` or `PromiseLike<T>`.
    ///
    /// Fast path: if the type is a direct `TypeReference` to the global
    /// `Promise` or `PromiseLike` type, returns the first type argument.
    ///
    /// Returns `None` if the type is not a Promise-like.
    ///
    /// Mirrors tsgo's `getPromisedTypeOfPromise`. Currently only implements
    /// the fast path (direct Promise<T> reference). The slow path (duck-typed
    /// thenable via `.then` method inspection) is not yet implemented.
    fn get_promised_type_of_promise(&self, type_id: TypeId) -> Option<TypeId> {
        let flags = self.type_arena.get_flags(type_id);

        // any is not a promise
        if flags.intersects(TypeFlags::Any) {
            return None;
        }

        // Fast path: check if it's a direct reference to the global Promise type
        if let Some(promise_type) = self.promise_type {
            if self.is_reference_to_type(type_id, promise_type) {
                return self.get_first_type_argument(type_id);
            }
        }

        // Also check PromiseLike<T>
        if let Some(promise_like_type) = self.promise_like_type {
            if self.is_reference_to_type(type_id, promise_like_type) {
                return self.get_first_type_argument(type_id);
            }
        }

        // TODO: Slow path — duck-typed thenable detection via `.then` method.
        // Look up `.then` property, extract call signatures, get the first
        // parameter of the onfulfilled callback. This handles custom thenable
        // types that are structurally compatible with Promise.

        None
    }

    /// Check if a type is a reference (direct or instantiated) to a specific target type.
    ///
    /// Handles both:
    /// - Unresolved `TypeReference { target: Some(target_type), ... }`
    /// - Resolved `StructuredType { kind: Interface { target: Some(target_type), ... }, ... }`
    fn is_reference_to_type(&self, type_id: TypeId, target_type: TypeId) -> bool {
        match self.type_arena.get_data(type_id) {
            TypeData::TypeReference(tr) => tr.target == Some(target_type),
            TypeData::Structured(s) => {
                if let oxc_types::StructuredTypeKind::Interface { target, .. } = &s.kind {
                    *target == Some(target_type)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get the first type argument from a `TypeReference` or resolved interface.
    ///
    /// Returns `None` if the type has no type arguments.
    fn get_first_type_argument(&self, type_id: TypeId) -> Option<TypeId> {
        match self.type_arena.get_data(type_id) {
            TypeData::TypeReference(tr) => tr.resolved_type_arguments.first().copied(),
            TypeData::Structured(s) => {
                if let oxc_types::StructuredTypeKind::Interface {
                    resolved_type_arguments, ..
                } = &s.kind
                {
                    resolved_type_arguments.first().copied()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
