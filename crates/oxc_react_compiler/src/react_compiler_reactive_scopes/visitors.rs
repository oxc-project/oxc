// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Visitor and transform traits for ReactiveFunction.
//!
//! Corresponds to `src/ReactiveScopes/visitors.ts` in the TypeScript compiler.

use oxc_allocator::CloneIn;
use oxc_diagnostics::OxcDiagnostic;

use crate::react_compiler_hir::visitors::{
    each_instruction_value_lvalue, each_instruction_value_operand, each_terminal_operand,
};
use crate::react_compiler_hir::{
    EvaluationOrder, FunctionId, InstructionValue, ParamPattern, Place, PrunedReactiveScopeBlock,
    ReactiveBlock, ReactiveFunction, ReactiveInstruction, ReactiveScopeBlock, ReactiveStatement,
    ReactiveTerminal, ReactiveTerminalStatement, ReactiveValue, environment::Environment,
};

// =============================================================================
// ReactiveFunctionVisitor trait
// =============================================================================

/// Visitor trait for walking a ReactiveFunction tree.
///
/// Override individual `visit_*` methods to customize behavior; call the
/// corresponding `traverse_*` to continue the default recursion.
///
/// TS: `class ReactiveFunctionVisitor<TState>`
pub trait ReactiveFunctionVisitor<'a> {
    type State;

    /// Provide Environment access. The default traversal uses this to include
    /// FunctionExpression/ObjectMethod context places as operands (matching the
    /// TS `eachInstructionValueOperand` behavior).
    fn env(&self) -> &Environment<'a>;

    fn visit_id(&self, _id: EvaluationOrder, _state: &mut Self::State) {}

    fn visit_place(&self, _id: EvaluationOrder, _place: &Place, _state: &mut Self::State) {}

    fn visit_lvalue(&self, _id: EvaluationOrder, _lvalue: &Place, _state: &mut Self::State) {}

    fn visit_param(&self, _place: &Place, _state: &mut Self::State) {}

    /// Walk an inner HIR function, visiting params, instructions (with lvalues,
    /// value-lvalues, operands, and nested functions), and terminal operands.
    /// TS: `visitHirFunction`
    fn visit_hir_function(&self, func_id: FunctionId, state: &mut Self::State) {
        let inner_func = &self.env().functions[func_id];
        for param in &inner_func.params {
            let place = match param {
                ParamPattern::Place(p) => p,
                ParamPattern::Spread(s) => &s.place,
            };
            self.visit_param(place, state);
        }
        let block_ids: Vec<_> = inner_func.body.blocks.keys().copied().collect();
        for block_id in block_ids {
            let inner_func = &self.env().functions[func_id];
            let block = &inner_func.body.blocks[&block_id];
            let instr_ids: Vec<_> = block.instructions.iter().copied().collect();
            let terminal_operands = each_terminal_operand(&block.terminal);
            let terminal_id = block.terminal.evaluation_order();

            for instr_id in &instr_ids {
                let inner_func = &self.env().functions[func_id];
                let instr = &inner_func.instructions[instr_id.index()];
                // Build a temporary ReactiveInstruction for the visitor
                let reactive_instr = ReactiveInstruction {
                    id: instr.id,
                    lvalue: Some(instr.lvalue),
                    value: ReactiveValue::Instruction(instr.value.clone_in(self.env().allocator)),
                    span: instr.span,
                };
                self.visit_instruction(&reactive_instr, state);
                // Recurse into nested functions
                match &instr.value {
                    InstructionValue::FunctionExpression { lowered_func, .. }
                    | InstructionValue::ObjectMethod { lowered_func, .. } => {
                        self.visit_hir_function(lowered_func.func, state);
                    }
                    _ => {}
                }
            }
            for operand in &terminal_operands {
                self.visit_place(terminal_id, operand, state);
            }
        }
    }

    fn visit_value(&self, id: EvaluationOrder, value: &ReactiveValue<'a>, state: &mut Self::State) {
        self.traverse_value(id, value, state);
    }

    fn traverse_value(
        &self,
        id: EvaluationOrder,
        value: &ReactiveValue<'a>,
        state: &mut Self::State,
    ) {
        match value {
            ReactiveValue::OptionalExpression { value: inner, .. } => {
                self.visit_value(id, inner, state);
            }
            ReactiveValue::LogicalExpression { left, right, .. } => {
                self.visit_value(id, left, state);
                self.visit_value(id, right, state);
            }
            ReactiveValue::ConditionalExpression { test, consequent, alternate, .. } => {
                self.visit_value(id, test, state);
                self.visit_value(id, consequent, state);
                self.visit_value(id, alternate, state);
            }
            ReactiveValue::SequenceExpression {
                instructions, id: seq_id, value: inner, ..
            } => {
                for instr in instructions {
                    self.visit_instruction(instr, state);
                }
                self.visit_value(*seq_id, inner, state);
            }
            ReactiveValue::Instruction(instr_value) => {
                let operands = each_instruction_value_operand(instr_value, self.env());
                for place in &operands {
                    self.visit_place(id, place, state);
                }
            }
        }
    }

    fn visit_instruction(&self, instruction: &ReactiveInstruction<'a>, state: &mut Self::State) {
        self.traverse_instruction(instruction, state);
    }

    fn traverse_instruction(&self, instruction: &ReactiveInstruction<'a>, state: &mut Self::State) {
        self.visit_id(instruction.id, state);
        // Visit instruction-level lvalue
        if let Some(lvalue) = &instruction.lvalue {
            self.visit_lvalue(instruction.id, lvalue, state);
        }
        // Visit value-level lvalues (TS: eachInstructionValueLValue)
        if let ReactiveValue::Instruction(iv) = &instruction.value {
            for place in each_instruction_value_lvalue(iv) {
                self.visit_lvalue(instruction.id, &place, state);
            }
        }
        self.visit_value(instruction.id, &instruction.value, state);
    }

    fn visit_terminal(&self, stmt: &ReactiveTerminalStatement<'a>, state: &mut Self::State) {
        self.traverse_terminal(stmt, state);
    }

    fn traverse_terminal(&self, stmt: &ReactiveTerminalStatement<'a>, state: &mut Self::State) {
        let terminal = &stmt.terminal;
        let id = terminal_id(terminal);
        self.visit_id(id, state);
        match terminal {
            ReactiveTerminal::Break { .. } | ReactiveTerminal::Continue { .. } => {}
            ReactiveTerminal::Return { value, id, .. } => {
                self.visit_place(*id, value, state);
            }
            ReactiveTerminal::Throw { value, id, .. } => {
                self.visit_place(*id, value, state);
            }
            ReactiveTerminal::For { init, test, update, loop_block, id, .. } => {
                self.visit_value(*id, init, state);
                self.visit_value(*id, test, state);
                self.visit_block(loop_block, state);
                if let Some(update) = update {
                    self.visit_value(*id, update, state);
                }
            }
            ReactiveTerminal::ForOf { init, test, loop_block, id, .. } => {
                self.visit_value(*id, init, state);
                self.visit_value(*id, test, state);
                self.visit_block(loop_block, state);
            }
            ReactiveTerminal::ForIn { init, loop_block, id, .. } => {
                self.visit_value(*id, init, state);
                self.visit_block(loop_block, state);
            }
            ReactiveTerminal::DoWhile { loop_block, test, id, .. } => {
                self.visit_block(loop_block, state);
                self.visit_value(*id, test, state);
            }
            ReactiveTerminal::While { test, loop_block, id, .. } => {
                self.visit_value(*id, test, state);
                self.visit_block(loop_block, state);
            }
            ReactiveTerminal::If { test, consequent, alternate, id, .. } => {
                self.visit_place(*id, test, state);
                self.visit_block(consequent, state);
                if let Some(alt) = alternate {
                    self.visit_block(alt, state);
                }
            }
            ReactiveTerminal::Switch { test, cases, id, .. } => {
                self.visit_place(*id, test, state);
                for case in cases {
                    if let Some(t) = &case.test {
                        self.visit_place(*id, t, state);
                    }
                    if let Some(block) = &case.block {
                        self.visit_block(block, state);
                    }
                }
            }
            ReactiveTerminal::Label { block, .. } => {
                self.visit_block(block, state);
            }
            ReactiveTerminal::Try { block, handler_binding, handler, id, .. } => {
                self.visit_block(block, state);
                if let Some(binding) = handler_binding {
                    self.visit_place(*id, binding, state);
                }
                self.visit_block(handler, state);
            }
        }
    }

    fn visit_scope(&self, scope: &ReactiveScopeBlock<'a>, state: &mut Self::State) {
        self.traverse_scope(scope, state);
    }

    fn traverse_scope(&self, scope: &ReactiveScopeBlock<'a>, state: &mut Self::State) {
        self.visit_block(&scope.instructions, state);
    }

    fn visit_pruned_scope(&self, scope: &PrunedReactiveScopeBlock<'a>, state: &mut Self::State) {
        self.traverse_pruned_scope(scope, state);
    }

    fn traverse_pruned_scope(&self, scope: &PrunedReactiveScopeBlock<'a>, state: &mut Self::State) {
        self.visit_block(&scope.instructions, state);
    }

    fn visit_block(&self, block: &ReactiveBlock<'a>, state: &mut Self::State) {
        self.traverse_block(block, state);
    }

    fn traverse_block(&self, block: &ReactiveBlock<'a>, state: &mut Self::State) {
        for stmt in block {
            match stmt {
                ReactiveStatement::Instruction(instr) => {
                    self.visit_instruction(instr, state);
                }
                ReactiveStatement::Scope(scope) => {
                    self.visit_scope(scope, state);
                }
                ReactiveStatement::PrunedScope(scope) => {
                    self.visit_pruned_scope(scope, state);
                }
                ReactiveStatement::Terminal(terminal) => {
                    self.visit_terminal(terminal, state);
                }
            }
        }
    }
}

/// Entry point for visiting a reactive function.
/// TS: `visitReactiveFunction`
pub fn visit_reactive_function<'a, V: ReactiveFunctionVisitor<'a>>(
    func: &ReactiveFunction<'a>,
    visitor: &V,
    state: &mut V::State,
) {
    visitor.visit_block(&func.body, state);
}

// =============================================================================
// Transformed enum
// =============================================================================

/// Result of transforming a ReactiveStatement.
/// TS: `Transformed<T>`
pub enum Transformed<T> {
    Keep,
    Remove,
    Replace(T),
    ReplaceMany(Vec<T>),
}

// =============================================================================
// ReactiveFunctionTransform trait
// =============================================================================

/// Transform trait for modifying a ReactiveFunction tree in-place.
///
/// Extends the visitor pattern with `transform_*` methods that can modify
/// or remove statements. The `traverse_block` implementation handles applying
/// transform results to the block.
///
/// TS: `class ReactiveFunctionTransform<TState>`
pub trait ReactiveFunctionTransform<'a> {
    type State;

    /// Provide Environment access. The default traversal uses this to include
    /// FunctionExpression/ObjectMethod context places as operands (matching the
    /// TS `eachInstructionValueOperand` behavior).
    fn env(&self) -> &Environment<'a>;

    fn visit_id(
        &mut self,
        _id: EvaluationOrder,
        _state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        Ok(())
    }

    fn visit_place(
        &mut self,
        _id: EvaluationOrder,
        _place: &Place,
        _state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        Ok(())
    }

    fn visit_lvalue(
        &mut self,
        _id: EvaluationOrder,
        _lvalue: &Place,
        _state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        Ok(())
    }

    fn visit_value(
        &mut self,
        id: EvaluationOrder,
        value: &mut ReactiveValue<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.traverse_value(id, value, state)
    }

    fn traverse_value(
        &mut self,
        id: EvaluationOrder,
        value: &mut ReactiveValue<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        match value {
            ReactiveValue::OptionalExpression { value: inner, .. } => {
                self.visit_value(id, inner, state)?;
            }
            ReactiveValue::LogicalExpression { left, right, .. } => {
                self.visit_value(id, left, state)?;
                self.visit_value(id, right, state)?;
            }
            ReactiveValue::ConditionalExpression { test, consequent, alternate, .. } => {
                self.visit_value(id, test, state)?;
                self.visit_value(id, consequent, state)?;
                self.visit_value(id, alternate, state)?;
            }
            ReactiveValue::SequenceExpression {
                instructions, id: seq_id, value: inner, ..
            } => {
                let seq_id = *seq_id;
                for instr in instructions.iter_mut() {
                    self.visit_instruction(instr, state)?;
                }
                self.visit_value(seq_id, inner, state)?;
            }
            ReactiveValue::Instruction(instr_value) => {
                // Collect operands before visiting to avoid borrow conflict
                // (self.env() borrows self immutably, self.visit_place() needs &mut self).
                let operands = each_instruction_value_operand(instr_value, self.env());
                for place in &operands {
                    self.visit_place(id, place, state)?;
                }
            }
        }
        Ok(())
    }

    fn visit_instruction(
        &mut self,
        instruction: &mut ReactiveInstruction<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.traverse_instruction(instruction, state)
    }

    fn traverse_instruction(
        &mut self,
        instruction: &mut ReactiveInstruction<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.visit_id(instruction.id, state)?;
        // Visit instruction-level lvalue
        if let Some(lvalue) = &instruction.lvalue {
            self.visit_lvalue(instruction.id, lvalue, state)?;
        }
        // Visit value-level lvalues (TS: eachInstructionValueLValue)
        if let ReactiveValue::Instruction(iv) = &instruction.value {
            for place in each_instruction_value_lvalue(iv) {
                self.visit_lvalue(instruction.id, &place, state)?;
            }
        }
        self.visit_value(instruction.id, &mut instruction.value, state)?;
        Ok(())
    }

    fn visit_terminal(
        &mut self,
        stmt: &mut ReactiveTerminalStatement<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.traverse_terminal(stmt, state)
    }

    fn traverse_terminal(
        &mut self,
        stmt: &mut ReactiveTerminalStatement<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        let terminal = &mut stmt.terminal;
        let id = terminal_id(terminal);
        self.visit_id(id, state)?;
        match terminal {
            ReactiveTerminal::Break { .. } | ReactiveTerminal::Continue { .. } => {}
            ReactiveTerminal::Return { value, id, .. } => {
                self.visit_place(*id, value, state)?;
            }
            ReactiveTerminal::Throw { value, id, .. } => {
                self.visit_place(*id, value, state)?;
            }
            ReactiveTerminal::For { init, test, update, loop_block, id, .. } => {
                let id = *id;
                self.visit_value(id, init, state)?;
                self.visit_value(id, test, state)?;
                if let Some(update) = update {
                    self.visit_value(id, update, state)?;
                }
                self.visit_block(loop_block, state)?;
            }
            ReactiveTerminal::ForOf { init, test, loop_block, id, .. } => {
                let id = *id;
                self.visit_value(id, init, state)?;
                self.visit_value(id, test, state)?;
                self.visit_block(loop_block, state)?;
            }
            ReactiveTerminal::ForIn { init, loop_block, id, .. } => {
                let id = *id;
                self.visit_value(id, init, state)?;
                self.visit_block(loop_block, state)?;
            }
            ReactiveTerminal::DoWhile { loop_block, test, id, .. } => {
                let id = *id;
                self.visit_block(loop_block, state)?;
                self.visit_value(id, test, state)?;
            }
            ReactiveTerminal::While { test, loop_block, id, .. } => {
                let id = *id;
                self.visit_value(id, test, state)?;
                self.visit_block(loop_block, state)?;
            }
            ReactiveTerminal::If { test, consequent, alternate, id, .. } => {
                self.visit_place(*id, test, state)?;
                self.visit_block(consequent, state)?;
                if let Some(alt) = alternate {
                    self.visit_block(alt, state)?;
                }
            }
            ReactiveTerminal::Switch { test, cases, id, .. } => {
                let id = *id;
                self.visit_place(id, test, state)?;
                for case in cases.iter_mut() {
                    if let Some(t) = &case.test {
                        self.visit_place(id, t, state)?;
                    }
                    if let Some(block) = &mut case.block {
                        self.visit_block(block, state)?;
                    }
                }
            }
            ReactiveTerminal::Label { block, .. } => {
                self.visit_block(block, state)?;
            }
            ReactiveTerminal::Try { block, handler_binding, handler, id, .. } => {
                let id = *id;
                self.visit_block(block, state)?;
                if let Some(binding) = handler_binding {
                    self.visit_place(id, binding, state)?;
                }
                self.visit_block(handler, state)?;
            }
        }
        Ok(())
    }

    fn visit_scope(
        &mut self,
        scope: &mut ReactiveScopeBlock<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.traverse_scope(scope, state)
    }

    fn traverse_scope(
        &mut self,
        scope: &mut ReactiveScopeBlock<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.visit_block(&mut scope.instructions, state)
    }

    fn visit_pruned_scope(
        &mut self,
        scope: &mut PrunedReactiveScopeBlock<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.traverse_pruned_scope(scope, state)
    }

    fn traverse_pruned_scope(
        &mut self,
        scope: &mut PrunedReactiveScopeBlock<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.visit_block(&mut scope.instructions, state)
    }

    fn visit_block(
        &mut self,
        block: &mut ReactiveBlock<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        self.traverse_block(block, state)
    }

    fn transform_instruction(
        &mut self,
        instruction: &mut ReactiveInstruction<'a>,
        state: &mut Self::State,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        self.visit_instruction(instruction, state)?;
        Ok(Transformed::Keep)
    }

    fn transform_terminal(
        &mut self,
        stmt: &mut ReactiveTerminalStatement<'a>,
        state: &mut Self::State,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        self.visit_terminal(stmt, state)?;
        Ok(Transformed::Keep)
    }

    fn transform_scope(
        &mut self,
        scope: &mut ReactiveScopeBlock<'a>,
        state: &mut Self::State,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        self.visit_scope(scope, state)?;
        Ok(Transformed::Keep)
    }

    fn transform_pruned_scope(
        &mut self,
        scope: &mut PrunedReactiveScopeBlock<'a>,
        state: &mut Self::State,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        self.visit_pruned_scope(scope, state)?;
        Ok(Transformed::Keep)
    }

    /// Dispatch a single statement to its matching `transform_*` hook.
    fn transform_stmt(
        &mut self,
        stmt: &mut ReactiveStatement<'a>,
        state: &mut Self::State,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        match stmt {
            ReactiveStatement::Instruction(instr) => self.transform_instruction(instr, state),
            ReactiveStatement::Scope(scope) => self.transform_scope(scope, state),
            ReactiveStatement::PrunedScope(scope) => self.transform_pruned_scope(scope, state),
            ReactiveStatement::Terminal(terminal) => self.transform_terminal(terminal, state),
        }
    }

    fn traverse_block(
        &mut self,
        block: &mut ReactiveBlock<'a>,
        state: &mut Self::State,
    ) -> Result<(), OxcDiagnostic> {
        // Fast path: while statements are only kept or replaced 1:1 the block's
        // length never changes, so mutate `block` in place — no allocation, no
        // moves. This covers every read-only visitor pass and every transform
        // that doesn't restructure the block.
        let mut i = 0;
        let first_change = loop {
            let Some(stmt) = block.get_mut(i) else { return Ok(()) };
            match self.transform_stmt(stmt, state)? {
                Transformed::Keep => i += 1,
                Transformed::Replace(replacement) => {
                    block[i] = replacement;
                    i += 1;
                }
                // `Remove`/`ReplaceMany` change the length — hand off to the
                // rebuild below rather than shifting the suffix in place.
                change => break change,
            }
        };

        // Slow path: a `Remove`/`ReplaceMany` at `i` needs restructuring. Detach
        // the unprocessed tail (`block` keeps the finalized prefix `[0..i]`) and
        // rebuild it in a single linear pass, *moving* each statement back onto
        // `block` — never cloning. A pass that removes or expands many statements
        // in one block therefore stays O(n) instead of the O(n²) of repeated
        // `remove`/`splice` suffix shifts.
        let mut tail = block.split_off(i).into_iter();
        // The statement at `i` was already transformed into `first_change`; drop
        // its now-superseded original.
        tail.next();
        match first_change {
            Transformed::Remove => {}
            Transformed::ReplaceMany(replacements) => block.extend(replacements),
            // The fast-path loop only breaks on `Remove`/`ReplaceMany`.
            Transformed::Keep | Transformed::Replace(_) => unreachable!(),
        }
        for mut stmt in tail {
            match self.transform_stmt(&mut stmt, state)? {
                Transformed::Keep => block.push(stmt),
                Transformed::Remove => {}
                Transformed::Replace(replacement) => block.push(replacement),
                Transformed::ReplaceMany(replacements) => block.extend(replacements),
            }
        }
        Ok(())
    }
}

/// Entry point for transforming a reactive function.
/// TS: `visitReactiveFunction` (used with transforms too)
pub fn transform_reactive_function<'a, T: ReactiveFunctionTransform<'a>>(
    func: &mut ReactiveFunction<'a>,
    transform: &mut T,
    state: &mut T::State,
) -> Result<(), OxcDiagnostic> {
    transform.visit_block(&mut func.body, state)
}

// =============================================================================
// Helper: extract terminal ID
// =============================================================================

fn terminal_id(terminal: &ReactiveTerminal) -> EvaluationOrder {
    match terminal {
        ReactiveTerminal::Break { id, .. }
        | ReactiveTerminal::Continue { id, .. }
        | ReactiveTerminal::Return { id, .. }
        | ReactiveTerminal::Throw { id, .. }
        | ReactiveTerminal::Switch { id, .. }
        | ReactiveTerminal::DoWhile { id, .. }
        | ReactiveTerminal::While { id, .. }
        | ReactiveTerminal::For { id, .. }
        | ReactiveTerminal::ForOf { id, .. }
        | ReactiveTerminal::ForIn { id, .. }
        | ReactiveTerminal::If { id, .. }
        | ReactiveTerminal::Label { id, .. }
        | ReactiveTerminal::Try { id, .. } => *id,
    }
}
