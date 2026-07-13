use std::borrow::Cow;

use cow_utils::CowUtils;
use rustc_hash::FxHashSet;

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::*;
use crate::react_compiler_utils::{FxIndexMap, FxIndexSet, IdentIndexMap};
use crate::scope::BindingKind as AstBindingKind;
use crate::scope::DeclKind;
use crate::scope::ScopeId;
use crate::scope::ScopeKind;
use crate::scope::ScopeResolver;
use crate::scope::SymbolId;

use oxc_allocator::CloneIn;
use oxc_ast::ast as oxc;
use oxc_ast::ast::BinaryOperator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_str::{Ident, Str, format_ident};

use crate::react_compiler_lowering::FunctionNode;
use crate::react_compiler_lowering::find_context_identifiers::find_context_identifiers;
use crate::react_compiler_lowering::hir_builder::HirBuilder;
use crate::react_compiler_lowering::hir_builder::is_always_reserved_word;
use crate::react_compiler_lowering::hir_builder::reserved_identifier_diagnostic;
use crate::react_compiler_lowering::identifier_loc_index::IdentifierLocIndex;
use crate::react_compiler_lowering::identifier_loc_index::build_identifier_loc_index;

fn validate_ts_this_parameter(
    scope: &ScopeResolver<'_, '_>,
    function_scope: ScopeId,
) -> Result<(), OxcDiagnostic> {
    let Some(symbol_id) = scope.get_binding(function_scope, "this") else {
        return Ok(());
    };
    if matches!(scope.binding_kind(symbol_id), AstBindingKind::Param) {
        return Err(reserved_identifier_diagnostic("this"));
    }
    Ok(())
}

fn is_class_scope_descendant(scope: &ScopeResolver<'_, '_>, scope_id: ScopeId) -> bool {
    scope.ancestors(scope_id).skip(1).any(|s| scope.scope_kind(s) == ScopeKind::Class)
}

fn validate_ts_this_parameters_within(
    scope: &ScopeResolver<'_, '_>,
    function_scope: ScopeId,
) -> Result<(), OxcDiagnostic> {
    for scope_id in scope.function_scopes() {
        if !scope.ancestors(scope_id).any(|s| s == function_scope) {
            continue;
        }
        if is_class_scope_descendant(scope, scope_id) {
            continue;
        }
        validate_ts_this_parameter(scope, scope_id)?;
    }
    Ok(())
}

/// Get the Babel-style type name of an Expression node (e.g. "Identifier", "NumericLiteral").
fn build_temporary_place(builder: &mut HirBuilder<'_, '_>, span: Option<Span>) -> Place {
    let id = builder.make_temporary(span);
    Place { identifier: id, reactive: false, effect: Effect::Unknown, span }
}

/// Promote a temporary identifier to a named identifier (for destructuring).
/// Corresponds to TS `promoteTemporary(identifier)`.
fn promote_temporary(builder: &mut HirBuilder<'_, '_>, identifier_id: IdentifierId) {
    let env = builder.environment_mut();
    let decl_id = env.identifiers[identifier_id.index()].declaration_id;
    env.identifiers[identifier_id.index()].name =
        Some(IdentifierName::Promoted(format_ident!(env.allocator, "#t{}", decl_id.index())));
}

fn lower_value_to_temporary<'a>(
    builder: &mut HirBuilder<'a, '_>,
    value: InstructionValue<'a>,
) -> Result<Place, OxcDiagnostic> {
    // Optimization: if loading an unnamed temporary, skip creating a new instruction
    if let InstructionValue::LoadLocal { ref place, .. } = value {
        let ident = &builder.environment().identifiers[place.identifier.index()];
        if ident.name.is_none() {
            return Ok(place.clone());
        }
    }
    let span = value.span().cloned();
    let place = build_temporary_place(builder, span);
    builder.push(Instruction {
        id: EvaluationOrder::UNSET,
        lvalue: place.clone(),
        value,
        span,
        effects: None,
    });
    Ok(place)
}

fn lower_expression_to_temporary<'a>(
    builder: &mut HirBuilder<'a, '_>,
    expr: &oxc::Expression<'a>,
) -> Result<Place, OxcDiagnostic> {
    let value = lower_expression(builder, expr)?;
    lower_value_to_temporary(builder, value)
}

// =============================================================================
// Operator conversion
// =============================================================================

fn is_binding_in_block_direct_statements(
    declaration_start: Option<u32>,
    stmts: &[oxc::Statement],
) -> bool {
    let decl_start = match declaration_start {
        Some(pos) => pos,
        None => return false,
    };
    for stmt in stmts {
        if matches!(
            stmt,
            oxc::Statement::VariableDeclaration(_)
                | oxc::Statement::FunctionDeclaration(_)
                | oxc::Statement::ClassDeclaration(_)
        ) {
            let span = stmt.span();
            if decl_start >= span.start && decl_start < span.end {
                return true;
            }
        }
    }
    false
}

// =============================================================================
// Statement position helpers
// =============================================================================

fn statement_start(stmt: &oxc::Statement) -> Option<u32> {
    Some(stmt.span().start)
}

fn statement_end(stmt: &oxc::Statement) -> Option<u32> {
    Some(stmt.span().end)
}

/// Collect binding names from a pattern that are declared in the given scope.
fn collect_binding_names_from_pattern(
    pattern: &oxc::BindingPattern,
    scope_id: ScopeId,
    scope: &ScopeResolver<'_, '_>,
    out: &mut FxHashSet<SymbolId>,
) {
    match pattern {
        oxc::BindingPattern::BindingIdentifier(id) => {
            if let Some(symbol_id) = scope.get_binding(scope_id, id.name.as_str()) {
                out.insert(symbol_id);
            }
        }
        oxc::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_names_from_pattern(&prop.value, scope_id, scope, out);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_names_from_pattern(&rest.argument, scope_id, scope, out);
            }
        }
        oxc::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_names_from_pattern(elem, scope_id, scope, out);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_names_from_pattern(&rest.argument, scope_id, scope, out);
            }
        }
        oxc::BindingPattern::AssignmentPattern(assign) => {
            collect_binding_names_from_pattern(&assign.left, scope_id, scope, out);
        }
    }
}

// =============================================================================
// lower_block_statement (with hoisting)
// =============================================================================

/// Lower a BlockStatement with hoisting support.
///
/// Implements the TS BlockStatement hoisting pass: identifies forward references to
/// block-scoped bindings and emits DeclareContext instructions to hoist them.
fn lower_block_statement<'a>(
    builder: &mut HirBuilder<'a, '_>,
    statements: &[oxc::Statement<'a>],
    block_scope: Option<ScopeId>,
    parent_scope: Option<ScopeId>,
) -> Result<(), OxcDiagnostic> {
    let _ = lower_block_statement_inner(builder, statements, block_scope, None, parent_scope);
    Ok(())
}

fn lower_block_statement_with_scope<'a>(
    builder: &mut HirBuilder<'a, '_>,
    statements: &[oxc::Statement<'a>],
    scope_override: ScopeId,
) -> Result<(), OxcDiagnostic> {
    let _ = lower_block_statement_inner(builder, statements, None, Some(scope_override), None);
    Ok(())
}

fn lower_block_statement_inner<'a>(
    builder: &mut HirBuilder<'a, '_>,
    statements: &[oxc::Statement<'a>],
    block_scope: Option<ScopeId>,
    scope_override: Option<ScopeId>,
    parent_scope: Option<ScopeId>,
) -> Result<(), OxcDiagnostic> {
    use crate::scope::BindingKind as AstBindingKind;

    // Look up the block's scope to identify hoistable bindings. Use the scope
    // override when provided (for function body blocks that share the function's
    // scope); otherwise the block's own `scope_id` cell.
    let block_scope_id = scope_override.or(block_scope);

    let scope_id = match block_scope_id {
        Some(id) => id,
        None => {
            for body_stmt in statements {
                lower_statement(builder, body_stmt, None, parent_scope)?;
            }
            return Ok(());
        }
    };

    // Collect hoistable bindings from this scope AND direct child block scopes.
    // In Babel, a function body BlockStatement shares the function's scope, so
    // all bindings (var, const, let) are in one scope. But our scope extraction
    // may split them: function scope has params/var, child block scope has const/let.
    // Including child block scope bindings matches TS behavior where
    // stmt.scope.bindings includes all bindings accessible in the block.
    //
    // IMPORTANT: Only include bindings whose declaration falls within THIS block's
    // statement range. Bindings declared in nested blocks (e.g., inside an `if`
    // branch) should NOT be hoisted at the parent level — they'll be handled when
    // that nested block is recursively lowered. This prevents DeclareContext from
    // being emitted before an `if` terminal for variables declared within the branch.
    let scope = builder.scope();
    let hoistable: Vec<(SymbolId, Ident, AstBindingKind, DeclKind, Option<u32>)> = scope
        .bindings_with_child_blocks(scope_id)
        .into_iter()
        .filter(|&sid| {
            // Type-only symbols (type parameters, interfaces, pure type aliases)
            // are not part of Babel's `scope.bindings`, so its hoisting analysis
            // never treats a pure-type-position reference to one as a
            // referenced-before-declared use. OXC does give them a symbol, so
            // without this guard a generic function that mentions its own type
            // parameter `T` inside a nested function over-bails with
            // "Unsupported declaration type for hoisting". See
            // `ScopeResolver::is_type_only_binding`.
            !scope.is_type_only_binding(sid)
                && !matches!(
                    scope.binding_kind(sid),
                    AstBindingKind::Param | AstBindingKind::Module
                )
                && !matches!(
                    scope.decl_kind(sid),
                    DeclKind::FunctionExpression
                        | DeclKind::TSTypeAliasDeclaration
                        | DeclKind::TSEnumDeclaration
                )
        })
        .map(|sid| {
            (
                sid,
                scope.symbol_ident(sid),
                scope.binding_kind(sid),
                scope.decl_kind(sid),
                scope.declaration_ident(sid).map(|id| id.span.start),
            )
        })
        .collect();

    if hoistable.is_empty() {
        // No hoistable bindings, just lower statements normally
        for body_stmt in statements {
            lower_statement(builder, body_stmt, None, Some(scope_id))?;
        }
        return Ok(());
    }

    // Track which bindings have been "declared" (their declaration statement has been seen)
    let mut declared: FxHashSet<SymbolId> = FxHashSet::default();

    for body_stmt in statements {
        let stmt_start = statement_start(body_stmt).unwrap_or(0);
        let stmt_end = statement_end(body_stmt).unwrap_or(u32::MAX);
        let is_function_decl = matches!(body_stmt, oxc::Statement::FunctionDeclaration(_));

        // Collect ranges of nested function scopes within this statement.
        // Used to check per-reference whether a reference is inside a nested function,
        // rather than checking once per-statement.
        let nested_function_ranges: Vec<(u32, u32)> = if is_function_decl {
            // For function declarations, fnDepth starts at 1 (all refs are inside)
            vec![(stmt_start, stmt_end)]
        } else {
            builder
                .scope()
                .function_scope_ranges()
                .iter()
                .copied()
                .filter(|&(pos, _)| pos > stmt_start && pos < stmt_end)
                .collect()
        };

        // Find references to not-yet-declared hoistable bindings within this statement
        struct HoistInfo<'a> {
            binding_id: SymbolId,
            name: Ident<'a>,
            kind: AstBindingKind,
            declaration_type: DeclKind,
            first_ref_span: Span,
        }
        let mut will_hoist: Vec<HoistInfo> = Vec::new();

        for (binding_id, name, kind, decl_type, decl_start) in &hoistable {
            if declared.contains(binding_id) {
                continue;
            }

            // Find the first reference to this binding in the statement's range.
            // Exclude JSX identifier references: while Babel's scope system links JSX
            // tag names to local bindings (and the context capture pass includes them),
            // the TS hoisting analysis does NOT traverse JSX elements. This mismatch
            // is intentional — it matches the TS behavior where <colgroup> adds
            // "colgroup" to the context but does NOT trigger hoisting, causing
            // EnterSSA to error with "Expected identifier to be defined before use".
            let mut refs_in_stmt: Vec<Span> = scope
                .reference_ids(*binding_id)
                .iter()
                .filter_map(|&ref_id| {
                    let entry = builder.identifier_spans().reference(ref_id)?;
                    let ref_start = entry.span.start;
                    if ref_start < stmt_start || ref_start >= stmt_end {
                        return None;
                    }
                    if entry.is_jsx() {
                        return None;
                    }
                    Some(entry.span)
                })
                .collect();
            // For hoisted bindings (function declarations) outside their own
            // declaration statement, the declaration site itself counts as a
            // reference (Babel's binding references include the declaration).
            let decl_counts_as_ref = matches!(kind, AstBindingKind::Hoisted) && !is_function_decl;
            if decl_counts_as_ref {
                if let Some(decl_span) = builder.identifier_spans().declaration_span(*binding_id) {
                    if decl_span.start >= stmt_start && decl_span.start < stmt_end {
                        refs_in_stmt.push(decl_span);
                    }
                }
            }

            if refs_in_stmt.is_empty() {
                continue;
            }

            let first_ref_span = *refs_in_stmt.iter().min_by_key(|s| s.start).unwrap();

            // Hoist if: (1) binding is "hoisted" kind (function declaration), or
            // (2) any reference to this binding is inside a nested function scope.
            // Check per-reference rather than per-statement to correctly handle
            // statements that contain both nested functions and top-level code.
            let is_hoisted_kind = matches!(kind, AstBindingKind::Hoisted);
            let refs_in_nested_fn: Vec<Span> = refs_in_stmt
                .iter()
                .copied()
                .filter(|s| {
                    nested_function_ranges
                        .iter()
                        .any(|&(fn_start, fn_end)| s.start >= fn_start && s.start < fn_end)
                })
                .collect();
            let should_hoist = is_hoisted_kind || !refs_in_nested_fn.is_empty();
            if should_hoist {
                // Bindings pulled in from CHILD block scopes (the
                // scope_bindings_with_children descent compensates for scope
                // splitting) only hoist when declared as a direct statement of
                // THIS block; ones declared inside nested control-flow blocks
                // are handled when those blocks are recursively lowered. TS
                // never sees child-block bindings here (Babel's
                // stmt.scope.bindings holds only the block's own scope), so the
                // guard must NOT apply to own-scope bindings: catch params and
                // for-in/for-of head vars belong to the block's scope without
                // being declared by any direct statement, and TS hoists them.
                if scope.symbol_scope(*binding_id) != scope_id
                    && !is_binding_in_block_direct_statements(*decl_start, statements)
                {
                    continue;
                }
                // For hoisted bindings (function declarations), use the first reference
                // overall. For non-hoisted bindings, use the first reference inside a
                // nested function.
                let first_ref_span = if is_hoisted_kind {
                    first_ref_span
                } else {
                    *refs_in_nested_fn.iter().min_by_key(|s| s.start).unwrap()
                };
                will_hoist.push(HoistInfo {
                    binding_id: *binding_id,
                    name: *name,
                    kind: *kind,
                    declaration_type: *decl_type,
                    first_ref_span,
                });
            }
        }

        // Sort by first reference position to match TS traversal order
        will_hoist.sort_by_key(|h| h.first_ref_span.start);

        // Emit DeclareContext for hoisted bindings
        for info in &will_hoist {
            if builder.environment().is_hoisted_identifier(info.binding_id) {
                continue;
            }

            let hoist_kind = match info.kind {
                AstBindingKind::Const | AstBindingKind::Var => InstructionKind::HoistedConst,
                AstBindingKind::Let => InstructionKind::HoistedLet,
                AstBindingKind::Hoisted => InstructionKind::HoistedFunction,
                _ => {
                    if info.declaration_type == DeclKind::FunctionDeclaration {
                        InstructionKind::HoistedFunction
                    } else if info.declaration_type == DeclKind::VariableDeclarator {
                        // Unsupported hoisting for this declaration kind
                        builder.record_error(
                            ErrorCategory::Todo
                                .diagnostic("Handle non-const declarations for hoisting")
                                .with_help(format!(
                                    "variable \"{}\" declared with {:?}",
                                    info.name, info.kind
                                )),
                        )?;
                        continue;
                    } else {
                        builder.record_error(
                            ErrorCategory::Todo
                                .diagnostic("Unsupported declaration type for hoisting")
                                .with_help(format!(
                                    "variable \"{}\" declared with {}",
                                    info.name,
                                    info.declaration_type.as_str()
                                )),
                        )?;
                        continue;
                    }
                }
            };

            let ref_span = Some(info.first_ref_span);
            let identifier = builder.resolve_binding(info.name, info.binding_id)?;
            let place =
                Place { effect: Effect::Unknown, identifier, reactive: false, span: ref_span };
            lower_value_to_temporary(
                builder,
                InstructionValue::DeclareContext {
                    lvalue: LValue { kind: hoist_kind, place },
                    span: ref_span,
                },
            )?;
            builder.environment_mut().add_hoisted_identifier(info.binding_id);
            // Hoisted identifiers also become context identifiers (matching TS addHoistedIdentifier)
            builder.add_context_identifier(info.binding_id);
        }

        // After processing the statement, mark any bindings it declares as "seen".
        // This must cover all statement types that can introduce bindings.
        match body_stmt {
            oxc::Statement::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    if let Some(symbol_id) = scope.get_binding(scope_id, id.name.as_str()) {
                        declared.insert(symbol_id);
                    }
                }
            }
            oxc::Statement::VariableDeclaration(var_decl) => {
                for decl in &var_decl.declarations {
                    collect_binding_names_from_pattern(&decl.id, scope_id, scope, &mut declared);
                }
            }
            oxc::Statement::ClassDeclaration(cls) => {
                if let Some(id) = &cls.id {
                    if let Some(symbol_id) = scope.get_binding(scope_id, id.name.as_str()) {
                        declared.insert(symbol_id);
                    }
                }
            }
            _ => {
                // For other statement types (e.g. ForStatement with VariableDeclaration in init),
                // we rely on the per-symbol reference check for forward references.
                // Any bindings declared by child scopes won't be in this block's scope anyway.
            }
        }

        lower_statement(builder, body_stmt, None, Some(scope_id))?;
    }
    Ok(())
}

// =============================================================================
// lower_statement
// =============================================================================

enum FunctionBody<'b, 'a> {
    Block(&'b oxc::FunctionBody<'a>),
    Expression(&'b oxc::Expression<'a>),
}

type LowerInnerResult<'a> = Result<
    (HirFunction<'a>, IdentIndexMap<'a, SymbolId>, FxIndexMap<SymbolId, IdentifierId>),
    OxcDiagnostic,
>;

/// Main entry point: lower a function AST node into HIR.
///
/// Receives a `FunctionNode` (discovered by the entrypoint) and lowers it to HIR.
pub fn lower<'a>(
    func: &FunctionNode<'_, 'a>,
    scope: &ScopeResolver<'_, 'a>,
    env: &mut Environment<'a>,
) -> Result<HirFunction<'a>, OxcDiagnostic> {
    // Extract params, body, generator, is_async, span, scope_id, and the AST function's own id
    // Note: `id` param may include inferred names (e.g., from `const Foo = () => {}`),
    // but the HIR function's `id` field should only include the function's own AST id
    // (FunctionDeclaration.id or FunctionExpression.id, NOT arrow functions).
    let (params, body, generator, is_async, span, ast_id) = match func {
        FunctionNode::Function(f) => {
            let body_ref = f.body.as_deref().expect("component function has a body");
            (
                f.params.as_ref(),
                FunctionBody::Block(body_ref),
                f.generator,
                f.r#async,
                f.span,
                f.id.as_ref().map(|id| id.name),
            )
        }
        FunctionNode::Arrow(arrow) => {
            let body = if arrow.expression {
                match arrow.body.statements.first() {
                    Some(oxc::Statement::ExpressionStatement(es)) => {
                        FunctionBody::Expression(&es.expression)
                    }
                    _ => FunctionBody::Block(arrow.body.as_ref()),
                }
            } else {
                FunctionBody::Block(arrow.body.as_ref())
            };
            (arrow.params.as_ref(), body, false, arrow.r#async, arrow.span, None)
        }
    };

    let scope_id = func.scope_id().unwrap_or_else(|| scope.program_scope());

    validate_ts_this_parameters_within(scope, scope_id)?;

    // Build identifier location index from the AST (replaces serialized referenceLocs/jsxReferencePositions)
    let identifier_spans = build_identifier_loc_index(func);

    // Pre-compute context identifiers: variables captured across function boundaries
    let context_identifiers = find_context_identifiers(func, scope, &identifier_spans)?;

    // For top-level functions, context is empty (no captured refs)
    let context_map: FxIndexMap<SymbolId, Option<Span>> = FxIndexMap::default();

    let (hir_func, _used_names, _child_bindings) = lower_inner(
        params,
        body,
        ast_id,
        generator,
        is_async,
        span,
        scope,
        env,
        None, // no pre-existing bindings for top-level
        None, // no pre-existing used_names for top-level
        context_map,
        scope_id,
        scope_id, // component_scope = function_scope for top-level
        &context_identifiers,
        true, // is_top_level
        &identifier_spans,
    )?;

    Ok(hir_func)
}

// =============================================================================
// Stubs for future milestones
// =============================================================================

/// Result of resolving an identifier for assignment.
#[allow(clippy::too_many_arguments)]
fn lower_inner<'a>(
    params: &oxc::FormalParameters<'a>,
    body: FunctionBody<'_, 'a>,
    id: Option<Ident<'a>>,
    generator: bool,
    is_async: bool,
    span: Span,
    scope: &ScopeResolver<'_, 'a>,
    env: &mut Environment<'a>,
    parent_bindings: Option<FxIndexMap<SymbolId, IdentifierId>>,
    parent_used_names: Option<IdentIndexMap<'a, SymbolId>>,
    context_map: FxIndexMap<SymbolId, Option<Span>>,
    function_scope: ScopeId,
    component_scope: ScopeId,
    context_identifiers: &FxHashSet<SymbolId>,
    is_top_level: bool,
    identifier_spans: &IdentifierLocIndex,
) -> LowerInnerResult<'a> {
    validate_ts_this_parameter(scope, function_scope)?;

    let mut builder = HirBuilder::new(
        env,
        scope,
        function_scope,
        component_scope,
        context_identifiers.clone(),
        parent_bindings,
        Some(context_map.clone()),
        None,
        parent_used_names,
        identifier_spans,
    );

    // Build context places from the captured refs
    let mut context: Vec<Place> = Vec::new();
    for (&symbol_id, ctx_span) in &context_map {
        let identifier = builder.resolve_binding(scope.symbol_ident(symbol_id), symbol_id)?;
        context.push(Place {
            identifier,
            effect: Effect::Unknown,
            reactive: false,
            span: *ctx_span,
        });
    }

    // Process parameters.
    let mut hir_params: Vec<ParamPattern> = Vec::new();
    for param in &params.items {
        if param.initializer.is_none()
            && let oxc::BindingPattern::BindingIdentifier(ident) = &param.pattern
        {
            if is_always_reserved_word(ident.name.as_str()) {
                return Err(reserved_identifier_diagnostic(ident.name.as_str()));
            }
            let param_span = ident.span;
            let mut binding = builder.resolve_identifier(
                ident.name,
                param_span,
                builder.scope().resolve_binding_identifier(ident),
            )?;
            if !matches!(binding, VariableBinding::Identifier { .. }) {
                if let Some(symbol_id) = builder
                    .scope()
                    .find_binding_in_descendants(ident.name.as_str(), builder.function_scope())
                {
                    let binding_kind = crate::react_compiler_lowering::convert_binding_kind(
                        &builder.scope().binding_kind(symbol_id),
                    );
                    let identifier = builder.resolve_binding_with_span(
                        ident.name,
                        symbol_id,
                        Some(param_span),
                    )?;
                    binding = VariableBinding::Identifier { identifier, binding_kind };
                }
            }
            match binding {
                VariableBinding::Identifier { identifier, .. } => {
                    builder.set_identifier_declaration_span(identifier, param_span);
                    let place = Place {
                        identifier,
                        effect: Effect::Unknown,
                        reactive: false,
                        span: Some(param_span),
                    };
                    hir_params.push(ParamPattern::Place(place));
                }
                _ => {
                    builder.record_diagnostic(
                        ErrorCategory::Invariant
                            .diagnostic("Could not find binding")
                            .with_help(format!(
                                "[BuildHIR] Could not find binding for param `{}`",
                                ident.name.as_str()
                            ))
                            .with_label(ident.span.label("Could not find binding")),
                    );
                }
            }
            continue;
        }
        // Destructuring (`{a}`, `[a]`), defaulted (`x = 1`, `{a} = obj`), and any
        // other non-plain-identifier param. Babel modeled a default param as an
        // `AssignmentPattern` param node; in oxc the default lives in
        // `FormalParameter.initializer` alongside the underlying `pattern`.
        // Create a temporary place for the param value, promote it, push it as the
        // param, then delegate the binding via `lower_binding_assignment` (running
        // the default ternary first when an initializer is present).
        let param_span = param.span;
        let place = build_temporary_place(&mut builder, Some(param_span));
        promote_temporary(&mut builder, place.identifier);
        hir_params.push(ParamPattern::Place(place.clone()));
        let value = if let Some(initializer) = &param.initializer {
            lower_default_to_temp(&mut builder, param_span, initializer, place)?
        } else {
            place
        };
        lower_binding_assignment(
            &mut builder,
            param_span,
            InstructionKind::Let,
            &param.pattern,
            value,
            AssignmentStyle::Assignment,
        )?;
    }

    // Rest parameter (`...rest`). Babel modeled this as a `RestElement` param; in
    // oxc it is the separate `params.rest` field. Push a spread param place and
    // delegate the binding of the rest argument.
    if let Some(rest) = &params.rest {
        let rest_span = rest.span;
        let place = build_temporary_place(&mut builder, Some(rest_span));
        hir_params.push(ParamPattern::Spread(SpreadPattern { place: place.clone() }));
        lower_binding_assignment(
            &mut builder,
            rest_span,
            InstructionKind::Let,
            &rest.rest.argument,
            place,
            AssignmentStyle::Assignment,
        )?;
    }

    // Lower the body
    let mut directives: Vec<Str<'a>> = Vec::new();
    match body {
        FunctionBody::Expression(expr) => {
            let fallthrough = builder.reserve(BlockKind::Block);
            let value = lower_expression_to_temporary(&mut builder, expr)?;
            builder.terminate_with_continuation(
                Terminal::Return {
                    value,
                    return_variant: ReturnVariant::Implicit,
                    id: EvaluationOrder::UNSET,
                    span: None,
                    effects: None,
                },
                fallthrough,
            );
        }
        FunctionBody::Block(block) => {
            directives = block.directives.iter().map(|d| d.expression.value).collect();
            // A function body shares the function's scope (the scope cell lives on
            // the function node, not the block), so pass it as the scope override.
            lower_block_statement_with_scope(&mut builder, &block.statements, function_scope)?;
        }
    }

    // Emit final Return(Void, undefined)
    let undefined_value =
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, span: None };
    let return_value = lower_value_to_temporary(&mut builder, undefined_value)?;
    builder.terminate(
        Terminal::Return {
            value: return_value,
            return_variant: ReturnVariant::Void,
            id: EvaluationOrder::UNSET,
            span: None,
            effects: None,
        },
        None,
    );

    // Build the HIR
    let (hir_body, instructions, used_names, child_bindings) = builder.build()?;

    // Create the returns place
    let returns =
        crate::react_compiler_lowering::hir_builder::create_temporary_place(env, Some(span));

    Ok((
        HirFunction {
            span: Some(span),
            id,
            name_hint: None,
            fn_type: if is_top_level { env.fn_type } else { ReactFunctionType::Other },
            params: hir_params,
            returns,
            context,
            body: hir_body,
            instructions,
            generator,
            is_async,
            directives,
            aliasing_effects: None,
        },
        used_names,
        child_bindings,
    ))
}

// =============================================================================
// lower_expression / lower_statement — Stage 1a skeleton catch-all arms.
//
// Arms are ported incrementally from `git show HEAD:.../build_hir.rs` + the
// convert-ast reference. Until an arm lands, the catch-all bails to an undefined
// primitive / no-op so the crate compiles and the differential green-set grows.
// =============================================================================

// =============================================================================
// lower_identifier
// =============================================================================

/// Resolve an identifier to a Place. Local/context identifiers return a Place
/// referencing the binding; globals/imports emit a LoadGlobal. AST-agnostic.
fn lower_identifier<'a>(
    builder: &mut HirBuilder<'a, '_>,
    name: Ident<'a>,
    span: Span,
    symbol: Option<SymbolId>,
) -> Result<Place, OxcDiagnostic> {
    let binding = builder.resolve_identifier(name, span, symbol)?;
    match binding {
        VariableBinding::Identifier { identifier, .. } => {
            Ok(Place { identifier, effect: Effect::Unknown, reactive: false, span: Some(span) })
        }
        _ => {
            if let VariableBinding::Global { name } = binding {
                if name == "eval" {
                    builder.record_error(
                        ErrorCategory::UnsupportedSyntax
                            .diagnostic("The 'eval' function is not supported")
                            .with_help("Eval is an anti-pattern in JavaScript, and the code executed cannot be evaluated by React Compiler")
                            .with_label(span),
                    )?;
                }
            }
            let non_local_binding = match binding {
                VariableBinding::Global { name } => NonLocalBinding::Global { name },
                VariableBinding::ImportDefault { name, module } => {
                    NonLocalBinding::ImportDefault { name, module }
                }
                VariableBinding::ImportSpecifier { name, module, imported } => {
                    NonLocalBinding::ImportSpecifier { name, module, imported }
                }
                VariableBinding::ImportNamespace { name, module } => {
                    NonLocalBinding::ImportNamespace { name, module }
                }
                VariableBinding::ModuleLocal { name } => NonLocalBinding::ModuleLocal { name },
                VariableBinding::Identifier { .. } => unreachable!(),
            };
            let instr_value =
                InstructionValue::LoadGlobal { binding: non_local_binding, span: Some(span) };
            lower_value_to_temporary(builder, instr_value)
        }
    }
}

fn convert_unary_operator(op: oxc::UnaryOperator) -> UnaryOperator {
    use oxc::UnaryOperator as O;
    match op {
        O::UnaryNegation => UnaryOperator::Minus,
        O::UnaryPlus => UnaryOperator::Plus,
        O::LogicalNot => UnaryOperator::Not,
        O::BitwiseNot => UnaryOperator::BitwiseNot,
        O::Typeof => UnaryOperator::TypeOf,
        O::Void => UnaryOperator::Void,
        O::Delete => unreachable!("delete is handled in the UnaryExpression arm"),
    }
}

enum MemberProperty<'a> {
    Literal(PropertyLiteral<'a>),
    Computed(Place),
}

struct LoweredMemberExpression<'a> {
    object: Place,
    property: MemberProperty<'a>,
    value: InstructionValue<'a>,
}

/// Lower a member access (oxc's Static / Computed / PrivateField variants) into a
/// receiver place + property + load value.
fn lower_member_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    member: &oxc::MemberExpression<'a>,
) -> Result<LoweredMemberExpression<'a>, OxcDiagnostic> {
    lower_member_expression_impl(builder, member, None)
}

fn lower_member_expression_impl<'a>(
    builder: &mut HirBuilder<'a, '_>,
    member: &oxc::MemberExpression<'a>,
    lowered_object: Option<Place>,
) -> Result<LoweredMemberExpression<'a>, OxcDiagnostic> {
    match member {
        oxc::MemberExpression::StaticMemberExpression(m) => {
            let span = Some(m.span);
            let object = match lowered_object {
                Some(obj) => obj,
                None => lower_expression_to_temporary(builder, &m.object)?,
            };
            let prop_literal = PropertyLiteral::String(m.property.name);
            let value = InstructionValue::PropertyLoad {
                object: object.clone(),
                property: prop_literal,
                span,
            };
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Literal(prop_literal),
                value,
            })
        }
        oxc::MemberExpression::ComputedMemberExpression(m) => {
            let span = Some(m.span);
            let object = match lowered_object {
                Some(obj) => obj,
                None => lower_expression_to_temporary(builder, &m.object)?,
            };
            // A numeric computed index is treated as a PropertyLoad (matches TS).
            if let oxc::Expression::NumericLiteral(lit) = &m.expression {
                let prop_literal = PropertyLiteral::Number(FloatValue::new(lit.value));
                let value = InstructionValue::PropertyLoad {
                    object: object.clone(),
                    property: prop_literal,
                    span,
                };
                return Ok(LoweredMemberExpression {
                    object,
                    property: MemberProperty::Literal(prop_literal),
                    value,
                });
            }
            let property = lower_expression_to_temporary(builder, &m.expression)?;
            let value = InstructionValue::ComputedLoad {
                object: object.clone(),
                property: property.clone(),
                span,
            };
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Computed(property),
                value,
            })
        }
        oxc::MemberExpression::PrivateFieldExpression(m) => {
            let span = Some(m.span);
            let object = match lowered_object {
                Some(obj) => obj,
                None => lower_expression_to_temporary(builder, &m.object)?,
            };
            // TODO(stage1a-arms): private field access needs a private-name property
            // load + OriginalNode bail; defer to a later batch.
            builder.record_error(
                ErrorCategory::Todo
                    .diagnostic("(BuildHIR::lowerMemberExpression) Handle private field property")
                    .with_labels(span),
            )?;
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Literal(PropertyLiteral::String(Ident::empty())),
                value: InstructionValue::Primitive { value: PrimitiveValue::Undefined, span },
            })
        }
    }
}

/// Build a HIR `TemplateQuasi` from an oxc `TemplateElement`.
fn template_quasi_from_oxc<'a>(q: &oxc::TemplateElement<'a>) -> TemplateQuasi<'a> {
    TemplateQuasi { raw: q.value.raw, cooked: q.value.cooked }
}

/// Lower the `import` keyword callee of an `ImportExpression`. The original Babel
/// path treats this as the `Import` node, which bails (records an error) and
/// returns an undefined primitive that is then loaded to a temporary.
fn lower_import_keyword_to_temporary(
    builder: &mut HirBuilder<'_, '_>,
    span: &Option<Span>,
) -> Result<Place, OxcDiagnostic> {
    builder.record_error(
        ErrorCategory::Todo
            .diagnostic("(BuildHIR::lowerExpression) Handle Import expressions")
            .with_labels(*span),
    )?;
    lower_value_to_temporary(
        builder,
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, span: *span },
    )
}

/// Lower a `PrivateName` operand (e.g. the left side of `#f in obj`). The original
/// Babel path bails (records an error) and returns an undefined primitive that is
/// then loaded to a temporary.
fn lower_private_name_to_temporary(
    builder: &mut HirBuilder<'_, '_>,
    span: oxc_span::Span,
) -> Result<Place, OxcDiagnostic> {
    let span = Some(span);
    builder.record_error(
        ErrorCategory::Todo
            .diagnostic("(BuildHIR::lowerExpression) Handle PrivateName expressions")
            .with_labels(span),
    )?;
    lower_value_to_temporary(
        builder,
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, span },
    )
}

/// Coarse classification of an oxc TS type, mirroring `lower_type_annotation`
/// (array / primitive / everything else).
fn classify_ts_type(ty: &oxc::TSType) -> crate::react_compiler_hir::RawTypeCategory {
    use crate::react_compiler_hir::RawTypeCategory;
    match ty {
        oxc::TSType::TSArrayType(_) => RawTypeCategory::Array,
        oxc::TSType::TSTypeReference(r) => match &r.type_name {
            oxc::TSTypeName::IdentifierReference(id) if id.name == "Array" => {
                RawTypeCategory::Array
            }
            _ => RawTypeCategory::Other,
        },
        oxc::TSType::TSBooleanKeyword(_)
        | oxc::TSType::TSNullKeyword(_)
        | oxc::TSType::TSNumberKeyword(_)
        | oxc::TSType::TSStringKeyword(_)
        | oxc::TSType::TSSymbolKeyword(_)
        | oxc::TSType::TSUndefinedKeyword(_)
        | oxc::TSType::TSVoidKeyword(_) => RawTypeCategory::Primitive,
        _ => RawTypeCategory::Other,
    }
}

/// Lower the HIR `Type` for a TS type annotation from its coarse classification,
/// mirroring `lower_type_annotation`.
fn lower_ts_type<'a>(builder: &mut HirBuilder<'a, '_>, ty: &oxc::TSType) -> Type<'a> {
    use crate::react_compiler_hir::RawTypeCategory;
    match classify_ts_type(ty) {
        RawTypeCategory::Array => Type::Object { shape_id: Some(object_shape::BUILT_IN_ARRAY_ID) },
        RawTypeCategory::Primitive => Type::Primitive,
        RawTypeCategory::Other => builder.make_type(),
    }
}

/// Lower `x as T` / `x satisfies T` / `<T>x` to a `TypeCastExpression`: the inner
/// expression is lowered to a temporary and the type metadata is attached. Mirrors
/// the original Babel `TSAsExpression`/`TSSatisfiesExpression`/`TSTypeAssertion`
/// arms. The original `TSType` AST node is stored directly so codegen can re-emit
/// it by cloning (applying any identifier renames) instead of re-parsing source.
fn lower_type_cast_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    span: oxc_span::Span,
    expression: &oxc::Expression<'a>,
    type_annotation: &oxc::TSType<'a>,
    cast: fn(&'a oxc::TSType<'a>) -> TypeCast<'a>,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    let span = Some(span);
    let value = lower_expression_to_temporary(builder, expression)?;
    // `lower_ts_type` allocates a type var (via `make_type`); keep the call for
    // behavior parity even though the resulting type is no longer stored.
    lower_ts_type(builder, type_annotation);
    // Clone the annotation into the arena (preserving semantic id cells so
    // codegen's rename-by-reference-identity still applies) so the HIR holds no
    // borrows of the input `Program` reference.
    let allocator = builder.environment().allocator;
    let type_annotation = &*allocator.alloc(type_annotation.clone_in_with_semantic_ids(allocator));
    Ok(InstructionValue::TypeCastExpression { value, cast: cast(type_annotation), span })
}

/// Lower a member-expression update target (oxc's member variants of
/// `SimpleAssignmentTarget`) into a receiver place + property + load value,
/// mirroring `lower_member_expression_impl`.
fn lower_member_expression_from_simple_target<'a>(
    builder: &mut HirBuilder<'a, '_>,
    target: &oxc::SimpleAssignmentTarget<'a>,
) -> Result<LoweredMemberExpression<'a>, OxcDiagnostic> {
    match target {
        oxc::SimpleAssignmentTarget::StaticMemberExpression(m) => {
            let span = Some(m.span);
            let object = lower_expression_to_temporary(builder, &m.object)?;
            let prop_literal = PropertyLiteral::String(m.property.name);
            let value = InstructionValue::PropertyLoad {
                object: object.clone(),
                property: prop_literal,
                span,
            };
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Literal(prop_literal),
                value,
            })
        }
        oxc::SimpleAssignmentTarget::ComputedMemberExpression(m) => {
            let span = Some(m.span);
            let object = lower_expression_to_temporary(builder, &m.object)?;
            if let oxc::Expression::NumericLiteral(lit) = &m.expression {
                let prop_literal = PropertyLiteral::Number(FloatValue::new(lit.value));
                let value = InstructionValue::PropertyLoad {
                    object: object.clone(),
                    property: prop_literal,
                    span,
                };
                return Ok(LoweredMemberExpression {
                    object,
                    property: MemberProperty::Literal(prop_literal),
                    value,
                });
            }
            let property = lower_expression_to_temporary(builder, &m.expression)?;
            let value = InstructionValue::ComputedLoad {
                object: object.clone(),
                property: property.clone(),
                span,
            };
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Computed(property),
                value,
            })
        }
        oxc::SimpleAssignmentTarget::PrivateFieldExpression(m) => {
            let span = Some(m.span);
            let object = lower_expression_to_temporary(builder, &m.object)?;
            builder.record_error(
                ErrorCategory::Todo
                    .diagnostic("(BuildHIR::lowerMemberExpression) Handle private field property")
                    .with_labels(span),
            )?;
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Literal(PropertyLiteral::String(Ident::empty())),
                value: InstructionValue::Primitive { value: PrimitiveValue::Undefined, span },
            })
        }
        _ => {
            unreachable!("lower_member_expression_from_simple_target called on a non-member target")
        }
    }
}

fn lower_arguments<'a>(
    builder: &mut HirBuilder<'a, '_>,
    args: &[oxc::Argument<'a>],
) -> Result<Vec<PlaceOrSpread>, OxcDiagnostic> {
    let mut result = Vec::new();
    for arg in args {
        match arg {
            oxc::Argument::SpreadElement(spread) => {
                let place = lower_expression_to_temporary(builder, &spread.argument)?;
                result.push(PlaceOrSpread::Spread(SpreadPattern { place }));
            }
            _ => {
                let expr = arg.as_expression().expect("non-spread argument is an expression");
                let place = lower_expression_to_temporary(builder, expr)?;
                result.push(PlaceOrSpread::Place(place));
            }
        }
    }
    Ok(result)
}

/// Result of resolving an identifier for assignment.
enum IdentifierForAssignment<'a> {
    Place(Place),
    Global { name: Ident<'a> },
}

/// Resolve an identifier as an assignment target. AST-agnostic. Returns None if
/// the binding could not be found (error recorded).
fn lower_identifier_for_assignment<'a>(
    builder: &mut HirBuilder<'a, '_>,
    span: Span,
    ident_span: Span,
    kind: InstructionKind,
    name: Ident<'a>,
    symbol: Option<SymbolId>,
) -> Result<Option<IdentifierForAssignment<'a>>, OxcDiagnostic> {
    let mut binding = builder.resolve_identifier(name, ident_span, symbol)?;
    if !matches!(binding, VariableBinding::Identifier { .. }) && kind != InstructionKind::Reassign {
        if let Some(symbol_id) =
            builder.scope().find_binding_in_descendants(name.as_str(), builder.function_scope())
        {
            let bk = crate::react_compiler_lowering::convert_binding_kind(
                &builder.scope().binding_kind(symbol_id),
            );
            let identifier =
                builder.resolve_binding_with_span(name, symbol_id, Some(ident_span))?;
            binding = VariableBinding::Identifier { identifier, binding_kind: bk };
        }
    }
    match binding {
        VariableBinding::Identifier { identifier, binding_kind, .. } => {
            if kind != InstructionKind::Reassign {
                builder.set_identifier_declaration_span(identifier, ident_span);
            }
            if binding_kind == BindingKind::Const && kind == InstructionKind::Reassign {
                builder.record_error(
                    ErrorCategory::Syntax
                        .diagnostic("Cannot reassign a `const` variable")
                        .with_help(format!("`{}` is declared as const", name))
                        .with_label(span),
                )?;
                return Ok(None);
            }
            Ok(Some(IdentifierForAssignment::Place(Place {
                identifier,
                effect: Effect::Unknown,
                reactive: false,
                span: Some(span),
            })))
        }
        VariableBinding::Global { name: gname } => {
            if kind == InstructionKind::Reassign {
                Ok(Some(IdentifierForAssignment::Global { name: gname }))
            } else {
                builder.record_error(
                    ErrorCategory::Invariant
                        .diagnostic("Could not find binding for declaration")
                        .with_label(span),
                )?;
                Ok(None)
            }
        }
        _ => {
            if kind == InstructionKind::Reassign {
                Ok(Some(IdentifierForAssignment::Global { name }))
            } else {
                builder.record_error(
                    ErrorCategory::Invariant
                        .diagnostic("Could not find binding for declaration")
                        .with_label(span),
                )?;
                Ok(None)
            }
        }
    }
}

/// The style of assignment (used internally by the lower-assignment helpers).
/// Mirrors the original `AssignmentStyle`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AssignmentStyle {
    /// Assignment via `=`
    Assignment,
    /// Destructuring assignment
    Destructure,
}

/// Assign `value` to a binding pattern (variable declaration / destructuring param).
/// Faithful translation of the original `lower_assignment` for the BindingPattern
/// targets (Identifier / Object / Array / Assignment). The original unified binding
/// patterns and assignment targets under `PatternLike`; oxc splits them, so this
/// handles only the binding side. `kind` is never `Reassign` on this path, so
/// `force_temporaries` is always false.
fn lower_binding_assignment<'a>(
    builder: &mut HirBuilder<'a, '_>,
    span: Span,
    kind: InstructionKind,
    target: &oxc::BindingPattern<'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, OxcDiagnostic> {
    match target {
        oxc::BindingPattern::BindingIdentifier(id) => {
            let result = lower_identifier_for_assignment(
                builder,
                span,
                id.span,
                kind,
                id.name,
                builder.scope().resolve_binding_identifier(id),
            )?;
            match result {
                None => Ok(None),
                Some(IdentifierForAssignment::Global { name }) => {
                    let temp = lower_value_to_temporary(
                        builder,
                        InstructionValue::StoreGlobal { name, value, span: Some(span) },
                    )?;
                    Ok(Some(temp))
                }
                Some(IdentifierForAssignment::Place(place)) => {
                    if builder.is_context_identifier(builder.scope().resolve_binding_identifier(id))
                    {
                        let is_hoisted = builder
                            .scope()
                            .resolve_binding_identifier(id)
                            .map(|s| builder.environment().is_hoisted_identifier(s))
                            .unwrap_or(false);
                        if kind == InstructionKind::Const && !is_hoisted {
                            builder.record_error(
                                ErrorCategory::Syntax
                                    .diagnostic("Expected `const` declaration not to be reassigned")
                                    .with_label(span),
                            )?;
                        }
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreContext {
                                lvalue: LValue { place, kind },
                                value,
                                span: Some(span),
                            },
                        )?;
                        Ok(Some(temp))
                    } else {
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreLocal {
                                lvalue: LValue { place, kind },
                                value,
                                span: Some(span),
                            },
                        )?;
                        Ok(Some(temp))
                    }
                }
            }
        }
        oxc::BindingPattern::ArrayPattern(pattern) => {
            let mut items: Vec<ArrayPatternElement> = Vec::new();
            let mut followups: Vec<(Place, &oxc::BindingPattern)> = Vec::new();

            for element in &pattern.elements {
                match element {
                    None => {
                        items.push(ArrayPatternElement::Hole);
                    }
                    Some(oxc::BindingPattern::BindingIdentifier(id)) => {
                        let is_context = builder
                            .is_context_identifier(builder.scope().resolve_binding_identifier(id));
                        // force_temporaries is always false on the binding path.
                        let can_use_direct =
                            matches!(assignment_style, AssignmentStyle::Assignment) || !is_context;
                        if can_use_direct {
                            match lower_identifier_for_assignment(
                                builder,
                                id.span,
                                id.span,
                                kind,
                                id.name,
                                builder.scope().resolve_binding_identifier(id),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    items.push(ArrayPatternElement::Place(place));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    let temp = build_temporary_place(builder, Some(id.span));
                                    promote_temporary(builder, temp.identifier);
                                    items.push(ArrayPatternElement::Place(temp.clone()));
                                    followups.push((temp, element.as_ref().unwrap()));
                                }
                                None => {
                                    items.push(ArrayPatternElement::Hole);
                                }
                            }
                        } else {
                            let temp = build_temporary_place(builder, Some(id.span));
                            promote_temporary(builder, temp.identifier);
                            items.push(ArrayPatternElement::Place(temp.clone()));
                            followups.push((temp, element.as_ref().unwrap()));
                        }
                    }
                    Some(other) => {
                        let temp = build_temporary_place(builder, Some(other.span()));
                        promote_temporary(builder, temp.identifier);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((temp, other));
                    }
                }
            }

            if let Some(rest) = &pattern.rest {
                match &rest.argument {
                    oxc::BindingPattern::BindingIdentifier(id) => {
                        let is_context = builder
                            .is_context_identifier(builder.scope().resolve_binding_identifier(id));
                        let can_use_direct =
                            matches!(assignment_style, AssignmentStyle::Assignment) || !is_context;
                        if can_use_direct {
                            match lower_identifier_for_assignment(
                                builder,
                                rest.span,
                                id.span,
                                kind,
                                id.name,
                                builder.scope().resolve_binding_identifier(id),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    items
                                        .push(ArrayPatternElement::Spread(SpreadPattern { place }));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    let temp = build_temporary_place(builder, Some(rest.span));
                                    promote_temporary(builder, temp.identifier);
                                    items.push(ArrayPatternElement::Spread(SpreadPattern {
                                        place: temp.clone(),
                                    }));
                                    followups.push((temp, &rest.argument));
                                }
                                None => {}
                            }
                        } else {
                            let temp = build_temporary_place(builder, Some(rest.span));
                            promote_temporary(builder, temp.identifier);
                            items.push(ArrayPatternElement::Spread(SpreadPattern {
                                place: temp.clone(),
                            }));
                            followups.push((temp, &rest.argument));
                        }
                    }
                    _ => {
                        let temp = build_temporary_place(builder, Some(rest.span));
                        promote_temporary(builder, temp.identifier);
                        items.push(ArrayPatternElement::Spread(SpreadPattern {
                            place: temp.clone(),
                        }));
                        followups.push((temp, &rest.argument));
                    }
                }
            }

            let temporary = lower_value_to_temporary(
                builder,
                InstructionValue::Destructure {
                    lvalue: LValuePattern { pattern: Pattern::Array(ArrayPattern { items }), kind },
                    value,
                    span: Some(span),
                },
            )?;

            for (place, path) in followups {
                lower_binding_assignment(
                    builder,
                    path.span(),
                    kind,
                    path,
                    place,
                    assignment_style,
                )?;
            }
            Ok(Some(temporary))
        }
        oxc::BindingPattern::ObjectPattern(pattern) => {
            let mut properties: Vec<ObjectPropertyOrSpread> = Vec::new();
            let mut followups: Vec<(Place, &oxc::BindingPattern)> = Vec::new();

            for prop in &pattern.properties {
                if prop.computed {
                    builder.record_error(
                        ErrorCategory::Todo
                            .diagnostic("(BuildHIR::lowerAssignment) Handle computed properties in ObjectPattern").with_label(prop.span),
                    )?;
                    continue;
                }

                let key = match lower_object_property_key(builder, &prop.key, false)? {
                    Some(k) => k,
                    None => continue,
                };

                match &prop.value {
                    oxc::BindingPattern::BindingIdentifier(id) => {
                        let is_context = builder
                            .is_context_identifier(builder.scope().resolve_binding_identifier(id));
                        let can_use_direct =
                            matches!(assignment_style, AssignmentStyle::Assignment) || !is_context;
                        if can_use_direct {
                            match lower_identifier_for_assignment(
                                builder,
                                id.span,
                                id.span,
                                kind,
                                id.name,
                                builder.scope().resolve_binding_identifier(id),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    properties.push(ObjectPropertyOrSpread::Property(
                                        ObjectProperty {
                                            key,
                                            property_type: ObjectPropertyType::Property,
                                            place,
                                        },
                                    ));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    builder.record_error(
                                        ErrorCategory::Todo
                                            .diagnostic("Expected reassignment of globals to enable forceTemporaries").with_label(id.span),
                                    )?;
                                }
                                None => {
                                    continue;
                                }
                            }
                        } else {
                            let temp = build_temporary_place(builder, Some(id.span));
                            promote_temporary(builder, temp.identifier);
                            properties.push(ObjectPropertyOrSpread::Property(ObjectProperty {
                                key,
                                property_type: ObjectPropertyType::Property,
                                place: temp.clone(),
                            }));
                            followups.push((temp, &prop.value));
                        }
                    }
                    other => {
                        let temp = build_temporary_place(builder, Some(other.span()));
                        promote_temporary(builder, temp.identifier);
                        properties.push(ObjectPropertyOrSpread::Property(ObjectProperty {
                            key,
                            property_type: ObjectPropertyType::Property,
                            place: temp.clone(),
                        }));
                        followups.push((temp, other));
                    }
                }
            }

            if let Some(rest) = &pattern.rest {
                match &rest.argument {
                    oxc::BindingPattern::BindingIdentifier(id) => {
                        let is_context = builder
                            .is_context_identifier(builder.scope().resolve_binding_identifier(id));
                        let can_use_direct =
                            matches!(assignment_style, AssignmentStyle::Assignment) || !is_context;
                        if can_use_direct {
                            match lower_identifier_for_assignment(
                                builder,
                                rest.span,
                                id.span,
                                kind,
                                id.name,
                                builder.scope().resolve_binding_identifier(id),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    properties.push(ObjectPropertyOrSpread::Spread(
                                        SpreadPattern { place },
                                    ));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    builder.record_error(
                                        ErrorCategory::Todo
                                            .diagnostic("Expected reassignment of globals to enable forceTemporaries").with_label(rest.span),
                                    )?;
                                }
                                None => {}
                            }
                        } else {
                            let temp = build_temporary_place(builder, Some(rest.span));
                            promote_temporary(builder, temp.identifier);
                            properties.push(ObjectPropertyOrSpread::Spread(SpreadPattern {
                                place: temp.clone(),
                            }));
                            followups.push((temp, &rest.argument));
                        }
                    }
                    other => {
                        builder.record_error(
                            ErrorCategory::Todo
                                .diagnostic(format!(
                                    "(BuildHIR::lowerAssignment) Handle {} rest element in ObjectPattern",
                                    match other {
                                        oxc::BindingPattern::ObjectPattern(_) => "ObjectPattern",
                                        oxc::BindingPattern::ArrayPattern(_) => "ArrayPattern",
                                        oxc::BindingPattern::AssignmentPattern(_) => "AssignmentPattern",
                                        _ => "unknown",
                                    }
                                )).with_label(rest.span),
                        )?;
                    }
                }
            }

            let temporary = lower_value_to_temporary(
                builder,
                InstructionValue::Destructure {
                    lvalue: LValuePattern {
                        pattern: Pattern::Object(ObjectPattern { properties }),
                        kind,
                    },
                    value,
                    span: Some(span),
                },
            )?;

            for (place, path) in followups {
                lower_binding_assignment(
                    builder,
                    path.span(),
                    kind,
                    path,
                    place,
                    assignment_style,
                )?;
            }
            Ok(Some(temporary))
        }
        oxc::BindingPattern::AssignmentPattern(pattern) => {
            // Default value: if value === undefined, use default, else use value.
            let temp = lower_default_to_temp(builder, pattern.span, &pattern.right, value)?;
            // Recursively assign the resolved value to the left pattern.
            lower_binding_assignment(
                builder,
                pattern.span,
                kind,
                &pattern.left,
                temp,
                assignment_style,
            )
        }
    }
}

/// Lower the default-value ternary `value === undefined ? default : value` into a
/// fresh temporary and return it. Shared by the `AssignmentPattern` arm and by the
/// default-parameter (`FormalParameter.initializer`) lowering, which in Babel was a
/// single `AssignmentPattern` param node.
fn lower_default_to_temp<'a>(
    builder: &mut HirBuilder<'a, '_>,
    pat_span: Span,
    default: &oxc::Expression<'a>,
    value: Place,
) -> Result<Place, OxcDiagnostic> {
    let temp = build_temporary_place(builder, Some(pat_span));

    let test_block = builder.reserve(BlockKind::Value);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;

    let temp_consequent = temp.clone();
    let consequent = builder.try_enter(BlockKind::Value, |builder, _| {
        let default_value = lower_reorderable_expression(builder, default)?;
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { place: temp_consequent.clone(), kind: InstructionKind::Const },
                value: default_value,
                span: Some(pat_span),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder::UNSET,
            span: Some(pat_span),
        })
    });

    let temp_alternate = temp.clone();
    let value_alternate = value.clone();
    let alternate = builder.try_enter(BlockKind::Value, |builder, _| {
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { place: temp_alternate.clone(), kind: InstructionKind::Const },
                value: value_alternate.clone(),
                span: Some(pat_span),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder::UNSET,
            span: Some(pat_span),
        })
    });

    builder.terminate_with_continuation(
        Terminal::Ternary {
            test: test_block.id,
            fallthrough: continuation_id,
            id: EvaluationOrder::UNSET,
            span: Some(pat_span),
        },
        test_block,
    );

    let undef = lower_value_to_temporary(
        builder,
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, span: Some(pat_span) },
    )?;
    let test = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression {
            left: value,
            operator: BinaryOperator::StrictEquality,
            right: undef,
            span: Some(pat_span),
        },
    )?;
    builder.terminate_with_continuation(
        Terminal::Branch {
            test,
            consequent: consequent?,
            alternate: alternate?,
            fallthrough: continuation_id,
            id: EvaluationOrder::UNSET,
            span: Some(pat_span),
        },
        continuation_block,
    );

    Ok(temp)
}

/// Resolve a member-expression assignment target (oxc's member variants of
/// `SimpleAssignmentTarget`) and store `value` into it, returning the store
/// temporary. Mirrors the `PatternLike::MemberExpression` arm of the original
/// `lower_assignment`.
fn lower_member_assignment_target<'a>(
    builder: &mut HirBuilder<'a, '_>,
    span: Span,
    kind: InstructionKind,
    target: &oxc::SimpleAssignmentTarget<'a>,
    value: Place,
) -> Result<Option<Place>, OxcDiagnostic> {
    // MemberExpression may only appear in an assignment expression (Reassign).
    if kind != InstructionKind::Reassign {
        builder.record_error(
            ErrorCategory::Invariant
                .diagnostic("MemberExpression may only appear in an assignment expression")
                .with_label(span),
        )?;
        return Ok(None);
    }
    match target {
        oxc::SimpleAssignmentTarget::StaticMemberExpression(member) => {
            let object = lower_expression_to_temporary(builder, &member.object)?;
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::PropertyStore {
                    object,
                    property: PropertyLiteral::String(member.property.name),
                    value,
                    span: Some(span),
                },
            )?;
            Ok(Some(temp))
        }
        oxc::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            let object = lower_expression_to_temporary(builder, &member.object)?;
            // A numeric computed index is treated as a PropertyStore (matches the
            // original `member.computed && NumericLiteral` branch).
            if let oxc::Expression::NumericLiteral(num) = &member.expression {
                let temp = lower_value_to_temporary(
                    builder,
                    InstructionValue::PropertyStore {
                        object,
                        property: PropertyLiteral::Number(FloatValue::new(num.value)),
                        value,
                        span: Some(span),
                    },
                )?;
                return Ok(Some(temp));
            }
            let property_place = lower_expression_to_temporary(builder, &member.expression)?;
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::ComputedStore {
                    object,
                    property: property_place,
                    value,
                    span: Some(span),
                },
            )?;
            Ok(Some(temp))
        }
        oxc::SimpleAssignmentTarget::PrivateFieldExpression(member) => {
            // Babel modeled `a.#b = v` as a non-computed MemberExpression with a
            // PrivateName property; the original `lower_assignment` member arm hit
            // the generic property `_` branch and bailed with this Todo.
            lower_expression_to_temporary(builder, &member.object)?;
            builder.record_error(
                ErrorCategory::Todo
                    .diagnostic("(BuildHIR::lowerAssignment) Handle PrivateName properties in MemberExpression").with_label(member.field.span),
            )?;
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::Primitive { value: PrimitiveValue::Undefined, span: Some(span) },
            )?;
            Ok(Some(temp))
        }
        _ => unreachable!("lower_member_assignment_target called on a non-member target"),
    }
}

/// True if `maybe` is a bare identifier assignment target that resolves to a local
/// binding (used to compute `force_temporaries`).
fn assignment_target_is_local_identifier<'a>(
    builder: &mut HirBuilder<'a, '_>,
    maybe: &oxc::AssignmentTargetMaybeDefault<'a>,
) -> Result<bool, OxcDiagnostic> {
    match maybe {
        oxc::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
            if builder.is_context_identifier(builder.scope().resolve_reference(id)) {
                return Ok(false);
            }
            let symbol = builder.scope().resolve_reference(id);
            match builder.resolve_identifier(id.name, id.span, symbol)? {
                VariableBinding::Identifier { .. } => Ok(true),
                _ => Ok(false),
            }
        }
        _ => Ok(false),
    }
}

/// Assign `value` to an assignment-expression target (`x`, `a.b`, `[a, b]`,
/// `{a, b}`). Faithful translation of the original `lower_assignment` for the
/// `PatternLike` cases that came from `AssignmentExpression.left` / destructuring
/// targets; oxc models these as `AssignmentTarget`.
fn lower_assignment_target<'a>(
    builder: &mut HirBuilder<'a, '_>,
    span: Span,
    kind: InstructionKind,
    target: &oxc::AssignmentTarget<'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, OxcDiagnostic> {
    match target {
        oxc::AssignmentTarget::AssignmentTargetIdentifier(id) => {
            let result = lower_identifier_for_assignment(
                builder,
                span,
                id.span,
                kind,
                id.name,
                builder.scope().resolve_reference(id),
            )?;
            match result {
                None => Ok(None),
                Some(IdentifierForAssignment::Global { name }) => {
                    let temp = lower_value_to_temporary(
                        builder,
                        InstructionValue::StoreGlobal { name, value, span: Some(span) },
                    )?;
                    Ok(Some(temp))
                }
                Some(IdentifierForAssignment::Place(place)) => {
                    if builder.is_context_identifier(builder.scope().resolve_reference(id)) {
                        let is_hoisted = builder
                            .scope()
                            .resolve_reference(id)
                            .map(|s| builder.environment().is_hoisted_identifier(s))
                            .unwrap_or(false);
                        if kind == InstructionKind::Const && !is_hoisted {
                            builder.record_error(
                                ErrorCategory::Syntax
                                    .diagnostic("Expected `const` declaration not to be reassigned")
                                    .with_label(span),
                            )?;
                        }
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreContext {
                                lvalue: LValue { place, kind },
                                value,
                                span: Some(span),
                            },
                        )?;
                        Ok(Some(temp))
                    } else {
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreLocal {
                                lvalue: LValue { place, kind },
                                value,
                                span: Some(span),
                            },
                        )?;
                        Ok(Some(temp))
                    }
                }
            }
        }
        oxc::AssignmentTarget::StaticMemberExpression(_)
        | oxc::AssignmentTarget::ComputedMemberExpression(_)
        | oxc::AssignmentTarget::PrivateFieldExpression(_) => {
            let simple = target.as_simple_assignment_target().unwrap();
            lower_member_assignment_target(builder, span, kind, simple, value)
        }
        oxc::AssignmentTarget::ArrayAssignmentTarget(pattern) => {
            let mut items: Vec<ArrayPatternElement> = Vec::new();
            let mut followups: Vec<(Place, FollowupTarget)> = Vec::new();

            // force_temporaries: when kind is Reassign and any element is
            // non-identifier, a context variable, or a non-local binding.
            let force_temporaries = if kind == InstructionKind::Reassign {
                let mut found = false;
                if pattern.rest.is_some() {
                    found = true;
                }
                if !found {
                    for elem in &pattern.elements {
                        match elem {
                            Some(maybe) => {
                                if !assignment_target_is_local_identifier(builder, maybe)? {
                                    found = true;
                                    break;
                                }
                            }
                            None => {
                                found = true;
                                break;
                            }
                        }
                    }
                }
                found
            } else {
                false
            };

            for element in &pattern.elements {
                match element {
                    None => {
                        items.push(ArrayPatternElement::Hole);
                    }
                    Some(oxc::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id)) => {
                        let is_context =
                            builder.is_context_identifier(builder.scope().resolve_reference(id));
                        let can_use_direct = !force_temporaries
                            && (matches!(assignment_style, AssignmentStyle::Assignment)
                                || !is_context);
                        if can_use_direct {
                            match lower_identifier_for_assignment(
                                builder,
                                id.span,
                                id.span,
                                kind,
                                id.name,
                                builder.scope().resolve_reference(id),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    items.push(ArrayPatternElement::Place(place));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    let temp = build_temporary_place(builder, Some(id.span));
                                    promote_temporary(builder, temp.identifier);
                                    items.push(ArrayPatternElement::Place(temp.clone()));
                                    followups.push((
                                        temp,
                                        FollowupTarget::MaybeDefault(element.as_ref().unwrap()),
                                    ));
                                }
                                None => {
                                    items.push(ArrayPatternElement::Hole);
                                }
                            }
                        } else {
                            let temp = build_temporary_place(builder, Some(id.span));
                            promote_temporary(builder, temp.identifier);
                            items.push(ArrayPatternElement::Place(temp.clone()));
                            followups.push((
                                temp,
                                FollowupTarget::MaybeDefault(element.as_ref().unwrap()),
                            ));
                        }
                    }
                    Some(other) => {
                        let temp = build_temporary_place(builder, Some(other.span()));
                        promote_temporary(builder, temp.identifier);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((temp, FollowupTarget::MaybeDefault(other)));
                    }
                }
            }

            if let Some(rest) = &pattern.rest {
                match &rest.target {
                    oxc::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                        let is_context =
                            builder.is_context_identifier(builder.scope().resolve_reference(id));
                        let can_use_direct = !force_temporaries
                            && (matches!(assignment_style, AssignmentStyle::Assignment)
                                || !is_context);
                        if can_use_direct {
                            match lower_identifier_for_assignment(
                                builder,
                                rest.span,
                                id.span,
                                kind,
                                id.name,
                                builder.scope().resolve_reference(id),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    items
                                        .push(ArrayPatternElement::Spread(SpreadPattern { place }));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    let temp = build_temporary_place(builder, Some(rest.span));
                                    promote_temporary(builder, temp.identifier);
                                    items.push(ArrayPatternElement::Spread(SpreadPattern {
                                        place: temp.clone(),
                                    }));
                                    followups.push((temp, FollowupTarget::Target(&rest.target)));
                                }
                                None => {}
                            }
                        } else {
                            let temp = build_temporary_place(builder, Some(rest.span));
                            promote_temporary(builder, temp.identifier);
                            items.push(ArrayPatternElement::Spread(SpreadPattern {
                                place: temp.clone(),
                            }));
                            followups.push((temp, FollowupTarget::Target(&rest.target)));
                        }
                    }
                    _ => {
                        let temp = build_temporary_place(builder, Some(rest.span));
                        promote_temporary(builder, temp.identifier);
                        items.push(ArrayPatternElement::Spread(SpreadPattern {
                            place: temp.clone(),
                        }));
                        followups.push((temp, FollowupTarget::Target(&rest.target)));
                    }
                }
            }

            let temporary = lower_value_to_temporary(
                builder,
                InstructionValue::Destructure {
                    lvalue: LValuePattern { pattern: Pattern::Array(ArrayPattern { items }), kind },
                    value,
                    span: Some(span),
                },
            )?;

            for (place, path) in followups {
                lower_followup_target(builder, kind, path, place, assignment_style)?;
            }
            Ok(Some(temporary))
        }
        oxc::AssignmentTarget::ObjectAssignmentTarget(pattern) => {
            let mut properties: Vec<ObjectPropertyOrSpread> = Vec::new();
            let mut followups: Vec<(Place, FollowupTarget)> = Vec::new();

            let force_temporaries = if kind == InstructionKind::Reassign {
                let mut found = false;
                if pattern.rest.is_some() {
                    found = true;
                }
                if !found {
                    for prop in &pattern.properties {
                        match prop {
                            oxc::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                                p,
                            ) => {
                                // `{foo}` (init None) is a bare identifier; `{foo = d}`
                                // (init Some) is an AssignmentPattern in Babel terms.
                                if p.init.is_some() {
                                    found = true;
                                    break;
                                }
                                let symbol = builder.scope().resolve_reference(&p.binding);
                                match builder.resolve_identifier(
                                    p.binding.name,
                                    p.binding.span,
                                    symbol,
                                )? {
                                    VariableBinding::Identifier { .. } => {}
                                    _ => {
                                        found = true;
                                        break;
                                    }
                                }
                            }
                            oxc::AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
                                if !assignment_target_is_local_identifier(builder, &p.binding)? {
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                found
            } else {
                false
            };

            for prop in &pattern.properties {
                match prop {
                    oxc::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(p) => {
                        let key = ObjectPropertyKey::Identifier { name: p.binding.name };
                        let id = &p.binding;
                        if let Some(default) = &p.init {
                            // `{foo = d}` — Babel shorthand AssignmentPattern. Lower
                            // via a promoted temporary + default followup.
                            let temp = build_temporary_place(builder, Some(p.span));
                            promote_temporary(builder, temp.identifier);
                            properties.push(ObjectPropertyOrSpread::Property(ObjectProperty {
                                key,
                                property_type: ObjectPropertyType::Property,
                                place: temp.clone(),
                            }));
                            followups.push((
                                temp,
                                FollowupTarget::Default {
                                    span: p.span,
                                    default,
                                    binding: FollowupBinding::Identifier(id),
                                },
                            ));
                            continue;
                        }
                        let is_context =
                            builder.is_context_identifier(builder.scope().resolve_reference(id));
                        let can_use_direct = !force_temporaries
                            && (matches!(assignment_style, AssignmentStyle::Assignment)
                                || !is_context);
                        if can_use_direct {
                            match lower_identifier_for_assignment(
                                builder,
                                id.span,
                                id.span,
                                kind,
                                id.name,
                                builder.scope().resolve_reference(id),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    properties.push(ObjectPropertyOrSpread::Property(
                                        ObjectProperty {
                                            key,
                                            property_type: ObjectPropertyType::Property,
                                            place,
                                        },
                                    ));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    builder.record_error(
                                        ErrorCategory::Todo
                                            .diagnostic("Expected reassignment of globals to enable forceTemporaries").with_label(id.span),
                                    )?;
                                }
                                None => {
                                    continue;
                                }
                            }
                        } else {
                            let temp = build_temporary_place(builder, Some(id.span));
                            promote_temporary(builder, temp.identifier);
                            properties.push(ObjectPropertyOrSpread::Property(ObjectProperty {
                                key,
                                property_type: ObjectPropertyType::Property,
                                place: temp.clone(),
                            }));
                            followups.push((temp, FollowupTarget::Identifier(id)));
                        }
                    }
                    oxc::AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
                        if p.computed {
                            builder.record_error(
                                ErrorCategory::Todo
                                    .diagnostic("(BuildHIR::lowerAssignment) Handle computed properties in ObjectPattern").with_label(p.span),
                            )?;
                            continue;
                        }
                        let key = match lower_object_property_key(builder, &p.name, false)? {
                            Some(k) => k,
                            None => continue,
                        };
                        match &p.binding {
                            oxc::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
                                let is_context = builder
                                    .is_context_identifier(builder.scope().resolve_reference(id));
                                let can_use_direct = !force_temporaries
                                    && (matches!(assignment_style, AssignmentStyle::Assignment)
                                        || !is_context);
                                if can_use_direct {
                                    match lower_identifier_for_assignment(
                                        builder,
                                        id.span,
                                        id.span,
                                        kind,
                                        id.name,
                                        builder.scope().resolve_reference(id),
                                    )? {
                                        Some(IdentifierForAssignment::Place(place)) => {
                                            properties.push(ObjectPropertyOrSpread::Property(
                                                ObjectProperty {
                                                    key,
                                                    property_type: ObjectPropertyType::Property,
                                                    place,
                                                },
                                            ));
                                        }
                                        Some(IdentifierForAssignment::Global { .. }) => {
                                            builder.record_error(
                                                ErrorCategory::Todo
                                                    .diagnostic("Expected reassignment of globals to enable forceTemporaries").with_label(id.span),
                                            )?;
                                        }
                                        None => {
                                            continue;
                                        }
                                    }
                                } else {
                                    let temp = build_temporary_place(builder, Some(id.span));
                                    promote_temporary(builder, temp.identifier);
                                    properties.push(ObjectPropertyOrSpread::Property(
                                        ObjectProperty {
                                            key,
                                            property_type: ObjectPropertyType::Property,
                                            place: temp.clone(),
                                        },
                                    ));
                                    followups
                                        .push((temp, FollowupTarget::MaybeDefault(&p.binding)));
                                }
                            }
                            other => {
                                let temp = build_temporary_place(builder, Some(other.span()));
                                promote_temporary(builder, temp.identifier);
                                properties.push(ObjectPropertyOrSpread::Property(ObjectProperty {
                                    key,
                                    property_type: ObjectPropertyType::Property,
                                    place: temp.clone(),
                                }));
                                followups.push((temp, FollowupTarget::MaybeDefault(other)));
                            }
                        }
                    }
                }
            }

            if let Some(rest) = &pattern.rest {
                match &rest.target {
                    oxc::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                        let is_context =
                            builder.is_context_identifier(builder.scope().resolve_reference(id));
                        let can_use_direct = !force_temporaries
                            && (matches!(assignment_style, AssignmentStyle::Assignment)
                                || !is_context);
                        if can_use_direct {
                            match lower_identifier_for_assignment(
                                builder,
                                rest.span,
                                id.span,
                                kind,
                                id.name,
                                builder.scope().resolve_reference(id),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    properties.push(ObjectPropertyOrSpread::Spread(
                                        SpreadPattern { place },
                                    ));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    builder.record_error(
                                        ErrorCategory::Todo
                                            .diagnostic("Expected reassignment of globals to enable forceTemporaries").with_label(rest.span),
                                    )?;
                                }
                                None => {}
                            }
                        } else {
                            let temp = build_temporary_place(builder, Some(rest.span));
                            promote_temporary(builder, temp.identifier);
                            properties.push(ObjectPropertyOrSpread::Spread(SpreadPattern {
                                place: temp.clone(),
                            }));
                            followups.push((temp, FollowupTarget::Target(&rest.target)));
                        }
                    }
                    other => {
                        builder.record_error(
                            ErrorCategory::Todo
                                .diagnostic(format!(
                                    "(BuildHIR::lowerAssignment) Handle {} rest element in ObjectPattern",
                                    match other {
                                        oxc::AssignmentTarget::ObjectAssignmentTarget(_) => {
                                            "ObjectPattern"
                                        }
                                        oxc::AssignmentTarget::ArrayAssignmentTarget(_) => "ArrayPattern",
                                        oxc::AssignmentTarget::StaticMemberExpression(_)
                                        | oxc::AssignmentTarget::ComputedMemberExpression(_)
                                        | oxc::AssignmentTarget::PrivateFieldExpression(_) => {
                                            "MemberExpression"
                                        }
                                        _ => "unknown",
                                    }
                                )).with_label(rest.span),
                        )?;
                    }
                }
            }

            let temporary = lower_value_to_temporary(
                builder,
                InstructionValue::Destructure {
                    lvalue: LValuePattern {
                        pattern: Pattern::Object(ObjectPattern { properties }),
                        kind,
                    },
                    value,
                    span: Some(span),
                },
            )?;

            for (place, path) in followups {
                lower_followup_target(builder, kind, path, place, assignment_style)?;
            }
            Ok(Some(temporary))
        }
        // TS assignment-target wrappers (e.g. `(x as T) = ...`). The original
        // recorded the TS-faithful Todo once in find_context_identifiers and
        // returned None here.
        oxc::AssignmentTarget::TSAsExpression(_)
        | oxc::AssignmentTarget::TSSatisfiesExpression(_)
        | oxc::AssignmentTarget::TSNonNullExpression(_)
        | oxc::AssignmentTarget::TSTypeAssertion(_) => Ok(None),
    }
}

/// A destructuring followup target: either a nested assignment target, a
/// with-default wrapper element, or (for `{foo = d}` object shorthand) an
/// identifier with a default expression.
enum FollowupTarget<'b, 'a> {
    Target(&'b oxc::AssignmentTarget<'a>),
    MaybeDefault(&'b oxc::AssignmentTargetMaybeDefault<'a>),
    /// A bare `{foo}` shorthand object property binding that needs a promoted
    /// temporary followup (the Babel `obj_prop.value == Identifier` case).
    Identifier(&'b oxc::IdentifierReference<'a>),
    Default {
        span: oxc_span::Span,
        default: &'b oxc::Expression<'a>,
        binding: FollowupBinding<'b, 'a>,
    },
}

enum FollowupBinding<'b, 'a> {
    Identifier(&'b oxc::IdentifierReference<'a>),
    Target(&'b oxc::AssignmentTarget<'a>),
}

/// Store `value` into the identifier-target `id` (a bare destructuring binding).
/// Mirrors the `PatternLike::Identifier` followup path of the original
/// `lower_assignment` (re-resolving the binding for the store).
fn lower_identifier_followup_store<'a>(
    builder: &mut HirBuilder<'a, '_>,
    span: Span,
    kind: InstructionKind,
    id: &oxc::IdentifierReference<'a>,
    value: Place,
) -> Result<Option<Place>, OxcDiagnostic> {
    let result = lower_identifier_for_assignment(
        builder,
        span,
        id.span,
        kind,
        id.name,
        builder.scope().resolve_reference(id),
    )?;
    match result {
        None => Ok(None),
        Some(IdentifierForAssignment::Global { name }) => {
            let t = lower_value_to_temporary(
                builder,
                InstructionValue::StoreGlobal { name, value, span: Some(span) },
            )?;
            Ok(Some(t))
        }
        Some(IdentifierForAssignment::Place(place)) => {
            if builder.is_context_identifier(builder.scope().resolve_reference(id)) {
                let t = lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreContext {
                        lvalue: LValue { place, kind },
                        value,
                        span: Some(span),
                    },
                )?;
                Ok(Some(t))
            } else {
                let t = lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { place, kind },
                        value,
                        span: Some(span),
                    },
                )?;
                Ok(Some(t))
            }
        }
    }
}

/// Lower a single destructuring followup (the recursion step shared by
/// `lower_assignment_target`).
fn lower_followup_target<'a>(
    builder: &mut HirBuilder<'a, '_>,
    kind: InstructionKind,
    target: FollowupTarget<'_, 'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, OxcDiagnostic> {
    match target {
        FollowupTarget::Target(t) => {
            lower_assignment_target(builder, t.span(), kind, t, value, assignment_style)
        }
        FollowupTarget::MaybeDefault(m) => {
            lower_assignment_target_maybe_default(builder, kind, m, value, assignment_style)
        }
        FollowupTarget::Identifier(id) => {
            lower_identifier_followup_store(builder, id.span, kind, id, value)
        }
        FollowupTarget::Default { span, default, binding } => lower_assignment_target_default(
            builder,
            span,
            kind,
            default,
            binding,
            value,
            assignment_style,
        ),
    }
}

/// Lower an `AssignmentTargetMaybeDefault` (array element / object property
/// binding). The with-default wrapper (`[a = 1]`, `{a: b = 1}`) is the Babel
/// `AssignmentPattern` case.
fn lower_assignment_target_maybe_default<'a>(
    builder: &mut HirBuilder<'a, '_>,
    kind: InstructionKind,
    maybe: &oxc::AssignmentTargetMaybeDefault<'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, OxcDiagnostic> {
    match maybe {
        oxc::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
            lower_assignment_target_default(
                builder,
                with_default.span,
                kind,
                &with_default.init,
                FollowupBinding::Target(&with_default.binding),
                value,
                assignment_style,
            )
        }
        _ => {
            let target = maybe.as_assignment_target().unwrap();
            lower_assignment_target(builder, target.span(), kind, target, value, assignment_style)
        }
    }
}

/// Lower a default-value assignment target (`AssignmentPattern`): if `value ===
/// undefined`, use the default, else use `value`, then assign the result into the
/// inner binding. Faithful translation of the `PatternLike::AssignmentPattern` arm.
fn lower_assignment_target_default<'a>(
    builder: &mut HirBuilder<'a, '_>,
    span: oxc_span::Span,
    kind: InstructionKind,
    default: &oxc::Expression<'a>,
    binding: FollowupBinding<'_, 'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, OxcDiagnostic> {
    let temp = build_temporary_place(builder, Some(span));

    let test_block = builder.reserve(BlockKind::Value);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;

    let temp_consequent = temp.clone();
    let consequent = builder.try_enter(BlockKind::Value, |builder, _| {
        let default_value = lower_reorderable_expression(builder, default)?;
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { place: temp_consequent.clone(), kind: InstructionKind::Const },
                value: default_value,
                span: Some(span),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder::UNSET,
            span: Some(span),
        })
    });

    let temp_alternate = temp.clone();
    let value_alternate = value.clone();
    let alternate = builder.try_enter(BlockKind::Value, |builder, _| {
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { place: temp_alternate.clone(), kind: InstructionKind::Const },
                value: value_alternate.clone(),
                span: Some(span),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder::UNSET,
            span: Some(span),
        })
    });

    builder.terminate_with_continuation(
        Terminal::Ternary {
            test: test_block.id,
            fallthrough: continuation_id,
            id: EvaluationOrder::UNSET,
            span: Some(span),
        },
        test_block,
    );

    let undef = lower_value_to_temporary(
        builder,
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, span: Some(span) },
    )?;
    let test = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression {
            left: value,
            operator: BinaryOperator::StrictEquality,
            right: undef,
            span: Some(span),
        },
    )?;
    builder.terminate_with_continuation(
        Terminal::Branch {
            test,
            consequent: consequent?,
            alternate: alternate?,
            fallthrough: continuation_id,
            id: EvaluationOrder::UNSET,
            span: Some(span),
        },
        continuation_block,
    );

    // Recursively assign the resolved value to the inner binding.
    match binding {
        FollowupBinding::Target(t) => {
            lower_assignment_target(builder, span, kind, t, temp, assignment_style)
        }
        FollowupBinding::Identifier(id) => {
            // `{foo = d}` shorthand: the binding is the identifier `foo` itself.
            lower_identifier_followup_store(builder, span, kind, id, temp)
        }
    }
}

/// True if any node in the receiver spine carries `optional == true`.
///
/// Mirrors `convert_ast::expr_contains_optional`: in oxc a chain like `a?.b.c` is
/// a single `ChainExpression` where each member/call carries its own `.optional`
/// flag, so a node "is in optional context" iff itself or anything deeper in its
/// receiver spine is optional. This is the predicate Babel used to decide whether
/// a node became `Optional{Member,Call}Expression` (vs a plain member/call).
fn expr_contains_optional(expr: &oxc::Expression) -> bool {
    match expr {
        oxc::Expression::CallExpression(c) => c.optional || expr_contains_optional(&c.callee),
        oxc::Expression::StaticMemberExpression(m) => {
            m.optional || expr_contains_optional(&m.object)
        }
        oxc::Expression::ComputedMemberExpression(m) => {
            m.optional || expr_contains_optional(&m.object)
        }
        oxc::Expression::PrivateFieldExpression(p) => {
            p.optional || expr_contains_optional(&p.object)
        }
        _ => false,
    }
}

/// Lower an oxc `ChainExpression` (`a?.b?.c()` etc.). oxc represents the whole
/// optional chain as one node wrapping nested member/call nodes carrying per-node
/// `.optional` flags; Babel instead split each link into
/// `Optional{Member,Call}Expression`. This fuses `convert_chain_expression` with
/// the original `lower_optional_member_expression` / `lower_optional_call_expression`
/// dispatch, reproducing the same `Optional` terminal / `OptionalCall`-`OptionalLoad`
/// HIR structure.
fn lower_chain_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    chain: &oxc::ChainExpression<'a>,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    match &chain.expression {
        oxc::ChainElement::CallExpression(call) => {
            lower_optional_call_expression_impl(builder, call, None)
        }
        oxc::ChainElement::TSNonNullExpression(ts) => {
            // `foo?.bar!` — the non-null assertion wraps a chain-context expression.
            // The original lowered `TSNonNullExpression` by recursing into its inner
            // expression (span-transparent); preserve that, keeping chain awareness.
            lower_chain_subexpr(builder, &ts.expression)
        }
        // The `@inherit MemberExpression` variants of `ChainElement`.
        oxc::ChainElement::StaticMemberExpression(_)
        | oxc::ChainElement::ComputedMemberExpression(_)
        | oxc::ChainElement::PrivateFieldExpression(_) => {
            let member = chain.expression.as_member_expression().unwrap();
            let place = lower_optional_member_expression_impl(builder, member, None)?.1;
            Ok(InstructionValue::LoadLocal { span: place.span, place })
        }
    }
}

/// Lower an expression that appears as a callee/object inside a chain (or wrapped
/// by a chain-context `TSNonNullExpression`), as an `InstructionValue`.
///
/// Faithful to the original: Babel's regular `lower_expression` routed
/// `OptionalMemberExpression`/`OptionalCallExpression` into the optional impls and
/// `TSNonNullExpression` by recursing into its inner expression, while everything
/// else went through normal lowering. The oxc equivalent uses `expr_contains_optional`
/// to detect optional-context member/call nodes.
fn lower_chain_subexpr<'a>(
    builder: &mut HirBuilder<'a, '_>,
    expr: &oxc::Expression<'a>,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    match expr {
        oxc::Expression::StaticMemberExpression(_)
        | oxc::Expression::ComputedMemberExpression(_)
        | oxc::Expression::PrivateFieldExpression(_)
            if expr_contains_optional(expr) =>
        {
            let member = expr.as_member_expression().unwrap();
            let place = lower_optional_member_expression_impl(builder, member, None)?.1;
            Ok(InstructionValue::LoadLocal { span: place.span, place })
        }
        oxc::Expression::CallExpression(call) if expr_contains_optional(expr) => {
            lower_optional_call_expression_impl(builder, call, None)
        }
        oxc::Expression::TSNonNullExpression(ts) => lower_chain_subexpr(builder, &ts.expression),
        _ => lower_expression(builder, expr),
    }
}

/// Returns `(object, value_place)`. The `value_place` holds the result temporary;
/// the top-level caller wraps it in `LoadLocal`. `member` is one of the three oxc
/// member variants. `parent_alternate` threads the shared null/undefined block so a
/// chain only creates one alternate at the first `?.`.
fn lower_optional_member_expression_impl<'a>(
    builder: &mut HirBuilder<'a, '_>,
    member: &oxc::MemberExpression<'a>,
    parent_alternate: Option<BlockId>,
) -> Result<(Place, Place), OxcDiagnostic> {
    let optional = member.optional();
    let span = Some(member.span());
    let place = build_temporary_place(builder, span);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;
    let consequent = builder.reserve(BlockKind::Value);

    // Block to evaluate if the receiver is null/undefined — sets result to undefined.
    // Only create an alternate when first entering an optional subtree.
    let alternate = if let Some(parent_alt) = parent_alternate {
        Ok(parent_alt)
    } else {
        builder.try_enter(BlockKind::Value, |builder, _block_id| {
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::Primitive { value: PrimitiveValue::Undefined, span },
            )?;
            lower_value_to_temporary(
                builder,
                InstructionValue::StoreLocal {
                    lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                    value: temp,
                    span,
                },
            )?;
            Ok(Terminal::Goto {
                block: continuation_id,
                variant: GotoVariant::Break,
                id: EvaluationOrder::UNSET,
                span,
            })
        })
    }?;

    let object_expr = member.object();
    let mut object: Option<Place> = None;
    let test_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
        match object_expr {
            oxc::Expression::StaticMemberExpression(_)
            | oxc::Expression::ComputedMemberExpression(_)
            | oxc::Expression::PrivateFieldExpression(_)
                if expr_contains_optional(object_expr) =>
            {
                let object_member = object_expr.as_member_expression().unwrap();
                let (_obj, value) =
                    lower_optional_member_expression_impl(builder, object_member, Some(alternate))?;
                object = Some(value);
            }
            oxc::Expression::CallExpression(opt_call) if expr_contains_optional(object_expr) => {
                let value =
                    lower_optional_call_expression_impl(builder, opt_call, Some(alternate))?;
                let value_place = lower_value_to_temporary(builder, value)?;
                object = Some(value_place);
            }
            other => {
                object = Some(lower_expression_to_temporary(builder, other)?);
            }
        }
        let test_place = object.as_ref().unwrap().clone();
        Ok(Terminal::Branch {
            test: test_place,
            consequent: consequent.id,
            alternate,
            fallthrough: continuation_id,
            id: EvaluationOrder::UNSET,
            span,
        })
    });

    let obj = object.unwrap();

    // Block to evaluate if the receiver is non-null/undefined.
    builder.try_enter_reserved(consequent, |builder| {
        let lowered = lower_member_expression_impl(builder, member, Some(obj.clone()))?;
        let temp = lower_value_to_temporary(builder, lowered.value)?;
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                value: temp,
                span,
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder::UNSET,
            span,
        })
    })?;

    builder.terminate_with_continuation(
        Terminal::Optional {
            optional,
            test: test_block?,
            fallthrough: continuation_id,
            id: EvaluationOrder::UNSET,
            span,
        },
        continuation_block,
    );

    Ok((obj, place))
}

/// Lower an oxc optional `CallExpression` (a call link inside a `ChainExpression`).
/// `parent_alternate` threads the shared null/undefined block.
fn lower_optional_call_expression_impl<'a>(
    builder: &mut HirBuilder<'a, '_>,
    call: &oxc::CallExpression<'a>,
    parent_alternate: Option<BlockId>,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    let span = Some(call.span);
    let place = build_temporary_place(builder, span);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;
    let consequent = builder.reserve(BlockKind::Value);

    let alternate = if let Some(parent_alt) = parent_alternate {
        Ok(parent_alt)
    } else {
        builder.try_enter(BlockKind::Value, |builder, _block_id| {
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::Primitive { value: PrimitiveValue::Undefined, span },
            )?;
            lower_value_to_temporary(
                builder,
                InstructionValue::StoreLocal {
                    lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                    value: temp,
                    span,
                },
            )?;
            Ok(Terminal::Goto {
                block: continuation_id,
                variant: GotoVariant::Break,
                id: EvaluationOrder::UNSET,
                span,
            })
        })
    }?;

    // Track callee info for building the call in the consequent block.
    enum CalleeInfo {
        CallExpression { callee: Place },
        MethodCall { receiver: Place, property: Place },
    }

    let mut callee_info: Option<CalleeInfo> = None;

    let test_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
        match &call.callee {
            oxc::Expression::CallExpression(opt_call) if expr_contains_optional(&call.callee) => {
                let value =
                    lower_optional_call_expression_impl(builder, opt_call, Some(alternate))?;
                let value_place = lower_value_to_temporary(builder, value)?;
                callee_info = Some(CalleeInfo::CallExpression { callee: value_place });
            }
            oxc::Expression::StaticMemberExpression(_)
            | oxc::Expression::ComputedMemberExpression(_)
            | oxc::Expression::PrivateFieldExpression(_)
                if expr_contains_optional(&call.callee) =>
            {
                let callee_member = call.callee.as_member_expression().unwrap();
                let (obj, value) =
                    lower_optional_member_expression_impl(builder, callee_member, Some(alternate))?;
                callee_info = Some(CalleeInfo::MethodCall { receiver: obj, property: value });
            }
            oxc::Expression::StaticMemberExpression(_)
            | oxc::Expression::ComputedMemberExpression(_)
            | oxc::Expression::PrivateFieldExpression(_) => {
                let callee_member = call.callee.as_member_expression().unwrap();
                let lowered = lower_member_expression(builder, callee_member)?;
                let property_place = lower_value_to_temporary(builder, lowered.value)?;
                callee_info = Some(CalleeInfo::MethodCall {
                    receiver: lowered.object,
                    property: property_place,
                });
            }
            other => {
                let callee_place = lower_expression_to_temporary(builder, other)?;
                callee_info = Some(CalleeInfo::CallExpression { callee: callee_place });
            }
        }

        let test_place = match callee_info.as_ref().unwrap() {
            CalleeInfo::CallExpression { callee } => callee.clone(),
            CalleeInfo::MethodCall { property, .. } => property.clone(),
        };

        Ok(Terminal::Branch {
            test: test_place,
            consequent: consequent.id,
            alternate,
            fallthrough: continuation_id,
            id: EvaluationOrder::UNSET,
            span,
        })
    });

    // Block to evaluate if the callee is non-null/undefined.
    builder.try_enter_reserved(consequent, |builder| {
        let args = lower_arguments(builder, &call.arguments)?;
        let temp = build_temporary_place(builder, span);

        match callee_info.as_ref().unwrap() {
            CalleeInfo::CallExpression { callee } => {
                builder.push(Instruction {
                    id: EvaluationOrder::UNSET,
                    lvalue: temp.clone(),
                    value: InstructionValue::CallExpression { callee: callee.clone(), args, span },
                    span,
                    effects: None,
                });
            }
            CalleeInfo::MethodCall { receiver, property } => {
                builder.push(Instruction {
                    id: EvaluationOrder::UNSET,
                    lvalue: temp.clone(),
                    value: InstructionValue::MethodCall {
                        receiver: receiver.clone(),
                        property: property.clone(),
                        args,
                        span,
                    },
                    span,
                    effects: None,
                });
            }
        }

        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                value: temp,
                span,
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder::UNSET,
            span,
        })
    })?;

    builder.terminate_with_continuation(
        Terminal::Optional {
            optional: call.optional,
            test: test_block?,
            fallthrough: continuation_id,
            id: EvaluationOrder::UNSET,
            span,
        },
        continuation_block,
    );

    Ok(InstructionValue::LoadLocal { place: place.clone(), span: place.span })
}

// =============================================================================
// Function / arrow lowering
// =============================================================================

/// Lower a function/arrow expression to a `FunctionExpression` instruction value.
/// Mirrors the original `lower_function_to_value`.
fn lower_function_to_value<'a>(
    builder: &mut HirBuilder<'a, '_>,
    func: FunctionNode<'_, 'a>,
    expr_type: FunctionExpressionType,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    let span = match func {
        FunctionNode::Arrow(arrow) => Some(arrow.span),
        FunctionNode::Function(f) => Some(f.span),
    };
    let name = match func {
        FunctionNode::Function(f) => f.id.as_ref().map(|id| id.name),
        FunctionNode::Arrow(_) => None,
    };
    let lowered_func = lower_function(builder, func)?;
    Ok(InstructionValue::FunctionExpression {
        name,
        name_hint: None,
        lowered_func,
        expr_type,
        span,
    })
}

/// Lower a nested function/arrow node into a `LoweredFunction`. Mirrors the
/// original `lower_function`.
fn lower_function<'a>(
    builder: &mut HirBuilder<'a, '_>,
    func: FunctionNode<'_, 'a>,
) -> Result<LoweredFunction, OxcDiagnostic> {
    // Extract function parts from the AST node
    let (params, body, id, generator, is_async, func_span) = match func {
        FunctionNode::Arrow(arrow) => {
            let body = if arrow.expression {
                match arrow.body.statements.first() {
                    Some(oxc::Statement::ExpressionStatement(es)) => {
                        FunctionBody::Expression(&es.expression)
                    }
                    _ => FunctionBody::Block(arrow.body.as_ref()),
                }
            } else {
                FunctionBody::Block(arrow.body.as_ref())
            };
            (arrow.params.as_ref(), body, None, false, arrow.r#async, arrow.span)
        }
        FunctionNode::Function(f) => {
            let body_ref = f.body.as_deref().expect("function expression has a body");
            (
                f.params.as_ref(),
                FunctionBody::Block(body_ref),
                f.id.as_ref().map(|id| id.name),
                f.generator,
                f.r#async,
                f.span,
            )
        }
    };

    let function_scope = func.scope_id().unwrap_or_else(|| builder.scope().program_scope());

    let component_scope = builder.component_scope();
    let scope = builder.scope();

    let parent_bindings = builder.bindings().clone();
    let parent_used_names = builder.used_names().clone();
    let context_ids = builder.context_identifiers().clone();
    let ident_spans = builder.identifier_spans();

    // Gather captured context
    let captured_context =
        gather_captured_context(scope, function_scope, component_scope, ident_spans);
    let merged_context: FxIndexMap<SymbolId, Option<Span>> = {
        let parent_context = builder.context().clone();
        let mut merged = parent_context;
        for (k, v) in captured_context {
            merged.insert(k, v);
        }
        merged
    };

    let env = builder.environment_mut();
    let (hir_func, child_used_names, child_bindings) = lower_inner(
        params,
        body,
        id,
        generator,
        is_async,
        func_span,
        scope,
        env,
        Some(parent_bindings),
        Some(parent_used_names),
        merged_context,
        function_scope,
        component_scope,
        &context_ids,
        false, // nested function
        ident_spans,
    )?;

    builder.merge_used_names(child_used_names);
    builder.merge_bindings(child_bindings);

    let func_id = builder.environment_mut().add_function(hir_func);
    Ok(LoweredFunction { func: func_id })
}

/// Lower a function declaration statement to a FunctionExpression + StoreLocal.
fn lower_function_declaration<'a>(
    builder: &mut HirBuilder<'a, '_>,
    func_decl: &oxc::Function<'a>,
) -> Result<(), OxcDiagnostic> {
    let span = func_decl.span;

    let func_name = func_decl.id.as_ref().map(|id| id.name);

    // Find the function's scope
    let function_scope =
        func_decl.scope_id.get().unwrap_or_else(|| builder.scope().program_scope());

    let component_scope = builder.component_scope();
    let scope = builder.scope();

    let parent_bindings = builder.bindings().clone();
    let parent_used_names = builder.used_names().clone();
    let context_ids = builder.context_identifiers().clone();
    let ident_spans = builder.identifier_spans();

    // Gather captured context
    let captured_context =
        gather_captured_context(scope, function_scope, component_scope, ident_spans);
    let merged_context: FxIndexMap<SymbolId, Option<Span>> = {
        let parent_context = builder.context().clone();
        let mut merged = parent_context;
        for (k, v) in captured_context {
            merged.insert(k, v);
        }
        merged
    };

    let body_ref = func_decl.body.as_deref().expect("function declaration has a body");
    let env = builder.environment_mut();
    let (hir_func, child_used_names, child_bindings) = lower_inner(
        func_decl.params.as_ref(),
        FunctionBody::Block(body_ref),
        func_decl.id.as_ref().map(|id| id.name),
        func_decl.generator,
        func_decl.r#async,
        span,
        scope,
        env,
        Some(parent_bindings),
        Some(parent_used_names),
        merged_context,
        function_scope,
        component_scope,
        &context_ids,
        false, // nested function
        ident_spans,
    )?;

    builder.merge_used_names(child_used_names);
    builder.merge_bindings(child_bindings);

    let func_id = builder.environment_mut().add_function(hir_func);
    let lowered_func = LoweredFunction { func: func_id };

    // Emit FunctionExpression instruction
    let fn_value = InstructionValue::FunctionExpression {
        name: func_name,
        name_hint: None,
        lowered_func,
        expr_type: FunctionExpressionType::FunctionDeclaration,
        span: Some(span),
    };
    let fn_place = lower_value_to_temporary(builder, fn_value)?;

    // Resolve the binding for the function name and store. TS resolves the id
    // via Babel's `path.scope.getBinding(name)`, which starts at the function's
    // OWN scope: a body-level local that shadows the function's name resolves
    // to that inner binding — storing the function into the shadow while
    // references elsewhere resolve to the hoisted binding in the parent scope.
    // This is a known TS quirk that we reproduce for parity (see
    // todo-repro-named-function-with-shadowed-local-same-name). Fall back to
    // node-based resolution when the scope walk fails (degraded scope info,
    // e.g. synthetic scopes, or backends that split function-body scopes).
    if let Some(name) = func_name {
        if let Some(id_node) = &func_decl.id {
            let ident_span = id_node.span;
            let scope_binding =
                builder.get_function_declaration_binding(function_scope, name.as_str());
            let mut is_context = false;
            let binding = match scope_binding {
                Some(symbol_id) => {
                    is_context = builder.is_context_binding(symbol_id);
                    let binding_kind = crate::react_compiler_lowering::convert_binding_kind(
                        &builder.scope().binding_kind(symbol_id),
                    );
                    let identifier =
                        builder.resolve_binding_with_span(name, symbol_id, Some(ident_span))?;
                    VariableBinding::Identifier { identifier, binding_kind }
                }
                None => {
                    let mut binding = builder.resolve_identifier(
                        name,
                        ident_span,
                        builder.scope().resolve_binding_identifier(id_node),
                    )?;
                    if matches!(&binding, VariableBinding::Global { .. }) {
                        // For function redeclarations (e.g., `function x() {} function x() {}`),
                        // the redeclaration's identifier does not resolve as a declaration
                        // site (only the first declaration does). Retry with the binding
                        // found on the scope chain, resolving through its first declaration.
                        let fallback = {
                            let scope = builder.scope();
                            let scope_id =
                                func_decl.scope_id.get().unwrap_or_else(|| scope.program_scope());
                            scope.find_binding(scope_id, name.as_str())
                        };
                        if let Some(symbol_id) = fallback {
                            let symbol =
                                builder.scope().declaration_ident(symbol_id).map(|_| symbol_id);
                            binding = builder.resolve_identifier(name, ident_span, symbol)?;
                        }
                    }
                    if matches!(&binding, VariableBinding::Identifier { .. }) {
                        is_context = builder.is_context_identifier(
                            builder.scope().resolve_binding_identifier(id_node),
                        );
                    }
                    binding
                }
            };
            match binding {
                VariableBinding::Identifier { identifier, .. } => {
                    // Don't override the identifier's declaration span here.
                    // For function redeclarations (e.g., `function x() {} function x() {}`),
                    // the identifier's span should remain the first declaration's span,
                    // which was already set during define_binding.
                    // Use the full function declaration span for the Place,
                    // matching the TS behavior where lowerAssignment uses stmt.node.span
                    let place = Place {
                        identifier,
                        reactive: false,
                        effect: Effect::Unknown,
                        span: Some(span),
                    };
                    if is_context {
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreContext {
                                lvalue: LValue { kind: InstructionKind::Function, place },
                                value: fn_place,
                                span: Some(span),
                            },
                        )?;
                    } else {
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreLocal {
                                lvalue: LValue { kind: InstructionKind::Function, place },
                                value: fn_place,
                                span: Some(span),
                            },
                        )?;
                    }
                }
                _ => {
                    builder.record_error(
                        ErrorCategory::Invariant
                            .diagnostic(format!(
                                "Could not find binding for function declaration `{}`",
                                name
                            ))
                            .with_label(span),
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Lower a function expression used as an object method.
fn lower_function_for_object_method<'a>(
    builder: &mut HirBuilder<'a, '_>,
    method_span: oxc_span::Span,
    func: &oxc::Function<'a>,
    params: &oxc::FormalParameters<'a>,
    body: &oxc::FunctionBody<'a>,
    generator: bool,
    is_async: bool,
) -> Result<LoweredFunction, OxcDiagnostic> {
    let func_span = method_span;

    let function_scope = func.scope_id.get().unwrap_or_else(|| builder.scope().program_scope());

    let component_scope = builder.component_scope();
    let scope = builder.scope();

    let parent_bindings = builder.bindings().clone();
    let parent_used_names = builder.used_names().clone();
    let context_ids = builder.context_identifiers().clone();
    let ident_spans = builder.identifier_spans();

    let captured_context =
        gather_captured_context(scope, function_scope, component_scope, ident_spans);
    let merged_context: FxIndexMap<SymbolId, Option<Span>> = {
        let parent_context = builder.context().clone();
        let mut merged = parent_context;
        for (k, v) in captured_context {
            merged.insert(k, v);
        }
        merged
    };

    let env = builder.environment_mut();
    let (hir_func, child_used_names, child_bindings) = lower_inner(
        params,
        FunctionBody::Block(body),
        None,
        generator,
        is_async,
        func_span,
        scope,
        env,
        Some(parent_bindings),
        Some(parent_used_names),
        merged_context,
        function_scope,
        component_scope,
        &context_ids,
        false, // nested function
        ident_spans,
    )?;

    builder.merge_used_names(child_used_names);
    builder.merge_bindings(child_bindings);

    let func_id = builder.environment_mut().add_function(hir_func);
    Ok(LoweredFunction { func: func_id })
}

fn gather_captured_context(
    scope: &ScopeResolver<'_, '_>,
    function_scope: ScopeId,
    component_scope: ScopeId,
    identifier_spans: &IdentifierLocIndex,
) -> FxIndexMap<SymbolId, Option<Span>> {
    let root_node = scope.capture_root_node(function_scope);
    let parent_scope = scope.scope_parent(function_scope);
    let pure_scopes = match parent_scope {
        Some(parent) => capture_scopes(scope, parent, component_scope),
        None => FxIndexSet::default(),
    };

    // Collect the earliest (lowest source position) reference location for each
    // captured binding. Using the minimum position makes the result independent of
    // reference iteration order, matching the behavior the TS compiler gets from
    // Babel's position-ordered traversal.
    let mut captured: rustc_hash::FxHashMap<
        SymbolId,
        (u32, Option<Span>), // (min_position, span)
    > = rustc_hash::FxHashMap::default();

    for symbol_id in scope.symbols() {
        // Skip type-only bindings
        if matches!(
            scope.decl_kind(symbol_id),
            DeclKind::TSTypeAliasDeclaration | DeclKind::TSEnumDeclaration
        ) {
            continue;
        }
        if !pure_scopes.contains(&scope.symbol_scope(symbol_id)) {
            continue;
        }
        let declaration_start = scope.declaration_ident(symbol_id).map(|id| id.span.start);
        for &ref_id in scope.reference_ids(symbol_id) {
            // Only references the identifier walk recorded participate; the walk
            // covers exactly the compiled function's subtree.
            let Some(entry) = identifier_spans.reference(ref_id) else { continue };
            let ref_start = entry.span.start;
            // Skip type-annotation references: TS's gatherCapturedContext traverse
            // skips TypeAnnotation/TSTypeAnnotation/TypeAlias/TSTypeAliasDeclaration
            // subtrees, so identifiers there never become captures (they DO still
            // feed FindContextIdentifiers and the hoisting analysis, which have no
            // such skip in TS).
            if entry.in_type_annotation {
                continue;
            }
            if !scope.node_within(scope.reference_node_id(ref_id), root_node) {
                continue;
            }
            // Skip references whose start offset aliases the binding's own
            // declaration offset. Hermes desugars (component syntax) reuse the
            // original source offsets for generated nodes, so a sibling
            // reference structurally OUTSIDE this function (e.g. the forwardRef
            // argument naming the desugared inner function) can fall inside the
            // function's position range and alias the declaration position. In
            // real source a non-declaration reference can never share its
            // declaration's offset, so this only filters desugared aliases.
            if declaration_start == Some(ref_start) {
                continue;
            }
            let span = Some(entry.opening_element_span.unwrap_or(entry.span));
            captured
                .entry(symbol_id)
                .and_modify(|(min_pos, existing_span)| {
                    if ref_start < *min_pos {
                        *min_pos = ref_start;
                        *existing_span = span;
                    }
                })
                .or_insert((ref_start, span));
        }
    }

    // Sort captured entries by source position so context declarations appear
    // in source order, matching the TS compiler's position-ordered traversal.
    let mut sorted: Vec<_> = captured.into_iter().collect();
    sorted.sort_by_key(|(_, (pos, _))| *pos);

    sorted.into_iter().map(|(sid, (_, span))| (sid, span)).collect()
}

fn capture_scopes(
    scope: &ScopeResolver<'_, '_>,
    from: ScopeId,
    to: ScopeId,
) -> FxIndexSet<ScopeId> {
    let mut result = FxIndexSet::default();
    let mut current = Some(from);
    while let Some(scope_id) = current {
        result.insert(scope_id);
        if scope_id == to {
            break;
        }
        current = scope.scope_parent(scope_id);
    }
    result
}

fn lower_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    expr: &oxc::Expression<'a>,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    match expr {
        oxc::Expression::Identifier(ident) => {
            let span = Some(ident.span);
            let symbol = builder.scope().resolve_reference(ident);
            let place = lower_identifier(builder, ident.name, ident.span, symbol)?;
            if builder.is_context_identifier(symbol) {
                Ok(InstructionValue::LoadContext { place, span })
            } else {
                Ok(InstructionValue::LoadLocal { place, span })
            }
        }
        oxc::Expression::NullLiteral(lit) => {
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Null, span: Some(lit.span) })
        }
        oxc::Expression::BooleanLiteral(lit) => Ok(InstructionValue::Primitive {
            value: PrimitiveValue::Boolean(lit.value),
            span: Some(lit.span),
        }),
        oxc::Expression::NumericLiteral(lit) => Ok(InstructionValue::Primitive {
            value: PrimitiveValue::Number(FloatValue::new(lit.value)),
            span: Some(lit.span),
        }),
        oxc::Expression::StringLiteral(lit) => Ok(InstructionValue::Primitive {
            value: PrimitiveValue::String(lit.value),
            span: Some(lit.span),
        }),
        oxc::Expression::RegExpLiteral(regexp) => Ok(InstructionValue::RegExpLiteral {
            pattern: regexp.regex.pattern.text,
            flags: Str::from_str_in(
                &regexp.regex.flags.to_string(),
                &builder.environment().allocator,
            ),
            span: Some(regexp.span),
        }),
        oxc::Expression::BinaryExpression(bin) => {
            let span = Some(bin.span);
            let left = lower_expression_to_temporary(builder, &bin.left)?;
            let right = lower_expression_to_temporary(builder, &bin.right)?;
            Ok(InstructionValue::BinaryExpression { operator: bin.operator, left, right, span })
        }
        oxc::Expression::UnaryExpression(unary) => {
            let span = Some(unary.span);
            match unary.operator {
                oxc::UnaryOperator::Delete => {
                    // `delete obj.prop` / `delete obj[key]` mutate `obj`; lower to
                    // PropertyDelete / ComputedDelete so mutation inference (and the
                    // frozen-value check) sees the mutation, matching TS `BuildHIR`.
                    // Unwrap parens first: oxc keeps `delete (obj.prop)` as a
                    // `ParenthesizedExpression`, unlike Babel.
                    if let Some(member) =
                        unary.argument.without_parentheses().as_member_expression()
                    {
                        let lowered = lower_member_expression(builder, member)?;
                        match lowered.property {
                            MemberProperty::Literal(property) => {
                                Ok(InstructionValue::PropertyDelete {
                                    object: lowered.object,
                                    property,
                                    span,
                                })
                            }
                            MemberProperty::Computed(property) => {
                                Ok(InstructionValue::ComputedDelete {
                                    object: lowered.object,
                                    property,
                                    span,
                                })
                            }
                        }
                    } else {
                        // Anything else — a bare identifier, an optional chain
                        // (`delete obj?.prop`, kept as a `ChainExpression`), etc. — can't
                        // delete an object property; the fork rejects it rather than
                        // silently dropping the delete.
                        builder.record_error(
                            ErrorCategory::Syntax
                                .diagnostic("Only object properties can be deleted")
                                .with_labels(span),
                        )?;
                        Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
                    }
                }
                op => {
                    let value = lower_expression_to_temporary(builder, &unary.argument)?;
                    Ok(InstructionValue::UnaryExpression {
                        operator: convert_unary_operator(op),
                        value,
                        span,
                    })
                }
            }
        }
        oxc::Expression::LogicalExpression(logical) => {
            let span = Some(logical.span);
            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let test_block = builder.reserve(BlockKind::Value);
            let test_block_id = test_block.id;
            let place = build_temporary_place(builder, span);
            let left_span = Some(logical.left.span());
            let left_place = build_temporary_place(builder, left_span);

            let consequent_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
                lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                        value: left_place.clone(),
                        span: left_place.span,
                    },
                )?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span: left_place.span,
                })
            });

            let alternate_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
                let right = lower_expression_to_temporary(builder, &logical.right)?;
                let right_span = right.span;
                lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                        value: right,
                        span: right_span,
                    },
                )?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span: right_span,
                })
            });

            let hir_op = logical.operator;

            builder.terminate_with_continuation(
                Terminal::Logical {
                    operator: hir_op,
                    test: test_block_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                test_block,
            );

            let left_value = lower_expression_to_temporary(builder, &logical.left)?;
            builder.push(Instruction {
                id: EvaluationOrder::UNSET,
                lvalue: left_place.clone(),
                value: InstructionValue::LoadLocal { place: left_value, span },
                effects: None,
                span,
            });

            builder.terminate_with_continuation(
                Terminal::Branch {
                    test: left_place,
                    consequent: consequent_block?,
                    alternate: alternate_block?,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );

            Ok(InstructionValue::LoadLocal { place: place.clone(), span: place.span })
        }
        oxc::Expression::StaticMemberExpression(_)
        | oxc::Expression::ComputedMemberExpression(_)
        | oxc::Expression::PrivateFieldExpression(_) => {
            let lowered = lower_member_expression(builder, expr.as_member_expression().unwrap())?;
            Ok(lowered.value)
        }
        oxc::Expression::CallExpression(call) => {
            let span = Some(call.span);
            if let Some(member) = call.callee.as_member_expression() {
                let lowered = lower_member_expression(builder, member)?;
                let property = lower_value_to_temporary(builder, lowered.value)?;
                let args = lower_arguments(builder, &call.arguments)?;
                Ok(InstructionValue::MethodCall { receiver: lowered.object, property, args, span })
            } else {
                let callee = lower_expression_to_temporary(builder, &call.callee)?;
                let args = lower_arguments(builder, &call.arguments)?;
                Ok(InstructionValue::CallExpression { callee, args, span })
            }
        }
        oxc::Expression::ConditionalExpression(cond) => {
            let span = Some(cond.span);
            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let test_block = builder.reserve(BlockKind::Value);
            let test_block_id = test_block.id;
            let place = build_temporary_place(builder, span);

            // Block for the consequent (test is truthy)
            let consequent_ast_span = Some(cond.consequent.span());
            let consequent_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
                let consequent = lower_expression_to_temporary(builder, &cond.consequent)?;
                lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                        value: consequent,
                        span,
                    },
                )?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span: consequent_ast_span,
                })
            });

            // Block for the alternate (test is falsy)
            let alternate_ast_span = Some(cond.alternate.span());
            let alternate_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
                let alternate = lower_expression_to_temporary(builder, &cond.alternate)?;
                lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                        value: alternate,
                        span,
                    },
                )?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span: alternate_ast_span,
                })
            });

            builder.terminate_with_continuation(
                Terminal::Ternary {
                    test: test_block_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                test_block,
            );

            // Now in test block: lower test expression
            let test_place = lower_expression_to_temporary(builder, &cond.test)?;
            builder.terminate_with_continuation(
                Terminal::Branch {
                    test: test_place,
                    consequent: consequent_block?,
                    alternate: alternate_block?,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );

            Ok(InstructionValue::LoadLocal { place: place.clone(), span: place.span })
        }
        oxc::Expression::SequenceExpression(seq) => {
            let span = Some(seq.span);

            if seq.expressions.is_empty() {
                builder.record_error(
                    ErrorCategory::Syntax
                        .diagnostic("Expected sequence expression to have at least one expression")
                        .with_labels(span),
                )?;
                return Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span });
            }

            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let place = build_temporary_place(builder, span);

            let sequence_block = builder.try_enter(BlockKind::Sequence, |builder, _block_id| {
                let mut last: Option<Place> = None;
                for item in &seq.expressions {
                    last = Some(lower_expression_to_temporary(builder, item)?);
                }
                if let Some(last) = last {
                    lower_value_to_temporary(
                        builder,
                        InstructionValue::StoreLocal {
                            lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                            value: last,
                            span,
                        },
                    )?;
                }
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span,
                })
            });

            builder.terminate_with_continuation(
                Terminal::Sequence {
                    block: sequence_block?,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );
            Ok(InstructionValue::LoadLocal { place, span })
        }
        oxc::Expression::NewExpression(new_expr) => {
            let span = Some(new_expr.span);
            let callee = lower_expression_to_temporary(builder, &new_expr.callee)?;
            let args = lower_arguments(builder, &new_expr.arguments)?;
            Ok(InstructionValue::NewExpression { callee, args, span })
        }
        oxc::Expression::TemplateLiteral(tmpl) => {
            let span = Some(tmpl.span);
            let subexprs: Vec<Place> = tmpl
                .expressions
                .iter()
                .map(|e| lower_expression_to_temporary(builder, e))
                .collect::<Result<Vec<_>, _>>()?;
            let quasis: Vec<TemplateQuasi> =
                tmpl.quasis.iter().map(template_quasi_from_oxc).collect();
            Ok(InstructionValue::TemplateLiteral { subexprs, quasis, span })
        }
        oxc::Expression::TaggedTemplateExpression(tagged) => {
            let span = Some(tagged.span);
            // Upstream React Compiler bails on any interpolation here; the oxc port
            // instead lowers the tag plus every quasi and every `${...}`
            // subexpression (mirroring `TemplateLiteral`). This is a deliberate
            // divergence from the TS reference.
            //
            // We still bail when any quasi's cooked value differs from its raw value
            // (e.g. escape sequences or graphql templates), matching upstream's
            // single-quasi behavior — the HIR only round-trips raw==cooked quasis.
            if tagged.quasi.quasis.iter().any(|q| {
                q.value.raw.as_str() != q.value.cooked.map(|c| c.to_string()).unwrap_or_default()
            }) {
                builder.record_error(
                    ErrorCategory::Todo
                        .diagnostic("(BuildHIR::lowerExpression) Handle tagged template where cooked value is different from raw value")
                        .with_labels(span),
                )?;
                return Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span });
            }
            // Evaluation order: the tag is evaluated first, then each interpolated
            // subexpression left-to-right.
            let tag = lower_expression_to_temporary(builder, &tagged.tag)?;
            let subexprs: Vec<Place> = tagged
                .quasi
                .expressions
                .iter()
                .map(|e| lower_expression_to_temporary(builder, e))
                .collect::<Result<Vec<_>, _>>()?;
            let quasis: Vec<TemplateQuasi> =
                tagged.quasi.quasis.iter().map(template_quasi_from_oxc).collect();
            Ok(InstructionValue::TaggedTemplateExpression { tag, quasis, subexprs, span })
        }
        oxc::Expression::AwaitExpression(await_expr) => {
            let span = Some(await_expr.span);
            let value = lower_expression_to_temporary(builder, &await_expr.argument)?;
            Ok(InstructionValue::Await { value, span })
        }
        oxc::Expression::YieldExpression(yld) => {
            let span = Some(yld.span);
            builder.record_error(
                ErrorCategory::Todo
                    .diagnostic("(BuildHIR::lowerExpression) Handle YieldExpression expressions")
                    .with_labels(span),
            )?;
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
        }
        oxc::Expression::MetaProperty(meta) => {
            let span = Some(meta.span);
            if meta.meta.name == "import" && meta.property.name == "meta" {
                Ok(InstructionValue::MetaProperty {
                    meta: meta.meta.name,
                    property: meta.property.name,
                    span,
                })
            } else {
                builder.record_error(
                    ErrorCategory::Todo
                        .diagnostic("(BuildHIR::lowerExpression) Handle MetaProperty expressions other than import.meta")
                        .with_labels(span),
                )?;
                Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
            }
        }
        oxc::Expression::ClassExpression(cls) => {
            let span = Some(cls.span);
            builder.record_error(
                ErrorCategory::Todo
                    .diagnostic("(BuildHIR::lowerExpression) Handle ClassExpression expressions")
                    .with_labels(span),
            )?;
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
        }
        oxc::Expression::Super(sup) => {
            let span = Some(sup.span);
            builder.record_error(
                ErrorCategory::Todo
                    .diagnostic("(BuildHIR::lowerExpression) Handle Super expressions")
                    .with_labels(span),
            )?;
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
        }
        oxc::Expression::ThisExpression(this) => {
            let span = Some(this.span);
            builder.record_error(
                ErrorCategory::Todo
                    .diagnostic("(BuildHIR::lowerExpression) Handle ThisExpression expressions")
                    .with_labels(span),
            )?;
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
        }
        oxc::Expression::ImportExpression(imp) => {
            // oxc's `import(source, options?)` maps to Babel's
            // `CallExpression { callee: Import, arguments: [source] + options? }`.
            // The `Import` keyword callee bails (records an error), then the source
            // and options arguments are lowered left-to-right.
            let span = Some(imp.span);
            // The `import` keyword has no standalone node in oxc; synthesize its
            // span ([start, start+6)) so the callee bail error and temporary carry
            // the keyword span, matching Babel's `Import` node span.
            let import_keyword_span = Some(oxc_span::Span::new(imp.span.start, imp.span.start + 6));
            let callee = lower_import_keyword_to_temporary(builder, &import_keyword_span)?;
            let mut args: Vec<PlaceOrSpread> = Vec::new();
            let source = lower_expression_to_temporary(builder, &imp.source)?;
            args.push(PlaceOrSpread::Place(source));
            if let Some(options) = &imp.options {
                let options = lower_expression_to_temporary(builder, options)?;
                args.push(PlaceOrSpread::Place(options));
            }
            Ok(InstructionValue::CallExpression { callee, args, span })
        }
        oxc::Expression::PrivateInExpression(priv_in) => {
            // `#f in obj` maps to Babel's `BinaryExpression { op: In, left: PrivateName, right }`.
            // The PrivateName left operand bails (records an error), then the right
            // operand is lowered.
            let span = Some(priv_in.span);
            let left = lower_private_name_to_temporary(builder, priv_in.left.span)?;
            let right = lower_expression_to_temporary(builder, &priv_in.right)?;
            Ok(InstructionValue::BinaryExpression {
                operator: BinaryOperator::In,
                left,
                right,
                span,
            })
        }
        oxc::Expression::UpdateExpression(update) => {
            let span = Some(update.span);
            match &update.argument {
                oxc::SimpleAssignmentTarget::StaticMemberExpression(_)
                | oxc::SimpleAssignmentTarget::ComputedMemberExpression(_)
                | oxc::SimpleAssignmentTarget::PrivateFieldExpression(_) => {
                    let binary_op = match update.operator {
                        oxc::UpdateOperator::Increment => BinaryOperator::Addition,
                        oxc::UpdateOperator::Decrement => BinaryOperator::Subtraction,
                    };
                    // Use the member expression's span (not the update expression's)
                    // to match TS behavior where the inner operations use leftExpr.node.span
                    let member_span = Some(update.argument.span());
                    let lowered =
                        lower_member_expression_from_simple_target(builder, &update.argument)?;
                    let object = lowered.object;
                    let lowered_property = lowered.property;
                    let prev_value = lower_value_to_temporary(builder, lowered.value)?;

                    let one = lower_value_to_temporary(
                        builder,
                        InstructionValue::Primitive {
                            value: PrimitiveValue::Number(FloatValue::new(1.0)),
                            span: None,
                        },
                    )?;
                    let updated = lower_value_to_temporary(
                        builder,
                        InstructionValue::BinaryExpression {
                            operator: binary_op,
                            left: prev_value.clone(),
                            right: one,
                            span: member_span,
                        },
                    )?;

                    // Store back using the property from the lowered member expression.
                    // For prefix, the result is the PropertyStore/ComputedStore lvalue
                    // (matching TS which uses newValuePlace). For postfix, it's prev_value.
                    let new_value_place = match lowered_property {
                        MemberProperty::Literal(prop_literal) => lower_value_to_temporary(
                            builder,
                            InstructionValue::PropertyStore {
                                object,
                                property: prop_literal,
                                value: updated,
                                span: member_span,
                            },
                        )?,
                        MemberProperty::Computed(prop_place) => lower_value_to_temporary(
                            builder,
                            InstructionValue::ComputedStore {
                                object,
                                property: prop_place,
                                value: updated,
                                span: member_span,
                            },
                        )?,
                    };

                    // Return previous for postfix, newValuePlace for prefix
                    let result_place = if update.prefix { new_value_place } else { prev_value };
                    Ok(InstructionValue::LoadLocal {
                        place: result_place.clone(),
                        span: result_place.span,
                    })
                }
                oxc::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    let symbol = builder.scope().resolve_reference(ident);
                    if builder.is_context_identifier(symbol) {
                        builder.record_error(
                            ErrorCategory::Todo
                                .diagnostic("(BuildHIR::lowerExpression) Handle UpdateExpression to variables captured within lambdas.")
                                .with_labels(span),
                        )?;
                        return Ok(InstructionValue::Primitive {
                            value: PrimitiveValue::Undefined,
                            span,
                        });
                    }

                    let ident_span = ident.span;
                    let binding = builder.resolve_identifier(ident.name, ident_span, symbol)?;
                    if matches!(binding, VariableBinding::Global { .. }) {
                        builder.record_error(
                            ErrorCategory::Todo
                                .diagnostic("UpdateExpression where argument is a global is not yet supported")
                                .with_labels(span),
                        )?;
                        return Ok(InstructionValue::Primitive {
                            value: PrimitiveValue::Undefined,
                            span,
                        });
                    }
                    let identifier = match binding {
                        VariableBinding::Identifier { identifier, .. } => identifier,
                        _ => {
                            builder.record_error(
                                ErrorCategory::Todo
                                    .diagnostic("(BuildHIR::lowerExpression) Support UpdateExpression where argument is a global")
                                    .with_labels(span),
                            )?;
                            return Ok(InstructionValue::Primitive {
                                value: PrimitiveValue::Undefined,
                                span,
                            });
                        }
                    };
                    let lvalue_place = Place {
                        identifier,
                        effect: Effect::Unknown,
                        reactive: false,
                        span: Some(ident_span),
                    };

                    // Load the current value
                    let value = lower_identifier(
                        builder,
                        ident.name,
                        ident_span,
                        builder.scope().resolve_reference(ident),
                    )?;

                    let operation = update.operator;

                    if update.prefix {
                        Ok(InstructionValue::PrefixUpdate {
                            lvalue: lvalue_place,
                            operation,
                            value,
                            span,
                        })
                    } else {
                        Ok(InstructionValue::PostfixUpdate {
                            lvalue: lvalue_place,
                            operation,
                            value,
                            span,
                        })
                    }
                }
                _ => {
                    builder.record_error(
                        ErrorCategory::Todo
                            .diagnostic("UpdateExpression with unsupported argument type")
                            .with_labels(span),
                    )?;
                    Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
                }
            }
        }
        // `x as T` / `x satisfies T` / `<T>x` lower the inner expression to a
        // temporary and emit a `TypeCastExpression` carrying the type metadata,
        // mirroring the original Babel logic.
        oxc::Expression::TSAsExpression(ts) => lower_type_cast_expression(
            builder,
            ts.span,
            &ts.expression,
            &ts.type_annotation,
            TypeCast::As,
        ),
        oxc::Expression::TSSatisfiesExpression(ts) => lower_type_cast_expression(
            builder,
            ts.span,
            &ts.expression,
            &ts.type_annotation,
            TypeCast::Satisfies,
        ),
        oxc::Expression::TSTypeAssertion(ts) => lower_type_cast_expression(
            builder,
            ts.span,
            &ts.expression,
            &ts.type_annotation,
            TypeCast::As,
        ),
        // `x!` and `x<T>` unwrap to their inner expression (the original also just
        // unwraps these).
        oxc::Expression::TSNonNullExpression(ts) => lower_expression(builder, &ts.expression),
        oxc::Expression::TSInstantiationExpression(ts) => lower_expression(builder, &ts.expression),
        // oxc parses with `preserve_parens: true`, so `(expr)` is a real
        // `ParenthesizedExpression` node. The original Babel AST never carried
        // paren nodes (convert_ast stripped them), so unwrap to the inner
        // expression to reproduce the original HIR.
        oxc::Expression::ParenthesizedExpression(paren) => {
            lower_expression(builder, &paren.expression)
        }
        oxc::Expression::V8IntrinsicExpression(_) => {
            unreachable!(
                "V8IntrinsicExpression: oxc does not emit this without ParseOptions::allow_v8_intrinsics"
            )
        }
        oxc::Expression::ObjectExpression(obj) => {
            let span = Some(obj.span);
            let mut properties: Vec<ObjectPropertyOrSpread> = Vec::new();
            for prop in &obj.properties {
                match prop {
                    oxc::ObjectPropertyKind::ObjectProperty(p) => {
                        // In oxc, getters/setters/methods are encoded as an
                        // `ObjectProperty` whose value is a `FunctionExpression`
                        // (the Babel AST instead carried a separate `ObjectMethod`
                        // node). Route those through `lower_object_method`.
                        if p.method
                            || matches!(p.kind, oxc::PropertyKind::Get | oxc::PropertyKind::Set)
                        {
                            if let Some(method_prop) = lower_object_method(builder, p)? {
                                properties.push(ObjectPropertyOrSpread::Property(method_prop));
                            }
                            continue;
                        }
                        let key = lower_object_property_key(builder, &p.key, p.computed)?;
                        let key = match key {
                            Some(k) => k,
                            None => continue,
                        };
                        let value = lower_expression_to_temporary(builder, &p.value)?;
                        properties.push(ObjectPropertyOrSpread::Property(ObjectProperty {
                            key,
                            property_type: ObjectPropertyType::Property,
                            place: value,
                        }));
                    }
                    oxc::ObjectPropertyKind::SpreadProperty(spread) => {
                        let place = lower_expression_to_temporary(builder, &spread.argument)?;
                        properties.push(ObjectPropertyOrSpread::Spread(SpreadPattern { place }));
                    }
                }
            }
            Ok(InstructionValue::ObjectExpression { properties, span })
        }
        oxc::Expression::ArrayExpression(arr) => {
            let span = Some(arr.span);
            let mut elements: Vec<ArrayElement> = Vec::new();
            for element in &arr.elements {
                match element {
                    oxc::ArrayExpressionElement::Elision(_) => {
                        elements.push(ArrayElement::Hole);
                    }
                    oxc::ArrayExpressionElement::SpreadElement(spread) => {
                        let place = lower_expression_to_temporary(builder, &spread.argument)?;
                        elements.push(ArrayElement::Spread(SpreadPattern { place }));
                    }
                    _ => {
                        let expr = element.to_expression();
                        let place = lower_expression_to_temporary(builder, expr)?;
                        elements.push(ArrayElement::Place(place));
                    }
                }
            }
            Ok(InstructionValue::ArrayExpression { elements, span })
        }
        oxc::Expression::JSXElement(jsx_element) => lower_jsx_element_expr(builder, jsx_element),
        oxc::Expression::JSXFragment(jsx_fragment) => {
            lower_jsx_fragment_expr(builder, jsx_fragment)
        }
        oxc::Expression::ChainExpression(chain) => lower_chain_expression(builder, chain),
        oxc::Expression::ArrowFunctionExpression(arrow) => lower_function_to_value(
            builder,
            FunctionNode::Arrow(arrow),
            FunctionExpressionType::ArrowFunctionExpression,
        ),
        oxc::Expression::FunctionExpression(func) => lower_function_to_value(
            builder,
            FunctionNode::Function(func),
            FunctionExpressionType::FunctionExpression,
        ),
        oxc::Expression::AssignmentExpression(assign) => {
            lower_assignment_expression(builder, assign)
        }
        _ => {
            // not-yet-ported arms bail to undefined (differential green-set grows as arms land)
            let span = Some(expr.span());
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
        }
    }
}

/// Lower an `AssignmentExpression`. Faithful translation of the original
/// `Expression::AssignmentExpression` arm, adapted to oxc's `AssignmentTarget`
/// split. `=` handles identifier / member / destructuring targets; compound
/// operators (`+=` etc.) handle identifier / member targets and bail on patterns.
fn lower_assignment_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    assign: &oxc::AssignmentExpression<'a>,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    let span = Some(assign.span);

    if matches!(assign.operator, oxc::AssignmentOperator::Assign) {
        match &assign.left {
            oxc::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let symbol = builder.scope().resolve_reference(ident);
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let ident_span = ident.span;
                let binding = builder.resolve_identifier(ident.name, ident_span, symbol)?;
                match binding {
                    VariableBinding::Identifier { identifier, binding_kind } => {
                        if binding_kind == BindingKind::Const {
                            builder.record_error(
                                ErrorCategory::Syntax
                                    .diagnostic("Cannot reassign a `const` variable")
                                    .with_help(format!(
                                        "`{}` is declared as const",
                                        ident.name.as_str()
                                    ))
                                    .with_label(ident_span),
                            )?;
                            return Ok(InstructionValue::Primitive {
                                value: PrimitiveValue::Undefined,
                                span: Some(ident_span),
                            });
                        }
                        let place = Place {
                            identifier,
                            reactive: false,
                            effect: Effect::Unknown,
                            span: Some(ident_span),
                        };
                        if builder.is_context_identifier(symbol) {
                            let temp = lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreContext {
                                    lvalue: LValue {
                                        kind: InstructionKind::Reassign,
                                        place: place.clone(),
                                    },
                                    value: right,
                                    span: place.span,
                                },
                            )?;
                            Ok(InstructionValue::LoadLocal { place: temp.clone(), span: temp.span })
                        } else {
                            let temp = lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreLocal {
                                    lvalue: LValue {
                                        kind: InstructionKind::Reassign,
                                        place: place.clone(),
                                    },
                                    value: right,
                                    span: place.span,
                                },
                            )?;
                            Ok(InstructionValue::LoadLocal { place: temp.clone(), span: temp.span })
                        }
                    }
                    _ => {
                        let name = ident.name;
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreGlobal {
                                name,
                                value: right,
                                span: Some(ident_span),
                            },
                        )?;
                        Ok(InstructionValue::LoadLocal { place: temp.clone(), span: temp.span })
                    }
                }
            }
            oxc::AssignmentTarget::StaticMemberExpression(_)
            | oxc::AssignmentTarget::ComputedMemberExpression(_)
            | oxc::AssignmentTarget::PrivateFieldExpression(_) => {
                let simple = assign.left.as_simple_assignment_target().unwrap();
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let left_span = Some(simple.span());
                let temp = match simple {
                    oxc::SimpleAssignmentTarget::StaticMemberExpression(member) => {
                        let object = lower_expression_to_temporary(builder, &member.object)?;
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::PropertyStore {
                                object,
                                property: PropertyLiteral::String(member.property.name),
                                value: right,
                                span: left_span,
                            },
                        )?
                    }
                    oxc::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                        let object = lower_expression_to_temporary(builder, &member.object)?;
                        if let oxc::Expression::NumericLiteral(num) = &member.expression {
                            lower_value_to_temporary(
                                builder,
                                InstructionValue::PropertyStore {
                                    object,
                                    property: PropertyLiteral::Number(FloatValue::new(num.value)),
                                    value: right,
                                    span: left_span,
                                },
                            )?
                        } else {
                            let prop = lower_expression_to_temporary(builder, &member.expression)?;
                            lower_value_to_temporary(
                                builder,
                                InstructionValue::ComputedStore {
                                    object,
                                    property: prop,
                                    value: right,
                                    span: left_span,
                                },
                            )?
                        }
                    }
                    oxc::SimpleAssignmentTarget::PrivateFieldExpression(member) => {
                        // Babel modeled `a.#b = x` as a non-computed MemberExpression
                        // whose property is a PrivateName; the original fell to the
                        // generic property arm, lowering the PrivateName (which bails
                        // to an undefined temp) and emitting a ComputedStore.
                        let object = lower_expression_to_temporary(builder, &member.object)?;
                        let prop = lower_private_name_to_temporary(builder, member.field.span)?;
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::ComputedStore {
                                object,
                                property: prop,
                                value: right,
                                span: left_span,
                            },
                        )?
                    }
                    _ => unreachable!(),
                };
                Ok(InstructionValue::LoadLocal { place: temp.clone(), span: temp.span })
            }
            _ => {
                // Destructuring assignment
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let result = lower_assignment_target(
                    builder,
                    assign.left.span(),
                    InstructionKind::Reassign,
                    &assign.left,
                    right.clone(),
                    AssignmentStyle::Destructure,
                )?;
                match result {
                    Some(place) => {
                        Ok(InstructionValue::LoadLocal { place: place.clone(), span: place.span })
                    }
                    None => Ok(InstructionValue::LoadLocal { place: right, span }),
                }
            }
        }
    } else {
        // Compound assignment operators
        let binary_op = match assign.operator {
            oxc::AssignmentOperator::Addition => Some(BinaryOperator::Addition),
            oxc::AssignmentOperator::Subtraction => Some(BinaryOperator::Subtraction),
            oxc::AssignmentOperator::Multiplication => Some(BinaryOperator::Multiplication),
            oxc::AssignmentOperator::Division => Some(BinaryOperator::Division),
            oxc::AssignmentOperator::Remainder => Some(BinaryOperator::Remainder),
            oxc::AssignmentOperator::Exponential => Some(BinaryOperator::Exponential),
            oxc::AssignmentOperator::ShiftLeft => Some(BinaryOperator::ShiftLeft),
            oxc::AssignmentOperator::ShiftRight => Some(BinaryOperator::ShiftRight),
            oxc::AssignmentOperator::ShiftRightZeroFill => Some(BinaryOperator::ShiftRightZeroFill),
            oxc::AssignmentOperator::BitwiseOR => Some(BinaryOperator::BitwiseOR),
            oxc::AssignmentOperator::BitwiseXOR => Some(BinaryOperator::BitwiseXOR),
            oxc::AssignmentOperator::BitwiseAnd => Some(BinaryOperator::BitwiseAnd),
            oxc::AssignmentOperator::LogicalOr
            | oxc::AssignmentOperator::LogicalAnd
            | oxc::AssignmentOperator::LogicalNullish => {
                builder.record_error(
                    ErrorCategory::Todo
                        .diagnostic(
                            "Logical assignment operators (||=, &&=, ??=) are not yet supported",
                        )
                        .with_labels(span),
                )?;
                return Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span });
            }
            oxc::AssignmentOperator::Assign => unreachable!(),
        };
        let binary_op = match binary_op {
            Some(op) => op,
            None => {
                return Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span });
            }
        };

        match &assign.left {
            oxc::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let ident_span = ident.span;
                let symbol = builder.scope().resolve_reference(ident);
                let left_place = lower_identifier(builder, ident.name, ident_span, symbol)?;
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let binary_place = lower_value_to_temporary(
                    builder,
                    InstructionValue::BinaryExpression {
                        operator: binary_op,
                        left: left_place,
                        right,
                        span,
                    },
                )?;
                let binding = builder.resolve_identifier(ident.name, ident_span, symbol)?;
                match binding {
                    VariableBinding::Identifier { identifier, .. } => {
                        let place = Place {
                            identifier,
                            reactive: false,
                            effect: Effect::Unknown,
                            span: Some(ident_span),
                        };
                        if builder.is_context_identifier(symbol) {
                            lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreContext {
                                    lvalue: LValue {
                                        kind: InstructionKind::Reassign,
                                        place: place.clone(),
                                    },
                                    value: binary_place,
                                    span,
                                },
                            )?;
                            Ok(InstructionValue::LoadContext { place, span })
                        } else {
                            lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreLocal {
                                    lvalue: LValue {
                                        kind: InstructionKind::Reassign,
                                        place: place.clone(),
                                    },
                                    value: binary_place,
                                    span,
                                },
                            )?;
                            Ok(InstructionValue::LoadLocal { place, span })
                        }
                    }
                    _ => {
                        let name = ident.name;
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreGlobal { name, value: binary_place, span },
                        )?;
                        Ok(InstructionValue::LoadLocal { place: temp.clone(), span: temp.span })
                    }
                }
            }
            oxc::AssignmentTarget::StaticMemberExpression(_)
            | oxc::AssignmentTarget::ComputedMemberExpression(_)
            | oxc::AssignmentTarget::PrivateFieldExpression(_) => {
                let simple = assign.left.as_simple_assignment_target().unwrap();
                let member_span = Some(simple.span());
                let lowered = lower_member_expression_from_simple_target(builder, simple)?;
                let object = lowered.object;
                let lowered_property = lowered.property;
                let current_value = lower_value_to_temporary(builder, lowered.value)?;
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let result = lower_value_to_temporary(
                    builder,
                    InstructionValue::BinaryExpression {
                        operator: binary_op,
                        left: current_value,
                        right,
                        span: member_span,
                    },
                )?;
                match lowered_property {
                    MemberProperty::Literal(prop_literal) => Ok(InstructionValue::PropertyStore {
                        object,
                        property: prop_literal,
                        value: result,
                        span: member_span,
                    }),
                    MemberProperty::Computed(prop_place) => Ok(InstructionValue::ComputedStore {
                        object,
                        property: prop_place,
                        value: result,
                        span: member_span,
                    }),
                }
            }
            _ => {
                builder.record_error(
                    ErrorCategory::Todo
                        .diagnostic("Compound assignment to complex pattern is not yet supported")
                        .with_labels(span),
                )?;
                Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, span })
            }
        }
    }
}

/// Lower a JSX element expression. Faithful translation of the original Babel
/// `Expression::JSXElement` arm, adapted to oxc's JSX shapes.
///
/// fbt note: the original tracked fbt/fbs sub-tags (`collect_fbt_sub_tags`) and
/// reported duplicates, and incremented `builder.fbt_depth` around the children so
/// JSX text whitespace is preserved within fbt subtrees. Both behaviors are ported
/// below.
fn lower_jsx_element_expr<'a>(
    builder: &mut HirBuilder<'a, '_>,
    jsx_element: &oxc::JSXElement<'a>,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    let span = Some(jsx_element.span);
    let opening_span = Some(jsx_element.opening_element.span);
    let closing_span = jsx_element.closing_element.as_ref().map(|c| c.span);

    // Lower the tag name
    let tag = lower_jsx_element_name(builder, &jsx_element.opening_element.name)?;

    // Lower attributes (props)
    let mut props: Vec<JsxAttribute> = Vec::new();
    for attr_item in &jsx_element.opening_element.attributes {
        match attr_item {
            oxc::JSXAttributeItem::SpreadAttribute(spread) => {
                let argument = lower_expression_to_temporary(builder, &spread.argument)?;
                props.push(JsxAttribute::SpreadAttribute { argument });
            }
            oxc::JSXAttributeItem::Attribute(attr) => {
                // Get the attribute name
                let prop_name = match &attr.name {
                    oxc::JSXAttributeName::Identifier(id) => {
                        let name = id.name.as_str();
                        if name.contains(':') {
                            builder.record_error(
                                ErrorCategory::Todo
                                    .diagnostic(format!(
                                        "(BuildHIR::lowerExpression) Unexpected colon in attribute name `{}`",
                                        name
                                    )).with_label(id.span),
                            )?;
                        }
                        Ident::from(name)
                    }
                    oxc::JSXAttributeName::NamespacedName(ns) => format_ident!(
                        builder.environment().allocator,
                        "{}:{}",
                        ns.namespace.name,
                        ns.name.name
                    ),
                };

                // Get the attribute value
                let value = match &attr.value {
                    Some(oxc::JSXAttributeValue::StringLiteral(s)) => {
                        let str_span = Some(s.span);
                        let decoded = match decode_jsx_entities(s.value.as_str()) {
                            Cow::Borrowed(text) => Str::from(text),
                            Cow::Owned(text) => {
                                Str::from_str_in(&text, &builder.environment().allocator)
                            }
                        };
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::Primitive {
                                value: PrimitiveValue::String(decoded),
                                span: str_span,
                            },
                        )?
                    }
                    Some(oxc::JSXAttributeValue::ExpressionContainer(container)) => {
                        match &container.expression {
                            oxc::JSXExpression::EmptyExpression(_) => {
                                // Empty expression container - skip this attribute
                                continue;
                            }
                            other => {
                                let expr = other
                                    .as_expression()
                                    .expect("non-empty JSX expression is an expression");
                                lower_expression_to_temporary(builder, expr)?
                            }
                        }
                    }
                    Some(oxc::JSXAttributeValue::Element(el)) => {
                        let val = lower_jsx_element_expr(builder, el)?;
                        lower_value_to_temporary(builder, val)?
                    }
                    Some(oxc::JSXAttributeValue::Fragment(frag)) => {
                        let val = lower_jsx_fragment_expr(builder, frag)?;
                        lower_value_to_temporary(builder, val)?
                    }
                    None => {
                        // No value means boolean true (e.g., <div disabled />)
                        let attr_span = Some(attr.span);
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::Primitive {
                                value: PrimitiveValue::Boolean(true),
                                span: attr_span,
                            },
                        )?
                    }
                };

                props.push(JsxAttribute::Attribute { name: prop_name, place: value });
            }
        }
    }

    // Check if this is an fbt/fbs tag, which requires special whitespace handling
    let is_fbt = matches!(&tag, JsxTag::Builtin(b) if b.name == "fbt" || b.name == "fbs");

    // Check that fbt/fbs tags are module-level imports, not local bindings.
    // Matches TS: Diagnostics.invariant(tagIdentifier.kind !== 'Identifier', ...)
    if is_fbt {
        let tag_name = match &tag {
            JsxTag::Builtin(b) => b.name,
            _ => Ident::from("fbt"),
        };
        // Get the opening element's name identifier and check if it's a local binding.
        let jsx_id_name = match &jsx_element.opening_element.name {
            oxc::JSXElementName::Identifier(id) => Some((id.name.as_str(), id.span)),
            oxc::JSXElementName::IdentifierReference(id) => Some((id.name.as_str(), id.span)),
            _ => None,
        };
        if let Some((name, span)) = jsx_id_name {
            let id_span = Some(span);
            // Check if fbt/fbs tag name resolves to a local binding.
            // JSX identifiers may not be in our position-based reference map,
            // so check if ANY binding with this name exists in the function scope.
            let is_local_binding = builder.has_local_binding(name);
            if is_local_binding {
                let reason = format!("<{}> tags should be module-level imports", tag_name);
                return Err(ErrorCategory::Invariant
                    .diagnostic(&reason)
                    .with_labels(id_span.map(|s| s.label(reason))));
            }
        }
    }

    // Check for duplicate fbt:enum, fbt:plural, fbt:pronoun tags.
    if is_fbt {
        let tag_name = match &tag {
            JsxTag::Builtin(b) => b.name.as_str(),
            _ => "fbt",
        };
        let mut enum_spans: Vec<Option<Span>> = Vec::new();
        let mut plural_spans: Vec<Option<Span>> = Vec::new();
        let mut pronoun_spans: Vec<Option<Span>> = Vec::new();
        collect_fbt_sub_tags(
            builder,
            &jsx_element.children,
            tag_name,
            &mut enum_spans,
            &mut plural_spans,
            &mut pronoun_spans,
        );

        for (name, locations) in
            [("enum", &enum_spans), ("plural", &plural_spans), ("pronoun", &pronoun_spans)]
        {
            if locations.len() > 1 {
                let diag = ErrorCategory::Todo
                    .diagnostic("Support duplicate fbt tags")
                    .with_help(format!(
                        "Support `<{}>` tags with multiple `<{}:{}>` values",
                        tag_name, tag_name, name
                    ))
                    .with_labels(locations.iter().filter_map(|span| {
                        span.map(|s| {
                            s.label(format!("Multiple `<{}:{}>` tags found", tag_name, name))
                        })
                    }));
                builder.environment_mut().record_diagnostic(diag);
            }
        }
    }

    // Increment fbt counter before traversing into children, as whitespace
    // in jsx text is handled differently for fbt subtrees.
    if is_fbt {
        builder.fbt_depth += 1;
    }

    // Lower children
    let children: Vec<Place> = jsx_element
        .children
        .iter()
        .map(|child| lower_jsx_element(builder, child))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    if is_fbt {
        builder.fbt_depth -= 1;
    }

    Ok(InstructionValue::JsxExpression {
        tag,
        props,
        children: if children.is_empty() { None } else { Some(children) },
        span,
        opening_span,
        closing_span,
    })
}

/// Lower a JSX fragment expression. Faithful translation of the original
/// `Expression::JSXFragment` arm.
fn lower_jsx_fragment_expr<'a>(
    builder: &mut HirBuilder<'a, '_>,
    jsx_fragment: &oxc::JSXFragment<'a>,
) -> Result<InstructionValue<'a>, OxcDiagnostic> {
    let span = Some(jsx_fragment.span);

    // Lower children
    let children: Vec<Place> = jsx_fragment
        .children
        .iter()
        .map(|child| lower_jsx_element(builder, child))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(InstructionValue::JsxFragment { children, span })
}

/// Lower a JSX element name into a `JsxTag`. Faithful translation of the original
/// `lower_jsx_element_name`, adapted to oxc's `JSXElementName` shape (which splits
/// out `IdentifierReference`, `MemberExpression`, and `ThisExpression`; the latter
/// maps to the identifier `"this"`).
fn lower_jsx_element_name<'a>(
    builder: &mut HirBuilder<'a, '_>,
    name: &oxc::JSXElementName<'a>,
) -> Result<JsxTag<'a>, OxcDiagnostic> {
    // Lower a simple JSX tag identifier (component-vs-builtin split on case).
    fn lower_tag_identifier<'a>(
        builder: &mut HirBuilder<'a, '_>,
        tag: Ident<'a>,
        span: oxc_span::Span,
        symbol: Option<SymbolId>,
    ) -> Result<JsxTag<'a>, OxcDiagnostic> {
        if tag.starts_with(|c: char| c.is_ascii_uppercase()) {
            // Component tag: resolve as identifier and load
            let place = lower_identifier(builder, tag, span, symbol)?;
            let load_value = if builder.is_context_identifier(symbol) {
                InstructionValue::LoadContext { place, span: Some(span) }
            } else {
                InstructionValue::LoadLocal { place, span: Some(span) }
            };
            let temp = lower_value_to_temporary(builder, load_value)?;
            Ok(JsxTag::Place(temp))
        } else {
            // Builtin HTML tag
            Ok(JsxTag::Builtin(BuiltinTag { name: tag }))
        }
    }

    match name {
        oxc::JSXElementName::Identifier(id) => {
            lower_tag_identifier(builder, Ident::from(id.name.as_str()), id.span, None)
        }
        oxc::JSXElementName::IdentifierReference(id) => {
            let symbol = builder.scope().resolve_reference(id);
            lower_tag_identifier(builder, id.name, id.span, symbol)
        }
        oxc::JSXElementName::ThisExpression(this) => {
            // `<this.Foo />`-style `this` tag lowers as the identifier "this".
            lower_tag_identifier(builder, Ident::from("this"), this.span, None)
        }
        oxc::JSXElementName::MemberExpression(member) => {
            let place = lower_jsx_member_expression(builder, member)?;
            Ok(JsxTag::Place(place))
        }
        oxc::JSXElementName::NamespacedName(ns) => {
            let namespace = ns.namespace.name.as_str();
            let name = ns.name.name.as_str();
            let tag = format!("{}:{}", namespace, name);
            let span = Some(ns.span);
            if namespace.contains(':') || name.contains(':') {
                builder.record_error(
                    ErrorCategory::Syntax
                        .diagnostic(
                            "Expected JSXNamespacedName to have no colons in the namespace or name",
                        )
                        .with_help(format!("Got `{}` : `{}`", namespace, name))
                        .with_labels(span),
                )?;
            }
            let place = lower_value_to_temporary(
                builder,
                InstructionValue::Primitive {
                    value: PrimitiveValue::String(Str::from_str_in(
                        &tag,
                        &builder.environment().allocator,
                    )),
                    span,
                },
            )?;
            Ok(JsxTag::Place(place))
        }
    }
}

/// Lower a JSX member expression tag (`<a.b.c />`) into a `Place`. Faithful
/// translation of the original `lower_jsx_member_expression`, adapted to oxc's
/// `JSXMemberExpressionObject` (where the leaf object may be a `ThisExpression`,
/// which lowers as the identifier `"this"`).
fn lower_jsx_member_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    expr: &oxc::JSXMemberExpression<'a>,
) -> Result<Place, OxcDiagnostic> {
    // Use the full member expression's span for instruction locs (matching TS: exprPath.node.span)
    let expr_span = Some(expr.span);
    let object = match &expr.object {
        oxc::JSXMemberExpressionObject::IdentifierReference(id) => {
            let symbol = builder.scope().resolve_reference(id);
            lower_jsx_member_object_identifier(builder, id.name, id.span, symbol, &expr_span)?
        }
        oxc::JSXMemberExpressionObject::ThisExpression(this) => lower_jsx_member_object_identifier(
            builder,
            Ident::from("this"),
            this.span,
            None,
            &expr_span,
        )?,
        oxc::JSXMemberExpressionObject::MemberExpression(inner) => {
            lower_jsx_member_expression(builder, inner)?
        }
    };
    let prop_name = expr.property.name.as_str();
    let value = InstructionValue::PropertyLoad {
        object,
        property: PropertyLiteral::String(Ident::from(prop_name)),
        span: expr_span,
    };
    lower_value_to_temporary(builder, value)
}

/// Lower the leaf identifier of a JSX member expression object. Uses the
/// identifier's own span for the place, but the enclosing member expression's span
/// for the load instruction (matching TS).
fn lower_jsx_member_object_identifier<'a>(
    builder: &mut HirBuilder<'a, '_>,
    name: Ident<'a>,
    span: oxc_span::Span,
    symbol: Option<SymbolId>,
    expr_span: &Option<Span>,
) -> Result<Place, OxcDiagnostic> {
    let place = lower_identifier(builder, name, span, symbol)?;
    let load_value = if builder.is_context_identifier(symbol) {
        InstructionValue::LoadContext { place, span: *expr_span }
    } else {
        InstructionValue::LoadLocal { place, span: *expr_span }
    };
    lower_value_to_temporary(builder, load_value)
}

/// Lower a single JSX child into an optional `Place`. Faithful translation of the
/// original `lower_jsx_element` (the JSXChild handler), adapted to oxc's `JSXChild`.
fn lower_jsx_element<'a>(
    builder: &mut HirBuilder<'a, '_>,
    child: &oxc::JSXChild<'a>,
) -> Result<Option<Place>, OxcDiagnostic> {
    match child {
        oxc::JSXChild::Text(text) => {
            // oxc keeps JSX text raw; decode entities first so the value matches
            // Babel's `JSXText.value` (the Babel bridge decoded in convert_ast).
            let decoded = decode_jsx_entities(text.value.as_str());
            // FBT whitespace normalization differs from standard JSX.
            // Since the fbt transform runs after, preserve all whitespace
            // in FBT subtrees as is.
            let value = if builder.fbt_depth > 0 {
                Some(match decoded {
                    Cow::Borrowed(text) => Str::from(text),
                    Cow::Owned(ref text) => {
                        Str::from_str_in(text, &builder.environment().allocator)
                    }
                })
            } else {
                trim_jsx_text(&decoded)
                    .map(|text| Str::from_str_in(&text, &builder.environment().allocator))
            };
            match value {
                None => Ok(None),
                Some(value) => {
                    let span = Some(text.span);
                    let place = lower_value_to_temporary(
                        builder,
                        InstructionValue::JSXText { value, span },
                    )?;
                    Ok(Some(place))
                }
            }
        }
        oxc::JSXChild::Element(element) => {
            let value = lower_jsx_element_expr(builder, element)?;
            Ok(Some(lower_value_to_temporary(builder, value)?))
        }
        oxc::JSXChild::Fragment(fragment) => {
            let value = lower_jsx_fragment_expr(builder, fragment)?;
            Ok(Some(lower_value_to_temporary(builder, value)?))
        }
        oxc::JSXChild::ExpressionContainer(container) => match &container.expression {
            oxc::JSXExpression::EmptyExpression(_) => Ok(None),
            other => {
                let expr =
                    other.as_expression().expect("non-empty JSX expression is an expression");
                Ok(Some(lower_expression_to_temporary(builder, expr)?))
            }
        },
        oxc::JSXChild::Spread(spread) => {
            Ok(Some(lower_expression_to_temporary(builder, &spread.expression)?))
        }
    }
}

/// Recursively collect the locations of `<tag:enum>`, `<tag:plural>`, and
/// `<tag:pronoun>` sub-tags within fbt/fbs children. Faithful translation of the
/// original Babel `collect_fbt_sub_tags`, adapted to oxc's JSX shapes.
fn collect_fbt_sub_tags(
    builder: &HirBuilder<'_, '_>,
    children: &[oxc::JSXChild],
    tag_name: &str,
    enum_spans: &mut Vec<Option<Span>>,
    plural_spans: &mut Vec<Option<Span>>,
    pronoun_spans: &mut Vec<Option<Span>>,
) {
    for child in children {
        match child {
            oxc::JSXChild::Element(el) => {
                collect_fbt_sub_tags_from_element(
                    builder,
                    el,
                    tag_name,
                    enum_spans,
                    plural_spans,
                    pronoun_spans,
                );
            }
            oxc::JSXChild::Fragment(frag) => {
                collect_fbt_sub_tags(
                    builder,
                    &frag.children,
                    tag_name,
                    enum_spans,
                    plural_spans,
                    pronoun_spans,
                );
            }
            oxc::JSXChild::ExpressionContainer(container) => {
                if let Some(expr) = container.expression.as_expression() {
                    collect_fbt_sub_tags_from_expr(
                        builder,
                        expr,
                        tag_name,
                        enum_spans,
                        plural_spans,
                        pronoun_spans,
                    );
                }
            }
            _ => {}
        }
    }
}

fn collect_fbt_sub_tags_from_element(
    builder: &HirBuilder<'_, '_>,
    el: &oxc::JSXElement,
    tag_name: &str,
    enum_spans: &mut Vec<Option<Span>>,
    plural_spans: &mut Vec<Option<Span>>,
    pronoun_spans: &mut Vec<Option<Span>>,
) {
    if let oxc::JSXElementName::NamespacedName(ns) = &el.opening_element.name {
        if ns.namespace.name == tag_name {
            let span = Some(ns.span);
            match ns.name.name.as_str() {
                "enum" => enum_spans.push(span),
                "plural" => plural_spans.push(span),
                "pronoun" => pronoun_spans.push(span),
                _ => {}
            }
        }
    }
    collect_fbt_sub_tags(builder, &el.children, tag_name, enum_spans, plural_spans, pronoun_spans);
    // Also traverse JSX attributes (matching TS expr.traverse which visits all nodes)
    for attr in &el.opening_element.attributes {
        if let oxc::JSXAttributeItem::Attribute(a) = attr {
            match &a.value {
                Some(oxc::JSXAttributeValue::ExpressionContainer(container)) => {
                    if let Some(expr) = container.expression.as_expression() {
                        collect_fbt_sub_tags_from_expr(
                            builder,
                            expr,
                            tag_name,
                            enum_spans,
                            plural_spans,
                            pronoun_spans,
                        );
                    }
                }
                Some(oxc::JSXAttributeValue::Element(nested)) => {
                    collect_fbt_sub_tags_from_element(
                        builder,
                        nested,
                        tag_name,
                        enum_spans,
                        plural_spans,
                        pronoun_spans,
                    );
                }
                _ => {}
            }
        }
    }
}

fn collect_fbt_sub_tags_from_expr(
    builder: &HirBuilder<'_, '_>,
    expr: &oxc::Expression,
    tag_name: &str,
    enum_spans: &mut Vec<Option<Span>>,
    plural_spans: &mut Vec<Option<Span>>,
    pronoun_spans: &mut Vec<Option<Span>>,
) {
    match expr {
        oxc::Expression::JSXElement(el) => {
            collect_fbt_sub_tags_from_element(
                builder,
                el,
                tag_name,
                enum_spans,
                plural_spans,
                pronoun_spans,
            );
        }
        oxc::Expression::JSXFragment(frag) => {
            collect_fbt_sub_tags(
                builder,
                &frag.children,
                tag_name,
                enum_spans,
                plural_spans,
                pronoun_spans,
            );
        }
        oxc::Expression::ConditionalExpression(cond) => {
            collect_fbt_sub_tags_from_expr(
                builder,
                &cond.consequent,
                tag_name,
                enum_spans,
                plural_spans,
                pronoun_spans,
            );
            collect_fbt_sub_tags_from_expr(
                builder,
                &cond.alternate,
                tag_name,
                enum_spans,
                plural_spans,
                pronoun_spans,
            );
        }
        oxc::Expression::LogicalExpression(log) => {
            collect_fbt_sub_tags_from_expr(
                builder,
                &log.left,
                tag_name,
                enum_spans,
                plural_spans,
                pronoun_spans,
            );
            collect_fbt_sub_tags_from_expr(
                builder,
                &log.right,
                tag_name,
                enum_spans,
                plural_spans,
                pronoun_spans,
            );
        }
        oxc::Expression::ParenthesizedExpression(paren) => {
            collect_fbt_sub_tags_from_expr(
                builder,
                &paren.expression,
                tag_name,
                enum_spans,
                plural_spans,
                pronoun_spans,
            );
        }
        oxc::Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                if let Some(oxc::Statement::ExpressionStatement(es)) = arrow.body.statements.first()
                {
                    collect_fbt_sub_tags_from_expr(
                        builder,
                        &es.expression,
                        tag_name,
                        enum_spans,
                        plural_spans,
                        pronoun_spans,
                    );
                }
            } else {
                collect_fbt_sub_tags_from_stmts(
                    builder,
                    &arrow.body.statements,
                    tag_name,
                    enum_spans,
                    plural_spans,
                    pronoun_spans,
                );
            }
        }
        oxc::Expression::CallExpression(call) => {
            for arg in &call.arguments {
                if let Some(arg_expr) = arg.as_expression() {
                    collect_fbt_sub_tags_from_expr(
                        builder,
                        arg_expr,
                        tag_name,
                        enum_spans,
                        plural_spans,
                        pronoun_spans,
                    );
                }
            }
        }
        _ => {}
    }
}

fn collect_fbt_sub_tags_from_stmts(
    builder: &HirBuilder<'_, '_>,
    stmts: &[oxc::Statement],
    tag_name: &str,
    enum_spans: &mut Vec<Option<Span>>,
    plural_spans: &mut Vec<Option<Span>>,
    pronoun_spans: &mut Vec<Option<Span>>,
) {
    for stmt in stmts {
        match stmt {
            oxc::Statement::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    collect_fbt_sub_tags_from_expr(
                        builder,
                        arg,
                        tag_name,
                        enum_spans,
                        plural_spans,
                        pronoun_spans,
                    );
                }
            }
            oxc::Statement::ExpressionStatement(expr_stmt) => {
                collect_fbt_sub_tags_from_expr(
                    builder,
                    &expr_stmt.expression,
                    tag_name,
                    enum_spans,
                    plural_spans,
                    pronoun_spans,
                );
            }
            _ => {}
        }
    }
}

/// Split a string on line endings, handling \r\n, \n, and \r.
fn split_line_endings(s: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            lines.push(&s[start..i]);
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                i += 2;
            } else {
                i += 1;
            }
            start = i;
        } else if bytes[i] == b'\n' {
            lines.push(&s[start..i]);
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }
    lines.push(&s[start..]);
    lines
}

/// Trims whitespace according to the JSX spec.
/// Implementation ported from Babel's cleanJSXElementLiteralChild.
fn trim_jsx_text(original: &str) -> Option<String> {
    // Split on \r\n, \n, or \r to handle all line ending styles (matching TS split(/\r\n|\n|\r/))
    let lines: Vec<&str> = split_line_endings(original);

    // NOTE: when builder.fbt_depth > 0, the TS skips whitespace trimming entirely.
    // That check is handled by the caller (lower_jsx_element) before calling this function.

    let mut last_non_empty_line = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.contains(|c: char| c != ' ' && c != '\t') {
            last_non_empty_line = i;
        }
    }

    let mut str = String::new();

    for (i, line) in lines.iter().enumerate() {
        let is_first_line = i == 0;
        let is_last_line = i == lines.len() - 1;
        let is_last_non_empty_line = i == last_non_empty_line;

        // Replace rendered whitespace tabs with spaces
        let mut trimmed_line = line.cow_replace('\t', " ").into_owned();

        // Trim whitespace touching a newline (leading whitespace on non-first lines)
        if !is_first_line {
            trimmed_line = trimmed_line.trim_start_matches(' ').to_string();
        }

        // Trim whitespace touching an endline (trailing whitespace on non-last lines)
        if !is_last_line {
            trimmed_line = trimmed_line.trim_end_matches(' ').to_string();
        }

        if !trimmed_line.is_empty() {
            if !is_last_non_empty_line {
                trimmed_line.push(' ');
            }
            str.push_str(&trimmed_line);
        }
    }

    if str.is_empty() { None } else { Some(str) }
}

/// Decode XML/HTML entities in JSX text (`&amp;` → `&`, `&gt;` → `>`, `&#123;`
/// → `{`, `&#x1F600;` → emoji, …) so the lowered JSX text/attribute value matches
/// Babel's decoded text. oxc keeps JSX text raw in the AST. Mirrors the
/// `decode_jsx_entities` helper in `convert_ast.rs`. Unrecognized `&…;` sequences
/// are kept verbatim.
fn decode_jsx_entities(s: &str) -> Cow<'_, str> {
    if !s.contains('&') {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.char_indices();
    let mut prev = 0;
    while let Some((i, c)) = chars.next() {
        if c != '&' {
            continue;
        }
        let mut start = i;
        let mut end = None;
        for (j, c) in chars.by_ref() {
            if c == ';' {
                end = Some(j);
                break;
            } else if c == '&' {
                start = j;
            }
        }
        let Some(end) = end else { break };
        out.push_str(&s[prev..start]);
        prev = end + 1;
        let word = &s[start + 1..end];
        let decoded = if let Some(num) = word.strip_prefix('#') {
            if let Some(hex) = num.strip_prefix(['x', 'X']) {
                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
            } else {
                num.parse::<u32>().ok().and_then(char::from_u32)
            }
        } else {
            oxc_syntax::xml_entities::XML_ENTITIES.get(word).copied()
        };
        match decoded {
            Some(c) => out.push(c),
            // Not a recognized entity — keep the `&…;` literal.
            None => {
                out.push('&');
                out.push_str(word);
                out.push(';');
            }
        }
    }
    out.push_str(&s[prev..]);
    Cow::Owned(out)
}

/// Get the Babel-style type name of an oxc `Expression` node. Mirrors the
/// original `expression_type_name` (which read Babel-shaped variants), mapping
/// oxc's split member/chain shapes back to the Babel names the original emitted
/// (e.g. `StaticMemberExpression`/`ComputedMemberExpression`/`PrivateFieldExpression`
/// → "MemberExpression"; `ChainExpression` → "OptionalMemberExpression").
fn expression_type_name(expr: &oxc::Expression) -> &'static str {
    match expr {
        oxc::Expression::Identifier(_) => "Identifier",
        oxc::Expression::StringLiteral(_) => "StringLiteral",
        oxc::Expression::NumericLiteral(_) => "NumericLiteral",
        oxc::Expression::BooleanLiteral(_) => "BooleanLiteral",
        oxc::Expression::NullLiteral(_) => "NullLiteral",
        oxc::Expression::BigIntLiteral(_) => "BigIntLiteral",
        oxc::Expression::RegExpLiteral(_) => "RegExpLiteral",
        oxc::Expression::CallExpression(_) => "CallExpression",
        oxc::Expression::StaticMemberExpression(_)
        | oxc::Expression::ComputedMemberExpression(_)
        | oxc::Expression::PrivateFieldExpression(_) => "MemberExpression",
        oxc::Expression::ChainExpression(_) => "OptionalMemberExpression",
        oxc::Expression::BinaryExpression(_) => "BinaryExpression",
        oxc::Expression::PrivateInExpression(_) => "BinaryExpression",
        oxc::Expression::LogicalExpression(_) => "LogicalExpression",
        oxc::Expression::UnaryExpression(_) => "UnaryExpression",
        oxc::Expression::UpdateExpression(_) => "UpdateExpression",
        oxc::Expression::ConditionalExpression(_) => "ConditionalExpression",
        oxc::Expression::AssignmentExpression(_) => "AssignmentExpression",
        oxc::Expression::SequenceExpression(_) => "SequenceExpression",
        oxc::Expression::ArrowFunctionExpression(_) => "ArrowFunctionExpression",
        oxc::Expression::FunctionExpression(_) => "FunctionExpression",
        oxc::Expression::ObjectExpression(_) => "ObjectExpression",
        oxc::Expression::ArrayExpression(_) => "ArrayExpression",
        oxc::Expression::NewExpression(_) => "NewExpression",
        oxc::Expression::TemplateLiteral(_) => "TemplateLiteral",
        oxc::Expression::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
        oxc::Expression::AwaitExpression(_) => "AwaitExpression",
        oxc::Expression::YieldExpression(_) => "YieldExpression",
        oxc::Expression::MetaProperty(_) => "MetaProperty",
        oxc::Expression::ClassExpression(_) => "ClassExpression",
        oxc::Expression::Super(_) => "Super",
        oxc::Expression::ImportExpression(_) => "Import",
        oxc::Expression::ThisExpression(_) => "ThisExpression",
        oxc::Expression::ParenthesizedExpression(_) => "ParenthesizedExpression",
        oxc::Expression::JSXElement(_) => "JSXElement",
        oxc::Expression::JSXFragment(_) => "JSXFragment",
        oxc::Expression::TSAsExpression(_) => "TSAsExpression",
        oxc::Expression::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
        oxc::Expression::TSNonNullExpression(_) => "TSNonNullExpression",
        oxc::Expression::TSTypeAssertion(_) => "TSTypeAssertion",
        oxc::Expression::TSInstantiationExpression(_) => "TSInstantiationExpression",
        oxc::Expression::V8IntrinsicExpression(_) => "V8IntrinsicExpression",
    }
}

/// Lower an oxc object getter/setter/method (`ObjectProperty` whose value is a
/// `FunctionExpression`). Faithful to the original `lower_object_method`:
/// `get`/`set` record a Todo error and are skipped. The `method` case lowers the
/// key and the nested function (`lower_function_for_object_method`) and emits an
/// `ObjectMethod` instruction value.
fn lower_object_method<'a>(
    builder: &mut HirBuilder<'a, '_>,
    method: &oxc::ObjectProperty<'a>,
) -> Result<Option<ObjectProperty<'a>>, OxcDiagnostic> {
    // In oxc, a shorthand method is encoded as `kind: Init, method: true`; only
    // getters/setters carry a non-`Init` `PropertyKind`.
    let is_method = method.method && matches!(method.kind, oxc::PropertyKind::Init);
    if !is_method {
        let kind_str = match method.kind {
            oxc::PropertyKind::Get => "get",
            oxc::PropertyKind::Set => "set",
            oxc::PropertyKind::Init => "method",
        };
        builder.record_error(
            ErrorCategory::Todo
                .diagnostic(format!(
                    "(BuildHIR::lowerExpression) Handle {} functions in ObjectExpression",
                    kind_str
                ))
                .with_label(method.span),
        )?;
        return Ok(None);
    }

    let key = lower_object_property_key(builder, &method.key, method.computed)?
        .unwrap_or(ObjectPropertyKey::String { name: Ident::empty() });

    let func = match &method.value {
        oxc::Expression::FunctionExpression(func) => func,
        _ => unreachable!("object method value is always a FunctionExpression in oxc"),
    };
    let body = func.body.as_ref().expect("object method always has a body");
    let lowered_func = lower_function_for_object_method(
        builder,
        method.span,
        func,
        &func.params,
        body,
        func.generator,
        func.r#async,
    )?;

    let span = Some(method.span);
    let method_value = InstructionValue::ObjectMethod { span, lowered_func };
    let method_place = lower_value_to_temporary(builder, method_value)?;

    Ok(Some(ObjectProperty { key, property_type: ObjectPropertyType::Method, place: method_place }))
}

/// Lower an object property key. Faithful to the original `lower_object_property_key`.
fn lower_object_property_key<'a>(
    builder: &mut HirBuilder<'a, '_>,
    key: &oxc::PropertyKey<'a>,
    computed: bool,
) -> Result<Option<ObjectPropertyKey<'a>>, OxcDiagnostic> {
    match key {
        oxc::PropertyKey::StringLiteral(lit) => {
            Ok(Some(ObjectPropertyKey::String { name: Ident::from(lit.value.as_str()) }))
        }
        oxc::PropertyKey::StaticIdentifier(ident) if !computed => {
            Ok(Some(ObjectPropertyKey::Identifier { name: ident.name }))
        }
        oxc::PropertyKey::Identifier(ident) if !computed => {
            Ok(Some(ObjectPropertyKey::Identifier { name: ident.name }))
        }
        oxc::PropertyKey::NumericLiteral(lit) if !computed => {
            Ok(Some(ObjectPropertyKey::Identifier {
                name: format_ident!(builder.environment().allocator, "{}", lit.value),
            }))
        }
        _ if computed => {
            let place = lower_expression_to_temporary(builder, key.to_expression())?;
            Ok(Some(ObjectPropertyKey::Computed { name: place }))
        }
        _ => {
            let span = match key {
                oxc::PropertyKey::StaticIdentifier(i) => Some(i.span),
                oxc::PropertyKey::Identifier(i) => Some(i.span),
                _ => None,
            };
            builder.record_error(
                ErrorCategory::Todo
                    .diagnostic("Unsupported key type in ObjectExpression")
                    .with_labels(span),
            )?;
            Ok(None)
        }
    }
}

/// Lower a reorderable expression. Faithful to the original
/// `lower_reorderable_expression`: records an error when the expression cannot be
/// safely reordered, then lowers it to a temporary regardless.
fn lower_reorderable_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    expr: &oxc::Expression<'a>,
) -> Result<Place, OxcDiagnostic> {
    if !is_reorderable_expression(builder, expr, true) {
        builder.record_error(
            ErrorCategory::Todo
                .diagnostic(format!(
                    "(BuildHIR::node.lowerReorderableExpression) Expression type `{}` cannot be safely reordered",
                    expression_type_name(expr)
                )).with_label(expr.span()),
        )?;
    }
    lower_expression_to_temporary(builder, expr)
}

/// Faithful to the original `is_reorderable_expression`. oxc's split member
/// shapes (Static/Computed/PrivateField) map to the original `MemberExpression`
/// arm; optional chains (`ChainExpression`) were not handled by the original
/// (`OptionalMemberExpression` fell to `_ => false`), so they return false here.
fn is_reorderable_expression(
    builder: &HirBuilder<'_, '_>,
    expr: &oxc::Expression,
    allow_local_identifiers: bool,
) -> bool {
    match expr {
        oxc::Expression::Identifier(ident) => {
            match builder.scope().resolve_reference(ident) {
                None => {
                    // global, safe to reorder
                    true
                }
                Some(symbol_id) => {
                    if builder.scope().symbol_scope(symbol_id) == builder.scope().program_scope() {
                        // Module-scope binding (ModuleLocal, imports), safe to reorder
                        true
                    } else {
                        allow_local_identifiers
                    }
                }
            }
        }
        oxc::Expression::RegExpLiteral(_)
        | oxc::Expression::StringLiteral(_)
        | oxc::Expression::NumericLiteral(_)
        | oxc::Expression::NullLiteral(_)
        | oxc::Expression::BooleanLiteral(_)
        | oxc::Expression::BigIntLiteral(_) => true,
        oxc::Expression::UnaryExpression(unary) => {
            matches!(
                unary.operator,
                oxc::UnaryOperator::LogicalNot
                    | oxc::UnaryOperator::UnaryPlus
                    | oxc::UnaryOperator::UnaryNegation
            ) && is_reorderable_expression(builder, &unary.argument, allow_local_identifiers)
        }
        oxc::Expression::LogicalExpression(logical) => {
            is_reorderable_expression(builder, &logical.left, allow_local_identifiers)
                && is_reorderable_expression(builder, &logical.right, allow_local_identifiers)
        }
        oxc::Expression::ConditionalExpression(cond) => {
            is_reorderable_expression(builder, &cond.test, allow_local_identifiers)
                && is_reorderable_expression(builder, &cond.consequent, allow_local_identifiers)
                && is_reorderable_expression(builder, &cond.alternate, allow_local_identifiers)
        }
        oxc::Expression::ArrayExpression(arr) => arr.elements.iter().all(|element| match element {
            oxc::ArrayExpressionElement::Elision(_) => false, // holes are not reorderable
            // A spread is a Babel `Expression::SpreadElement`, which the original
            // hit via the catch-all `_ => false` (no SpreadElement arm), so any
            // array containing a spread is not reorderable.
            oxc::ArrayExpressionElement::SpreadElement(_) => false,
            _ => {
                is_reorderable_expression(builder, element.to_expression(), allow_local_identifiers)
            }
        }),
        oxc::Expression::ObjectExpression(obj) => obj.properties.iter().all(|prop| match prop {
            oxc::ObjectPropertyKind::ObjectProperty(p) => {
                !p.computed
                    && !p.method
                    && matches!(p.kind, oxc::PropertyKind::Init)
                    && is_reorderable_expression(builder, &p.value, allow_local_identifiers)
            }
            _ => false,
        }),
        oxc::Expression::StaticMemberExpression(_)
        | oxc::Expression::ComputedMemberExpression(_)
        | oxc::Expression::PrivateFieldExpression(_) => {
            // Allow member expressions where the innermost object is a global or module-local
            let mut inner = expr;
            loop {
                inner = match inner {
                    oxc::Expression::StaticMemberExpression(m) => &m.object,
                    oxc::Expression::ComputedMemberExpression(m) => &m.object,
                    oxc::Expression::PrivateFieldExpression(m) => &m.object,
                    _ => break,
                };
            }
            if let oxc::Expression::Identifier(ident) = inner {
                match builder.scope().resolve_reference(ident) {
                    None => true, // global
                    Some(symbol_id) => {
                        // Module-scope bindings (ModuleLocal, imports) are safe to reorder
                        builder.scope().symbol_scope(symbol_id) == builder.scope().program_scope()
                    }
                }
            } else {
                false
            }
        }
        oxc::Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                match arrow.body.statements.first() {
                    Some(oxc::Statement::ExpressionStatement(es)) => {
                        is_reorderable_expression(builder, &es.expression, false)
                    }
                    _ => arrow.body.statements.is_empty(),
                }
            } else {
                arrow.body.statements.is_empty()
            }
        }
        oxc::Expression::CallExpression(call) => {
            is_reorderable_expression(builder, &call.callee, allow_local_identifiers)
                && call.arguments.iter().all(|arg| match arg {
                    // A spread argument is a Babel `Expression::SpreadElement`,
                    // which the original hit via the catch-all `_ => false`.
                    oxc::Argument::SpreadElement(_) => false,
                    _ => is_reorderable_expression(
                        builder,
                        arg.to_expression(),
                        allow_local_identifiers,
                    ),
                })
        }
        oxc::Expression::NewExpression(new_expr) => {
            is_reorderable_expression(builder, &new_expr.callee, allow_local_identifiers)
                && new_expr.arguments.iter().all(|arg| match arg {
                    // A spread argument is a Babel `Expression::SpreadElement`,
                    // which the original hit via the catch-all `_ => false`.
                    oxc::Argument::SpreadElement(_) => false,
                    _ => is_reorderable_expression(
                        builder,
                        arg.to_expression(),
                        allow_local_identifiers,
                    ),
                })
        }
        // TypeScript type wrappers: recurse into the inner expression.
        oxc::Expression::TSAsExpression(ts) => {
            is_reorderable_expression(builder, &ts.expression, allow_local_identifiers)
        }
        oxc::Expression::TSSatisfiesExpression(ts) => {
            is_reorderable_expression(builder, &ts.expression, allow_local_identifiers)
        }
        oxc::Expression::TSNonNullExpression(ts) => {
            is_reorderable_expression(builder, &ts.expression, allow_local_identifiers)
        }
        oxc::Expression::TSInstantiationExpression(ts) => {
            is_reorderable_expression(builder, &ts.expression, allow_local_identifiers)
        }
        oxc::Expression::TSTypeAssertion(ts) => {
            is_reorderable_expression(builder, &ts.expression, allow_local_identifiers)
        }
        oxc::Expression::ParenthesizedExpression(p) => {
            is_reorderable_expression(builder, &p.expression, allow_local_identifiers)
        }
        _ => false,
    }
}

fn lower_statement<'a>(
    builder: &mut HirBuilder<'a, '_>,
    stmt: &oxc::Statement<'a>,
    label: Option<Ident<'a>>,
    parent_scope: Option<crate::scope::ScopeId>,
) -> Result<(), OxcDiagnostic> {
    match stmt {
        oxc::Statement::EmptyStatement(_) => {}
        oxc::Statement::DebuggerStatement(dbg) => {
            let span = Some(dbg.span);
            lower_value_to_temporary(builder, InstructionValue::Debugger { span })?;
        }
        oxc::Statement::ExpressionStatement(expr_stmt) => {
            lower_expression_to_temporary(builder, &expr_stmt.expression)?;
        }
        oxc::Statement::ReturnStatement(ret) => {
            let span = Some(ret.span);
            let value = if let Some(arg) = &ret.argument {
                lower_expression_to_temporary(builder, arg)?
            } else {
                lower_value_to_temporary(
                    builder,
                    InstructionValue::Primitive { value: PrimitiveValue::Undefined, span: None },
                )?
            };
            let fallthrough = builder.reserve(BlockKind::Block);
            builder.terminate_with_continuation(
                Terminal::Return {
                    value,
                    return_variant: ReturnVariant::Explicit,
                    id: EvaluationOrder::UNSET,
                    span,
                    effects: None,
                },
                fallthrough,
            );
        }
        oxc::Statement::ThrowStatement(throw) => {
            let span = Some(throw.span);
            let value = lower_expression_to_temporary(builder, &throw.argument)?;
            if builder.resolve_throw_handler().is_some() {
                builder.record_error(
                    ErrorCategory::Todo
                        .diagnostic(
                            "(BuildHIR::lowerStatement) Support ThrowStatement inside of try/catch",
                        )
                        .with_labels(span),
                )?;
            }
            let fallthrough = builder.reserve(BlockKind::Block);
            builder.terminate_with_continuation(
                Terminal::Throw { value, id: EvaluationOrder::UNSET, span },
                fallthrough,
            );
        }
        oxc::Statement::BlockStatement(block) => {
            lower_block_statement(builder, &block.body, block.scope_id.get(), parent_scope)?;
        }
        oxc::Statement::VariableDeclaration(var_decl) => {
            lower_variable_declaration(builder, var_decl)?;
        }
        oxc::Statement::FunctionDeclaration(func_decl) if func_decl.body.is_some() => {
            lower_function_declaration(builder, func_decl)?;
        }
        oxc::Statement::IfStatement(if_stmt) => {
            let span = Some(if_stmt.span);
            // Block for code following the if
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Block for the consequent (if the test is truthy)
            let consequent_span = Some(if_stmt.consequent.span());
            let consequent_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                lower_statement(builder, &if_stmt.consequent, None, parent_scope)?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span: consequent_span,
                })
            })?;

            // Block for the alternate (if the test is not truthy)
            let alternate_block = if let Some(alternate) = &if_stmt.alternate {
                let alternate_span = Some(alternate.span());
                builder.try_enter(BlockKind::Block, |builder, _block_id| {
                    lower_statement(builder, alternate, None, parent_scope)?;
                    Ok(Terminal::Goto {
                        block: continuation_id,
                        variant: GotoVariant::Break,
                        id: EvaluationOrder::UNSET,
                        span: alternate_span,
                    })
                })?
            } else {
                // If there is no else clause, use the continuation directly
                continuation_id
            };

            let test = lower_expression_to_temporary(builder, &if_stmt.test)?;
            builder.terminate_with_continuation(
                Terminal::If {
                    test,
                    consequent: consequent_block,
                    alternate: alternate_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );
        }
        oxc::Statement::ForStatement(for_stmt) => {
            let span = Some(for_stmt.span);

            let test_block = builder.reserve(BlockKind::Loop);
            let test_block_id = test_block.id;
            // Block for code following the loop
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Init block: lower init expression/declaration, then goto test
            let init_block = builder.try_enter(BlockKind::Loop, |builder, _block_id| {
                let init_span = match &for_stmt.init {
                    None => {
                        // No init expression (e.g., `for (; ...)`), add a placeholder
                        let placeholder = InstructionValue::Primitive {
                            value: PrimitiveValue::Undefined,
                            span,
                        };
                        lower_value_to_temporary(builder, placeholder)?;
                        span
                    }
                    Some(oxc::ForStatementInit::VariableDeclaration(var_decl)) => {
                        let init_span = Some(var_decl.span);
                        lower_variable_declaration(builder, var_decl)?;
                        init_span
                    }
                    Some(init) => {
                        let expr = init.to_expression();
                        let init_span = Some(expr.span());
                                                builder.record_error(
                            ErrorCategory::Todo
                                .diagnostic("(BuildHIR::lowerStatement) Handle non-variable initialization in ForStatement")
                                .with_labels(span),
                        )?;
                        lower_expression_to_temporary(builder, expr)?;
                        init_span
                    }
                };
                Ok(Terminal::Goto {
                    block: test_block_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span: init_span,
                })
            })?;

            // Update block (optional)
            let update_block_id = if let Some(update) = &for_stmt.update {
                let update_span = Some(update.span());
                Some(builder.try_enter(BlockKind::Loop, |builder, _block_id| {
                    lower_expression_to_temporary(builder, update)?;
                    Ok(Terminal::Goto {
                        block: test_block_id,
                        variant: GotoVariant::Break,
                        id: EvaluationOrder::UNSET,
                        span: update_span,
                    })
                })?)
            } else {
                None
            };

            // Loop body block
            let continue_target = update_block_id.unwrap_or(test_block_id);
            let body_span = Some(for_stmt.body.span());
            let body_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(label, continue_target, continuation_id, |builder| {
                    lower_statement(builder, &for_stmt.body, None, parent_scope)?;
                    Ok(Terminal::Goto {
                        block: continue_target,
                        variant: GotoVariant::Continue,
                        id: EvaluationOrder::UNSET,
                        span: body_span,
                    })
                })
            })?;

            // Emit For terminal, then fill in the test block
            builder.terminate_with_continuation(
                Terminal::For {
                    init: init_block,
                    test: test_block_id,
                    update: update_block_id,
                    loop_block: body_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                test_block,
            );

            // Fill in the test block
            if let Some(test_expr) = &for_stmt.test {
                let test = lower_expression_to_temporary(builder, test_expr)?;
                builder.terminate_with_continuation(
                    Terminal::Branch {
                        test,
                        consequent: body_block,
                        alternate: continuation_id,
                        fallthrough: continuation_id,
                        id: EvaluationOrder::UNSET,
                        span,
                    },
                    continuation_block,
                );
            } else {
                builder.record_error(
                    ErrorCategory::Todo
                        .diagnostic("(BuildHIR::lowerStatement) Handle empty test in ForStatement")
                        .with_labels(span),
                )?;
                // Treat `for(;;)` as `while(true)` to keep the builder state consistent
                let true_val =
                    InstructionValue::Primitive { value: PrimitiveValue::Boolean(true), span };
                let test = lower_value_to_temporary(builder, true_val)?;
                builder.terminate_with_continuation(
                    Terminal::Branch {
                        test,
                        consequent: body_block,
                        alternate: continuation_id,
                        fallthrough: continuation_id,
                        id: EvaluationOrder::UNSET,
                        span,
                    },
                    continuation_block,
                );
            }
        }
        oxc::Statement::WhileStatement(while_stmt) => {
            let span = Some(while_stmt.span);
            // Block used to evaluate whether to (re)enter or exit the loop
            let conditional_block = builder.reserve(BlockKind::Loop);
            let conditional_id = conditional_block.id;
            // Block for code following the loop
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Loop body
            let body_span = Some(while_stmt.body.span());
            let loop_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(label, conditional_id, continuation_id, |builder| {
                    lower_statement(builder, &while_stmt.body, None, parent_scope)?;
                    Ok(Terminal::Goto {
                        block: conditional_id,
                        variant: GotoVariant::Continue,
                        id: EvaluationOrder::UNSET,
                        span: body_span,
                    })
                })
            })?;

            // Emit While terminal, jumping to the conditional block
            builder.terminate_with_continuation(
                Terminal::While {
                    test: conditional_id,
                    loop_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                conditional_block,
            );

            // Fill in the conditional block: lower test, branch
            let test = lower_expression_to_temporary(builder, &while_stmt.test)?;
            builder.terminate_with_continuation(
                Terminal::Branch {
                    test,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: conditional_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );
        }
        oxc::Statement::DoWhileStatement(do_while_stmt) => {
            let span = Some(do_while_stmt.span);
            // Block used to evaluate whether to (re)enter or exit the loop
            let conditional_block = builder.reserve(BlockKind::Loop);
            let conditional_id = conditional_block.id;
            // Block for code following the loop
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Loop body, executed at least once unconditionally prior to exit
            let body_span = Some(do_while_stmt.body.span());
            let loop_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(label, conditional_id, continuation_id, |builder| {
                    lower_statement(builder, &do_while_stmt.body, None, parent_scope)?;
                    Ok(Terminal::Goto {
                        block: conditional_id,
                        variant: GotoVariant::Continue,
                        id: EvaluationOrder::UNSET,
                        span: body_span,
                    })
                })
            })?;

            // Jump to the conditional block
            builder.terminate_with_continuation(
                Terminal::DoWhile {
                    loop_block,
                    test: conditional_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                conditional_block,
            );

            // Fill in the conditional block: lower test, branch
            let test = lower_expression_to_temporary(builder, &do_while_stmt.test)?;
            builder.terminate_with_continuation(
                Terminal::Branch {
                    test,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: conditional_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );
        }
        oxc::Statement::ForInStatement(for_in) => {
            let span = Some(for_in.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;
            let init_block = builder.reserve(BlockKind::Loop);
            let init_block_id = init_block.id;

            let body_span = Some(for_in.body.span());
            let loop_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(label, init_block_id, continuation_id, |builder| {
                    lower_statement(builder, &for_in.body, None, parent_scope)?;
                    Ok(Terminal::Goto {
                        block: init_block_id,
                        variant: GotoVariant::Continue,
                        id: EvaluationOrder::UNSET,
                        span: body_span,
                    })
                })
            })?;

            let value = lower_expression_to_temporary(builder, &for_in.right)?;
            builder.terminate_with_continuation(
                Terminal::ForIn {
                    init: init_block_id,
                    loop_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                init_block,
            );

            // Lower the init: NextPropertyOf + assignment
            let left_span = for_in.left.span();
            let next_property = lower_value_to_temporary(
                builder,
                InstructionValue::NextPropertyOf { value, span: Some(left_span) },
            )?;

            let assign_result =
                lower_for_in_of_left(builder, &for_in.left, left_span, next_property.clone())?;
            // Use the assign result (StoreLocal temp) as the test, matching TS behavior
            let test_value = assign_result.unwrap_or(next_property);
            let test = lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal { place: test_value, span: Some(left_span) },
            )?;
            builder.terminate_with_continuation(
                Terminal::Branch {
                    test,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );
        }
        oxc::Statement::ForOfStatement(for_of) => {
            let span = Some(for_of.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;
            let init_block = builder.reserve(BlockKind::Loop);
            let init_block_id = init_block.id;
            let test_block = builder.reserve(BlockKind::Loop);
            let test_block_id = test_block.id;

            if for_of.r#await {
                builder.record_error(
                    ErrorCategory::Todo
                        .diagnostic("(BuildHIR::lowerStatement) Handle for-await loops")
                        .with_labels(span),
                )?;
                return Ok(());
            }

            let body_span = Some(for_of.body.span());
            let loop_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(label, init_block_id, continuation_id, |builder| {
                    lower_statement(builder, &for_of.body, None, parent_scope)?;
                    Ok(Terminal::Goto {
                        block: init_block_id,
                        variant: GotoVariant::Continue,
                        id: EvaluationOrder::UNSET,
                        span: body_span,
                    })
                })
            })?;

            let value = lower_expression_to_temporary(builder, &for_of.right)?;
            builder.terminate_with_continuation(
                Terminal::ForOf {
                    init: init_block_id,
                    test: test_block_id,
                    loop_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                init_block,
            );

            // Init block: GetIterator, goto test
            let iterator = lower_value_to_temporary(
                builder,
                InstructionValue::GetIterator { collection: value.clone(), span: value.span },
            )?;
            builder.terminate_with_continuation(
                Terminal::Goto {
                    block: test_block_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                test_block,
            );

            // Test block: IteratorNext, assign, branch
            let left_span = for_of.left.span();
            let advance_iterator = lower_value_to_temporary(
                builder,
                InstructionValue::IteratorNext {
                    iterator,
                    collection: value,
                    span: Some(left_span),
                },
            )?;

            let assign_result =
                lower_for_in_of_left(builder, &for_of.left, left_span, advance_iterator.clone())?;
            // Use the assign result (StoreLocal temp) as the test, matching TS behavior
            let test_value = assign_result.unwrap_or(advance_iterator);
            let test = lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal { place: test_value, span: Some(left_span) },
            )?;
            builder.terminate_with_continuation(
                Terminal::Branch {
                    test,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );
        }
        oxc::Statement::SwitchStatement(switch_stmt) => {
            let span = Some(switch_stmt.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Iterate through cases in reverse order so that previous blocks can
            // fallthrough to successors
            let mut fallthrough = continuation_id;
            let mut cases: Vec<Case> = Vec::new();
            let mut has_default = false;

            for ii in (0..switch_stmt.cases.len()).rev() {
                let case = &switch_stmt.cases[ii];
                let case_span = Some(case.span);

                if case.test.is_none() {
                    if has_default {
                        builder.record_error(
                            ErrorCategory::Syntax
                                .diagnostic(
                                    "Expected at most one `default` branch in a switch statement",
                                )
                                .with_labels(case_span),
                        )?;
                        break;
                    }
                    has_default = true;
                }

                let fallthrough_target = fallthrough;
                let block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                    builder.switch_scope(label, continuation_id, |builder| {
                        for consequent in &case.consequent {
                            lower_statement(builder, consequent, None, parent_scope)?;
                        }
                        Ok(Terminal::Goto {
                            block: fallthrough_target,
                            variant: GotoVariant::Break,
                            id: EvaluationOrder::UNSET,
                            span: case_span,
                        })
                    })
                })?;

                let test = if let Some(test_expr) = &case.test {
                    Some(lower_reorderable_expression(builder, test_expr)?)
                } else {
                    None
                };

                cases.push(Case { test, block });
                fallthrough = block;
            }

            // Reverse back to original order
            cases.reverse();

            // If no default case, add one that jumps to continuation
            if !has_default {
                cases.push(Case { test: None, block: continuation_id });
            }

            let test = lower_expression_to_temporary(builder, &switch_stmt.discriminant)?;
            builder.terminate_with_continuation(
                Terminal::Switch {
                    test,
                    cases,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );
        }
        oxc::Statement::TryStatement(try_stmt) => {
            let span = Some(try_stmt.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            let handler_clause = match &try_stmt.handler {
                Some(h) => h,
                None => {
                    builder.record_error(
                        ErrorCategory::Todo
                            .diagnostic("(BuildHIR::lowerStatement) Handle TryStatement without a catch clause")
                            .with_labels(span),
                    )?;
                    return Ok(());
                }
            };

            if try_stmt.finalizer.is_some() {
                builder.record_error(
                    ErrorCategory::Todo
                        .diagnostic("(BuildHIR::lowerStatement) Handle TryStatement with a finalizer ('finally') clause")
                        .with_labels(span),
                )?;
            }

            // Set up handler binding if catch has a param
            let handler_binding_info: Option<(Place, &oxc::BindingPattern)> = if let Some(param) =
                &handler_clause.param
            {
                // Check for destructuring in catch clause params.
                // Match TS behavior: Babel doesn't register destructured catch bindings
                // in its scope, so resolveIdentifier fails and records an invariant error.
                let is_destructuring = matches!(
                    &param.pattern,
                    oxc::BindingPattern::ObjectPattern(_) | oxc::BindingPattern::ArrayPattern(_)
                );
                if is_destructuring {
                    let mut id_spans = Vec::new();
                    collect_catch_pattern_identifier_spans(&param.pattern, &mut id_spans);
                    for id_span in id_spans {
                        builder.record_error(
                            ErrorCategory::Invariant
                                .diagnostic("(BuildHIR::lowerAssignment) Could not find binding for declaration.")
                                .with_label(id_span),
                        )?;
                    }
                    None
                } else {
                    let param_span = Some(param.pattern.span());
                    let id = builder.make_temporary(param_span);
                    promote_temporary(builder, id);
                    let place = Place {
                        identifier: id,
                        effect: Effect::Unknown,
                        reactive: false,
                        span: param_span,
                    };
                    // Emit DeclareLocal for the catch binding
                    lower_value_to_temporary(
                        builder,
                        InstructionValue::DeclareLocal {
                            lvalue: LValue { kind: InstructionKind::Catch, place: place.clone() },
                            span: param_span,
                        },
                    )?;
                    Some((place, &param.pattern))
                }
            } else {
                None
            };

            // Create the handler (catch) block
            let handler_binding_for_block = handler_binding_info.clone();
            let handler_span = handler_clause.span;
            // Use the catch param's span for the assignment, matching TS: handlerBinding.path.node.span
            let handler_param_span = handler_clause.param.as_ref().map(|p| p.pattern.span());
            let handler_block = builder.try_enter(BlockKind::Catch, |builder, _block_id| {
                if let Some((ref place, pattern)) = handler_binding_for_block {
                    lower_binding_assignment(
                        builder,
                        handler_param_span.unwrap_or(handler_span),
                        InstructionKind::Catch,
                        pattern,
                        place.clone(),
                        AssignmentStyle::Assignment,
                    )?;
                }
                // Lower the catch body using lower_block_statement to get hoisting support.
                // Use the catch clause's scope (which contains the catch param binding).
                // Fall back to the body block's own scope if the catch clause scope is missing.
                let catch_scope =
                    handler_clause.scope_id.get().or_else(|| handler_clause.body.scope_id.get());
                if let Some(scope_id) = catch_scope {
                    lower_block_statement_with_scope(builder, &handler_clause.body.body, scope_id)?;
                } else {
                    lower_block_statement(
                        builder,
                        &handler_clause.body.body,
                        handler_clause.body.scope_id.get(),
                        parent_scope,
                    )?;
                }
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span: Some(handler_span),
                })
            })?;

            // Create the try block
            let try_body_span = Some(try_stmt.block.span);
            let try_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.try_enter_try_catch(handler_block, |builder| {
                    lower_block_statement(
                        builder,
                        &try_stmt.block.body,
                        try_stmt.block.scope_id.get(),
                        parent_scope,
                    )?;
                    Ok(())
                })?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Try,
                    id: EvaluationOrder::UNSET,
                    span: try_body_span,
                })
            })?;

            builder.terminate_with_continuation(
                Terminal::Try {
                    block: try_block,
                    handler_binding: handler_binding_info.map(|(place, _)| place),
                    handler: handler_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                continuation_block,
            );
        }
        oxc::Statement::BreakStatement(brk) => {
            let span = Some(brk.span);
            let label_name = brk.label.as_ref().map(|l| l.name.as_str());
            let target = builder.lookup_break(label_name)?;
            let fallthrough = builder.reserve(BlockKind::Block);
            builder.terminate_with_continuation(
                Terminal::Goto {
                    block: target,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                fallthrough,
            );
        }
        oxc::Statement::ContinueStatement(cont) => {
            let span = Some(cont.span);
            let label_name = cont.label.as_ref().map(|l| l.name.as_str());
            let target = builder.lookup_continue(label_name)?;
            let fallthrough = builder.reserve(BlockKind::Block);
            builder.terminate_with_continuation(
                Terminal::Goto {
                    block: target,
                    variant: GotoVariant::Continue,
                    id: EvaluationOrder::UNSET,
                    span,
                },
                fallthrough,
            );
        }
        oxc::Statement::LabeledStatement(labeled_stmt) => {
            let label_name = labeled_stmt.label.name;
            let span = Some(labeled_stmt.span);

            // Check if the body is a loop statement - if so, delegate with label
            match &labeled_stmt.body {
                oxc::Statement::ForStatement(_)
                | oxc::Statement::WhileStatement(_)
                | oxc::Statement::DoWhileStatement(_)
                | oxc::Statement::ForInStatement(_)
                | oxc::Statement::ForOfStatement(_) => {
                    // Labeled loops are special because of continue, push the label down
                    lower_statement(builder, &labeled_stmt.body, Some(label_name), parent_scope)?;
                }
                _ => {
                    // All other statements create a continuation block to allow `break`
                    let continuation_block = builder.reserve(BlockKind::Block);
                    let continuation_id = continuation_block.id;
                    let body_span = Some(labeled_stmt.body.span());
                    let block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                        builder.label_scope(label_name, continuation_id, |builder| {
                            lower_statement(builder, &labeled_stmt.body, None, parent_scope)?;
                            Ok(())
                        })?;
                        Ok(Terminal::Goto {
                            block: continuation_id,
                            variant: GotoVariant::Break,
                            id: EvaluationOrder::UNSET,
                            span: body_span,
                        })
                    })?;

                    builder.terminate_with_continuation(
                        Terminal::Label {
                            block,
                            fallthrough: continuation_id,
                            id: EvaluationOrder::UNSET,
                            span,
                        },
                        continuation_block,
                    );
                }
            }
        }
        oxc::Statement::WithStatement(with_stmt) => {
            builder.record_error(
                ErrorCategory::UnsupportedSyntax
                    .diagnostic("JavaScript 'with' syntax is not supported")
                    .with_help("'with' syntax is considered deprecated and removed from JavaScript standards, consider alternatives").with_label(with_stmt.span),
            )?;
        }
        oxc::Statement::ClassDeclaration(cls) => {
            builder.record_error(
                ErrorCategory::UnsupportedSyntax
                    .diagnostic("Inline `class` declarations are not supported")
                    .with_help("Move class declarations outside of components/hooks")
                    .with_label(cls.span),
            )?;
        }
        oxc::Statement::ImportDeclaration(_)
        | oxc::Statement::ExportNamedDeclaration(_)
        | oxc::Statement::ExportDefaultDeclaration(_)
        | oxc::Statement::ExportAllDeclaration(_) => {
            builder.record_error(
                ErrorCategory::Syntax
                    .diagnostic("JavaScript `import` and `export` statements may only appear at the top level of a module").with_label(stmt.span()),
            )?;
        }
        oxc::Statement::TSEnumDeclaration(_) => {
            // Inline TS `enum` has runtime semantics but no HIR representation, and
            // the compiled body is rebuilt from HIR. Flag the function to be skipped
            // (silently, no diagnostic) once lowering finishes, rather than dropping
            // the enum from the output. Other functions in the file are unaffected.
            builder.environment_mut().skip_compilation = true;
        }
        _ => {
            // Remaining statements are skipped: bodyless FunctionDeclaration
            // (== Babel TSDeclareFunction), TS/Flow type-only declarations
            // (TSTypeAlias/TSInterface/TSModule/TSGlobal/TSImportEquals/
            // TSExportAssignment/TSNamespaceExport). The Flow `EnumDeclaration`
            // arm is moot since oxc has no Flow enum.
        }
    }
    Ok(())
}

/// Lower a `VariableDeclaration`, mirroring the original `Statement::VariableDeclaration`
/// arm (extracted so the `ForStatement` init can reuse it without re-wrapping in a
/// `Statement`).
fn lower_variable_declaration<'a>(
    builder: &mut HirBuilder<'a, '_>,
    var_decl: &oxc::VariableDeclaration<'a>,
) -> Result<(), OxcDiagnostic> {
    use oxc::VariableDeclarationKind as VK;
    if matches!(var_decl.kind, VK::Var) {
        builder.record_error(
            ErrorCategory::Todo
                .diagnostic("(BuildHIR::lowerStatement) Handle var kinds in VariableDeclaration")
                .with_label(var_decl.span),
        )?;
        // Treat `var` as `let` so references to the variable don't break
    }
    if matches!(var_decl.kind, VK::Using | VK::AwaitUsing) {
        // `using`/`await using` disposal semantics aren't preserved yet. Flag the
        // function to be skipped (silently, no diagnostic) once lowering finishes,
        // rather than miscompiling it. It's still lowered as `const` below so the HIR
        // stays valid until the pipeline checks the flag. Other functions in the file
        // are unaffected.
        builder.environment_mut().skip_compilation = true;
    }
    let kind = match var_decl.kind {
        VK::Let | VK::Var => InstructionKind::Let,
        VK::Const | VK::Using | VK::AwaitUsing => InstructionKind::Const,
    };
    for declarator in &var_decl.declarations {
        if let Some(init) = &declarator.init {
            let value = lower_expression_to_temporary(builder, init)?;
            let assign_style = match &declarator.id {
                oxc::BindingPattern::ObjectPattern(_) | oxc::BindingPattern::ArrayPattern(_) => {
                    AssignmentStyle::Destructure
                }
                _ => AssignmentStyle::Assignment,
            };
            lower_binding_assignment(
                builder,
                var_decl.span,
                kind,
                &declarator.id,
                value,
                assign_style,
            )?;
        } else if let oxc::BindingPattern::BindingIdentifier(id) = &declarator.id {
            // No init: emit DeclareLocal or DeclareContext
            let id_span = id.span;
            let mut binding = builder.resolve_identifier(
                id.name,
                id_span,
                builder.scope().resolve_binding_identifier(id),
            )?;
            if !matches!(binding, VariableBinding::Identifier { .. }) {
                // Direct resolution failed (synthetic $$gen vars). Try scope lookup
                // including descendants.
                if let Some(symbol_id) = builder
                    .scope()
                    .find_binding_in_descendants(id.name.as_str(), builder.function_scope())
                {
                    let binding_kind = crate::react_compiler_lowering::convert_binding_kind(
                        &builder.scope().binding_kind(symbol_id),
                    );
                    let identifier =
                        builder.resolve_binding_with_span(id.name, symbol_id, Some(id_span))?;
                    binding = VariableBinding::Identifier { identifier, binding_kind };
                }
            }
            match binding {
                VariableBinding::Identifier { identifier, .. } => {
                    // Update the identifier's span to the declaration site
                    // (it may have been first created at a reference site during hoisting)
                    builder.set_identifier_declaration_span(identifier, id_span);
                    let place = Place {
                        identifier,
                        effect: Effect::Unknown,
                        reactive: false,
                        span: Some(id_span),
                    };
                    if builder.is_context_identifier(builder.scope().resolve_binding_identifier(id))
                    {
                        if kind == InstructionKind::Const {
                            builder.record_error(
                                ErrorCategory::Syntax
                                    .diagnostic("Expect `const` declaration not to be reassigned")
                                    .with_label(id_span),
                            )?;
                        }
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::DeclareContext {
                                lvalue: LValue { kind: InstructionKind::Let, place },
                                span: Some(id_span),
                            },
                        )?;
                    } else {
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::DeclareLocal {
                                lvalue: LValue { kind, place },
                                span: Some(id_span),
                            },
                        )?;
                    }
                }
                _ => {
                    builder.record_error(
                        ErrorCategory::Invariant
                            .diagnostic("Could not find binding for declaration")
                            .with_label(id_span),
                    )?;
                }
            }
        } else {
            builder.record_error(
                ErrorCategory::Syntax
                    .diagnostic("Expected variable declaration to be an identifier if no initializer was provided").with_label(declarator.span),
            )?;
        }
    }
    Ok(())
}

/// Lower the `left` target of a for-in / for-of loop, dispatching to the binding
/// assignment (for `VariableDeclaration`) or assignment-target (for plain
/// patterns) lowering. Mirrors the original `ForInOfLeft` match.
fn lower_for_in_of_left<'a>(
    builder: &mut HirBuilder<'a, '_>,
    left: &oxc::ForStatementLeft<'a>,
    left_span: Span,
    value: Place,
) -> Result<Option<Place>, OxcDiagnostic> {
    match left {
        oxc::ForStatementLeft::VariableDeclaration(var_decl) => {
            if var_decl.declarations.len() != 1 {
                builder.record_error(
                    ErrorCategory::Invariant
                        .diagnostic(format!(
                            "Expected only one declaration in for-in/of init, got {}",
                            var_decl.declarations.len()
                        ))
                        .with_label(left_span),
                )?;
            }
            if let Some(declarator) = var_decl.declarations.first() {
                lower_binding_assignment(
                    builder,
                    left_span,
                    InstructionKind::Let,
                    &declarator.id,
                    value,
                    AssignmentStyle::Assignment,
                )
            } else {
                Ok(None)
            }
        }
        _ => lower_assignment_target(
            builder,
            left_span,
            InstructionKind::Reassign,
            left.to_assignment_target(),
            value,
            AssignmentStyle::Assignment,
        ),
    }
}

/// Collect identifier locs from a destructured catch-clause pattern, for error
/// reporting (Babel doesn't register destructured catch bindings).
fn collect_catch_pattern_identifier_spans(pat: &oxc::BindingPattern, locs: &mut Vec<Span>) {
    match pat {
        oxc::BindingPattern::BindingIdentifier(id) => {
            locs.push(id.span);
        }
        oxc::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_catch_pattern_identifier_spans(&prop.value, locs);
            }
            if let Some(rest) = &obj.rest {
                collect_catch_pattern_identifier_spans(&rest.argument, locs);
            }
        }
        oxc::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_catch_pattern_identifier_spans(elem, locs);
            }
            if let Some(rest) = &arr.rest {
                collect_catch_pattern_identifier_spans(&rest.argument, locs);
            }
        }
        // The original matched only Identifier/Object/Array; AssignmentPattern
        // (destructuring defaults) fell through its `_ => {}` catch-all.
        oxc::BindingPattern::AssignmentPattern(_) => {}
    }
}
