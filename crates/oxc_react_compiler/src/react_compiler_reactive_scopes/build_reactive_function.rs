// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Converts the HIR CFG into a tree-structured ReactiveFunction.
//!
//! Corresponds to `src/ReactiveScopes/BuildReactiveFunction.ts`.

use std::mem::discriminant;

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashSet;

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::{
    BasicBlock, BlockId, EvaluationOrder, GotoVariant, HirFunction, InstructionId,
    InstructionValue, Place, PrimitiveValue, PrunedReactiveScopeBlock, ReactiveBlock,
    ReactiveFunction, ReactiveInstruction, ReactiveLabel, ReactiveScopeBlock, ReactiveStatement,
    ReactiveSwitchCase, ReactiveTerminal, ReactiveTerminalStatement, ReactiveTerminalTargetKind,
    ReactiveValue, Terminal,
};
use oxc_span::Span;

/// Convert the HIR CFG into a tree-structured ReactiveFunction.
pub fn build_reactive_function<'a>(
    hir: &HirFunction<'a>,
    env: &Environment<'a>,
) -> Result<ReactiveFunction<'a>, OxcDiagnostic> {
    let mut ctx = Context::new(hir);
    let mut driver = Driver { cx: &mut ctx, hir, env };

    let entry_block_id = hir.body.entry;
    let mut body = Vec::new();
    driver.visit_block(entry_block_id, &mut body)?;

    Ok(ReactiveFunction {
        span: hir.span,
        id: hir.id,
        name_hint: hir.name_hint,
        params: hir.params.clone(),
        generator: hir.generator,
        is_async: hir.is_async,
        body,
        directives: hir.directives.clone(),
    })
}

// =============================================================================
// ControlFlowTarget
// =============================================================================

#[derive(Debug)]
enum ControlFlowTarget {
    If {
        block: BlockId,
        id: u32,
    },
    Switch {
        block: BlockId,
        id: u32,
    },
    Case {
        block: BlockId,
        id: u32,
    },
    Loop {
        block: BlockId,
        continue_block: BlockId,
        loop_block: Option<BlockId>,
        owns_loop: bool,
        id: u32,
    },
}

impl ControlFlowTarget {
    fn block(&self) -> BlockId {
        match self {
            ControlFlowTarget::If { block, .. }
            | ControlFlowTarget::Switch { block, .. }
            | ControlFlowTarget::Case { block, .. }
            | ControlFlowTarget::Loop { block, .. } => *block,
        }
    }

    fn id(&self) -> u32 {
        match self {
            ControlFlowTarget::If { id, .. }
            | ControlFlowTarget::Switch { id, .. }
            | ControlFlowTarget::Case { id, .. }
            | ControlFlowTarget::Loop { id, .. } => *id,
        }
    }

    fn is_loop(&self) -> bool {
        matches!(self, ControlFlowTarget::Loop { .. })
    }
}

// =============================================================================
// Context
// =============================================================================

struct Context<'a, 'h> {
    ir: &'h HirFunction<'a>,
    next_schedule_id: u32,
    emitted: FxHashSet<BlockId>,
    scope_fallthroughs: FxHashSet<BlockId>,
    scheduled: FxHashSet<BlockId>,
    catch_handlers: FxHashSet<BlockId>,
    control_flow_stack: Vec<ControlFlowTarget>,
}

impl<'a, 'h> Context<'a, 'h> {
    fn new(ir: &'h HirFunction<'a>) -> Self {
        Self {
            ir,
            next_schedule_id: 0,
            emitted: FxHashSet::default(),
            scope_fallthroughs: FxHashSet::default(),
            scheduled: FxHashSet::default(),
            catch_handlers: FxHashSet::default(),
            control_flow_stack: Vec::new(),
        }
    }

    fn block(&self, id: BlockId) -> &BasicBlock {
        &self.ir.body.blocks[&id]
    }

    fn schedule_catch_handler(&mut self, block: BlockId) {
        self.catch_handlers.insert(block);
    }

    fn reachable(&self, id: BlockId) -> bool {
        let block = self.block(id);
        !matches!(block.terminal, Terminal::Unreachable { .. })
    }

    fn schedule(&mut self, block: BlockId, target_type: &str) -> Result<u32, OxcDiagnostic> {
        let id = self.next_schedule_id;
        self.next_schedule_id += 1;
        if self.scheduled.contains(&block) {
            return Err(ErrorCategory::Invariant
                .diagnostic(format!("Break block is already scheduled: bb{}", block.index())));
        }
        self.scheduled.insert(block);
        let target = match target_type {
            "if" => ControlFlowTarget::If { block, id },
            "switch" => ControlFlowTarget::Switch { block, id },
            "case" => ControlFlowTarget::Case { block, id },
            _ => {
                return Err(ErrorCategory::Invariant
                    .diagnostic(format!("Unknown target type: {}", target_type)));
            }
        };
        self.control_flow_stack.push(target);
        Ok(id)
    }

    fn schedule_loop(
        &mut self,
        fallthrough_block: BlockId,
        continue_block: BlockId,
        loop_block: Option<BlockId>,
    ) -> Result<u32, OxcDiagnostic> {
        let id = self.next_schedule_id;
        self.next_schedule_id += 1;
        self.scheduled.insert(fallthrough_block);
        if self.scheduled.contains(&continue_block) {
            return Err(ErrorCategory::Invariant.diagnostic(format!(
                "Continue block is already scheduled: bb{}",
                continue_block.index()
            )));
        }
        self.scheduled.insert(continue_block);
        let mut owns_loop = false;
        if let Some(lb) = loop_block {
            owns_loop = !self.scheduled.contains(&lb);
            self.scheduled.insert(lb);
        }

        self.control_flow_stack.push(ControlFlowTarget::Loop {
            block: fallthrough_block,
            continue_block,
            loop_block,
            owns_loop,
            id,
        });
        Ok(id)
    }

    fn unschedule(&mut self, schedule_id: u32) -> Result<(), OxcDiagnostic> {
        let last = self.control_flow_stack.pop().expect("Can only unschedule the last target");
        if last.id() != schedule_id {
            return Err(ErrorCategory::Invariant.diagnostic("Can only unschedule the last target"));
        }
        match &last {
            ControlFlowTarget::Loop { block, continue_block, loop_block, owns_loop, .. } => {
                // TS: always removes block from scheduled for loops
                // (ownsBlock is boolean, so `!== null` is always true)
                self.scheduled.remove(block);
                self.scheduled.remove(continue_block);
                if *owns_loop {
                    if let Some(lb) = loop_block {
                        self.scheduled.remove(lb);
                    }
                }
            }
            _ => {
                self.scheduled.remove(&last.block());
            }
        }
        Ok(())
    }

    fn unschedule_all(&mut self, schedule_ids: &[u32]) -> Result<(), OxcDiagnostic> {
        for &id in schedule_ids.iter().rev() {
            self.unschedule(id)?;
        }
        Ok(())
    }

    fn is_scheduled(&self, block: BlockId) -> bool {
        self.scheduled.contains(&block) || self.catch_handlers.contains(&block)
    }

    fn get_break_target(
        &self,
        block: BlockId,
    ) -> Result<(BlockId, ReactiveTerminalTargetKind), OxcDiagnostic> {
        let mut has_preceding_loop = false;
        for i in (0..self.control_flow_stack.len()).rev() {
            let target = &self.control_flow_stack[i];
            if target.block() == block {
                let kind = if target.is_loop() {
                    if has_preceding_loop {
                        ReactiveTerminalTargetKind::Labeled
                    } else {
                        ReactiveTerminalTargetKind::Unlabeled
                    }
                } else if i == self.control_flow_stack.len() - 1 {
                    ReactiveTerminalTargetKind::Implicit
                } else {
                    ReactiveTerminalTargetKind::Labeled
                };
                return Ok((target.block(), kind));
            }
            has_preceding_loop = has_preceding_loop || target.is_loop();
        }
        Err(ErrorCategory::Invariant
            .diagnostic(format!("Expected a break target for bb{}", block.index())))
    }

    fn get_continue_target(&self, block: BlockId) -> Option<(BlockId, ReactiveTerminalTargetKind)> {
        let mut has_preceding_loop = false;
        for i in (0..self.control_flow_stack.len()).rev() {
            let target = &self.control_flow_stack[i];
            if let ControlFlowTarget::Loop { block: fallthrough_block, continue_block, .. } = target
            {
                if *continue_block == block {
                    let kind = if has_preceding_loop {
                        ReactiveTerminalTargetKind::Labeled
                    } else if i == self.control_flow_stack.len() - 1 {
                        ReactiveTerminalTargetKind::Implicit
                    } else {
                        ReactiveTerminalTargetKind::Unlabeled
                    };
                    return Some((*fallthrough_block, kind));
                }
            }
            has_preceding_loop = has_preceding_loop || target.is_loop();
        }
        None
    }
}

// =============================================================================
// Driver
// =============================================================================

struct Driver<'a, 'b, 'h> {
    cx: &'b mut Context<'a, 'h>,
    hir: &'h HirFunction<'a>,
    #[allow(dead_code)]
    env: &'h Environment<'a>,
}

impl<'a, 'b, 'h> Driver<'a, 'b, 'h> {
    fn traverse_block(&mut self, block_id: BlockId) -> Result<ReactiveBlock<'a>, OxcDiagnostic> {
        let mut block_value = Vec::new();
        self.visit_block(block_id, &mut block_value)?;
        Ok(block_value)
    }

    fn visit_block(
        &mut self,
        mut block_id: BlockId,
        block_value: &mut ReactiveBlock<'a>,
    ) -> Result<(), OxcDiagnostic> {
        // Use a loop to avoid deep recursion for fallthrough chains.
        // Each terminal that would tail-call visit_block(fallthrough, block_value)
        // instead sets next_block and continues the loop.
        loop {
            // Extract data from block before any mutable operations
            let block = &self.hir.body.blocks[&block_id];
            let block_id_val = block.id;
            let instructions: Vec<_> = block.instructions.clone();
            let terminal = block.terminal.clone();

            if !self.cx.emitted.insert(block_id_val) {
                return Err(ErrorCategory::Invariant
                    .diagnostic(format!("Block bb{} was already emitted", block_id_val.index())));
            }

            // Emit instructions
            for instr_id in &instructions {
                let instr = &self.hir.instructions[instr_id.index()];
                block_value.push(ReactiveStatement::Instruction(ReactiveInstruction {
                    id: instr.id,
                    lvalue: Some(instr.lvalue.clone()),
                    value: ReactiveValue::Instruction(instr.value.clone()),
                    span: instr.span,
                }));
            }

            // Process terminal
            let mut schedule_ids: Vec<u32> = Vec::new();
            let mut next_block: Option<BlockId> = None;

            match &terminal {
                Terminal::If { test, consequent, alternate, fallthrough, id, .. } => {
                    // TS: reachable(fallthrough) && !isScheduled(fallthrough)
                    let fallthrough_id =
                        if self.cx.reachable(*fallthrough) && !self.cx.is_scheduled(*fallthrough) {
                            Some(*fallthrough)
                        } else {
                            None
                        };
                    // TS: alternate !== fallthrough ? alternate : null
                    let alternate_id =
                        if *alternate != *fallthrough { Some(*alternate) } else { None };

                    if let Some(ft) = fallthrough_id {
                        schedule_ids.push(self.cx.schedule(ft, "if")?);
                    }

                    let consequent_block = if self.cx.is_scheduled(*consequent) {
                        return Err(ErrorCategory::Invariant.diagnostic(format!(
                            "Unexpected 'if' where consequent is already scheduled (bb{})",
                            consequent.index()
                        )));
                    } else {
                        self.traverse_block(*consequent)?
                    };

                    let alternate_block = if let Some(alt) = alternate_id {
                        if self.cx.is_scheduled(alt) {
                            return Err(ErrorCategory::Invariant.diagnostic(format!(
                                "Unexpected 'if' where the alternate is already scheduled (bb{})",
                                alt.index()
                            )));
                        } else {
                            Some(self.traverse_block(alt)?)
                        }
                    } else {
                        None
                    };

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::If {
                            test: test.clone(),
                            consequent: consequent_block,
                            alternate: alternate_block,
                            id: *id,
                        },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::Switch { test, cases, fallthrough, id, .. } => {
                    // TS: reachable(fallthrough) && !isScheduled(fallthrough)
                    let fallthrough_id =
                        if self.cx.reachable(*fallthrough) && !self.cx.is_scheduled(*fallthrough) {
                            Some(*fallthrough)
                        } else {
                            None
                        };
                    if let Some(ft) = fallthrough_id {
                        schedule_ids.push(self.cx.schedule(ft, "switch")?);
                    }

                    // TS processes cases in reverse order, then reverses the result.
                    // This ensures that later cases are scheduled when earlier cases
                    // are traversed, matching fallthrough semantics.
                    let mut reactive_cases = Vec::new();
                    for case in cases.iter().rev() {
                        let case_block_id = case.block;

                        if self.cx.is_scheduled(case_block_id) {
                            // TS: asserts case.block === fallthrough, then skips (return)
                            if case_block_id != *fallthrough {
                                return Err(ErrorCategory::Invariant.diagnostic("Unexpected 'switch' where a case is already scheduled and block is not the fallthrough"));
                            }
                            continue;
                        }

                        let consequent = self.traverse_block(case_block_id)?;
                        let case_schedule_id = self.cx.schedule(case_block_id, "case")?;
                        schedule_ids.push(case_schedule_id);

                        reactive_cases.push(ReactiveSwitchCase {
                            test: case.test.clone(),
                            block: Some(consequent),
                        });
                    }
                    reactive_cases.reverse();

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Switch {
                            test: test.clone(),
                            cases: reactive_cases,
                            id: *id,
                        },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::DoWhile { loop_block, test, fallthrough, id, span } => {
                    let fallthrough_id =
                        if !self.cx.is_scheduled(*fallthrough) { Some(*fallthrough) } else { None };
                    let loop_id =
                        if !self.cx.is_scheduled(*loop_block) && *loop_block != *fallthrough {
                            Some(*loop_block)
                        } else {
                            None
                        };

                    schedule_ids.push(self.cx.schedule_loop(
                        *fallthrough,
                        *test,
                        Some(*loop_block),
                    )?);

                    let loop_body = if let Some(lid) = loop_id {
                        self.traverse_block(lid)?
                    } else {
                        return Err(ErrorCategory::Invariant.diagnostic(
                            "Unexpected 'do-while' where the loop is already scheduled",
                        ));
                    };
                    let test_result = self.visit_value_block(*test, *span, None)?;

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::DoWhile {
                            loop_block: loop_body,
                            test: test_result.value,
                            id: *id,
                        },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::While { test, loop_block, fallthrough, id, span } => {
                    // TS: reachable(fallthrough) && !isScheduled(fallthrough)
                    let fallthrough_id =
                        if self.cx.reachable(*fallthrough) && !self.cx.is_scheduled(*fallthrough) {
                            Some(*fallthrough)
                        } else {
                            None
                        };
                    let loop_id =
                        if !self.cx.is_scheduled(*loop_block) && *loop_block != *fallthrough {
                            Some(*loop_block)
                        } else {
                            None
                        };

                    schedule_ids.push(self.cx.schedule_loop(
                        *fallthrough,
                        *test,
                        Some(*loop_block),
                    )?);

                    let test_result = self.visit_value_block(*test, *span, None)?;

                    let loop_body = if let Some(lid) = loop_id {
                        self.traverse_block(lid)?
                    } else {
                        return Err(ErrorCategory::Invariant
                            .diagnostic("Unexpected 'while' where the loop is already scheduled"));
                    };

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::While {
                            test: test_result.value,
                            loop_block: loop_body,
                            id: *id,
                        },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::For { init, test, update, loop_block, fallthrough, id, span } => {
                    let loop_id =
                        if !self.cx.is_scheduled(*loop_block) && *loop_block != *fallthrough {
                            Some(*loop_block)
                        } else {
                            None
                        };

                    let fallthrough_id =
                        if !self.cx.is_scheduled(*fallthrough) { Some(*fallthrough) } else { None };

                    // Continue block is update (if present) or test
                    let continue_block = update.unwrap_or(*test);
                    schedule_ids.push(self.cx.schedule_loop(
                        *fallthrough,
                        continue_block,
                        Some(*loop_block),
                    )?);

                    let init_result = self.visit_value_block(*init, *span, None)?;
                    let init_value = self.value_block_result_to_sequence(init_result, *span);

                    let test_result = self.visit_value_block(*test, *span, None)?;

                    let update_result = match update {
                        Some(u) => Some(self.visit_value_block(*u, *span, None)?),
                        None => None,
                    };

                    let loop_body = if let Some(lid) = loop_id {
                        self.traverse_block(lid)?
                    } else {
                        return Err(ErrorCategory::Invariant
                            .diagnostic("Unexpected 'for' where the loop is already scheduled"));
                    };

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::For {
                            init: init_value,
                            test: test_result.value,
                            update: update_result.map(|r| r.value),
                            loop_block: loop_body,
                            id: *id,
                        },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::ForOf { init, test, loop_block, fallthrough, id, span } => {
                    let loop_id =
                        if !self.cx.is_scheduled(*loop_block) && *loop_block != *fallthrough {
                            Some(*loop_block)
                        } else {
                            None
                        };

                    let fallthrough_id =
                        if !self.cx.is_scheduled(*fallthrough) { Some(*fallthrough) } else { None };

                    // TS: scheduleLoop(fallthrough, init, loop)
                    schedule_ids.push(self.cx.schedule_loop(
                        *fallthrough,
                        *init,
                        Some(*loop_block),
                    )?);

                    let init_result = self.visit_value_block(*init, *span, None)?;
                    let init_value = self.value_block_result_to_sequence(init_result, *span);

                    let test_result = self.visit_value_block(*test, *span, None)?;
                    let test_value = self.value_block_result_to_sequence(test_result, *span);

                    let loop_body = if let Some(lid) = loop_id {
                        self.traverse_block(lid)?
                    } else {
                        return Err(ErrorCategory::Invariant.diagnostic(
                            "Unexpected 'for-of' where the loop is already scheduled",
                        ));
                    };

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::ForOf {
                            init: init_value,
                            test: test_value,
                            loop_block: loop_body,
                            id: *id,
                            span: *span,
                        },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::ForIn { init, loop_block, fallthrough, id, span } => {
                    let loop_id =
                        if !self.cx.is_scheduled(*loop_block) && *loop_block != *fallthrough {
                            Some(*loop_block)
                        } else {
                            None
                        };

                    let fallthrough_id =
                        if !self.cx.is_scheduled(*fallthrough) { Some(*fallthrough) } else { None };

                    schedule_ids.push(self.cx.schedule_loop(
                        *fallthrough,
                        *init,
                        Some(*loop_block),
                    )?);

                    let init_result = self.visit_value_block(*init, *span, None)?;
                    let init_value = self.value_block_result_to_sequence(init_result, *span);

                    let loop_body = if let Some(lid) = loop_id {
                        self.traverse_block(lid)?
                    } else {
                        return Err(ErrorCategory::Invariant.diagnostic(
                            "Unexpected 'for-in' where the loop is already scheduled",
                        ));
                    };

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::ForIn {
                            init: init_value,
                            loop_block: loop_body,
                            id: *id,
                            span: *span,
                        },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::Label { block: label_block, fallthrough, id, .. } => {
                    // TS: reachable(fallthrough) && !isScheduled(fallthrough)
                    let fallthrough_id =
                        if self.cx.reachable(*fallthrough) && !self.cx.is_scheduled(*fallthrough) {
                            Some(*fallthrough)
                        } else {
                            None
                        };
                    if let Some(ft) = fallthrough_id {
                        schedule_ids.push(self.cx.schedule(ft, "if")?);
                    }

                    if self.cx.is_scheduled(*label_block) {
                        return Err(ErrorCategory::Invariant.diagnostic(
                            "Unexpected 'label' where the block is already scheduled",
                        ));
                    }
                    let label_body = self.traverse_block(*label_block)?;

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Label { block: label_body, id: *id },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::Sequence { .. }
                | Terminal::Optional { .. }
                | Terminal::Ternary { .. }
                | Terminal::Logical { .. } => {
                    let fallthrough = match &terminal {
                        Terminal::Sequence { fallthrough, .. }
                        | Terminal::Optional { fallthrough, .. }
                        | Terminal::Ternary { fallthrough, .. }
                        | Terminal::Logical { fallthrough, .. } => *fallthrough,
                        _ => unreachable!(),
                    };
                    let fallthrough_id =
                        if !self.cx.is_scheduled(fallthrough) { Some(fallthrough) } else { None };
                    if let Some(ft) = fallthrough_id {
                        schedule_ids.push(self.cx.schedule(ft, "if")?);
                    }

                    let result = self.visit_value_block_terminal(&terminal)?;
                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Instruction(ReactiveInstruction {
                        id: result.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        span: *terminal_span(&terminal),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::Goto { block: goto_block, variant, id, .. } => {
                    match variant {
                        GotoVariant::Break => {
                            if let Some(stmt) = self.visit_break(*goto_block, *id)? {
                                block_value.push(stmt);
                            }
                        }
                        GotoVariant::Continue => {
                            let stmt = self.visit_continue(*goto_block, *id)?;
                            block_value.push(stmt);
                        }
                        GotoVariant::Try => {
                            // noop
                        }
                    }
                }

                Terminal::MaybeThrow { continuation, .. } => {
                    if !self.cx.is_scheduled(*continuation) {
                        next_block = Some(*continuation);
                    }
                }

                Terminal::Try {
                    block: try_block,
                    handler_binding,
                    handler,
                    fallthrough,
                    id,
                    ..
                } => {
                    let fallthrough_id =
                        if self.cx.reachable(*fallthrough) && !self.cx.is_scheduled(*fallthrough) {
                            Some(*fallthrough)
                        } else {
                            None
                        };
                    if let Some(ft) = fallthrough_id {
                        schedule_ids.push(self.cx.schedule(ft, "if")?);
                    }
                    self.cx.schedule_catch_handler(*handler);

                    let try_body = self.traverse_block(*try_block)?;
                    let handler_body = self.traverse_block(*handler)?;

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Try {
                            block: try_body,
                            handler_binding: handler_binding.clone(),
                            handler: handler_body,
                            id: *id,
                        },
                        label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::Scope { fallthrough, block: scope_block, scope, .. } => {
                    let fallthrough_id =
                        if !self.cx.is_scheduled(*fallthrough) { Some(*fallthrough) } else { None };
                    if let Some(ft) = fallthrough_id {
                        schedule_ids.push(self.cx.schedule(ft, "if")?);
                        self.cx.scope_fallthroughs.insert(ft);
                    }

                    if self.cx.is_scheduled(*scope_block) {
                        return Err(ErrorCategory::Invariant.diagnostic(
                            "Unexpected 'scope' where the block is already scheduled",
                        ));
                    }
                    let scope_body = self.traverse_block(*scope_block)?;

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::Scope(ReactiveScopeBlock {
                        scope: *scope,
                        instructions: scope_body,
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::PrunedScope { fallthrough, block: scope_block, scope, .. } => {
                    let fallthrough_id =
                        if !self.cx.is_scheduled(*fallthrough) { Some(*fallthrough) } else { None };
                    if let Some(ft) = fallthrough_id {
                        schedule_ids.push(self.cx.schedule(ft, "if")?);
                        self.cx.scope_fallthroughs.insert(ft);
                    }

                    if self.cx.is_scheduled(*scope_block) {
                        return Err(ErrorCategory::Invariant.diagnostic(
                            "Unexpected 'scope' where the block is already scheduled",
                        ));
                    }
                    let scope_body = self.traverse_block(*scope_block)?;

                    self.cx.unschedule_all(&schedule_ids)?;
                    block_value.push(ReactiveStatement::PrunedScope(PrunedReactiveScopeBlock {
                        scope: *scope,
                        instructions: scope_body,
                    }));

                    next_block = fallthrough_id;
                }

                Terminal::Return { value, id, .. } => {
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Return { value: value.clone(), id: *id },
                        label: None,
                    }));
                }

                Terminal::Throw { value, id, .. } => {
                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Throw { value: value.clone(), id: *id },
                        label: None,
                    }));
                }

                Terminal::Unreachable { .. } => {
                    // noop
                }

                Terminal::Branch { test, consequent, alternate, id, .. } => {
                    let consequent_block = if self.cx.is_scheduled(*consequent) {
                        if let Some(stmt) = self.visit_break(*consequent, *id)? {
                            vec![stmt]
                        } else {
                            Vec::new()
                        }
                    } else {
                        self.traverse_block(*consequent)?
                    };

                    if self.cx.is_scheduled(*alternate) {
                        return Err(ErrorCategory::Invariant.diagnostic(
                            "Unexpected 'branch' where the alternate is already scheduled",
                        ));
                    }
                    let alternate_block = self.traverse_block(*alternate)?;

                    block_value.push(ReactiveStatement::Terminal(ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::If {
                            test: test.clone(),
                            consequent: consequent_block,
                            alternate: Some(alternate_block),
                            id: *id,
                        },
                        label: None,
                    }));
                }
            }
            match next_block {
                Some(nb) => block_id = nb,
                None => return Ok(()),
            }
        } // end loop
    }

    // =========================================================================
    // Value block processing
    // =========================================================================

    fn visit_value_block(
        &mut self,
        block_id: BlockId,
        span: Option<Span>,
        fallthrough: Option<BlockId>,
    ) -> Result<ValueBlockResult<'a>, OxcDiagnostic> {
        let block = &self.hir.body.blocks[&block_id];
        let block_id_val = block.id;
        let terminal = block.terminal.clone();
        let instructions: Vec<_> = block.instructions.clone();

        // If we've reached the fallthrough, stop
        if let Some(ft) = fallthrough {
            if block_id == ft {
                return Err(ErrorCategory::Invariant.diagnostic(format!(
                    "Did not expect to reach the fallthrough of a value block (bb{})",
                    block_id.index()
                )));
            }
        }

        match &terminal {
            Terminal::Branch { test, id: term_id, .. } => {
                if instructions.is_empty() {
                    Ok(ValueBlockResult {
                        block: block_id_val,
                        place: test.clone(),
                        value: ReactiveValue::Instruction(InstructionValue::LoadLocal {
                            place: test.clone(),
                            span: test.span,
                        }),
                        id: *term_id,
                    })
                } else {
                    Ok(self.extract_value_block_result(&instructions, block_id_val))
                }
            }
            Terminal::Goto { .. } => {
                if instructions.is_empty() {
                    return Err(ErrorCategory::Invariant
                        .diagnostic("Unexpected empty block with `goto` terminal")
                        .with_help(format!("Block bb{} is empty", block_id.index()))
                        .with_labels(
                            span.map(|s| s.label("Unexpected empty block with `goto` terminal")),
                        ));
                }
                Ok(self.extract_value_block_result(&instructions, block_id_val))
            }
            Terminal::MaybeThrow { continuation, .. } => {
                let continuation_id = *continuation;
                let continuation_block = self.cx.block(continuation_id);
                let cont_instructions_empty = continuation_block.instructions.is_empty();
                let cont_is_goto = matches!(continuation_block.terminal, Terminal::Goto { .. });
                let cont_block_id = continuation_block.id;

                if cont_instructions_empty && cont_is_goto {
                    Ok(self.extract_value_block_result(&instructions, cont_block_id))
                } else {
                    let continuation =
                        self.visit_value_block(continuation_id, span, fallthrough)?;
                    Ok(self.wrap_with_sequence(&instructions, continuation))
                }
            }
            _ => {
                // Value block ended in a value terminal, recurse to get the value
                // of that terminal and stitch them together in a sequence.
                // TS: visitValueBlock(init.fallthrough, span) — does NOT propagate fallthrough
                let init = self.visit_value_block_terminal(&terminal)?;
                let init_fallthrough = init.fallthrough;
                let init_instr = ReactiveInstruction {
                    id: init.id,
                    lvalue: Some(init.place),
                    value: init.value,
                    span,
                };
                let final_result = self.visit_value_block(init_fallthrough, span, None)?;

                // Combine block instructions + init instruction, then wrap
                let mut all_instrs: Vec<ReactiveInstruction<'a>> = instructions
                    .iter()
                    .map(|iid| {
                        let instr = &self.hir.instructions[iid.index()];
                        ReactiveInstruction {
                            id: instr.id,
                            lvalue: Some(instr.lvalue.clone()),
                            value: ReactiveValue::Instruction(instr.value.clone()),
                            span: instr.span,
                        }
                    })
                    .collect();
                all_instrs.push(init_instr);

                if all_instrs.is_empty() {
                    Ok(final_result)
                } else {
                    Ok(ValueBlockResult {
                        block: final_result.block,
                        place: final_result.place.clone(),
                        value: ReactiveValue::SequenceExpression {
                            instructions: all_instrs,
                            id: final_result.id,
                            value: Box::new(final_result.value),
                        },
                        id: final_result.id,
                    })
                }
            }
        }
    }

    fn visit_test_block(
        &mut self,
        test_block_id: BlockId,
        span: Option<Span>,
        terminal_kind: &str,
    ) -> Result<TestBlockResult<'a>, OxcDiagnostic> {
        let test = self.visit_value_block(test_block_id, span, None)?;
        let test_block = &self.hir.body.blocks[&test.block];
        match &test_block.terminal {
            Terminal::Branch { consequent, alternate, span: branch_span, .. } => {
                Ok(TestBlockResult {
                    test,
                    consequent: *consequent,
                    alternate: *alternate,
                    branch_span: *branch_span,
                })
            }
            other => Err(ErrorCategory::Invariant.diagnostic(format!(
                "Expected a branch terminal for {} test block, got {:?}",
                terminal_kind,
                discriminant(other)
            ))),
        }
    }

    fn visit_value_block_terminal(
        &mut self,
        terminal: &Terminal,
    ) -> Result<ValueTerminalResult<'a>, OxcDiagnostic> {
        match terminal {
            Terminal::Sequence { block, fallthrough, id, span } => {
                let block_result = self.visit_value_block(*block, *span, Some(*fallthrough))?;
                Ok(ValueTerminalResult {
                    value: block_result.value,
                    place: block_result.place,
                    fallthrough: *fallthrough,
                    id: *id,
                })
            }
            Terminal::Optional { optional, test, fallthrough, id, span } => {
                let test_result = self.visit_test_block(*test, *span, "optional")?;
                let consequent =
                    self.visit_value_block(test_result.consequent, *span, Some(*fallthrough))?;
                let call = ReactiveValue::SequenceExpression {
                    instructions: vec![ReactiveInstruction {
                        id: test_result.test.id,
                        lvalue: Some(test_result.test.place.clone()),
                        value: test_result.test.value,
                        span: test_result.branch_span,
                    }],
                    id: consequent.id,
                    value: Box::new(consequent.value),
                };
                Ok(ValueTerminalResult {
                    place: consequent.place,
                    value: ReactiveValue::OptionalExpression {
                        optional: *optional,
                        value: Box::new(call),
                    },
                    fallthrough: *fallthrough,
                    id: *id,
                })
            }
            Terminal::Logical { operator, test, fallthrough, id, span } => {
                let test_result = self.visit_test_block(*test, *span, "logical")?;
                let left_final =
                    self.visit_value_block(test_result.consequent, *span, Some(*fallthrough))?;
                let left = ReactiveValue::SequenceExpression {
                    instructions: vec![ReactiveInstruction {
                        id: test_result.test.id,
                        lvalue: Some(test_result.test.place.clone()),
                        value: test_result.test.value,
                        span: *span,
                    }],
                    id: left_final.id,
                    value: Box::new(left_final.value),
                };
                let right =
                    self.visit_value_block(test_result.alternate, *span, Some(*fallthrough))?;
                Ok(ValueTerminalResult {
                    place: left_final.place,
                    value: ReactiveValue::LogicalExpression {
                        operator: *operator,
                        left: Box::new(left),
                        right: Box::new(right.value),
                    },
                    fallthrough: *fallthrough,
                    id: *id,
                })
            }
            Terminal::Ternary { test, fallthrough, id, span } => {
                let test_result = self.visit_test_block(*test, *span, "ternary")?;
                let consequent =
                    self.visit_value_block(test_result.consequent, *span, Some(*fallthrough))?;
                let alternate =
                    self.visit_value_block(test_result.alternate, *span, Some(*fallthrough))?;
                Ok(ValueTerminalResult {
                    place: consequent.place,
                    value: ReactiveValue::ConditionalExpression {
                        test: Box::new(test_result.test.value),
                        consequent: Box::new(consequent.value),
                        alternate: Box::new(alternate.value),
                    },
                    fallthrough: *fallthrough,
                    id: *id,
                })
            }
            Terminal::MaybeThrow { .. } => Err(ErrorCategory::Invariant
                .diagnostic("Unexpected maybe-throw in visit_value_block_terminal")),
            Terminal::Label { .. } => Err(ErrorCategory::Todo.diagnostic(
                "Support labeled statements combined with value blocks is not yet implemented",
            )),
            _ => {
                Err(ErrorCategory::Invariant.diagnostic("Unsupported terminal kind in value block"))
            }
        }
    }

    fn extract_value_block_result(
        &self,
        instructions: &[InstructionId],
        block_id: BlockId,
    ) -> ValueBlockResult<'a> {
        let last_id = instructions.last().expect("Expected non-empty instructions");
        let last_instr = &self.hir.instructions[last_id.index()];

        let remaining: Vec<ReactiveInstruction<'a>> = instructions[..instructions.len() - 1]
            .iter()
            .map(|iid| {
                let instr = &self.hir.instructions[iid.index()];
                ReactiveInstruction {
                    id: instr.id,
                    lvalue: Some(instr.lvalue.clone()),
                    value: ReactiveValue::Instruction(instr.value.clone()),
                    span: instr.span,
                }
            })
            .collect();

        // If the last instruction is a StoreLocal to a temporary (unnamed identifier),
        // convert it to a LoadLocal of the value being stored, matching the TS behavior.
        let (value, place) = match &last_instr.value {
            InstructionValue::StoreLocal { lvalue, value: store_value, .. } => {
                let ident = &self.env.identifiers[lvalue.place.identifier.index()];
                if ident.name.is_none() {
                    (
                        ReactiveValue::Instruction(InstructionValue::LoadLocal {
                            place: store_value.clone(),
                            span: store_value.span,
                        }),
                        lvalue.place.clone(),
                    )
                } else {
                    (
                        ReactiveValue::Instruction(last_instr.value.clone()),
                        last_instr.lvalue.clone(),
                    )
                }
            }
            _ => (ReactiveValue::Instruction(last_instr.value.clone()), last_instr.lvalue.clone()),
        };
        let id = last_instr.id;

        if remaining.is_empty() {
            ValueBlockResult { block: block_id, place, value, id }
        } else {
            ValueBlockResult {
                block: block_id,
                place,
                value: ReactiveValue::SequenceExpression {
                    instructions: remaining,
                    id,
                    value: Box::new(value),
                },
                id,
            }
        }
    }

    fn wrap_with_sequence(
        &self,
        instructions: &[InstructionId],
        continuation: ValueBlockResult<'a>,
    ) -> ValueBlockResult<'a> {
        if instructions.is_empty() {
            return continuation;
        }

        let reactive_instrs: Vec<ReactiveInstruction<'a>> = instructions
            .iter()
            .map(|iid| {
                let instr = &self.hir.instructions[iid.index()];
                ReactiveInstruction {
                    id: instr.id,
                    lvalue: Some(instr.lvalue.clone()),
                    value: ReactiveValue::Instruction(instr.value.clone()),
                    span: instr.span,
                }
            })
            .collect();

        ValueBlockResult {
            block: continuation.block,
            place: continuation.place.clone(),
            value: ReactiveValue::SequenceExpression {
                instructions: reactive_instrs,
                id: continuation.id,
                value: Box::new(continuation.value),
            },
            id: continuation.id,
        }
    }

    /// Converts the result of visit_value_block into a SequenceExpression that includes
    /// the instruction with its lvalue. This is needed for for/for-of/for-in init/test
    /// blocks where the instruction's lvalue assignment must be preserved.
    ///
    /// This also flattens nested SequenceExpressions that can occur from MaybeThrow
    /// handling in try-catch blocks.
    ///
    /// TS: valueBlockResultToSequence()
    fn value_block_result_to_sequence(
        &self,
        result: ValueBlockResult<'a>,
        span: Option<Span>,
    ) -> ReactiveValue<'a> {
        // Collect all instructions from potentially nested SequenceExpressions
        let mut instructions: Vec<ReactiveInstruction<'a>> = Vec::new();
        let mut inner_value = result.value;

        // Flatten nested SequenceExpressions
        while let ReactiveValue::SequenceExpression { instructions: seq_instrs, value, .. } =
            inner_value
        {
            instructions.extend(seq_instrs);
            inner_value = *value;
        }

        // Only add the final instruction if the innermost value is not just a LoadLocal
        // of the same place we're storing to (which would be a no-op).
        let is_load_of_same_place = match &inner_value {
            ReactiveValue::Instruction(InstructionValue::LoadLocal { place, .. }) => {
                place.identifier == result.place.identifier
            }
            _ => false,
        };

        if !is_load_of_same_place {
            instructions.push(ReactiveInstruction {
                id: result.id,
                lvalue: Some(result.place),
                value: inner_value,
                span,
            });
        }

        ReactiveValue::SequenceExpression {
            instructions,
            id: result.id,
            value: Box::new(ReactiveValue::Instruction(InstructionValue::Primitive {
                value: PrimitiveValue::Undefined,
                span,
            })),
        }
    }

    fn visit_break(
        &self,
        block: BlockId,
        id: EvaluationOrder,
    ) -> Result<Option<ReactiveStatement<'a>>, OxcDiagnostic> {
        let (target_block, target_kind) = self.cx.get_break_target(block)?;
        if self.cx.scope_fallthroughs.contains(&target_block) {
            if target_kind != ReactiveTerminalTargetKind::Implicit {
                return Err(ErrorCategory::Invariant
                    .diagnostic("Expected reactive scope to implicitly break to fallthrough"));
            }
            return Ok(None);
        }
        Ok(Some(ReactiveStatement::Terminal(ReactiveTerminalStatement {
            terminal: ReactiveTerminal::Break { target: target_block, id, target_kind },
            label: None,
        })))
    }

    fn visit_continue(
        &self,
        block: BlockId,
        id: EvaluationOrder,
    ) -> Result<ReactiveStatement<'a>, OxcDiagnostic> {
        let (target_block, target_kind) = match self.cx.get_continue_target(block) {
            Some(result) => result,
            None => {
                return Err(ErrorCategory::Invariant.diagnostic(format!(
                    "Expected continue target to be scheduled for bb{}",
                    block.index()
                )));
            }
        };

        Ok(ReactiveStatement::Terminal(ReactiveTerminalStatement {
            terminal: ReactiveTerminal::Continue { target: target_block, id, target_kind },
            label: None,
        }))
    }
}

// =============================================================================
// Helper types
// =============================================================================

struct ValueBlockResult<'a> {
    block: BlockId,
    place: Place,
    value: ReactiveValue<'a>,
    id: EvaluationOrder,
}

struct TestBlockResult<'a> {
    test: ValueBlockResult<'a>,
    consequent: BlockId,
    alternate: BlockId,
    branch_span: Option<Span>,
}

struct ValueTerminalResult<'a> {
    value: ReactiveValue<'a>,
    place: Place,
    fallthrough: BlockId,
    id: EvaluationOrder,
}

/// Helper to get span from a terminal
fn terminal_span(terminal: &Terminal) -> &Option<Span> {
    match terminal {
        Terminal::If { span, .. }
        | Terminal::Branch { span, .. }
        | Terminal::Logical { span, .. }
        | Terminal::Ternary { span, .. }
        | Terminal::Optional { span, .. }
        | Terminal::Throw { span, .. }
        | Terminal::Return { span, .. }
        | Terminal::Goto { span, .. }
        | Terminal::Switch { span, .. }
        | Terminal::DoWhile { span, .. }
        | Terminal::While { span, .. }
        | Terminal::For { span, .. }
        | Terminal::ForOf { span, .. }
        | Terminal::ForIn { span, .. }
        | Terminal::Label { span, .. }
        | Terminal::Sequence { span, .. }
        | Terminal::Unreachable { span, .. }
        | Terminal::MaybeThrow { span, .. }
        | Terminal::Scope { span, .. }
        | Terminal::PrunedScope { span, .. }
        | Terminal::Try { span, .. } => span,
    }
}
