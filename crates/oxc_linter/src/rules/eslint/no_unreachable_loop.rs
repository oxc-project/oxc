use std::collections::VecDeque;

use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind, AstType,
    ast::{Expression, Statement},
};
use oxc_cfg::{
    EdgeType, InstructionKind, LabeledInstruction,
    graph::{Direction, visit::EdgeRef},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
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

const LOOP_NODE_TYPES: &AstTypesBitset = &AstTypesBitset::from_types(&[
    AstType::WhileStatement,
    AstType::DoWhileStatement,
    AstType::ForStatement,
    AstType::ForInStatement,
    AstType::ForOfStatement,
]);

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
    correctness,
    config = NoUnreachableLoop,
    version = "next",
    short_description = "Disallow loops whose body allows only one iteration.",
);

impl Rule for NoUnreachableLoop {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.semantic().nodes().contains_any(LOOP_NODE_TYPES)
    }

    fn run_once(&self, ctx: &LintContext) {
        let analysis = LoopAnalysis::new(ctx);

        for loop_id in &analysis.loop_nodes {
            self.run_on_loop(ctx.nodes().get_node(*loop_id), ctx, &analysis);
        }
    }
}

impl NoUnreachableLoop {
    fn run_on_loop<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>, analysis: &LoopAnalysis) {
        let loop_type = match node.kind() {
            AstKind::WhileStatement(statement) => {
                if is_static_false(&statement.test)
                    || body_is_unreachable(&statement.body, ctx, analysis)
                {
                    return;
                }
                LoopType::While
            }
            AstKind::DoWhileStatement(statement) => {
                if is_static_false(&statement.test)
                    || body_is_unreachable(&statement.body, ctx, analysis)
                {
                    return;
                }
                LoopType::DoWhile
            }
            AstKind::ForStatement(statement) => {
                if statement.test.as_ref().is_some_and(|test| is_static_false(test))
                    || body_is_unreachable(&statement.body, ctx, analysis)
                {
                    return;
                }
                LoopType::For
            }
            AstKind::ForInStatement(statement) => {
                if body_is_unreachable(&statement.body, ctx, analysis) {
                    return;
                }
                LoopType::ForIn
            }
            AstKind::ForOfStatement(statement) => {
                if body_is_unreachable(&statement.body, ctx, analysis) {
                    return;
                }
                LoopType::ForOf
            }
            _ => return,
        };

        if self.0.ignore.contains(&loop_type) || is_unreachable_node(node.id(), ctx, analysis) {
            return;
        }

        if !analysis.has_next_iteration_path(node.id()) {
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
    analysis: &LoopAnalysis,
) -> bool {
    is_unreachable_node(body.node_id(), ctx, analysis)
}

fn is_unreachable_node(node_id: NodeId, ctx: &LintContext<'_>, analysis: &LoopAnalysis) -> bool {
    analysis.unreachable[ctx.nodes().cfg_id(node_id).index()]
}

struct LoopAnalysis {
    loop_nodes: Vec<NodeId>,
    unreachable: Vec<bool>,
    has_next_iteration_path: Vec<bool>,
}

impl LoopAnalysis {
    fn new(ctx: &LintContext<'_>) -> Self {
        let (loop_nodes, direct_owners) = collect_direct_owned_blocks(ctx);
        let unreachable = unreachable_blocks(ctx, &loop_nodes);
        let owners = collect_owned_blocks(ctx, &direct_owners);
        let synthetic_continuations =
            collect_synthetic_continuation_blocks(ctx, &unreachable, &owners);
        let next_iteration_targets = collect_next_iteration_targets(ctx, &direct_owners, &owners);
        let has_next_iteration_path = collect_next_iteration_paths(
            ctx,
            &unreachable,
            &owners,
            &synthetic_continuations,
            &next_iteration_targets,
        );

        Self { loop_nodes, unreachable, has_next_iteration_path }
    }

    fn has_next_iteration_path(&self, loop_id: NodeId) -> bool {
        self.has_next_iteration_path[loop_id.index()]
    }
}

fn unreachable_blocks(ctx: &LintContext<'_>, loop_nodes: &[NodeId]) -> Vec<bool> {
    if loop_nodes.iter().any(|loop_id| is_static_infinite_loop(ctx.nodes().kind(*loop_id))) {
        return effective_unreachable_blocks(ctx);
    }

    let mut unreachable = vec![true; ctx.cfg().basic_blocks.len()];
    for block_id in ctx.cfg().graph().node_indices() {
        unreachable[block_id.index()] = ctx.cfg().basic_block(block_id).is_unreachable();
    }
    unreachable
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

fn collect_direct_owned_blocks(ctx: &LintContext<'_>) -> (Vec<NodeId>, Vec<Vec<NodeId>>) {
    let (mut loop_nodes, nearest_loop_by_node) = collect_nearest_loop_by_node(ctx);
    let mut direct_owners = vec![Vec::new(); ctx.cfg().basic_blocks.len()];

    for block_id in ctx.cfg().graph().node_indices() {
        for instruction in ctx.cfg().basic_block(block_id).instructions() {
            let Some(node_id) = instruction.node_id else {
                continue;
            };
            let Some(loop_id) = nearest_loop_by_node[node_id.index()] else {
                continue;
            };

            push_loop_id(&mut direct_owners[block_id.index()], loop_id);
            push_loop_id(&mut direct_owners[ctx.nodes().cfg_id(loop_id).index()], loop_id);
        }
    }

    loop_nodes.sort_unstable_by_key(|node_id| node_id.index());
    (loop_nodes, direct_owners)
}

fn collect_nearest_loop_by_node(ctx: &LintContext<'_>) -> (Vec<NodeId>, Vec<Option<NodeId>>) {
    let mut loop_nodes = Vec::new();
    let mut nearest_loop_by_node = vec![None; ctx.nodes().len()];

    for (node_id, node) in ctx.nodes().iter_enumerated() {
        if is_loop(node.kind()) {
            nearest_loop_by_node[node_id.index()] = Some(node_id);
            loop_nodes.push(node_id);
        } else if node_id != NodeId::ROOT {
            nearest_loop_by_node[node_id.index()] =
                nearest_loop_by_node[ctx.nodes().parent_id(node_id).index()];
        }
    }

    (loop_nodes, nearest_loop_by_node)
}

fn collect_owned_blocks(ctx: &LintContext<'_>, direct_owners: &[Vec<NodeId>]) -> Vec<Vec<NodeId>> {
    let mut owners = direct_owners.to_vec();

    for block_id in ctx.cfg().graph().node_indices() {
        if !ctx.cfg().basic_block(block_id).instructions().is_empty()
            || !ctx
                .cfg()
                .graph()
                .edges_directed(block_id, Direction::Incoming)
                .any(|edge| matches!(edge.weight(), EdgeType::Backedge))
        {
            continue;
        }

        let mut loop_ids = Vec::new();
        for edge in ctx
            .cfg()
            .graph()
            .edges_directed(block_id, Direction::Outgoing)
            .filter(|edge| matches!(edge.weight(), EdgeType::Backedge))
        {
            extend_loop_ids(&mut loop_ids, &direct_owners[edge.target().index()]);
        }
        extend_loop_ids(&mut owners[block_id.index()], &loop_ids);
    }

    owners
}

fn collect_synthetic_continuation_blocks(
    ctx: &LintContext<'_>,
    unreachable: &[bool],
    owners: &[Vec<NodeId>],
) -> Vec<Vec<NodeId>> {
    let mut continuation_blocks = vec![Vec::new(); ctx.cfg().basic_blocks.len()];
    let mut queue = VecDeque::new();

    for edge in ctx
        .cfg()
        .graph()
        .edge_references()
        .filter(|edge| matches!(edge.weight(), EdgeType::Normal | EdgeType::Jump))
    {
        let source = edge.source();
        let target = edge.target();

        if unreachable[source.index()]
            || !ctx.cfg().basic_block(target).instructions().is_empty()
            || owners[source.index()].is_empty()
        {
            continue;
        }

        if extend_loop_ids(&mut continuation_blocks[target.index()], &owners[source.index()]) {
            queue.push_back(target);
        }
    }

    while let Some(source) = queue.pop_front() {
        if unreachable[source.index()] {
            continue;
        }

        let loop_ids = continuation_blocks[source.index()].clone();
        for edge in ctx
            .cfg()
            .graph()
            .edges_directed(source, Direction::Outgoing)
            .filter(|edge| matches!(edge.weight(), EdgeType::Normal | EdgeType::Jump))
        {
            let target = edge.target();
            if !ctx.cfg().basic_block(target).instructions().is_empty() {
                continue;
            }

            if extend_loop_ids(&mut continuation_blocks[target.index()], &loop_ids) {
                queue.push_back(target);
            }
        }
    }

    continuation_blocks
}

fn collect_next_iteration_targets(
    ctx: &LintContext<'_>,
    direct_owners: &[Vec<NodeId>],
    owners: &[Vec<NodeId>],
) -> Vec<Vec<NodeId>> {
    let mut next_iteration_targets = owners.to_vec();

    for block_id in ctx.cfg().graph().node_indices() {
        if !ctx.cfg().basic_block(block_id).instructions().is_empty() {
            continue;
        }

        let mut loop_ids = Vec::new();
        for edge in ctx
            .cfg()
            .graph()
            .edges_directed(block_id, Direction::Outgoing)
            .filter(|edge| matches!(edge.weight(), EdgeType::Backedge))
        {
            extend_loop_ids(&mut loop_ids, &direct_owners[edge.target().index()]);
        }
        extend_loop_ids(&mut next_iteration_targets[block_id.index()], &loop_ids);
    }

    next_iteration_targets
}

fn collect_next_iteration_paths(
    ctx: &LintContext<'_>,
    unreachable: &[bool],
    owners: &[Vec<NodeId>],
    synthetic_continuations: &[Vec<NodeId>],
    next_iteration_targets: &[Vec<NodeId>],
) -> Vec<bool> {
    let mut has_next_iteration_path = vec![false; ctx.nodes().len()];

    for edge in ctx.cfg().graph().edge_references() {
        let source = edge.source();
        if unreachable[source.index()] {
            continue;
        }

        match edge.weight() {
            EdgeType::Backedge => {
                mark_loop_ids(&mut has_next_iteration_path, &owners[source.index()]);

                for loop_id in &synthetic_continuations[source.index()] {
                    if next_iteration_targets[edge.target().index()].contains(loop_id) {
                        has_next_iteration_path[loop_id.index()] = true;
                    }
                }
            }
            EdgeType::Jump => {
                for instruction in ctx.cfg().basic_block(source).instructions() {
                    match instruction.kind {
                        InstructionKind::Continue(LabeledInstruction::Unlabeled) => {
                            mark_continuable_loop_ids(
                                &mut has_next_iteration_path,
                                source.index(),
                                owners,
                                synthetic_continuations,
                                next_iteration_targets,
                            );
                        }
                        InstructionKind::Continue(LabeledInstruction::Labeled) => {
                            mark_continuable_loop_ids(
                                &mut has_next_iteration_path,
                                edge.target().index(),
                                owners,
                                synthetic_continuations,
                                next_iteration_targets,
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    has_next_iteration_path
}

fn mark_continuable_loop_ids(
    has_next_iteration_path: &mut [bool],
    block_index: usize,
    owners: &[Vec<NodeId>],
    synthetic_continuations: &[Vec<NodeId>],
    next_iteration_targets: &[Vec<NodeId>],
) {
    mark_loop_ids(has_next_iteration_path, &owners[block_index]);
    mark_loop_ids(has_next_iteration_path, &synthetic_continuations[block_index]);
    mark_loop_ids(has_next_iteration_path, &next_iteration_targets[block_index]);
}

fn mark_loop_ids(target: &mut [bool], loop_ids: &[NodeId]) {
    for loop_id in loop_ids {
        target[loop_id.index()] = true;
    }
}

fn extend_loop_ids(target: &mut Vec<NodeId>, loop_ids: &[NodeId]) -> bool {
    let original_len = target.len();
    for loop_id in loop_ids {
        push_loop_id(target, *loop_id);
    }
    target.len() != original_len
}

fn push_loop_id(loop_ids: &mut Vec<NodeId>, loop_id: NodeId) {
    if !loop_ids.contains(&loop_id) {
        loop_ids.push(loop_id);
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
