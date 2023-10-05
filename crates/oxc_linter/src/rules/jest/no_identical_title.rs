use std::collections::HashMap;

use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNodeId, ReferenceId};
use oxc_span::{Atom, Span};

use crate::{
    context::LintContext,
    jest_ast_util::{parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind},
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-identical-title): {0:?}")]
#[diagnostic(severity(warning), help("{1:?}"))]
struct NoIdenticalTitleDiagnostic(&'static str, &'static str, #[label] pub Span);

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
    restriction
);

impl Rule for NoIdenticalTitle {
    fn run_once(&self, ctx: &LintContext) {
        // TODO: support detect import from "@jest/globals"
        let references = ctx.scopes().root_unresolved_references().iter().filter(|(key, _)| {
            DESCRIBE_NAMES.contains(&key.as_str()) || TEST_NAMES.contains(&key.as_str())
        });
        let mut title_to_span_mapping = HashMap::new();
        let mut span_to_parent_mapping = HashMap::new();

        for (_, reference_ids) in references {
            for &reference_id in reference_ids {
                let Some((span, title, kind, parent_id)) = process_reference(reference_id, ctx)
                else {
                    continue;
                };

                span_to_parent_mapping.insert(span, parent_id);
                title_to_span_mapping
                    .entry(title)
                    .and_modify(|e: &mut Vec<(JestFnKind, Span)>| e.push((kind, span)))
                    .or_insert_with(|| vec![(kind, span)]);
            }
        }

        for kind_and_span in title_to_span_mapping.values() {
            let mut kind_and_spans = kind_and_span
                .iter()
                .filter_map(|(kind, call_expr)| {
                    let Some(parent) = span_to_parent_mapping.get(call_expr) else { return None };
                    Some((*call_expr, *kind, *parent))
                })
                .collect::<Vec<(Span, JestFnKind, AstNodeId)>>();
            kind_and_spans.sort_by(|a, b| a.2.cmp(&b.2));

            for i in 1..kind_and_spans.len() {
                let (span, kind, parent_id) = kind_and_spans[i];
                let (_, prev_kind, prev_parent) = kind_and_spans[i - 1];

                if kind == prev_kind && parent_id == prev_parent {
                    let (error, help) = Message::details(kind);
                    ctx.diagnostic(NoIdenticalTitleDiagnostic(error, help, span));
                }
            }
        }
    }
}

const DESCRIBE_NAMES: [&str; 3] = ["describe", "fdescribe", "xdescribe"];
const TEST_NAMES: [&str; 5] = ["it", "fit", "xit", "test", "xtest"];

fn process_reference<'a>(
    reference_id: ReferenceId,
    ctx: &LintContext<'a>,
) -> Option<(Span, &'a Atom, JestFnKind, AstNodeId)> {
    let reference = ctx.symbols().get_reference(reference_id);
    let node = ctx.nodes().parent_node(reference.node_id())?;
    let node = get_closest_call_expr(node, ctx)?;
    let closest_block = get_closest_block(node, ctx)?;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return None;
    };
    let jest_fn_call = parse_general_jest_fn_call(call_expr, node, ctx)?;
    match call_expr.arguments.get(0) {
        Some(Argument::Expression(Expression::StringLiteral(string_lit))) => {
            Some((call_expr.span, &string_lit.value, jest_fn_call.kind, closest_block.id()))
        }
        Some(Argument::Expression(Expression::TemplateLiteral(template_lit))) => {
            match template_lit.quasi() {
                Some(quasi) => Some((call_expr.span, quasi, jest_fn_call.kind, closest_block.id())),
                None => None,
            }
        }
        _ => None,
    }
}

fn get_closest_block<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    match node.kind() {
        AstKind::BlockStatement(_) | AstKind::FunctionBody(_) | AstKind::Program(_) => Some(node),
        _ => {
            let parent = ctx.nodes().parent_node(node.id())?;
            get_closest_block(parent, ctx)
        }
    }
}

fn get_closest_call_expr<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    match node.kind() {
        AstKind::CallExpression(_) => Some(node),
        AstKind::MemberExpression(member_expr) => {
            if member_expr.static_property_name() == Some("each") {
                return None;
            }
            let parent = ctx.nodes().parent_node(node.id())?;
            get_closest_call_expr(parent, ctx)
        }
        _ => None,
    }
}

struct Message;

impl Message {
    fn details(kind: JestFnKind) -> (&'static str, &'static str) {
        match kind {
            // (error, help)
            JestFnKind::General(JestGeneralFnKind::Describe) => (
                "Describe block title is used multiple times in the same describe block.",
                "Change the title of describe block.",
            ),
            JestFnKind::General(JestGeneralFnKind::Test) => (
                "Test title is used multiple times in the same describe block.",
                "Change the title of test.",
            ),
            _ => unreachable!(),
        }
    }
}

#[allow(clippy::too_many_lines)]
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

    Tester::new(NoIdenticalTitle::NAME, pass, fail).test_and_snapshot();
}
