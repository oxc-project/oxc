/// Validate memoized effect dependencies.
///
/// Port of `Validation/ValidateMemoizedEffectDependencies.ts` from the React Compiler.
///
/// Validates that all known effect dependencies are memoized. The algorithm checks two things:
/// - Disallow effect dependencies that should be memoized (have a reactive scope assigned) but
///   where that reactive scope does not exist. This checks for cases where a reactive scope was
///   pruned for some reason, such as spanning a hook.
/// - Disallow effect dependencies whose mutable range encompasses the effect call.
///
/// This latter check corresponds to any values which Forget knows may be mutable and may be
/// mutated after the effect. Note that it's possible Forget may miss memoizing a value for some
/// other reason, but in general this is a bug. The only reason Forget would _choose_ to skip
/// memoization of an effect dependency is because it's mutated later.
///
/// Example:
///
/// ```javascript
/// const object = {}; // mutable range starts here...
///
/// useEffect(() => {
///   console.log('hello');
/// }, [object]); // the dependency array picks up the mutable range of its mutable contents
///
/// mutate(object); // ... mutable range ends here after this mutation
/// ```
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        CallArg, Identifier, InstructionValue, ReactiveFunction, ReactiveInstruction,
        ReactiveScope, ReactiveValue, ScopeId,
        object_shape::{
            BUILT_IN_USE_EFFECT_HOOK_ID, BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID,
            BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID,
        },
        types::{FunctionType, Type},
    },
    reactive_scopes::infer_reactive_scope_variables::is_mutable,
    reactive_scopes::visitors::{ReactiveVisitor, visit_reactive_function},
};

/// Validate that all effect dependencies are memoized.
///
/// # Errors
/// Returns a `CompilerError` if any effect dependency is not memoized.
pub fn validate_memoized_effect_dependencies(func: &ReactiveFunction) -> Result<(), CompilerError> {
    let mut visitor =
        MemoizedEffectDepsVisitor { errors: CompilerError::new(), scopes: FxHashSet::default() };
    visit_reactive_function(func, &mut visitor);
    visitor.errors.into_result()
}

struct MemoizedEffectDepsVisitor {
    errors: CompilerError,
    /// Set of scope IDs whose dependencies are all memoized (i.e. they were
    /// successfully compiled and their transitive deps are also memoized).
    scopes: FxHashSet<ScopeId>,
}

impl ReactiveVisitor for MemoizedEffectDepsVisitor {
    /// Called *after* the scope's inner instructions have been visited, matching
    /// the TS `override visitScope` which calls `this.traverseScope` first and
    /// then records the scope.
    fn exit_scope_block(&mut self, scope: &ReactiveScope) {
        // Record this scope as memoized only if all of its own dependencies are
        // themselves memoized — this enables the transitive check.
        let all_deps_memoized =
            scope.dependencies.iter().all(|dep| !is_unmemoized(&dep.identifier, &self.scopes));
        if all_deps_memoized {
            self.scopes.insert(scope.id);
            for id in &scope.merged {
                self.scopes.insert(*id);
            }
        }
    }

    fn visit_instruction(&mut self, instr: &ReactiveInstruction) {
        let call = match &instr.value {
            ReactiveValue::Instruction(v) => match v.as_ref() {
                InstructionValue::CallExpression(c) => c,
                _ => return,
            },
            _ => return,
        };

        // Must be a call to useEffect / useLayoutEffect / useInsertionEffect.
        if !is_effect_hook(&call.callee.identifier) {
            return;
        }

        // The dependency array is the second argument.
        if call.args.len() < 2 {
            return;
        }
        let deps_arg = match &call.args[1] {
            CallArg::Place(p) => p,
            CallArg::Spread(_) => return,
        };

        // Check if the dep is mutable at the call site OR unmemoized.
        let is_bad = is_mutable(&deps_arg.identifier, instr.id)
            || is_unmemoized(&deps_arg.identifier, &self.scopes);

        if is_bad {
            self.errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::EffectDependencies,
                    "React Compiler has skipped optimizing this component because the effect \
                     dependencies could not be memoized. Unmemoized effect dependencies can \
                     trigger an infinite loop or other unexpected behavior"
                        .to_string(),
                    None,
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(instr.loc),
                    message: Some(
                        "React Compiler has skipped optimizing this component because the effect \
                         dependencies could not be memoized. Unmemoized effect dependencies can \
                         trigger an infinite loop or other unexpected behavior"
                            .to_string(),
                    ),
                }),
            );
        }
    }
}

/// Returns `true` if the identifier has a reactive scope assigned but that
/// scope is not in the set of completed (memoized) scopes.
fn is_unmemoized(identifier: &Identifier, scopes: &FxHashSet<ScopeId>) -> bool {
    identifier.scope.as_deref().is_some_and(|s| !scopes.contains(&s.id))
}

/// Returns `true` if the identifier is typed as a
/// `useEffect` / `useLayoutEffect` / `useInsertionEffect` hook.
fn is_effect_hook(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_EFFECT_HOOK_ID
            || id == BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID
            || id == BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID
    )
}
