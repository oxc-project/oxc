// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Reactive function types — tree representation of a compiled function.
//!
//! `ReactiveFunction` is derived from the HIR CFG by `BuildReactiveFunction`.
//! Control flow constructs (if/switch/loops/try) and reactive scopes become
//! nested blocks rather than block references.
//!
//! Corresponds to the reactive types in `HIR.ts`.

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use oxc_str::{Ident, Str};

use crate::react_compiler_hir::{
    BlockId, EvaluationOrder, InstructionValue, ParamPattern, Place, ScopeId,
};
use oxc_allocator::{Allocator, Box as ArenaBox, CloneIn, CloneInSemanticIds, Vec as ArenaVec};
use oxc_ast::ast::LogicalOperator;
use oxc_span::Span;

// =============================================================================
// ReactiveFunction
// =============================================================================

/// Tree representation of a compiled function, converted from the CFG-based HIR.
/// TS: ReactiveFunction in HIR.ts
#[derive(Debug)]
pub struct ReactiveFunction<'a> {
    pub span: Option<Span>,
    pub id: Option<Ident<'a>>,
    pub name_hint: Option<Ident<'a>>,
    pub params: Vec<ParamPattern>,
    pub generator: bool,
    pub is_async: bool,
    pub body: ReactiveBlock<'a>,
    pub directives: Vec<Str<'a>>,
    // No env field — passed separately per established Rust convention
}

// =============================================================================
// ReactiveBlock and ReactiveStatement
// =============================================================================

/// TS: ReactiveBlock = Array<ReactiveStatement>
pub type ReactiveBlock<'a> = oxc_allocator::Vec<'a, ReactiveStatement<'a>>;

/// TS: ReactiveStatement (discriminated union with 'kind' field)
#[derive(Debug)]
pub enum ReactiveStatement<'a> {
    Instruction(ReactiveInstruction<'a>),
    Terminal(ArenaBox<'a, ReactiveTerminalStatement<'a>>),
    Scope(ReactiveScopeBlock<'a>),
    PrunedScope(PrunedReactiveScopeBlock<'a>),
}

// =============================================================================
// ReactiveInstruction and ReactiveValue
// =============================================================================

/// TS: ReactiveInstruction
#[derive(Debug)]
pub struct ReactiveInstruction<'a> {
    pub id: EvaluationOrder,
    pub lvalue: Option<Place>,
    pub value: ReactiveValue<'a>,
    pub span: Option<Span>,
}

/// Extends InstructionValue with compound expression types that were
/// separate blocks+terminals in HIR but become nested expressions here.
/// TS: ReactiveValue = InstructionValue | ReactiveLogicalValue | ...
#[derive(Debug)]
pub enum ReactiveValue<'a> {
    /// All ~35 base instruction value kinds
    Instruction(InstructionValue<'a>),

    /// TS: ReactiveLogicalValue
    LogicalExpression {
        operator: LogicalOperator,
        left: ArenaBox<'a, ReactiveValue<'a>>,
        right: ArenaBox<'a, ReactiveValue<'a>>,
    },

    /// TS: ReactiveTernaryValue
    ConditionalExpression {
        test: ArenaBox<'a, ReactiveValue<'a>>,
        consequent: ArenaBox<'a, ReactiveValue<'a>>,
        alternate: ArenaBox<'a, ReactiveValue<'a>>,
    },

    /// TS: ReactiveSequenceValue
    SequenceExpression {
        instructions: ArenaVec<'a, ReactiveInstruction<'a>>,
        id: EvaluationOrder,
        value: ArenaBox<'a, ReactiveValue<'a>>,
    },

    /// TS: ReactiveOptionalCallValue
    OptionalExpression { value: ArenaBox<'a, ReactiveValue<'a>>, optional: bool },
}

// =============================================================================
// Terminals
// =============================================================================

#[derive(Debug)]
pub struct ReactiveTerminalStatement<'a> {
    pub terminal: ReactiveTerminal<'a>,
    pub label: Option<ReactiveLabel>,
}

#[derive(Debug, Clone)]
pub struct ReactiveLabel {
    pub id: BlockId,
    pub implicit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactiveTerminalTargetKind {
    Implicit,
    Labeled,
    Unlabeled,
}

impl Display for ReactiveTerminalTargetKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ReactiveTerminalTargetKind::Implicit => write!(f, "implicit"),
            ReactiveTerminalTargetKind::Labeled => write!(f, "labeled"),
            ReactiveTerminalTargetKind::Unlabeled => write!(f, "unlabeled"),
        }
    }
}

#[derive(Debug)]
pub enum ReactiveTerminal<'a> {
    Break {
        target: BlockId,
        id: EvaluationOrder,
        target_kind: ReactiveTerminalTargetKind,
    },
    Continue {
        target: BlockId,
        id: EvaluationOrder,
        target_kind: ReactiveTerminalTargetKind,
    },
    Return {
        value: Place,
        id: EvaluationOrder,
    },
    Throw {
        value: Place,
        id: EvaluationOrder,
    },
    Switch {
        test: Place,
        cases: ArenaVec<'a, ReactiveSwitchCase<'a>>,
        id: EvaluationOrder,
    },
    DoWhile {
        loop_block: ReactiveBlock<'a>,
        test: ReactiveValue<'a>,
        id: EvaluationOrder,
    },
    While {
        test: ReactiveValue<'a>,
        loop_block: ReactiveBlock<'a>,
        id: EvaluationOrder,
    },
    For {
        init: ReactiveValue<'a>,
        test: ReactiveValue<'a>,
        update: Option<ReactiveValue<'a>>,
        loop_block: ReactiveBlock<'a>,
        id: EvaluationOrder,
    },
    ForOf {
        init: ReactiveValue<'a>,
        test: ReactiveValue<'a>,
        loop_block: ReactiveBlock<'a>,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    ForIn {
        init: ReactiveValue<'a>,
        loop_block: ReactiveBlock<'a>,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    If {
        test: Place,
        consequent: ReactiveBlock<'a>,
        alternate: Option<ReactiveBlock<'a>>,
        id: EvaluationOrder,
    },
    Label {
        block: ReactiveBlock<'a>,
        id: EvaluationOrder,
    },
    Try {
        block: ReactiveBlock<'a>,
        handler_binding: Option<Place>,
        handler: ReactiveBlock<'a>,
        id: EvaluationOrder,
    },
}

#[derive(Debug)]
pub struct ReactiveSwitchCase<'a> {
    pub test: Option<Place>,
    pub block: Option<ReactiveBlock<'a>>,
}

// =============================================================================
// Scope Blocks
// =============================================================================

#[derive(Debug)]
pub struct ReactiveScopeBlock<'a> {
    pub scope: ScopeId,
    pub instructions: ReactiveBlock<'a>,
}

#[derive(Debug)]
pub struct PrunedReactiveScopeBlock<'a> {
    pub scope: ScopeId,
    pub instructions: ReactiveBlock<'a>,
}

// =============================================================================
// Arena `CloneIn` for the reactive subtree
//
// The reactive tree is std-`Vec`/`Box` based, but it embeds `InstructionValue`,
// which is arena-backed and no longer `Clone`. So the reactive types drop
// `derive(Clone)` and provide same-arena `CloneIn` instead (`Cloned = Self`).
// The std `Vec`/`Box` fields are cloned by recursing manually; only the embedded
// `InstructionValue` needs the allocator.
// =============================================================================

fn clone_reactive_block_in<'a>(
    block: &[ReactiveStatement<'a>],
    sem: CloneInSemanticIds,
    alloc: &'a Allocator,
) -> ArenaVec<'a, ReactiveStatement<'a>> {
    ArenaVec::from_iter_in(block.iter().map(|stmt| stmt.clone_in_impl(sem, alloc)), &alloc)
}

fn clone_reactive_instructions_in<'a>(
    instructions: &[ReactiveInstruction<'a>],
    sem: CloneInSemanticIds,
    alloc: &'a Allocator,
) -> ArenaVec<'a, ReactiveInstruction<'a>> {
    ArenaVec::from_iter_in(instructions.iter().map(|instr| instr.clone_in_impl(sem, alloc)), &alloc)
}

impl<'a> CloneIn<'a> for ReactiveFunction<'a> {
    type Cloned = ReactiveFunction<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ReactiveFunction {
            span: self.span,
            id: self.id,
            name_hint: self.name_hint,
            params: self.params.clone(),
            generator: self.generator,
            is_async: self.is_async,
            body: clone_reactive_block_in(&self.body, sem, alloc),
            directives: self.directives.clone(),
        }
    }
}

impl<'a> CloneIn<'a> for ReactiveStatement<'a> {
    type Cloned = ReactiveStatement<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        match self {
            ReactiveStatement::Instruction(instr) => {
                ReactiveStatement::Instruction(instr.clone_in_impl(sem, alloc))
            }
            ReactiveStatement::Terminal(terminal) => ReactiveStatement::Terminal(ArenaBox::new_in(
                terminal.as_ref().clone_in_impl(sem, alloc),
                &alloc,
            )),
            ReactiveStatement::Scope(scope) => {
                ReactiveStatement::Scope(scope.clone_in_impl(sem, alloc))
            }
            ReactiveStatement::PrunedScope(scope) => {
                ReactiveStatement::PrunedScope(scope.clone_in_impl(sem, alloc))
            }
        }
    }
}

impl<'a> CloneIn<'a> for ReactiveInstruction<'a> {
    type Cloned = ReactiveInstruction<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ReactiveInstruction {
            id: self.id,
            lvalue: self.lvalue,
            value: self.value.clone_in_impl(sem, alloc),
            span: self.span,
        }
    }
}

impl<'a> CloneIn<'a> for ReactiveValue<'a> {
    type Cloned = ReactiveValue<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        match self {
            ReactiveValue::Instruction(value) => {
                ReactiveValue::Instruction(value.clone_in_impl(sem, alloc))
            }
            ReactiveValue::LogicalExpression { operator, left, right } => {
                ReactiveValue::LogicalExpression {
                    operator: *operator,
                    left: ArenaBox::new_in(left.as_ref().clone_in_impl(sem, alloc), &alloc),
                    right: ArenaBox::new_in(right.as_ref().clone_in_impl(sem, alloc), &alloc),
                }
            }
            ReactiveValue::ConditionalExpression { test, consequent, alternate } => {
                ReactiveValue::ConditionalExpression {
                    test: ArenaBox::new_in(test.as_ref().clone_in_impl(sem, alloc), &alloc),
                    consequent: ArenaBox::new_in(
                        consequent.as_ref().clone_in_impl(sem, alloc),
                        &alloc,
                    ),
                    alternate: ArenaBox::new_in(
                        alternate.as_ref().clone_in_impl(sem, alloc),
                        &alloc,
                    ),
                }
            }
            ReactiveValue::SequenceExpression { instructions, id, value } => {
                ReactiveValue::SequenceExpression {
                    instructions: clone_reactive_instructions_in(instructions, sem, alloc),
                    id: *id,
                    value: ArenaBox::new_in(value.as_ref().clone_in_impl(sem, alloc), &alloc),
                }
            }
            ReactiveValue::OptionalExpression { value, optional } => {
                ReactiveValue::OptionalExpression {
                    value: ArenaBox::new_in(value.as_ref().clone_in_impl(sem, alloc), &alloc),
                    optional: *optional,
                }
            }
        }
    }
}

impl<'a> CloneIn<'a> for ReactiveTerminalStatement<'a> {
    type Cloned = ReactiveTerminalStatement<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ReactiveTerminalStatement {
            terminal: self.terminal.clone_in_impl(sem, alloc),
            label: self.label.clone(),
        }
    }
}

impl<'a> CloneIn<'a> for ReactiveTerminal<'a> {
    type Cloned = ReactiveTerminal<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        match self {
            ReactiveTerminal::Break { target, id, target_kind } => ReactiveTerminal::Break {
                target: *target,
                id: *id,
                target_kind: target_kind.clone(),
            },
            ReactiveTerminal::Continue { target, id, target_kind } => ReactiveTerminal::Continue {
                target: *target,
                id: *id,
                target_kind: target_kind.clone(),
            },
            ReactiveTerminal::Return { value, id } => {
                ReactiveTerminal::Return { value: *value, id: *id }
            }
            ReactiveTerminal::Throw { value, id } => {
                ReactiveTerminal::Throw { value: *value, id: *id }
            }
            ReactiveTerminal::Switch { test, cases, id } => ReactiveTerminal::Switch {
                test: *test,
                cases: ArenaVec::from_iter_in(
                    cases.iter().map(|case| case.clone_in_impl(sem, alloc)),
                    &alloc,
                ),
                id: *id,
            },
            ReactiveTerminal::DoWhile { loop_block, test, id } => ReactiveTerminal::DoWhile {
                loop_block: clone_reactive_block_in(loop_block, sem, alloc),
                test: test.clone_in_impl(sem, alloc),
                id: *id,
            },
            ReactiveTerminal::While { test, loop_block, id } => ReactiveTerminal::While {
                test: test.clone_in_impl(sem, alloc),
                loop_block: clone_reactive_block_in(loop_block, sem, alloc),
                id: *id,
            },
            ReactiveTerminal::For { init, test, update, loop_block, id } => ReactiveTerminal::For {
                init: init.clone_in_impl(sem, alloc),
                test: test.clone_in_impl(sem, alloc),
                update: update.as_ref().map(|value| value.clone_in_impl(sem, alloc)),
                loop_block: clone_reactive_block_in(loop_block, sem, alloc),
                id: *id,
            },
            ReactiveTerminal::ForOf { init, test, loop_block, id, span } => {
                ReactiveTerminal::ForOf {
                    init: init.clone_in_impl(sem, alloc),
                    test: test.clone_in_impl(sem, alloc),
                    loop_block: clone_reactive_block_in(loop_block, sem, alloc),
                    id: *id,
                    span: *span,
                }
            }
            ReactiveTerminal::ForIn { init, loop_block, id, span } => ReactiveTerminal::ForIn {
                init: init.clone_in_impl(sem, alloc),
                loop_block: clone_reactive_block_in(loop_block, sem, alloc),
                id: *id,
                span: *span,
            },
            ReactiveTerminal::If { test, consequent, alternate, id } => ReactiveTerminal::If {
                test: *test,
                consequent: clone_reactive_block_in(consequent, sem, alloc),
                alternate: alternate
                    .as_ref()
                    .map(|block| clone_reactive_block_in(block, sem, alloc)),
                id: *id,
            },
            ReactiveTerminal::Label { block, id } => ReactiveTerminal::Label {
                block: clone_reactive_block_in(block, sem, alloc),
                id: *id,
            },
            ReactiveTerminal::Try { block, handler_binding, handler, id } => {
                ReactiveTerminal::Try {
                    block: clone_reactive_block_in(block, sem, alloc),
                    handler_binding: *handler_binding,
                    handler: clone_reactive_block_in(handler, sem, alloc),
                    id: *id,
                }
            }
        }
    }
}

impl<'a> CloneIn<'a> for ReactiveSwitchCase<'a> {
    type Cloned = ReactiveSwitchCase<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ReactiveSwitchCase {
            test: self.test,
            block: self.block.as_ref().map(|block| clone_reactive_block_in(block, sem, alloc)),
        }
    }
}

impl<'a> CloneIn<'a> for ReactiveScopeBlock<'a> {
    type Cloned = ReactiveScopeBlock<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ReactiveScopeBlock {
            scope: self.scope,
            instructions: clone_reactive_block_in(&self.instructions, sem, alloc),
        }
    }
}

impl<'a> CloneIn<'a> for PrunedReactiveScopeBlock<'a> {
    type Cloned = PrunedReactiveScopeBlock<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        PrunedReactiveScopeBlock {
            scope: self.scope,
            instructions: clone_reactive_block_in(&self.instructions, sem, alloc),
        }
    }
}
