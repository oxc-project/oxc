use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_cfg::BlockNodeId;
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{ToBoolean, WithoutGlobalReferenceInformation};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use smallvec::SmallVec;

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
    version = "next",
    short_description = "Disallow loops whose body allows only one iteration.",
);

impl Rule for NoUnreachableLoop {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (loop_type, body, test_prevents_body) = match node.kind() {
            AstKind::WhileStatement(statement) => {
                (LoopType::While, &statement.body, is_static_false(&statement.test))
            }
            AstKind::DoWhileStatement(statement) => (LoopType::DoWhile, &statement.body, false),
            AstKind::ForStatement(statement) => {
                let test_prevents_body =
                    statement.test.as_ref().is_some_and(|test| is_static_false(test));
                (LoopType::For, &statement.body, test_prevents_body)
            }
            AstKind::ForInStatement(statement) => (LoopType::ForIn, &statement.body, false),
            AstKind::ForOfStatement(statement) => (LoopType::ForOf, &statement.body, false),
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
        if is_unreachable_node(node.id(), ctx, None)
            || (!test_prevents_body && body_is_unreachable(body, ctx, None))
        {
            return;
        }

        if loop_body_allows_next_iteration(node.id(), body, ctx) {
            return;
        }

        // This loop looks like a violation. Before reporting, rule out the
        // dead-code case where a previous static infinite loop rendered this
        // loop unreachable. The corrected reachability map is built only on this
        // rare path. `effective_unreachable_blocks` only ever marks *more*
        // blocks unreachable than the CFG, so it can only turn a report into a
        // non-report, never the other way around.
        let unreachable = effective_unreachable_blocks(ctx);
        if is_unreachable_node(node.id(), ctx, Some(&unreachable))
            || (!test_prevents_body && body_is_unreachable(body, ctx, Some(&unreachable)))
        {
            return;
        }

        ctx.diagnostic(no_unreachable_loop_diagnostic(node.kind().span()));
    }
}

fn is_static_false(expr: &Expression<'_>) -> bool {
    static_primitive_boolean(expr) == Some(false)
}

fn is_static_true(expr: &Expression<'_>) -> bool {
    static_primitive_boolean(expr) == Some(true)
}

fn static_primitive_boolean(expr: &Expression<'_>) -> Option<bool> {
    let expr = expr.without_parentheses();
    // `ToBoolean` also evaluates object/array/function expressions to `true`, but ESLint's
    // loop code path logic still treats those tests as possibly throwing while evaluated.
    if matches!(
        expr,
        Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_)
    ) {
        expr.to_boolean(&WithoutGlobalReferenceInformation)
    } else {
        None
    }
}

fn expression_can_throw_to_catch(expr: &Expression<'_>) -> bool {
    static_primitive_boolean(expr).is_none()
}

fn loop_test_can_throw_to_catch(expr: &Expression<'_>) -> bool {
    !is_static_true(expr)
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

#[derive(Debug, Default, Clone)]
struct FlowSummary<'a> {
    normal: bool,
    continues_to_target: bool,
    breaks_to_target: bool,
    breaks_switch: bool,
    can_throw_to_catch: bool,
    breaks_to_labels: SmallVec<[&'a str; 1]>,
}

impl<'a> FlowSummary<'a> {
    fn normal() -> Self {
        Self { normal: true, ..Self::default() }
    }

    fn continue_to_target() -> Self {
        Self { continues_to_target: true, ..Self::default() }
    }

    fn break_to_target() -> Self {
        Self { breaks_to_target: true, ..Self::default() }
    }

    fn break_to_label(label: &'a str) -> Self {
        Self { breaks_to_labels: SmallVec::from_slice(&[label]), ..Self::default() }
    }

    fn abrupt() -> Self {
        Self::default()
    }

    fn merge(self, other: Self) -> Self {
        Self {
            normal: self.normal || other.normal,
            continues_to_target: self.continues_to_target || other.continues_to_target,
            breaks_to_target: self.breaks_to_target || other.breaks_to_target,
            breaks_switch: self.breaks_switch || other.breaks_switch,
            can_throw_to_catch: self.can_throw_to_catch || other.can_throw_to_catch,
            breaks_to_labels: {
                let mut labels = self.breaks_to_labels;
                labels.extend(other.breaks_to_labels);
                labels
            },
        }
    }

    fn without_normal(mut self) -> Self {
        self.normal = false;
        self
    }

    fn without_switch_break(mut self) -> Self {
        self.breaks_switch = false;
        self
    }

    fn consume_label(&mut self, label: &str) -> bool {
        let original_len = self.breaks_to_labels.len();
        self.breaks_to_labels.retain(|break_label| *break_label != label);
        self.breaks_to_labels.len() != original_len
    }
}

fn loop_body_allows_next_iteration(
    loop_id: NodeId,
    body: &Statement<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let summary = summarize_statement(body, loop_id, ctx, false);
    summary.normal || summary.continues_to_target
}

fn summarize_statement<'a>(
    statement: &'a Statement<'a>,
    target_loop: NodeId,
    ctx: &LintContext<'_>,
    in_switch: bool,
) -> FlowSummary<'a> {
    match statement {
        Statement::BlockStatement(block) => {
            summarize_statement_list(&block.body, target_loop, ctx, in_switch)
        }
        Statement::BreakStatement(statement) => {
            if statement.label.is_none() && in_switch {
                FlowSummary { normal: true, breaks_switch: true, ..FlowSummary::default() }
            } else if break_targets_loop(
                statement.node_id.get(),
                statement.label.as_ref(),
                target_loop,
                ctx,
            ) {
                FlowSummary::break_to_target()
            } else if let Some(label) = &statement.label {
                FlowSummary::break_to_label(label.name.as_str())
            } else {
                FlowSummary::abrupt()
            }
        }
        Statement::ContinueStatement(statement) => {
            if continue_targets_loop(
                statement.node_id.get(),
                statement.label.as_ref(),
                target_loop,
                ctx,
            ) {
                FlowSummary::continue_to_target()
            } else {
                FlowSummary::abrupt()
            }
        }
        Statement::ReturnStatement(statement) => FlowSummary {
            can_throw_to_catch: statement.argument.is_some(),
            ..FlowSummary::abrupt()
        },
        Statement::ThrowStatement(_) => {
            FlowSummary { can_throw_to_catch: true, ..FlowSummary::abrupt() }
        }
        Statement::IfStatement(statement) => {
            let consequent =
                summarize_statement(&statement.consequent, target_loop, ctx, in_switch);
            let alternate =
                statement.alternate.as_ref().map_or_else(FlowSummary::normal, |alternate| {
                    summarize_statement(alternate, target_loop, ctx, in_switch)
                });
            FlowSummary { can_throw_to_catch: true, ..consequent.merge(alternate) }
        }
        Statement::SwitchStatement(statement) => summarize_switch(statement, target_loop, ctx),
        Statement::TryStatement(statement) => {
            let block_summary =
                summarize_statement_list(&statement.block.body, target_loop, ctx, in_switch);
            let mut summary = if let Some(handler) = &statement.handler {
                if block_summary.can_throw_to_catch {
                    let handler_summary =
                        summarize_statement_list(&handler.body.body, target_loop, ctx, in_switch);
                    FlowSummary {
                        can_throw_to_catch: handler_summary.can_throw_to_catch,
                        ..block_summary.merge(handler_summary)
                    }
                } else {
                    FlowSummary { can_throw_to_catch: false, ..block_summary }
                }
            } else {
                block_summary
            };

            if let Some(finalizer) = &statement.finalizer {
                let finalizer =
                    summarize_statement_list(&finalizer.body, target_loop, ctx, in_switch);
                if !finalizer.normal {
                    if !summary.normal
                        && !finally_continue_targets_do_while(
                            &summary,
                            &finalizer,
                            target_loop,
                            ctx,
                        )
                    {
                        let mut finalizer = finalizer;
                        finalizer.continues_to_target = summary.continues_to_target;
                        return finalizer;
                    }
                    return finalizer;
                }
                if summary.normal {
                    summary = summary.merge(finalizer.without_normal());
                } else if finalizer.can_throw_to_catch {
                    summary.can_throw_to_catch = true;
                }
            }

            summary
        }
        Statement::LabeledStatement(statement) => {
            let mut summary = summarize_statement(&statement.body, target_loop, ctx, in_switch);
            if summary.consume_label(statement.label.name.as_str()) {
                summary.normal = true;
            }
            summary
        }
        Statement::WhileStatement(statement) => summarize_nested_loop(
            statement.node_id.get(),
            &statement.body,
            !is_static_true(&statement.test),
            loop_test_can_throw_to_catch(&statement.test),
            target_loop,
            ctx,
        ),
        Statement::DoWhileStatement(statement) => {
            let body = summarize_statement(&statement.body, statement.node_id.get(), ctx, false);
            let completes = body.breaks_to_target
                || (!is_static_true(&statement.test) && (body.normal || body.continues_to_target));
            let outer = summarize_statement(&statement.body, target_loop, ctx, false);
            let test_can_throw = (body.normal || body.continues_to_target)
                && loop_test_can_throw_to_catch(&statement.test);
            FlowSummary { normal: completes, can_throw_to_catch: test_can_throw, ..outer }
        }
        Statement::ForStatement(statement) => {
            let can_skip = statement.test.as_ref().is_some_and(|test| !is_static_true(test));
            summarize_for(statement, can_skip, target_loop, ctx)
        }
        Statement::ForInStatement(statement) => summarize_nested_loop(
            statement.node_id.get(),
            &statement.body,
            true,
            true,
            target_loop,
            ctx,
        ),
        Statement::ForOfStatement(statement) => summarize_nested_loop(
            statement.node_id.get(),
            &statement.body,
            true,
            true,
            target_loop,
            ctx,
        ),
        Statement::WithStatement(statement) => FlowSummary {
            can_throw_to_catch: true,
            ..summarize_statement(&statement.body, target_loop, ctx, in_switch)
        },
        _ => FlowSummary { can_throw_to_catch: true, ..FlowSummary::normal() },
    }
}

fn summarize_statement_list<'a>(
    statements: &'a [Statement<'a>],
    target_loop: NodeId,
    ctx: &LintContext<'_>,
    in_switch: bool,
) -> FlowSummary<'a> {
    let mut summary = FlowSummary::default();
    let mut reachable = true;

    for statement in statements {
        if !reachable {
            break;
        }

        let statement_summary = summarize_statement(statement, target_loop, ctx, in_switch);
        if statement_summary.breaks_switch {
            summary = summary.merge(statement_summary);
            summary.normal = true;
            return summary;
        }
        reachable = statement_summary.normal;
        summary = summary.merge(statement_summary);
    }

    summary.normal = reachable;
    summary
}

fn finally_continue_targets_do_while(
    summary: &FlowSummary<'_>,
    finalizer: &FlowSummary<'_>,
    target_loop: NodeId,
    ctx: &LintContext<'_>,
) -> bool {
    !summary.breaks_to_target
        && finalizer.continues_to_target
        && matches!(ctx.nodes().kind(target_loop), AstKind::DoWhileStatement(_))
}

fn summarize_switch<'a>(
    statement: &'a oxc_ast::ast::SwitchStatement<'a>,
    target_loop: NodeId,
    ctx: &LintContext<'_>,
) -> FlowSummary<'a> {
    let mut summary = FlowSummary::default();
    let has_default = statement.cases.iter().any(|case| case.test.is_none());

    if !has_default {
        summary.normal = true;
    }
    summary.can_throw_to_catch = true;

    for index in 0..statement.cases.len() {
        let mut case_summary = FlowSummary::default();
        let mut reachable = true;

        for case in statement.cases.iter().skip(index) {
            if !reachable {
                break;
            }

            let consequent = summarize_statement_list(&case.consequent, target_loop, ctx, true);
            let breaks_switch = consequent.breaks_switch;
            let consequent_normal = consequent.normal;
            case_summary = case_summary.merge(consequent.without_normal().without_switch_break());
            if breaks_switch {
                case_summary.normal = true;
                reachable = false;
                break;
            }
            reachable = consequent_normal;
        }

        if reachable {
            case_summary.normal = true;
        }
        summary = summary.merge(case_summary);
    }

    summary
}

fn summarize_nested_loop<'a>(
    loop_id: NodeId,
    body: &'a Statement<'a>,
    can_skip: bool,
    loop_head_can_throw: bool,
    target_loop: NodeId,
    ctx: &LintContext<'_>,
) -> FlowSummary<'a> {
    let outer = summarize_statement(body, target_loop, ctx, false);
    FlowSummary {
        normal: can_skip || summarize_statement(body, loop_id, ctx, false).breaks_to_target,
        continues_to_target: outer.continues_to_target,
        breaks_to_target: outer.breaks_to_target,
        breaks_switch: outer.breaks_switch,
        can_throw_to_catch: loop_head_can_throw || outer.can_throw_to_catch,
        breaks_to_labels: outer.breaks_to_labels,
    }
}

fn summarize_for<'a>(
    statement: &'a oxc_ast::ast::ForStatement<'a>,
    can_skip: bool,
    target_loop: NodeId,
    ctx: &LintContext<'_>,
) -> FlowSummary<'a> {
    let outer = summarize_statement(&statement.body, target_loop, ctx, false);
    let body = summarize_statement(&statement.body, statement.node_id.get(), ctx, false);
    let update_can_run = body.normal || body.continues_to_target;
    let can_throw_to_catch = statement
        .init
        .as_ref()
        .is_some_and(|init| init.as_expression().is_none_or(expression_can_throw_to_catch))
        || statement.test.as_ref().is_some_and(loop_test_can_throw_to_catch)
        || (update_can_run && statement.update.as_ref().is_some_and(expression_can_throw_to_catch))
        || outer.can_throw_to_catch;

    FlowSummary {
        normal: can_skip || body.breaks_to_target,
        continues_to_target: outer.continues_to_target,
        breaks_to_target: outer.breaks_to_target,
        breaks_switch: outer.breaks_switch,
        can_throw_to_catch,
        breaks_to_labels: outer.breaks_to_labels,
    }
}

fn continue_targets_loop(
    node_id: NodeId,
    label: Option<&oxc_ast::ast::LabelIdentifier<'_>>,
    target_loop: NodeId,
    ctx: &LintContext<'_>,
) -> bool {
    label.map_or_else(
        || nearest_loop(node_id, ctx).is_some_and(|loop_id| loop_id == target_loop),
        |label| label_targets_loop(label.name.as_str(), target_loop, ctx),
    )
}

fn break_targets_loop(
    node_id: NodeId,
    label: Option<&oxc_ast::ast::LabelIdentifier<'_>>,
    target_loop: NodeId,
    ctx: &LintContext<'_>,
) -> bool {
    label.map_or_else(
        || nearest_breakable(node_id, ctx).is_some_and(|loop_id| loop_id == target_loop),
        |label| label_targets_loop(label.name.as_str(), target_loop, ctx),
    )
}

fn nearest_breakable(mut node_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    loop {
        let kind = ctx.nodes().kind(node_id);
        if is_loop(kind) {
            return Some(node_id);
        }
        if matches!(kind, AstKind::SwitchStatement(_)) {
            return None;
        }
        if node_id == NodeId::ROOT {
            return None;
        }
        node_id = ctx.nodes().parent_id(node_id);
    }
}

fn label_targets_loop(label_name: &str, target_loop: NodeId, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().ancestor_ids(target_loop).any(|ancestor_id| {
        matches!(
            ctx.nodes().kind(ancestor_id),
            AstKind::LabeledStatement(statement)
                if statement.label.name.as_str() == label_name
                    && statement.body.node_id() == target_loop
        )
    })
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
        "{ do ; while (a) break; }",
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
        let loop_source = template.replace("<body>", body);
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
        ("while (a) { label: { break label; } }", None).into(),
        ("while (a) { try { continue; } finally { break; } }", None).into(),
        ("function foo() { do { try { return; } finally { continue; } } while (a) }", None)
            .into(),
        ("do { try { throw err; } finally { continue; } } while (a)", None).into(),
        (
            "function foo() { do { try { return; } catch (e) { foo(); } finally { continue; } } while (a) }",
            None,
        )
            .into(),
        ("while (a) { try { if (foo) {} } catch { continue; } break; }", None).into(),
        ("while (a) { try { switch (foo) {} } catch { continue; } break; }", None).into(),
        ("while (a) { try { while (foo) {} } catch { continue; } break; }", None).into(),
        ("while (a) { try { for (;;foo()) {} } catch { continue; } break; }", None).into(),
        ("function foo() { return; while (a); }", None).into(),
        ("function foo() { return; while (a) break; }", None).into(),
        ("while(true); while(true);", None).into(),
        ("while(true); while(true) break;", None).into(),
        ("while (1); while (a) break;", None).into(),
        ("while (\"x\"); while (a) break;", None).into(),
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
            "function foo() { while (bar) { switch (baz) { case 1: break; default: return; } return; } }",
            None,
        )
            .into(),
        ("function foo() { while (a) { try { return; } finally { foo(); } } }", None).into(),
        ("function foo() { while (a) { try { return; } finally { continue; } } }", None).into(),
        (
            "function foo() { while (a) { try { return; } catch (e) { foo(); } finally { foo(); } } }",
            None,
        )
            .into(),
        (
            "function foo() { do { try { break; } finally { continue; } } while (a) }",
            None,
        )
            .into(),
        ("while (false) break;", None).into(),
        ("for (;false;) break;", None).into(),
        ("do break; while (false);", None).into(),
        ("while (a) { try { while (true) {} } catch { continue; } break; }", None).into(),
        ("while ({}); while (a) break;", None).into(),
        ("while ([]); while (a) break;", None).into(),
        (
            "while (a) break; do break; while (b); for (;;) break; for (c in d) break; for (e of f) break;",
            Some(serde_json::json!([{ "ignore": [] }])),
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
