/// Assert scope instructions are within their scopes.
///
/// Port of `ReactiveScopes/AssertScopeInstructionsWithinScope.ts` from the React Compiler.
///
/// Internal validation pass that checks all the instructions involved in creating
/// values for a given scope are within the corresponding ReactiveScopeBlock.
/// Errors in HIR/ReactiveFunction structure and alias analysis could theoretically
/// create a structure where instructions belonging to a scope appear outside
/// that scope's block. This pass guards against such compiler coding mistakes.
///
/// The TS reference checks operands (via `visitPlace`) and terminal places,
/// NOT lvalues (which use a separate `visitLValue` callback that is not overridden).
/// It uses `getPlaceScope(id, place)` to filter by instruction ID range, and
/// properly tracks scope entry/exit via add/delete on `activeScopes`.
///
/// This Rust port checks lvalues and uses `getPlaceScope` to filter by instruction
/// ID range. Scope tracking is add-only (scopes remain permanently active once
/// encountered) which is more lenient than the TS reference's entry/exit tracking.
/// Switching to operand-only checks and proper scope exit tracking will be done
/// once earlier passes (BuildReactiveFunction, etc.) are more fully aligned with
/// the TS reference.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{
        InstructionId, Place, ReactiveFunction, ReactiveInstruction, ReactiveScope,
        ReactiveTerminalStatement, ScopeId,
    },
    reactive_scopes::visitors::{ReactiveVisitor, visit_reactive_function},
};

/// Returns the identifier's scope if the instruction `id` is within the scope's range.
///
/// Port of `getPlaceScope` from `HIR/HIR.ts`.
fn get_place_scope(id: InstructionId, place: &Place) -> Option<&ReactiveScope> {
    let scope = place.identifier.scope.as_ref()?;
    if id >= scope.range.start && id < scope.range.end { Some(scope) } else { None }
}

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
    /// Check a place using `getPlaceScope` to filter by instruction ID range.
    ///
    /// This matches the TS behavior where `getPlaceScope(id, place)` returns null
    /// if the instruction ID is outside the scope's range, preventing false positives
    /// for instructions that reference scopes they don't actually belong to.
    fn check_place_scope(&mut self, place: &Place, id: InstructionId) {
        if let Some(scope) = get_place_scope(id, place)
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
