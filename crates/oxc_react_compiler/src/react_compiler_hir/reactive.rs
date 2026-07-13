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
use oxc_ast::ast::LogicalOperator;
use oxc_span::Span;

// =============================================================================
// ReactiveFunction
// =============================================================================

/// Tree representation of a compiled function, converted from the CFG-based HIR.
/// TS: ReactiveFunction in HIR.ts
#[derive(Debug, Clone)]
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
pub type ReactiveBlock<'a> = Vec<ReactiveStatement<'a>>;

/// TS: ReactiveStatement (discriminated union with 'kind' field)
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ReactiveStatement<'a> {
    Instruction(ReactiveInstruction<'a>),
    Terminal(ReactiveTerminalStatement<'a>),
    Scope(ReactiveScopeBlock<'a>),
    PrunedScope(PrunedReactiveScopeBlock<'a>),
}

// =============================================================================
// ReactiveInstruction and ReactiveValue
// =============================================================================

/// TS: ReactiveInstruction
#[derive(Debug, Clone)]
pub struct ReactiveInstruction<'a> {
    pub id: EvaluationOrder,
    pub lvalue: Option<Place>,
    pub value: ReactiveValue<'a>,
    pub span: Option<Span>,
}

/// Extends InstructionValue with compound expression types that were
/// separate blocks+terminals in HIR but become nested expressions here.
/// TS: ReactiveValue = InstructionValue | ReactiveLogicalValue | ...
#[derive(Debug, Clone)]
pub enum ReactiveValue<'a> {
    /// All ~35 base instruction value kinds
    Instruction(InstructionValue<'a>),

    /// TS: ReactiveLogicalValue
    LogicalExpression {
        operator: LogicalOperator,
        left: Box<ReactiveValue<'a>>,
        right: Box<ReactiveValue<'a>>,
    },

    /// TS: ReactiveTernaryValue
    ConditionalExpression {
        test: Box<ReactiveValue<'a>>,
        consequent: Box<ReactiveValue<'a>>,
        alternate: Box<ReactiveValue<'a>>,
    },

    /// TS: ReactiveSequenceValue
    SequenceExpression {
        instructions: Vec<ReactiveInstruction<'a>>,
        id: EvaluationOrder,
        value: Box<ReactiveValue<'a>>,
    },

    /// TS: ReactiveOptionalCallValue
    OptionalExpression { value: Box<ReactiveValue<'a>>, optional: bool },
}

// =============================================================================
// Terminals
// =============================================================================

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
        cases: Vec<ReactiveSwitchCase<'a>>,
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

#[derive(Debug, Clone)]
pub struct ReactiveSwitchCase<'a> {
    pub test: Option<Place>,
    pub block: Option<ReactiveBlock<'a>>,
}

// =============================================================================
// Scope Blocks
// =============================================================================

#[derive(Debug, Clone)]
pub struct ReactiveScopeBlock<'a> {
    pub scope: ScopeId,
    pub instructions: ReactiveBlock<'a>,
}

#[derive(Debug, Clone)]
pub struct PrunedReactiveScopeBlock<'a> {
    pub scope: ScopeId,
    pub instructions: ReactiveBlock<'a>,
}
