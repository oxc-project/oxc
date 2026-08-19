use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, Statement},
};
use oxc_cfg::{
    BlockNodeId, EdgeType, ErrorEdgeKind, EvalConstConditionResult, Instruction, InstructionKind,
    LabeledInstruction, ReturnInstructionKind,
    graph::{Direction, visit::EdgeRef},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{
    GlobalContext,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{IsGlobalReference, NodeId};
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::effective_unreachable_blocks,
};

fn no_unreachable_loop_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid loop. Its body allows only one iteration.")
        .with_help("Remove the loop or make at least one path continue to the next iteration.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoUnreachableLoopConfig {
    ignore: Vec<LoopType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
enum LoopType {
    #[serde(rename = "WhileStatement")]
    While,
    #[serde(rename = "DoWhileStatement")]
    DoWhile,
    #[serde(rename = "ForStatement")]
    For,
    #[serde(rename = "ForInStatement")]
    ForIn,
    #[serde(rename = "ForOfStatement")]
    ForOf,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
pub struct NoUnreachableLoop(Box<NoUnreachableLoopConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow loops whose body allows only one iteration.
    ///
    /// ### Why is this bad?
    ///
    /// A loop that always exits before a second iteration is usually accidental
    /// and can be replaced with simpler control flow.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (const item of items) {
    ///   console.log(item);
    ///   break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// for (const item of items) {
    ///   console.log(item);
    /// }
    /// ```
    NoUnreachableLoop,
    eslint,
    nursery,
    config = NoUnreachableLoop,
    version = "1.79.0",
    short_description = "Disallow loops whose body allows only one iteration.",
);

impl Rule for NoUnreachableLoop {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (loop_type, body) = match node.kind() {
            AstKind::WhileStatement(statement) => {
                if is_static_false(&statement.test) {
                    return;
                }
                (LoopType::While, &statement.body)
            }
            AstKind::DoWhileStatement(statement) => (LoopType::DoWhile, &statement.body),
            AstKind::ForStatement(statement) => {
                if statement.test.as_ref().is_some_and(|test| is_static_false(test)) {
                    return;
                }
                (LoopType::For, &statement.body)
            }
            AstKind::ForInStatement(statement) => (LoopType::ForIn, &statement.body),
            AstKind::ForOfStatement(statement) => (LoopType::ForOf, &statement.body),
            _ => return,
        };

        if self.0.ignore.contains(&loop_type) {
            return;
        }

        // First pass uses the control flow graph's own reachability. The
        // next-iteration search prunes infinite loops itself, so this alone
        // decides every case except one: a loop that is dead code *after* an
        // infinite loop is reported as reachable here (the CFG does not
        // propagate unreachability past an infinite loop).
        if is_unreachable_node(node.id(), ctx, None) || body_is_unreachable(body, ctx, None) {
            return;
        }

        if has_next_iteration_path(node.id(), ctx.nodes().cfg_id(body.node_id()), ctx, None) {
            return;
        }

        // This loop looks like a violation. Before reporting, rule out dead
        // code after a previous static infinite loop. `effective_unreachable_blocks`
        // is built only on this rare path and can only turn a report into a
        // non-report, never the other way around.
        let unreachable = effective_unreachable_blocks(ctx);
        if is_unreachable_node(node.id(), ctx, Some(&unreachable))
            || body_is_unreachable(body, ctx, Some(&unreachable))
        {
            return;
        }

        ctx.diagnostic(no_unreachable_loop_diagnostic(node.kind().span()));
    }
}

fn is_static_false(expr: &Expression<'_>) -> bool {
    matches!(expr.without_parentheses(), Expression::BooleanLiteral(lit) if !lit.value)
}

fn is_static_true(expr: &Expression<'_>) -> bool {
    matches!(expr.without_parentheses(), Expression::BooleanLiteral(lit) if lit.value)
}

fn body_is_unreachable(
    body: &Statement<'_>,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    is_unreachable_node(body.node_id(), ctx, unreachable)
}

fn is_unreachable_node(
    node_id: NodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    is_unreachable_block(ctx.nodes().cfg_id(node_id), ctx, unreachable)
}

fn is_unreachable_block(
    block_id: BlockNodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    unreachable.map_or_else(
        || ctx.cfg().basic_block(block_id).is_unreachable(),
        |unreachable| unreachable[block_id.index()],
    )
}

fn has_next_iteration_path(
    loop_id: NodeId,
    start: BlockNodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let mut stack = vec![start];
    let mut seen = vec![false; cfg.basic_blocks.len()];

    while let Some(source) = stack.pop() {
        if seen[source.index()] || is_unreachable_block(source, ctx, unreachable) {
            continue;
        }
        seen[source.index()] = true;

        // Same for every outgoing edge of `source`, so compute it at most once
        // per block rather than once per `Normal` edge.
        let mut source_is_infinite_loop_exit = None;

        for edge in graph.edges_directed(source, Direction::Outgoing) {
            match edge.weight() {
                EdgeType::Backedge => {
                    if owns_block(source, loop_id, ctx)
                        || (is_synthetic_continuation(source, loop_id, ctx, unreachable)
                            && reaches_next_iteration_target(
                                edge.target(),
                                loop_id,
                                start,
                                ctx,
                                unreachable,
                            ))
                    {
                        return true;
                    }
                }
                EdgeType::Jump => {
                    let mut has_abrupt_jump = false;
                    let jump_can_complete =
                        jump_can_complete_after_finalizer(source, ctx, unreachable);
                    for instruction in cfg.basic_block(source).instructions() {
                        match instruction.kind {
                            InstructionKind::Continue(LabeledInstruction::Unlabeled)
                                if jump_can_complete
                                    && can_continue_loop_from(
                                        source,
                                        loop_id,
                                        start,
                                        ctx,
                                        unreachable,
                                    ) =>
                            {
                                return true;
                            }
                            InstructionKind::Continue(LabeledInstruction::Labeled)
                                if jump_can_complete
                                    && can_continue_loop_from(
                                        edge.target(),
                                        loop_id,
                                        start,
                                        ctx,
                                        unreachable,
                                    ) =>
                            {
                                return true;
                            }
                            InstructionKind::Break(_)
                                if jump_can_complete
                                    && is_block_within_loop(edge.target(), loop_id, ctx) =>
                            {
                                stack.push(edge.target());
                            }
                            InstructionKind::Break(_)
                                if reaches_next_iteration_target(
                                    edge.target(),
                                    loop_id,
                                    start,
                                    ctx,
                                    unreachable,
                                ) =>
                            {
                                return true;
                            }
                            InstructionKind::Continue(_) | InstructionKind::Break(_) => {
                                has_abrupt_jump = true;
                            }
                            _ => {}
                        }
                    }

                    if !has_abrupt_jump {
                        stack.push(edge.target());
                    }
                }
                EdgeType::Normal => {
                    let is_exit = *source_is_infinite_loop_exit
                        .get_or_insert_with(|| is_static_infinite_loop_exit(source, ctx));
                    if !is_exit {
                        stack.push(edge.target());
                    }
                }
                EdgeType::Unreachable
                | EdgeType::NewFunction
                | EdgeType::Error(ErrorEdgeKind::Implicit) => {}
                // Convergence point after a finalizer completes, i.e. the
                // statement following the `try`. Only reachable through a
                // `Finalize` edge, which is gated below, so following it here
                // cannot resurrect an abrupt path.
                EdgeType::Join => {
                    stack.push(edge.target());
                }
                EdgeType::Finalize => {
                    if finalizer_can_continue_to_loop(
                        edge.target(),
                        loop_id,
                        start,
                        ctx,
                        unreachable,
                    ) {
                        return true;
                    }

                    // Control runs the finalizer and resumes after the `try`.
                    // When the block instead exits abruptly the finalizer
                    // resumes that exit, so the only way it reaches the next
                    // iteration is the `continue` the check above looks for.
                    //
                    // The builder attaches a `Finalize` edge to every block
                    // under the finalizer, not just the protected region's
                    // fallthrough exit, so `source` can be an intermediate
                    // block (an `if` condition, say) whose every branch still
                    // exits abruptly. That does not leak a next-iteration path:
                    // when the protected region cannot complete normally the
                    // builder marks the block after the `try` unreachable and
                    // emits `Unreachable` in place of `Join`, which this search
                    // does not follow. See the `try`/`finally` fail cases.
                    if block_completes_normally(source, ctx) {
                        stack.push(edge.target());
                    }
                }
                EdgeType::Error(ErrorEdgeKind::Explicit) => {
                    if explicit_error_edge_can_throw(source, ctx) {
                        stack.push(edge.target());
                    }
                }
            }
        }
    }

    false
}

fn finalizer_can_continue_to_loop(
    start: BlockNodeId,
    loop_id: NodeId,
    body_start: BlockNodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let mut stack = vec![start];
    let mut seen = vec![false; cfg.basic_blocks.len()];

    while let Some(current) = stack.pop() {
        if seen[current.index()] || is_unreachable_block(current, ctx, unreachable) {
            continue;
        }
        seen[current.index()] = true;

        let mut has_abrupt_exit = false;
        for instruction in cfg.basic_block(current).instructions() {
            match instruction.kind {
                InstructionKind::Continue(LabeledInstruction::Unlabeled)
                    if can_continue_loop_from(current, loop_id, body_start, ctx, unreachable) =>
                {
                    return true;
                }
                InstructionKind::Continue(LabeledInstruction::Labeled) => {
                    for edge in graph
                        .edges_directed(current, Direction::Outgoing)
                        .filter(|edge| matches!(edge.weight(), EdgeType::Jump))
                    {
                        if can_continue_loop_from(
                            edge.target(),
                            loop_id,
                            body_start,
                            ctx,
                            unreachable,
                        ) {
                            return true;
                        }
                    }
                    has_abrupt_exit = true;
                }
                InstructionKind::Break(_)
                | InstructionKind::Return(_)
                | InstructionKind::Throw
                | InstructionKind::Continue(_) => has_abrupt_exit = true,
                _ => {}
            }
        }

        if has_abrupt_exit {
            continue;
        }

        for edge in graph.edges_directed(current, Direction::Outgoing) {
            match edge.weight() {
                EdgeType::Normal | EdgeType::Jump | EdgeType::Backedge | EdgeType::Finalize => {
                    stack.push(edge.target());
                }
                EdgeType::Error(ErrorEdgeKind::Explicit) => {
                    if explicit_error_edge_can_throw(current, ctx) {
                        stack.push(edge.target());
                    }
                }
                EdgeType::Unreachable
                | EdgeType::NewFunction
                | EdgeType::Join
                | EdgeType::Error(ErrorEdgeKind::Implicit) => {}
            }
        }
    }

    false
}

fn jump_can_complete_after_finalizer(
    block_id: BlockNodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let mut finalizers = graph
        .edges_directed(block_id, Direction::Outgoing)
        .filter(|edge| matches!(edge.weight(), EdgeType::Finalize))
        .map(|edge| edge.target())
        .peekable();

    if finalizers.peek().is_none() {
        return true;
    }

    finalizers.any(|finalizer| finalizer_can_complete_without_override(finalizer, ctx, unreachable))
}

fn finalizer_can_complete_without_override(
    start: BlockNodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let mut stack = vec![start];
    let mut seen = vec![false; cfg.basic_blocks.len()];

    while let Some(current) = stack.pop() {
        if seen[current.index()] || is_unreachable_block(current, ctx, unreachable) {
            continue;
        }
        seen[current.index()] = true;

        if cfg.basic_block(current).instructions().iter().any(|instruction| {
            matches!(
                instruction.kind,
                InstructionKind::Break(_)
                    | InstructionKind::Continue(_)
                    | InstructionKind::Return(_)
                    | InstructionKind::Throw
            )
        }) {
            continue;
        }

        for edge in graph.edges_directed(current, Direction::Outgoing) {
            match edge.weight() {
                EdgeType::Normal | EdgeType::Jump | EdgeType::Backedge | EdgeType::Finalize => {
                    stack.push(edge.target());
                }
                EdgeType::Error(ErrorEdgeKind::Explicit) => {
                    if explicit_error_edge_can_throw(current, ctx) {
                        stack.push(edge.target());
                    }
                }
                EdgeType::Unreachable | EdgeType::Join => return true,
                EdgeType::NewFunction | EdgeType::Error(ErrorEdgeKind::Implicit) => {}
            }
        }
    }

    false
}

fn explicit_error_edge_can_throw(block_id: BlockNodeId, ctx: &LintContext<'_>) -> bool {
    let mut can_throw = false;

    for instruction in ctx.cfg().basic_block(block_id).instructions() {
        match instruction.kind {
            InstructionKind::Break(_)
            | InstructionKind::Continue(_)
            | InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined)
            | InstructionKind::ImplicitReturn
            | InstructionKind::Unreachable => return can_throw,
            InstructionKind::Throw
            | InstructionKind::Iteration(_)
            | InstructionKind::Return(ReturnInstructionKind::NotImplicitUndefined) => {
                return true;
            }
            InstructionKind::Statement | InstructionKind::Condition => {
                can_throw |= instruction
                    .node_id
                    .is_some_and(|node_id| ast_node_can_throw(ctx.nodes().kind(node_id), ctx));
            }
        }
    }

    can_throw
}

fn ast_node_can_throw<'a>(node: AstKind<'a>, ctx: &LintContext<'a>) -> bool {
    let ctx = NoUnreachableLoopSideEffectsContext { ctx };
    match node {
        AstKind::ExpressionStatement(e) => e.expression.may_have_side_effects(&ctx),
        AstKind::VariableDeclaration(e) => e.may_have_side_effects(&ctx),
        AstKind::IdentifierReference(e) => e.may_have_side_effects(&ctx),
        AstKind::TemplateLiteral(e) => e.may_have_side_effects(&ctx),
        AstKind::TaggedTemplateExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::ComputedMemberExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::StaticMemberExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::ArrayExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::Class(e) => e.may_have_side_effects(&ctx),
        AstKind::CallExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::NewExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::UnaryExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::BinaryExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::LogicalExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::AssignmentExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::UpdateExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::SequenceExpression(e) => e.may_have_side_effects(&ctx),
        AstKind::ParenthesizedExpression(e) => e.expression.may_have_side_effects(&ctx),
        AstKind::TSAsExpression(e) => e.expression.may_have_side_effects(&ctx),
        AstKind::TSSatisfiesExpression(e) => e.expression.may_have_side_effects(&ctx),
        AstKind::TSTypeAssertion(e) => e.expression.may_have_side_effects(&ctx),
        AstKind::TSNonNullExpression(e) => e.expression.may_have_side_effects(&ctx),
        AstKind::TSInstantiationExpression(e) => e.expression.may_have_side_effects(&ctx),
        AstKind::ForInStatement(_) | AstKind::ForOfStatement(_) | AstKind::AwaitExpression(_) => {
            true
        }
        _ => false,
    }
}

struct NoUnreachableLoopSideEffectsContext<'c, 'a> {
    ctx: &'c LintContext<'a>,
}

impl<'a> GlobalContext<'a> for NoUnreachableLoopSideEffectsContext<'_, 'a> {
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool {
        reference.is_global_reference(self.ctx.scoping())
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for NoUnreachableLoopSideEffectsContext<'_, 'a> {
    fn annotations(&self) -> bool {
        false
    }

    fn manual_pure_functions(&self, _callee: &Expression) -> bool {
        false
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        PropertyReadSideEffects::All
    }

    fn unknown_global_side_effects(&self) -> bool {
        true
    }
}

fn is_static_infinite_loop_exit(block_id: BlockNodeId, ctx: &LintContext<'_>) -> bool {
    ctx.cfg()
        .is_infinite_loop_start(block_id, |instruction| match instruction {
            Instruction { kind: InstructionKind::Condition, node_id: Some(id) } => {
                match ctx.nodes().kind(*id) {
                    AstKind::BooleanLiteral(lit) => EvalConstConditionResult::Eval(lit.value),
                    _ => EvalConstConditionResult::Fail,
                }
            }
            _ => EvalConstConditionResult::NotFound,
        })
        .is_some_and(|(_, loop_end)| loop_end == block_id)
}

fn can_continue_loop_from(
    block_id: BlockNodeId,
    loop_id: NodeId,
    body_start: BlockNodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    owns_block(block_id, loop_id, ctx)
        || is_synthetic_continuation(block_id, loop_id, ctx, unreachable)
        || reaches_next_iteration_target(block_id, loop_id, body_start, ctx, unreachable)
}

fn reaches_next_iteration_target(
    block_id: BlockNodeId,
    loop_id: NodeId,
    body_start: BlockNodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let mut stack = vec![block_id];
    let mut seen = vec![false; cfg.basic_blocks.len()];

    while let Some(current) = stack.pop() {
        if seen[current.index()] || is_unreachable_block(current, ctx, unreachable) {
            continue;
        }
        seen[current.index()] = true;

        if current == body_start {
            return true;
        }

        if !cfg.basic_block(current).instructions().is_empty()
            && !is_loop_control_block(current, loop_id, ctx)
        {
            continue;
        }

        for edge in graph.edges_directed(current, Direction::Outgoing) {
            match edge.weight() {
                EdgeType::Normal | EdgeType::Jump => stack.push(edge.target()),
                EdgeType::Backedge
                    if !backedge_targets_enclosing_loop(
                        edge.source(),
                        edge.target(),
                        loop_id,
                        ctx,
                    ) =>
                {
                    stack.push(edge.target());
                }
                _ => {}
            }
        }
    }

    false
}

fn backedge_targets_enclosing_loop(
    source: BlockNodeId,
    target: BlockNodeId,
    loop_id: NodeId,
    ctx: &LintContext<'_>,
) -> bool {
    if owns_block(source, loop_id, ctx) || owns_block(target, loop_id, ctx) {
        return false;
    }

    loop_owner_for_backedge_target(target, ctx).is_some_and(|owner| {
        owner != loop_id && ctx.nodes().ancestor_ids(loop_id).any(|ancestor| ancestor == owner)
    })
}

fn loop_owner_for_backedge_target(block_id: BlockNodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    if let Some(loop_id) = nearest_loop_for_block(block_id, ctx) {
        return Some(loop_id);
    }

    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let mut stack = vec![block_id];
    let mut seen = vec![false; cfg.basic_blocks.len()];

    while let Some(current) = stack.pop() {
        if seen[current.index()] || !cfg.basic_block(current).instructions().is_empty() {
            continue;
        }
        seen[current.index()] = true;

        for edge in graph.edges_directed(current, Direction::Incoming) {
            if let Some(loop_id) = loop_statement_for_block(edge.source(), ctx) {
                return Some(loop_id);
            }
        }

        for edge in graph.edges_directed(current, Direction::Outgoing) {
            if matches!(edge.weight(), EdgeType::Normal | EdgeType::Jump | EdgeType::Backedge) {
                stack.push(edge.target());
            }
        }
    }

    None
}

fn loop_statement_for_block(block_id: BlockNodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    ctx.cfg().basic_block(block_id).instructions().iter().find_map(|instruction| {
        instruction.node_id.filter(|node_id| is_loop(ctx.nodes().kind(*node_id)))
    })
}

fn is_loop_control_block(block_id: BlockNodeId, loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    ctx.cfg().basic_block(block_id).instructions().iter().any(|instruction| {
        matches!(instruction.kind, InstructionKind::Condition | InstructionKind::Iteration(_))
            && instruction
                .node_id
                .and_then(|node_id| enclosing_loop(node_id, ctx))
                .is_some_and(|nearest_loop| nearest_loop == loop_id)
    })
}

fn is_synthetic_continuation(
    block_id: BlockNodeId,
    loop_id: NodeId,
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    if !cfg.basic_block(block_id).instructions().is_empty() {
        return false;
    }

    let mut stack = vec![block_id];
    let mut seen = vec![false; cfg.basic_blocks.len()];
    while let Some(current) = stack.pop() {
        if seen[current.index()] {
            continue;
        }
        seen[current.index()] = true;

        for edge in graph
            .edges_directed(current, Direction::Incoming)
            // `Join` reaches the empty block the builder emits after a
            // finalizer. Without it, the block that resumes the loop after a
            // `try`/`finally` looks like it belongs to nothing.
            .filter(|edge| {
                matches!(edge.weight(), EdgeType::Normal | EdgeType::Jump | EdgeType::Join)
            })
        {
            let source = edge.source();
            if is_unreachable_block(source, ctx, unreachable) {
                continue;
            }

            if owns_block(source, loop_id, ctx)
                || (matches!(edge.weight(), EdgeType::Normal)
                    && nested_loop_can_complete_normally(source, loop_id, ctx))
            {
                return true;
            }

            if cfg.basic_block(source).instructions().is_empty() {
                stack.push(source);
            }
        }
    }

    false
}

/// `true` when control can fall off the end of `block_id` instead of leaving it
/// through `break`, `continue`, `return` or `throw`. An implicit throw is an
/// error edge rather than an instruction, so it does not count as abrupt here.
fn block_completes_normally(block_id: BlockNodeId, ctx: &LintContext<'_>) -> bool {
    !ctx.cfg().basic_block(block_id).instructions().iter().any(|instruction| {
        matches!(
            instruction.kind,
            InstructionKind::Break(_)
                | InstructionKind::Continue(_)
                | InstructionKind::Return(_)
                | InstructionKind::Throw
        )
    })
}

fn owns_block(block_id: BlockNodeId, loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    directly_owns_block(block_id, loop_id, ctx)
        || empty_backedge_targets_loop(block_id, loop_id, ctx)
}

fn is_block_within_loop(block_id: BlockNodeId, loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    owns_block(block_id, loop_id, ctx)
        || ctx.cfg().basic_block(block_id).instructions().iter().any(|instruction| {
            instruction.node_id.is_some_and(|node_id| {
                ctx.nodes().ancestor_ids(node_id).any(|ancestor| ancestor == loop_id)
            })
        })
}

fn directly_owns_block(block_id: BlockNodeId, loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    if block_id == ctx.nodes().cfg_id(loop_id) {
        return true;
    }

    ctx.cfg().basic_block(block_id).instructions().iter().any(|instruction| {
        instruction
            .node_id
            .and_then(|node_id| enclosing_loop(node_id, ctx))
            .is_some_and(|nearest_loop| nearest_loop == loop_id)
    })
}

fn empty_backedge_targets_loop(
    block_id: BlockNodeId,
    loop_id: NodeId,
    ctx: &LintContext<'_>,
) -> bool {
    ctx.cfg().basic_block(block_id).instructions().is_empty()
        && ctx
            .cfg()
            .graph()
            .edges_directed(block_id, Direction::Incoming)
            .any(|edge| matches!(edge.weight(), EdgeType::Backedge))
        && empty_block_backedges_to_loop(block_id, loop_id, ctx)
}

fn empty_block_backedges_to_loop(
    block_id: BlockNodeId,
    loop_id: NodeId,
    ctx: &LintContext<'_>,
) -> bool {
    ctx.cfg().basic_block(block_id).instructions().is_empty()
        && ctx
            .cfg()
            .graph()
            .edges_directed(block_id, Direction::Outgoing)
            .filter(|edge| matches!(edge.weight(), EdgeType::Backedge))
            .any(|edge| directly_owns_block(edge.target(), loop_id, ctx))
}

fn nested_loop_can_complete_normally(
    block_id: BlockNodeId,
    loop_id: NodeId,
    ctx: &LintContext<'_>,
) -> bool {
    if let Some(nested_loop_id) = nearest_loop_for_block(block_id, ctx)
        && nested_loop_id != loop_id
        && enclosing_loop(nested_loop_id, ctx).is_some_and(|id| id == loop_id)
        && loop_can_complete_normally(ctx.nodes().kind(nested_loop_id))
    {
        return true;
    }

    false
}

fn nearest_loop_for_block(block_id: BlockNodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    ctx.cfg().basic_block(block_id).instructions().iter().find_map(|instruction| {
        instruction.node_id.and_then(|node_id| enclosing_loop(node_id, ctx))
    })
}

fn enclosing_loop(node_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    ctx.nodes().ancestor_kinds(node_id).find(|ancestor| is_loop(*ancestor)).map(|n| n.node_id())
}

fn loop_can_complete_normally(kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::WhileStatement(statement) => !is_static_true(&statement.test),
        AstKind::DoWhileStatement(statement) => !is_static_true(&statement.test),
        AstKind::ForStatement(statement) => {
            statement.test.as_ref().is_some_and(|test| !is_static_true(test))
        }
        AstKind::ForInStatement(_) | AstKind::ForOfStatement(_) => true,
        _ => false,
    }
}

fn is_loop(kind: AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
    )
}

#[test]
fn test() {
    use crate::tester::{TestCase, Tester};

    let loop_templates: &[&[&str]] = &[
        &["while (a) <body>", "while (a && b) <body>"],
        &["do <body> while (a)", "do <body> while (a && b)"],
        &[
            "for (a; b; c) <body>",
            "for (var i = 0; i < a.length; i++) <body>",
            "for (; b; c) <body>",
            "for (; b < foo; c++) <body>",
            "for (a; ; c) <body>",
            "for (a = 0; ; c++) <body>",
            "for (a; b;) <body>",
            "for (a = 0; b < foo; ) <body>",
            "for (; ; c) <body>",
            "for (; ; c++) <body>",
            "for (; b;) <body>",
            "for (; b < foo; ) <body>",
            "for (a; ;) <body>",
            "for (a = 0; ;) <body>",
            "for (;;) <body>",
        ],
        &[
            "for (a in b) <body>",
            "for (a in f(b)) <body>",
            "for (var a in b) <body>",
            "for (let a in f(b)) <body>",
        ],
        &[
            "for (a of b) <body>",
            "for (a of f(b)) <body>",
            "for ({ a, b } of c) <body>",
            "for (var a of f(b)) <body>",
            "async function foo() { for await (const a of b) <body> }",
        ],
    ];

    let valid_loop_bodies = &[
        ";",
        "{}",
        "{ bar(); }",
        "continue;",
        "{ continue; }",
        "{ if (foo) break; }",
        "{ if (foo) { return; } bar(); }",
        "{ if (foo) { bar(); } else { break; } }",
        "{ if (foo) { continue; } return; }",
        "{ switch (foo) { case 1: return; } }",
        "{ switch (foo) { case 1: break; default: return; } }",
        "{ switch (foo) { case 1: break; default: break; } bar(); }",
        "{ switch (foo) { case 1: continue; default: return; } throw err; }",
        "{ try { return bar(); } catch (e) {} }",
        "{ continue; break; }",
        "() => a;",
        "{ () => a }",
        "(() => a)();",
        "{ (() => a)() }",
        "while (a);",
        "do ; while (a)",
        "for (a; b; c);",
        "for (; b;);",
        "for (; ; c) if (foo) break;",
        "for (;;) if (foo) break;",
        "while (true) if (foo) break;",
        "while (foo) if (bar) return;",
        "for (a in b);",
        "for (a of b);",
    ];

    let invalid_loop_bodies = &[
        "break;",
        "{ break; }",
        "return;",
        "{ return; }",
        "throw err;",
        "{ throw err; }",
        "{ foo(); break; }",
        "{ break; foo(); }",
        "if (foo) break; else return;",
        "{ if (foo) { return; } else { break; } bar(); }",
        "{ if (foo) { return; } throw err; }",
        "{ switch (foo) { default: throw err; } }",
        "{ switch (foo) { case 1: throw err; default: return; } }",
        "{ switch (foo) { case 1: something(); default: return; } }",
        "{ try { return bar(); } catch (e) { break; } }",
        "{ break; continue; }",
        "{ () => a; break; }",
        "{ (() => a)(); break; }",
        "{ while (a); break; }",
        "{ do ; while (a); break; }",
        "{ for (a; b; c); break; }",
        "{ for (; b;); break; }",
        "{ for (; ; c) if (foo) break; break; }",
        "{ for(;;) if (foo) break; break; }",
        "{ for (a in b); break; }",
        "{ for (a of b); break; }",
        "for (;;);",
        "{ for (var i = 0; ; i< 10) { foo(); } }",
        "while (true);",
    ];

    let source_code = |template: &str, body: &str| {
        let (prefix, suffix) = template.split_once("<body>").unwrap();
        let loop_source = format!("{prefix}{body}{suffix}");
        if body.contains("return") && !template.contains("function") {
            format!("function someFunc() {{ {loop_source} }}")
        } else {
            loop_source
        }
    };

    let mut pass = Vec::<TestCase>::new();
    for templates in loop_templates {
        for template in *templates {
            for body in valid_loop_bodies {
                pass.push(source_code(template, body).into());
            }
        }
    }
    pass.extend([
        ("while (false) { foo(); }", None).into(),
        ("while (bar) { foo(); if (true) { break; } }", None).into(),
        ("do foo(); while (false)", None).into(),
        ("for (x = 1; x < 10; i++) { if (x > 0) { foo(); throw err; } }", None).into(),
        ("for (x of []);", None).into(),
        ("for (x of [1]);", None).into(),
        ("function foo() { return; while (a); }", None).into(),
        ("function foo() { return; while (a) break; }", None).into(),
        ("while(true); while(true);", None).into(),
        ("while(true); while(true) break;", None).into(),
        ("while (true) {} while (foo) break;", None).into(),
        ("while (a) { try { continue; } finally {} }", None).into(),
        // A `try` body that completes normally runs the finalizer and then
        // resumes after the `try`, so every loop below iterates.
        ("while (a) { try { foo(); } finally { bar(); } }", None).into(),
        ("while (a) { try { foo(); } finally {} }", None).into(),
        ("while (a) { try { foo(); } catch (e) { bar(); } finally { baz(); } }", None).into(),
        ("while (a) { try { foo(); } finally { bar(); } baz(); }", None).into(),
        ("function f() { while (a) { try { foo(); } catch (e) { return; } finally { bar(); } } }", None)
            .into(),
        ("while (a) { try { foo(); } finally { continue; } }", None).into(),
        ("function f() { while (a) { try { foo(); } finally { if (a) continue; else return; } } }", None)
            .into(),
        ("while (a) { try { try { foo(); } finally { bar(); } } finally { baz(); } }", None).into(),
        // A branch that can fall through keeps the loop alive, even when a
        // sibling branch exits.
        ("function f() { while (a) { try { if (b) { return; } } finally { bar(); } } }", None)
            .into(),
        ("function f() { while (a) { try { if (b) return; foo(); } finally { bar(); } } }", None)
            .into(),
        ("function f() { while (a) { try { if (b) continue; else return; } finally { bar(); } } }", None)
            .into(),
        // Nothing falls out of the `try`, but the `catch` completes normally.
        ("function f() { while (a) { try { if (b) return; else return; } catch (e) {} finally { bar(); } } }", None)
            .into(),
        ("for (const x of xs) { try { foo(x); } finally { bar(x); } }", None).into(),
        ("for (let i = 0; i < a; i++) { try { foo(); } finally { bar(); } }", None).into(),
        ("for (const k in a) { try { foo(); } finally { bar(); } }", None).into(),
        ("do { try { foo(); } finally { bar(); } } while (a);", None).into(),
        ("outer: while (a) { try { foo(); } finally { continue outer; } }", None).into(),
        ("while (a) { try { break; } finally { continue; } }", None).into(),
        ("while (a) { try { break; } finally { if (foo) continue; } }", None).into(),
        ("while (a) { try { throw err; } finally { continue; } }", None).into(),
        ("function foo() { while (a) { try { return; } finally { continue; } } }", None).into(),
        (
            "while (a) break;",
            Some(serde_json::json!([{ "ignore": ["WhileStatement"] }])),
        )
            .into(),
        (
            "do break; while (a)",
            Some(serde_json::json!([{ "ignore": ["DoWhileStatement"] }])),
        )
            .into(),
        (
            "for (a; b; c) break;",
            Some(serde_json::json!([{ "ignore": ["ForStatement"] }])),
        )
            .into(),
        (
            "for (a in b) break;",
            Some(serde_json::json!([{ "ignore": ["ForInStatement"] }])),
        )
            .into(),
        (
            "for (a of b) break;",
            Some(serde_json::json!([{ "ignore": ["ForOfStatement"] }])),
        )
            .into(),
        (
            "for (var key in obj) { hasEnumerableProperties = true; break; } for (const a of b) break;",
            Some(serde_json::json!([{ "ignore": ["ForInStatement", "ForOfStatement"] }])),
        )
            .into(),
        (
            "function foo() { for (let i = 0; i <= 3; i++) { try { bar(); return; } catch (error) {} } }",
            None,
        )
            .into(),
        (
            "function f() { while (a) { try { if (mayThrow()) {} return; } catch { continue; } } }",
            None,
        )
        .into(),
        ("while (a) { try { for (const x of iterable) {} return; } catch { continue; } }", None).into()
    ]);

    let mut fail = Vec::<TestCase>::new();
    for templates in loop_templates {
        for template in *templates {
            for body in invalid_loop_bodies {
                fail.push(source_code(template, body).into());
            }
        }
    }
    fail.extend([
        // The finalizer runs, then resumes the abrupt exit that entered it.
        // The `if`/`switch`/inner-loop shapes matter because the builder gives
        // every block under the finalizer a `Finalize` edge, so the search also
        // sees intermediate blocks that carry no abrupt instruction themselves.
        ("function f() { while (a) { try { if (b) return; else return; } finally { bar(); } } }", None)
            .into(),
        ("while (a) { try { if (b) break; else break; } finally { bar(); } }", None).into(),
        ("function f() { while (a) { try { if (b) return; else throw err; } finally { bar(); } } }", None)
            .into(),
        ("function f() { while (a) { try { switch (b) { case 1: return; default: return; } } finally { bar(); } } }", None)
            .into(),
        ("function f() { while (a) { try { for (;;) { return; } } finally { bar(); } } }", None)
            .into(),
        ("function f() { while (a) { try { foo(); } finally { return; } } }", None).into(),
        ("while (a) { try { foo(); } finally { break; } }", None).into(),
        ("function f() { while (a) { try { return; } finally { bar(); } } }", None).into(),
        ("while (a) { try { break; } finally { bar(); } }", None).into(),
        ("while (a) { try { throw err; } finally { bar(); } }", None).into(),
        ("function f() { while (a) { try { foo(); } finally { bar(); } return; } }", None).into(),
        ("function f() { while (a) { try { return; } catch (e) { return; } finally { bar(); } } }", None)
            .into(),
        ("function f() { while (a) { try { ; return; } catch { continue; } } }", None).into(),
        ("function f() { while (a) { try { let value = 1; return; } catch { continue; } } }", None)
            .into(),
        ("while (foo) { for (a of b) { if (baz) { break; } else { throw err; } } }", None)
            .into(),
        (
            "lbl: for (var i = 0; i < 10; i++) { while (foo) break lbl; } /* outer is valid because inner can have 0 iterations */",
            None,
        )
            .into(),
        (
            "for (a in b) { while (foo) { if(baz) { break; } else { break; } } break; }",
            None,
        )
            .into(),
        (
            "function foo() { for (var i = 0; i < 10; i++) { do { return; } while(i) } }",
            None,
        )
            .into(),
        ("lbl: while(foo) { do { break lbl; } while(baz) }", None).into(),
        ("lbl: for (a in b) { while(foo) { continue lbl; } }", None).into(),
        ("for (a of b) { for(;;) { if (foo) { throw err; } } }", None).into(),
        (
            "function foo () { for (a in b) { while (true) { if (bar) { return; } } } }",
            None,
        )
            .into(),
        ("do for (var i = 1; i < 10; i++) break; while(foo)", None).into(),
        ("do { for (var i = 1; i < 10; i++) continue; break; } while(foo)", None).into(),
        ("for (;;) { for (var i = 1; i < 10; i ++) break; if (foo) break; continue; }", None)
            .into(),
        (
            "while (a) break; do break; while (b); for (;;) break; for (c in d) break; for (e of f) break;",
            Some(serde_json::json!([{ "ignore": [] }])),
        )
            .into(),
        ("do { break; } while (false)", None).into(),
        ("function foo() { do { return; } while (false) }", None).into(),
        ("while (a) { try { break; } catch { } }", None).into(),
        ("function foo() { while (a) { try { return; } catch { } } }", None).into(),
        ("while (a) { try { continue; } finally { break; } }", None).into(),
        (
            "while (a) { label: try { break label; } finally { break; } bar(); }",
            None,
        )
            .into(),
        ("function foo() { while (a) { try { continue; } finally { return; } } }", None).into(),
        ("for (;;) { while (a) break; }", None).into(),
        (
            "while (a) { while (true) { switch (foo) { default: break; } break; } }",
            None,
        )
            .into(),
        (
            "while (a) break;",
            Some(serde_json::json!([{ "ignore": ["DoWhileStatement"] }])),
        )
            .into(),
        (
            "do break; while (a)",
            Some(serde_json::json!([{ "ignore": ["WhileStatement"] }])),
        )
            .into(),
        (
            "for (a in b) break; for (c of d) break;",
            Some(serde_json::json!([{ "ignore": ["ForStatement"] }])),
        )
            .into(),
        (
            "for (a in b) break; for (;;) break; for (c of d) break;",
            Some(serde_json::json!([{ "ignore": ["ForInStatement", "ForOfStatement"] }])),
        )
            .into(),
    ]);

    Tester::new(NoUnreachableLoop::NAME, NoUnreachableLoop::PLUGIN, pass, fail).test_and_snapshot();
}
