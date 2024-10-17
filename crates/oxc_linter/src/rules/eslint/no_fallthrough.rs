use std::ops::Range;

use cow_utils::CowUtils;
use itertools::Itertools;
use oxc_ast::{
    ast::{Statement, SwitchCase, SwitchStatement},
    AstKind,
};
use oxc_cfg::{
    graph::{
        visit::{neighbors_filtered_by_edge_weight, EdgeRef},
        Direction,
    },
    BlockNodeId, EdgeType, ErrorEdgeKind, InstructionKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_fallthrough_case_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a 'break' statement before 'case'.").with_label(span)
}

fn no_fallthrough_default_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a 'break' statement before 'default'.").with_label(span)
}

fn no_unused_fallthrough_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Found a comment that would permit fallthrough, but case cannot fall through.",
    )
    .with_label(span)
}

#[derive(Debug, Clone)]
struct Config {
    /// The custom comment pattern to match against. If set to None, the rule
    /// will use the default pattern. Otherwise, if this is Some, the rule will
    /// use the provided pattern.
    comment_pattern: Option<Regex>,
    allow_empty_case: bool,
    report_unused_fallthrough_comment: bool,
}

#[derive(Debug, Clone)]
pub struct NoFallthrough(Box<Config>);

impl NoFallthrough {
    fn new(
        comment_pattern: Option<&str>,
        allow_empty_case: Option<bool>,
        report_unused_fallthrough_comment: Option<bool>,
    ) -> Self {
        Self(Box::new(Config {
            comment_pattern: comment_pattern
                .map(|pattern| Regex::new(format!("(?iu){pattern}").as_str()).unwrap()),
            allow_empty_case: allow_empty_case.unwrap_or(false),
            report_unused_fallthrough_comment: report_unused_fallthrough_comment.unwrap_or(false),
        }))
    }
}

impl Default for NoFallthrough {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow fallthrough of `case` statements
    ///
    /// This rule is aimed at eliminating unintentional fallthrough of one case
    /// to the other. As such, it flags any fallthrough scenarios that are not
    /// marked by a comment.
    ///
    /// ### Why is this bad?
    ///
    /// The switch statement in JavaScript is one of the more error-prone
    /// constructs of the language thanks in part to the ability to “fall
    /// through” from one case to the next. For example:
    ///
    /// ```js
    /// switch(foo) {
    ///     case 1:
    ///     doSomething();
    ///
    /// case 2:
    ///     doSomethingElse();
    /// }
    /// ```
    ///
    /// In this example, if `foo` is `1`, then execution will flow through both
    /// cases, as the first falls through to the second. You can prevent this by
    /// using `break`, as in this example:
    ///
    /// ```js
    /// switch(foo) {
    ///     case 1:
    ///         doSomething();
    ///         break;
    ///
    ///     case 2:
    ///         doSomethingElse();
    /// }
    /// ```
    ///
    /// That works fine when you don’t want a fallthrough, but what if the
    /// fallthrough is intentional, there is no way to indicate that in the
    /// language. It’s considered a best practice to always indicate when a
    /// fallthrough is intentional using a comment which matches the
    /// `/falls?\s?through/i`` regular expression but isn’t a directive:
    ///
    /// ```js
    /// switch(foo) {
    ///     case 1:
    ///         doSomething();
    ///         // falls through
    ///
    ///     case 2:
    ///         doSomethingElse();
    /// }
    ///
    /// switch(foo) {
    ///     case 1:
    ///         doSomething();
    ///         // fall through
    ///
    ///     case 2:
    ///         doSomethingElse();
    /// }
    ///
    /// switch(foo) {
    ///     case 1:
    ///         doSomething();
    ///         // fallsthrough
    ///
    ///     case 2:
    ///         doSomethingElse();
    /// }
    ///
    /// switch(foo) {
    ///     case 1: {
    ///         doSomething();
    ///         // falls through
    ///     }
    ///
    ///     case 2: {
    ///         doSomethingElse();
    ///     }
    /// }
    /// ```
    ///
    /// In this example, there is no confusion as to the expected behavior. It
    /// is clear that the first case is meant to fall through to the second
    /// case.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*oxlint no-fallthrough: "error"*/
    ///
    /// switch(foo) {
    ///     case 1:
    ///         doSomething();
    ///
    ///     case 2:
    ///         doSomething();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /*oxlint no-fallthrough: "error"*/
    ///
    /// switch(foo) {
    ///     case 1:
    ///         doSomething();
    ///         break;
    ///
    ///     case 2:
    ///         doSomething();
    /// }
    ///
    /// function bar(foo) {
    ///     switch(foo) {
    ///         case 1:
    ///             doSomething();
    ///             return;
    ///
    ///         case 2:
    ///             doSomething();
    ///     }
    /// }
    ///
    /// switch(foo) {
    ///     case 1:
    ///         doSomething();
    ///         throw new Error("Boo!");
    ///
    ///     case 2:
    ///         doSomething();
    /// }
    ///
    /// switch(foo) {
    ///     case 1:
    ///     case 2:
    ///         doSomething();
    /// }
    ///
    /// switch(foo) {
    ///     case 1: case 2:
    ///         doSomething();
    /// }
    ///
    /// switch(foo) {
    ///     case 1:
    ///         doSomething();
    ///         // falls through
    ///
    ///     case 2:
    ///         doSomething();
    /// }
    ///
    /// switch(foo) {
    ///     case 1: {
    ///         doSomething();
    ///         // falls through
    ///     }
    ///
    ///     case 2: {
    ///         doSomethingElse();
    ///     }
    /// }
    /// ```
    ///
    /// Note that the last case statement in these examples does not cause a
    /// warning because there is nothing to fall through into.
    NoFallthrough,
    // TODO: add options section to docs
    pedantic, // Fall through code are still incorrect.
    pending // TODO: add a dangerous suggestion for this rule.
);

impl Rule for NoFallthrough {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(value) = value.get(0) else { return Self::default() };
        let comment_pattern = value.get("commentPattern").and_then(serde_json::Value::as_str);
        let allow_empty_case = value.get("allowEmptyCase").and_then(serde_json::Value::as_bool);
        let report_unused_fallthrough_comment =
            value.get("reportUnusedFallthroughComment").and_then(serde_json::Value::as_bool);

        Self::new(comment_pattern, allow_empty_case, report_unused_fallthrough_comment)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch) = node.kind() else { return };

        let cfg = ctx.cfg();
        let switch_id = node.cfg_id();
        let graph = cfg.graph();

        let (cfg_ids, tests, default, exit) = get_switch_semantic_cases(ctx, node, switch);

        let Some(default_or_exit) = default.or(exit) else {
            // TODO: our `get_switch_semantic_cases` can't evaluate cfg_ids for switch statements
            // with conditional discriminant. If we can access the IDs correctly it should never be `None`.
            return;
        };

        let fallthroughs: FxHashSet<BlockNodeId> = neighbors_filtered_by_edge_weight(
            graph,
            switch_id,
            &|e| match e {
                EdgeType::Normal | EdgeType::Jump | EdgeType::Error(ErrorEdgeKind::Explicit) => {
                    None
                }
                _ => Some(None),
            },
            &mut |node, last_cond: Option<BlockNodeId>| {
                let node = *node;

                if node == switch_id {
                    return (last_cond, true);
                }
                if node == default_or_exit {
                    return (last_cond, false);
                }
                if tests.contains_key(&node) {
                    return (last_cond, true);
                }
                if cfg.basic_block(node).is_unreachable() {
                    return (None, false);
                }

                let fallthrough = graph
                    .edges_directed(node, Direction::Outgoing)
                    .find(|it| {
                        let target = it.target();
                        if let Some(default) = default {
                            if default == target {
                                return true;
                            }
                        }
                        tests.contains_key(&target)
                    })
                    .map(|e| e.target());

                (fallthrough, fallthrough.is_none())
            },
        )
        .into_iter()
        .flatten()
        .collect();

        let mut iter = switch.cases.iter().zip(cfg_ids).peekable();
        while let Some((case, _)) = iter.next() {
            let Some((next_case, next_cfg_id)) = iter.peek() else { continue };
            if !fallthroughs.contains(next_cfg_id) {
                if self.0.report_unused_fallthrough_comment {
                    if let Some(span) = self.maybe_allow_fallthrough_trivia(ctx, case, next_case) {
                        ctx.diagnostic(no_unused_fallthrough_diagnostic(span));
                    }
                }
                continue;
            }
            let is_illegal_fallthrough = {
                let is_fallthrough = !case.consequent.is_empty()
                    || (!self.0.allow_empty_case
                        && Self::has_blanks_between(ctx, case.span.start..next_case.span.start));
                is_fallthrough
                    && self.maybe_allow_fallthrough_trivia(ctx, case, next_case).is_none()
            };

            if is_illegal_fallthrough {
                let span = next_case.span;
                if next_case.is_default_case() {
                    ctx.diagnostic(no_fallthrough_default_diagnostic(span));
                } else {
                    ctx.diagnostic(no_fallthrough_case_diagnostic(span));
                }
            }
        }
    }
}

fn possible_fallthrough_comment_span(case: &SwitchCase) -> (u32, Option<u32>) {
    if let Ok(Statement::BlockStatement(block)) = case.consequent.iter().exactly_one() {
        let span = block.span;
        if let Some(last) = block.body.last() {
            (last.span().end, Some(span.end))
        } else {
            (span.start, Some(span.end))
        }
    } else if let Some(last) = case.consequent.last() {
        (last.span().end, None)
    } else {
        (case.span.end, None)
    }
}

impl NoFallthrough {
    fn has_blanks_between(ctx: &LintContext, range: Range<u32>) -> bool {
        let in_between = &ctx.semantic().source_text()[range.start as usize..range.end as usize];
        // check for at least 2 new lines, we allow the first new line for formatting.
        in_between.bytes().filter(|it| *it == b'\n').nth(1).is_some()
    }

    fn maybe_allow_fallthrough_trivia(
        &self,
        ctx: &LintContext,
        case: &SwitchCase,
        fall: &SwitchCase,
    ) -> Option<Span> {
        let semantic = ctx.semantic();
        let is_fallthrough_comment_in_range = |range: Range<u32>| {
            let comment = semantic
                .comments_range(range)
                .map(|comment| {
                    &semantic.source_text()[comment.span.start as usize..comment.span.end as usize]
                })
                .last()
                .map(str::trim);

            comment.is_some_and(|comment| self.is_comment_fall_through(comment))
        };

        let (start, end) = possible_fallthrough_comment_span(case);

        if let Some(end) = end {
            let range = start..end;
            if is_fallthrough_comment_in_range(range.clone()) {
                return Some(Span::new(start, end));
            }
        }

        let range = start..fall.span.start;
        if is_fallthrough_comment_in_range(range.clone()) {
            Some(Span::new(start, fall.span.start))
        } else {
            None
        }
    }

    fn is_comment_fall_through(&self, comment: &str) -> bool {
        if comment.starts_with("oxlint-") || comment.starts_with("eslint-") {
            return false;
        }
        if let Some(custom_pattern) = &self.0.comment_pattern {
            custom_pattern.is_match(comment)
        } else {
            // We are doing a quick check here to see if it starts with the expected "falls" comment,
            // so that we don't need to initialize the pattern matcher if we don't need it.
            let comment = comment.trim().cow_to_ascii_lowercase();
            comment == "falls through"
                || comment == "fall through"
                || comment == "fallsthrough"
                || comment == "fallthrough"
        }
    }
}

/// Get semantic information about a switch cases and its exit point.
// ----------------------------------------!README!-----------------------------------------------
// >> PLEASE DON'T MAKE IT A REPEATING PATTERN IN THE PROJECT, ONE TIME HACK TO GET IT DONE
//  >>  TODO: it is a hack to get our cases `cfg_id`s. please replace me with semantic API when
//          one became available. This code is highly volitile and has a lot of assumptions about
//          the current shape of the CFG, It is just a slow and dirty workaround!
// ----------------------------------------------------------------------------------------------
// TREAT LIKE BLACK MAGIC, IT BREAKS WITH SMALLEST CHANGES TO THE SWITCH CASE CFG!
// NOTE: DO NOT COPY -- DO NOT REUSE -- DO NOT EXTEND
// NOTE: DO NOT COPY -- DO NOT REUSE -- DO NOT EXTEND
// NOTE: DO NOT COPY -- DO NOT REUSE -- DO NOT EXTEND
// NOTE: DO NOT COPY -- DO NOT REUSE -- DO NOT EXTEND
// NOTE: DO NOT COPY -- DO NOT REUSE -- DO NOT EXTEND
// IF U NEED THIS AS AN API COMMENT ON THE ISSUE OR CREATE A DUP IF IT IS CLOSED!
// TAKE IT AS A MAGICAL BLACK BOX, NO DOCUMENTATION TO PREVENT REUSE!
// Issue: <https://github.com/oxc-project/oxc/issues/3662>
fn get_switch_semantic_cases(
    ctx: &LintContext,
    node: &AstNode,
    switch: &SwitchStatement,
) -> (
    Vec<BlockNodeId>,
    FxHashMap<BlockNodeId, /* is_empty */ bool>,
    /* default */ Option<BlockNodeId>,
    /* exit */ Option<BlockNodeId>,
) {
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let has_default = switch.cases.iter().any(SwitchCase::is_default_case);
    let (tests, exit) = graph
        .edges_directed(node.cfg_id(), Direction::Outgoing)
        .fold((Vec::new(), None), |(mut conds, exit), it| {
            let target = it.target();
            if !matches!(it.weight(), EdgeType::Normal) {
                (conds, exit)
            } else if cfg
                .basic_block(target)
                .instructions()
                .iter()
                .any(|it| matches!(it.kind, InstructionKind::Condition))
            {
                let is_empty = graph
                    .edges_directed(target, Direction::Outgoing)
                    .filter(|it| matches!(it.weight(), EdgeType::Jump))
                    .exactly_one()
                    .ok()
                    .and_then(|it| {
                        cfg.basic_block(it.target())
                            .instructions()
                            .first()
                            .and_then(|it| it.node_id)
                            .map(|id| ctx.nodes().parent_kind(id))
                            .and_then(|it| match it {
                                Some(AstKind::SwitchCase(case)) => Some(case),
                                _ => None,
                            })
                    })
                    .is_some_and(|it| it.consequent.is_empty() || it.consequent.iter().exactly_one().is_ok_and(|it| matches!(it, Statement::BlockStatement(b) if b.body.is_empty())));
                conds.push((target, is_empty));
                (conds, exit)
            } else {
                (conds, Some(target))
            }
        });

    let mut cfg_ids: Vec<_> = tests.iter().rev().map(|it| it.0).collect();
    let (default, exit) = if has_default {
        if let Some(exit) = exit {
            cfg_ids.push(exit);
        }
        (exit, None)
    } else {
        (None, exit)
    };
    (cfg_ids, FxHashMap::from_iter(tests), default, exit)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("switch(foo) { case 0: a(); /* falls through */ case 1: b(); }", None),
        ("switch(foo) { case 0: a()\n /* falls through */ case 1: b(); }", None),
        ("switch(foo) { case 0: a(); /* fall through */ case 1: b(); }", None),
        ("switch(foo) { case 0: a(); /* fallthrough */ case 1: b(); }", None),
        ("switch(foo) { case 0: a(); /* FALLS THROUGH */ case 1: b(); }", None),
        ("switch(foo) { case 0: { a(); /* falls through */ } case 1: b(); }", None),
        ("switch(foo) { case 0: { a()\n /* falls through */ } case 1: b(); }", None),
        ("switch(foo) { case 0: { a(); /* fall through */ } case 1: b(); }", None),
        ("switch(foo) { case 0: { a(); /* fallthrough */ } case 1: b(); }", None),
        ("switch(foo) { case 0: { a(); /* FALLS THROUGH */ } case 1: b(); }", None),
        ("switch(foo) { case 0: { a(); } /* falls through */ case 1: b(); }", None),
        ("switch(foo) { case 0: { a(); /* falls through */ } /* comment */ case 1: b(); }", None),
        ("switch(foo) { case 0: { /* falls through */ } case 1: b(); }", None),
        ("function foo() { switch(foo) { case 0: a(); return; case 1: b(); }; }", None),
        ("switch(foo) { case 0: a(); throw 'foo'; case 1: b(); }", None),
        ("while (a) { switch(foo) { case 0: a(); continue; case 1: b(); } }", None),
        ("switch(foo) { case 0: a(); break; case 1: b(); }", None),
        ("switch(foo) { case 0: case 1: a(); break; case 2: b(); }", None),
        ("switch(foo) { case 0: case 1: break; case 2: b(); }", None),
        ("switch(foo) { case 0: case 1: break; default: b(); }", None),
        ("switch(foo) { case 0: case 1: a(); }", None),
        ("switch(foo) { case 0: case 1: a(); break; }", None),
        ("switch(foo) { case 0: case 1: break; }", None),
        ("switch(foo) { case 0:\n case 1: break; }", None),
        ("switch(foo) { case 0: // comment\n case 1: break; }", None),
        ("function foo() { switch(foo) { case 0: case 1: return; } }", None),
        ("function foo() { switch(foo) { case 0: {return;}\n case 1: {return;} } }", None),
        ("switch(foo) { case 0: case 1: {break;} }", None),
        ("switch(foo) { }", None),
        (
            "switch(foo) { case 0: switch(bar) { case 2: break; } /* falls through */ case 1: break; }",
            None,
        ),
        ("function foo() { switch(foo) { case 1: return a; a++; }}", None),
        ("switch (foo) { case 0: a(); /* falls through */ default:  b(); /* comment */ }", None),
        ("switch (foo) { case 0: a(); /* falls through */ default: /* comment */ b(); }", None),
        ("switch (foo) { case 0: if (a) { break; } else { throw 0; } default: b(); }", None),
        ("switch (foo) { case 0: try { break; } finally {} default: b(); }", None),
        ("switch (foo) { case 0: try {} finally { break; } default: b(); }", None),
        ("switch (foo) { case 0: try { throw 0; } catch (err) { break; } default: b(); }", None),
        ("switch (foo) { case 0: do { throw 0; } while(a); default: b(); }", None),
        (
            "switch (foo) { case 0: a(); \n// eslint-disable-next-line no-fallthrough\n case 1: }",
            None,
        ),
        (
            "switch(foo) { case 0: a(); /* no break */ case 1: b(); }",
            Some(serde_json::json!([{
                "commentPattern": "no break"
            }])),
        ),
        (
            "switch(foo) { case 0: a(); /* no break: need to execute b() */ case 1: b(); }",
            Some(serde_json::json!([{
                "commentPattern": "no break:\\s?\\w+"
            }])),
        ),
        (
            "switch(foo) { case 0: a();\n// need to execute b(), so\n// falling through\n case 1: b(); }",
            Some(serde_json::json!([{
                "commentPattern": "falling through"
            }])),
        ),
        (
            "switch(foo) { case 0: a(); /* break omitted */ default:  b(); /* comment */ }",
            Some(serde_json::json!([{
                "commentPattern": "break omitted"
            }])),
        ),
        (
            "switch(foo) { case 0: a(); /* caution: break is omitted intentionally */ case 1: b(); /* break omitted */ default: c(); }",
            Some(serde_json::json!([{
                "commentPattern": "break[\\s\\w]+omitted"
            }])),
        ),
        (
            "switch(foo) { case 0: \n\n\n case 1: b(); }",
            Some(serde_json::json!([{ "allowEmptyCase": true }])),
        ),
        (
            "switch(foo) { case 0: \n /* with comments */  \n case 1: b(); }",
            Some(serde_json::json!([{ "allowEmptyCase": true }])),
        ),
        (
            "switch (a) {\n case 1: ; break; \n case 3: }",
            Some(serde_json::json!([{ "allowEmptyCase": true }])),
        ),
        (
            "switch (a) {\n case 1: ; break; \n case 3: }",
            Some(serde_json::json!([{ "allowEmptyCase": false }])),
        ),
        (
            "switch(foo) { case 0: a(); break; /* falls through */ case 1: b(); }",
            Some(serde_json::json!([{
                "reportUnusedFallthroughComment": false
            }])),
        ),
    ];

    let fail = vec![
        ("switch(foo) { case 0: a();\ncase 1: b() }", None),
        ("switch(foo) { case 0: a();\ndefault: b() }", None),
        ("switch(foo) { case 0: a(); default: b() }", None),
        ("switch(foo) { case 0: if (a) { break; } default: b() }", None),
        ("switch(foo) { case 0: try { throw 0; } catch (err) {} default: b() }", None),
        ("switch(foo) { case 0: while (a) { break; } default: b() }", None),
        ("switch(foo) { case 0: do { break; } while (a); default: b() }", None),
        ("switch(foo) { case 0:\n\n default: b() }", None),
        ("switch(foo) { case 0: {} default: b() }", None),
        ("switch(foo) { case 0: a(); { /* falls through */ } default: b() }", None),
        ("switch(foo) { case 0: { /* falls through */ } a(); default: b() }", None),
        ("switch(foo) { case 0: if (a) { /* falls through */ } default: b() }", None),
        ("switch(foo) { case 0: { { /* falls through */ } } default: b() }", None),
        ("switch(foo) { case 0: { /* comment */ } default: b() }", None),
        ("switch(foo) { case 0:\n // comment\n default: b() }", None),
        ("switch(foo) { case 0: a(); /* falling through */ default: b() }", None),
        (
            "switch(foo) { case 0: a();\n/* no break */\ncase 1: b(); }",
            Some(serde_json::json!([{
                "commentPattern": "break omitted"
            }])),
        ),
        (
            "switch(foo) { case 0: a();\n/* no break */\n/* todo: fix readability */\ndefault: b() }",
            Some(serde_json::json!([{
                "commentPattern": "no break"
            }])),
        ),
        (
            "switch(foo) { case 0: { a();\n/* no break */\n/* todo: fix readability */ }\ndefault: b() }",
            Some(serde_json::json!([{
                "commentPattern": "no break"
            }])),
        ),
        ("switch(foo) { case 0: \n /* with comments */  \ncase 1: b(); }", None),
        (
            "switch(foo) { case 0:\n\ncase 1: b(); }",
            Some(serde_json::json!([{
                "allowEmptyCase": false
            }])),
        ),
        ("switch(foo) { case 0:\n\ncase 1: b(); }", Some(serde_json::json!([{}]))),
        (
            "switch (a) { case 1: \n ; case 2:  }",
            Some(serde_json::json!([{ "allowEmptyCase": false }])),
        ),
        (
            "switch (a) { case 1: ; case 2: ; case 3: }",
            Some(serde_json::json!([{ "allowEmptyCase": true }])),
        ),
        (
            "switch (foo) { case 0: a(); \n// eslint-enable no-fallthrough\n case 1: }",
            Some(serde_json::json!([{}])),
        ),
        (
            "switch(foo) { case 0: a(); break; /* falls through */ case 1: b(); }",
            Some(serde_json::json!([{
                "reportUnusedFallthroughComment": true
            }])),
        ),
        // TODO: it should fail but doesn't, we ignore conditional discriminants for now.
        // ("switch (a === b ? c : d) { case 1: ; case 2: ; case 3: ; }", None)
    ];

    Tester::new(NoFallthrough::NAME, pass, fail).test_and_snapshot();
}
