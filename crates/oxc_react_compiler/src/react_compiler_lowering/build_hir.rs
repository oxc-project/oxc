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

fn lower_expression(
    builder: &mut HirBuilder,
    expr: &oxc::Expression,
) -> Result<InstructionValue, CompilerError> {
    let loc = builder.source_location(expr.span());
    Ok(InstructionValue::Primitive { value: PrimitiveValue::Undefined, loc })
}

fn lower_statement(
    builder: &mut HirBuilder,
    stmt: &oxc::Statement,
    _label: Option<&str>,
    _parent_scope: Option<crate::react_compiler_ast::scope::ScopeId>,
) -> Result<(), CompilerDiagnostic> {
    let _ = (builder, stmt);
    Ok(())
}
