use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_cfg::{
    BlockNodeId, EdgeType, EvalConstConditionResult, Instruction, InstructionKind,
    LabeledInstruction,
    graph::{Direction, visit::EdgeRef},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
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

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
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
    /// while (foo) {
    ///   break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// while (foo) {
    ///   continue;
    /// }
    /// ```
    NoUnreachableLoop,
    eslint,
    nursery,
    config = NoUnreachableLoop,
    version = "next",
    short_description = "Disallow loops whose body allows only one iteration.",
);

impl Rule for NoUnreachableLoop {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (loop_type, body) = match node.kind() {
            AstKind::WhileStatement(statement) => {
                if is_static_false(&statement.test) {
                    return;
                }
                (LoopType::While, &statement.body)
            }
            AstKind::DoWhileStatement(statement) => {
                if is_static_false(&statement.test) {
                    return;
                }
                (LoopType::DoWhile, &statement.body)
            }
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

        if has_natural_next_iteration_path(node.id(), ctx) {
            return;
        }

        let unreachable =
            is_static_infinite_loop(node.kind()).then(|| ctx.effective_unreachable_blocks());

        if is_unreachable_node(node.id(), ctx, unreachable)
            || body_is_unreachable(body, ctx, unreachable)
        {
            return;
        }

        if !has_next_iteration_path(node.id(), ctx.nodes().cfg_id(body.node_id()), ctx, unreachable)
        {
            ctx.diagnostic(no_unreachable_loop_diagnostic(node.kind().span()));
        }
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

fn has_natural_next_iteration_path(loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    for edge in ctx.cfg().graph().edges_directed(ctx.nodes().cfg_id(loop_id), Direction::Incoming) {
        if !matches!(edge.weight(), EdgeType::Backedge) {
            continue;
        }

        let source = edge.source();
        if !ctx.cfg().basic_block(source).is_unreachable()
            && nearest_loop_for_block(source, ctx)
                .is_some_and(|nearest_loop| nearest_loop == loop_id)
        {
            return true;
        }
    }

    false
}

fn is_static_infinite_loop(kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::WhileStatement(statement) => is_static_true(&statement.test),
        AstKind::DoWhileStatement(statement) => is_static_true(&statement.test),
        AstKind::ForStatement(statement) => {
            statement.test.as_ref().is_none_or(|test| is_static_true(test))
        }
        _ => false,
    }
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
    let mut seen = Vec::new();

    while let Some(source) = stack.pop() {
        if seen.contains(&source) || is_unreachable_block(source, ctx, unreachable) {
            continue;
        }
        seen.push(source);

        // Same for every outgoing edge of `source`, so compute it at most once
        // per block rather than once per `Normal` edge.
        let mut source_is_infinite_loop_exit = None;

        for edge in graph.edges_directed(source, Direction::Outgoing) {
            match edge.weight() {
                EdgeType::Backedge => {
                    if owns_block(source, loop_id, ctx)
                        || (is_synthetic_continuation(source, loop_id, ctx, unreachable)
                            && is_next_iteration_target(edge.target(), loop_id, ctx))
                    {
                        return true;
                    }
                }
                EdgeType::Jump => {
                    let mut has_break = false;
                    for instruction in cfg.basic_block(source).instructions() {
                        match instruction.kind {
                            InstructionKind::Continue(LabeledInstruction::Unlabeled)
                                if can_continue_loop_from(source, loop_id, ctx, unreachable) =>
                            {
                                return true;
                            }
                            InstructionKind::Continue(LabeledInstruction::Labeled)
                                if can_continue_loop_from(
                                    edge.target(),
                                    loop_id,
                                    ctx,
                                    unreachable,
                                ) =>
                            {
                                return true;
                            }
                            InstructionKind::Break(_) => has_break = true,
                            _ => {}
                        }
                    }

                    if !has_break {
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
                | EdgeType::Finalize
                | EdgeType::Join
                | EdgeType::Error(_) => {}
            }
        }
    }

    false
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
    ctx: &LintContext<'_>,
    unreachable: Option<&[bool]>,
) -> bool {
    owns_block(block_id, loop_id, ctx)
        || is_synthetic_continuation(block_id, loop_id, ctx, unreachable)
        || is_next_iteration_target(block_id, loop_id, ctx)
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
    let mut seen = Vec::new();
    while let Some(current) = stack.pop() {
        if seen.contains(&current) {
            continue;
        }
        seen.push(current);

        for edge in graph
            .edges_directed(current, Direction::Incoming)
            .filter(|edge| matches!(edge.weight(), EdgeType::Normal | EdgeType::Jump))
        {
            let source = edge.source();
            if is_unreachable_block(source, ctx, unreachable) {
                continue;
            }

            if owns_block(source, loop_id, ctx)
                || (matches!(edge.weight(), EdgeType::Normal)
                    && can_skip_nested_loop(source, loop_id, ctx))
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

fn is_next_iteration_target(block_id: BlockNodeId, loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    owns_block(block_id, loop_id, ctx) || empty_block_backedges_to_loop(block_id, loop_id, ctx)
}

fn owns_block(block_id: BlockNodeId, loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    directly_owns_block(block_id, loop_id, ctx)
        || empty_backedge_targets_loop(block_id, loop_id, ctx)
}

fn directly_owns_block(block_id: BlockNodeId, loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    if block_id == ctx.nodes().cfg_id(loop_id) {
        return true;
    }

    ctx.cfg().basic_block(block_id).instructions().iter().any(|instruction| {
        instruction
            .node_id
            .and_then(|node_id| nearest_loop(node_id, ctx))
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

fn can_skip_nested_loop(block_id: BlockNodeId, loop_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let Some(nested_loop_id) = nearest_loop_for_block(block_id, ctx) else {
        return false;
    };
    nested_loop_id != loop_id
        && enclosing_loop(nested_loop_id, ctx).is_some_and(|id| id == loop_id)
        && loop_can_have_zero_iterations(ctx.nodes().kind(nested_loop_id))
}

fn nearest_loop_for_block(block_id: BlockNodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    ctx.cfg()
        .basic_block(block_id)
        .instructions()
        .iter()
        .find_map(|instruction| instruction.node_id.and_then(|node_id| nearest_loop(node_id, ctx)))
}

fn enclosing_loop(node_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    ctx.nodes().ancestor_ids(node_id).find(|ancestor_id| is_loop(ctx.nodes().kind(*ancestor_id)))
}

fn loop_can_have_zero_iterations(kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::WhileStatement(statement) => !is_static_true(&statement.test),
        AstKind::ForStatement(statement) => {
            statement.test.as_ref().is_some_and(|test| !is_static_true(test))
        }
        _ => false,
    }
}

fn nearest_loop(mut node_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    loop {
        if is_loop(ctx.nodes().kind(node_id)) {
            return Some(node_id);
        }
        if node_id == NodeId::ROOT {
            return None;
        }
        node_id = ctx.nodes().parent_id(node_id);
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
    use crate::tester::Tester;

    let pass = vec![
        ("while (false) { foo(); }", None),
        ("while (bar) { foo(); if (true) { break; } }", None),
        ("do foo(); while (false)", None),
        ("for (x = 1; x < 10; i++) { if (x > 0) { foo(); throw err; } }", None),
        ("for (x of []);", None),
        ("for (x of [1]);", None),
        ("function foo() { return; while (a); }", None),
        ("function foo() { return; while (a) break; }", None),
        ("while(true); while(true);", None),
        ("while(true); while(true) break;", None),
        ("while (a) break;", Some(serde_json::json!([{ "ignore": ["WhileStatement"] }]))),
        ("do break; while (a)", Some(serde_json::json!([{ "ignore": ["DoWhileStatement"] }]))),
        ("for (a; b; c) break;", Some(serde_json::json!([{ "ignore": ["ForStatement"] }]))),
        ("for (a in b) break;", Some(serde_json::json!([{ "ignore": ["ForInStatement"] }]))),
        ("for (a of b) break;", Some(serde_json::json!([{ "ignore": ["ForOfStatement"] }]))),
        (
            "for (var key in obj) { hasEnumerableProperties = true; break; } for (const a of b) break;",
            Some(serde_json::json!([{ "ignore": ["ForInStatement", "ForOfStatement"] }])),
        ),
    ];

    let fail = vec![
        ("while (foo) { for (a of b) { if (baz) { break; } else { throw err; } } }", None),
        (
            "lbl: for (var i = 0; i < 10; i++) { while (foo) break lbl; } /* outer is valid because inner can have 0 iterations */",
            None,
        ),
        ("for (a in b) { while (foo) { if(baz) { break; } else { break; } } break; }", None),
        ("function foo() { for (var i = 0; i < 10; i++) { do { return; } while(i) } }", None),
        ("lbl: while(foo) { do { break lbl; } while(baz) }", None),
        ("lbl: for (a in b) { while(foo) { continue lbl; } }", None),
        ("for (a of b) { for(;;) { if (foo) { throw err; } } }", None),
        ("function foo () { for (a in b) { while (true) { if (bar) { return; } } } }", None),
        ("do for (var i = 1; i < 10; i++) break; while(foo)", None),
        ("do { for (var i = 1; i < 10; i++) continue; break; } while(foo)", None),
        ("for (;;) { for (var i = 1; i < 10; i ++) break; if (foo) break; continue; }", None),
        (
            "while (a) break; do break; while (b); for (;;) break; for (c in d) break; for (e of f) break;",
            Some(serde_json::json!([{ "ignore": [] }])),
        ),
        ("while (a) break;", Some(serde_json::json!([{ "ignore": ["DoWhileStatement"] }]))),
        ("do break; while (a)", Some(serde_json::json!([{ "ignore": ["WhileStatement"] }]))),
        (
            "for (a in b) break; for (c of d) break;",
            Some(serde_json::json!([{ "ignore": ["ForStatement"] }])),
        ),
        (
            "for (a in b) break; for (;;) break; for (c of d) break;",
            Some(serde_json::json!([{ "ignore": ["ForInStatement", "ForOfStatement"] }])),
        ),
    ];

    Tester::new(NoUnreachableLoop::NAME, NoUnreachableLoop::PLUGIN, pass, fail).test_and_snapshot();
}
