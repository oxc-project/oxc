// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! RenameVariables — renames variables for output, assigns unique names,
//! handles SSA renames.
//!
//! Corresponds to `src/ReactiveScopes/RenameVariables.ts`.

use rustc_hash::FxHashMap;

use oxc_str::{Ident, IdentHashMap, IdentHashSet, format_ident};

use crate::react_compiler_hir::DeclarationId;
use crate::react_compiler_hir::EvaluationOrder;
use crate::react_compiler_hir::FunctionId;
use crate::react_compiler_hir::IdentifierId;
use crate::react_compiler_hir::IdentifierName;
use crate::react_compiler_hir::InstructionValue;
use crate::react_compiler_hir::ParamPattern;
use crate::react_compiler_hir::Place;
use crate::react_compiler_hir::PrunedReactiveScopeBlock;
use crate::react_compiler_hir::ReactiveBlock;
use crate::react_compiler_hir::ReactiveFunction;
use crate::react_compiler_hir::ReactiveScopeBlock;
use crate::react_compiler_hir::ReactiveStatement;
use crate::react_compiler_hir::ReactiveTerminal;
use crate::react_compiler_hir::ReactiveTerminalStatement;
use crate::react_compiler_hir::ReactiveValue;
use crate::react_compiler_hir::environment::Environment;

use crate::react_compiler_reactive_scopes::visitors;
use crate::react_compiler_reactive_scopes::visitors::ReactiveFunctionVisitor;

// =============================================================================
// Scopes
// =============================================================================

struct Scopes<'a> {
    seen: FxHashMap<DeclarationId, IdentifierName<'a>>,
    stack: Vec<IdentHashMap<'a, DeclarationId>>,
    globals: IdentHashSet<'a>,
    names: IdentHashSet<'a>,
}

impl<'a> Scopes<'a> {
    fn new(globals: IdentHashSet<'a>) -> Self {
        Self {
            seen: FxHashMap::default(),
            stack: vec![IdentHashMap::default()],
            globals,
            names: IdentHashSet::default(),
        }
    }

    fn visit_identifier(&mut self, identifier_id: IdentifierId, env: &Environment<'a>) {
        let identifier = &env.identifiers[identifier_id.0 as usize];
        let original_name = match &identifier.name {
            Some(name) => *name,
            None => return,
        };
        let declaration_id = identifier.declaration_id;

        if self.seen.contains_key(&declaration_id) {
            return;
        }

        let original_value = original_name.value();
        let is_promoted = matches!(original_name, IdentifierName::Promoted(_));
        let is_promoted_temp = is_promoted && original_value.starts_with("#t");
        let is_promoted_jsx = is_promoted && original_value.starts_with("#T");

        let mut name: Ident<'a>;
        let mut id: u32 = 0;
        if is_promoted_temp {
            name = format_ident!(env.allocator, "t{}", id);
            id += 1;
        } else if is_promoted_jsx {
            name = format_ident!(env.allocator, "T{}", id);
            id += 1;
        } else {
            name = Ident::from(original_value);
        }

        while self.lookup(&name).is_some() || self.globals.contains(&name) {
            if is_promoted_temp {
                name = format_ident!(env.allocator, "t{}", id);
                id += 1;
            } else if is_promoted_jsx {
                name = format_ident!(env.allocator, "T{}", id);
                id += 1;
            } else {
                name = format_ident!(env.allocator, "{}${}", original_value, id);
                id += 1;
            }
        }

        let identifier_name = IdentifierName::Named(name);
        self.seen.insert(declaration_id, identifier_name);
        self.stack.last_mut().unwrap().insert(name, declaration_id);
        self.names.insert(name);
    }

    fn lookup(&self, name: &str) -> Option<DeclarationId> {
        for scope in self.stack.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(*id);
            }
        }
        None
    }

    fn enter(&mut self) {
        self.stack.push(IdentHashMap::default());
    }

    fn leave(&mut self) {
        self.stack.pop();
    }
}

// =============================================================================
// Visitor — TS: `class Visitor extends ReactiveFunctionVisitor<Scopes>`
// =============================================================================

struct Visitor<'a, 'e> {
    env: &'e Environment<'a>,
}

impl<'a, 'e> ReactiveFunctionVisitor<'a> for Visitor<'a, 'e> {
    type State = Scopes<'a>;

    fn env(&self) -> &Environment<'a> {
        self.env
    }

    /// TS: `visitParam(place, state) { state.visit(place.identifier) }`
    fn visit_param(&self, place: &Place, state: &mut Scopes<'a>) {
        state.visit_identifier(place.identifier, self.env);
    }

    /// TS: `visitLValue(_id, lvalue, state) { state.visit(lvalue.identifier) }`
    fn visit_lvalue(&self, _id: EvaluationOrder, lvalue: &Place, state: &mut Scopes<'a>) {
        state.visit_identifier(lvalue.identifier, self.env);
    }

    /// TS: `visitPlace(_id, place, state) { state.visit(place.identifier) }`
    fn visit_place(&self, _id: EvaluationOrder, place: &Place, state: &mut Scopes<'a>) {
        state.visit_identifier(place.identifier, self.env);
    }

    /// TS: `visitBlock(block, state) { state.enter(() => { this.traverseBlock(block, state) }) }`
    fn visit_block(&self, block: &ReactiveBlock<'a>, state: &mut Scopes<'a>) {
        state.enter();
        self.traverse_block(block, state);
        state.leave();
    }

    /// TS: `visitPrunedScope(scopeBlock, state) { this.traverseBlock(scopeBlock.instructions, state) }`
    /// No enter/leave — names assigned inside pruned scopes remain visible in
    /// the enclosing scope, preventing name reuse.
    fn visit_pruned_scope(&self, scope: &PrunedReactiveScopeBlock<'a>, state: &mut Scopes<'a>) {
        self.traverse_block(&scope.instructions, state);
    }

    /// TS: `visitScope(scope, state) { for (const [_, decl] of scope.scope.declarations) state.visit(decl.identifier); this.traverseScope(scope, state) }`
    fn visit_scope(&self, scope: &ReactiveScopeBlock<'a>, state: &mut Scopes<'a>) {
        let scope_data = &self.env.scopes[scope.scope.0 as usize];
        let decl_ids: Vec<IdentifierId> =
            scope_data.declarations.iter().map(|(_, d)| d.identifier).collect();
        for id in decl_ids {
            state.visit_identifier(id, self.env);
        }
        self.traverse_scope(scope, state);
    }

    /// TS: `visitValue(id, value, state) { this.traverseValue(id, value, state); if (value.kind === 'FunctionExpression' || value.kind === 'ObjectMethod') this.visitHirFunction(value.loweredFunc.func, state) }`
    fn visit_value(&self, id: EvaluationOrder, value: &ReactiveValue<'a>, state: &mut Scopes<'a>) {
        self.traverse_value(id, value, state);
        if let ReactiveValue::Instruction(
            InstructionValue::FunctionExpression { lowered_func, .. }
            | InstructionValue::ObjectMethod { lowered_func, .. },
        ) = value
        {
            self.visit_hir_function(lowered_func.func, state);
        }
    }
}

// =============================================================================
// Public entry point
// =============================================================================

/// Renames variables for output — assigns unique names, handles SSA renames.
/// Returns a Set of all unique variable names used.
/// TS: `renameVariables`
pub fn rename_variables<'a>(
    func: &mut ReactiveFunction<'a>,
    env: &mut Environment<'a>,
) -> IdentHashSet<'a> {
    rename_variables_with_parent(func, env, None)
}

fn rename_variables_with_parent<'a>(
    func: &mut ReactiveFunction<'a>,
    env: &mut Environment<'a>,
    parent_names: Option<&IdentHashSet<'a>>,
) -> IdentHashSet<'a> {
    let globals = collect_referenced_globals(&func.body, env);

    // Phase 1: Use ReactiveFunctionVisitor to compute the rename mapping.
    // This collects DeclarationId -> IdentifierName without mutating env.
    let mut scopes = Scopes::new(globals.clone());
    // If parent names are provided (for outlined functions), pre-populate
    // the scope stack so that parameter names don't collide with parent
    // variables. In the TS compiler, outlined functions are placed in the
    // parent function body and processed within the parent's scope context.
    if let Some(parent) = parent_names {
        scopes.enter();
        for name in parent {
            scopes.stack.last_mut().unwrap().insert(*name, DeclarationId(u32::MAX));
            scopes.names.insert(*name);
        }
    }
    rename_variables_impl(func, &Visitor { env }, &mut scopes);

    // Phase 2: Apply the computed renames to all identifiers in env.
    for identifier in env.identifiers.iter_mut() {
        if let Some(mapped_name) = scopes.seen.get(&identifier.declaration_id) {
            if identifier.name.is_some() {
                identifier.name = Some(*mapped_name);
            }
        }
    }

    let mut result: IdentHashSet<'a> = scopes.names;
    result.extend(globals);
    result
}

/// TS: `renameVariablesImpl`
fn rename_variables_impl<'a>(
    func: &ReactiveFunction<'a>,
    visitor: &Visitor<'a, '_>,
    scopes: &mut Scopes<'a>,
) {
    scopes.enter();
    for param in &func.params {
        let place = match param {
            ParamPattern::Place(p) => p,
            ParamPattern::Spread(s) => &s.place,
        };
        visitor.visit_param(place, scopes);
    }
    visitors::visit_reactive_function(func, visitor, scopes);
    scopes.leave();
}

// =============================================================================
// CollectReferencedGlobals
// =============================================================================

/// Collects all globally referenced names from the reactive function.
/// TS: `collectReferencedGlobals`
fn collect_referenced_globals<'a>(
    block: &ReactiveBlock<'a>,
    env: &Environment<'a>,
) -> IdentHashSet<'a> {
    let mut globals = IdentHashSet::default();
    collect_globals_block(block, &mut globals, env);
    globals
}

fn collect_globals_block<'a>(
    block: &ReactiveBlock<'a>,
    globals: &mut IdentHashSet<'a>,
    env: &Environment<'a>,
) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(instr) => {
                collect_globals_value(&instr.value, globals, env);
            }
            ReactiveStatement::Scope(scope) => {
                collect_globals_block(&scope.instructions, globals, env);
            }
            ReactiveStatement::PrunedScope(scope) => {
                collect_globals_block(&scope.instructions, globals, env);
            }
            ReactiveStatement::Terminal(terminal) => {
                collect_globals_terminal(terminal, globals, env);
            }
        }
    }
}

fn collect_globals_value<'a>(
    value: &ReactiveValue<'a>,
    globals: &mut IdentHashSet<'a>,
    env: &Environment<'a>,
) {
    match value {
        ReactiveValue::Instruction(iv) => {
            if let InstructionValue::LoadGlobal { binding, .. } = iv {
                globals.insert(binding.name());
            }
            // Visit inner functions
            match iv {
                InstructionValue::FunctionExpression { lowered_func, .. }
                | InstructionValue::ObjectMethod { lowered_func, .. } => {
                    collect_globals_hir_function(lowered_func.func, globals, env);
                }
                _ => {}
            }
        }
        ReactiveValue::SequenceExpression { instructions, value: inner, .. } => {
            for instr in instructions {
                collect_globals_value(&instr.value, globals, env);
            }
            collect_globals_value(inner, globals, env);
        }
        ReactiveValue::ConditionalExpression { test, consequent, alternate, .. } => {
            collect_globals_value(test, globals, env);
            collect_globals_value(consequent, globals, env);
            collect_globals_value(alternate, globals, env);
        }
        ReactiveValue::LogicalExpression { left, right, .. } => {
            collect_globals_value(left, globals, env);
            collect_globals_value(right, globals, env);
        }
        ReactiveValue::OptionalExpression { value: inner, .. } => {
            collect_globals_value(inner, globals, env);
        }
    }
}

/// Recursively collects LoadGlobal names from an inner HIR function.
fn collect_globals_hir_function<'a>(
    func_id: FunctionId,
    globals: &mut IdentHashSet<'a>,
    env: &Environment<'a>,
) {
    let inner_func = &env.functions[func_id.0 as usize];
    let block_ids: Vec<_> = inner_func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let inner_func = &env.functions[func_id.0 as usize];
        let block = &inner_func.body.blocks[&block_id];
        for instr_id in &block.instructions {
            let instr = &inner_func.instructions[instr_id.0 as usize];
            if let InstructionValue::LoadGlobal { binding, .. } = &instr.value {
                globals.insert(binding.name());
            }
            // Recurse into nested function expressions
            match &instr.value {
                InstructionValue::FunctionExpression { lowered_func, .. }
                | InstructionValue::ObjectMethod { lowered_func, .. } => {
                    collect_globals_hir_function(lowered_func.func, globals, env);
                }
                _ => {}
            }
        }
    }
}

fn collect_globals_terminal<'a>(
    stmt: &ReactiveTerminalStatement<'a>,
    globals: &mut IdentHashSet<'a>,
    env: &Environment<'a>,
) {
    match &stmt.terminal {
        ReactiveTerminal::Break { .. } | ReactiveTerminal::Continue { .. } => {}
        ReactiveTerminal::Return { .. } | ReactiveTerminal::Throw { .. } => {}
        ReactiveTerminal::For { init, test, update, loop_block, .. } => {
            collect_globals_value(init, globals, env);
            collect_globals_value(test, globals, env);
            collect_globals_block(loop_block, globals, env);
            if let Some(update) = update {
                collect_globals_value(update, globals, env);
            }
        }
        ReactiveTerminal::ForOf { init, test, loop_block, .. } => {
            collect_globals_value(init, globals, env);
            collect_globals_value(test, globals, env);
            collect_globals_block(loop_block, globals, env);
        }
        ReactiveTerminal::ForIn { init, loop_block, .. } => {
            collect_globals_value(init, globals, env);
            collect_globals_block(loop_block, globals, env);
        }
        ReactiveTerminal::DoWhile { loop_block, test, .. } => {
            collect_globals_block(loop_block, globals, env);
            collect_globals_value(test, globals, env);
        }
        ReactiveTerminal::While { test, loop_block, .. } => {
            collect_globals_value(test, globals, env);
            collect_globals_block(loop_block, globals, env);
        }
        ReactiveTerminal::If { consequent, alternate, .. } => {
            collect_globals_block(consequent, globals, env);
            if let Some(alt) = alternate {
                collect_globals_block(alt, globals, env);
            }
        }
        ReactiveTerminal::Switch { cases, .. } => {
            for case in cases {
                if let Some(block) = &case.block {
                    collect_globals_block(block, globals, env);
                }
            }
        }
        ReactiveTerminal::Label { block, .. } => {
            collect_globals_block(block, globals, env);
        }
        ReactiveTerminal::Try { block, handler, .. } => {
            collect_globals_block(block, globals, env);
            collect_globals_block(handler, globals, env);
        }
    }
}
