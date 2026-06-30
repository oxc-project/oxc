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

        for node in ctx.nodes() {
            self.run_on_loop(node, ctx, &analysis);
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

        if !has_next_iteration_path(node.id(), ctx, analysis) {
            ctx.diagnostic(no_unreachable_loop_diagnostic(node.kind().span()));
        }
    }
}

fn is_static_false(expr: &Expression<'_>) -> bool {
    matches!(expr.without_parentheses(), Expression::BooleanLiteral(lit) if !lit.value)
}

fn body_is_unreachable(
    body: &Statement<'_>,
    ctx: &LintContext<'_>,
    analysis: &LoopAnalysis,
) -> bool {
    is_unreachable_node(body.node_id(), ctx, analysis)
}

fn is_unreachable_node(node_id: NodeId, ctx: &LintContext<'_>, analysis: &LoopAnalysis) -> bool {
    analysis.unreachable_blocks[ctx.nodes().cfg_id(node_id).index()]
}

fn has_next_iteration_path(
    loop_id: NodeId,
    ctx: &LintContext<'_>,
    analysis: &LoopAnalysis,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();

    graph.edge_references().any(|edge| {
        let source = edge.source();
        if analysis.unreachable_blocks[source.index()] {
            return false;
        }

        match edge.weight() {
            EdgeType::Backedge => {
                analysis.owns_block(source, loop_id)
                    || (analysis.is_synthetic_continuation(source, loop_id)
                        && (analysis.owns_block(edge.target(), loop_id)
                            || edge.target() == ctx.nodes().cfg_id(loop_id)))
            }
            EdgeType::Jump => cfg.basic_block(source).instructions().iter().any(|instruction| {
                match instruction.kind {
                    InstructionKind::Continue(LabeledInstruction::Unlabeled) => {
                        analysis.can_continue_loop_from(source, loop_id)
                    }
                    InstructionKind::Continue(LabeledInstruction::Labeled) => {
                        analysis.can_continue_loop_from(edge.target(), loop_id)
                    }
                    _ => false,
                }
            }),
            _ => false,
        }
    })
}

struct LoopAnalysis {
    unreachable_blocks: Vec<bool>,
    owned_blocks: Vec<Vec<NodeId>>,
    synthetic_continuation_blocks: Vec<Vec<NodeId>>,
}

impl LoopAnalysis {
    fn new(ctx: &LintContext<'_>) -> Self {
        let owned_blocks = collect_owned_blocks(ctx);
        let synthetic_continuation_blocks =
            collect_synthetic_continuation_blocks(ctx, &owned_blocks);

        Self {
            unreachable_blocks: effective_unreachable_blocks(ctx),
            owned_blocks,
            synthetic_continuation_blocks,
        }
    }

    fn owns_block(&self, block_id: oxc_cfg::BlockNodeId, loop_id: NodeId) -> bool {
        self.owned_blocks[block_id.index()].contains(&loop_id)
    }

    fn is_synthetic_continuation(&self, block_id: oxc_cfg::BlockNodeId, loop_id: NodeId) -> bool {
        self.synthetic_continuation_blocks[block_id.index()].contains(&loop_id)
    }

    fn can_continue_loop_from(&self, block_id: oxc_cfg::BlockNodeId, loop_id: NodeId) -> bool {
        self.owns_block(block_id, loop_id) || self.is_synthetic_continuation(block_id, loop_id)
    }
}

fn collect_owned_blocks(ctx: &LintContext<'_>) -> Vec<Vec<NodeId>> {
    let mut owned_blocks = vec![Vec::new(); ctx.cfg().basic_blocks.len()];

    for node in ctx.nodes() {
        let Some(loop_id) = closest_loop_ancestor(node.id(), ctx) else {
            continue;
        };
        push_loop_id(&mut owned_blocks, ctx.nodes().cfg_id(node.id()), loop_id);
    }

    for block_id in ctx.cfg().graph().node_indices() {
        for instruction in ctx.cfg().basic_block(block_id).instructions() {
            let Some(node_id) = instruction.node_id else {
                continue;
            };
            let Some(loop_id) = closest_loop_ancestor(node_id, ctx) else {
                continue;
            };
            push_loop_id(&mut owned_blocks, block_id, loop_id);
        }
    }

    owned_blocks
}

fn collect_synthetic_continuation_blocks(
    ctx: &LintContext<'_>,
    owned_blocks: &[Vec<NodeId>],
) -> Vec<Vec<NodeId>> {
    let mut continuation_blocks = vec![Vec::new(); ctx.cfg().basic_blocks.len()];

    for block_id in ctx.cfg().graph().node_indices() {
        if !ctx.cfg().basic_block(block_id).instructions().is_empty() {
            continue;
        }

        for edge in ctx
            .cfg()
            .graph()
            .edges_directed(block_id, Direction::Incoming)
            .filter(|edge| matches!(edge.weight(), EdgeType::Normal | EdgeType::Jump))
        {
            for loop_id in &owned_blocks[edge.source().index()] {
                push_loop_id(&mut continuation_blocks, block_id, *loop_id);
            }
        }
    }

    continuation_blocks
}

fn push_loop_id(
    loop_ids_by_block: &mut [Vec<NodeId>],
    block_id: oxc_cfg::BlockNodeId,
    loop_id: NodeId,
) {
    let loop_ids = &mut loop_ids_by_block[block_id.index()];
    if !loop_ids.contains(&loop_id) {
        loop_ids.push(loop_id);
    }
}

fn closest_loop_ancestor(node_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    if is_loop(ctx.nodes().kind(node_id)) {
        return Some(node_id);
    }

    ctx.nodes().ancestor_ids(node_id).find(|id| is_loop(ctx.nodes().kind(*id)))
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
