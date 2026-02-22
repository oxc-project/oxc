/// Assert scope instructions are within their scopes.
///
/// Port of `ReactiveScopes/AssertScopeInstructionsWithinScope.ts` from the React Compiler.
///
/// Internal validation pass that checks all the instructions involved in creating
/// values for a given scope are within the corresponding ReactiveScopeBlock.
/// Errors in HIR/ReactiveFunction structure and alias analysis could theoretically
/// create a structure where instructions belonging to a scope appear outside
/// that scope's block. This pass guards against such compiler coding mistakes.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{ReactiveFunction, ReactiveInstruction, ReactiveTerminalStatement, ScopeId},
    reactive_scopes::visitors::{ReactiveVisitor, visit_reactive_function},
};

/// Assert that all scope instructions are within their corresponding scopes.
///
/// # Errors
/// Returns a `CompilerError` if any instruction is outside its scope.
pub fn assert_scope_instructions_within_scopes(
    func: &ReactiveFunction,
) -> Result<(), CompilerError> {
    // First pass: collect all existing scope IDs
    let mut scope_collector = ScopeCollector { existing_scopes: FxHashSet::default() };
    visit_reactive_function(func, &mut scope_collector);

    // Second pass: check instructions against scopes
    let mut checker = ScopeChecker {
        existing_scopes: scope_collector.existing_scopes,
        active_scopes: FxHashSet::default(),
        error: None,
    };
    visit_reactive_function(func, &mut checker);

    if let Some(err) = checker.error {
        return Err(err);
    }
    Ok(())
}

struct ScopeCollector {
    existing_scopes: FxHashSet<ScopeId>,
}

impl ReactiveVisitor for ScopeCollector {
    fn visit_scope_block(&mut self, scope: &crate::hir::ReactiveScope) {
        self.existing_scopes.insert(scope.id);
    }
}

struct ScopeChecker {
    existing_scopes: FxHashSet<ScopeId>,
    active_scopes: FxHashSet<ScopeId>,
    error: Option<CompilerError>,
}

impl ScopeChecker {
    fn check_place_scope(&mut self, place: &crate::hir::Place, id: crate::hir::InstructionId) {
        if let Some(ref scope) = place.identifier.scope
            && self.existing_scopes.contains(&scope.id)
            && !self.active_scopes.contains(&scope.id)
        {
            self.error = Some(CompilerError::invariant(
                "Encountered an instruction that should be part of a scope, but where that scope has already completed",
                Some(&format!(
                    "Instruction [{}] is part of scope @{}, but that scope has already completed",
                    id.0, scope.id.0
                )),
                GENERATED_SOURCE,
            ));
        }
    }
}

impl ReactiveVisitor for ScopeChecker {
    fn visit_instruction(&mut self, instr: &ReactiveInstruction) {
        if self.error.is_some() {
            return;
        }
        if let Some(ref lvalue) = instr.lvalue {
            self.check_place_scope(lvalue, instr.id);
        }
    }

    fn visit_terminal(&mut self, _stmt: &ReactiveTerminalStatement) {
        // Terminals are checked through their children
    }

    fn visit_scope_block(&mut self, scope: &crate::hir::ReactiveScope) {
        self.active_scopes.insert(scope.id);
    }
}
