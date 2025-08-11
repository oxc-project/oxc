use std::ops::Deref;

use itertools::Itertools;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        JestGeneralFnKind, ParsedGeneralJestFnCall, collect_possible_jest_call_node,
        parse_general_jest_fn_call,
    },
};

fn padding_around_test_blocks_diagnostic(span: Span, name: &CompactStr) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Missing padding before {name} block"))
        .with_help(format!("Make sure there is an empty new line before the {name} block"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundTestBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a line of padding before and after 1 or more test/it statements
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const thing = 123;
    /// test('foo', () => {});
    /// test('bar', () => {});
    /// ```
    ///
    /// ```js
    /// const thing = 123;
    /// it('foo', () => {});
    /// it('bar', () => {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const thing = 123;
    ///
    /// test('foo', () => {});
    ///
    /// test('bar', () => {});
    /// ```
    ///
    /// ```js
    /// const thing = 123;
    ///
    /// it('foo', () => {});
    ///
    /// it('bar', () => {});
    /// ```
    PaddingAroundTestBlocks,
    jest,
    style,
    fix
);

enum NodeType {
    Test(CompactStr),
    Statement,
}

impl Rule for PaddingAroundTestBlocks {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut nodes = collect_possible_jest_call_node(ctx)
            .iter()
            .filter_map(|possible_jest_node| {
                let node = possible_jest_node.node;
                let AstKind::CallExpression(call_expr) = node.kind() else {
                    return None;
                };
                let jest_fn_call = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx)?;
                let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
                let kind = kind.to_general()?;
                if kind != JestGeneralFnKind::Test {
                    return None;
                }
                Some((NodeType::Test(name.deref().into()), node))
            })
            .collect_vec();
        for node in ctx.nodes() {
            if node.kind().is_statement() {
                nodes.push((NodeType::Statement, node));
            }
        }
        nodes.sort_by_key(|(_, node)| node.span().end);
        let mut prev_node: Option<&AstNode<'_>> = None;
        for (node_type, node) in nodes {
            match node_type {
                NodeType::Test(name) => {
                    if let Some(prev_node) = prev_node {
                        if prev_node.span().end > node.span().start {
                            continue;
                        }
                        let mut comments_range =
                            ctx.comments_range(prev_node.span().end..node.span().start);
                        let mut span_between_start = prev_node.span().end;
                        let mut span_between_end = node.span().start;
                        if let Some(last_comment_span) =
                            comments_range.next_back().map(|comment| comment.span)
                        {
                            let space_after_last_comment = ctx
                                .source_range(Span::new(last_comment_span.end, node.span().start));
                            let space_before_last_comment = ctx.source_range(Span::new(
                                prev_node.span().end,
                                last_comment_span.start,
                            ));
                            if space_after_last_comment.matches('\n').count() > 1
                                || space_before_last_comment.matches('\n').count() == 0
                            {
                                span_between_start = last_comment_span.end;
                            } else {
                                span_between_end = last_comment_span.start;
                            }
                        }
                        let span_between = Span::new(span_between_start, span_between_end);
                        let content = ctx.source_range(span_between);
                        if content.matches('\n').count() < 2 {
                            ctx.diagnostic_with_fix(
                                padding_around_test_blocks_diagnostic(
                                    Span::new(span_between_end, span_between_end),
                                    &name,
                                ),
                                |fixer| {
                                    let spaces_after_last_line = content
                                        .rfind('\n')
                                        .map_or("", |index| content.split_at(index + 1).1);
                                    fixer.replace(
                                        span_between,
                                        format!("\n\n{spaces_after_last_line}"),
                                    )
                                },
                            );
                        }
                    }
                    prev_node = Some(node);
                }
                NodeType::Statement => {
                    prev_node = Some(node);
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "test('foo', () => {});",
        "test('foo', () => {});\n\ntest('bar', () => {});",
        "const thing = 123;\n\ntest('foo', () => {});",
        "{ test('foo', () => {}); }",
        "describe('foo', () => {\ntest('bar', () => {});\n});",
    ];

    let fail = vec![
        "test('foo', () => {});test('bar', () => {});",
        "test('foo', () => {});\ntest('bar', () => {});",
        "it('foo', () => {});\nfit('bar', () => {});\ntest('baz', () => {});",
        r"
const foo = 'bar';
const bar = 'baz';
it('foo', () => {
  // stuff
});
fit('bar', () => {
  // stuff
});
test('foo foo', () => {});
test('bar bar', () => {});

// Nesting
 describe('other bar', () => {
     const thing = 123;
     test('is another bar w/ test', () => {
     });
     // With a comment
     it('is another bar w/ it', () => {
     });
     test.skip('skipping', () => {}); // Another comment
     it.skip('skipping too', () => {});
 });xtest('weird', () => {});
 test
   .skip('skippy skip', () => {});
 xit('bar foo', () => {});
            ",
    ];

    let fix = vec![
        (
            "test('foo', () => {});test('bar', () => {});",
            "test('foo', () => {});\n\ntest('bar', () => {});",
        ),
        (
            "test('foo', () => {});\ntest('bar', () => {});",
            "test('foo', () => {});\n\ntest('bar', () => {});",
        ),
        (
            "it('foo', () => {});\nfit('bar', () => {});\ntest('baz', () => {});",
            "it('foo', () => {});\n\nfit('bar', () => {});\n\ntest('baz', () => {});",
        ),
        (
            r"
const foo = 'bar';
const bar = 'baz';
it('foo', () => {
  // stuff
});
fit('bar', () => {
  // stuff
});
test('foo foo', () => {});
test('bar bar', () => {});

// Nesting
describe('other bar', () => {
    const thing = 123;
    test('is another bar w/ test', () => {
    });
    // With a comment
    it('is another bar w/ it', () => {
    });
    test.skip('skipping', () => {}); // Another comment
    it.skip('skipping too', () => {});
});xtest('weird', () => {});
test
  .skip('skippy skip', () => {});
xit('bar foo', () => {});
        ",
            r"
const foo = 'bar';
const bar = 'baz';

it('foo', () => {
  // stuff
});

fit('bar', () => {
  // stuff
});

test('foo foo', () => {});

test('bar bar', () => {});

// Nesting
describe('other bar', () => {
    const thing = 123;

    test('is another bar w/ test', () => {
    });

    // With a comment
    it('is another bar w/ it', () => {
    });

    test.skip('skipping', () => {}); // Another comment

    it.skip('skipping too', () => {});
});

xtest('weird', () => {});

test
  .skip('skippy skip', () => {});

xit('bar foo', () => {});
        ",
        ),
    ];
    Tester::new(PaddingAroundTestBlocks::NAME, PaddingAroundTestBlocks::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
