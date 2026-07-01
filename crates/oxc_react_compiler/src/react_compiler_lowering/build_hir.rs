use rustc_hash::FxHashSet;

use crate::react_compiler_diagnostics::CompilerDiagnostic;
use crate::react_compiler_diagnostics::CompilerDiagnosticDetail;
use crate::react_compiler_diagnostics::CompilerError;
use crate::react_compiler_diagnostics::CompilerErrorDetail;
use crate::react_compiler_diagnostics::ErrorCategory;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::*;
use crate::react_compiler_utils::FxIndexMap;
use crate::react_compiler_utils::FxIndexSet;
use crate::scope::BindingId;
use crate::scope::BindingKind as AstBindingKind;
use crate::scope::ScopeId;
use crate::scope::ScopeInfo;
use crate::scope::ScopeKind;

use oxc_ast::ast as oxc;
use oxc_span::GetSpan;

use crate::react_compiler_lowering::FunctionNode;
use crate::react_compiler_lowering::find_context_identifiers::find_context_identifiers;
use crate::react_compiler_lowering::hir_builder::HirBuilder;
use crate::react_compiler_lowering::hir_builder::is_always_reserved_word;
use crate::react_compiler_lowering::hir_builder::reserved_identifier_diagnostic;
use crate::react_compiler_lowering::identifier_loc_index::IdentifierLocIndex;
use crate::react_compiler_lowering::identifier_loc_index::build_identifier_loc_index;
use crate::react_compiler_lowering::source_loc::LineOffsets;

fn validate_ts_this_parameter(
    scope_info: &ScopeInfo,
    function_scope: ScopeId,
) -> Result<(), CompilerError> {
    let Some(scope) = scope_info.scopes.get(function_scope.0 as usize) else {
        return Ok(());
    };
    let Some(binding_id) = scope.bindings.get("this") else {
        return Ok(());
    };
    let Some(binding) = scope_info.bindings.get(binding_id.0 as usize) else {
        return Ok(());
    };
    if matches!(binding.kind, AstBindingKind::Param) {
        return Err(CompilerError::from(reserved_identifier_diagnostic("this")));
    }
    Ok(())
}

fn is_class_scope_descendant(scope_info: &ScopeInfo, mut scope_id: ScopeId) -> bool {
    while let Some(scope) = scope_info.scopes.get(scope_id.0 as usize) {
        let Some(parent) = scope.parent else {
            return false;
        };
        let Some(parent_scope) = scope_info.scopes.get(parent.0 as usize) else {
            return false;
        };
        if matches!(parent_scope.kind, ScopeKind::Class) {
            return true;
        }
        scope_id = parent;
    }
    false
}

fn validate_ts_this_parameters_in_function_range(
    scope_info: &ScopeInfo,
    start: u32,
    end: u32,
) -> Result<(), CompilerError> {
    if start >= end {
        return Ok(());
    }
    for (node_start, scope_id) in &scope_info.node_to_scope {
        if *node_start < start || *node_start >= end {
            continue;
        }
        let Some(scope) = scope_info.scopes.get(scope_id.0 as usize) else {
            continue;
        };
        if !matches!(scope.kind, ScopeKind::Function)
            || is_class_scope_descendant(scope_info, *scope_id)
        {
            continue;
        }
        validate_ts_this_parameter(scope_info, *scope_id)?;
    }
    Ok(())
}

/// Get the Babel-style type name of an Expression node (e.g. "Identifier", "NumericLiteral").
fn build_temporary_place(builder: &mut HirBuilder<'_, '_>, loc: Option<SourceLocation>) -> Place {
    let id = builder.make_temporary(loc.clone());
    Place { identifier: id, reactive: false, effect: Effect::Unknown, loc }
}

/// Promote a temporary identifier to a named identifier (for destructuring).
/// Corresponds to TS `promoteTemporary(identifier)`.
fn promote_temporary(builder: &mut HirBuilder<'_, '_>, identifier_id: IdentifierId) {
    let env = builder.environment_mut();
    let decl_id = env.identifiers[identifier_id.0 as usize].declaration_id;
    env.identifiers[identifier_id.0 as usize].name =
        Some(IdentifierName::Promoted(format!("#t{}", decl_id.0)));
}

fn lower_value_to_temporary<'a>(
    builder: &mut HirBuilder<'a, '_>,
    value: InstructionValue<'a>,
) -> Result<Place, CompilerError> {
    // Optimization: if loading an unnamed temporary, skip creating a new instruction
    if let InstructionValue::LoadLocal { ref place, .. } = value {
        let ident = &builder.environment().identifiers[place.identifier.0 as usize];
        if ident.name.is_none() {
            return Ok(place.clone());
        }
    }
    let loc = value.loc().cloned();
    let place = build_temporary_place(builder, loc.clone());
    builder.push(Instruction {
        id: EvaluationOrder(0),
        lvalue: place.clone(),
        value,
        loc,
        effects: None,
    });
    Ok(place)
}

fn lower_expression_to_temporary<'a>(
    builder: &mut HirBuilder<'a, '_>,
    expr: &'a oxc::Expression<'a>,
) -> Result<Place, CompilerError> {
    let value = lower_expression(builder, expr)?;
    lower_value_to_temporary(builder, value)
}

// =============================================================================
// Operator conversion
// =============================================================================

fn is_binding_in_block_direct_statements(
    binding: &crate::scope::BindingData,
    stmts: &[oxc::Statement],
) -> bool {
    let decl_start = match binding.declaration_start {
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
    scope_id: crate::scope::ScopeId,
    scope_info: &ScopeInfo,
    out: &mut FxHashSet<BindingId>,
) {
    match pattern {
        oxc::BindingPattern::BindingIdentifier(id) => {
            if let Some(&binding_id) =
                scope_info.scopes[scope_id.0 as usize].bindings.get(id.name.as_str())
            {
                out.insert(binding_id);
            }
        }
        oxc::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_names_from_pattern(&prop.value, scope_id, scope_info, out);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_names_from_pattern(&rest.argument, scope_id, scope_info, out);
            }
        }
        oxc::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_names_from_pattern(elem, scope_id, scope_info, out);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_names_from_pattern(&rest.argument, scope_id, scope_info, out);
            }
        }
        oxc::BindingPattern::AssignmentPattern(assign) => {
            collect_binding_names_from_pattern(&assign.left, scope_id, scope_info, out);
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
    statements: &'a [oxc::Statement<'a>],
    block_node_id: u32,
    parent_scope: Option<crate::scope::ScopeId>,
) -> Result<(), CompilerError> {
    let _ = lower_block_statement_inner(builder, statements, block_node_id, None, parent_scope);
    Ok(())
}

fn lower_block_statement_with_scope<'a>(
    builder: &mut HirBuilder<'a, '_>,
    statements: &'a [oxc::Statement<'a>],
    block_node_id: u32,
    scope_override: crate::scope::ScopeId,
) -> Result<(), CompilerError> {
    let _ =
        lower_block_statement_inner(builder, statements, block_node_id, Some(scope_override), None);
    Ok(())
}

fn lower_block_statement_inner<'a>(
    builder: &mut HirBuilder<'a, '_>,
    statements: &'a [oxc::Statement<'a>],
    block_node_id: u32,
    scope_override: Option<crate::scope::ScopeId>,
    parent_scope: Option<crate::scope::ScopeId>,
) -> Result<(), CompilerDiagnostic> {
    use crate::scope::BindingKind as AstBindingKind;

    // Look up the block's scope to identify hoistable bindings.
    // Use the scope override if provided (for function body blocks that share the function's scope).
    let block_scope_id = scope_override.or_else(|| {
        let found = builder.scope_info().resolve_scope_for_node(Some(block_node_id));
        if found.is_some() {
            return found;
        }
        // Fallback for synthetic blocks (start=0 from Hermes match desugar):
        // find a descendant scope of the parent that contains the block's declarations.
        let mut decl_names = Vec::new();
        for stmt in statements {
            if let oxc::Statement::VariableDeclaration(vd) = stmt {
                for d in &vd.declarations {
                    if let oxc::BindingPattern::BindingIdentifier(id) = &d.id {
                        decl_names.push(id.name.as_str());
                    }
                }
            }
        }
        if decl_names.is_empty() {
            return None;
        }
        let search_parent = parent_scope.unwrap_or_else(|| builder.function_scope());
        let found =
            builder.scope_info().find_block_scope_by_bindings(&decl_names, search_parent, |sid| {
                builder.is_synthetic_scope_claimed(sid)
            });
        if let Some(sid) = found {
            builder.claim_synthetic_scope(sid);
        }
        found
    });

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
    let hoistable: Vec<(BindingId, String, AstBindingKind, String, Option<u32>, Option<u32>)> =
        builder
            .scope_info()
            .scope_bindings_with_children(scope_id)
            .filter(|b| {
                !matches!(b.kind, AstBindingKind::Param | AstBindingKind::Module)
                    && b.declaration_type != "FunctionExpression"
                    && b.declaration_type != "TypeAlias"
                    && b.declaration_type != "OpaqueType"
                    && b.declaration_type != "InterfaceDeclaration"
                    && b.declaration_type != "TSTypeAliasDeclaration"
                    && b.declaration_type != "TSInterfaceDeclaration"
                    && b.declaration_type != "TSEnumDeclaration"
            })
            .map(|b| {
                (
                    b.id,
                    b.name.clone(),
                    b.kind.clone(),
                    b.declaration_type.clone(),
                    b.declaration_start,
                    b.declaration_node_id,
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
    let mut declared: FxHashSet<BindingId> = FxHashSet::default();

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
            let scope_info = builder.scope_info();
            scope_info
                .node_to_scope
                .iter()
                .filter(|&(&pos, &sid)| {
                    pos > stmt_start
                        && pos < stmt_end
                        && matches!(scope_info.scopes[sid.0 as usize].kind, ScopeKind::Function)
                })
                .filter_map(|(&pos, _)| {
                    scope_info.node_to_scope_end.get(&pos).map(|&end| (pos, end))
                })
                .collect()
        };

        // Find references to not-yet-declared hoistable bindings within this statement
        struct HoistInfo {
            binding_id: BindingId,
            name: String,
            kind: AstBindingKind,
            declaration_type: String,
            first_ref_pos: u32,
            first_ref_nid: u32,
        }
        let mut will_hoist: Vec<HoistInfo> = Vec::new();

        for (binding_id, name, kind, decl_type, _decl_start, decl_node_id) in &hoistable {
            if declared.contains(binding_id) {
                continue;
            }

            // Find the first reference (not declaration) to this binding in the statement's range.
            // Exclude JSX identifier references: while Babel's scope system links JSX
            // tag names to local bindings (and the context capture pass includes them),
            // the TS hoisting analysis does NOT traverse JSX elements. This mismatch
            // is intentional — it matches the TS behavior where <colgroup> adds
            // "colgroup" to the context but does NOT trigger hoisting, causing
            // EnterSSA to error with "Expected identifier to be defined before use".
            //
            // The decl_start filter excludes the binding's own declaration position from
            // counting as a reference. For hoisted bindings (function declarations), this
            // filter is only applied when the current statement IS a FunctionDeclaration,
            // since that's the only statement type where decl_start is a declaration, not
            // a reference.
            let apply_decl_filter = !matches!(kind, AstBindingKind::Hoisted) || is_function_decl;
            let refs_in_stmt: Vec<(u32, u32)> = builder
                .scope_info()
                .ref_node_id_to_binding
                .iter()
                .filter_map(|(&ref_nid, &ref_bid)| {
                    if ref_bid != *binding_id {
                        return None;
                    }
                    let entry = builder.identifier_locs().get(&ref_nid)?;
                    let ref_start = entry.start;
                    if ref_start < stmt_start || ref_start >= stmt_end {
                        return None;
                    }
                    if apply_decl_filter && *decl_node_id == Some(ref_nid) {
                        return None;
                    }
                    if entry.is_jsx {
                        return None;
                    }
                    Some((ref_start, ref_nid))
                })
                .collect();

            if refs_in_stmt.is_empty() {
                continue;
            }

            let (first_ref_pos, first_ref_nid) =
                *refs_in_stmt.iter().min_by_key(|(pos, _)| *pos).unwrap();

            // Hoist if: (1) binding is "hoisted" kind (function declaration), or
            // (2) any reference to this binding is inside a nested function scope.
            // Check per-reference rather than per-statement to correctly handle
            // statements that contain both nested functions and top-level code.
            let is_hoisted_kind = matches!(kind, AstBindingKind::Hoisted);
            let refs_in_nested_fn: Vec<(u32, u32)> = refs_in_stmt
                .iter()
                .copied()
                .filter(|&(ref_pos, _)| {
                    nested_function_ranges
                        .iter()
                        .any(|&(fn_start, fn_end)| ref_pos >= fn_start && ref_pos < fn_end)
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
                let binding_data = &builder.scope_info().bindings[binding_id.0 as usize];
                if binding_data.scope != scope_id
                    && !is_binding_in_block_direct_statements(binding_data, statements)
                {
                    continue;
                }
                // For hoisted bindings (function declarations), use the first reference
                // overall. For non-hoisted bindings, use the first reference inside a
                // nested function.
                let (hoist_ref_pos, hoist_ref_nid) = if is_hoisted_kind {
                    (first_ref_pos, first_ref_nid)
                } else {
                    *refs_in_nested_fn.iter().min_by_key(|(pos, _)| *pos).unwrap()
                };
                will_hoist.push(HoistInfo {
                    binding_id: *binding_id,
                    name: name.clone(),
                    kind: kind.clone(),
                    declaration_type: decl_type.clone(),
                    first_ref_pos: hoist_ref_pos,
                    first_ref_nid: hoist_ref_nid,
                });
            }
        }

        // Sort by first reference position to match TS traversal order
        will_hoist.sort_by_key(|h| h.first_ref_pos);

        // Emit DeclareContext for hoisted bindings
        for info in &will_hoist {
            if builder.environment().is_hoisted_identifier(info.binding_id.0) {
                continue;
            }

            let hoist_kind = match info.kind {
                AstBindingKind::Const | AstBindingKind::Var => InstructionKind::HoistedConst,
                AstBindingKind::Let => InstructionKind::HoistedLet,
                AstBindingKind::Hoisted => InstructionKind::HoistedFunction,
                _ => {
                    if info.declaration_type == "FunctionDeclaration" {
                        InstructionKind::HoistedFunction
                    } else if info.declaration_type == "VariableDeclarator" {
                        // Unsupported hoisting for this declaration kind
                        builder.record_error(CompilerErrorDetail {
                            category: ErrorCategory::Todo,
                            reason: "Handle non-const declarations for hoisting".to_string(),
                            description: Some(format!(
                                "variable \"{}\" declared with {:?}",
                                info.name, info.kind
                            )),
                            loc: None,
                            suggestions: None,
                        })?;
                        continue;
                    } else {
                        builder.record_error(CompilerErrorDetail {
                            category: ErrorCategory::Todo,
                            reason: "Unsupported declaration type for hoisting".to_string(),
                            description: Some(format!(
                                "variable \"{}\" declared with {}",
                                info.name, info.declaration_type
                            )),
                            loc: None,
                            suggestions: None,
                        })?;
                        continue;
                    }
                }
            };

            // Look up the reference location for the DeclareContext instruction.
            let ref_loc = builder.identifier_locs().get(&info.first_ref_nid).map(|e| e.loc.clone());
            let identifier = builder.resolve_binding(&info.name, info.binding_id)?;
            let place = Place {
                effect: Effect::Unknown,
                identifier,
                reactive: false,
                loc: ref_loc.clone(),
            };
            lower_value_to_temporary(
                builder,
                InstructionValue::DeclareContext {
                    lvalue: LValue { kind: hoist_kind, place },
                    loc: ref_loc,
                },
            )?;
            builder.environment_mut().add_hoisted_identifier(info.binding_id.0);
            // Hoisted identifiers also become context identifiers (matching TS addHoistedIdentifier)
            builder.add_context_identifier(info.binding_id);
        }

        // After processing the statement, mark any bindings it declares as "seen".
        // This must cover all statement types that can introduce bindings.
        match body_stmt {
            oxc::Statement::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    if let Some(&binding_id) = builder.scope_info().scopes[scope_id.0 as usize]
                        .bindings
                        .get(id.name.as_str())
                    {
                        declared.insert(binding_id);
                    }
                }
            }
            oxc::Statement::VariableDeclaration(var_decl) => {
                for decl in &var_decl.declarations {
                    collect_binding_names_from_pattern(
                        &decl.id,
                        scope_id,
                        builder.scope_info(),
                        &mut declared,
                    );
                }
            }
            oxc::Statement::ClassDeclaration(cls) => {
                if let Some(id) = &cls.id {
                    if let Some(&binding_id) = builder.scope_info().scopes[scope_id.0 as usize]
                        .bindings
                        .get(id.name.as_str())
                    {
                        declared.insert(binding_id);
                    }
                }
            }
            _ => {
                // For other statement types (e.g. ForStatement with VariableDeclaration in init),
                // we rely on the reference_to_binding check for forward references.
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

enum FunctionBody<'a> {
    Block(&'a oxc::FunctionBody<'a>),
    Expression(&'a oxc::Expression<'a>),
}

/// Main entry point: lower a function AST node into HIR.
///
/// Receives a `FunctionNode` (discovered by the entrypoint) and lowers it to HIR.
/// The `id` parameter provides the function name (which may come from the variable
/// declarator rather than the function node itself, e.g. `const Foo = () => {}`).
pub fn lower<'a>(
    func: &'a FunctionNode<'a>,
    _id: Option<&str>,
    scope_info: &ScopeInfo,
    env: &mut Environment<'a>,
    line_offsets: &LineOffsets,
) -> Result<HirFunction<'a>, CompilerError> {
    // Extract params, body, generator, is_async, loc, scope_id, and the AST function's own id
    // Note: `id` param may include inferred names (e.g., from `const Foo = () => {}`),
    // but the HIR function's `id` field should only include the function's own AST id
    // (FunctionDeclaration.id or FunctionExpression.id, NOT arrow functions).
    let (params, body, generator, is_async, loc, start, end, ast_id) = match func {
        FunctionNode::Function(f) => {
            let body_ref = f.body.as_deref().expect("component function has a body");
            (
                f.params.as_ref(),
                FunctionBody::Block(body_ref),
                f.generator,
                f.r#async,
                Some(line_offsets.source_location(f.span)),
                f.span.start,
                f.span.end,
                f.id.as_ref().map(|id| id.name.as_str()),
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
            (
                arrow.params.as_ref(),
                body,
                false,
                arrow.r#async,
                Some(line_offsets.source_location(arrow.span)),
                arrow.span.start,
                arrow.span.end,
                None,
            )
        }
    };

    let scope_id =
        scope_info.resolve_scope_for_node(func.node_id()).unwrap_or(scope_info.program_scope);

    validate_ts_this_parameters_in_function_range(scope_info, start, end)?;

    // Build identifier location index from the AST (replaces serialized referenceLocs/jsxReferencePositions)
    let identifier_locs = build_identifier_loc_index(func, scope_info, line_offsets);

    // Pre-compute context identifiers: variables captured across function boundaries
    let context_identifiers =
        find_context_identifiers(func, scope_info, env, &identifier_locs, line_offsets)?;

    // For top-level functions, context is empty (no captured refs)
    let context_map: FxIndexMap<crate::scope::BindingId, Option<SourceLocation>> =
        FxIndexMap::default();

    let (hir_func, _used_names, _child_bindings) = lower_inner(
        params,
        body,
        ast_id,
        generator,
        is_async,
        loc,
        scope_info,
        env,
        None, // no pre-existing bindings for top-level
        None, // no pre-existing used_names for top-level
        context_map,
        scope_id,
        scope_id, // component_scope = function_scope for top-level
        &context_identifiers,
        true, // is_top_level
        &identifier_locs,
        line_offsets,
    )?;

    Ok(hir_func)
}

// =============================================================================
// Stubs for future milestones
// =============================================================================

/// Result of resolving an identifier for assignment.
fn lower_inner<'a>(
    params: &'a oxc::FormalParameters<'a>,
    body: FunctionBody<'a>,
    id: Option<&str>,
    generator: bool,
    is_async: bool,
    loc: Option<SourceLocation>,
    scope_info: &ScopeInfo,
    env: &mut Environment<'a>,
    parent_bindings: Option<FxIndexMap<crate::scope::BindingId, IdentifierId>>,
    parent_used_names: Option<FxIndexMap<String, crate::scope::BindingId>>,
    context_map: FxIndexMap<crate::scope::BindingId, Option<SourceLocation>>,
    function_scope: crate::scope::ScopeId,
    component_scope: crate::scope::ScopeId,
    context_identifiers: &FxHashSet<crate::scope::BindingId>,
    is_top_level: bool,
    identifier_locs: &IdentifierLocIndex,
    line_offsets: &LineOffsets,
) -> Result<
    (
        HirFunction<'a>,
        FxIndexMap<String, crate::scope::BindingId>,
        FxIndexMap<crate::scope::BindingId, IdentifierId>,
    ),
    CompilerError,
> {
    validate_ts_this_parameter(scope_info, function_scope)?;

    let mut builder = HirBuilder::new(
        env,
        scope_info,
        function_scope,
        component_scope,
        context_identifiers.clone(),
        parent_bindings,
        Some(context_map.clone()),
        None,
        parent_used_names,
        identifier_locs,
        line_offsets,
    );

    // Build context places from the captured refs
    let mut context: Vec<Place> = Vec::new();
    for (&binding_id, ctx_loc) in &context_map {
        let binding = &scope_info.bindings[binding_id.0 as usize];
        let identifier = builder.resolve_binding(&binding.name, binding_id)?;
        context.push(Place {
            identifier,
            effect: Effect::Unknown,
            reactive: false,
            loc: ctx_loc.clone(),
        });
    }

    // Process parameters.
    let mut hir_params: Vec<ParamPattern> = Vec::new();
    for param in &params.items {
        if param.initializer.is_none()
            && let oxc::BindingPattern::BindingIdentifier(ident) = &param.pattern
        {
            if is_always_reserved_word(ident.name.as_str()) {
                return Err(CompilerError::from(reserved_identifier_diagnostic(
                    ident.name.as_str(),
                )));
            }
            let start = ident.span.start;
            let param_loc = builder.source_location(ident.span);
            let mut binding = builder.resolve_identifier(
                ident.name.as_str(),
                start,
                param_loc.clone(),
                Some(ident.span.start),
            )?;
            if !matches!(binding, VariableBinding::Identifier { .. }) {
                if let Some((binding_id, binding_data)) = builder
                    .scope_info()
                    .find_binding_id_in_descendants(ident.name.as_str(), builder.function_scope())
                {
                    let binding_kind =
                        crate::react_compiler_lowering::convert_binding_kind(&binding_data.kind);
                    let identifier = builder.resolve_binding_with_loc(
                        ident.name.as_str(),
                        binding_id,
                        param_loc.clone(),
                    )?;
                    binding = VariableBinding::Identifier { identifier, binding_kind };
                }
            }
            match binding {
                VariableBinding::Identifier { identifier, .. } => {
                    builder.set_identifier_declaration_loc(identifier, &param_loc);
                    let place = Place {
                        identifier,
                        effect: Effect::Unknown,
                        reactive: false,
                        loc: param_loc,
                    };
                    hir_params.push(ParamPattern::Place(place));
                }
                _ => {
                    builder.record_diagnostic(
                        CompilerDiagnostic::new(
                            ErrorCategory::Invariant,
                            "Could not find binding",
                            Some(format!(
                                "[BuildHIR] Could not find binding for param `{}`",
                                ident.name.as_str()
                            )),
                        )
                        .with_detail(CompilerDiagnosticDetail::Error {
                            loc: builder.source_location(ident.span),
                            message: Some("Could not find binding".to_string()),
                            identifier_name: None,
                        }),
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
        let param_loc = builder.source_location(param.span);
        let place = build_temporary_place(&mut builder, param_loc.clone());
        promote_temporary(&mut builder, place.identifier);
        hir_params.push(ParamPattern::Place(place.clone()));
        let value = if let Some(initializer) = &param.initializer {
            lower_default_to_temp(&mut builder, param_loc.clone(), initializer, place)?
        } else {
            place
        };
        lower_binding_assignment(
            &mut builder,
            param_loc,
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
        let rest_loc = builder.source_location(rest.span);
        let place = build_temporary_place(&mut builder, rest_loc.clone());
        hir_params.push(ParamPattern::Spread(SpreadPattern { place: place.clone() }));
        lower_binding_assignment(
            &mut builder,
            rest_loc,
            InstructionKind::Let,
            &rest.rest.argument,
            place,
            AssignmentStyle::Assignment,
        )?;
    }

    // Lower the body
    let mut directives: Vec<String> = Vec::new();
    match body {
        FunctionBody::Expression(expr) => {
            let fallthrough = builder.reserve(BlockKind::Block);
            let value = lower_expression_to_temporary(&mut builder, expr)?;
            builder.terminate_with_continuation(
                Terminal::Return {
                    value,
                    return_variant: ReturnVariant::Implicit,
                    id: EvaluationOrder(0),
                    loc: None,
                    effects: None,
                },
                fallthrough,
            );
        }
        FunctionBody::Block(block) => {
            directives = block.directives.iter().map(|d| d.expression.value.to_string()).collect();
            // A function body shares the function's scope (node_to_scope maps the
            // function node, not the block), so pass it as the scope override.
            lower_block_statement_with_scope(
                &mut builder,
                &block.statements,
                block.span.start,
                function_scope,
            )?;
        }
    }

    // Emit final Return(Void, undefined)
    let undefined_value =
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc: None };
    let return_value = lower_value_to_temporary(&mut builder, undefined_value)?;
    builder.terminate(
        Terminal::Return {
            value: return_value,
            return_variant: ReturnVariant::Void,
            id: EvaluationOrder(0),
            loc: None,
            effects: None,
        },
        None,
    );

    // Build the HIR
    let (hir_body, instructions, used_names, child_bindings) = builder.build()?;

    // Create the returns place
    let returns =
        crate::react_compiler_lowering::hir_builder::create_temporary_place(env, loc.clone());

    Ok((
        HirFunction {
            loc,
            id: id.map(|s| s.to_string()),
            name_hint: None,
            fn_type: if is_top_level { env.fn_type } else { ReactFunctionType::Other },
            params: hir_params,
            return_type_annotation: None,
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
fn lower_identifier(
    builder: &mut HirBuilder<'_, '_>,
    name: &str,
    start: u32,
    loc: Option<SourceLocation>,
    node_id: Option<u32>,
) -> Result<Place, CompilerError> {
    let binding = builder.resolve_identifier(name, start, loc.clone(), node_id)?;
    match binding {
        VariableBinding::Identifier { identifier, .. } => {
            Ok(Place { identifier, effect: Effect::Unknown, reactive: false, loc })
        }
        _ => {
            if let VariableBinding::Global { ref name } = binding {
                if name == "eval" {
                    builder.record_error(CompilerErrorDetail {
                        category: ErrorCategory::UnsupportedSyntax,
                        reason: "The 'eval' function is not supported".to_string(),
                        description: Some(
                            "Eval is an anti-pattern in JavaScript, and the code executed cannot be evaluated by React Compiler".to_string(),
                        ),
                        loc: loc.clone(),
                        suggestions: None,
                    })?;
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
                InstructionValue::LoadGlobal { binding: non_local_binding, loc: loc.clone() };
            lower_value_to_temporary(builder, instr_value)
        }
    }
}

fn convert_binary_operator(op: oxc::BinaryOperator) -> BinaryOperator {
    use oxc::BinaryOperator as O;
    match op {
        O::Addition => BinaryOperator::Add,
        O::Subtraction => BinaryOperator::Subtract,
        O::Multiplication => BinaryOperator::Multiply,
        O::Division => BinaryOperator::Divide,
        O::Remainder => BinaryOperator::Modulo,
        O::Exponential => BinaryOperator::Exponent,
        O::Equality => BinaryOperator::Equal,
        O::StrictEquality => BinaryOperator::StrictEqual,
        O::Inequality => BinaryOperator::NotEqual,
        O::StrictInequality => BinaryOperator::StrictNotEqual,
        O::LessThan => BinaryOperator::LessThan,
        O::LessEqualThan => BinaryOperator::LessEqual,
        O::GreaterThan => BinaryOperator::GreaterThan,
        O::GreaterEqualThan => BinaryOperator::GreaterEqual,
        O::ShiftLeft => BinaryOperator::ShiftLeft,
        O::ShiftRight => BinaryOperator::ShiftRight,
        O::ShiftRightZeroFill => BinaryOperator::UnsignedShiftRight,
        O::BitwiseOR => BinaryOperator::BitwiseOr,
        O::BitwiseXOR => BinaryOperator::BitwiseXor,
        O::BitwiseAnd => BinaryOperator::BitwiseAnd,
        O::In => BinaryOperator::In,
        O::Instanceof => BinaryOperator::InstanceOf,
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

enum MemberProperty {
    Literal(PropertyLiteral),
    Computed(Place),
}

struct LoweredMemberExpression<'a> {
    object: Place,
    property: MemberProperty,
    value: InstructionValue<'a>,
}

/// Lower a member access (oxc's Static / Computed / PrivateField variants) into a
/// receiver place + property + load value.
fn lower_member_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    member: &'a oxc::MemberExpression<'a>,
) -> Result<LoweredMemberExpression<'a>, CompilerError> {
    lower_member_expression_impl(builder, member, None)
}

fn lower_member_expression_impl<'a>(
    builder: &mut HirBuilder<'a, '_>,
    member: &'a oxc::MemberExpression<'a>,
    lowered_object: Option<Place>,
) -> Result<LoweredMemberExpression<'a>, CompilerError> {
    match member {
        oxc::MemberExpression::StaticMemberExpression(m) => {
            let loc = builder.source_location(m.span);
            let object = match lowered_object {
                Some(obj) => obj,
                None => lower_expression_to_temporary(builder, &m.object)?,
            };
            let prop_literal = PropertyLiteral::String(m.property.name.to_string());
            let value = InstructionValue::PropertyLoad {
                object: object.clone(),
                property: prop_literal.clone(),
                loc,
            };
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Literal(prop_literal),
                value,
            })
        }
        oxc::MemberExpression::ComputedMemberExpression(m) => {
            let loc = builder.source_location(m.span);
            let object = match lowered_object {
                Some(obj) => obj,
                None => lower_expression_to_temporary(builder, &m.object)?,
            };
            // A numeric computed index is treated as a PropertyLoad (matches TS).
            if let oxc::Expression::NumericLiteral(lit) = &m.expression {
                let prop_literal = PropertyLiteral::Number(FloatValue::new(lit.value));
                let value = InstructionValue::PropertyLoad {
                    object: object.clone(),
                    property: prop_literal.clone(),
                    loc,
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
                loc,
            };
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Computed(property),
                value,
            })
        }
        oxc::MemberExpression::PrivateFieldExpression(m) => {
            let loc = builder.source_location(m.span);
            let object = match lowered_object {
                Some(obj) => obj,
                None => lower_expression_to_temporary(builder, &m.object)?,
            };
            // TODO(stage1a-arms): private field access needs a private-name property
            // load + OriginalNode bail; defer to a later batch.
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "(BuildHIR::lowerMemberExpression) Handle private field property"
                    .to_string(),
                description: None,
                loc: loc.clone(),
                suggestions: None,
            })?;
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Literal(PropertyLiteral::String(String::new())),
                value: InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc },
            })
        }
    }
}

/// Build a HIR `TemplateQuasi` from an oxc `TemplateElement`.
fn template_quasi_from_oxc(q: &oxc::TemplateElement) -> TemplateQuasi {
    TemplateQuasi { raw: q.value.raw.to_string(), cooked: q.value.cooked.map(|c| c.to_string()) }
}

/// Lower the `import` keyword callee of an `ImportExpression`. The original Babel
/// path treats this as the `Import` node, which bails (records an error) and
/// returns an undefined primitive that is then loaded to a temporary.
fn lower_import_keyword_to_temporary(
    builder: &mut HirBuilder<'_, '_>,
    loc: &Option<SourceLocation>,
) -> Result<Place, CompilerError> {
    builder.record_error(CompilerErrorDetail {
        category: ErrorCategory::Todo,
        reason: "(BuildHIR::lowerExpression) Handle Import expressions".to_string(),
        description: None,
        loc: loc.clone(),
        suggestions: None,
    })?;
    lower_value_to_temporary(
        builder,
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc: loc.clone() },
    )
}

/// Lower a `PrivateName` operand (e.g. the left side of `#f in obj`). The original
/// Babel path bails (records an error) and returns an undefined primitive that is
/// then loaded to a temporary.
fn lower_private_name_to_temporary(
    builder: &mut HirBuilder<'_, '_>,
    span: oxc_span::Span,
) -> Result<Place, CompilerError> {
    let loc = builder.source_location(span);
    builder.record_error(CompilerErrorDetail {
        category: ErrorCategory::Todo,
        reason: "(BuildHIR::lowerExpression) Handle PrivateName expressions".to_string(),
        description: None,
        loc: loc.clone(),
        suggestions: None,
    })?;
    lower_value_to_temporary(
        builder,
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc },
    )
}

/// Babel/ESTree node-type tag for an oxc TS type, used as a
/// `TypeCastExpression`'s `type_annotation_name` (mirrors `get_type_annotation_name`,
/// which reads the unwrapped type's tag).
fn ts_type_node_type(ty: &oxc::TSType) -> &'static str {
    match ty {
        oxc::TSType::TSAnyKeyword(_) => "TSAnyKeyword",
        oxc::TSType::TSBigIntKeyword(_) => "TSBigIntKeyword",
        oxc::TSType::TSBooleanKeyword(_) => "TSBooleanKeyword",
        oxc::TSType::TSIntrinsicKeyword(_) => "TSIntrinsicKeyword",
        oxc::TSType::TSNeverKeyword(_) => "TSNeverKeyword",
        oxc::TSType::TSNullKeyword(_) => "TSNullKeyword",
        oxc::TSType::TSNumberKeyword(_) => "TSNumberKeyword",
        oxc::TSType::TSObjectKeyword(_) => "TSObjectKeyword",
        oxc::TSType::TSStringKeyword(_) => "TSStringKeyword",
        oxc::TSType::TSSymbolKeyword(_) => "TSSymbolKeyword",
        oxc::TSType::TSThisType(_) => "TSThisType",
        oxc::TSType::TSUndefinedKeyword(_) => "TSUndefinedKeyword",
        oxc::TSType::TSUnknownKeyword(_) => "TSUnknownKeyword",
        oxc::TSType::TSVoidKeyword(_) => "TSVoidKeyword",
        oxc::TSType::TSArrayType(_) => "TSArrayType",
        oxc::TSType::TSUnionType(_) => "TSUnionType",
        oxc::TSType::TSParenthesizedType(_) => "TSParenthesizedType",
        oxc::TSType::TSLiteralType(_) => "TSLiteralType",
        oxc::TSType::TSTypeReference(_) => "TSTypeReference",
        oxc::TSType::TSTypeOperatorType(_) => "TSTypeOperator",
        oxc::TSType::TSTupleType(_) => "TSTupleType",
        oxc::TSType::TSIntersectionType(_) => "TSIntersectionType",
        oxc::TSType::TSTypeLiteral(_) => "TSTypeLiteral",
        oxc::TSType::TSTypeQuery(_) => "TSTypeQuery",
        oxc::TSType::TSFunctionType(_) => "TSFunctionType",
        oxc::TSType::TSConstructorType(_) => "TSConstructorType",
        oxc::TSType::TSConditionalType(_) => "TSConditionalType",
        oxc::TSType::TSIndexedAccessType(_) => "TSIndexedAccessType",
        oxc::TSType::TSInferType(_) => "TSInferType",
        oxc::TSType::TSImportType(_) => "TSImportType",
        oxc::TSType::TSMappedType(_) => "TSMappedType",
        oxc::TSType::TSNamedTupleMember(_) => "TSNamedTupleMember",
        oxc::TSType::TSTemplateLiteralType(_) => "TSTemplateLiteralType",
        oxc::TSType::TSTypePredicate(_) => "TSTypePredicate",
        oxc::TSType::JSDocNullableType(_) => "JSDocNullableType",
        oxc::TSType::JSDocNonNullableType(_) => "JSDocNonNullableType",
        oxc::TSType::JSDocUnknownType(_) => "JSDocUnknownType",
    }
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
fn lower_ts_type(builder: &mut HirBuilder<'_, '_>, ty: &oxc::TSType) -> Type {
    use crate::react_compiler_hir::RawTypeCategory;
    match classify_ts_type(ty) {
        RawTypeCategory::Array => Type::Object { shape_id: Some("BuiltInArray".to_string()) },
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
    expression: &'a oxc::Expression<'a>,
    type_annotation: &'a oxc::TSType<'a>,
    type_annotation_kind: &str,
) -> Result<InstructionValue<'a>, CompilerError> {
    let loc = builder.source_location(span);
    let value = lower_expression_to_temporary(builder, expression)?;
    let type_ = lower_ts_type(builder, type_annotation);
    let type_annotation_name = Some(ts_type_node_type(type_annotation).to_string());
    Ok(InstructionValue::TypeCastExpression {
        value,
        type_,
        type_annotation_name,
        type_annotation_kind: Some(type_annotation_kind.to_string()),
        type_annotation: Some(type_annotation),
        loc,
    })
}

/// Lower a member-expression update target (oxc's member variants of
/// `SimpleAssignmentTarget`) into a receiver place + property + load value,
/// mirroring `lower_member_expression_impl`.
fn lower_member_expression_from_simple_target<'a>(
    builder: &mut HirBuilder<'a, '_>,
    target: &'a oxc::SimpleAssignmentTarget<'a>,
) -> Result<LoweredMemberExpression<'a>, CompilerError> {
    match target {
        oxc::SimpleAssignmentTarget::StaticMemberExpression(m) => {
            let loc = builder.source_location(m.span);
            let object = lower_expression_to_temporary(builder, &m.object)?;
            let prop_literal = PropertyLiteral::String(m.property.name.to_string());
            let value = InstructionValue::PropertyLoad {
                object: object.clone(),
                property: prop_literal.clone(),
                loc,
            };
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Literal(prop_literal),
                value,
            })
        }
        oxc::SimpleAssignmentTarget::ComputedMemberExpression(m) => {
            let loc = builder.source_location(m.span);
            let object = lower_expression_to_temporary(builder, &m.object)?;
            if let oxc::Expression::NumericLiteral(lit) = &m.expression {
                let prop_literal = PropertyLiteral::Number(FloatValue::new(lit.value));
                let value = InstructionValue::PropertyLoad {
                    object: object.clone(),
                    property: prop_literal.clone(),
                    loc,
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
                loc,
            };
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Computed(property),
                value,
            })
        }
        oxc::SimpleAssignmentTarget::PrivateFieldExpression(m) => {
            let loc = builder.source_location(m.span);
            let object = lower_expression_to_temporary(builder, &m.object)?;
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "(BuildHIR::lowerMemberExpression) Handle private field property"
                    .to_string(),
                description: None,
                loc: loc.clone(),
                suggestions: None,
            })?;
            Ok(LoweredMemberExpression {
                object,
                property: MemberProperty::Literal(PropertyLiteral::String(String::new())),
                value: InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc },
            })
        }
        _ => {
            unreachable!("lower_member_expression_from_simple_target called on a non-member target")
        }
    }
}

fn lower_arguments<'a>(
    builder: &mut HirBuilder<'a, '_>,
    args: &'a [oxc::Argument<'a>],
) -> Result<Vec<PlaceOrSpread>, CompilerError> {
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
enum IdentifierForAssignment {
    Place(Place),
    Global { name: String },
}

/// Resolve an identifier as an assignment target. AST-agnostic. Returns None if
/// the binding could not be found (error recorded).
fn lower_identifier_for_assignment(
    builder: &mut HirBuilder<'_, '_>,
    loc: Option<SourceLocation>,
    ident_loc: Option<SourceLocation>,
    kind: InstructionKind,
    name: &str,
    start: u32,
    node_id: Option<u32>,
) -> Result<Option<IdentifierForAssignment>, CompilerError> {
    let mut binding = builder.resolve_identifier(name, start, ident_loc.clone(), node_id)?;
    if !matches!(binding, VariableBinding::Identifier { .. }) && kind != InstructionKind::Reassign {
        if let Some((binding_id, binding_data)) =
            builder.scope_info().find_binding_id_in_descendants(name, builder.function_scope())
        {
            let bk = crate::react_compiler_lowering::convert_binding_kind(&binding_data.kind);
            let identifier =
                builder.resolve_binding_with_loc(name, binding_id, ident_loc.clone())?;
            binding = VariableBinding::Identifier { identifier, binding_kind: bk };
        }
    }
    match binding {
        VariableBinding::Identifier { identifier, binding_kind, .. } => {
            if kind != InstructionKind::Reassign {
                builder.set_identifier_declaration_loc(identifier, &ident_loc);
            }
            if binding_kind == BindingKind::Const && kind == InstructionKind::Reassign {
                builder.record_error(CompilerErrorDetail {
                    reason: "Cannot reassign a `const` variable".to_string(),
                    category: ErrorCategory::Syntax,
                    loc: loc.clone(),
                    description: Some(format!("`{}` is declared as const", name)),
                    suggestions: None,
                })?;
                return Ok(None);
            }
            Ok(Some(IdentifierForAssignment::Place(Place {
                identifier,
                effect: Effect::Unknown,
                reactive: false,
                loc,
            })))
        }
        VariableBinding::Global { name: gname } => {
            if kind == InstructionKind::Reassign {
                Ok(Some(IdentifierForAssignment::Global { name: gname }))
            } else {
                builder.record_error(CompilerErrorDetail {
                    reason: "Could not find binding for declaration".to_string(),
                    category: ErrorCategory::Invariant,
                    loc,
                    description: None,
                    suggestions: None,
                })?;
                Ok(None)
            }
        }
        _ => {
            if kind == InstructionKind::Reassign {
                Ok(Some(IdentifierForAssignment::Global { name: name.to_string() }))
            } else {
                builder.record_error(CompilerErrorDetail {
                    reason: "Could not find binding for declaration".to_string(),
                    category: ErrorCategory::Invariant,
                    loc,
                    description: None,
                    suggestions: None,
                })?;
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
    loc: Option<SourceLocation>,
    kind: InstructionKind,
    target: &'a oxc::BindingPattern<'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, CompilerError> {
    match target {
        oxc::BindingPattern::BindingIdentifier(id) => {
            let start = id.span.start;
            let id_loc = builder.source_location(id.span);
            let result = lower_identifier_for_assignment(
                builder,
                loc.clone(),
                id_loc,
                kind,
                id.name.as_str(),
                start,
                Some(start),
            )?;
            match result {
                None => Ok(None),
                Some(IdentifierForAssignment::Global { name }) => {
                    let temp = lower_value_to_temporary(
                        builder,
                        InstructionValue::StoreGlobal { name, value, loc },
                    )?;
                    Ok(Some(temp))
                }
                Some(IdentifierForAssignment::Place(place)) => {
                    if builder.is_context_identifier(id.name.as_str(), start, Some(start)) {
                        let is_hoisted = builder
                            .scope_info()
                            .resolve_reference_for_node(Some(start))
                            .map(|b| builder.environment().is_hoisted_identifier(b.id.0))
                            .unwrap_or(false);
                        if kind == InstructionKind::Const && !is_hoisted {
                            builder.record_error(CompilerErrorDetail {
                                reason: "Expected `const` declaration not to be reassigned"
                                    .to_string(),
                                category: ErrorCategory::Syntax,
                                loc: loc.clone(),
                                suggestions: None,
                                description: None,
                            })?;
                        }
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreContext {
                                lvalue: LValue { place, kind },
                                value,
                                loc,
                            },
                        )?;
                        Ok(Some(temp))
                    } else {
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreLocal {
                                lvalue: LValue { place, kind },
                                value,
                                type_annotation: None,
                                loc,
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
                        let start = id.span.start;
                        let is_context =
                            builder.is_context_identifier(id.name.as_str(), start, Some(start));
                        // force_temporaries is always false on the binding path.
                        let can_use_direct =
                            matches!(assignment_style, AssignmentStyle::Assignment) || !is_context;
                        if can_use_direct {
                            let id_loc = builder.source_location(id.span);
                            match lower_identifier_for_assignment(
                                builder,
                                id_loc.clone(),
                                id_loc,
                                kind,
                                id.name.as_str(),
                                start,
                                Some(start),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    items.push(ArrayPatternElement::Place(place));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    let temp = build_temporary_place(
                                        builder,
                                        builder.source_location(id.span),
                                    );
                                    promote_temporary(builder, temp.identifier);
                                    items.push(ArrayPatternElement::Place(temp.clone()));
                                    followups.push((temp, element.as_ref().unwrap()));
                                }
                                None => {
                                    items.push(ArrayPatternElement::Hole);
                                }
                            }
                        } else {
                            let temp =
                                build_temporary_place(builder, builder.source_location(id.span));
                            promote_temporary(builder, temp.identifier);
                            items.push(ArrayPatternElement::Place(temp.clone()));
                            followups.push((temp, element.as_ref().unwrap()));
                        }
                    }
                    Some(other) => {
                        let elem_loc = builder.source_location(other.span());
                        let temp = build_temporary_place(builder, elem_loc);
                        promote_temporary(builder, temp.identifier);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((temp, other));
                    }
                }
            }

            if let Some(rest) = &pattern.rest {
                match &rest.argument {
                    oxc::BindingPattern::BindingIdentifier(id) => {
                        let start = id.span.start;
                        let is_context =
                            builder.is_context_identifier(id.name.as_str(), start, Some(start));
                        let can_use_direct =
                            matches!(assignment_style, AssignmentStyle::Assignment) || !is_context;
                        if can_use_direct {
                            let rest_loc = builder.source_location(rest.span);
                            let id_loc = builder.source_location(id.span);
                            match lower_identifier_for_assignment(
                                builder,
                                rest_loc,
                                id_loc,
                                kind,
                                id.name.as_str(),
                                start,
                                Some(start),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    items
                                        .push(ArrayPatternElement::Spread(SpreadPattern { place }));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    let temp = build_temporary_place(
                                        builder,
                                        builder.source_location(rest.span),
                                    );
                                    promote_temporary(builder, temp.identifier);
                                    items.push(ArrayPatternElement::Spread(SpreadPattern {
                                        place: temp.clone(),
                                    }));
                                    followups.push((temp, &rest.argument));
                                }
                                None => {}
                            }
                        } else {
                            let temp =
                                build_temporary_place(builder, builder.source_location(rest.span));
                            promote_temporary(builder, temp.identifier);
                            items.push(ArrayPatternElement::Spread(SpreadPattern {
                                place: temp.clone(),
                            }));
                            followups.push((temp, &rest.argument));
                        }
                    }
                    _ => {
                        let temp =
                            build_temporary_place(builder, builder.source_location(rest.span));
                        promote_temporary(builder, temp.identifier);
                        items.push(ArrayPatternElement::Spread(SpreadPattern {
                            place: temp.clone(),
                        }));
                        followups.push((temp, &rest.argument));
                    }
                }
            }

            let pattern_loc = builder.source_location(pattern.span);
            let temporary = lower_value_to_temporary(
                builder,
                InstructionValue::Destructure {
                    lvalue: LValuePattern {
                        pattern: Pattern::Array(ArrayPattern { items, loc: pattern_loc }),
                        kind,
                    },
                    value: value.clone(),
                    loc: loc.clone(),
                },
            )?;

            for (place, path) in followups {
                let followup_loc = builder.source_location(path.span()).or(loc.clone());
                lower_binding_assignment(
                    builder,
                    followup_loc,
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
                    builder.record_error(CompilerErrorDetail {
                        reason: "(BuildHIR::lowerAssignment) Handle computed properties in ObjectPattern".to_string(),
                        category: ErrorCategory::Todo,
                        loc: builder.source_location(prop.span),
                        description: None,
                        suggestions: None,
                    })?;
                    continue;
                }

                let key = match lower_object_property_key(builder, &prop.key, false)? {
                    Some(k) => k,
                    None => continue,
                };

                match &prop.value {
                    oxc::BindingPattern::BindingIdentifier(id) => {
                        let start = id.span.start;
                        let is_context =
                            builder.is_context_identifier(id.name.as_str(), start, Some(start));
                        let can_use_direct =
                            matches!(assignment_style, AssignmentStyle::Assignment) || !is_context;
                        if can_use_direct {
                            let id_loc = builder.source_location(id.span);
                            match lower_identifier_for_assignment(
                                builder,
                                id_loc.clone(),
                                id_loc,
                                kind,
                                id.name.as_str(),
                                start,
                                Some(start),
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
                                    builder.record_error(CompilerErrorDetail {
                                        reason: "Expected reassignment of globals to enable forceTemporaries".to_string(),
                                        category: ErrorCategory::Todo,
                                        loc: builder.source_location(id.span),
                                        description: None,
                                        suggestions: None,
                                    })?;
                                }
                                None => {
                                    continue;
                                }
                            }
                        } else {
                            let temp =
                                build_temporary_place(builder, builder.source_location(id.span));
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
                        let elem_loc = builder.source_location(other.span());
                        let temp = build_temporary_place(builder, elem_loc);
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
                        let start = id.span.start;
                        let is_context =
                            builder.is_context_identifier(id.name.as_str(), start, Some(start));
                        let can_use_direct =
                            matches!(assignment_style, AssignmentStyle::Assignment) || !is_context;
                        if can_use_direct {
                            let rest_loc = builder.source_location(rest.span);
                            let id_loc = builder.source_location(id.span);
                            match lower_identifier_for_assignment(
                                builder,
                                rest_loc,
                                id_loc,
                                kind,
                                id.name.as_str(),
                                start,
                                Some(start),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    properties.push(ObjectPropertyOrSpread::Spread(
                                        SpreadPattern { place },
                                    ));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    builder.record_error(CompilerErrorDetail {
                                        reason: "Expected reassignment of globals to enable forceTemporaries".to_string(),
                                        category: ErrorCategory::Todo,
                                        loc: builder.source_location(rest.span),
                                        description: None,
                                        suggestions: None,
                                    })?;
                                }
                                None => {}
                            }
                        } else {
                            let temp =
                                build_temporary_place(builder, builder.source_location(rest.span));
                            promote_temporary(builder, temp.identifier);
                            properties.push(ObjectPropertyOrSpread::Spread(SpreadPattern {
                                place: temp.clone(),
                            }));
                            followups.push((temp, &rest.argument));
                        }
                    }
                    other => {
                        builder.record_error(CompilerErrorDetail {
                            reason: format!(
                                "(BuildHIR::lowerAssignment) Handle {} rest element in ObjectPattern",
                                match other {
                                    oxc::BindingPattern::ObjectPattern(_) => "ObjectPattern",
                                    oxc::BindingPattern::ArrayPattern(_) => "ArrayPattern",
                                    oxc::BindingPattern::AssignmentPattern(_) => "AssignmentPattern",
                                    _ => "unknown",
                                }
                            ),
                            category: ErrorCategory::Todo,
                            loc: builder.source_location(rest.span),
                            description: None,
                            suggestions: None,
                        })?;
                    }
                }
            }

            let pattern_loc = builder.source_location(pattern.span);
            let temporary = lower_value_to_temporary(
                builder,
                InstructionValue::Destructure {
                    lvalue: LValuePattern {
                        pattern: Pattern::Object(ObjectPattern { properties, loc: pattern_loc }),
                        kind,
                    },
                    value: value.clone(),
                    loc: loc.clone(),
                },
            )?;

            for (place, path) in followups {
                let followup_loc = builder.source_location(path.span()).or(loc.clone());
                lower_binding_assignment(
                    builder,
                    followup_loc,
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
            let pat_loc = builder.source_location(pattern.span);
            let temp = lower_default_to_temp(builder, pat_loc.clone(), &pattern.right, value)?;
            // Recursively assign the resolved value to the left pattern.
            lower_binding_assignment(builder, pat_loc, kind, &pattern.left, temp, assignment_style)
        }
    }
}

/// Lower the default-value ternary `value === undefined ? default : value` into a
/// fresh temporary and return it. Shared by the `AssignmentPattern` arm and by the
/// default-parameter (`FormalParameter.initializer`) lowering, which in Babel was a
/// single `AssignmentPattern` param node.
fn lower_default_to_temp<'a>(
    builder: &mut HirBuilder<'a, '_>,
    pat_loc: Option<SourceLocation>,
    default: &'a oxc::Expression<'a>,
    value: Place,
) -> Result<Place, CompilerError> {
    let temp = build_temporary_place(builder, pat_loc.clone());

    let test_block = builder.reserve(BlockKind::Value);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;

    let temp_consequent = temp.clone();
    let pat_loc_consequent = pat_loc.clone();
    let consequent = builder.try_enter(BlockKind::Value, |builder, _| {
        let default_value = lower_reorderable_expression(builder, default)?;
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { place: temp_consequent.clone(), kind: InstructionKind::Const },
                value: default_value,
                type_annotation: None,
                loc: pat_loc_consequent.clone(),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder(0),
            loc: pat_loc_consequent.clone(),
        })
    });

    let temp_alternate = temp.clone();
    let pat_loc_alternate = pat_loc.clone();
    let value_alternate = value.clone();
    let alternate = builder.try_enter(BlockKind::Value, |builder, _| {
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { place: temp_alternate.clone(), kind: InstructionKind::Const },
                value: value_alternate.clone(),
                type_annotation: None,
                loc: pat_loc_alternate.clone(),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder(0),
            loc: pat_loc_alternate.clone(),
        })
    });

    builder.terminate_with_continuation(
        Terminal::Ternary {
            test: test_block.id,
            fallthrough: continuation_id,
            id: EvaluationOrder(0),
            loc: pat_loc.clone(),
        },
        test_block,
    );

    let undef = lower_value_to_temporary(
        builder,
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc: pat_loc.clone() },
    )?;
    let test = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression {
            left: value,
            operator: BinaryOperator::StrictEqual,
            right: undef,
            loc: pat_loc.clone(),
        },
    )?;
    builder.terminate_with_continuation(
        Terminal::Branch {
            test,
            consequent: consequent?,
            alternate: alternate?,
            fallthrough: continuation_id,
            id: EvaluationOrder(0),
            loc: pat_loc,
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
    loc: Option<SourceLocation>,
    kind: InstructionKind,
    target: &'a oxc::SimpleAssignmentTarget<'a>,
    value: Place,
) -> Result<Option<Place>, CompilerError> {
    // MemberExpression may only appear in an assignment expression (Reassign).
    if kind != InstructionKind::Reassign {
        builder.record_error(CompilerErrorDetail {
            category: ErrorCategory::Invariant,
            reason: "MemberExpression may only appear in an assignment expression".to_string(),
            description: None,
            loc: loc.clone(),
            suggestions: None,
        })?;
        return Ok(None);
    }
    match target {
        oxc::SimpleAssignmentTarget::StaticMemberExpression(member) => {
            let object = lower_expression_to_temporary(builder, &member.object)?;
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::PropertyStore {
                    object,
                    property: PropertyLiteral::String(member.property.name.to_string()),
                    value,
                    loc,
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
                        loc,
                    },
                )?;
                return Ok(Some(temp));
            }
            let property_place = lower_expression_to_temporary(builder, &member.expression)?;
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::ComputedStore { object, property: property_place, value, loc },
            )?;
            Ok(Some(temp))
        }
        oxc::SimpleAssignmentTarget::PrivateFieldExpression(member) => {
            // Babel modeled `a.#b = v` as a non-computed MemberExpression with a
            // PrivateName property; the original `lower_assignment` member arm hit
            // the generic property `_` branch and bailed with this Todo.
            let object = lower_expression_to_temporary(builder, &member.object)?;
            let _ = object;
            builder.record_error(CompilerErrorDetail {
                reason:
                    "(BuildHIR::lowerAssignment) Handle PrivateName properties in MemberExpression"
                        .to_string(),
                category: ErrorCategory::Todo,
                loc: builder.source_location(member.field.span),
                description: None,
                suggestions: None,
            })?;
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc },
            )?;
            Ok(Some(temp))
        }
        _ => unreachable!("lower_member_assignment_target called on a non-member target"),
    }
}

/// True if `maybe` is a bare identifier assignment target that resolves to a local
/// binding (used to compute `force_temporaries`).
fn assignment_target_is_local_identifier(
    builder: &mut HirBuilder<'_, '_>,
    maybe: &oxc::AssignmentTargetMaybeDefault,
) -> Result<bool, CompilerError> {
    match maybe {
        oxc::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
            let start = id.span.start;
            if builder.is_context_identifier(id.name.as_str(), start, Some(start)) {
                return Ok(false);
            }
            let ident_loc = builder.source_location(id.span);
            match builder.resolve_identifier(id.name.as_str(), start, ident_loc, Some(start))? {
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
    loc: Option<SourceLocation>,
    kind: InstructionKind,
    target: &'a oxc::AssignmentTarget<'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, CompilerError> {
    match target {
        oxc::AssignmentTarget::AssignmentTargetIdentifier(id) => {
            let start = id.span.start;
            let id_loc = builder.source_location(id.span);
            let result = lower_identifier_for_assignment(
                builder,
                loc.clone(),
                id_loc,
                kind,
                id.name.as_str(),
                start,
                Some(start),
            )?;
            match result {
                None => Ok(None),
                Some(IdentifierForAssignment::Global { name }) => {
                    let temp = lower_value_to_temporary(
                        builder,
                        InstructionValue::StoreGlobal { name, value, loc },
                    )?;
                    Ok(Some(temp))
                }
                Some(IdentifierForAssignment::Place(place)) => {
                    if builder.is_context_identifier(id.name.as_str(), start, Some(start)) {
                        let is_hoisted = builder
                            .scope_info()
                            .resolve_reference_for_node(Some(start))
                            .map(|b| builder.environment().is_hoisted_identifier(b.id.0))
                            .unwrap_or(false);
                        if kind == InstructionKind::Const && !is_hoisted {
                            builder.record_error(CompilerErrorDetail {
                                reason: "Expected `const` declaration not to be reassigned"
                                    .to_string(),
                                category: ErrorCategory::Syntax,
                                loc: loc.clone(),
                                suggestions: None,
                                description: None,
                            })?;
                        }
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreContext {
                                lvalue: LValue { place, kind },
                                value,
                                loc,
                            },
                        )?;
                        Ok(Some(temp))
                    } else {
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreLocal {
                                lvalue: LValue { place, kind },
                                value,
                                type_annotation: None,
                                loc,
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
            lower_member_assignment_target(builder, loc, kind, simple, value)
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
                        let start = id.span.start;
                        let is_context =
                            builder.is_context_identifier(id.name.as_str(), start, Some(start));
                        let can_use_direct = !force_temporaries
                            && (matches!(assignment_style, AssignmentStyle::Assignment)
                                || !is_context);
                        if can_use_direct {
                            let id_loc = builder.source_location(id.span);
                            match lower_identifier_for_assignment(
                                builder,
                                id_loc.clone(),
                                id_loc,
                                kind,
                                id.name.as_str(),
                                start,
                                Some(start),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    items.push(ArrayPatternElement::Place(place));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    let temp = build_temporary_place(
                                        builder,
                                        builder.source_location(id.span),
                                    );
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
                            let temp =
                                build_temporary_place(builder, builder.source_location(id.span));
                            promote_temporary(builder, temp.identifier);
                            items.push(ArrayPatternElement::Place(temp.clone()));
                            followups.push((
                                temp,
                                FollowupTarget::MaybeDefault(element.as_ref().unwrap()),
                            ));
                        }
                    }
                    Some(other) => {
                        let elem_loc = builder.source_location(other.span());
                        let temp = build_temporary_place(builder, elem_loc);
                        promote_temporary(builder, temp.identifier);
                        items.push(ArrayPatternElement::Place(temp.clone()));
                        followups.push((temp, FollowupTarget::MaybeDefault(other)));
                    }
                }
            }

            if let Some(rest) = &pattern.rest {
                match &rest.target {
                    oxc::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                        let start = id.span.start;
                        let is_context =
                            builder.is_context_identifier(id.name.as_str(), start, Some(start));
                        let can_use_direct = !force_temporaries
                            && (matches!(assignment_style, AssignmentStyle::Assignment)
                                || !is_context);
                        if can_use_direct {
                            let rest_loc = builder.source_location(rest.span);
                            let id_loc = builder.source_location(id.span);
                            match lower_identifier_for_assignment(
                                builder,
                                rest_loc,
                                id_loc,
                                kind,
                                id.name.as_str(),
                                start,
                                Some(start),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    items
                                        .push(ArrayPatternElement::Spread(SpreadPattern { place }));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    let temp = build_temporary_place(
                                        builder,
                                        builder.source_location(rest.span),
                                    );
                                    promote_temporary(builder, temp.identifier);
                                    items.push(ArrayPatternElement::Spread(SpreadPattern {
                                        place: temp.clone(),
                                    }));
                                    followups.push((temp, FollowupTarget::Target(&rest.target)));
                                }
                                None => {}
                            }
                        } else {
                            let temp =
                                build_temporary_place(builder, builder.source_location(rest.span));
                            promote_temporary(builder, temp.identifier);
                            items.push(ArrayPatternElement::Spread(SpreadPattern {
                                place: temp.clone(),
                            }));
                            followups.push((temp, FollowupTarget::Target(&rest.target)));
                        }
                    }
                    _ => {
                        let temp =
                            build_temporary_place(builder, builder.source_location(rest.span));
                        promote_temporary(builder, temp.identifier);
                        items.push(ArrayPatternElement::Spread(SpreadPattern {
                            place: temp.clone(),
                        }));
                        followups.push((temp, FollowupTarget::Target(&rest.target)));
                    }
                }
            }

            let pattern_loc = builder.source_location(pattern.span);
            let temporary = lower_value_to_temporary(
                builder,
                InstructionValue::Destructure {
                    lvalue: LValuePattern {
                        pattern: Pattern::Array(ArrayPattern { items, loc: pattern_loc }),
                        kind,
                    },
                    value: value.clone(),
                    loc: loc.clone(),
                },
            )?;

            for (place, path) in followups {
                lower_followup_target(builder, loc.clone(), kind, path, place, assignment_style)?;
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
                                let start = p.binding.span.start;
                                let ident_loc = builder.source_location(p.binding.span);
                                match builder.resolve_identifier(
                                    p.binding.name.as_str(),
                                    start,
                                    ident_loc,
                                    Some(start),
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
                        let key =
                            ObjectPropertyKey::Identifier { name: p.binding.name.to_string() };
                        let id = &p.binding;
                        let start = id.span.start;
                        if let Some(default) = &p.init {
                            // `{foo = d}` — Babel shorthand AssignmentPattern. Lower
                            // via a promoted temporary + default followup.
                            let elem_loc = builder.source_location(p.span);
                            let temp = build_temporary_place(builder, elem_loc);
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
                            builder.is_context_identifier(id.name.as_str(), start, Some(start));
                        let can_use_direct = !force_temporaries
                            && (matches!(assignment_style, AssignmentStyle::Assignment)
                                || !is_context);
                        if can_use_direct {
                            let id_loc = builder.source_location(id.span);
                            match lower_identifier_for_assignment(
                                builder,
                                id_loc.clone(),
                                id_loc,
                                kind,
                                id.name.as_str(),
                                start,
                                Some(start),
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
                                    builder.record_error(CompilerErrorDetail {
                                        reason: "Expected reassignment of globals to enable forceTemporaries".to_string(),
                                        category: ErrorCategory::Todo,
                                        loc: builder.source_location(id.span),
                                        description: None,
                                        suggestions: None,
                                    })?;
                                }
                                None => {
                                    continue;
                                }
                            }
                        } else {
                            let temp =
                                build_temporary_place(builder, builder.source_location(id.span));
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
                            builder.record_error(CompilerErrorDetail {
                                reason: "(BuildHIR::lowerAssignment) Handle computed properties in ObjectPattern".to_string(),
                                category: ErrorCategory::Todo,
                                loc: builder.source_location(p.span),
                                description: None,
                                suggestions: None,
                            })?;
                            continue;
                        }
                        let key = match lower_object_property_key(builder, &p.name, false)? {
                            Some(k) => k,
                            None => continue,
                        };
                        match &p.binding {
                            oxc::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
                                let start = id.span.start;
                                let is_context = builder.is_context_identifier(
                                    id.name.as_str(),
                                    start,
                                    Some(start),
                                );
                                let can_use_direct = !force_temporaries
                                    && (matches!(assignment_style, AssignmentStyle::Assignment)
                                        || !is_context);
                                if can_use_direct {
                                    let id_loc = builder.source_location(id.span);
                                    match lower_identifier_for_assignment(
                                        builder,
                                        id_loc.clone(),
                                        id_loc,
                                        kind,
                                        id.name.as_str(),
                                        start,
                                        Some(start),
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
                                            builder.record_error(CompilerErrorDetail {
                                                reason: "Expected reassignment of globals to enable forceTemporaries".to_string(),
                                                category: ErrorCategory::Todo,
                                                loc: builder.source_location(id.span),
                                                description: None,
                                                suggestions: None,
                                            })?;
                                        }
                                        None => {
                                            continue;
                                        }
                                    }
                                } else {
                                    let temp = build_temporary_place(
                                        builder,
                                        builder.source_location(id.span),
                                    );
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
                                let elem_loc = builder.source_location(other.span());
                                let temp = build_temporary_place(builder, elem_loc);
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
                        let start = id.span.start;
                        let is_context =
                            builder.is_context_identifier(id.name.as_str(), start, Some(start));
                        let can_use_direct = !force_temporaries
                            && (matches!(assignment_style, AssignmentStyle::Assignment)
                                || !is_context);
                        if can_use_direct {
                            let rest_loc = builder.source_location(rest.span);
                            let id_loc = builder.source_location(id.span);
                            match lower_identifier_for_assignment(
                                builder,
                                rest_loc,
                                id_loc,
                                kind,
                                id.name.as_str(),
                                start,
                                Some(start),
                            )? {
                                Some(IdentifierForAssignment::Place(place)) => {
                                    properties.push(ObjectPropertyOrSpread::Spread(
                                        SpreadPattern { place },
                                    ));
                                }
                                Some(IdentifierForAssignment::Global { .. }) => {
                                    builder.record_error(CompilerErrorDetail {
                                        reason: "Expected reassignment of globals to enable forceTemporaries".to_string(),
                                        category: ErrorCategory::Todo,
                                        loc: builder.source_location(rest.span),
                                        description: None,
                                        suggestions: None,
                                    })?;
                                }
                                None => {}
                            }
                        } else {
                            let temp =
                                build_temporary_place(builder, builder.source_location(rest.span));
                            promote_temporary(builder, temp.identifier);
                            properties.push(ObjectPropertyOrSpread::Spread(SpreadPattern {
                                place: temp.clone(),
                            }));
                            followups.push((temp, FollowupTarget::Target(&rest.target)));
                        }
                    }
                    other => {
                        builder.record_error(CompilerErrorDetail {
                            reason: format!(
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
                            ),
                            category: ErrorCategory::Todo,
                            loc: builder.source_location(rest.span),
                            description: None,
                            suggestions: None,
                        })?;
                    }
                }
            }

            let pattern_loc = builder.source_location(pattern.span);
            let temporary = lower_value_to_temporary(
                builder,
                InstructionValue::Destructure {
                    lvalue: LValuePattern {
                        pattern: Pattern::Object(ObjectPattern { properties, loc: pattern_loc }),
                        kind,
                    },
                    value: value.clone(),
                    loc: loc.clone(),
                },
            )?;

            for (place, path) in followups {
                lower_followup_target(builder, loc.clone(), kind, path, place, assignment_style)?;
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
enum FollowupTarget<'a> {
    Target(&'a oxc::AssignmentTarget<'a>),
    MaybeDefault(&'a oxc::AssignmentTargetMaybeDefault<'a>),
    /// A bare `{foo}` shorthand object property binding that needs a promoted
    /// temporary followup (the Babel `obj_prop.value == Identifier` case).
    Identifier(&'a oxc::IdentifierReference<'a>),
    Default {
        span: oxc_span::Span,
        default: &'a oxc::Expression<'a>,
        binding: FollowupBinding<'a>,
    },
}

enum FollowupBinding<'a> {
    Identifier(&'a oxc::IdentifierReference<'a>),
    Target(&'a oxc::AssignmentTarget<'a>),
}

/// Store `value` into the identifier-target `id` (a bare destructuring binding).
/// Mirrors the `PatternLike::Identifier` followup path of the original
/// `lower_assignment` (re-resolving the binding for the store).
fn lower_identifier_followup_store(
    builder: &mut HirBuilder<'_, '_>,
    loc: Option<SourceLocation>,
    kind: InstructionKind,
    id: &oxc::IdentifierReference,
    value: Place,
) -> Result<Option<Place>, CompilerError> {
    let start = id.span.start;
    let id_loc = builder.source_location(id.span);
    let result = lower_identifier_for_assignment(
        builder,
        loc.clone(),
        id_loc,
        kind,
        id.name.as_str(),
        start,
        Some(start),
    )?;
    match result {
        None => Ok(None),
        Some(IdentifierForAssignment::Global { name }) => {
            let t = lower_value_to_temporary(
                builder,
                InstructionValue::StoreGlobal { name, value, loc },
            )?;
            Ok(Some(t))
        }
        Some(IdentifierForAssignment::Place(place)) => {
            if builder.is_context_identifier(id.name.as_str(), start, Some(start)) {
                let t = lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreContext { lvalue: LValue { place, kind }, value, loc },
                )?;
                Ok(Some(t))
            } else {
                let t = lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { place, kind },
                        value,
                        type_annotation: None,
                        loc,
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
    loc: Option<SourceLocation>,
    kind: InstructionKind,
    target: FollowupTarget<'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, CompilerError> {
    match target {
        FollowupTarget::Target(t) => {
            let followup_loc = builder.source_location(t.span()).or(loc);
            lower_assignment_target(builder, followup_loc, kind, t, value, assignment_style)
        }
        FollowupTarget::MaybeDefault(m) => {
            lower_assignment_target_maybe_default(builder, loc, kind, m, value, assignment_style)
        }
        FollowupTarget::Identifier(id) => {
            let followup_loc = builder.source_location(id.span).or(loc);
            lower_identifier_followup_store(builder, followup_loc, kind, id, value)
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
    loc: Option<SourceLocation>,
    kind: InstructionKind,
    maybe: &'a oxc::AssignmentTargetMaybeDefault<'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, CompilerError> {
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
            let followup_loc = builder.source_location(target.span()).or(loc);
            lower_assignment_target(builder, followup_loc, kind, target, value, assignment_style)
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
    default: &'a oxc::Expression<'a>,
    binding: FollowupBinding<'a>,
    value: Place,
    assignment_style: AssignmentStyle,
) -> Result<Option<Place>, CompilerError> {
    let pat_loc = builder.source_location(span);

    let temp = build_temporary_place(builder, pat_loc.clone());

    let test_block = builder.reserve(BlockKind::Value);
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;

    let temp_consequent = temp.clone();
    let pat_loc_consequent = pat_loc.clone();
    let consequent = builder.try_enter(BlockKind::Value, |builder, _| {
        let default_value = lower_reorderable_expression(builder, default)?;
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { place: temp_consequent.clone(), kind: InstructionKind::Const },
                value: default_value,
                type_annotation: None,
                loc: pat_loc_consequent.clone(),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder(0),
            loc: pat_loc_consequent.clone(),
        })
    });

    let temp_alternate = temp.clone();
    let pat_loc_alternate = pat_loc.clone();
    let value_alternate = value.clone();
    let alternate = builder.try_enter(BlockKind::Value, |builder, _| {
        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { place: temp_alternate.clone(), kind: InstructionKind::Const },
                value: value_alternate.clone(),
                type_annotation: None,
                loc: pat_loc_alternate.clone(),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder(0),
            loc: pat_loc_alternate.clone(),
        })
    });

    builder.terminate_with_continuation(
        Terminal::Ternary {
            test: test_block.id,
            fallthrough: continuation_id,
            id: EvaluationOrder(0),
            loc: pat_loc.clone(),
        },
        test_block,
    );

    let undef = lower_value_to_temporary(
        builder,
        InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc: pat_loc.clone() },
    )?;
    let test = lower_value_to_temporary(
        builder,
        InstructionValue::BinaryExpression {
            left: value,
            operator: BinaryOperator::StrictEqual,
            right: undef,
            loc: pat_loc.clone(),
        },
    )?;
    builder.terminate_with_continuation(
        Terminal::Branch {
            test,
            consequent: consequent?,
            alternate: alternate?,
            fallthrough: continuation_id,
            id: EvaluationOrder(0),
            loc: pat_loc.clone(),
        },
        continuation_block,
    );

    // Recursively assign the resolved value to the inner binding.
    match binding {
        FollowupBinding::Target(t) => {
            lower_assignment_target(builder, pat_loc, kind, t, temp, assignment_style)
        }
        FollowupBinding::Identifier(id) => {
            // `{foo = d}` shorthand: the binding is the identifier `foo` itself.
            lower_identifier_followup_store(builder, pat_loc, kind, id, temp)
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
    chain: &'a oxc::ChainExpression<'a>,
) -> Result<InstructionValue<'a>, CompilerError> {
    match &chain.expression {
        oxc::ChainElement::CallExpression(call) => {
            lower_optional_call_expression_impl(builder, call, None)
        }
        oxc::ChainElement::TSNonNullExpression(ts) => {
            // `foo?.bar!` — the non-null assertion wraps a chain-context expression.
            // The original lowered `TSNonNullExpression` by recursing into its inner
            // expression (loc-transparent); preserve that, keeping chain awareness.
            lower_chain_subexpr(builder, &ts.expression)
        }
        // The `@inherit MemberExpression` variants of `ChainElement`.
        oxc::ChainElement::StaticMemberExpression(_)
        | oxc::ChainElement::ComputedMemberExpression(_)
        | oxc::ChainElement::PrivateFieldExpression(_) => {
            let member = chain.expression.as_member_expression().unwrap();
            let place = lower_optional_member_expression_impl(builder, member, None)?.1;
            Ok(InstructionValue::LoadLocal { loc: place.loc.clone(), place })
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
    expr: &'a oxc::Expression<'a>,
) -> Result<InstructionValue<'a>, CompilerError> {
    match expr {
        oxc::Expression::StaticMemberExpression(_)
        | oxc::Expression::ComputedMemberExpression(_)
        | oxc::Expression::PrivateFieldExpression(_)
            if expr_contains_optional(expr) =>
        {
            let member = expr.as_member_expression().unwrap();
            let place = lower_optional_member_expression_impl(builder, member, None)?.1;
            Ok(InstructionValue::LoadLocal { loc: place.loc.clone(), place })
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
    member: &'a oxc::MemberExpression<'a>,
    parent_alternate: Option<BlockId>,
) -> Result<(Place, Place), CompilerError> {
    let optional = member.optional();
    let loc = builder.source_location(member.span());
    let place = build_temporary_place(builder, loc.clone());
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
                InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc: loc.clone() },
            )?;
            lower_value_to_temporary(
                builder,
                InstructionValue::StoreLocal {
                    lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                    value: temp,
                    type_annotation: None,
                    loc: loc.clone(),
                },
            )?;
            Ok(Terminal::Goto {
                block: continuation_id,
                variant: GotoVariant::Break,
                id: EvaluationOrder(0),
                loc: loc.clone(),
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
            id: EvaluationOrder(0),
            loc: loc.clone(),
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
                type_annotation: None,
                loc: loc.clone(),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder(0),
            loc: loc.clone(),
        })
    })?;

    builder.terminate_with_continuation(
        Terminal::Optional {
            optional,
            test: test_block?,
            fallthrough: continuation_id,
            id: EvaluationOrder(0),
            loc: loc.clone(),
        },
        continuation_block,
    );

    Ok((obj, place))
}

/// Lower an oxc optional `CallExpression` (a call link inside a `ChainExpression`).
/// `parent_alternate` threads the shared null/undefined block.
fn lower_optional_call_expression_impl<'a>(
    builder: &mut HirBuilder<'a, '_>,
    call: &'a oxc::CallExpression<'a>,
    parent_alternate: Option<BlockId>,
) -> Result<InstructionValue<'a>, CompilerError> {
    let optional = call.optional;
    let loc = builder.source_location(call.span);
    let place = build_temporary_place(builder, loc.clone());
    let continuation_block = builder.reserve(builder.current_block_kind());
    let continuation_id = continuation_block.id;
    let consequent = builder.reserve(BlockKind::Value);

    let alternate = if let Some(parent_alt) = parent_alternate {
        Ok(parent_alt)
    } else {
        builder.try_enter(BlockKind::Value, |builder, _block_id| {
            let temp = lower_value_to_temporary(
                builder,
                InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc: loc.clone() },
            )?;
            lower_value_to_temporary(
                builder,
                InstructionValue::StoreLocal {
                    lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                    value: temp,
                    type_annotation: None,
                    loc: loc.clone(),
                },
            )?;
            Ok(Terminal::Goto {
                block: continuation_id,
                variant: GotoVariant::Break,
                id: EvaluationOrder(0),
                loc: loc.clone(),
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
            id: EvaluationOrder(0),
            loc: loc.clone(),
        })
    });

    // Block to evaluate if the callee is non-null/undefined.
    builder.try_enter_reserved(consequent, |builder| {
        let args = lower_arguments(builder, &call.arguments)?;
        let temp = build_temporary_place(builder, loc.clone());

        match callee_info.as_ref().unwrap() {
            CalleeInfo::CallExpression { callee } => {
                builder.push(Instruction {
                    id: EvaluationOrder(0),
                    lvalue: temp.clone(),
                    value: InstructionValue::CallExpression {
                        callee: callee.clone(),
                        args,
                        loc: loc.clone(),
                    },
                    loc: loc.clone(),
                    effects: None,
                });
            }
            CalleeInfo::MethodCall { receiver, property } => {
                builder.push(Instruction {
                    id: EvaluationOrder(0),
                    lvalue: temp.clone(),
                    value: InstructionValue::MethodCall {
                        receiver: receiver.clone(),
                        property: property.clone(),
                        args,
                        loc: loc.clone(),
                    },
                    loc: loc.clone(),
                    effects: None,
                });
            }
        }

        lower_value_to_temporary(
            builder,
            InstructionValue::StoreLocal {
                lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                value: temp,
                type_annotation: None,
                loc: loc.clone(),
            },
        )?;
        Ok(Terminal::Goto {
            block: continuation_id,
            variant: GotoVariant::Break,
            id: EvaluationOrder(0),
            loc: loc.clone(),
        })
    })?;

    builder.terminate_with_continuation(
        Terminal::Optional {
            optional,
            test: test_block?,
            fallthrough: continuation_id,
            id: EvaluationOrder(0),
            loc: loc.clone(),
        },
        continuation_block,
    );

    Ok(InstructionValue::LoadLocal { place: place.clone(), loc: place.loc })
}

// =============================================================================
// Function / arrow lowering
// =============================================================================

/// Lower a function/arrow expression to a `FunctionExpression` instruction value.
/// Mirrors the original `lower_function_to_value`.
fn lower_function_to_value<'a>(
    builder: &mut HirBuilder<'a, '_>,
    func: FunctionNode<'a>,
    expr_type: FunctionExpressionType,
) -> Result<InstructionValue<'a>, CompilerError> {
    let loc = match func {
        FunctionNode::Arrow(arrow) => builder.source_location(arrow.span),
        FunctionNode::Function(f) => builder.source_location(f.span),
    };
    let name = match func {
        FunctionNode::Function(f) => f.id.as_ref().map(|id| id.name.to_string()),
        FunctionNode::Arrow(_) => None,
    };
    let lowered_func = lower_function(builder, func)?;
    Ok(InstructionValue::FunctionExpression { name, name_hint: None, lowered_func, expr_type, loc })
}

/// Lower a nested function/arrow node into a `LoweredFunction`. Mirrors the
/// original `lower_function`.
fn lower_function<'a>(
    builder: &mut HirBuilder<'a, '_>,
    func: FunctionNode<'a>,
) -> Result<LoweredFunction, CompilerError> {
    // Extract function parts from the AST node
    let (params, body, id, generator, is_async, func_start, func_end, func_loc, func_node_id) =
        match func {
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
                (
                    arrow.params.as_ref(),
                    body,
                    None::<&str>,
                    false,
                    arrow.r#async,
                    arrow.span.start,
                    arrow.span.end,
                    builder.source_location(arrow.span),
                    Some(arrow.span.start),
                )
            }
            FunctionNode::Function(f) => {
                let body_ref = f.body.as_deref().expect("function expression has a body");
                (
                    f.params.as_ref(),
                    FunctionBody::Block(body_ref),
                    f.id.as_ref().map(|id| id.name.as_str()),
                    f.generator,
                    f.r#async,
                    f.span.start,
                    f.span.end,
                    builder.source_location(f.span),
                    Some(f.span.start),
                )
            }
        };

    // Find the function's scope. For synthetic zero-width functions (e.g., desugared
    // match IIFEs from Hermes with start=end=0), node_to_scope won't have an entry.
    let function_scope =
        if let Some(scope) = builder.scope_info().resolve_scope_for_node(func_node_id) {
            scope
        } else if func_start < func_end {
            builder.scope_info().program_scope
        } else {
            let parent = builder.function_scope();
            let scope_info = builder.scope_info();
            let mapped: rustc_hash::FxHashSet<crate::scope::ScopeId> =
                scope_info.node_to_scope.values().copied().collect();
            let param_names: Vec<String> = params
                .items
                .iter()
                .filter_map(|p| {
                    if let oxc::BindingPattern::BindingIdentifier(id) = &p.pattern {
                        Some(id.name.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let mut descendants = rustc_hash::FxHashSet::default();
            descendants.insert(parent);
            let mut changed = true;
            while changed {
                changed = false;
                for (i, scope) in scope_info.scopes.iter().enumerate() {
                    let sid = crate::scope::ScopeId(i as u32);
                    if let Some(p) = scope.parent {
                        if descendants.contains(&p) && !descendants.contains(&sid) {
                            descendants.insert(sid);
                            changed = true;
                        }
                    }
                }
            }
            let mut found = scope_info.program_scope;
            for (i, scope) in scope_info.scopes.iter().enumerate() {
                let sid = crate::scope::ScopeId(i as u32);
                if let Some(p) = scope.parent {
                    if descendants.contains(&p)
                        && matches!(scope.kind, ScopeKind::Function)
                        && !mapped.contains(&sid)
                        && !builder.is_synthetic_scope_claimed(sid)
                    {
                        if !param_names.is_empty() {
                            let all_match =
                                param_names.iter().all(|name| scope.bindings.contains_key(name));
                            if !all_match {
                                continue;
                            }
                        }
                        found = sid;
                        break;
                    }
                }
            }
            builder.claim_synthetic_scope(found);
            found
        };

    let component_scope = builder.component_scope();
    let scope_info = builder.scope_info();

    let parent_bindings = builder.bindings().clone();
    let parent_used_names = builder.used_names().clone();
    let context_ids = builder.context_identifiers().clone();
    let ident_locs = builder.identifier_locs();
    let line_offsets = builder.line_offsets();

    // For synthetic functions with zero-width position ranges, position-based
    // reference filtering fails. Walk the body AST to collect actual positions.
    let ref_override = if func_start >= func_end {
        Some(collect_identifier_node_ids_from_body(&body))
    } else {
        None
    };

    // Gather captured context
    let captured_context = gather_captured_context(
        scope_info,
        function_scope,
        component_scope,
        func_start,
        func_end,
        ident_locs,
        ref_override.as_ref(),
    );
    let merged_context: FxIndexMap<crate::scope::BindingId, Option<SourceLocation>> = {
        let parent_context = builder.context().clone();
        let mut merged = parent_context;
        for (k, v) in captured_context {
            merged.insert(k, v);
        }
        merged
    };

    // Use scope_info_and_env_mut to avoid conflicting borrows
    let (scope_info, env) = builder.scope_info_and_env_mut();
    let (hir_func, child_used_names, child_bindings) = lower_inner(
        params,
        body,
        id,
        generator,
        is_async,
        func_loc,
        scope_info,
        env,
        Some(parent_bindings),
        Some(parent_used_names),
        merged_context,
        function_scope,
        component_scope,
        &context_ids,
        false, // nested function
        ident_locs,
        line_offsets,
    )?;

    builder.merge_used_names(child_used_names);
    builder.merge_bindings(child_bindings);

    let func_id = builder.environment_mut().add_function(hir_func);
    Ok(LoweredFunction { func: func_id })
}

/// Lower a function declaration statement to a FunctionExpression + StoreLocal.
fn lower_function_declaration<'a>(
    builder: &mut HirBuilder<'a, '_>,
    func_decl: &'a oxc::Function<'a>,
) -> Result<(), CompilerError> {
    let loc = builder.source_location(func_decl.span);
    let func_start = func_decl.span.start;
    let func_end = func_decl.span.end;

    let func_name = func_decl.id.as_ref().map(|id| id.name.to_string());

    // Find the function's scope
    let function_scope = builder
        .scope_info()
        .resolve_scope_for_node(Some(func_decl.span.start))
        .unwrap_or(builder.scope_info().program_scope);

    let component_scope = builder.component_scope();
    let scope_info = builder.scope_info();

    let parent_bindings = builder.bindings().clone();
    let parent_used_names = builder.used_names().clone();
    let context_ids = builder.context_identifiers().clone();
    let ident_locs = builder.identifier_locs();
    let line_offsets = builder.line_offsets();

    // Gather captured context
    let captured_context = gather_captured_context(
        scope_info,
        function_scope,
        component_scope,
        func_start,
        func_end,
        ident_locs,
        None,
    );
    let merged_context: FxIndexMap<crate::scope::BindingId, Option<SourceLocation>> = {
        let parent_context = builder.context().clone();
        let mut merged = parent_context;
        for (k, v) in captured_context {
            merged.insert(k, v);
        }
        merged
    };

    let body_ref = func_decl.body.as_deref().expect("function declaration has a body");
    let (scope_info, env) = builder.scope_info_and_env_mut();
    let (hir_func, child_used_names, child_bindings) = lower_inner(
        func_decl.params.as_ref(),
        FunctionBody::Block(body_ref),
        func_decl.id.as_ref().map(|id| id.name.as_str()),
        func_decl.generator,
        func_decl.r#async,
        loc.clone(),
        scope_info,
        env,
        Some(parent_bindings),
        Some(parent_used_names),
        merged_context,
        function_scope,
        component_scope,
        &context_ids,
        false, // nested function
        ident_locs,
        line_offsets,
    )?;

    builder.merge_used_names(child_used_names);
    builder.merge_bindings(child_bindings);

    let func_id = builder.environment_mut().add_function(hir_func);
    let lowered_func = LoweredFunction { func: func_id };

    // Emit FunctionExpression instruction
    let fn_value = InstructionValue::FunctionExpression {
        name: func_name.clone(),
        name_hint: None,
        lowered_func,
        expr_type: FunctionExpressionType::FunctionDeclaration,
        loc: loc.clone(),
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
    if let Some(ref name) = func_name {
        if let Some(id_node) = &func_decl.id {
            let start = id_node.span.start;
            let ident_loc = builder.source_location(id_node.span);
            let scope_binding = builder.get_function_declaration_binding(function_scope, name);
            let mut is_context = false;
            let binding = match scope_binding {
                Some(binding_id) => {
                    is_context = builder.is_context_binding(binding_id);
                    let binding_kind = crate::react_compiler_lowering::convert_binding_kind(
                        &builder.scope_info().bindings[binding_id.0 as usize].kind,
                    );
                    let identifier =
                        builder.resolve_binding_with_loc(name, binding_id, ident_loc.clone())?;
                    VariableBinding::Identifier { identifier, binding_kind }
                }
                None => {
                    let mut binding = builder.resolve_identifier(
                        name,
                        start,
                        ident_loc.clone(),
                        Some(id_node.span.start),
                    )?;
                    if matches!(&binding, VariableBinding::Global { .. }) {
                        // For function redeclarations (e.g., `function x() {} function x() {}`),
                        // the redeclaration's identifier may not be in ref_node_id_to_binding
                        // (OXC/SWC don't map constant violations). Retry using the first
                        // declaration's node_id from the scope chain.
                        let fallback = {
                            let si = builder.scope_info();
                            let scope_id = si
                                .resolve_scope_for_node(Some(func_decl.span.start))
                                .unwrap_or(si.program_scope);
                            si.get_binding(scope_id, name).map(|bid| {
                                let b = &si.bindings[bid.0 as usize];
                                (b.declaration_start.unwrap_or(0), b.declaration_node_id)
                            })
                        };
                        if let Some((ds, ds_node_id)) = fallback {
                            binding = builder.resolve_identifier(
                                name,
                                ds,
                                ident_loc.clone(),
                                ds_node_id,
                            )?;
                        }
                    }
                    if matches!(&binding, VariableBinding::Identifier { .. }) {
                        is_context =
                            builder.is_context_identifier(name, start, Some(id_node.span.start));
                    }
                    binding
                }
            };
            match binding {
                VariableBinding::Identifier { identifier, .. } => {
                    // Don't override the identifier's declaration loc here.
                    // For function redeclarations (e.g., `function x() {} function x() {}`),
                    // the identifier's loc should remain the first declaration's loc,
                    // which was already set during define_binding.
                    // Use the full function declaration loc for the Place,
                    // matching the TS behavior where lowerAssignment uses stmt.node.loc
                    let place = Place {
                        identifier,
                        reactive: false,
                        effect: Effect::Unknown,
                        loc: loc.clone(),
                    };
                    if is_context {
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreContext {
                                lvalue: LValue { kind: InstructionKind::Function, place },
                                value: fn_place,
                                loc,
                            },
                        )?;
                    } else {
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreLocal {
                                lvalue: LValue { kind: InstructionKind::Function, place },
                                value: fn_place,
                                type_annotation: None,
                                loc,
                            },
                        )?;
                    }
                }
                _ => {
                    builder.record_error(CompilerErrorDetail {
                        category: ErrorCategory::Invariant,
                        reason: format!(
                            "Could not find binding for function declaration `{}`",
                            name
                        ),
                        description: None,
                        loc,
                        suggestions: None,
                    })?;
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
    params: &'a oxc::FormalParameters<'a>,
    body: &'a oxc::FunctionBody<'a>,
    generator: bool,
    is_async: bool,
) -> Result<LoweredFunction, CompilerError> {
    let func_start = method_span.start;
    let func_end = method_span.end;
    let func_loc = builder.source_location(method_span);

    let function_scope = builder
        .scope_info()
        .resolve_scope_for_node(Some(method_span.start))
        .unwrap_or(builder.scope_info().program_scope);

    let component_scope = builder.component_scope();
    let scope_info = builder.scope_info();

    let parent_bindings = builder.bindings().clone();
    let parent_used_names = builder.used_names().clone();
    let context_ids = builder.context_identifiers().clone();
    let ident_locs = builder.identifier_locs();
    let line_offsets = builder.line_offsets();

    let captured_context = gather_captured_context(
        scope_info,
        function_scope,
        component_scope,
        func_start,
        func_end,
        ident_locs,
        None,
    );
    let merged_context: FxIndexMap<crate::scope::BindingId, Option<SourceLocation>> = {
        let parent_context = builder.context().clone();
        let mut merged = parent_context;
        for (k, v) in captured_context {
            merged.insert(k, v);
        }
        merged
    };

    let (scope_info, env) = builder.scope_info_and_env_mut();
    let (hir_func, child_used_names, child_bindings) = lower_inner(
        params,
        FunctionBody::Block(body),
        None,
        generator,
        is_async,
        func_loc,
        scope_info,
        env,
        Some(parent_bindings),
        Some(parent_used_names),
        merged_context,
        function_scope,
        component_scope,
        &context_ids,
        false, // nested function
        ident_locs,
        line_offsets,
    )?;

    builder.merge_used_names(child_used_names);
    builder.merge_bindings(child_bindings);

    let func_id = builder.environment_mut().add_function(hir_func);
    Ok(LoweredFunction { func: func_id })
}

fn gather_captured_context(
    scope_info: &ScopeInfo,
    function_scope: crate::scope::ScopeId,
    component_scope: crate::scope::ScopeId,
    func_start: u32,
    func_end: u32,
    identifier_locs: &IdentifierLocIndex,
    ref_node_ids_override: Option<&FxIndexSet<u32>>,
) -> FxIndexMap<crate::scope::BindingId, Option<SourceLocation>> {
    let parent_scope = scope_info.scopes[function_scope.0 as usize].parent;
    let pure_scopes = match parent_scope {
        Some(parent) => capture_scopes(scope_info, parent, component_scope),
        None => FxIndexSet::default(),
    };

    // Collect the earliest (lowest source position) reference location for each
    // captured binding. Using the minimum position makes the result independent of
    // ref_node_id_to_binding iteration order, matching the behavior the TS compiler
    // gets from Babel's position-ordered traversal.
    let mut captured: rustc_hash::FxHashMap<
        crate::scope::BindingId,
        (u32, Option<SourceLocation>), // (min_position, loc)
    > = rustc_hash::FxHashMap::default();

    for (&ref_nid, &binding_id) in &scope_info.ref_node_id_to_binding {
        if let Some(allowed) = ref_node_ids_override {
            if !allowed.contains(&ref_nid) {
                continue;
            }
        } else {
            // Range check: use the position stored in identifier_locs
            let ref_start = identifier_locs.get(&ref_nid).map(|e| e.start).unwrap_or(0);
            if ref_start < func_start || ref_start >= func_end {
                continue;
            }
        }
        let binding = &scope_info.bindings[binding_id.0 as usize];
        // Skip references that are actually the binding's own declaration site
        if binding.declaration_node_id == Some(ref_nid) {
            continue;
        }
        // Skip function/class declaration names that are not expression references.
        // Skip type-annotation references: TS's gatherCapturedContext traverse
        // skips TypeAnnotation/TSTypeAnnotation/TypeAlias/TSTypeAliasDeclaration
        // subtrees, so identifiers there never become captures (they DO still
        // feed FindContextIdentifiers and the hoisting analysis, which have no
        // such skip in TS).
        if let Some(entry) = identifier_locs.get(&ref_nid) {
            if entry.is_declaration_name || entry.in_type_annotation {
                continue;
            }
        }
        // Skip type-only bindings
        if binding.declaration_type == "TypeAlias"
            || binding.declaration_type == "OpaqueType"
            || binding.declaration_type == "InterfaceDeclaration"
            || binding.declaration_type == "TSTypeAliasDeclaration"
            || binding.declaration_type == "TSInterfaceDeclaration"
            || binding.declaration_type == "TSEnumDeclaration"
        {
            continue;
        }
        if pure_scopes.contains(&binding.scope) {
            let ref_start = identifier_locs.get(&ref_nid).map(|e| e.start).unwrap_or(0);
            // Skip references whose start offset aliases the binding's own
            // declaration offset. Hermes desugars (component syntax) reuse the
            // original source offsets for generated nodes, so a sibling
            // reference structurally OUTSIDE this function (e.g. the forwardRef
            // argument naming the desugared inner function) can fall inside the
            // function's position range and alias the declaration position. In
            // real source a non-declaration reference can never share its
            // declaration's offset, so this only filters desugared aliases.
            if binding.declaration_start == Some(ref_start) {
                continue;
            }
            let loc = identifier_locs.get(&ref_nid).map(|entry| {
                if let Some(oe_loc) = &entry.opening_element_loc {
                    oe_loc.clone()
                } else {
                    entry.loc.clone()
                }
            });
            captured
                .entry(binding.id)
                .and_modify(|(min_pos, existing_loc)| {
                    if ref_start < *min_pos {
                        *min_pos = ref_start;
                        *existing_loc = loc.clone();
                    }
                })
                .or_insert((ref_start, loc));
        }
    }

    // Sort captured entries by source position so context declarations appear
    // in source order, matching the TS compiler's position-ordered traversal.
    let mut sorted: Vec<_> = captured.into_iter().collect();
    sorted.sort_by_key(|(_, (pos, _))| *pos);

    sorted.into_iter().map(|(bid, (_, loc))| (bid, loc)).collect()
}

fn capture_scopes(
    scope_info: &ScopeInfo,
    from: crate::scope::ScopeId,
    to: crate::scope::ScopeId,
) -> FxIndexSet<crate::scope::ScopeId> {
    let mut result = FxIndexSet::default();
    let mut current = Some(from);
    while let Some(scope_id) = current {
        result.insert(scope_id);
        if scope_id == to {
            break;
        }
        current = scope_info.scopes[scope_id.0 as usize].parent;
    }
    result
}

fn collect_identifier_node_ids_from_body(body: &FunctionBody) -> FxIndexSet<u32> {
    let mut positions = FxIndexSet::default();
    match body {
        FunctionBody::Block(block) => {
            for stmt in &block.statements {
                collect_identifier_node_ids_from_stmt(stmt, &mut positions);
            }
        }
        FunctionBody::Expression(expr) => {
            collect_identifier_node_ids_from_expr(expr, &mut positions);
        }
    }
    positions
}

fn collect_identifier_node_ids_from_stmt(stmt: &oxc::Statement, positions: &mut FxIndexSet<u32>) {
    match stmt {
        oxc::Statement::ExpressionStatement(s) => {
            collect_identifier_node_ids_from_expr(&s.expression, positions)
        }
        oxc::Statement::ReturnStatement(s) => {
            if let Some(arg) = &s.argument {
                collect_identifier_node_ids_from_expr(arg, positions);
            }
        }
        oxc::Statement::ThrowStatement(s) => {
            collect_identifier_node_ids_from_expr(&s.argument, positions)
        }
        oxc::Statement::BlockStatement(s) => {
            for stmt in &s.body {
                collect_identifier_node_ids_from_stmt(stmt, positions);
            }
        }
        oxc::Statement::IfStatement(s) => {
            collect_identifier_node_ids_from_expr(&s.test, positions);
            collect_identifier_node_ids_from_stmt(&s.consequent, positions);
            if let Some(alt) = &s.alternate {
                collect_identifier_node_ids_from_stmt(alt, positions);
            }
        }
        oxc::Statement::VariableDeclaration(s) => {
            for decl in &s.declarations {
                if let Some(init) = &decl.init {
                    collect_identifier_node_ids_from_expr(init, positions);
                }
            }
        }
        _ => {}
    }
}

fn collect_identifier_node_ids_from_expr(expr: &oxc::Expression, positions: &mut FxIndexSet<u32>) {
    match expr {
        oxc::Expression::Identifier(id) => {
            positions.insert(id.span.start);
        }
        oxc::Expression::CallExpression(call) => {
            collect_identifier_node_ids_from_expr(&call.callee, positions);
            for arg in &call.arguments {
                if let Some(e) = arg.as_expression() {
                    collect_identifier_node_ids_from_expr(e, positions);
                } else if let oxc::Argument::SpreadElement(s) = arg {
                    collect_identifier_node_ids_from_expr(&s.argument, positions);
                }
            }
        }
        oxc::Expression::BinaryExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.left, positions);
            collect_identifier_node_ids_from_expr(&e.right, positions);
        }
        oxc::Expression::ConditionalExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.test, positions);
            collect_identifier_node_ids_from_expr(&e.consequent, positions);
            collect_identifier_node_ids_from_expr(&e.alternate, positions);
        }
        oxc::Expression::LogicalExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.left, positions);
            collect_identifier_node_ids_from_expr(&e.right, positions);
        }
        oxc::Expression::StaticMemberExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.object, positions);
        }
        oxc::Expression::ComputedMemberExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.object, positions);
        }
        oxc::Expression::PrivateFieldExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.object, positions);
        }
        oxc::Expression::ChainExpression(chain) => {
            collect_identifier_node_ids_from_chain_element(&chain.expression, positions);
        }
        oxc::Expression::UpdateExpression(e) => {
            collect_identifier_node_ids_from_simple_target(&e.argument, positions);
        }
        oxc::Expression::FunctionExpression(func) => {
            if let Some(body) = func.body.as_deref() {
                for stmt in &body.statements {
                    collect_identifier_node_ids_from_stmt(stmt, positions);
                }
            }
        }
        oxc::Expression::UnaryExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.argument, positions);
        }
        oxc::Expression::ParenthesizedExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.expression, positions);
        }
        oxc::Expression::TSAsExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.expression, positions);
        }
        oxc::Expression::TSSatisfiesExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.expression, positions);
        }
        oxc::Expression::TSTypeAssertion(e) => {
            collect_identifier_node_ids_from_expr(&e.expression, positions);
        }
        oxc::Expression::TSNonNullExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.expression, positions);
        }
        oxc::Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                if let Some(oxc::Statement::ExpressionStatement(es)) = arrow.body.statements.first()
                {
                    collect_identifier_node_ids_from_expr(&es.expression, positions);
                }
            } else {
                for stmt in &arrow.body.statements {
                    collect_identifier_node_ids_from_stmt(stmt, positions);
                }
            }
        }
        oxc::Expression::JSXElement(el) => {
            collect_identifier_node_ids_from_jsx_element(el, positions);
        }
        oxc::Expression::JSXFragment(frag) => {
            for child in &frag.children {
                collect_identifier_node_ids_from_jsx_child(child, positions);
            }
        }
        oxc::Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                if let Some(e) = elem.as_expression() {
                    collect_identifier_node_ids_from_expr(e, positions);
                } else if let oxc::ArrayExpressionElement::SpreadElement(s) = elem {
                    collect_identifier_node_ids_from_expr(&s.argument, positions);
                }
            }
        }
        oxc::Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    oxc::ObjectPropertyKind::ObjectProperty(p) => {
                        collect_identifier_node_ids_from_expr(&p.value, positions);
                    }
                    oxc::ObjectPropertyKind::SpreadProperty(s) => {
                        collect_identifier_node_ids_from_expr(&s.argument, positions);
                    }
                }
            }
        }
        oxc::Expression::NewExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.callee, positions);
            for arg in &e.arguments {
                if let Some(ex) = arg.as_expression() {
                    collect_identifier_node_ids_from_expr(ex, positions);
                } else if let oxc::Argument::SpreadElement(s) = arg {
                    collect_identifier_node_ids_from_expr(&s.argument, positions);
                }
            }
        }
        oxc::Expression::AssignmentExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.right, positions);
        }
        oxc::Expression::TemplateLiteral(e) => {
            for expr in &e.expressions {
                collect_identifier_node_ids_from_expr(expr, positions);
            }
        }
        oxc::Expression::SequenceExpression(e) => {
            for expr in &e.expressions {
                collect_identifier_node_ids_from_expr(expr, positions);
            }
        }
        _ => {}
    }
}

fn collect_identifier_node_ids_from_chain_element(
    element: &oxc::ChainElement,
    positions: &mut FxIndexSet<u32>,
) {
    match element {
        oxc::ChainElement::CallExpression(call) => {
            collect_identifier_node_ids_from_expr(&call.callee, positions);
            for arg in &call.arguments {
                if let Some(e) = arg.as_expression() {
                    collect_identifier_node_ids_from_expr(e, positions);
                } else if let oxc::Argument::SpreadElement(s) = arg {
                    collect_identifier_node_ids_from_expr(&s.argument, positions);
                }
            }
        }
        oxc::ChainElement::TSNonNullExpression(e) => {
            collect_identifier_node_ids_from_expr(&e.expression, positions);
        }
        oxc::ChainElement::StaticMemberExpression(m) => {
            collect_identifier_node_ids_from_expr(&m.object, positions);
        }
        oxc::ChainElement::ComputedMemberExpression(m) => {
            collect_identifier_node_ids_from_expr(&m.object, positions);
        }
        oxc::ChainElement::PrivateFieldExpression(m) => {
            collect_identifier_node_ids_from_expr(&m.object, positions);
        }
    }
}

fn collect_identifier_node_ids_from_simple_target(
    target: &oxc::SimpleAssignmentTarget,
    positions: &mut FxIndexSet<u32>,
) {
    match target {
        oxc::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
            positions.insert(id.span.start);
        }
        oxc::SimpleAssignmentTarget::StaticMemberExpression(m) => {
            collect_identifier_node_ids_from_expr(&m.object, positions);
        }
        oxc::SimpleAssignmentTarget::ComputedMemberExpression(m) => {
            collect_identifier_node_ids_from_expr(&m.object, positions);
        }
        oxc::SimpleAssignmentTarget::PrivateFieldExpression(m) => {
            collect_identifier_node_ids_from_expr(&m.object, positions);
        }
        _ => {}
    }
}

fn collect_identifier_node_ids_from_jsx_element(
    el: &oxc::JSXElement,
    positions: &mut FxIndexSet<u32>,
) {
    if let oxc::JSXElementName::IdentifierReference(id) = &el.opening_element.name {
        positions.insert(id.span.start);
    }
    for attr in &el.opening_element.attributes {
        match attr {
            oxc::JSXAttributeItem::Attribute(a) => {
                if let Some(oxc::JSXAttributeValue::ExpressionContainer(c)) = &a.value {
                    if let Some(e) = c.expression.as_expression() {
                        collect_identifier_node_ids_from_expr(e, positions);
                    }
                }
            }
            oxc::JSXAttributeItem::SpreadAttribute(a) => {
                collect_identifier_node_ids_from_expr(&a.argument, positions);
            }
        }
    }
    for child in &el.children {
        collect_identifier_node_ids_from_jsx_child(child, positions);
    }
}

fn collect_identifier_node_ids_from_jsx_child(
    child: &oxc::JSXChild,
    positions: &mut FxIndexSet<u32>,
) {
    match child {
        oxc::JSXChild::ExpressionContainer(c) => {
            if let Some(e) = c.expression.as_expression() {
                collect_identifier_node_ids_from_expr(e, positions);
            }
        }
        oxc::JSXChild::Element(child_el) => {
            collect_identifier_node_ids_from_jsx_element(child_el, positions);
        }
        oxc::JSXChild::Fragment(frag) => {
            for c in &frag.children {
                collect_identifier_node_ids_from_jsx_child(c, positions);
            }
        }
        oxc::JSXChild::Spread(s) => {
            collect_identifier_node_ids_from_expr(&s.expression, positions);
        }
        _ => {}
    }
}

fn lower_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    expr: &'a oxc::Expression<'a>,
) -> Result<InstructionValue<'a>, CompilerError> {
    match expr {
        oxc::Expression::Identifier(ident) => {
            let loc = builder.source_location(ident.span);
            let start = ident.span.start;
            let place =
                lower_identifier(builder, ident.name.as_str(), start, loc.clone(), Some(start))?;
            if builder.is_context_identifier(ident.name.as_str(), start, Some(start)) {
                Ok(InstructionValue::LoadContext { place, loc })
            } else {
                Ok(InstructionValue::LoadLocal { place, loc })
            }
        }
        oxc::Expression::NullLiteral(lit) => Ok(InstructionValue::Primitive {
            value: PrimitiveValue::Null,
            loc: builder.source_location(lit.span),
        }),
        oxc::Expression::BooleanLiteral(lit) => Ok(InstructionValue::Primitive {
            value: PrimitiveValue::Boolean(lit.value),
            loc: builder.source_location(lit.span),
        }),
        oxc::Expression::NumericLiteral(lit) => Ok(InstructionValue::Primitive {
            value: PrimitiveValue::Number(FloatValue::new(lit.value)),
            loc: builder.source_location(lit.span),
        }),
        oxc::Expression::StringLiteral(lit) => Ok(InstructionValue::Primitive {
            value: PrimitiveValue::String(lit.value.to_string().into()),
            loc: builder.source_location(lit.span),
        }),
        oxc::Expression::BinaryExpression(bin) => {
            let loc = builder.source_location(bin.span);
            let left = lower_expression_to_temporary(builder, &bin.left)?;
            let right = lower_expression_to_temporary(builder, &bin.right)?;
            Ok(InstructionValue::BinaryExpression {
                operator: convert_binary_operator(bin.operator),
                left,
                right,
                loc,
            })
        }
        oxc::Expression::UnaryExpression(unary) => {
            let loc = builder.source_location(unary.span);
            match unary.operator {
                oxc::UnaryOperator::Delete => {
                    // TODO(stage1a-arms): delete needs member lowering
                    // (PropertyDelete / ComputedDelete).
                    Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
                }
                op => {
                    let value = lower_expression_to_temporary(builder, &unary.argument)?;
                    Ok(InstructionValue::UnaryExpression {
                        operator: convert_unary_operator(op),
                        value,
                        loc,
                    })
                }
            }
        }
        oxc::Expression::LogicalExpression(logical) => {
            let loc = builder.source_location(logical.span);
            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let test_block = builder.reserve(BlockKind::Value);
            let test_block_id = test_block.id;
            let place = build_temporary_place(builder, loc.clone());
            let left_loc = builder.source_location(logical.left.span());
            let left_place = build_temporary_place(builder, left_loc);

            let consequent_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
                lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                        value: left_place.clone(),
                        type_annotation: None,
                        loc: left_place.loc.clone(),
                    },
                )?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: left_place.loc.clone(),
                })
            });

            let alternate_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
                let right = lower_expression_to_temporary(builder, &logical.right)?;
                let right_loc = right.loc.clone();
                lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                        value: right,
                        type_annotation: None,
                        loc: right_loc.clone(),
                    },
                )?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: right_loc,
                })
            });

            let hir_op = match logical.operator {
                oxc::LogicalOperator::And => LogicalOperator::And,
                oxc::LogicalOperator::Or => LogicalOperator::Or,
                oxc::LogicalOperator::Coalesce => LogicalOperator::NullishCoalescing,
            };

            builder.terminate_with_continuation(
                Terminal::Logical {
                    operator: hir_op,
                    test: test_block_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                test_block,
            );

            let left_value = lower_expression_to_temporary(builder, &logical.left)?;
            builder.push(Instruction {
                id: EvaluationOrder(0),
                lvalue: left_place.clone(),
                value: InstructionValue::LoadLocal { place: left_value, loc: loc.clone() },
                effects: None,
                loc: loc.clone(),
            });

            builder.terminate_with_continuation(
                Terminal::Branch {
                    test: left_place,
                    consequent: consequent_block?,
                    alternate: alternate_block?,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                continuation_block,
            );

            Ok(InstructionValue::LoadLocal { place: place.clone(), loc: place.loc.clone() })
        }
        oxc::Expression::StaticMemberExpression(_)
        | oxc::Expression::ComputedMemberExpression(_)
        | oxc::Expression::PrivateFieldExpression(_) => {
            let lowered = lower_member_expression(builder, expr.as_member_expression().unwrap())?;
            Ok(lowered.value)
        }
        oxc::Expression::CallExpression(call) => {
            let loc = builder.source_location(call.span);
            if let Some(member) = call.callee.as_member_expression() {
                let lowered = lower_member_expression(builder, member)?;
                let property = lower_value_to_temporary(builder, lowered.value)?;
                let args = lower_arguments(builder, &call.arguments)?;
                Ok(InstructionValue::MethodCall { receiver: lowered.object, property, args, loc })
            } else {
                let callee = lower_expression_to_temporary(builder, &call.callee)?;
                let args = lower_arguments(builder, &call.arguments)?;
                Ok(InstructionValue::CallExpression { callee, args, loc })
            }
        }
        oxc::Expression::ConditionalExpression(cond) => {
            let loc = builder.source_location(cond.span);
            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let test_block = builder.reserve(BlockKind::Value);
            let test_block_id = test_block.id;
            let place = build_temporary_place(builder, loc.clone());

            // Block for the consequent (test is truthy)
            let consequent_ast_loc = builder.source_location(cond.consequent.span());
            let consequent_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
                let consequent = lower_expression_to_temporary(builder, &cond.consequent)?;
                lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                        value: consequent,
                        type_annotation: None,
                        loc: loc.clone(),
                    },
                )?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: consequent_ast_loc,
                })
            });

            // Block for the alternate (test is falsy)
            let alternate_ast_loc = builder.source_location(cond.alternate.span());
            let alternate_block = builder.try_enter(BlockKind::Value, |builder, _block_id| {
                let alternate = lower_expression_to_temporary(builder, &cond.alternate)?;
                lower_value_to_temporary(
                    builder,
                    InstructionValue::StoreLocal {
                        lvalue: LValue { kind: InstructionKind::Const, place: place.clone() },
                        value: alternate,
                        type_annotation: None,
                        loc: loc.clone(),
                    },
                )?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: alternate_ast_loc,
                })
            });

            builder.terminate_with_continuation(
                Terminal::Ternary {
                    test: test_block_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
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
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                continuation_block,
            );

            Ok(InstructionValue::LoadLocal { place: place.clone(), loc: place.loc.clone() })
        }
        oxc::Expression::SequenceExpression(seq) => {
            let loc = builder.source_location(seq.span);

            if seq.expressions.is_empty() {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Syntax,
                    reason: "Expected sequence expression to have at least one expression"
                        .to_string(),
                    description: None,
                    loc: loc.clone(),
                    suggestions: None,
                })?;
                return Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc });
            }

            let continuation_block = builder.reserve(builder.current_block_kind());
            let continuation_id = continuation_block.id;
            let place = build_temporary_place(builder, loc.clone());

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
                            type_annotation: None,
                            loc: loc.clone(),
                        },
                    )?;
                }
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                })
            });

            builder.terminate_with_continuation(
                Terminal::Sequence {
                    block: sequence_block?,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                continuation_block,
            );
            Ok(InstructionValue::LoadLocal { place, loc })
        }
        oxc::Expression::NewExpression(new_expr) => {
            let loc = builder.source_location(new_expr.span);
            let callee = lower_expression_to_temporary(builder, &new_expr.callee)?;
            let args = lower_arguments(builder, &new_expr.arguments)?;
            Ok(InstructionValue::NewExpression { callee, args, loc })
        }
        oxc::Expression::TemplateLiteral(tmpl) => {
            let loc = builder.source_location(tmpl.span);
            let subexprs: Vec<Place> = tmpl
                .expressions
                .iter()
                .map(|e| lower_expression_to_temporary(builder, e))
                .collect::<Result<Vec<_>, _>>()?;
            let quasis: Vec<TemplateQuasi> =
                tmpl.quasis.iter().map(template_quasi_from_oxc).collect();
            Ok(InstructionValue::TemplateLiteral { subexprs, quasis, loc })
        }
        oxc::Expression::TaggedTemplateExpression(tagged) => {
            let loc = builder.source_location(tagged.span);
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
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Todo,
                    reason: "(BuildHIR::lowerExpression) Handle tagged template where cooked value is different from raw value".to_string(),
                    description: None,
                    loc: loc.clone(),
                    suggestions: None,
                })?;
                return Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc });
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
            Ok(InstructionValue::TaggedTemplateExpression { tag, quasis, subexprs, loc })
        }
        oxc::Expression::AwaitExpression(await_expr) => {
            let loc = builder.source_location(await_expr.span);
            let value = lower_expression_to_temporary(builder, &await_expr.argument)?;
            Ok(InstructionValue::Await { value, loc })
        }
        oxc::Expression::YieldExpression(yld) => {
            let loc = builder.source_location(yld.span);
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "(BuildHIR::lowerExpression) Handle YieldExpression expressions"
                    .to_string(),
                description: None,
                loc: loc.clone(),
                suggestions: None,
            })?;
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
        }
        oxc::Expression::MetaProperty(meta) => {
            let loc = builder.source_location(meta.span);
            if meta.meta.name == "import" && meta.property.name == "meta" {
                Ok(InstructionValue::MetaProperty {
                    meta: meta.meta.name.to_string(),
                    property: meta.property.name.to_string(),
                    loc,
                })
            } else {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Todo,
                    reason: "(BuildHIR::lowerExpression) Handle MetaProperty expressions other than import.meta".to_string(),
                    description: None,
                    loc: loc.clone(),
                    suggestions: None,
                })?;
                Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
            }
        }
        oxc::Expression::ClassExpression(cls) => {
            let loc = builder.source_location(cls.span);
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "(BuildHIR::lowerExpression) Handle ClassExpression expressions"
                    .to_string(),
                description: None,
                loc: loc.clone(),
                suggestions: None,
            })?;
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
        }
        oxc::Expression::Super(sup) => {
            let loc = builder.source_location(sup.span);
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "(BuildHIR::lowerExpression) Handle Super expressions".to_string(),
                description: None,
                loc: loc.clone(),
                suggestions: None,
            })?;
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
        }
        oxc::Expression::ThisExpression(this) => {
            let loc = builder.source_location(this.span);
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "(BuildHIR::lowerExpression) Handle ThisExpression expressions".to_string(),
                description: None,
                loc: loc.clone(),
                suggestions: None,
            })?;
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
        }
        oxc::Expression::ImportExpression(imp) => {
            // oxc's `import(source, options?)` maps to Babel's
            // `CallExpression { callee: Import, arguments: [source] + options? }`.
            // The `Import` keyword callee bails (records an error), then the source
            // and options arguments are lowered left-to-right.
            let loc = builder.source_location(imp.span);
            // The `import` keyword has no standalone node in oxc; synthesize its
            // span ([start, start+6)) so the callee bail error and temporary carry
            // the keyword loc, matching Babel's `Import` node loc.
            let import_keyword_loc =
                builder.source_location(oxc_span::Span::new(imp.span.start, imp.span.start + 6));
            let callee = lower_import_keyword_to_temporary(builder, &import_keyword_loc)?;
            let mut args: Vec<PlaceOrSpread> = Vec::new();
            let source = lower_expression_to_temporary(builder, &imp.source)?;
            args.push(PlaceOrSpread::Place(source));
            if let Some(options) = &imp.options {
                let options = lower_expression_to_temporary(builder, options)?;
                args.push(PlaceOrSpread::Place(options));
            }
            Ok(InstructionValue::CallExpression { callee, args, loc })
        }
        oxc::Expression::PrivateInExpression(priv_in) => {
            // `#f in obj` maps to Babel's `BinaryExpression { op: In, left: PrivateName, right }`.
            // The PrivateName left operand bails (records an error), then the right
            // operand is lowered.
            let loc = builder.source_location(priv_in.span);
            let left = lower_private_name_to_temporary(builder, priv_in.left.span)?;
            let right = lower_expression_to_temporary(builder, &priv_in.right)?;
            Ok(InstructionValue::BinaryExpression {
                operator: BinaryOperator::In,
                left,
                right,
                loc,
            })
        }
        oxc::Expression::UpdateExpression(update) => {
            let loc = builder.source_location(update.span);
            match &update.argument {
                oxc::SimpleAssignmentTarget::StaticMemberExpression(_)
                | oxc::SimpleAssignmentTarget::ComputedMemberExpression(_)
                | oxc::SimpleAssignmentTarget::PrivateFieldExpression(_) => {
                    let binary_op = match update.operator {
                        oxc::UpdateOperator::Increment => BinaryOperator::Add,
                        oxc::UpdateOperator::Decrement => BinaryOperator::Subtract,
                    };
                    // Use the member expression's loc (not the update expression's)
                    // to match TS behavior where the inner operations use leftExpr.node.loc
                    let member_loc = builder.source_location(update.argument.span());
                    let lowered =
                        lower_member_expression_from_simple_target(builder, &update.argument)?;
                    let object = lowered.object;
                    let lowered_property = lowered.property;
                    let prev_value = lower_value_to_temporary(builder, lowered.value)?;

                    let one = lower_value_to_temporary(
                        builder,
                        InstructionValue::Primitive {
                            value: PrimitiveValue::Number(FloatValue::new(1.0)),
                            loc: None,
                        },
                    )?;
                    let updated = lower_value_to_temporary(
                        builder,
                        InstructionValue::BinaryExpression {
                            operator: binary_op,
                            left: prev_value.clone(),
                            right: one,
                            loc: member_loc.clone(),
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
                                value: updated.clone(),
                                loc: member_loc,
                            },
                        )?,
                        MemberProperty::Computed(prop_place) => lower_value_to_temporary(
                            builder,
                            InstructionValue::ComputedStore {
                                object,
                                property: prop_place,
                                value: updated.clone(),
                                loc: member_loc,
                            },
                        )?,
                    };

                    // Return previous for postfix, newValuePlace for prefix
                    let result_place = if update.prefix { new_value_place } else { prev_value };
                    Ok(InstructionValue::LoadLocal {
                        place: result_place.clone(),
                        loc: result_place.loc.clone(),
                    })
                }
                oxc::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    let start = ident.span.start;
                    if builder.is_context_identifier(ident.name.as_str(), start, Some(start)) {
                        builder.record_error(CompilerErrorDetail {
                            category: ErrorCategory::Todo,
                            reason: "(BuildHIR::lowerExpression) Handle UpdateExpression to variables captured within lambdas.".to_string(),
                            description: None,
                            loc: loc.clone(),
                            suggestions: None,
                        })?;
                        return Ok(InstructionValue::Primitive {
                            value: PrimitiveValue::Undefined,
                            loc,
                        });
                    }

                    let ident_loc = builder.source_location(ident.span);
                    let binding = builder.resolve_identifier(
                        ident.name.as_str(),
                        start,
                        ident_loc.clone(),
                        Some(start),
                    )?;
                    if matches!(binding, VariableBinding::Global { .. }) {
                        builder.record_error(CompilerErrorDetail {
                            category: ErrorCategory::Todo,
                            reason:
                                "UpdateExpression where argument is a global is not yet supported"
                                    .to_string(),
                            description: None,
                            loc: loc.clone(),
                            suggestions: None,
                        })?;
                        return Ok(InstructionValue::Primitive {
                            value: PrimitiveValue::Undefined,
                            loc,
                        });
                    }
                    let identifier = match binding {
                        VariableBinding::Identifier { identifier, .. } => identifier,
                        _ => {
                            builder.record_error(CompilerErrorDetail {
                                category: ErrorCategory::Todo,
                                reason: "(BuildHIR::lowerExpression) Support UpdateExpression where argument is a global".to_string(),
                                description: None,
                                loc: loc.clone(),
                                suggestions: None,
                            })?;
                            return Ok(InstructionValue::Primitive {
                                value: PrimitiveValue::Undefined,
                                loc,
                            });
                        }
                    };
                    let lvalue_place = Place {
                        identifier,
                        effect: Effect::Unknown,
                        reactive: false,
                        loc: ident_loc.clone(),
                    };

                    // Load the current value
                    let value = lower_identifier(
                        builder,
                        ident.name.as_str(),
                        start,
                        ident_loc,
                        Some(start),
                    )?;

                    let operation = match update.operator {
                        oxc::UpdateOperator::Increment => UpdateOperator::Increment,
                        oxc::UpdateOperator::Decrement => UpdateOperator::Decrement,
                    };

                    if update.prefix {
                        Ok(InstructionValue::PrefixUpdate {
                            lvalue: lvalue_place,
                            operation,
                            value,
                            loc,
                        })
                    } else {
                        Ok(InstructionValue::PostfixUpdate {
                            lvalue: lvalue_place,
                            operation,
                            value,
                            loc,
                        })
                    }
                }
                _ => {
                    builder.record_error(CompilerErrorDetail {
                        category: ErrorCategory::Todo,
                        reason: "UpdateExpression with unsupported argument type".to_string(),
                        description: None,
                        loc: loc.clone(),
                        suggestions: None,
                    })?;
                    Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
                }
            }
        }
        // `x as T` / `x satisfies T` / `<T>x` lower the inner expression to a
        // temporary and emit a `TypeCastExpression` carrying the type metadata,
        // mirroring the original Babel logic.
        oxc::Expression::TSAsExpression(ts) => {
            lower_type_cast_expression(builder, ts.span, &ts.expression, &ts.type_annotation, "as")
        }
        oxc::Expression::TSSatisfiesExpression(ts) => lower_type_cast_expression(
            builder,
            ts.span,
            &ts.expression,
            &ts.type_annotation,
            "satisfies",
        ),
        oxc::Expression::TSTypeAssertion(ts) => {
            lower_type_cast_expression(builder, ts.span, &ts.expression, &ts.type_annotation, "as")
        }
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
            let loc = builder.source_location(obj.span);
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
            Ok(InstructionValue::ObjectExpression { properties, loc })
        }
        oxc::Expression::ArrayExpression(arr) => {
            let loc = builder.source_location(arr.span);
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
            Ok(InstructionValue::ArrayExpression { elements, loc })
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
            let loc = builder.source_location(expr.span());
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
        }
    }
}

/// Lower an `AssignmentExpression`. Faithful translation of the original
/// `Expression::AssignmentExpression` arm, adapted to oxc's `AssignmentTarget`
/// split. `=` handles identifier / member / destructuring targets; compound
/// operators (`+=` etc.) handle identifier / member targets and bail on patterns.
fn lower_assignment_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    assign: &'a oxc::AssignmentExpression<'a>,
) -> Result<InstructionValue<'a>, CompilerError> {
    let loc = builder.source_location(assign.span);

    if matches!(assign.operator, oxc::AssignmentOperator::Assign) {
        match &assign.left {
            oxc::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let start = ident.span.start;
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let ident_loc = builder.source_location(ident.span);
                let binding = builder.resolve_identifier(
                    ident.name.as_str(),
                    start,
                    ident_loc.clone(),
                    Some(start),
                )?;
                match binding {
                    VariableBinding::Identifier { identifier, binding_kind } => {
                        if binding_kind == BindingKind::Const {
                            builder.record_error(CompilerErrorDetail {
                                reason: "Cannot reassign a `const` variable".to_string(),
                                category: ErrorCategory::Syntax,
                                loc: ident_loc.clone(),
                                description: Some(format!(
                                    "`{}` is declared as const",
                                    ident.name.as_str()
                                )),
                                suggestions: None,
                            })?;
                            return Ok(InstructionValue::Primitive {
                                value: PrimitiveValue::Undefined,
                                loc: ident_loc,
                            });
                        }
                        let place = Place {
                            identifier,
                            reactive: false,
                            effect: Effect::Unknown,
                            loc: ident_loc,
                        };
                        if builder.is_context_identifier(ident.name.as_str(), start, Some(start)) {
                            let temp = lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreContext {
                                    lvalue: LValue {
                                        kind: InstructionKind::Reassign,
                                        place: place.clone(),
                                    },
                                    value: right,
                                    loc: place.loc.clone(),
                                },
                            )?;
                            Ok(InstructionValue::LoadLocal { place: temp.clone(), loc: temp.loc })
                        } else {
                            let temp = lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreLocal {
                                    lvalue: LValue {
                                        kind: InstructionKind::Reassign,
                                        place: place.clone(),
                                    },
                                    value: right,
                                    type_annotation: None,
                                    loc: place.loc.clone(),
                                },
                            )?;
                            Ok(InstructionValue::LoadLocal { place: temp.clone(), loc: temp.loc })
                        }
                    }
                    _ => {
                        let name = ident.name.to_string();
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreGlobal { name, value: right, loc: ident_loc },
                        )?;
                        Ok(InstructionValue::LoadLocal { place: temp.clone(), loc: temp.loc })
                    }
                }
            }
            oxc::AssignmentTarget::StaticMemberExpression(_)
            | oxc::AssignmentTarget::ComputedMemberExpression(_)
            | oxc::AssignmentTarget::PrivateFieldExpression(_) => {
                let simple = assign.left.as_simple_assignment_target().unwrap();
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let left_loc = builder.source_location(simple.span());
                let temp = match simple {
                    oxc::SimpleAssignmentTarget::StaticMemberExpression(member) => {
                        let object = lower_expression_to_temporary(builder, &member.object)?;
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::PropertyStore {
                                object,
                                property: PropertyLiteral::String(member.property.name.to_string()),
                                value: right,
                                loc: left_loc,
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
                                    loc: left_loc,
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
                                    loc: left_loc,
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
                                loc: left_loc,
                            },
                        )?
                    }
                    _ => unreachable!(),
                };
                Ok(InstructionValue::LoadLocal { place: temp.clone(), loc: temp.loc })
            }
            _ => {
                // Destructuring assignment
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let left_loc = builder.source_location(assign.left.span());
                let result = lower_assignment_target(
                    builder,
                    left_loc,
                    InstructionKind::Reassign,
                    &assign.left,
                    right.clone(),
                    AssignmentStyle::Destructure,
                )?;
                match result {
                    Some(place) => {
                        Ok(InstructionValue::LoadLocal { place: place.clone(), loc: place.loc })
                    }
                    None => Ok(InstructionValue::LoadLocal { place: right, loc }),
                }
            }
        }
    } else {
        // Compound assignment operators
        let binary_op = match assign.operator {
            oxc::AssignmentOperator::Addition => Some(BinaryOperator::Add),
            oxc::AssignmentOperator::Subtraction => Some(BinaryOperator::Subtract),
            oxc::AssignmentOperator::Multiplication => Some(BinaryOperator::Multiply),
            oxc::AssignmentOperator::Division => Some(BinaryOperator::Divide),
            oxc::AssignmentOperator::Remainder => Some(BinaryOperator::Modulo),
            oxc::AssignmentOperator::Exponential => Some(BinaryOperator::Exponent),
            oxc::AssignmentOperator::ShiftLeft => Some(BinaryOperator::ShiftLeft),
            oxc::AssignmentOperator::ShiftRight => Some(BinaryOperator::ShiftRight),
            oxc::AssignmentOperator::ShiftRightZeroFill => Some(BinaryOperator::UnsignedShiftRight),
            oxc::AssignmentOperator::BitwiseOR => Some(BinaryOperator::BitwiseOr),
            oxc::AssignmentOperator::BitwiseXOR => Some(BinaryOperator::BitwiseXor),
            oxc::AssignmentOperator::BitwiseAnd => Some(BinaryOperator::BitwiseAnd),
            oxc::AssignmentOperator::LogicalOr
            | oxc::AssignmentOperator::LogicalAnd
            | oxc::AssignmentOperator::LogicalNullish => {
                builder.record_error(CompilerErrorDetail {
                    reason: "Logical assignment operators (||=, &&=, ??=) are not yet supported"
                        .to_string(),
                    category: ErrorCategory::Todo,
                    loc: loc.clone(),
                    description: None,
                    suggestions: None,
                })?;
                return Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc });
            }
            oxc::AssignmentOperator::Assign => unreachable!(),
        };
        let binary_op = match binary_op {
            Some(op) => op,
            None => {
                return Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc });
            }
        };

        match &assign.left {
            oxc::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let start = ident.span.start;
                let ident_loc = builder.source_location(ident.span);
                let left_place = lower_identifier(
                    builder,
                    ident.name.as_str(),
                    start,
                    ident_loc.clone(),
                    Some(start),
                )?;
                let right = lower_expression_to_temporary(builder, &assign.right)?;
                let binary_place = lower_value_to_temporary(
                    builder,
                    InstructionValue::BinaryExpression {
                        operator: binary_op,
                        left: left_place,
                        right,
                        loc: loc.clone(),
                    },
                )?;
                let binding = builder.resolve_identifier(
                    ident.name.as_str(),
                    start,
                    ident_loc.clone(),
                    Some(start),
                )?;
                match binding {
                    VariableBinding::Identifier { identifier, .. } => {
                        let place = Place {
                            identifier,
                            reactive: false,
                            effect: Effect::Unknown,
                            loc: ident_loc,
                        };
                        if builder.is_context_identifier(ident.name.as_str(), start, Some(start)) {
                            lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreContext {
                                    lvalue: LValue {
                                        kind: InstructionKind::Reassign,
                                        place: place.clone(),
                                    },
                                    value: binary_place,
                                    loc: loc.clone(),
                                },
                            )?;
                            Ok(InstructionValue::LoadContext { place, loc })
                        } else {
                            lower_value_to_temporary(
                                builder,
                                InstructionValue::StoreLocal {
                                    lvalue: LValue {
                                        kind: InstructionKind::Reassign,
                                        place: place.clone(),
                                    },
                                    value: binary_place,
                                    type_annotation: None,
                                    loc: loc.clone(),
                                },
                            )?;
                            Ok(InstructionValue::LoadLocal { place, loc })
                        }
                    }
                    _ => {
                        let name = ident.name.to_string();
                        let temp = lower_value_to_temporary(
                            builder,
                            InstructionValue::StoreGlobal {
                                name,
                                value: binary_place,
                                loc: loc.clone(),
                            },
                        )?;
                        Ok(InstructionValue::LoadLocal { place: temp.clone(), loc: temp.loc })
                    }
                }
            }
            oxc::AssignmentTarget::StaticMemberExpression(_)
            | oxc::AssignmentTarget::ComputedMemberExpression(_)
            | oxc::AssignmentTarget::PrivateFieldExpression(_) => {
                let simple = assign.left.as_simple_assignment_target().unwrap();
                let member_loc = builder.source_location(simple.span());
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
                        loc: member_loc.clone(),
                    },
                )?;
                match lowered_property {
                    MemberProperty::Literal(prop_literal) => Ok(InstructionValue::PropertyStore {
                        object,
                        property: prop_literal,
                        value: result,
                        loc: member_loc,
                    }),
                    MemberProperty::Computed(prop_place) => Ok(InstructionValue::ComputedStore {
                        object,
                        property: prop_place,
                        value: result,
                        loc: member_loc,
                    }),
                }
            }
            _ => {
                builder.record_error(CompilerErrorDetail {
                    reason: "Compound assignment to complex pattern is not yet supported"
                        .to_string(),
                    category: ErrorCategory::Todo,
                    loc: loc.clone(),
                    description: None,
                    suggestions: None,
                })?;
                Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
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
    jsx_element: &'a oxc::JSXElement<'a>,
) -> Result<InstructionValue<'a>, CompilerError> {
    let loc = builder.source_location(jsx_element.span);
    let opening_loc = builder.source_location(jsx_element.opening_element.span);
    let closing_loc =
        jsx_element.closing_element.as_ref().and_then(|c| builder.source_location(c.span));

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
                            builder.record_error(CompilerErrorDetail {
                                category: ErrorCategory::Todo,
                                reason: format!(
                                    "(BuildHIR::lowerExpression) Unexpected colon in attribute name `{}`",
                                    name
                                ),
                                description: None,
                                loc: builder.source_location(id.span),
                                suggestions: None,
                            })?;
                        }
                        name.to_string()
                    }
                    oxc::JSXAttributeName::NamespacedName(ns) => {
                        format!("{}:{}", ns.namespace.name, ns.name.name)
                    }
                };

                // Get the attribute value
                let value = match &attr.value {
                    Some(oxc::JSXAttributeValue::StringLiteral(s)) => {
                        let str_loc = builder.source_location(s.span);
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::Primitive {
                                value: PrimitiveValue::String(
                                    decode_jsx_entities(s.value.as_str()).into(),
                                ),
                                loc: str_loc,
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
                        let attr_loc = builder.source_location(attr.span);
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::Primitive {
                                value: PrimitiveValue::Boolean(true),
                                loc: attr_loc,
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
    // Matches TS: CompilerError.invariant(tagIdentifier.kind !== 'Identifier', ...)
    if is_fbt {
        let tag_name = match &tag {
            JsxTag::Builtin(b) => b.name.clone(),
            _ => "fbt".to_string(),
        };
        // Get the opening element's name identifier and check if it's a local binding.
        let jsx_id_name = match &jsx_element.opening_element.name {
            oxc::JSXElementName::Identifier(id) => Some((id.name.as_str(), id.span)),
            oxc::JSXElementName::IdentifierReference(id) => Some((id.name.as_str(), id.span)),
            _ => None,
        };
        if let Some((name, span)) = jsx_id_name {
            let id_loc = builder.source_location(span);
            // Check if fbt/fbs tag name resolves to a local binding.
            // JSX identifiers may not be in our position-based reference map,
            // so check if ANY binding with this name exists in the function scope.
            let is_local_binding = builder.has_local_binding(name);
            if is_local_binding {
                // Record as a Diagnostic (not ErrorDetail) to match TS behavior
                // where CompilerError.invariant creates a CompilerDiagnostic.
                let reason = format!("<{}> tags should be module-level imports", tag_name);
                return Err(CompilerDiagnostic::new(ErrorCategory::Invariant, &reason, None)
                    .with_detail(CompilerDiagnosticDetail::Error {
                        loc: id_loc.clone(),
                        message: Some(reason.clone()),
                        identifier_name: None,
                    })
                    .into());
            }
        }
    }

    // Check for duplicate fbt:enum, fbt:plural, fbt:pronoun tags.
    if is_fbt {
        let tag_name = match &tag {
            JsxTag::Builtin(b) => b.name.as_str(),
            _ => "fbt",
        };
        let mut enum_locs: Vec<Option<SourceLocation>> = Vec::new();
        let mut plural_locs: Vec<Option<SourceLocation>> = Vec::new();
        let mut pronoun_locs: Vec<Option<SourceLocation>> = Vec::new();
        collect_fbt_sub_tags(
            builder,
            &jsx_element.children,
            tag_name,
            &mut enum_locs,
            &mut plural_locs,
            &mut pronoun_locs,
        );

        for (name, locations) in
            [("enum", &enum_locs), ("plural", &plural_locs), ("pronoun", &pronoun_locs)]
        {
            if locations.len() > 1 {
                let details: Vec<CompilerDiagnosticDetail> = locations
                    .iter()
                    .map(|loc| CompilerDiagnosticDetail::Error {
                        message: Some(format!("Multiple `<{}:{}>` tags found", tag_name, name)),
                        loc: loc.clone(),
                        identifier_name: None,
                    })
                    .collect();
                let mut diag = CompilerDiagnostic::new(
                    ErrorCategory::Todo,
                    "Support duplicate fbt tags",
                    Some(format!(
                        "Support `<{}>` tags with multiple `<{}:{}>` values",
                        tag_name, tag_name, name
                    )),
                );
                diag.details = details;
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
        loc,
        opening_loc,
        closing_loc,
    })
}

/// Lower a JSX fragment expression. Faithful translation of the original
/// `Expression::JSXFragment` arm.
fn lower_jsx_fragment_expr<'a>(
    builder: &mut HirBuilder<'a, '_>,
    jsx_fragment: &'a oxc::JSXFragment<'a>,
) -> Result<InstructionValue<'a>, CompilerError> {
    let loc = builder.source_location(jsx_fragment.span);

    // Lower children
    let children: Vec<Place> = jsx_fragment
        .children
        .iter()
        .map(|child| lower_jsx_element(builder, child))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(InstructionValue::JsxFragment { children, loc })
}

/// Lower a JSX element name into a `JsxTag`. Faithful translation of the original
/// `lower_jsx_element_name`, adapted to oxc's `JSXElementName` shape (which splits
/// out `IdentifierReference`, `MemberExpression`, and `ThisExpression`; the latter
/// maps to the identifier `"this"`).
fn lower_jsx_element_name(
    builder: &mut HirBuilder<'_, '_>,
    name: &oxc::JSXElementName,
) -> Result<JsxTag, CompilerError> {
    // Lower a simple JSX tag identifier (component-vs-builtin split on case).
    fn lower_tag_identifier(
        builder: &mut HirBuilder<'_, '_>,
        tag: &str,
        span: oxc_span::Span,
    ) -> Result<JsxTag, CompilerError> {
        let loc = builder.source_location(span);
        let start = span.start;
        if tag.starts_with(|c: char| c.is_ascii_uppercase()) {
            // Component tag: resolve as identifier and load
            let place = lower_identifier(builder, tag, start, loc.clone(), Some(start))?;
            let load_value = if builder.is_context_identifier(tag, start, Some(start)) {
                InstructionValue::LoadContext { place, loc }
            } else {
                InstructionValue::LoadLocal { place, loc }
            };
            let temp = lower_value_to_temporary(builder, load_value)?;
            Ok(JsxTag::Place(temp))
        } else {
            // Builtin HTML tag
            Ok(JsxTag::Builtin(BuiltinTag { name: tag.to_string(), loc }))
        }
    }

    match name {
        oxc::JSXElementName::Identifier(id) => {
            lower_tag_identifier(builder, id.name.as_str(), id.span)
        }
        oxc::JSXElementName::IdentifierReference(id) => {
            lower_tag_identifier(builder, id.name.as_str(), id.span)
        }
        oxc::JSXElementName::ThisExpression(this) => {
            // `<this.Foo />`-style `this` tag lowers as the identifier "this".
            lower_tag_identifier(builder, "this", this.span)
        }
        oxc::JSXElementName::MemberExpression(member) => {
            let place = lower_jsx_member_expression(builder, member)?;
            Ok(JsxTag::Place(place))
        }
        oxc::JSXElementName::NamespacedName(ns) => {
            let namespace = ns.namespace.name.as_str();
            let name = ns.name.name.as_str();
            let tag = format!("{}:{}", namespace, name);
            let loc = builder.source_location(ns.span);
            if namespace.contains(':') || name.contains(':') {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Syntax,
                    reason: "Expected JSXNamespacedName to have no colons in the namespace or name"
                        .to_string(),
                    description: Some(format!("Got `{}` : `{}`", namespace, name)),
                    loc: loc.clone(),
                    suggestions: None,
                })?;
            }
            let place = lower_value_to_temporary(
                builder,
                InstructionValue::Primitive {
                    value: PrimitiveValue::String(tag.into()),
                    loc: loc.clone(),
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
fn lower_jsx_member_expression(
    builder: &mut HirBuilder<'_, '_>,
    expr: &oxc::JSXMemberExpression,
) -> Result<Place, CompilerError> {
    // Use the full member expression's loc for instruction locs (matching TS: exprPath.node.loc)
    let expr_loc = builder.source_location(expr.span);
    let object = match &expr.object {
        oxc::JSXMemberExpressionObject::IdentifierReference(id) => {
            lower_jsx_member_object_identifier(builder, id.name.as_str(), id.span, &expr_loc)?
        }
        oxc::JSXMemberExpressionObject::ThisExpression(this) => {
            lower_jsx_member_object_identifier(builder, "this", this.span, &expr_loc)?
        }
        oxc::JSXMemberExpressionObject::MemberExpression(inner) => {
            lower_jsx_member_expression(builder, inner)?
        }
    };
    let prop_name = expr.property.name.as_str();
    let value = InstructionValue::PropertyLoad {
        object,
        property: PropertyLiteral::String(prop_name.to_string()),
        loc: expr_loc,
    };
    lower_value_to_temporary(builder, value)
}

/// Lower the leaf identifier of a JSX member expression object. Uses the
/// identifier's own loc for the place, but the enclosing member expression's loc
/// for the load instruction (matching TS).
fn lower_jsx_member_object_identifier(
    builder: &mut HirBuilder<'_, '_>,
    name: &str,
    span: oxc_span::Span,
    expr_loc: &Option<SourceLocation>,
) -> Result<Place, CompilerError> {
    let id_loc = builder.source_location(span);
    let start = span.start;
    let place = lower_identifier(builder, name, start, id_loc, Some(start))?;
    let load_value = if builder.is_context_identifier(name, start, Some(start)) {
        InstructionValue::LoadContext { place, loc: expr_loc.clone() }
    } else {
        InstructionValue::LoadLocal { place, loc: expr_loc.clone() }
    };
    lower_value_to_temporary(builder, load_value)
}

/// Lower a single JSX child into an optional `Place`. Faithful translation of the
/// original `lower_jsx_element` (the JSXChild handler), adapted to oxc's `JSXChild`.
fn lower_jsx_element<'a>(
    builder: &mut HirBuilder<'a, '_>,
    child: &'a oxc::JSXChild<'a>,
) -> Result<Option<Place>, CompilerError> {
    match child {
        oxc::JSXChild::Text(text) => {
            // oxc keeps JSX text raw; decode entities first so the value matches
            // Babel's `JSXText.value` (the Babel bridge decoded in convert_ast).
            let decoded = decode_jsx_entities(text.value.as_str());
            // FBT whitespace normalization differs from standard JSX.
            // Since the fbt transform runs after, preserve all whitespace
            // in FBT subtrees as is.
            let value = if builder.fbt_depth > 0 { Some(decoded) } else { trim_jsx_text(&decoded) };
            match value {
                None => Ok(None),
                Some(value) => {
                    let loc = builder.source_location(text.span);
                    let place = lower_value_to_temporary(
                        builder,
                        InstructionValue::JSXText { value, loc },
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
    enum_locs: &mut Vec<Option<SourceLocation>>,
    plural_locs: &mut Vec<Option<SourceLocation>>,
    pronoun_locs: &mut Vec<Option<SourceLocation>>,
) {
    for child in children {
        match child {
            oxc::JSXChild::Element(el) => {
                collect_fbt_sub_tags_from_element(
                    builder,
                    el,
                    tag_name,
                    enum_locs,
                    plural_locs,
                    pronoun_locs,
                );
            }
            oxc::JSXChild::Fragment(frag) => {
                collect_fbt_sub_tags(
                    builder,
                    &frag.children,
                    tag_name,
                    enum_locs,
                    plural_locs,
                    pronoun_locs,
                );
            }
            oxc::JSXChild::ExpressionContainer(container) => {
                if let Some(expr) = container.expression.as_expression() {
                    collect_fbt_sub_tags_from_expr(
                        builder,
                        expr,
                        tag_name,
                        enum_locs,
                        plural_locs,
                        pronoun_locs,
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
    enum_locs: &mut Vec<Option<SourceLocation>>,
    plural_locs: &mut Vec<Option<SourceLocation>>,
    pronoun_locs: &mut Vec<Option<SourceLocation>>,
) {
    if let oxc::JSXElementName::NamespacedName(ns) = &el.opening_element.name {
        if ns.namespace.name == tag_name {
            let loc = builder.source_location(ns.span);
            match ns.name.name.as_str() {
                "enum" => enum_locs.push(loc),
                "plural" => plural_locs.push(loc),
                "pronoun" => pronoun_locs.push(loc),
                _ => {}
            }
        }
    }
    collect_fbt_sub_tags(builder, &el.children, tag_name, enum_locs, plural_locs, pronoun_locs);
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
                            enum_locs,
                            plural_locs,
                            pronoun_locs,
                        );
                    }
                }
                Some(oxc::JSXAttributeValue::Element(nested)) => {
                    collect_fbt_sub_tags_from_element(
                        builder,
                        nested,
                        tag_name,
                        enum_locs,
                        plural_locs,
                        pronoun_locs,
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
    enum_locs: &mut Vec<Option<SourceLocation>>,
    plural_locs: &mut Vec<Option<SourceLocation>>,
    pronoun_locs: &mut Vec<Option<SourceLocation>>,
) {
    match expr {
        oxc::Expression::JSXElement(el) => {
            collect_fbt_sub_tags_from_element(
                builder,
                el,
                tag_name,
                enum_locs,
                plural_locs,
                pronoun_locs,
            );
        }
        oxc::Expression::JSXFragment(frag) => {
            collect_fbt_sub_tags(
                builder,
                &frag.children,
                tag_name,
                enum_locs,
                plural_locs,
                pronoun_locs,
            );
        }
        oxc::Expression::ConditionalExpression(cond) => {
            collect_fbt_sub_tags_from_expr(
                builder,
                &cond.consequent,
                tag_name,
                enum_locs,
                plural_locs,
                pronoun_locs,
            );
            collect_fbt_sub_tags_from_expr(
                builder,
                &cond.alternate,
                tag_name,
                enum_locs,
                plural_locs,
                pronoun_locs,
            );
        }
        oxc::Expression::LogicalExpression(log) => {
            collect_fbt_sub_tags_from_expr(
                builder,
                &log.left,
                tag_name,
                enum_locs,
                plural_locs,
                pronoun_locs,
            );
            collect_fbt_sub_tags_from_expr(
                builder,
                &log.right,
                tag_name,
                enum_locs,
                plural_locs,
                pronoun_locs,
            );
        }
        oxc::Expression::ParenthesizedExpression(paren) => {
            collect_fbt_sub_tags_from_expr(
                builder,
                &paren.expression,
                tag_name,
                enum_locs,
                plural_locs,
                pronoun_locs,
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
                        enum_locs,
                        plural_locs,
                        pronoun_locs,
                    );
                }
            } else {
                collect_fbt_sub_tags_from_stmts(
                    builder,
                    &arrow.body.statements,
                    tag_name,
                    enum_locs,
                    plural_locs,
                    pronoun_locs,
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
                        enum_locs,
                        plural_locs,
                        pronoun_locs,
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
    enum_locs: &mut Vec<Option<SourceLocation>>,
    plural_locs: &mut Vec<Option<SourceLocation>>,
    pronoun_locs: &mut Vec<Option<SourceLocation>>,
) {
    for stmt in stmts {
        match stmt {
            oxc::Statement::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    collect_fbt_sub_tags_from_expr(
                        builder,
                        arg,
                        tag_name,
                        enum_locs,
                        plural_locs,
                        pronoun_locs,
                    );
                }
            }
            oxc::Statement::ExpressionStatement(expr_stmt) => {
                collect_fbt_sub_tags_from_expr(
                    builder,
                    &expr_stmt.expression,
                    tag_name,
                    enum_locs,
                    plural_locs,
                    pronoun_locs,
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
        let mut trimmed_line = line.replace('\t', " ");

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
fn decode_jsx_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
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
    out
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
    method: &'a oxc::ObjectProperty<'a>,
) -> Result<Option<ObjectProperty>, CompilerError> {
    // In oxc, a shorthand method is encoded as `kind: Init, method: true`; only
    // getters/setters carry a non-`Init` `PropertyKind`.
    let is_method = method.method && matches!(method.kind, oxc::PropertyKind::Init);
    if !is_method {
        let kind_str = match method.kind {
            oxc::PropertyKind::Get => "get",
            oxc::PropertyKind::Set => "set",
            oxc::PropertyKind::Init => "method",
        };
        builder.record_error(CompilerErrorDetail {
            reason: format!(
                "(BuildHIR::lowerExpression) Handle {} functions in ObjectExpression",
                kind_str
            ),
            category: ErrorCategory::Todo,
            loc: builder.source_location(method.span),
            description: None,
            suggestions: None,
        })?;
        return Ok(None);
    }

    let key = lower_object_property_key(builder, &method.key, method.computed)?
        .unwrap_or(ObjectPropertyKey::String { name: String::new() });

    let func = match &method.value {
        oxc::Expression::FunctionExpression(func) => func,
        _ => unreachable!("object method value is always a FunctionExpression in oxc"),
    };
    let body = func.body.as_ref().expect("object method always has a body");
    let lowered_func = lower_function_for_object_method(
        builder,
        method.span,
        &func.params,
        body,
        func.generator,
        func.r#async,
    )?;

    let loc = builder.source_location(method.span);
    let method_value = InstructionValue::ObjectMethod { loc, lowered_func };
    let method_place = lower_value_to_temporary(builder, method_value)?;

    Ok(Some(ObjectProperty { key, property_type: ObjectPropertyType::Method, place: method_place }))
}

/// Lower an object property key. Faithful to the original `lower_object_property_key`.
fn lower_object_property_key<'a>(
    builder: &mut HirBuilder<'a, '_>,
    key: &'a oxc::PropertyKey<'a>,
    computed: bool,
) -> Result<Option<ObjectPropertyKey>, CompilerError> {
    match key {
        // Property keys stay String-typed; oxc atoms are valid UTF-8, so
        // `to_string()` reproduces the marker wire form for non-pathological keys.
        oxc::PropertyKey::StringLiteral(lit) => {
            Ok(Some(ObjectPropertyKey::String { name: lit.value.to_string() }))
        }
        oxc::PropertyKey::StaticIdentifier(ident) if !computed => {
            Ok(Some(ObjectPropertyKey::Identifier { name: ident.name.to_string() }))
        }
        oxc::PropertyKey::Identifier(ident) if !computed => {
            Ok(Some(ObjectPropertyKey::Identifier { name: ident.name.to_string() }))
        }
        oxc::PropertyKey::NumericLiteral(lit) if !computed => {
            Ok(Some(ObjectPropertyKey::Identifier { name: lit.value.to_string() }))
        }
        _ if computed => {
            let place = lower_expression_to_temporary(builder, key.to_expression())?;
            Ok(Some(ObjectPropertyKey::Computed { name: place }))
        }
        _ => {
            let loc = match key {
                oxc::PropertyKey::StaticIdentifier(i) => builder.source_location(i.span),
                oxc::PropertyKey::Identifier(i) => builder.source_location(i.span),
                _ => None,
            };
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "Unsupported key type in ObjectExpression".to_string(),
                description: None,
                loc,
                suggestions: None,
            })?;
            Ok(None)
        }
    }
}

/// Lower a reorderable expression. Faithful to the original
/// `lower_reorderable_expression`: records an error when the expression cannot be
/// safely reordered, then lowers it to a temporary regardless.
fn lower_reorderable_expression<'a>(
    builder: &mut HirBuilder<'a, '_>,
    expr: &'a oxc::Expression<'a>,
) -> Result<Place, CompilerError> {
    if !is_reorderable_expression(builder, expr, true) {
        builder.record_error(CompilerErrorDetail {
            category: ErrorCategory::Todo,
            reason: format!(
                "(BuildHIR::node.lowerReorderableExpression) Expression type `{}` cannot be safely reordered",
                expression_type_name(expr)
            ),
            description: None,
            loc: builder.source_location(expr.span()),
            suggestions: None,
        })?;
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
            let binding = builder.scope_info().resolve_reference_for_node(Some(ident.span.start));
            match binding {
                None => {
                    // global, safe to reorder
                    true
                }
                Some(b) => {
                    if b.scope == builder.scope_info().program_scope {
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
                match builder.scope_info().resolve_reference_for_node(Some(ident.span.start)) {
                    None => true, // global
                    Some(binding) => {
                        // Module-scope bindings (ModuleLocal, imports) are safe to reorder
                        binding.scope == builder.scope_info().program_scope
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
    stmt: &'a oxc::Statement<'a>,
    _label: Option<&str>,
    parent_scope: Option<crate::scope::ScopeId>,
) -> Result<(), CompilerDiagnostic> {
    match stmt {
        oxc::Statement::EmptyStatement(_) => {}
        oxc::Statement::DebuggerStatement(dbg) => {
            let loc = builder.source_location(dbg.span);
            lower_value_to_temporary(builder, InstructionValue::Debugger { loc })?;
        }
        oxc::Statement::ExpressionStatement(expr_stmt) => {
            lower_expression_to_temporary(builder, &expr_stmt.expression)?;
        }
        oxc::Statement::ReturnStatement(ret) => {
            let loc = builder.source_location(ret.span);
            let value = if let Some(arg) = &ret.argument {
                lower_expression_to_temporary(builder, arg)?
            } else {
                lower_value_to_temporary(
                    builder,
                    InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc: None },
                )?
            };
            let fallthrough = builder.reserve(BlockKind::Block);
            builder.terminate_with_continuation(
                Terminal::Return {
                    value,
                    return_variant: ReturnVariant::Explicit,
                    id: EvaluationOrder(0),
                    loc,
                    effects: None,
                },
                fallthrough,
            );
        }
        oxc::Statement::ThrowStatement(throw) => {
            let loc = builder.source_location(throw.span);
            let value = lower_expression_to_temporary(builder, &throw.argument)?;
            if builder.resolve_throw_handler().is_some() {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Todo,
                    reason: "(BuildHIR::lowerStatement) Support ThrowStatement inside of try/catch"
                        .to_string(),
                    description: None,
                    loc: loc.clone(),
                    suggestions: None,
                })?;
            }
            let fallthrough = builder.reserve(BlockKind::Block);
            builder.terminate_with_continuation(
                Terminal::Throw { value, id: EvaluationOrder(0), loc },
                fallthrough,
            );
        }
        oxc::Statement::BlockStatement(block) => {
            lower_block_statement(builder, &block.body, block.span.start, parent_scope)?;
        }
        oxc::Statement::VariableDeclaration(var_decl) => {
            lower_variable_declaration(builder, var_decl)?;
        }
        oxc::Statement::FunctionDeclaration(func_decl) if func_decl.body.is_some() => {
            lower_function_declaration(builder, func_decl)?;
        }
        oxc::Statement::IfStatement(if_stmt) => {
            let loc = builder.source_location(if_stmt.span);
            // Block for code following the if
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Block for the consequent (if the test is truthy)
            let consequent_loc = builder.source_location(if_stmt.consequent.span());
            let consequent_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                lower_statement(builder, &if_stmt.consequent, None, parent_scope)?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: consequent_loc,
                })
            })?;

            // Block for the alternate (if the test is not truthy)
            let alternate_block = if let Some(alternate) = &if_stmt.alternate {
                let alternate_loc = builder.source_location(alternate.span());
                builder.try_enter(BlockKind::Block, |builder, _block_id| {
                    lower_statement(builder, alternate, None, parent_scope)?;
                    Ok(Terminal::Goto {
                        block: continuation_id,
                        variant: GotoVariant::Break,
                        id: EvaluationOrder(0),
                        loc: alternate_loc,
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
                    id: EvaluationOrder(0),
                    loc,
                },
                continuation_block,
            );
        }
        oxc::Statement::ForStatement(for_stmt) => {
            let loc = builder.source_location(for_stmt.span);

            let test_block = builder.reserve(BlockKind::Loop);
            let test_block_id = test_block.id;
            // Block for code following the loop
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Init block: lower init expression/declaration, then goto test
            let init_block = builder.try_enter(BlockKind::Loop, |builder, _block_id| {
                let init_loc = match &for_stmt.init {
                    None => {
                        // No init expression (e.g., `for (; ...)`), add a placeholder
                        let placeholder = InstructionValue::Primitive {
                            value: PrimitiveValue::Undefined,
                            loc: loc.clone(),
                        };
                        lower_value_to_temporary(builder, placeholder)?;
                        loc.clone()
                    }
                    Some(oxc::ForStatementInit::VariableDeclaration(var_decl)) => {
                        let init_loc = builder.source_location(var_decl.span);
                        lower_variable_declaration(builder, var_decl)?;
                        init_loc
                    }
                    Some(init) => {
                        let expr = init.to_expression();
                        let init_loc = builder.source_location(expr.span());
                        builder.record_error(CompilerErrorDetail {
                            category: ErrorCategory::Todo,
                            reason: "(BuildHIR::lowerStatement) Handle non-variable initialization in ForStatement".to_string(),
                            description: None,
                            loc: loc.clone(),
                            suggestions: None,
                        })?;
                        lower_expression_to_temporary(builder, expr)?;
                        init_loc
                    }
                };
                Ok(Terminal::Goto {
                    block: test_block_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: init_loc,
                })
            })?;

            // Update block (optional)
            let update_block_id = if let Some(update) = &for_stmt.update {
                let update_loc = builder.source_location(update.span());
                Some(builder.try_enter(BlockKind::Loop, |builder, _block_id| {
                    lower_expression_to_temporary(builder, update)?;
                    Ok(Terminal::Goto {
                        block: test_block_id,
                        variant: GotoVariant::Break,
                        id: EvaluationOrder(0),
                        loc: update_loc,
                    })
                })?)
            } else {
                None
            };

            // Loop body block
            let continue_target = update_block_id.unwrap_or(test_block_id);
            let body_loc = builder.source_location(for_stmt.body.span());
            let body_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(
                    _label.map(|s| s.to_string()),
                    continue_target,
                    continuation_id,
                    |builder| {
                        lower_statement(builder, &for_stmt.body, None, parent_scope)?;
                        Ok(Terminal::Goto {
                            block: continue_target,
                            variant: GotoVariant::Continue,
                            id: EvaluationOrder(0),
                            loc: body_loc,
                        })
                    },
                )
            })?;

            // Emit For terminal, then fill in the test block
            builder.terminate_with_continuation(
                Terminal::For {
                    init: init_block,
                    test: test_block_id,
                    update: update_block_id,
                    loop_block: body_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
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
                        id: EvaluationOrder(0),
                        loc: loc.clone(),
                    },
                    continuation_block,
                );
            } else {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Todo,
                    reason: "(BuildHIR::lowerStatement) Handle empty test in ForStatement"
                        .to_string(),
                    description: None,
                    loc: loc.clone(),
                    suggestions: None,
                })?;
                // Treat `for(;;)` as `while(true)` to keep the builder state consistent
                let true_val = InstructionValue::Primitive {
                    value: PrimitiveValue::Boolean(true),
                    loc: loc.clone(),
                };
                let test = lower_value_to_temporary(builder, true_val)?;
                builder.terminate_with_continuation(
                    Terminal::Branch {
                        test,
                        consequent: body_block,
                        alternate: continuation_id,
                        fallthrough: continuation_id,
                        id: EvaluationOrder(0),
                        loc,
                    },
                    continuation_block,
                );
            }
        }
        oxc::Statement::WhileStatement(while_stmt) => {
            let loc = builder.source_location(while_stmt.span);
            // Block used to evaluate whether to (re)enter or exit the loop
            let conditional_block = builder.reserve(BlockKind::Loop);
            let conditional_id = conditional_block.id;
            // Block for code following the loop
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Loop body
            let body_loc = builder.source_location(while_stmt.body.span());
            let loop_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(
                    _label.map(|s| s.to_string()),
                    conditional_id,
                    continuation_id,
                    |builder| {
                        lower_statement(builder, &while_stmt.body, None, parent_scope)?;
                        Ok(Terminal::Goto {
                            block: conditional_id,
                            variant: GotoVariant::Continue,
                            id: EvaluationOrder(0),
                            loc: body_loc,
                        })
                    },
                )
            })?;

            // Emit While terminal, jumping to the conditional block
            builder.terminate_with_continuation(
                Terminal::While {
                    test: conditional_id,
                    loop_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
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
                    id: EvaluationOrder(0),
                    loc,
                },
                continuation_block,
            );
        }
        oxc::Statement::DoWhileStatement(do_while_stmt) => {
            let loc = builder.source_location(do_while_stmt.span);
            // Block used to evaluate whether to (re)enter or exit the loop
            let conditional_block = builder.reserve(BlockKind::Loop);
            let conditional_id = conditional_block.id;
            // Block for code following the loop
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Loop body, executed at least once unconditionally prior to exit
            let body_loc = builder.source_location(do_while_stmt.body.span());
            let loop_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(
                    _label.map(|s| s.to_string()),
                    conditional_id,
                    continuation_id,
                    |builder| {
                        lower_statement(builder, &do_while_stmt.body, None, parent_scope)?;
                        Ok(Terminal::Goto {
                            block: conditional_id,
                            variant: GotoVariant::Continue,
                            id: EvaluationOrder(0),
                            loc: body_loc,
                        })
                    },
                )
            })?;

            // Jump to the conditional block
            builder.terminate_with_continuation(
                Terminal::DoWhile {
                    loop_block,
                    test: conditional_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
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
                    id: EvaluationOrder(0),
                    loc,
                },
                continuation_block,
            );
        }
        oxc::Statement::ForInStatement(for_in) => {
            let loc = builder.source_location(for_in.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;
            let init_block = builder.reserve(BlockKind::Loop);
            let init_block_id = init_block.id;

            let body_loc = builder.source_location(for_in.body.span());
            let loop_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(
                    _label.map(|s| s.to_string()),
                    init_block_id,
                    continuation_id,
                    |builder| {
                        lower_statement(builder, &for_in.body, None, parent_scope)?;
                        Ok(Terminal::Goto {
                            block: init_block_id,
                            variant: GotoVariant::Continue,
                            id: EvaluationOrder(0),
                            loc: body_loc,
                        })
                    },
                )
            })?;

            let value = lower_expression_to_temporary(builder, &for_in.right)?;
            builder.terminate_with_continuation(
                Terminal::ForIn {
                    init: init_block_id,
                    loop_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                init_block,
            );

            // Lower the init: NextPropertyOf + assignment
            let left_loc = builder.source_location(for_in.left.span());
            let next_property = lower_value_to_temporary(
                builder,
                InstructionValue::NextPropertyOf { value, loc: left_loc.clone() },
            )?;

            let assign_result = lower_for_in_of_left(
                builder,
                &for_in.left,
                left_loc.clone(),
                next_property.clone(),
            )?;
            // Use the assign result (StoreLocal temp) as the test, matching TS behavior
            let test_value = assign_result.unwrap_or(next_property);
            let test = lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal { place: test_value, loc: left_loc.clone() },
            )?;
            builder.terminate_with_continuation(
                Terminal::Branch {
                    test,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                continuation_block,
            );
        }
        oxc::Statement::ForOfStatement(for_of) => {
            let loc = builder.source_location(for_of.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;
            let init_block = builder.reserve(BlockKind::Loop);
            let init_block_id = init_block.id;
            let test_block = builder.reserve(BlockKind::Loop);
            let test_block_id = test_block.id;

            if for_of.r#await {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Todo,
                    reason: "(BuildHIR::lowerStatement) Handle for-await loops".to_string(),
                    description: None,
                    loc: loc.clone(),
                    suggestions: None,
                })?;
                return Ok(());
            }

            let body_loc = builder.source_location(for_of.body.span());
            let loop_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.loop_scope(
                    _label.map(|s| s.to_string()),
                    init_block_id,
                    continuation_id,
                    |builder| {
                        lower_statement(builder, &for_of.body, None, parent_scope)?;
                        Ok(Terminal::Goto {
                            block: init_block_id,
                            variant: GotoVariant::Continue,
                            id: EvaluationOrder(0),
                            loc: body_loc,
                        })
                    },
                )
            })?;

            let value = lower_expression_to_temporary(builder, &for_of.right)?;
            builder.terminate_with_continuation(
                Terminal::ForOf {
                    init: init_block_id,
                    test: test_block_id,
                    loop_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                init_block,
            );

            // Init block: GetIterator, goto test
            let iterator = lower_value_to_temporary(
                builder,
                InstructionValue::GetIterator { collection: value.clone(), loc: value.loc.clone() },
            )?;
            builder.terminate_with_continuation(
                Terminal::Goto {
                    block: test_block_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                test_block,
            );

            // Test block: IteratorNext, assign, branch
            let left_loc = builder.source_location(for_of.left.span());
            let advance_iterator = lower_value_to_temporary(
                builder,
                InstructionValue::IteratorNext {
                    iterator: iterator.clone(),
                    collection: value.clone(),
                    loc: left_loc.clone(),
                },
            )?;

            let assign_result = lower_for_in_of_left(
                builder,
                &for_of.left,
                left_loc.clone(),
                advance_iterator.clone(),
            )?;
            // Use the assign result (StoreLocal temp) as the test, matching TS behavior
            let test_value = assign_result.unwrap_or(advance_iterator);
            let test = lower_value_to_temporary(
                builder,
                InstructionValue::LoadLocal { place: test_value, loc: left_loc.clone() },
            )?;
            builder.terminate_with_continuation(
                Terminal::Branch {
                    test,
                    consequent: loop_block,
                    alternate: continuation_id,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc: loc.clone(),
                },
                continuation_block,
            );
        }
        oxc::Statement::SwitchStatement(switch_stmt) => {
            let loc = builder.source_location(switch_stmt.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            // Iterate through cases in reverse order so that previous blocks can
            // fallthrough to successors
            let mut fallthrough = continuation_id;
            let mut cases: Vec<Case> = Vec::new();
            let mut has_default = false;

            for ii in (0..switch_stmt.cases.len()).rev() {
                let case = &switch_stmt.cases[ii];
                let case_loc = builder.source_location(case.span);

                if case.test.is_none() {
                    if has_default {
                        builder.record_error(CompilerErrorDetail {
                            category: ErrorCategory::Syntax,
                            reason: "Expected at most one `default` branch in a switch statement"
                                .to_string(),
                            description: None,
                            loc: case_loc.clone(),
                            suggestions: None,
                        })?;
                        break;
                    }
                    has_default = true;
                }

                let fallthrough_target = fallthrough;
                let block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                    builder.switch_scope(
                        _label.map(|s| s.to_string()),
                        continuation_id,
                        |builder| {
                            for consequent in &case.consequent {
                                lower_statement(builder, consequent, None, parent_scope)?;
                            }
                            Ok(Terminal::Goto {
                                block: fallthrough_target,
                                variant: GotoVariant::Break,
                                id: EvaluationOrder(0),
                                loc: case_loc.clone(),
                            })
                        },
                    )
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
                    id: EvaluationOrder(0),
                    loc,
                },
                continuation_block,
            );
        }
        oxc::Statement::TryStatement(try_stmt) => {
            let loc = builder.source_location(try_stmt.span);
            let continuation_block = builder.reserve(BlockKind::Block);
            let continuation_id = continuation_block.id;

            let handler_clause = match &try_stmt.handler {
                Some(h) => h,
                None => {
                    builder.record_error(CompilerErrorDetail {
                        category: ErrorCategory::Todo,
                        reason:
                            "(BuildHIR::lowerStatement) Handle TryStatement without a catch clause"
                                .to_string(),
                        description: None,
                        loc: loc.clone(),
                        suggestions: None,
                    })?;
                    return Ok(());
                }
            };

            if try_stmt.finalizer.is_some() {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Todo,
                    reason: "(BuildHIR::lowerStatement) Handle TryStatement with a finalizer ('finally') clause".to_string(),
                    description: None,
                    loc: loc.clone(),
                    suggestions: None,
                })?;
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
                    let mut id_locs = Vec::new();
                    collect_catch_pattern_identifier_locs(builder, &param.pattern, &mut id_locs);
                    for id_loc in id_locs {
                        builder.record_error(CompilerErrorDetail {
                                reason: "(BuildHIR::lowerAssignment) Could not find binding for declaration.".to_string(),
                                category: ErrorCategory::Invariant,
                                loc: id_loc,
                                description: None,
                                suggestions: None,
                            })?;
                    }
                    None
                } else {
                    let param_loc = builder.source_location(param.pattern.span());
                    let id = builder.make_temporary(param_loc.clone());
                    promote_temporary(builder, id);
                    let place = Place {
                        identifier: id,
                        effect: Effect::Unknown,
                        reactive: false,
                        loc: param_loc.clone(),
                    };
                    // Emit DeclareLocal for the catch binding
                    lower_value_to_temporary(
                        builder,
                        InstructionValue::DeclareLocal {
                            lvalue: LValue { kind: InstructionKind::Catch, place: place.clone() },
                            type_annotation: None,
                            loc: param_loc,
                        },
                    )?;
                    Some((place, &param.pattern))
                }
            } else {
                None
            };

            // Create the handler (catch) block
            let handler_binding_for_block = handler_binding_info.clone();
            let handler_loc = builder.source_location(handler_clause.span);
            // Use the catch param's loc for the assignment, matching TS: handlerBinding.path.node.loc
            let handler_param_loc =
                handler_clause.param.as_ref().map(|p| builder.source_location(p.pattern.span()));
            let handler_block = builder.try_enter(BlockKind::Catch, |builder, _block_id| {
                if let Some((ref place, pattern)) = handler_binding_for_block {
                    lower_binding_assignment(
                        builder,
                        handler_param_loc.clone().flatten().or_else(|| handler_loc.clone()),
                        InstructionKind::Catch,
                        pattern,
                        place.clone(),
                        AssignmentStyle::Assignment,
                    )?;
                }
                // Lower the catch body using lower_block_statement to get hoisting support.
                // Use the catch clause's scope (which contains the catch param binding).
                // Fall back to the body block's own scope if the catch clause scope is missing.
                let catch_scope = builder
                    .scope_info()
                    .resolve_scope_for_node(Some(handler_clause.span.start))
                    .or_else(|| {
                        builder
                            .scope_info()
                            .resolve_scope_for_node(Some(handler_clause.body.span.start))
                    });
                if let Some(scope_id) = catch_scope {
                    lower_block_statement_with_scope(
                        builder,
                        &handler_clause.body.body,
                        handler_clause.body.span.start,
                        scope_id,
                    )?;
                } else {
                    lower_block_statement(
                        builder,
                        &handler_clause.body.body,
                        handler_clause.body.span.start,
                        parent_scope,
                    )?;
                }
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc: handler_loc.clone(),
                })
            })?;

            // Create the try block
            let try_body_loc = builder.source_location(try_stmt.block.span);
            let try_block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                builder.try_enter_try_catch(handler_block, |builder| {
                    lower_block_statement(
                        builder,
                        &try_stmt.block.body,
                        try_stmt.block.span.start,
                        parent_scope,
                    )?;
                    Ok(())
                })?;
                Ok(Terminal::Goto {
                    block: continuation_id,
                    variant: GotoVariant::Try,
                    id: EvaluationOrder(0),
                    loc: try_body_loc.clone(),
                })
            })?;

            builder.terminate_with_continuation(
                Terminal::Try {
                    block: try_block,
                    handler_binding: handler_binding_info.map(|(place, _)| place),
                    handler: handler_block,
                    fallthrough: continuation_id,
                    id: EvaluationOrder(0),
                    loc,
                },
                continuation_block,
            );
        }
        oxc::Statement::BreakStatement(brk) => {
            let loc = builder.source_location(brk.span);
            let label_name = brk.label.as_ref().map(|l| l.name.as_str());
            let target = builder.lookup_break(label_name)?;
            let fallthrough = builder.reserve(BlockKind::Block);
            builder.terminate_with_continuation(
                Terminal::Goto {
                    block: target,
                    variant: GotoVariant::Break,
                    id: EvaluationOrder(0),
                    loc,
                },
                fallthrough,
            );
        }
        oxc::Statement::ContinueStatement(cont) => {
            let loc = builder.source_location(cont.span);
            let label_name = cont.label.as_ref().map(|l| l.name.as_str());
            let target = builder.lookup_continue(label_name)?;
            let fallthrough = builder.reserve(BlockKind::Block);
            builder.terminate_with_continuation(
                Terminal::Goto {
                    block: target,
                    variant: GotoVariant::Continue,
                    id: EvaluationOrder(0),
                    loc,
                },
                fallthrough,
            );
        }
        oxc::Statement::LabeledStatement(labeled_stmt) => {
            let label_name = labeled_stmt.label.name.as_str();
            let loc = builder.source_location(labeled_stmt.span);

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
                    let body_loc = builder.source_location(labeled_stmt.body.span());
                    let label_string = label_name.to_string();

                    let block = builder.try_enter(BlockKind::Block, |builder, _block_id| {
                        builder.label_scope(label_string, continuation_id, |builder| {
                            lower_statement(builder, &labeled_stmt.body, None, parent_scope)?;
                            Ok(())
                        })?;
                        Ok(Terminal::Goto {
                            block: continuation_id,
                            variant: GotoVariant::Break,
                            id: EvaluationOrder(0),
                            loc: body_loc,
                        })
                    })?;

                    builder.terminate_with_continuation(
                        Terminal::Label {
                            block,
                            fallthrough: continuation_id,
                            id: EvaluationOrder(0),
                            loc,
                        },
                        continuation_block,
                    );
                }
            }
        }
        oxc::Statement::WithStatement(with_stmt) => {
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::UnsupportedSyntax,
                reason: "JavaScript 'with' syntax is not supported".to_string(),
                description: Some("'with' syntax is considered deprecated and removed from JavaScript standards, consider alternatives".to_string()),
                loc: builder.source_location(with_stmt.span),
                suggestions: None,
            })?;
        }
        oxc::Statement::ClassDeclaration(cls) => {
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::UnsupportedSyntax,
                reason: "Inline `class` declarations are not supported".to_string(),
                description: Some(
                    "Move class declarations outside of components/hooks".to_string(),
                ),
                loc: builder.source_location(cls.span),
                suggestions: None,
            })?;
        }
        oxc::Statement::ImportDeclaration(_)
        | oxc::Statement::ExportNamedDeclaration(_)
        | oxc::Statement::ExportDefaultDeclaration(_)
        | oxc::Statement::ExportAllDeclaration(_) => {
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Syntax,
                reason: "JavaScript `import` and `export` statements may only appear at the top level of a module".to_string(),
                description: None,
                loc: builder.source_location(stmt.span()),
                suggestions: None,
            })?;
        }
        oxc::Statement::TSEnumDeclaration(e) => {
            // Inline TS `enum` has runtime semantics, so preserve it: emit an
            // `UnsupportedNode` carrying the borrowed oxc statement node so the
            // back-end can clone it verbatim into the output allocator (matching
            // the original Babel front-end, which wrapped the enum the same way).
            let loc = builder.source_location(e.span);
            lower_value_to_temporary(
                builder,
                InstructionValue::UnsupportedNode {
                    node_type: Some("TSEnumDeclaration".to_string()),
                    stmt,
                    loc,
                },
            )?;
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
    var_decl: &'a oxc::VariableDeclaration<'a>,
) -> Result<(), CompilerDiagnostic> {
    use oxc::VariableDeclarationKind as VK;
    if matches!(var_decl.kind, VK::Var) {
        builder.record_error(CompilerErrorDetail {
            reason: "(BuildHIR::lowerStatement) Handle var kinds in VariableDeclaration"
                .to_string(),
            category: ErrorCategory::Todo,
            loc: builder.source_location(var_decl.span),
            description: None,
            suggestions: None,
        })?;
        // Treat `var` as `let` so references to the variable don't break
    }
    let kind = match var_decl.kind {
        VK::Let | VK::Var => InstructionKind::Let,
        VK::Const | VK::Using | VK::AwaitUsing => InstructionKind::Const,
    };
    for declarator in &var_decl.declarations {
        let stmt_loc = builder.source_location(var_decl.span);
        if let Some(init) = &declarator.init {
            let value = lower_expression_to_temporary(builder, init)?;
            let assign_style = match &declarator.id {
                oxc::BindingPattern::ObjectPattern(_) | oxc::BindingPattern::ArrayPattern(_) => {
                    AssignmentStyle::Destructure
                }
                _ => AssignmentStyle::Assignment,
            };
            lower_binding_assignment(builder, stmt_loc, kind, &declarator.id, value, assign_style)?;
        } else if let oxc::BindingPattern::BindingIdentifier(id) = &declarator.id {
            // No init: emit DeclareLocal or DeclareContext
            let id_loc = builder.source_location(id.span);
            let mut binding = builder.resolve_identifier(
                id.name.as_str(),
                id.span.start,
                id_loc.clone(),
                Some(id.span.start),
            )?;
            if !matches!(binding, VariableBinding::Identifier { .. }) {
                // Position-based resolution failed (synthetic $$gen vars at
                // position 0). Try scope lookup including descendants.
                if let Some((binding_id, binding_data)) = builder
                    .scope_info()
                    .find_binding_id_in_descendants(id.name.as_str(), builder.function_scope())
                {
                    let binding_kind =
                        crate::react_compiler_lowering::convert_binding_kind(&binding_data.kind);
                    let identifier = builder.resolve_binding_with_loc(
                        id.name.as_str(),
                        binding_id,
                        id_loc.clone(),
                    )?;
                    binding = VariableBinding::Identifier { identifier, binding_kind };
                }
            }
            match binding {
                VariableBinding::Identifier { identifier, .. } => {
                    // Update the identifier's loc to the declaration site
                    // (it may have been first created at a reference site during hoisting)
                    builder.set_identifier_declaration_loc(identifier, &id_loc);
                    let place = Place {
                        identifier,
                        effect: Effect::Unknown,
                        reactive: false,
                        loc: id_loc.clone(),
                    };
                    if builder.is_context_identifier(
                        id.name.as_str(),
                        id.span.start,
                        Some(id.span.start),
                    ) {
                        if kind == InstructionKind::Const {
                            builder.record_error(CompilerErrorDetail {
                                reason: "Expect `const` declaration not to be reassigned"
                                    .to_string(),
                                category: ErrorCategory::Syntax,
                                loc: id_loc.clone(),
                                description: None,
                                suggestions: None,
                            })?;
                        }
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::DeclareContext {
                                lvalue: LValue { kind: InstructionKind::Let, place },
                                loc: id_loc,
                            },
                        )?;
                    } else {
                        let type_annotation = declarator
                            .type_annotation
                            .as_ref()
                            .map(|ann| ts_type_node_type(&ann.type_annotation).to_string());
                        lower_value_to_temporary(
                            builder,
                            InstructionValue::DeclareLocal {
                                lvalue: LValue { kind, place },
                                type_annotation,
                                loc: id_loc,
                            },
                        )?;
                    }
                }
                _ => {
                    builder.record_error(CompilerErrorDetail {
                        reason: "Could not find binding for declaration".to_string(),
                        category: ErrorCategory::Invariant,
                        loc: id_loc,
                        description: None,
                        suggestions: None,
                    })?;
                }
            }
        } else {
            builder.record_error(CompilerErrorDetail {
                reason: "Expected variable declaration to be an identifier if no initializer was provided".to_string(),
                category: ErrorCategory::Syntax,
                loc: builder.source_location(declarator.span),
                description: None,
                suggestions: None,
            })?;
        }
    }
    Ok(())
}

/// Lower the `left` target of a for-in / for-of loop, dispatching to the binding
/// assignment (for `VariableDeclaration`) or assignment-target (for plain
/// patterns) lowering. Mirrors the original `ForInOfLeft` match.
fn lower_for_in_of_left<'a>(
    builder: &mut HirBuilder<'a, '_>,
    left: &'a oxc::ForStatementLeft<'a>,
    left_loc: Option<SourceLocation>,
    value: Place,
) -> Result<Option<Place>, CompilerError> {
    match left {
        oxc::ForStatementLeft::VariableDeclaration(var_decl) => {
            if var_decl.declarations.len() != 1 {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Invariant,
                    reason: format!(
                        "Expected only one declaration in for-in/of init, got {}",
                        var_decl.declarations.len()
                    ),
                    description: None,
                    loc: left_loc.clone(),
                    suggestions: None,
                })?;
            }
            if let Some(declarator) = var_decl.declarations.first() {
                lower_binding_assignment(
                    builder,
                    left_loc,
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
            left_loc,
            InstructionKind::Reassign,
            left.to_assignment_target(),
            value,
            AssignmentStyle::Assignment,
        ),
    }
}

/// Collect identifier locs from a destructured catch-clause pattern, for error
/// reporting (Babel doesn't register destructured catch bindings).
fn collect_catch_pattern_identifier_locs(
    builder: &HirBuilder<'_, '_>,
    pat: &oxc::BindingPattern,
    locs: &mut Vec<Option<SourceLocation>>,
) {
    match pat {
        oxc::BindingPattern::BindingIdentifier(id) => {
            locs.push(builder.source_location(id.span));
        }
        oxc::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_catch_pattern_identifier_locs(builder, &prop.value, locs);
            }
            if let Some(rest) = &obj.rest {
                collect_catch_pattern_identifier_locs(builder, &rest.argument, locs);
            }
        }
        oxc::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_catch_pattern_identifier_locs(builder, elem, locs);
            }
            if let Some(rest) = &arr.rest {
                collect_catch_pattern_identifier_locs(builder, &rest.argument, locs);
            }
        }
        // The original matched only Identifier/Object/Array; AssignmentPattern
        // (destructuring defaults) fell through its `_ => {}` catch-all.
        oxc::BindingPattern::AssignmentPattern(_) => {}
    }
}
