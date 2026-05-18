use oxc_allocator::Vec as AllocatorVec;
use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, Function, FunctionBody, ReturnStatement, Statement,
    UnaryOperator,
};
use oxc_ast_visit::Visit;
use oxc_cfg::{
    EdgeType, InstructionKind, ReturnInstructionKind,
    graph::{Direction, visit::EdgeRef},
};

use oxc_semantic::{NodeId, ScopeFlags, Semantic};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

/// `StatementReturnStatus` describes whether the CFG corresponding to
/// the statement is termitated by return statement in all/some/nome of
/// its exit blocks.
///
/// For example, an "if" statement is terminated by explicit return if and only if either:
/// 1. the test is always true and the consequent is terminated by explicit return
/// 2. the test is always false and the alternate is terminated by explicit return
/// 3. both the consequent and the alternate is terminated by explicit return
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatementReturnStatus {
    /// Only explicit return on all paths
    AlwaysExplicit,
    /// Only implicit return on all paths
    AlwaysImplicit,
    /// Explicit or implicit return on all paths (no un-returned paths)
    AlwaysMixed,

    /// Only explicit return on some paths
    SomeExplicit,
    /// Only implicit return on some paths
    SomeImplicit,
    /// Explicit and implicit return on some paths
    SomeMixed,

    /// No return on all paths
    NotReturn,
}

impl StatementReturnStatus {
    fn create(must_return: bool, maybe_explicit: bool, maybe_implicit: bool) -> Self {
        match (must_return, maybe_explicit, maybe_implicit) {
            (true, true, true) => Self::AlwaysMixed,
            (true, true, false) => Self::AlwaysExplicit,
            (true, false, true) => Self::AlwaysImplicit,
            (false, true, true) => Self::SomeMixed,
            (false, true, false) => Self::SomeExplicit,
            (false, false, true) => Self::SomeImplicit,
            (false, false, false) => Self::NotReturn,
            (true, false, false) => unreachable!(),
        }
    }

    pub fn must_return(self) -> bool {
        matches!(self, Self::AlwaysExplicit | Self::AlwaysImplicit | Self::AlwaysMixed)
    }

    pub fn may_return_explicit(self) -> bool {
        matches!(
            self,
            Self::AlwaysExplicit | Self::AlwaysMixed | Self::SomeExplicit | Self::SomeMixed
        )
    }

    pub fn may_return_implicit(self) -> bool {
        matches!(
            self,
            Self::AlwaysImplicit | Self::AlwaysMixed | Self::SomeImplicit | Self::SomeMixed
        )
    }
}

pub fn check_function_body(function_node_id: NodeId, semantic: &Semantic) -> StatementReturnStatus {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    enum PendingExit {
        #[default]
        None,
        Explicit,
        Implicit,
    }

    fn pending_exit_after_block(
        block: &oxc_cfg::BasicBlock,
        incoming_exit: PendingExit,
    ) -> PendingExit {
        let mut pending_exit = incoming_exit;

        for instruction in block.instructions() {
            pending_exit = match instruction.kind {
                InstructionKind::Return(ReturnInstructionKind::NotImplicitUndefined)
                | InstructionKind::Throw => PendingExit::Explicit,
                InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined) => {
                    PendingExit::Implicit
                }
                InstructionKind::Unreachable => break,
                _ => pending_exit,
            };

            if matches!(
                instruction.kind,
                InstructionKind::Return(_) | InstructionKind::Throw | InstructionKind::Unreachable
            ) {
                break;
            }
        }

        pending_exit
    }

    fn is_terminal_edge(edge: &EdgeType) -> bool {
        matches!(
            edge,
            EdgeType::Jump
                | EdgeType::Normal
                | EdgeType::Backedge
                | EdgeType::Finalize
                | EdgeType::Join
                | EdgeType::Error(oxc_cfg::ErrorEdgeKind::Explicit)
        )
    }

    let cfg = semantic.cfg().expect("CFG should be available");

    let mut worklist = vec![(semantic.nodes().cfg_id(function_node_id), PendingExit::None)];
    let mut seen = FxHashSet::default();
    let mut terminal_states = vec![];

    while let Some((block_id, incoming_exit)) = worklist.pop() {
        if !seen.insert((block_id, incoming_exit)) {
            continue;
        }

        let block = cfg.basic_block(block_id);
        if block.is_unreachable() {
            continue;
        }

        let state_after_block = pending_exit_after_block(block, incoming_exit);
        let mut has_successor = false;

        for edge in cfg.graph().edges_directed(block_id, Direction::Outgoing) {
            let edge_kind = edge.weight();
            if !is_terminal_edge(edge_kind) {
                continue;
            }

            has_successor = true;
            let propagated_exit =
                if matches!(edge_kind, EdgeType::Error(oxc_cfg::ErrorEdgeKind::Explicit)) {
                    incoming_exit
                } else {
                    state_after_block
                };

            worklist.push((edge.target(), propagated_exit));
        }

        if !has_successor {
            // Only treat dead ends as function exits if the path is already carrying a
            // return/throw from an earlier block (e.g. through `finally`), or if this
            // terminal block has instructions of its own. CFG lowering can otherwise
            // leave behind empty helper blocks that are not observable callback exits.
            if matches!(state_after_block, PendingExit::None) && block.instructions().is_empty() {
                continue;
            }
            terminal_states.push(state_after_block);
        }
    }

    if terminal_states.is_empty() {
        return StatementReturnStatus::NotReturn;
    }

    StatementReturnStatus::create(
        terminal_states.iter().all(|exit| !matches!(exit, PendingExit::None)),
        terminal_states.iter().any(|exit| matches!(exit, PendingExit::Explicit)),
        terminal_states.iter().any(|exit| matches!(exit, PendingExit::Implicit)),
    )
}

/// Collect spans of **explicit** return values (`return <expr>`) in the given function body.
///
/// This is used by `array-callback-return` when `checkForEach` is enabled to highlight the
/// returned value(s) which are ignored by `forEach`.
pub fn get_explicit_return_spans(function: &FunctionBody) -> Vec<Span> {
    let mut finder = ReturnStatementFinder::default();

    finder.visit_function_body(function);
    finder.spans
}

/// Collect spans of **explicit** return values (`return <expr>`) in the given function body.
///
/// This is used by `array-callback-return` when `checkForEach` and `allowVoid` is enabled to highlight the
/// returned value(s) which are not prefixed with void by `forEach`.
/// This method returns a boolean to know if the empty return span is due a filtering or not.
/// For example arrow functions like this () => x, returns 0 entries.
pub fn get_no_voided_return_spans(function: &FunctionBody, allow_void: bool) -> (Vec<Span>, bool) {
    let mut finder = ReturnStatementFinder { allow_void, ..Default::default() };

    finder.visit_function_body(function);
    (finder.spans, finder.has_void_expression)
}

#[derive(Default)]
struct ReturnStatementFinder {
    spans: Vec<Span>,
    allow_void: bool,
    has_void_expression: bool,
}

impl Visit<'_> for ReturnStatementFinder {
    fn visit_return_statement(&mut self, return_statement: &ReturnStatement) {
        let Some(argument) = &return_statement.argument else {
            return;
        };

        if is_expression_void(argument) {
            self.has_void_expression = true;
            if !self.allow_void {
                self.spans.push(argument.span());
            }
        } else {
            self.spans.push(argument.span());
        }
    }

    fn visit_function(&mut self, _func: &Function<'_>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'_>) {}
}

pub fn is_void_arrow_return(statements: &AllocatorVec<'_, Statement>) -> bool {
    if statements.is_empty() {
        return false;
    }

    if statements.len() > 1 {
        return false;
    }

    let Some(statement_return) = statements.first() else {
        return false;
    };

    let Statement::ExpressionStatement(expression_return) = statement_return else {
        return false;
    };

    is_expression_void(&expression_return.expression)
}

fn is_expression_void(statement_expression: &Expression<'_>) -> bool {
    match statement_expression {
        Expression::UnaryExpression(void_expression) => {
            void_expression.operator == UnaryOperator::Void
        }
        _ => false,
    }
}
