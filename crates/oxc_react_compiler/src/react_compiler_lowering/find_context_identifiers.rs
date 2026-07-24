//! Rust equivalent of the TypeScript `FindContextIdentifiers` pass.
//!
//! Determines which bindings need StoreContext/LoadContext semantics by
//! walking the function's oxc AST with scope tracking to find variables that
//! cross function boundaries.
//!
//! This is a translation of the original immutable `ContextIdentifierVisitor`,
//! which was driven by the in-tree `AstWalker`/`Visitor`
//! (`crate::react_compiler_ast::visitor`). The original tracked two stacks:
//!
//! * a generic `scope_stack` (the active scope, used to resolve the lexical
//!   binding of a reassignment target by name), and
//! * a `function_stack` of the inner function scopes currently being walked
//!   (empty at the top level of the function being compiled).
//!
//! The shared walk in [`super::pre_pass`] reproduces both stacks exactly: the
//! generic stack is pushed for every scope-creating node the original
//! `AstWalker` pushed for (functions, arrows, blocks, for-loops, switch, catch,
//! class static blocks), while the function stack is pushed only for nested
//! function nodes — mirroring the original `enter_function_*` /
//! `enter_object_method` hooks.
//!
//! Identifiers inside TS type subtrees are deliberately NOT visited here: the
//! original walker walked type positions as opaque `RawNode`s, which never fired
//! `enter_identifier`. This pass was previously driven by `VisitJs`, which skips
//! type-space natively; the shared driver reproduces that by forwarding events
//! to this pass only outside TS type subtrees. Enum and namespace bodies —
//! runtime JS which `VisitJs` WOULD walk — are stubbed out to match the same
//! RawNode treatment. The post-walk supplement loop (driven by
//! `ref_node_id_to_binding`, which DOES include type references) recovers any
//! captured references hiding inside type annotations, matching the TS pass.

use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use oxc_ast::ast::AssignmentTargetMaybeDefault;
use oxc_ast::ast::*;
use oxc_ast::match_assignment_target;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::diagnostics::ErrorCategory;
use crate::scope::ScopeId;
use crate::scope::ScopeResolver;
use crate::scope::SymbolId;

use crate::react_compiler_lowering::identifier_loc_index::IdentifierLocIndex;

#[derive(Default)]
struct BindingInfo {
    reassigned: bool,
    reassigned_by_inner_fn: bool,
    referenced_by_inner_fn: bool,
}

/// State for the `FindContextIdentifiers` pre-pass. The AST walk driving it
/// lives in [`super::pre_pass`], where it shares a single traversal (and a
/// single generated-walk instantiation) with the identifier-loc pre-pass.
pub(super) struct ContextIdentifierVisitor<'a> {
    scope: &'a ScopeResolver<'a, 'a>,
    /// The active scope stack. Initialized with the function-being-compiled's
    /// scope and pushed/popped for every scope-creating node, mirroring the
    /// original `AstWalker`.
    scope_stack: Vec<ScopeId>,
    /// Stack of inner function scopes encountered during traversal.
    /// Empty when at the top level of the function being compiled.
    function_stack: Vec<ScopeId>,
    binding_info: FxHashMap<SymbolId, BindingInfo>,
    error: Option<OxcDiagnostic>,
}

impl<'a> ContextIdentifierVisitor<'a> {
    pub(super) fn new(scope: &'a ScopeResolver<'a, 'a>, function_scope: ScopeId) -> Self {
        Self {
            scope,
            scope_stack: vec![function_scope],
            function_stack: Vec::new(),
            binding_info: FxHashMap::default(),
            error: None,
        }
    }

    pub(super) fn current_scope(&self) -> ScopeId {
        self.scope_stack.last().copied().unwrap_or_else(|| self.scope.program_scope())
    }

    /// Push the generic scope a node creates (its `scope_id` cell), if any.
    /// Returns whether a scope was pushed, so the caller knows whether to pop.
    pub(super) fn enter_scope(&mut self, scope: Option<ScopeId>) -> bool {
        if let Some(scope) = scope {
            self.scope_stack.push(scope);
            true
        } else {
            false
        }
    }

    pub(super) fn exit_scope(&mut self, pushed: bool) {
        if pushed {
            self.scope_stack.pop();
        }
    }

    pub(super) fn push_function_scope(&mut self, scope: Option<ScopeId>) -> bool {
        if let Some(scope) = scope {
            self.function_stack.push(scope);
            true
        } else {
            false
        }
    }

    pub(super) fn pop_function_scope(&mut self, pushed: bool) {
        if pushed {
            self.function_stack.pop();
        }
    }

    pub(super) fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// The captured-reference check for a resolved identifier reference.
    pub(super) fn enter_identifier_reference(&mut self, it: &IdentifierReference<'_>) {
        self.check_captured_symbol(self.scope.resolve_reference(it));
    }

    /// Mirrors the original `enter_identifier`, which fired on pattern
    /// binding identifiers too. Only the declaration site of a captured
    /// binding resolves; everything else is a no-op.
    pub(super) fn enter_binding_identifier(&mut self, it: &BindingIdentifier<'_>) {
        self.check_captured_symbol(self.scope.resolve_binding_identifier(it));
    }

    /// Fire the capture check for the identifier references in a JSX element
    /// name, mirroring `walk_js::walk_jsx_element_name` combined with this
    /// pass's original overrides: `JSXIdentifier`s (lowercase tag names,
    /// member-expression parts) carry no reference, namespaced names were
    /// explicitly skipped, and `this` never resolves — all no-ops. Used for
    /// closing elements, which only this pass's original `VisitJs` walk
    /// visited (the identifier-loc walk has no closing-element handling).
    pub(super) fn check_jsx_element_name(&mut self, name: &JSXElementName<'_>) {
        match name {
            JSXElementName::IdentifierReference(id) => self.enter_identifier_reference(id),
            JSXElementName::MemberExpression(m) => self.check_jsx_member_expression(m),
            JSXElementName::Identifier(_)
            | JSXElementName::ThisExpression(_)
            | JSXElementName::NamespacedName(_) => {}
        }
    }

    fn check_jsx_member_expression(&mut self, expr: &JSXMemberExpression<'_>) {
        match &expr.object {
            JSXMemberExpressionObject::IdentifierReference(id) => {
                self.enter_identifier_reference(id);
            }
            JSXMemberExpressionObject::ThisExpression(_) => {}
            JSXMemberExpressionObject::MemberExpression(inner) => {
                self.check_jsx_member_expression(inner);
            }
        }
    }

    fn check_captured_symbol(&mut self, symbol: Option<SymbolId>) {
        let symbol_id = match symbol {
            Some(id) => id,
            None => return,
        };
        let &fn_scope = match self.function_stack.last() {
            Some(s) => s,
            None => return,
        };
        if is_captured_by_function(self.scope, self.scope.symbol_scope(symbol_id), fn_scope) {
            let info = self.binding_info.entry(symbol_id).or_default();
            info.referenced_by_inner_fn = true;
        }
    }

    pub(super) fn handle_reassignment_identifier(&mut self, name: &str, current_scope: ScopeId) {
        if let Some(symbol_id) = self.scope.find_binding(current_scope, name) {
            let info = self.binding_info.entry(symbol_id).or_default();
            info.reassigned = true;
            if let Some(&fn_scope) = self.function_stack.last()
                && is_captured_by_function(self.scope, self.scope.symbol_scope(symbol_id), fn_scope)
            {
                info.reassigned_by_inner_fn = true;
            }
        }
    }

    /// Record the TS-faithful Todo for an unsupported assignment-target wrapper
    /// node, recording the error once (the first time it is hit).
    fn record_unsupported_lval(&mut self, type_name: &str, span: Span) {
        if self.error.is_some() {
            return;
        }
        self.error = Some(make_unsupported_lval_error(type_name, Some(span)));
    }
}

impl<'a> ContextIdentifierVisitor<'a> {
    /// Recursively walk an assignment target to find all reassignment target
    /// identifiers, mirroring the original `walk_lval_for_reassignment`.
    pub(super) fn walk_assignment_target_for_reassignment(
        &mut self,
        target: &AssignmentTarget<'a>,
        current_scope: ScopeId,
    ) {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.handle_reassignment_identifier(&ident.name, current_scope);
            }
            AssignmentTarget::ArrayAssignmentTarget(pat) => {
                for element in pat.elements.iter().flatten() {
                    self.walk_maybe_default_for_reassignment(element, current_scope);
                }
                if let Some(rest) = &pat.rest {
                    self.walk_assignment_target_for_reassignment(&rest.target, current_scope);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(pat) => {
                for prop in &pat.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(p) => {
                            self.handle_reassignment_identifier(&p.binding.name, current_scope);
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
                            self.walk_maybe_default_for_reassignment(&p.binding, current_scope);
                        }
                    }
                }
                if let Some(rest) = &pat.rest {
                    self.walk_assignment_target_for_reassignment(&rest.target, current_scope);
                }
            }
            // Member expressions are interior mutability, not a variable
            // reassignment — no-op.
            AssignmentTarget::StaticMemberExpression(_)
            | AssignmentTarget::ComputedMemberExpression(_)
            | AssignmentTarget::PrivateFieldExpression(_) => {}
            // Unsupported TS assignment-target wrappers throw a TS-faithful Todo.
            AssignmentTarget::TSAsExpression(node) => {
                self.record_unsupported_lval("TSAsExpression", node.span);
            }
            AssignmentTarget::TSSatisfiesExpression(node) => {
                self.record_unsupported_lval("TSSatisfiesExpression", node.span);
            }
            AssignmentTarget::TSNonNullExpression(node) => {
                self.record_unsupported_lval("TSNonNullExpression", node.span);
            }
            AssignmentTarget::TSTypeAssertion(node) => {
                self.record_unsupported_lval("TSTypeAssertion", node.span);
            }
        }
    }

    fn walk_maybe_default_for_reassignment(
        &mut self,
        target: &AssignmentTargetMaybeDefault<'a>,
        current_scope: ScopeId,
    ) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(node) => {
                self.walk_assignment_target_for_reassignment(&node.binding, current_scope);
            }
            inner @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                self.walk_assignment_target_for_reassignment(
                    inner.to_assignment_target(),
                    current_scope,
                );
            }
        }
    }
}

/// Build the TS-faithful Todo error for an unsupported assignment-target wrapper
/// node, mirroring the TypeScript `FindContextIdentifiers` pass. TS throws
/// immediately (CompilerError.throwTodo in handleAssignment's default case),
/// aborting before BuildHIR ever runs or logs, so this must return Err rather
/// than record-and-continue: otherwise Rust emits HIR debug entries for a
/// function TS never lowered.
fn make_unsupported_lval_error(type_name: &str, span: Option<Span>) -> OxcDiagnostic {
    ErrorCategory::Todo
        .diagnostic(format!(
            "[FindContextIdentifiers] Cannot handle Object destructuring assignment target {type_name}"
        ))
        .with_labels(span)
}

/// Check if a binding declared at `binding_scope` is captured by a function at `function_scope`.
/// Returns true if the binding is declared above the function (in the parent scope or higher).
fn is_captured_by_function(
    scope: &ScopeResolver<'_, '_>,
    binding_scope: ScopeId,
    function_scope: ScopeId,
) -> bool {
    // The binding is captured when it is declared in a strict ancestor scope of
    // the function (its parent scope or higher).
    scope.ancestors(function_scope).skip(1).any(|s| s == binding_scope)
}

/// Find context identifiers for a function: variables that are captured across
/// function boundaries and need StoreContext/LoadContext semantics.
///
/// A binding is a context identifier if:
/// - It is reassigned from inside a nested function (`reassignedByInnerFn`), OR
/// - It is reassigned AND referenced from inside a nested function
///   (`reassigned && referencedByInnerFn`)
///
/// This is the Rust equivalent of the TypeScript `FindContextIdentifiers` pass.
/// The AST walk feeding `visitor` runs in [`super::pre_pass`]; this finishes
/// the analysis from the walk's collected state.
pub(super) fn find_context_identifiers(
    visitor: ContextIdentifierVisitor<'_>,
    identifier_spans: &IdentifierLocIndex,
) -> Result<FxHashSet<SymbolId>, OxcDiagnostic> {
    let ContextIdentifierVisitor { scope, mut binding_info, error, .. } = visitor;

    if let Some(error) = error {
        return Err(error);
    }

    // Supplement the walker-based analysis with resolved-reference data.
    // The AST walker doesn't visit identifiers inside type annotations, but
    // Babel's traverse (used by TS findContextIdentifiers) does — and oxc
    // resolves those type references too. So for any binding that is reassigned
    // but not yet marked referenced-by-inner-fn, check whether any of its
    // resolved references (including ones inside type annotations) sit within a
    // nested function scope.
    //
    // Declaration sites are excluded structurally: they are not references.
    let candidates: Vec<SymbolId> = binding_info
        .iter()
        .filter(|(_, info)| info.reassigned && !info.referenced_by_inner_fn)
        .map(|(&sid, _)| sid)
        .collect();
    for symbol_id in candidates {
        let binding_scope = scope.symbol_scope(symbol_id);
        'refs: for &ref_id in scope.reference_ids(symbol_id) {
            // Only references recorded by the identifier walk participate (the
            // walk covers exactly the compiled function's subtree).
            if identifier_spans.reference(ref_id).is_none() {
                continue;
            }
            // Check whether the reference sits inside a nested function that
            // captures the binding.
            let ref_node = scope.reference_node_id(ref_id);
            for fn_scope in scope.containing_function_scopes(ref_node) {
                if is_captured_by_function(scope, binding_scope, fn_scope) {
                    binding_info.get_mut(&symbol_id).unwrap().referenced_by_inner_fn = true;
                    break 'refs;
                }
            }
        }
    }

    // Collect results
    Ok(binding_info
        .into_iter()
        .filter(|(_, info)| {
            info.reassigned_by_inner_fn || (info.reassigned && info.referenced_by_inner_fn)
        })
        .map(|(id, _)| id)
        .collect())
}
