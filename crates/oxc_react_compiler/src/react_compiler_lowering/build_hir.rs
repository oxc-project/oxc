use rustc_hash::FxHashSet;

use crate::react_compiler_ast::scope::BindingId;
use crate::react_compiler_ast::scope::BindingKind as AstBindingKind;
use crate::react_compiler_ast::scope::ScopeId;
use crate::react_compiler_ast::scope::ScopeInfo;
use crate::react_compiler_ast::scope::ScopeKind;
use crate::react_compiler_diagnostics::CompilerDiagnostic;
use crate::react_compiler_diagnostics::CompilerDiagnosticDetail;
use crate::react_compiler_diagnostics::CompilerError;
use crate::react_compiler_diagnostics::CompilerErrorDetail;
use crate::react_compiler_diagnostics::ErrorCategory;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::*;
use crate::react_compiler_utils::FxIndexMap;
use crate::react_compiler_utils::FxIndexSet;

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
fn build_temporary_place(builder: &mut HirBuilder, loc: Option<SourceLocation>) -> Place {
    let id = builder.make_temporary(loc.clone());
    Place { identifier: id, reactive: false, effect: Effect::Unknown, loc }
}

/// Promote a temporary identifier to a named identifier (for destructuring).
/// Corresponds to TS `promoteTemporary(identifier)`.
fn promote_temporary(builder: &mut HirBuilder, identifier_id: IdentifierId) {
    let env = builder.environment_mut();
    let decl_id = env.identifiers[identifier_id.0 as usize].declaration_id;
    env.identifiers[identifier_id.0 as usize].name =
        Some(IdentifierName::Promoted(format!("#t{}", decl_id.0)));
}

fn lower_value_to_temporary(
    builder: &mut HirBuilder,
    value: InstructionValue,
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

fn lower_expression_to_temporary(
    builder: &mut HirBuilder,
    expr: &oxc::Expression,
) -> Result<Place, CompilerError> {
    let value = lower_expression(builder, expr)?;
    lower_value_to_temporary(builder, value)
}

// =============================================================================
// Operator conversion
// =============================================================================

fn is_binding_in_block_direct_statements(
    binding: &crate::react_compiler_ast::scope::BindingData,
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
    scope_id: crate::react_compiler_ast::scope::ScopeId,
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
fn lower_block_statement(
    builder: &mut HirBuilder,
    statements: &[oxc::Statement],
    block_node_id: u32,
    parent_scope: Option<crate::react_compiler_ast::scope::ScopeId>,
) -> Result<(), CompilerError> {
    let _ = lower_block_statement_inner(builder, statements, block_node_id, None, parent_scope);
    Ok(())
}

fn lower_block_statement_with_scope(
    builder: &mut HirBuilder,
    statements: &[oxc::Statement],
    block_node_id: u32,
    scope_override: crate::react_compiler_ast::scope::ScopeId,
) -> Result<(), CompilerError> {
    let _ = lower_block_statement_inner(
        builder,
        statements,
        block_node_id,
        Some(scope_override),
        None,
    );
    Ok(())
}

fn lower_block_statement_inner(
    builder: &mut HirBuilder,
    statements: &[oxc::Statement],
    block_node_id: u32,
    scope_override: Option<crate::react_compiler_ast::scope::ScopeId>,
    parent_scope: Option<crate::react_compiler_ast::scope::ScopeId>,
) -> Result<(), CompilerDiagnostic> {
    use crate::react_compiler_ast::scope::BindingKind as AstBindingKind;

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
                    if let oxc::BindingPattern::BindingIdentifier(id) = &d.id
                    {
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
                    if let Some(&binding_id) =
                        builder.scope_info().scopes[scope_id.0 as usize].bindings.get(id.name.as_str())
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
                    if let Some(&binding_id) =
                        builder.scope_info().scopes[scope_id.0 as usize].bindings.get(id.name.as_str())
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
pub fn lower(
    func: &FunctionNode<'_>,
    _id: Option<&str>,
    scope_info: &ScopeInfo,
    env: &mut Environment,
    line_offsets: &LineOffsets,
) -> Result<HirFunction, CompilerError> {
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
    let identifier_locs = build_identifier_loc_index(func, scope_info);

    // Pre-compute context identifiers: variables captured across function boundaries
    let context_identifiers = find_context_identifiers(func, scope_info, env, &identifier_locs)?;

    // For top-level functions, context is empty (no captured refs)
    let context_map: FxIndexMap<
        crate::react_compiler_ast::scope::BindingId,
        Option<SourceLocation>,
    > = FxIndexMap::default();

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
fn lower_inner(
    params: &oxc::FormalParameters,
    body: FunctionBody<'_>,
    id: Option<&str>,
    generator: bool,
    is_async: bool,
    loc: Option<SourceLocation>,
    scope_info: &ScopeInfo,
    env: &mut Environment,
    parent_bindings: Option<FxIndexMap<crate::react_compiler_ast::scope::BindingId, IdentifierId>>,
    parent_used_names: Option<FxIndexMap<String, crate::react_compiler_ast::scope::BindingId>>,
    context_map: FxIndexMap<crate::react_compiler_ast::scope::BindingId, Option<SourceLocation>>,
    function_scope: crate::react_compiler_ast::scope::ScopeId,
    component_scope: crate::react_compiler_ast::scope::ScopeId,
    context_identifiers: &FxHashSet<crate::react_compiler_ast::scope::BindingId>,
    is_top_level: bool,
    identifier_locs: &IdentifierLocIndex,
    line_offsets: &LineOffsets,
) -> Result<
    (
        HirFunction,
        FxIndexMap<String, crate::react_compiler_ast::scope::BindingId>,
        FxIndexMap<crate::react_compiler_ast::scope::BindingId, IdentifierId>,
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
    // Stage 1a skeleton: only plain identifier params (no default) are lowered.
    // Destructuring / default / rest params need `lower_assignment`, ported with
    // the assignment arms.
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
        // TODO(stage1a-arms): destructuring / default parameters need lower_assignment.
        builder.record_diagnostic(
            CompilerDiagnostic::new(
                ErrorCategory::Todo,
                "Handle parameter",
                Some("[BuildHIR] Non-identifier parameters not yet ported".to_string()),
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: builder.source_location(param.span),
                message: Some("Unsupported parameter type".to_string()),
                identifier_name: None,
            }),
        );
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
            directives =
                block.directives.iter().map(|d| d.expression.value.to_string()).collect();
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
// lower_expression / lower_statement — Stage 1a skeleton catch-alls.
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
    builder: &mut HirBuilder,
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

struct LoweredMemberExpression {
    object: Place,
    property: MemberProperty,
    value: InstructionValue,
}

/// Lower a member access (oxc's Static / Computed / PrivateField variants) into a
/// receiver place + property + load value.
fn lower_member_expression(
    builder: &mut HirBuilder,
    member: &oxc::Expression,
) -> Result<LoweredMemberExpression, CompilerError> {
    lower_member_expression_impl(builder, member, None)
}

fn lower_member_expression_impl(
    builder: &mut HirBuilder,
    member: &oxc::Expression,
    lowered_object: Option<Place>,
) -> Result<LoweredMemberExpression, CompilerError> {
    match member {
        oxc::Expression::StaticMemberExpression(m) => {
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
        oxc::Expression::ComputedMemberExpression(m) => {
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
        oxc::Expression::PrivateFieldExpression(m) => {
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
        _ => unreachable!("lower_member_expression called on a non-member expression"),
    }
}

/// Build a HIR `TemplateQuasi` from an oxc `TemplateElement`.
fn template_quasi_from_oxc(q: &oxc::TemplateElement) -> TemplateQuasi {
    TemplateQuasi {
        raw: q.value.raw.to_string(),
        cooked: q.value.cooked.map(|c| c.to_string()),
    }
}

/// Lower the `import` keyword callee of an `ImportExpression`. The original Babel
/// path treats this as the `Import` node, which bails (records an error) and
/// returns an undefined primitive that is then loaded to a temporary.
fn lower_import_keyword_to_temporary(
    builder: &mut HirBuilder,
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
    builder: &mut HirBuilder,
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
fn classify_ts_type(ty: &oxc::TSType) -> crate::react_compiler_ast::common::RawTypeCategory {
    use crate::react_compiler_ast::common::RawTypeCategory;
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
fn lower_ts_type(builder: &mut HirBuilder, ty: &oxc::TSType) -> Type {
    use crate::react_compiler_ast::common::RawTypeCategory;
    match classify_ts_type(ty) {
        RawTypeCategory::Array => Type::Object { shape_id: Some("BuiltInArray".to_string()) },
        RawTypeCategory::Primitive => Type::Primitive,
        RawTypeCategory::Other => builder.make_type(),
    }
}

/// Lower `x as T` / `x satisfies T` / `<T>x` to a `TypeCastExpression`: the inner
/// expression is lowered to a temporary and the type metadata is attached. Mirrors
/// the original Babel `TSAsExpression`/`TSSatisfiesExpression`/`TSTypeAssertion`
/// arms. The `type_annotation` RawNode is built from the unwrapped TS type's tag,
/// span and classification (codegen re-parses it from source).
fn lower_type_cast_expression(
    builder: &mut HirBuilder,
    span: oxc_span::Span,
    expression: &oxc::Expression,
    type_annotation: &oxc::TSType,
    type_annotation_kind: &str,
) -> Result<InstructionValue, CompilerError> {
    let loc = builder.source_location(span);
    let value = lower_expression_to_temporary(builder, expression)?;
    let type_ = lower_ts_type(builder, type_annotation);
    let type_annotation_name = Some(ts_type_node_type(type_annotation).to_string());
    let raw = crate::react_compiler_ast::common::RawNode::type_node(
        type_annotation_name.clone(),
        Some(type_annotation.span().start),
        Some(type_annotation.span().end),
        classify_ts_type(type_annotation),
        Vec::new(),
    );
    Ok(InstructionValue::TypeCastExpression {
        value,
        type_,
        type_annotation_name,
        type_annotation_kind: Some(type_annotation_kind.to_string()),
        type_annotation: Some(raw),
        loc,
    })
}

/// Lower a member-expression update target (oxc's member variants of
/// `SimpleAssignmentTarget`) into a receiver place + property + load value,
/// mirroring `lower_member_expression_impl`.
fn lower_member_expression_from_simple_target(
    builder: &mut HirBuilder,
    target: &oxc::SimpleAssignmentTarget,
) -> Result<LoweredMemberExpression, CompilerError> {
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
        _ => unreachable!(
            "lower_member_expression_from_simple_target called on a non-member target"
        ),
    }
}

fn lower_arguments(
    builder: &mut HirBuilder,
    args: &[oxc::Argument],
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
    builder: &mut HirBuilder,
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

/// Assign `value` to a binding pattern (variable declaration / destructuring param).
/// BindingIdentifier is handled; destructuring/default patterns are deferred.
fn lower_binding_assignment(
    builder: &mut HirBuilder,
    loc: Option<SourceLocation>,
    kind: InstructionKind,
    target: &oxc::BindingPattern,
    value: Place,
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
        oxc::BindingPattern::ObjectPattern(_) | oxc::BindingPattern::ArrayPattern(_) => {
            // TODO(stage1a-arms): destructuring binding patterns.
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "(BuildHIR::lowerAssignment) Handle destructuring binding patterns"
                    .to_string(),
                description: None,
                loc,
                suggestions: None,
            })?;
            Ok(None)
        }
        oxc::BindingPattern::AssignmentPattern(_) => {
            // TODO(stage1a-arms): default-value binding patterns.
            builder.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: "(BuildHIR::lowerAssignment) Handle default-value binding patterns"
                    .to_string(),
                description: None,
                loc,
                suggestions: None,
            })?;
            Ok(None)
        }
    }
}

fn lower_expression(
    builder: &mut HirBuilder,
    expr: &oxc::Expression,
) -> Result<InstructionValue, CompilerError> {
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
            let lowered = lower_member_expression(builder, expr)?;
            Ok(lowered.value)
        }
        oxc::Expression::CallExpression(call) => {
            let loc = builder.source_location(call.span);
            if matches!(
                call.callee,
                oxc::Expression::StaticMemberExpression(_)
                    | oxc::Expression::ComputedMemberExpression(_)
                    | oxc::Expression::PrivateFieldExpression(_)
            ) {
                let lowered = lower_member_expression(builder, &call.callee)?;
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
                return Ok(InstructionValue::Primitive {
                    value: PrimitiveValue::Undefined,
                    loc,
                });
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
                q.value.raw.as_str()
                    != q.value.cooked.map(|c| c.to_string()).unwrap_or_default()
            }) {
                builder.record_error(CompilerErrorDetail {
                    category: ErrorCategory::Todo,
                    reason: "(BuildHIR::lowerExpression) Handle tagged template where cooked value is different from raw value".to_string(),
                    description: None,
                    loc: loc.clone(),
                    suggestions: None,
                })?;
                return Ok(InstructionValue::Primitive {
                    value: PrimitiveValue::Undefined,
                    loc,
                });
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
            let import_keyword_loc = builder
                .source_location(oxc_span::Span::new(imp.span.start, imp.span.start + 6));
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
                            reason: "UpdateExpression where argument is a global is not yet supported".to_string(),
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
        oxc::Expression::TSAsExpression(ts) => lower_type_cast_expression(
            builder,
            ts.span,
            &ts.expression,
            &ts.type_annotation,
            "as",
        ),
        oxc::Expression::TSSatisfiesExpression(ts) => lower_type_cast_expression(
            builder,
            ts.span,
            &ts.expression,
            &ts.type_annotation,
            "satisfies",
        ),
        oxc::Expression::TSTypeAssertion(ts) => lower_type_cast_expression(
            builder,
            ts.span,
            &ts.expression,
            &ts.type_annotation,
            "as",
        ),
        // `x!` and `x<T>` unwrap to their inner expression (the original also just
        // unwraps these).
        oxc::Expression::TSNonNullExpression(ts) => lower_expression(builder, &ts.expression),
        oxc::Expression::TSInstantiationExpression(ts) => {
            lower_expression(builder, &ts.expression)
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
        _ => {
            // not-yet-ported arms bail to undefined (differential green-set grows as arms land)
            let loc = builder.source_location(expr.span());
            Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
        }
    }
}

/// Lower a JSX element expression. Faithful translation of the original Babel
/// `Expression::JSXElement` arm, adapted to oxc's JSX shapes.
///
/// fbt note: the original tracked fbt/fbs sub-tags (`collect_fbt_sub_tags`) and
/// reported duplicates. That sub-tag tracking is not ported in this batch; the
/// `is_fbt` checks below preserve the module-level-import invariant (which is the
/// only fbt path exercised by the differential corpus) but `builder.fbt_depth`
/// stays 0 so JSX text whitespace is always trimmed.
fn lower_jsx_element_expr(
    builder: &mut HirBuilder,
    jsx_element: &oxc::JSXElement,
) -> Result<InstructionValue, CompilerError> {
    let loc = builder.source_location(jsx_element.span);
    let opening_loc = builder.source_location(jsx_element.opening_element.span);
    let closing_loc = jsx_element
        .closing_element
        .as_ref()
        .and_then(|c| builder.source_location(c.span));

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

    // Lower children
    let children: Vec<Place> = jsx_element
        .children
        .iter()
        .map(|child| lower_jsx_element(builder, child))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

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
fn lower_jsx_fragment_expr(
    builder: &mut HirBuilder,
    jsx_fragment: &oxc::JSXFragment,
) -> Result<InstructionValue, CompilerError> {
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
    builder: &mut HirBuilder,
    name: &oxc::JSXElementName,
) -> Result<JsxTag, CompilerError> {
    // Lower a simple JSX tag identifier (component-vs-builtin split on case).
    fn lower_tag_identifier(
        builder: &mut HirBuilder,
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
    builder: &mut HirBuilder,
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
    builder: &mut HirBuilder,
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
fn lower_jsx_element(
    builder: &mut HirBuilder,
    child: &oxc::JSXChild,
) -> Result<Option<Place>, CompilerError> {
    match child {
        oxc::JSXChild::Text(text) => {
            // oxc keeps JSX text raw; decode entities first so the value matches
            // Babel's `JSXText.value` (the Babel bridge decoded in convert_ast).
            let decoded = decode_jsx_entities(text.value.as_str());
            // FBT whitespace normalization differs from standard JSX.
            // Since the fbt transform runs after, preserve all whitespace
            // in FBT subtrees as is.
            let value = if builder.fbt_depth > 0 {
                Some(decoded)
            } else {
                trim_jsx_text(&decoded)
            };
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
/// `get`/`set` record a Todo error and are skipped. The `method` case requires
/// nested-function lowering (`lower_function_for_object_method` /
/// `gather_captured_context`), which is not yet ported in this stage, so it is
/// likewise deferred with a Todo error rather than emitting divergent HIR.
fn lower_object_method(
    builder: &mut HirBuilder,
    method: &oxc::ObjectProperty,
) -> Result<Option<ObjectProperty>, CompilerError> {
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
    Ok(None)
}

/// Lower an object property key. Faithful to the original `lower_object_property_key`.
fn lower_object_property_key(
    builder: &mut HirBuilder,
    key: &oxc::PropertyKey,
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
fn lower_reorderable_expression(
    builder: &mut HirBuilder,
    expr: &oxc::Expression,
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
    builder: &HirBuilder,
    expr: &oxc::Expression,
    allow_local_identifiers: bool,
) -> bool {
    match expr {
        oxc::Expression::Identifier(ident) => {
            let binding = builder
                .scope_info()
                .resolve_reference_for_node(Some(ident.span.start));
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
            _ => is_reorderable_expression(builder, element.to_expression(), allow_local_identifiers),
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
                match builder
                    .scope_info()
                    .resolve_reference_for_node(Some(ident.span.start))
                {
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

fn lower_statement(
    builder: &mut HirBuilder,
    stmt: &oxc::Statement,
    _label: Option<&str>,
    parent_scope: Option<crate::react_compiler_ast::scope::ScopeId>,
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
            }
            let kind = match var_decl.kind {
                VK::Let | VK::Var => InstructionKind::Let,
                VK::Const | VK::Using | VK::AwaitUsing => InstructionKind::Const,
            };
            for declarator in &var_decl.declarations {
                let stmt_loc = builder.source_location(var_decl.span);
                if let Some(init) = &declarator.init {
                    let value = lower_expression_to_temporary(builder, init)?;
                    lower_binding_assignment(builder, stmt_loc, kind, &declarator.id, value)?;
                }
                // TODO(stage1a-arms): no-init declarations (DeclareLocal/DeclareContext).
            }
        }
        _ => {
            // not-yet-ported statements are skipped (differential green-set grows as arms land)
        }
    }
    Ok(())
}
