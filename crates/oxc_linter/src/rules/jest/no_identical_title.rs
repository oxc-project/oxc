use std::collections::HashMap;

use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodeId;
use oxc_span::{Atom, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, parse_general_jest_fn_call_new, JestFnKind,
        JestGeneralFnKind, PossibleJestNode,
    },
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum NoIdenticalTitleDiagnostic {
    #[error("eslint-plugin-jest(no-identical-title): Describe block title is used multiple times in the same describe block.")]
    #[diagnostic(severity(warning), help("Change the title of describe block."))]
    DescribeRepeat(#[label] Span),

    #[error("eslint-plugin-jest(no-identical-title): Test title is used multiple times in the same describe block.")]
    #[diagnostic(severity(warning), help("Change the title of test."))]
    TestRepeat(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct NoIdenticalTitle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule looks at the title of every test and test suite.
    /// It will report when two test suites or two test cases at the same level of a test suite have the same title.
    ///
    /// ### Why is this bad?
    ///
    /// Having identical titles for two different tests or test suites may create confusion.
    /// For example, when a test with the same title as another test in the same test suite fails, it is harder to know which one failed and thus harder to fix.
    ///
    /// ### Example
    /// ```javascript
    ///  describe('baz', () => {
    ///    //...
    ///  });
    ///
    ///  describe('baz', () => {
    ///    // Has the same title as a previous test suite
    ///    // ...
    ///  });
    /// ```
    NoIdenticalTitle,
    style
);

impl Rule for NoIdenticalTitle {
    fn run_once(&self, ctx: &LintContext) {
        let possible_jest_nodes = collect_possible_jest_call_node(ctx);
        let mut title_to_span_mapping = HashMap::new();
        let mut span_to_parent_mapping = HashMap::new();

        possible_jest_nodes
            .iter()
            .filter_map(|possible_jest_node| {
                let AstKind::CallExpression(call_expr) = possible_jest_node.node.kind() else {
                    return None;
                };
                filter_and_process_jest_result(call_expr, possible_jest_node, ctx)
            })
            .for_each(|(span, title, kind, parent_id)| {
                span_to_parent_mapping.insert(span, parent_id);
                title_to_span_mapping
                    .entry(title)
                    .and_modify(|e: &mut Vec<(JestFnKind, Span)>| e.push((kind, span)))
                    .or_insert_with(|| vec![(kind, span)]);
            });

        for kind_and_span in title_to_span_mapping.values() {
            let mut kind_and_spans = kind_and_span
                .iter()
                .filter_map(|(kind, span)| {
                    let parent = span_to_parent_mapping.get(span)?;
                    Some((*span, *kind, *parent))
                })
                .collect::<Vec<(Span, JestFnKind, AstNodeId)>>();
            // After being sorted by parent_id, the span with the same parent will be placed nearby.
            kind_and_spans.sort_by(|a, b| a.2.cmp(&b.2));

            // Skip the first element, for `describe('foo'); describe('foo');`, we only need to check the second one.
            for i in 1..kind_and_spans.len() {
                let (span, kind, parent_id) = kind_and_spans[i];
                let (_, prev_kind, prev_parent) = kind_and_spans[i - 1];

                if kind == prev_kind && parent_id == prev_parent {
                    match kind {
                        JestFnKind::General(JestGeneralFnKind::Describe) => {
                            ctx.diagnostic(NoIdenticalTitleDiagnostic::DescribeRepeat(span));
                        }
                        JestFnKind::General(JestGeneralFnKind::Test) => {
                            ctx.diagnostic(NoIdenticalTitleDiagnostic::TestRepeat(span));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn filter_and_process_jest_result<'a>(
    call_expr: &'a CallExpression<'a>,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) -> Option<(Span, &'a Atom, JestFnKind, AstNodeId)> {
    let Some(result) = parse_general_jest_fn_call_new(call_expr, possible_jest_node, ctx) else {
        return None;
    };
    let kind = result.kind;
    // we only need check `describe` or `test` block
    if !matches!(kind, JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test)) {
        return None;
    }

    if result.members.iter().any(|m| m.is_name_equal("each")) {
        return None;
    }

    let Some(parent_id) = get_closest_block(possible_jest_node.node, ctx) else {
        return None;
    };

    match call_expr.arguments.get(0) {
        Some(Argument::Expression(Expression::StringLiteral(string_lit))) => {
            Some((string_lit.span, &string_lit.value, kind, parent_id))
        }
        Some(Argument::Expression(Expression::TemplateLiteral(template_lit))) => {
            template_lit.quasi().map(|quasi| (template_lit.span, quasi, kind, parent_id))
        }
        _ => None,
    }
}

fn get_closest_block(node: &AstNode, ctx: &LintContext) -> Option<AstNodeId> {
    match node.kind() {
        AstKind::BlockStatement(_) | AstKind::FunctionBody(_) | AstKind::Program(_) => {
            Some(node.id())
        }
        _ => {
            let parent = ctx.nodes().parent_node(node.id())?;
            get_closest_block(parent, ctx)
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("it(); it();", None),
        ("describe(); describe();", None),
        ("describe('foo', () => {}); it('foo', () => {});", None),
        (
            "
              describe('foo', () => {
                it('works', () => {});
              });
            ",
            None,
        ),
        (
            "
              it('one', () => {});
              it('two', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {});
              describe('foe', () => {});
            ",
            None,
        ),
        (
            "
              it(`one`, () => {});
              it(`two`, () => {});
            ",
            None,
        ),
        (
            "
              describe(`foo`, () => {});
              describe(`foe`, () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                test('this', () => {});
                test('that', () => {});
              });
            ",
            None,
        ),
        (
            "
              test.concurrent('this', () => {});
              test.concurrent('that', () => {});
            ",
            None,
        ),
        (
            "
              test.concurrent('this', () => {});
              test.only.concurrent('that', () => {});
            ",
            None,
        ),
        (
            "
              test.only.concurrent('this', () => {});
              test.concurrent('that', () => {});
            ",
            None,
        ),
        (
            "
              test.only.concurrent('this', () => {});
              test.only.concurrent('that', () => {});
            ",
            None,
        ),
        (
            "
              test.only('this', () => {});
              test.only('that', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                it('works', () => {});

                describe('foe', () => {
                  it('works', () => {});
                });
              });
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                describe('foe', () => {
                  it('works', () => {});
                });

                it('works', () => {});
              });
            ",
            None,
        ),
        ("describe('foo', () => describe('foe', () => {}));", None),
        (
            "
              describe('foo', () => {
                describe('foe', () => {});
              });

              describe('foe', () => {});
            ",
            None,
        ),
        ("test('number' + n, function() {});", None),
        ("test('number' + n, function() {}); test('number' + n, function() {});", None),
        // ("it(`${n}`, function() {});", None),
        // ("it(`${n}`, function() {}); it(`${n}`, function() {});", None),
        (
            "
              describe('a class named ' + myClass.name, () => {
                describe('#myMethod', () => {});
              });

              describe('something else', () => {});
            ",
            None,
        ),
        (
            "
              describe('my class', () => {
                describe('#myMethod', () => {});
                describe('a class named ' + myClass.name, () => {});
              });
            ",
            None,
        ),
        (
            "
              const test = { content: () => 'foo' };
              test.content(`something that is not from jest`, () => {});
              test.content(`something that is not from jest`, () => {});
            ",
            None,
        ),
        (
            "
              const describe = { content: () => 'foo' };
              describe.content(`something that is not from jest`, () => {});
              describe.content(`something that is not from jest`, () => {});
            ",
            None,
        ),
        (
            "
              describe.each`
                description
                ${'b'}
              `('$description', () => {});

              describe.each`
                description
                ${'a'}
              `('$description', () => {});
            ",
            None,
        ),
        (
            "
              describe('top level', () => {
                describe.each``('nested each', () => {
                  describe.each``('nested nested each', () => {});
                });

                describe('nested', () => {});
              });
            ",
            None,
        ),
        (
            "
              describe.each``('my title', value => {});
              describe.each``('my title', value => {});
              describe.each([])('my title', value => {});
              describe.each([])('my title', value => {});
            ",
            None,
        ),
        (
            "
              describe.each([])('when the value is %s', value => {});
              describe.each([])('when the value is %s', value => {});
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
              describe('foo', () => {
                it('works', () => {});
                it('works', () => {});
              });
            ",
            None,
        ),
        (
            "
              it('works', () => {});
              it('works', () => {});
            ",
            None,
        ),
        (
            "
              test.only('this', () => {});
              test('this', () => {});
            ",
            None,
        ),
        (
            "
              xtest('this', () => {});
              test('this', () => {});
            ",
            None,
        ),
        (
            "
              test.only('this', () => {});
              test.only('this', () => {});
            ",
            None,
        ),
        (
            "
              test.concurrent('this', () => {});
              test.concurrent('this', () => {});
            ",
            None,
        ),
        (
            "
              test.only('this', () => {});
              test.concurrent('this', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {});
              describe('foo', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {});
              xdescribe('foo', () => {});
            ",
            None,
        ),
        (
            "
              fdescribe('foo', () => {});
              describe('foo', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                describe('foe', () => {});
              });
              describe('foo', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                it(`catches backticks with the same title`, () => {});
                it(`catches backticks with the same title`, () => {});
              });
            ",
            None,
        ),
        // (
        //     "
        //       context('foo', () => {
        //         describe('foe', () => {});
        //       });
        //       describe('foo', () => {});
        //     ",
        //     None,
        // ),
    ];

    Tester::new(NoIdenticalTitle::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
