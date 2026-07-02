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
//! The `Visit` impl below reproduces both stacks exactly: the generic stack is
//! pushed for every scope-creating node the original `AstWalker` pushed for
//! (functions, arrows, blocks, for-loops, switch, catch, class static blocks),
//! while the function stack is pushed only for nested function nodes — mirroring
//! the original `enter_function_*` / `enter_object_method` hooks.
//!
//! Identifiers inside TS type subtrees are deliberately NOT visited here: the
//! original walker walked type positions as opaque `RawNode`s, which never fired
//! `enter_identifier`. The post-walk supplement loop (driven by
//! `ref_node_id_to_binding`, which DOES include type references) recovers any
//! captured references hiding inside type annotations, matching the TS pass.

use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use oxc_ast::ast as oxc;
use oxc_ast::ast::AssignmentTargetMaybeDefault;
use oxc_ast::match_assignment_target;
use oxc_ast_visit::Visit;
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use crate::react_compiler_diagnostics::CompilerError;
use crate::react_compiler_diagnostics::CompilerErrorDetail;
use crate::react_compiler_diagnostics::ErrorCategory;
use crate::react_compiler_diagnostics::SourceLocation;
use crate::react_compiler_hir::environment::Environment;
use crate::scope::BindingId;
use crate::scope::ScopeId;
use crate::scope::ScopeInfo;
use crate::scope::ScopeKind;

use crate::react_compiler_lowering::FunctionNode;
use crate::react_compiler_lowering::source_loc::LineOffsets;

#[derive(Default)]
struct BindingInfo {
    reassigned: bool,
    reassigned_by_inner_fn: bool,
    referenced_by_inner_fn: bool,
}

struct ContextIdentifierVisitor<'a, 'b> {
    scope_info: &'a ScopeInfo,
    line_offsets: &'a LineOffsets,
    env: &'a mut Environment<'b>,
    /// The active scope stack. Initialized with the function-being-compiled's
    /// scope and pushed/popped for every scope-creating node, mirroring the
    /// original `AstWalker`.
    scope_stack: Vec<ScopeId>,
    /// Stack of inner function scopes encountered during traversal.
    /// Empty when at the top level of the function being compiled.
    function_stack: Vec<ScopeId>,
    binding_info: FxHashMap<BindingId, BindingInfo>,
    error: Option<CompilerError>,
}

impl<'a, 'b> ContextIdentifierVisitor<'a, 'b> {
    fn current_scope(&self) -> ScopeId {
        self.scope_stack.last().copied().unwrap_or(self.scope_info.program_scope)
    }

    /// Push the generic scope for a node, if it creates one. Returns whether a
    /// scope was pushed, so the caller knows whether to pop after walking.
    fn enter_scope(&mut self, span: Span) -> bool {
        if let Some(scope) = self.scope_info.resolve_scope_for_node(Some(span.start)) {
            self.scope_stack.push(scope);
            true
        } else {
            false
        }
    }

    fn exit_scope(&mut self, pushed: bool) {
        if pushed {
            self.scope_stack.pop();
        }
    }

    fn push_function_scope(&mut self, span: Span) -> bool {
        if let Some(scope) = self.scope_info.resolve_scope_for_node(Some(span.start)) {
            self.function_stack.push(scope);
            true
        } else {
            false
        }
    }

    fn pop_function_scope(&mut self, pushed: bool) {
        if pushed {
            self.function_stack.pop();
        }
    }

    fn check_captured_reference(&mut self, span: Span) {
        let binding_id = match self.scope_info.resolve_reference_id_for_node(Some(span.start)) {
            Some(id) => id,
            None => return,
        };
        let &fn_scope = match self.function_stack.last() {
            Some(s) => s,
            None => return,
        };
        let binding = &self.scope_info.bindings[binding_id.0 as usize];
        if is_captured_by_function(self.scope_info, binding.scope, fn_scope) {
            let info = self.binding_info.entry(binding_id).or_default();
            info.referenced_by_inner_fn = true;
        }
    }

    fn handle_reassignment_identifier(&mut self, name: &str, current_scope: ScopeId) {
        if let Some(binding_id) = self.scope_info.get_binding(current_scope, name) {
            let info = self.binding_info.entry(binding_id).or_default();
            info.reassigned = true;
            if let Some(&fn_scope) = self.function_stack.last() {
                let binding = &self.scope_info.bindings[binding_id.0 as usize];
                if is_captured_by_function(self.scope_info, binding.scope, fn_scope) {
                    info.reassigned_by_inner_fn = true;
                }
            }
        }
    }

    /// Record the TS-faithful Todo for an unsupported assignment-target wrapper
    /// node, recording the error once (the first time it is hit).
    fn record_unsupported_lval(&mut self, type_name: &str, span: Span) {
        if self.error.is_some() {
            return;
        }
        let loc = self.line_offsets.source_location(span);
        self.error = Some(make_unsupported_lval_error(self.env, type_name, Some(loc)));
    }
}

impl<'a, 'b> Visit<'a> for ContextIdentifierVisitor<'a, 'b> {
    // ---- function scopes (push BOTH the generic scope and the function stack) ----

    fn visit_function(&mut self, it: &oxc::Function<'a>, _flags: ScopeFlags) {
        let scope_pushed = self.enter_scope(it.span);
        let fn_pushed = self.push_function_scope(it.span);
        // The original Babel walker never visited the function NAME identifier
        // (`it.id`); it only walked the type-bearing parts (as opaque RawNodes),
        // then params, then body. oxc's `walk_function` DOES visit `it.id` via
        // `visit_binding_identifier`, which — with the inner function already on
        // `function_stack` — would spuriously mark a hoisted nested-function name
        // as referenced_by_inner_fn. Walk the parts manually, skipping `it.id`.
        // (Type parameters / return type are no-ops via the `visit_ts_*`
        // overrides, mirroring the original RawNode walk.)
        if let Some(this_param) = &it.this_param {
            self.visit_ts_this_parameter(this_param);
        }
        self.visit_formal_parameters(&it.params);
        if let Some(body) = &it.body {
            self.visit_function_body(body);
        }
        self.pop_function_scope(fn_pushed);
        self.exit_scope(scope_pushed);
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc::ArrowFunctionExpression<'a>) {
        let scope_pushed = self.enter_scope(it.span);
        let fn_pushed = self.push_function_scope(it.span);
        oxc_ast_visit::walk::walk_arrow_function_expression(self, it);
        self.pop_function_scope(fn_pushed);
        self.exit_scope(scope_pushed);
    }

    // ---- non-function scope-creating nodes (push only the generic scope) ----

    fn visit_block_statement(&mut self, it: &oxc::BlockStatement<'a>) {
        let pushed = self.enter_scope(it.span);
        oxc_ast_visit::walk::walk_block_statement(self, it);
        self.exit_scope(pushed);
    }

    fn visit_for_statement(&mut self, it: &oxc::ForStatement<'a>) {
        let pushed = self.enter_scope(it.span);
        oxc_ast_visit::walk::walk_for_statement(self, it);
        self.exit_scope(pushed);
    }

    fn visit_for_in_statement(&mut self, it: &oxc::ForInStatement<'a>) {
        let pushed = self.enter_scope(it.span);
        oxc_ast_visit::walk::walk_for_in_statement(self, it);
        self.exit_scope(pushed);
    }

    fn visit_for_of_statement(&mut self, it: &oxc::ForOfStatement<'a>) {
        let pushed = self.enter_scope(it.span);
        oxc_ast_visit::walk::walk_for_of_statement(self, it);
        self.exit_scope(pushed);
    }

    fn visit_switch_statement(&mut self, it: &oxc::SwitchStatement<'a>) {
        let pushed = self.enter_scope(it.span);
        oxc_ast_visit::walk::walk_switch_statement(self, it);
        self.exit_scope(pushed);
    }

    fn visit_catch_clause(&mut self, it: &oxc::CatchClause<'a>) {
        let pushed = self.enter_scope(it.span);
        oxc_ast_visit::walk::walk_catch_clause(self, it);
        self.exit_scope(pushed);
    }

    fn visit_static_block(&mut self, it: &oxc::StaticBlock<'a>) {
        let pushed = self.enter_scope(it.span);
        oxc_ast_visit::walk::walk_static_block(self, it);
        self.exit_scope(pushed);
    }

    // ---- identifier references (the captured-reference check) ----

    fn visit_identifier_reference(&mut self, it: &oxc::IdentifierReference<'a>) {
        self.check_captured_reference(it.span);
    }

    fn visit_binding_identifier(&mut self, it: &oxc::BindingIdentifier<'a>) {
        // Mirrors the original `enter_identifier`, which fired on pattern
        // binding identifiers too. `check_captured_reference` is a no-op unless
        // the node resolves to a captured reference.
        self.check_captured_reference(it.span);
    }

    fn visit_jsx_identifier(&mut self, it: &oxc::JSXIdentifier<'a>) {
        self.check_captured_reference(it.span);
    }

    fn visit_jsx_attribute(&mut self, it: &oxc::JSXAttribute<'a>) {
        // The original `AstWalker.walk_jsx_element` walked only attribute VALUES;
        // the attribute NAME was never visited. oxc's `walk_jsx_attribute` would
        // otherwise fire `visit_jsx_identifier` on the name. Visit only the value.
        if let Some(value) = &it.value {
            self.visit_jsx_attribute_value(value);
        }
    }

    fn visit_jsx_namespaced_name(&mut self, _it: &oxc::JSXNamespacedName<'a>) {
        // The original explicitly skipped JSXNamespacedName (both as an element
        // name and as an attribute name), never visiting its namespace/name
        // identifiers. oxc's `walk_jsx_namespaced_name` would visit both.
    }

    // ---- reassignment tracking ----

    fn visit_assignment_expression(&mut self, it: &oxc::AssignmentExpression<'a>) {
        let current_scope = self.current_scope();
        if self.error.is_none() {
            self.walk_assignment_target_for_reassignment(&it.left, current_scope);
        }
        oxc_ast_visit::walk::walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &oxc::UpdateExpression<'a>) {
        if let oxc::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &it.argument {
            let current_scope = self.current_scope();
            self.handle_reassignment_identifier(&ident.name, current_scope);
        }
        oxc_ast_visit::walk::walk_update_expression(self, it);
    }

    // ---- positions deliberately NOT visited, matching the original walker ----

    fn visit_static_member_expression(&mut self, it: &oxc::StaticMemberExpression<'a>) {
        // Non-computed member property names (`a.b` → `b`) were never visited.
        self.visit_expression(&it.object);
    }

    fn visit_object_property(&mut self, it: &oxc::ObjectProperty<'a>) {
        // Non-computed object keys were never visited.
        if it.computed {
            self.visit_property_key(&it.key);
        }
        self.visit_expression(&it.value);
    }

    fn visit_class(&mut self, it: &oxc::Class<'a>) {
        // The original walker did not recurse into a class's `super_class`
        // (extends) clause nor its body members; only the type-bearing parts
        // were walked, and those carried no `enter_identifier` calls. So the
        // class contributes nothing to the walker-based capture analysis.
        let _ = it;
    }

    // ---- skip TS type subtrees (the original walked them as opaque RawNodes) ----

    fn visit_ts_type(&mut self, _it: &oxc::TSType<'a>) {}

    fn visit_ts_type_annotation(&mut self, _it: &oxc::TSTypeAnnotation<'a>) {}

    fn visit_ts_type_parameter_instantiation(
        &mut self,
        _it: &oxc::TSTypeParameterInstantiation<'a>,
    ) {
    }

    fn visit_ts_type_parameter_declaration(&mut self, _it: &oxc::TSTypeParameterDeclaration<'a>) {}

    fn visit_ts_type_alias_declaration(&mut self, _it: &oxc::TSTypeAliasDeclaration<'a>) {}

    fn visit_ts_interface_declaration(&mut self, _it: &oxc::TSInterfaceDeclaration<'a>) {}

    fn visit_ts_enum_declaration(&mut self, _it: &oxc::TSEnumDeclaration<'a>) {}

    fn visit_ts_module_declaration(&mut self, _it: &oxc::TSModuleDeclaration<'a>) {}
}

impl<'a, 'b> ContextIdentifierVisitor<'a, 'b> {
    /// Recursively walk an assignment target to find all reassignment target
    /// identifiers, mirroring the original `walk_lval_for_reassignment`.
    fn walk_assignment_target_for_reassignment(
        &mut self,
        target: &oxc::AssignmentTarget<'a>,
        current_scope: ScopeId,
    ) {
        match target {
            oxc::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.handle_reassignment_identifier(&ident.name, current_scope);
            }
            oxc::AssignmentTarget::ArrayAssignmentTarget(pat) => {
                for element in pat.elements.iter().flatten() {
                    self.walk_maybe_default_for_reassignment(element, current_scope);
                }
                if let Some(rest) = &pat.rest {
                    self.walk_assignment_target_for_reassignment(&rest.target, current_scope);
                }
            }
            oxc::AssignmentTarget::ObjectAssignmentTarget(pat) => {
                for prop in &pat.properties {
                    match prop {
                        oxc::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(p) => {
                            self.handle_reassignment_identifier(&p.binding.name, current_scope);
                        }
                        oxc::AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
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
            oxc::AssignmentTarget::StaticMemberExpression(_)
            | oxc::AssignmentTarget::ComputedMemberExpression(_)
            | oxc::AssignmentTarget::PrivateFieldExpression(_) => {}
            // Unsupported TS assignment-target wrappers throw a TS-faithful Todo.
            oxc::AssignmentTarget::TSAsExpression(node) => {
                self.record_unsupported_lval("TSAsExpression", node.span);
            }
            oxc::AssignmentTarget::TSSatisfiesExpression(node) => {
                self.record_unsupported_lval("TSSatisfiesExpression", node.span);
            }
            oxc::AssignmentTarget::TSNonNullExpression(node) => {
                self.record_unsupported_lval("TSNonNullExpression", node.span);
            }
            oxc::AssignmentTarget::TSTypeAssertion(node) => {
                self.record_unsupported_lval("TSTypeAssertion", node.span);
            }
        }
    }

    fn walk_maybe_default_for_reassignment(
        &mut self,
        target: &oxc::AssignmentTargetMaybeDefault<'a>,
        current_scope: ScopeId,
    ) {
        match target {
            oxc::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(node) => {
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
fn make_unsupported_lval_error(
    env: &mut Environment,
    type_name: &str,
    loc: Option<SourceLocation>,
) -> CompilerError {
    let _ = env;
    let mut err = CompilerError::new();
    err.push_error_detail(CompilerErrorDetail {
        category: ErrorCategory::Todo,
        reason: format!(
            "[FindContextIdentifiers] Cannot handle Object destructuring assignment target {type_name}"
        ),
        description: None,
        loc,
        suggestions: None,
    });
    err
}

/// Check if a binding declared at `binding_scope` is captured by a function at `function_scope`.
/// Returns true if the binding is declared above the function (in the parent scope or higher).
fn is_captured_by_function(
    scope_info: &ScopeInfo,
    binding_scope: ScopeId,
    function_scope: ScopeId,
) -> bool {
    let fn_parent = match scope_info.scopes[function_scope.0 as usize].parent {
        Some(p) => p,
        None => return false,
    };
    if binding_scope == fn_parent {
        return true;
    }
    // Walk up from fn_parent to see if binding_scope is an ancestor
    let mut current = scope_info.scopes[fn_parent.0 as usize].parent;
    while let Some(scope_id) = current {
        if scope_id == binding_scope {
            return true;
        }
        current = scope_info.scopes[scope_id.0 as usize].parent;
    }
    false
}

/// Build a set of (BindingId, node_id) pairs for declaration sites in
/// ref_node_id_to_binding. These are entries where the reference's node_id
/// matches the binding's declaration_node_id — i.e., the "reference" is
/// actually the declaration itself.
fn build_declaration_node_ids(scope_info: &ScopeInfo) -> FxHashSet<(BindingId, u32)> {
    let mut result = FxHashSet::default();
    for (&ref_nid, &binding_id) in &scope_info.ref_node_id_to_binding {
        let binding = &scope_info.bindings[binding_id.0 as usize];
        if binding.declaration_node_id == Some(ref_nid) {
            result.insert((binding_id, ref_nid));
        }
    }
    result
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
pub fn find_context_identifiers(
    func: &FunctionNode<'_>,
    scope_info: &ScopeInfo,
    env: &mut Environment,
    identifier_locs: &crate::react_compiler_lowering::identifier_loc_index::IdentifierLocIndex,
    line_offsets: &LineOffsets,
) -> Result<FxHashSet<BindingId>, CompilerError> {
    let func_scope =
        scope_info.resolve_scope_for_node(func.node_id()).unwrap_or(scope_info.program_scope);

    let mut visitor = ContextIdentifierVisitor {
        scope_info,
        line_offsets,
        env,
        scope_stack: vec![func_scope],
        function_stack: Vec::new(),
        binding_info: FxHashMap::default(),
        error: None,
    };

    // Walk params and body (like Babel's func.traverse()): the function node
    // itself is not re-entered, so it is never pushed onto `function_stack`.
    match func {
        FunctionNode::Function(f) => {
            if let Some(this_param) = &f.this_param {
                visitor.visit_ts_this_parameter(this_param);
            }
            visitor.visit_formal_parameters(&f.params);
            if let Some(body) = &f.body {
                visitor.visit_function_body(body);
            }
        }
        FunctionNode::Arrow(arrow) => {
            visitor.visit_formal_parameters(&arrow.params);
            if arrow.expression {
                if let Some(oxc::Statement::ExpressionStatement(es)) = arrow.body.statements.first()
                {
                    visitor.visit_expression(&es.expression);
                } else {
                    visitor.visit_function_body(&arrow.body);
                }
            } else {
                visitor.visit_function_body(&arrow.body);
            }
        }
    }

    if let Some(error) = visitor.error {
        return Err(error);
    }

    // Supplement the walker-based analysis with referenceToBinding data.
    // The AST walker doesn't visit identifiers inside type annotations,
    // but Babel's traverse (used by TS findContextIdentifiers) does.
    // After scope extraction includes type annotation references,
    // we check if any reassigned binding has references in nested function scopes
    // via referenceToBinding — matching the TS behavior.
    //
    // We must skip declaration sites (e.g., the `x` in `function x() {}`),
    // which are included in reference_to_binding but are not true references.
    // Prefer node-ID comparison (immune to position-0 collisions from synthetic
    // nodes), falling back to position when node-IDs are unavailable.
    let declaration_node_ids = build_declaration_node_ids(scope_info);
    for (&ref_nid, &binding_id) in &scope_info.ref_node_id_to_binding {
        match visitor.binding_info.get(&binding_id) {
            Some(info) if info.reassigned && !info.referenced_by_inner_fn => {}
            _ => continue,
        }
        if declaration_node_ids.contains(&(binding_id, ref_nid)) {
            continue;
        }
        // Get the reference's position from identifier_locs for range checks
        let ref_pos = match identifier_locs.get(&ref_nid) {
            Some(entry) => entry.start,
            None => continue,
        };
        let binding = &scope_info.bindings[binding_id.0 as usize];
        // Check if ref_pos is inside a nested function scope
        for (&scope_start, &scope_id) in &scope_info.node_to_scope {
            if scope_start <= ref_pos {
                if let Some(&scope_end) = scope_info.node_to_scope_end.get(&scope_start) {
                    if ref_pos < scope_end
                        && matches!(
                            scope_info.scopes[scope_id.0 as usize].kind,
                            ScopeKind::Function
                        )
                        && is_captured_by_function(scope_info, binding.scope, scope_id)
                    {
                        visitor.binding_info.get_mut(&binding_id).unwrap().referenced_by_inner_fn =
                            true;
                        break;
                    }
                }
            }
        }
    }

    // Collect results
    Ok(visitor
        .binding_info
        .into_iter()
        .filter(|(_, info)| {
            info.reassigned_by_inner_fn || (info.reassigned && info.referenced_by_inner_fn)
        })
        .map(|(id, _)| id)
        .collect())
}
