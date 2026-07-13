// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Port of ValidatePreservedManualMemoization.ts
//!
//! Validates that all explicit manual memoization (useMemo/useCallback) was
//! accurately preserved, and that no originally memoized values became
//! unmemoized in the output.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::{
    ArrayPatternElement, Effect, ObjectPropertyOrSpread, Pattern, PropertyLiteral,
    PrunedReactiveScopeBlock, ReactiveTerminalStatement,
};
use crate::react_compiler_hir::{
    DeclarationId, DependencyPathEntry, Identifier, IdentifierId, IdentifierName, InstructionKind,
    InstructionValue, ManualMemoDependency, ManualMemoDependencyRoot, Place, ReactiveBlock,
    ReactiveFunction, ReactiveInstruction, ReactiveScopeBlock, ReactiveStatement, ReactiveTerminal,
    ReactiveValue, ScopeId,
};
use oxc_index::IndexSlice;
use oxc_span::Span;

/// State tracked during manual memo validation within a StartMemoize..FinishMemoize range.
struct ManualMemoBlockState<'h> {
    /// Reassigned temporaries (declaration_id -> set of identifier ids that were reassigned to it).
    reassignments: FxHashMap<DeclarationId, FxHashSet<IdentifierId>>,
    /// Source location of the StartMemoize instruction.
    span: Option<Span>,
    /// Declarations produced within this manual memo block.
    decls: FxHashSet<DeclarationId>,
    /// Normalized deps from source (useMemo/useCallback dep array).
    deps_from_source: Option<Vec<ManualMemoDependency<'h>>>,
    /// Manual memo id from StartMemoize.
    manual_memo_id: u32,
}

/// Top-level visitor state.
struct VisitorState<'a, 'h> {
    env: &'a mut Environment<'h>,
    manual_memo_state: Option<ManualMemoBlockState<'h>>,
    /// Completed (non-pruned) scope IDs.
    scopes: FxHashSet<ScopeId>,
    /// Completed pruned scope IDs.
    pruned_scopes: FxHashSet<ScopeId>,
    /// Map from identifier ID to its normalized manual memo dependency.
    temporaries: FxHashMap<IdentifierId, ManualMemoDependency<'h>>,
}

/// Validate that manual memoization (useMemo/useCallback) is preserved.
///
/// Walks the reactive function looking for StartMemoize/FinishMemoize instructions
/// and checks that:
/// 1. Dependencies' scopes have completed before the memo block starts
/// 2. Memoized values are actually within scopes (not unmemoized)
/// 3. Inferred scope dependencies match the source dependencies
pub fn validate_preserved_manual_memoization<'h>(
    func: &ReactiveFunction<'h>,
    env: &mut Environment<'h>,
) {
    let mut state = VisitorState {
        env,
        manual_memo_state: None,
        scopes: FxHashSet::default(),
        pruned_scopes: FxHashSet::default(),
        temporaries: FxHashMap::default(),
    };
    visit_block(&func.body, &mut state);
}

fn is_named(ident: &Identifier) -> bool {
    matches!(ident.name, Some(IdentifierName::Named(_)))
}

fn visit_block<'h>(block: &ReactiveBlock<'h>, state: &mut VisitorState<'_, 'h>) {
    for stmt in block {
        visit_statement(stmt, state);
    }
}

fn visit_statement<'h>(stmt: &ReactiveStatement<'h>, state: &mut VisitorState<'_, 'h>) {
    match stmt {
        ReactiveStatement::Instruction(instr) => {
            visit_instruction(instr, state);
        }
        ReactiveStatement::Terminal(terminal) => {
            visit_terminal(terminal, state);
        }
        ReactiveStatement::Scope(scope_block) => {
            visit_scope(scope_block, state);
        }
        ReactiveStatement::PrunedScope(pruned) => {
            visit_pruned_scope(pruned, state);
        }
    }
}

fn visit_terminal<'h>(terminal: &ReactiveTerminalStatement<'h>, state: &mut VisitorState<'_, 'h>) {
    match &terminal.terminal {
        ReactiveTerminal::If { consequent, alternate, .. } => {
            visit_block(consequent, state);
            if let Some(alt) = alternate {
                visit_block(alt, state);
            }
        }
        ReactiveTerminal::Switch { cases, .. } => {
            for case in cases {
                if let Some(ref block) = case.block {
                    visit_block(block, state);
                }
            }
        }
        ReactiveTerminal::For { loop_block, .. }
        | ReactiveTerminal::ForOf { loop_block, .. }
        | ReactiveTerminal::ForIn { loop_block, .. }
        | ReactiveTerminal::While { loop_block, .. }
        | ReactiveTerminal::DoWhile { loop_block, .. } => {
            visit_block(loop_block, state);
        }
        ReactiveTerminal::Label { block, .. } => {
            visit_block(block, state);
        }
        ReactiveTerminal::Try { block, handler, .. } => {
            visit_block(block, state);
            visit_block(handler, state);
        }
        _ => {}
    }
}

fn visit_scope<'h>(scope_block: &ReactiveScopeBlock<'h>, state: &mut VisitorState<'_, 'h>) {
    // Traverse the scope's instructions first
    visit_block(&scope_block.instructions, state);

    // After traversing, validate scope dependencies against manual memo deps
    if let Some(ref memo_state) = state.manual_memo_state {
        if let Some(ref deps_from_source) = memo_state.deps_from_source {
            let scope = &state.env.scopes[scope_block.scope];
            let deps = scope.dependencies.clone();
            let memo_span = memo_state.span;
            let decls = memo_state.decls.clone();
            let deps_from_source = deps_from_source.clone();
            let temporaries = state.temporaries.clone();
            for dep in &deps {
                validate_inferred_dep(
                    dep.identifier,
                    &dep.path,
                    &temporaries,
                    &decls,
                    &deps_from_source,
                    state.env,
                    memo_span,
                );
            }
        }
    }

    // Mark scope and merged scopes as completed
    let scope = &state.env.scopes[scope_block.scope];
    let merged = scope.merged.clone();
    state.scopes.insert(scope_block.scope);
    for merged_id in merged {
        state.scopes.insert(merged_id);
    }
}

fn visit_pruned_scope<'h>(pruned: &PrunedReactiveScopeBlock<'h>, state: &mut VisitorState<'_, 'h>) {
    visit_block(&pruned.instructions, state);
    state.pruned_scopes.insert(pruned.scope);
}

fn visit_instruction<'h>(instr: &ReactiveInstruction<'h>, state: &mut VisitorState<'_, 'h>) {
    // Record temporaries and deps in the instruction's value
    record_temporaries(instr, state);

    match &instr.value {
        ReactiveValue::Instruction(InstructionValue::StartMemoize {
            manual_memo_id,
            deps,
            has_invalid_deps,
            ..
        }) => {
            // TS: Diagnostics.invariant(state.manualMemoState == null, ...)
            if state.manual_memo_state.is_some() {
                return;
            }

            // TS: if (value.hasInvalidDeps === true) { return; }
            if *has_invalid_deps {
                return;
            }

            let deps_from_source = deps.clone();

            state.manual_memo_state = Some(ManualMemoBlockState {
                span: instr.span,
                decls: FxHashSet::default(),
                deps_from_source,
                manual_memo_id: *manual_memo_id,
                reassignments: FxHashMap::default(),
            });

            // Check that each dependency's scope has completed before the memo
            // TS: for (const {identifier, span} of eachInstructionValueOperand(value))
            let operand_places = start_memoize_operands(deps);
            for place in &operand_places {
                let ident = &state.env.identifiers[place.identifier];
                if let Some(scope_id) = ident.scope {
                    if !state.scopes.contains(&scope_id) && !state.pruned_scopes.contains(&scope_id)
                    {
                        let diag = ErrorCategory::PreserveManualMemo
                            .diagnostic("Existing memoization could not be preserved")
                            .with_help(
                                "React Compiler has skipped optimizing this component because the existing manual memoization could not be preserved. \
                                 This dependency may be mutated later, which could cause the value to change unexpectedly",
                            )
                            .with_labels(
                                place
                                    .span
                                    .map(|s| s.label("This dependency may be modified later")),
                            );
                        state.env.record_diagnostic(diag);
                    }
                }
            }
        }
        ReactiveValue::Instruction(InstructionValue::FinishMemoize {
            decl,
            pruned,
            manual_memo_id,
            ..
        }) => {
            if state.manual_memo_state.is_none() {
                // StartMemoize had invalid deps, skip validation
                return;
            }

            // TS: Diagnostics.invariant(state.manualMemoState.manualMemoId === value.manualMemoId, ...)
            if state.manual_memo_state.as_ref().is_none_or(|s| s.manual_memo_id != *manual_memo_id)
            {
                state.manual_memo_state = None;
                return;
            }

            let memo_state = state.manual_memo_state.take().unwrap();

            if !pruned {
                // Check if the declared value is unmemoized
                let decl_ident = &state.env.identifiers[decl.identifier];

                if decl_ident.scope.is_none() {
                    // If the manual memo was inlined (useMemo -> IIFE), check reassignments
                    let decls_to_check = memo_state
                        .reassignments
                        .get(&decl_ident.declaration_id)
                        .map(|ids| ids.iter().copied().collect::<Vec<_>>())
                        .unwrap_or_else(|| vec![decl.identifier]);

                    for id in decls_to_check {
                        if is_unmemoized(id, &state.scopes, &state.env.identifiers) {
                            record_unmemoized_error(decl.span, state.env);
                        }
                    }
                } else {
                    // Single identifier with scope
                    if is_unmemoized(decl.identifier, &state.scopes, &state.env.identifiers) {
                        record_unmemoized_error(decl.span, state.env);
                    }
                }
            }
        }
        ReactiveValue::Instruction(InstructionValue::StoreLocal { lvalue, value, .. }) => {
            // Track reassignments from inlining of manual memo
            if let Some(memo_state) = &mut state.manual_memo_state
                && lvalue.kind == InstructionKind::Reassign
            {
                let decl_id = state.env.identifiers[lvalue.place.identifier].declaration_id;
                memo_state.reassignments.entry(decl_id).or_default().insert(value.identifier);
            }
        }
        ReactiveValue::Instruction(InstructionValue::LoadLocal { place, .. })
            if let Some(memo_state) = &mut state.manual_memo_state =>
        {
            let place_ident = &state.env.identifiers[place.identifier];
            if let Some(ref lvalue) = instr.lvalue {
                let lvalue_ident = &state.env.identifiers[lvalue.identifier];
                if place_ident.scope.is_some() && lvalue_ident.scope.is_none() {
                    memo_state
                        .reassignments
                        .entry(lvalue_ident.declaration_id)
                        .or_default()
                        .insert(place.identifier);
                }
            }
        }
        _ => {}
    }
}

fn record_unmemoized_error(span: Option<Span>, env: &mut Environment) {
    let diag = ErrorCategory::PreserveManualMemo
        .diagnostic("Existing memoization could not be preserved")
        .with_help(
            "React Compiler has skipped optimizing this component because the existing manual memoization could not be preserved. This value was memoized in source but not in compilation output",
        )
        .with_labels(span.map(|s| s.label("Could not preserve existing memoization")));
    env.record_diagnostic(diag);
}

/// Record temporaries from an instruction.
/// TS: `recordTemporaries`
fn record_temporaries<'h>(instr: &ReactiveInstruction<'h>, state: &mut VisitorState<'_, 'h>) {
    let lvalue = &instr.lvalue;
    let lv_id = lvalue.as_ref().map(|lv| lv.identifier);
    if let Some(id) = lv_id {
        if state.temporaries.contains_key(&id) {
            return;
        }
    }

    if let Some(ref lvalue) = instr.lvalue {
        let lv_ident = &state.env.identifiers[lvalue.identifier];
        if is_named(lv_ident) && state.manual_memo_state.is_some() {
            state.manual_memo_state.as_mut().unwrap().decls.insert(lv_ident.declaration_id);
        }
    }

    // Record deps from the instruction value first (before setting lvalue temporary)
    record_deps_in_value(&instr.value, state);

    // Then set the lvalue temporary (TS always sets this, even for unnamed lvalues)
    if let Some(ref lvalue) = instr.lvalue {
        state.temporaries.insert(
            lvalue.identifier,
            ManualMemoDependency {
                root: ManualMemoDependencyRoot::NamedLocal {
                    value: lvalue.clone(),
                    constant: false,
                },
                path: Vec::new(),
                span: lvalue.span,
            },
        );
    }
}

/// Record dependencies from a reactive value.
/// TS: `recordDepsInValue`
fn record_deps_in_value<'h>(value: &ReactiveValue<'h>, state: &mut VisitorState<'_, 'h>) {
    match value {
        ReactiveValue::SequenceExpression { instructions, value, .. } => {
            for instr in instructions {
                visit_instruction(instr, state);
            }
            record_deps_in_value(value, state);
        }
        ReactiveValue::OptionalExpression { value: inner, .. } => {
            record_deps_in_value(inner, state);
        }
        ReactiveValue::ConditionalExpression { test, consequent, alternate, .. } => {
            record_deps_in_value(test, state);
            record_deps_in_value(consequent, state);
            record_deps_in_value(alternate, state);
        }
        ReactiveValue::LogicalExpression { left, right, .. } => {
            record_deps_in_value(left, state);
            record_deps_in_value(right, state);
        }
        ReactiveValue::Instruction(iv) => {
            // TS: collectMaybeMemoDependencies(value, this.temporaries, false)
            // Called for side-effect of building up the dependency chain through
            // LoadGlobal -> PropertyLoad -> ... The return value is discarded here
            // (only used in DropManualMemoization's caller), but we need to store
            // the result in temporaries for the lvalue of the enclosing instruction.
            // That storage is handled by record_temporaries after this function returns.

            // Track store targets within manual memo blocks
            // TS: if (value.kind === 'StoreLocal' || value.kind === 'StoreContext' || value.kind === 'Destructure')
            match iv {
                InstructionValue::StoreLocal { lvalue, .. }
                | InstructionValue::StoreContext { lvalue, .. } => {
                    if let Some(ref mut memo_state) = state.manual_memo_state {
                        let ident = &state.env.identifiers[lvalue.place.identifier];
                        memo_state.decls.insert(ident.declaration_id);
                        if is_named(ident) {
                            state.temporaries.insert(
                                lvalue.place.identifier,
                                ManualMemoDependency {
                                    root: ManualMemoDependencyRoot::NamedLocal {
                                        value: lvalue.place.clone(),
                                        constant: false,
                                    },
                                    path: Vec::new(),
                                    span: lvalue.place.span,
                                },
                            );
                        }
                    }
                }
                InstructionValue::Destructure { lvalue, .. } => {
                    if let Some(ref mut memo_state) = state.manual_memo_state {
                        for place in destructure_lvalue_places(&lvalue.pattern) {
                            let ident = &state.env.identifiers[place.identifier];
                            memo_state.decls.insert(ident.declaration_id);
                            if is_named(ident) {
                                state.temporaries.insert(
                                    place.identifier,
                                    ManualMemoDependency {
                                        root: ManualMemoDependencyRoot::NamedLocal {
                                            value: place.clone(),
                                            constant: false,
                                        },
                                        path: Vec::new(),
                                        span: place.span,
                                    },
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Get operand places from a StartMemoize instruction's deps.
fn start_memoize_operands(deps: &Option<Vec<ManualMemoDependency>>) -> Vec<Place> {
    let mut result = Vec::new();
    if let Some(deps) = deps {
        for dep in deps {
            if let ManualMemoDependencyRoot::NamedLocal { value, .. } = &dep.root {
                result.push(value.clone());
            }
        }
    }
    result
}

/// Get lvalue places from a Destructure pattern.
fn destructure_lvalue_places<'p>(pattern: &'p Pattern<'_>) -> Vec<&'p Place> {
    let mut result = Vec::new();
    match pattern {
        Pattern::Array(arr) => {
            for item in &arr.items {
                match item {
                    ArrayPatternElement::Place(place) => {
                        result.push(place);
                    }
                    ArrayPatternElement::Spread(spread) => {
                        result.push(&spread.place);
                    }
                    ArrayPatternElement::Hole => {}
                }
            }
        }
        Pattern::Object(obj) => {
            for entry in &obj.properties {
                match entry {
                    ObjectPropertyOrSpread::Property(prop) => {
                        result.push(&prop.place);
                    }
                    ObjectPropertyOrSpread::Spread(spread) => {
                        result.push(&spread.place);
                    }
                }
            }
        }
    }
    result
}

/// Check if an identifier is unmemoized (has a scope that hasn't completed).
fn is_unmemoized(
    id: IdentifierId,
    completed_scopes: &FxHashSet<ScopeId>,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
) -> bool {
    let ident = &identifiers[id];
    if let Some(scope_id) = ident.scope { !completed_scopes.contains(&scope_id) } else { false }
}

// =============================================================================
// Dependency comparison (port of compareDeps / validateInferredDep)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CompareDependencyResult {
    Ok = 0,
    RootDifference = 1,
    PathDifference = 2,
    Subpath = 3,
    RefAccessDifference = 4,
}

fn compare_deps(
    inferred: &ManualMemoDependency,
    source: &ManualMemoDependency,
) -> CompareDependencyResult {
    let roots_equal = match (&inferred.root, &source.root) {
        (
            ManualMemoDependencyRoot::Global { identifier_name: a },
            ManualMemoDependencyRoot::Global { identifier_name: b },
        ) => a == b,
        (
            ManualMemoDependencyRoot::NamedLocal { value: a, .. },
            ManualMemoDependencyRoot::NamedLocal { value: b, .. },
        ) => a.identifier == b.identifier,
        _ => false,
    };
    if !roots_equal {
        return CompareDependencyResult::RootDifference;
    }

    let min_len = inferred.path.len().min(source.path.len());
    let mut is_subpath = true;
    for i in 0..min_len {
        if inferred.path[i].property != source.path[i].property {
            is_subpath = false;
            break;
        } else if inferred.path[i].optional != source.path[i].optional {
            return CompareDependencyResult::PathDifference;
        }
    }

    if is_subpath
        && (source.path.len() == inferred.path.len()
            || (inferred.path.len() >= source.path.len()
                && !inferred.path.iter().any(|t| t.property.is_string("current"))))
    {
        CompareDependencyResult::Ok
    } else if is_subpath {
        if source.path.iter().any(|t| t.property.is_string("current"))
            || inferred.path.iter().any(|t| t.property.is_string("current"))
        {
            CompareDependencyResult::RefAccessDifference
        } else {
            CompareDependencyResult::Subpath
        }
    } else {
        CompareDependencyResult::PathDifference
    }
}

/// Pretty-print a reactive scope dependency (e.g., `x.a.b?.c`)
fn pretty_print_scope_dependency(
    dep_id: IdentifierId,
    dep_path: &[DependencyPathEntry],
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
) -> String {
    let ident = &identifiers[dep_id];
    let root_str = match &ident.name {
        Some(IdentifierName::Named(n) | IdentifierName::Promoted(n)) => n.as_str(),
        None => "[unnamed]",
    };
    let path_str: String = dep_path
        .iter()
        .map(|entry| {
            let prefix = if entry.optional { "?." } else { "." };
            match &entry.property {
                PropertyLiteral::String(s) => format!("{prefix}{s}"),
                PropertyLiteral::Number(n) => format!("{prefix}{n}"),
            }
        })
        .collect();
    format!("{root_str}{path_str}")
}

/// Pretty-print a manual memo dependency for error messages.
fn print_manual_memo_dependency(
    dep: &ManualMemoDependency,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    with_optional: bool,
) -> String {
    let root_str = match &dep.root {
        ManualMemoDependencyRoot::NamedLocal { value, .. } => {
            let ident = &identifiers[value.identifier];
            match &ident.name {
                Some(IdentifierName::Named(n) | IdentifierName::Promoted(n)) => n.as_str(),
                None => "[unnamed]",
            }
        }
        ManualMemoDependencyRoot::Global { identifier_name } => identifier_name.as_str(),
    };
    let path_str: String = dep
        .path
        .iter()
        .map(|entry| {
            let prefix = if with_optional && entry.optional { "?." } else { "." };
            match &entry.property {
                PropertyLiteral::String(s) => format!("{prefix}{s}"),
                PropertyLiteral::Number(n) => format!("{prefix}{n}"),
            }
        })
        .collect();
    format!("{root_str}{path_str}")
}

fn get_compare_dependency_result_description(result: CompareDependencyResult) -> &'static str {
    match result {
        CompareDependencyResult::Ok => "Dependencies equal",
        CompareDependencyResult::RootDifference | CompareDependencyResult::PathDifference => {
            "Inferred different dependency than source"
        }
        CompareDependencyResult::RefAccessDifference => "Differences in ref.current access",
        CompareDependencyResult::Subpath => "Inferred less specific property than source",
    }
}

/// Validate that an inferred dependency matches a source dependency or was produced
/// within the manual memo block.
fn validate_inferred_dep<'h>(
    dep_id: IdentifierId,
    dep_path: &[DependencyPathEntry<'h>],
    temporaries: &FxHashMap<IdentifierId, ManualMemoDependency<'h>>,
    decls_within_memo_block: &FxHashSet<DeclarationId>,
    valid_deps_in_memo_block: &[ManualMemoDependency<'h>],
    env: &mut Environment,
    memo_location: Option<Span>,
) {
    // Normalize the dependency through temporaries
    let normalized_dep = if let Some(temp) = temporaries.get(&dep_id) {
        let mut path = temp.path.clone();
        path.extend_from_slice(dep_path);
        ManualMemoDependency { root: temp.root.clone(), path, span: temp.span }
    } else {
        let ident = &env.identifiers[dep_id];
        // TS: Diagnostics.invariant(dep.identifier.name?.kind === 'named', ...)
        if !is_named(ident) {
            return;
        }
        ManualMemoDependency {
            root: ManualMemoDependencyRoot::NamedLocal {
                value: Place {
                    identifier: dep_id,
                    effect: Effect::Read,
                    reactive: false,
                    span: ident.span,
                },
                constant: false,
            },
            path: dep_path.to_vec(),
            span: ident.span,
        }
    };

    // Check if the dep was declared within the memo block
    if let ManualMemoDependencyRoot::NamedLocal { value, .. } = &normalized_dep.root {
        let ident = &env.identifiers[value.identifier];
        if decls_within_memo_block.contains(&ident.declaration_id) {
            return;
        }
    }

    // Compare against each valid source dependency
    let mut error_diagnostic: Option<CompareDependencyResult> = None;
    for source_dep in valid_deps_in_memo_block {
        let result = compare_deps(&normalized_dep, source_dep);
        if result == CompareDependencyResult::Ok {
            return;
        }
        error_diagnostic = Some(match error_diagnostic {
            Some(prev) => prev.max(result),
            None => result,
        });
    }

    let ident = &env.identifiers[dep_id];

    let extra = if is_named(ident) {
        // Use the original dep_id/dep_path (matching TS prettyPrintScopeDependency(dep))
        let dep_str = pretty_print_scope_dependency(dep_id, dep_path, &env.identifiers);
        let source_deps_str: String = valid_deps_in_memo_block
            .iter()
            .map(|d| print_manual_memo_dependency(d, &env.identifiers, true))
            .collect::<Vec<_>>()
            .join(", ");
        let result_desc = error_diagnostic
            .map(get_compare_dependency_result_description)
            .unwrap_or("Inferred dependency not present in source");
        Some(format!(
            "The inferred dependency was `{}`, but the source dependencies were [{}]. {}",
            dep_str, source_deps_str, result_desc
        ))
    } else {
        None
    };

    let description = format!(
        "React Compiler has skipped optimizing this component because the existing manual memoization could not be preserved. \
         The inferred dependencies did not match the manually specified dependencies, which could cause the value to change more or less frequently than expected. {}",
        extra.as_deref().unwrap_or_default()
    );

    let diag = ErrorCategory::PreserveManualMemo
        .diagnostic("Existing memoization could not be preserved")
        .with_help(description.trim().to_string())
        .with_labels(
            memo_location.map(|s| s.label("Could not preserve existing manual memoization")),
        );
    env.record_diagnostic(diag);
}
