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
/// Matches the TypeScript reference behavior:
/// - Checks OPERANDS (via `visitPlace` called from `traverseValue`) NOT lvalues.
///   Lvalues use a separate `visitLValue` callback that the TS checker does not override.
/// - Uses proper enter/exit scope tracking (add on enter, delete on exit via
///   `visit_scope_block` + `exit_scope_block`), matching TS `traverseScope` semantics.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{
        InstructionId, Place, ReactiveFunction, ReactiveInstruction, ReactiveValue, ScopeId,
        visitors::each_instruction_value_operand,
    },
    reactive_scopes::visitors::{ReactiveVisitor, visit_reactive_function},
};

/// Returns the identifier's scope if the instruction `id` is within the scope's range.
///
/// Port of `getPlaceScope` from `HIR/HIR.ts`.
fn get_place_scope(id: InstructionId, place: &Place) -> Option<&crate::hir::ReactiveScope> {
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
    /// Check a single place (operand) using `getPlaceScope` to filter by instruction ID range.
    ///
    /// Matches the TS `visitPlace` behavior: checks operands (reads), not lvalues.
    fn check_place(&mut self, place: &Place, id: InstructionId) {
        if self.error.is_some() {
            return;
        }
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

    /// Collect and check all Place operands from a ReactiveValue, recursively.
    ///
    /// Matches TS `traverseValue` which recurses into compound values (Logical,
    /// Ternary, Sequence, OptionalCall) and calls `visitPlace` on each operand.
    fn check_reactive_value(&mut self, value: &ReactiveValue, id: InstructionId) {
        if self.error.is_some() {
            return;
        }
        match value {
            ReactiveValue::Instruction(iv) => {
                for operand in each_instruction_value_operand(iv) {
                    self.check_place(operand, id);
                    if self.error.is_some() {
                        return;
                    }
                }
            }
            ReactiveValue::Logical(logical) => {
                self.check_reactive_value(&logical.left, id);
                self.check_reactive_value(&logical.right, id);
            }
            ReactiveValue::Ternary(ternary) => {
                self.check_reactive_value(&ternary.test, id);
                self.check_reactive_value(&ternary.consequent, id);
                self.check_reactive_value(&ternary.alternate, id);
            }
            ReactiveValue::Sequence(seq) => {
                // Visit inner instructions first (matching TS `traverseValue` SequenceExpression)
                for inner_instr in &seq.instructions {
                    self.check_instruction_operands(inner_instr);
                    if self.error.is_some() {
                        return;
                    }
                }
                // Then check the final value
                self.check_reactive_value(&seq.value, seq.id);
            }
            ReactiveValue::OptionalCall(opt) => {
                self.check_reactive_value(&opt.value, opt.id);
            }
        }
    }

    /// Check all operands of an instruction's value.
    fn check_instruction_operands(&mut self, instr: &ReactiveInstruction) {
        self.check_reactive_value(&instr.value, instr.id);
    }
}

impl ReactiveVisitor for ScopeChecker {
    fn visit_instruction(&mut self, instr: &ReactiveInstruction) {
        if self.error.is_some() {
            return;
        }
        // Check operands (reads), matching TS `visitPlace` behavior.
        // The TS reference does NOT check lvalues (no `visitLValue` override).
        self.check_instruction_operands(instr);
    }

    fn visit_scope_block(&mut self, scope: &crate::hir::ReactiveScope) {
        // Add on entry — matching TS `this.activeScopes.add(block.scope.id)`.
        self.active_scopes.insert(scope.id);
    }

    fn exit_scope_block(&mut self, scope: &crate::hir::ReactiveScope) {
        // Delete on exit — matching TS `this.activeScopes.delete(block.scope.id)`.
        self.active_scopes.remove(&scope.id);
    }
}
