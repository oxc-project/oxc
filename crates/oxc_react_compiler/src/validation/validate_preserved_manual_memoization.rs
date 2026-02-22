/// Validate preserved manual memoization.
///
/// Port of `Validation/ValidatePreservedManualMemoization.ts` from the React Compiler.
///
/// Validates that all explicit manual memoization (useMemo/useCallback) was
/// accurately preserved, and that no originally memoized values became
/// unmemoized in the output.
///
/// This can occur if a value's mutable range somehow extended to include a
/// hook and was pruned.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory,
        GENERATED_SOURCE, SourceLocation,
    },
    hir::{
        DeclarationId, IdentifierId, IdentifierName, InstructionValue, ManualMemoDependency,
        ManualMemoDependencyRoot, PrunedReactiveScopeBlock, ReactiveBlock, ReactiveFunction,
        ReactiveInstruction, ReactiveScopeBlock, ReactiveScopeDependency, ReactiveStatement,
        ReactiveTerminal, ReactiveValue, ScopeId,
        visitors::{each_instruction_value_lvalue, each_instruction_value_operand},
    },
    inference::drop_manual_memoization::collect_maybe_memo_dependencies,
};

/// Validate that manual memoization is preserved by the compiler.
///
/// # Errors
/// Returns a `CompilerError` if manual memoization would be degraded.
pub fn validate_preserved_manual_memoization(func: &ReactiveFunction) -> Result<(), CompilerError> {
    let mut state = VisitorState {
        errors: CompilerError::new(),
        manual_memo_state: None,
        scopes: FxHashSet::default(),
        pruned_scopes: FxHashSet::default(),
        temporaries: FxHashMap::default(),
    };

    visit_block(&func.body, &mut state);

    state.errors.into_result()
}

struct ManualMemoBlockState {
    /// Tracks reassigned temporaries for inlined useMemo.
    reassignments: FxHashMap<DeclarationId, FxHashSet<IdentifierId>>,
    /// The source location of the original memoization.
    loc: SourceLocation,
    /// Values produced within manual memoization blocks.
    decls: FxHashSet<DeclarationId>,
    /// Normalized deps list from useMemo/useCallback callsite in source.
    deps_from_source: Option<Vec<ManualMemoDependency>>,
    /// The manual memo ID from StartMemoize.
    manual_memo_id: u32,
}

struct VisitorState {
    errors: CompilerError,
    manual_memo_state: Option<ManualMemoBlockState>,
    scopes: FxHashSet<ScopeId>,
    pruned_scopes: FxHashSet<ScopeId>,
    temporaries: FxHashMap<IdentifierId, ManualMemoDependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareDependencyResult {
    Ok,
    RootDifference,
    PathDifference,
    Subpath,
    RefAccessDifference,
}

impl CompareDependencyResult {
    fn merge(self, other: Self) -> Self {
        // Higher priority = more severe
        let a = self.priority();
        let b = other.priority();
        if a >= b { self } else { other }
    }

    fn priority(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::RootDifference => 1,
            Self::PathDifference => 2,
            Self::Subpath => 3,
            Self::RefAccessDifference => 4,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Ok => "Dependencies equal",
            Self::RootDifference | Self::PathDifference => {
                "Inferred different dependency than source"
            }
            Self::RefAccessDifference => "Differences in ref.current access",
            Self::Subpath => "Inferred less specific property than source",
        }
    }
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
        ) => a.identifier.id == b.identifier.id,
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
                && !inferred.path.iter().any(|t| t.property.to_string() == "current")))
    {
        CompareDependencyResult::Ok
    } else if is_subpath {
        if source.path.iter().any(|t| t.property.to_string() == "current")
            || inferred.path.iter().any(|t| t.property.to_string() == "current")
        {
            CompareDependencyResult::RefAccessDifference
        } else {
            CompareDependencyResult::Subpath
        }
    } else {
        CompareDependencyResult::PathDifference
    }
}

/// Validate that an inferred dependency matches a source dependency or is
/// produced by earlier instructions in the same manual memoization block.
fn validate_inferred_dep(
    dep: &ReactiveScopeDependency,
    temporaries: &FxHashMap<IdentifierId, ManualMemoDependency>,
    decls_within_memo_block: &FxHashSet<DeclarationId>,
    valid_deps_in_memo_block: &[ManualMemoDependency],
    error_state: &mut CompilerError,
    memo_location: SourceLocation,
) {
    let normalized_dep: ManualMemoDependency;
    if let Some(maybe_normalized_root) = temporaries.get(&dep.identifier_id) {
        let mut path = maybe_normalized_root.path.clone();
        path.extend(dep.path.iter().cloned());
        normalized_dep = ManualMemoDependency {
            root: maybe_normalized_root.root.clone(),
            path,
            loc: maybe_normalized_root.loc,
        };
    } else {
        // Build a NamedLocal dependency from the scope dependency
        // We need to fabricate a Place here since we only have identifier_id
        normalized_dep = ManualMemoDependency {
            root: ManualMemoDependencyRoot::Global {
                identifier_name: format!("$id{}", dep.identifier_id.0),
            },
            path: dep.path.clone(),
            loc: GENERATED_SOURCE,
        };
    }

    // Check if this dependency was declared within the memo block
    if let ManualMemoDependencyRoot::NamedLocal { value, .. } = &normalized_dep.root
        && decls_within_memo_block.contains(&value.identifier.declaration_id)
    {
        return;
    }

    let mut error_diagnostic: Option<CompareDependencyResult> = None;
    for original_dep in valid_deps_in_memo_block {
        let compare_result = compare_deps(&normalized_dep, original_dep);
        if compare_result == CompareDependencyResult::Ok {
            return;
        }
        error_diagnostic = Some(match error_diagnostic {
            Some(existing) => existing.merge(compare_result),
            None => compare_result,
        });
    }

    let description = format!(
        "React Compiler has skipped optimizing this component because the existing manual \
         memoization could not be preserved. The inferred dependencies did not match the \
         manually specified dependencies, which could cause the value to change more or \
         less frequently than expected. {}",
        error_diagnostic.map_or(
            "Inferred dependency not present in source",
            CompareDependencyResult::description
        )
    );

    error_state.push_diagnostic(
        CompilerDiagnostic::create(
            ErrorCategory::PreserveManualMemo,
            "Existing memoization could not be preserved".to_string(),
            Some(description),
            None,
        )
        .with_detail(CompilerDiagnosticDetail::Error {
            loc: Some(memo_location),
            message: Some("Could not preserve existing manual memoization".to_string()),
        }),
    );
}

fn visit_block(block: &ReactiveBlock, state: &mut VisitorState) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(instr) => {
                visit_instruction(&instr.instruction, state);
            }
            ReactiveStatement::Terminal(term) => {
                visit_terminal(&term.terminal, state);
            }
            ReactiveStatement::Scope(scope) => {
                visit_scope(scope, state);
            }
            ReactiveStatement::PrunedScope(scope) => {
                visit_pruned_scope(scope, state);
            }
        }
    }
}

fn visit_scope(scope_block: &ReactiveScopeBlock, state: &mut VisitorState) {
    // First traverse the scope contents
    visit_block(&scope_block.instructions, state);

    // Then validate dependencies against manual memo deps
    if let Some(ref memo_state) = state.manual_memo_state
        && let Some(ref deps_from_source) = memo_state.deps_from_source
    {
        for dep in &scope_block.scope.dependencies {
            validate_inferred_dep(
                dep,
                &state.temporaries,
                &memo_state.decls,
                deps_from_source,
                &mut state.errors,
                memo_state.loc,
            );
        }
    }

    // Record this scope as completed
    state.scopes.insert(scope_block.scope.id);
    for id in &scope_block.scope.merged {
        state.scopes.insert(*id);
    }
}

fn visit_pruned_scope(scope_block: &PrunedReactiveScopeBlock, state: &mut VisitorState) {
    visit_block(&scope_block.instructions, state);
    state.pruned_scopes.insert(scope_block.scope.id);
}

fn record_deps_in_value(value: &ReactiveValue, state: &mut VisitorState) {
    match value {
        ReactiveValue::Sequence(seq) => {
            for instr in &seq.instructions {
                visit_instruction(instr, state);
            }
            record_deps_in_value(&seq.value, state);
        }
        ReactiveValue::Ternary(ternary) => {
            record_deps_in_value(&ternary.test, state);
            record_deps_in_value(&ternary.consequent, state);
            record_deps_in_value(&ternary.alternate, state);
        }
        ReactiveValue::Logical(logical) => {
            record_deps_in_value(&logical.left, state);
            record_deps_in_value(&logical.right, state);
        }
        ReactiveValue::OptionalCall(opt) => {
            record_deps_in_value(&opt.value, state);
        }
        ReactiveValue::Instruction(instr_value) => {
            // Collect memo dependencies from the instruction value
            collect_maybe_memo_dependencies(instr_value, &mut state.temporaries, false);

            // Track lvalues for store/context/destructure instructions
            match instr_value.as_ref() {
                InstructionValue::StoreLocal(_)
                | InstructionValue::StoreContext(_)
                | InstructionValue::Destructure(_) => {
                    for store_target in each_instruction_value_lvalue(instr_value) {
                        if let Some(IdentifierName::Named(_)) = &store_target.identifier.name {
                            state.temporaries.insert(
                                store_target.identifier.id,
                                ManualMemoDependency {
                                    root: ManualMemoDependencyRoot::NamedLocal {
                                        value: store_target.clone(),
                                        constant: false,
                                    },
                                    path: Vec::new(),
                                    loc: store_target.loc,
                                },
                            );
                        }
                    }
                    // Track declarations for memo state
                    if let Some(ref mut memo_state) = state.manual_memo_state {
                        for store_target in each_instruction_value_lvalue(instr_value) {
                            memo_state.decls.insert(store_target.identifier.declaration_id);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn visit_instruction(instruction: &ReactiveInstruction, state: &mut VisitorState) {
    let lvalue = &instruction.lvalue;
    let value = &instruction.value;

    // Record temporaries
    if let Some(lval) = lvalue {
        let lval_id = lval.identifier.id;
        if state.temporaries.contains_key(&lval_id) {
            // Already recorded
        } else {
            let is_named_local = matches!(&lval.identifier.name, Some(IdentifierName::Named(_)));
            if is_named_local && let Some(ref mut memo_state) = state.manual_memo_state {
                memo_state.decls.insert(lval.identifier.declaration_id);
            }
        }
    }

    record_deps_in_value(value, state);

    if let Some(lval) = lvalue {
        state.temporaries.insert(
            lval.identifier.id,
            ManualMemoDependency {
                root: ManualMemoDependencyRoot::NamedLocal { value: lval.clone(), constant: false },
                path: Vec::new(),
                loc: lval.loc,
            },
        );
    }

    // Handle StartMemoize/FinishMemoize
    if let ReactiveValue::Instruction(instr_value) = value {
        match instr_value.as_ref() {
            InstructionValue::StartMemoize(v) => {
                let deps_from_source = v.deps.clone();

                // Check scope dependencies of operands
                for operand in each_instruction_value_operand(instr_value) {
                    if let Some(ref scope) = operand.identifier.scope
                        && !state.scopes.contains(&scope.id)
                        && !state.pruned_scopes.contains(&scope.id)
                    {
                        state.errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::PreserveManualMemo,
                                "Existing memoization could not be preserved".to_string(),
                                Some(
                                    "React Compiler has skipped optimizing this component \
                                         because the existing manual memoization could not be \
                                         preserved. This dependency may be mutated later, which \
                                         could cause the value to change unexpectedly"
                                        .to_string(),
                                ),
                                None,
                            )
                            .with_detail(
                                CompilerDiagnosticDetail::Error {
                                    loc: Some(operand.loc),
                                    message: Some(
                                        "This dependency may be modified later".to_string(),
                                    ),
                                },
                            ),
                        );
                    }
                }

                state.manual_memo_state = Some(ManualMemoBlockState {
                    loc: instruction.loc,
                    decls: FxHashSet::default(),
                    deps_from_source,
                    manual_memo_id: v.manual_memo_id,
                    reassignments: FxHashMap::default(),
                });
            }
            InstructionValue::FinishMemoize(v) => {
                if let Some(memo_state) = state.manual_memo_state.take() {
                    // Verify matching StartMemoize/FinishMemoize
                    debug_assert_eq!(memo_state.manual_memo_id, v.manual_memo_id);
                    if !v.pruned {
                        for operand in each_instruction_value_operand(instr_value) {
                            let decls: Vec<IdentifierId> = if operand.identifier.scope.is_none() {
                                // If the manual memo was inlined, check reassignments
                                memo_state
                                    .reassignments
                                    .get(&operand.identifier.declaration_id)
                                    .map_or_else(
                                        || vec![operand.identifier.id],
                                        |ids| ids.iter().copied().collect(),
                                    )
                            } else {
                                vec![operand.identifier.id]
                            };

                            if !decls.is_empty() {
                                // In the TS version, each decl would look up its own
                                // scope. We approximate by checking the operand's scope.
                                if let Some(ref scope) = operand.identifier.scope
                                    && !state.scopes.contains(&scope.id)
                                {
                                    state.errors.push_diagnostic(
                                            CompilerDiagnostic::create(
                                                ErrorCategory::PreserveManualMemo,
                                                "Existing memoization could not be preserved"
                                                    .to_string(),
                                                Some(
                                                    "React Compiler has skipped optimizing this \
                                                     component because the existing manual \
                                                     memoization could not be preserved. This value \
                                                     was memoized in source but not in compilation \
                                                     output"
                                                        .to_string(),
                                                ),
                                                None,
                                            )
                                            .with_detail(CompilerDiagnosticDetail::Error {
                                                loc: Some(operand.loc),
                                                message: Some(
                                                    "Could not preserve existing memoization"
                                                        .to_string(),
                                                ),
                                            }),
                                        );
                                }
                            }
                        }
                    }
                }
            }
            InstructionValue::StoreLocal(v) => {
                if v.lvalue.kind == crate::hir::InstructionKind::Reassign
                    && let Some(ref mut memo_state) = state.manual_memo_state
                {
                    memo_state
                        .reassignments
                        .entry(v.lvalue.place.identifier.declaration_id)
                        .or_default()
                        .insert(v.value.identifier.id);
                }
            }
            InstructionValue::LoadLocal(v) => {
                if v.place.identifier.scope.is_some()
                    && let Some(lval) = lvalue
                    && lval.identifier.scope.is_none()
                    && let Some(ref mut memo_state) = state.manual_memo_state
                {
                    memo_state
                        .reassignments
                        .entry(lval.identifier.declaration_id)
                        .or_default()
                        .insert(v.place.identifier.id);
                }
            }
            _ => {}
        }
    }
}

fn visit_terminal(terminal: &ReactiveTerminal, state: &mut VisitorState) {
    match terminal {
        ReactiveTerminal::If(t) => {
            visit_block(&t.consequent, state);
            if let Some(ref alt) = t.alternate {
                visit_block(alt, state);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(ref block) = case.block {
                    visit_block(block, state);
                }
            }
        }
        ReactiveTerminal::While(t) => visit_block(&t.r#loop, state),
        ReactiveTerminal::DoWhile(t) => visit_block(&t.r#loop, state),
        ReactiveTerminal::For(t) => visit_block(&t.r#loop, state),
        ReactiveTerminal::ForOf(t) => visit_block(&t.r#loop, state),
        ReactiveTerminal::ForIn(t) => visit_block(&t.r#loop, state),
        ReactiveTerminal::Label(t) => visit_block(&t.block, state),
        ReactiveTerminal::Try(t) => {
            visit_block(&t.block, state);
            visit_block(&t.handler, state);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}
