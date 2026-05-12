use oxc_allocator::GetAddress;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    rules::PossibleJestNode,
    utils::{KnownMemberExpressionProperty, parse_expect_and_typeof_vitest_fn_call},
};

fn require_awaited_expect_poll_diagnostic(span: Span, member_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`expect.{member_name}` must be awaited or returned"))
        .with_help(format!("Add `await` to the `expect.{member_name}` call."))
        .with_label(span)
}

fn require_awaited_expect_poll_return_diagnostic(span: Span, member_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`expect.{member_name}` must be awaited or returned as the last expression"))
          .with_help(format!("Add `await` to the `expect.{member_name}` call or move it to the last position in the sequence."))
          .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireAwaitedExpectPoll;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule ensures that promises returned by `expect.poll` and `expect.element` calls are handled properly.
    ///
    /// ### Why is this bad?
    ///
    /// `expect.poll` and `expect.element` return promises. If not awaited or returned,
    /// the test completes before the assertion resolves, meaning the test will pass
    /// regardless of whether the assertion succeeds or fails.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('element exists', () => {
    ///   asyncInjectElement()
    ///
    ///   expect.poll(() => document.querySelector('.element')).toBeInTheDocument()
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('element exists', () => {
    ///   asyncInjectElement()
    ///
    ///   return expect
    ///     .poll(() => document.querySelector('.element'))
    ///     .toBeInTheDocument()
    /// })
    /// test('element exists', async () => {
    /// asyncInjectElement()
    ///
    /// await expect
    ///     .poll(() => document.querySelector('.element'))
    ///     .toBeInTheDocument()
    /// })
    /// ```
    RequireAwaitedExpectPoll,
    vitest,
    correctness,
    version = "1.58.0",
);

impl Rule for RequireAwaitedExpectPoll {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &crate::rules::PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        RequireAwaitedExpectPoll::run(jest_node, ctx);
    }
}

impl RequireAwaitedExpectPoll {
    fn run<'a, 'c>(possible_jest_node: &PossibleJestNode<'a, 'c>, ctx: &'c LintContext<'a>) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(expect) =
            parse_expect_and_typeof_vitest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        if !expect.members.first().is_some_and(is_awaited_member) {
            return;
        }

        let top_most_node = skip_sequence_expressions(skip_matchers_and_modifiers(node, ctx), ctx);

        if is_returned_or_awaited(top_most_node, ctx) {
            return;
        }

        if is_in_return_context(top_most_node, ctx) {
            ctx.diagnostic(require_awaited_expect_poll_return_diagnostic(
                call_expr.span,
                expect.members.first().unwrap().name().unwrap().as_ref(),
            ));
        } else {
            ctx.diagnostic(require_awaited_expect_poll_diagnostic(
                call_expr.span,
                expect.members.first().unwrap().name().unwrap().as_ref(),
            ));
        }
    }
}

fn is_awaited_member(member: &KnownMemberExpressionProperty<'_>) -> bool {
    member.is_name_equal("poll") || member.is_name_equal("element")
}

fn skip_sequence_expressions<'a, 'c>(
    node: &AstNode<'a>,
    ctx: &'c LintContext<'a>,
) -> &'c AstNode<'a> {
    let mut current_node = ctx.semantic().nodes().get_node(node.id());

    loop {
        let parent = ctx.semantic().nodes().parent_node(current_node.id());
        let parent_kind = parent.kind();

        match parent_kind {
            AstKind::ParenthesizedExpression(_) => current_node = parent,
            AstKind::SequenceExpression(sequence) => {
                if sequence.expressions.last().is_some_and(|last_expression| {
                    last_expression.address() != current_node.address()
                }) {
                    break;
                }

                current_node = parent;
            }
            _ => break,
        }
    }

    current_node
}

fn skip_matchers_and_modifiers<'a, 'c>(
    node: &AstNode<'_>,
    ctx: &'c LintContext<'a>,
) -> &'c AstNode<'a> {
    let mut current_node = ctx.semantic().nodes().get_node(node.id());

    loop {
        let parent = ctx.semantic().nodes().parent_node(current_node.id());
        let parent_kind = parent.kind();
        if !matches!(
            parent_kind,
            AstKind::StaticMemberExpression(_)
                | AstKind::ComputedMemberExpression(_)
                | AstKind::PrivateFieldExpression(_)
                | AstKind::CallExpression(_)
        ) {
            break;
        }

        current_node = parent;
    }

    current_node
}

fn is_in_return_context<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let parent = ctx.semantic().nodes().parent_node(node.id());

    matches!(parent.kind(), AstKind::ReturnStatement(_))
}

fn is_returned_or_awaited<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let parent = ctx.semantic().nodes().parent_node(node.id());

    matches!(parent.kind(), AstKind::ReturnStatement(_) | AstKind::AwaitExpression(_))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
                    test('should pass', async () => {
                      await expect.poll(() => element).toBeInTheDocument();
                    });
                  ",
"
                    test('should pass', async () => {
                      await expect.element(element).toBeInTheDocument();
                    });
                  ",
"
                    test('should pass', () => {
                      expect.syncElement(element).toBeInTheDocument();
                    });
                  ",
"
                    test('should pass', () => {
                      return expect.poll(() => element).toBeInTheDocument();
                    });
                  ",
"
                    test('should pass', () => {
                      return expect.element(element).toBeInTheDocument();
                    });
                  ",
"
                    test('should pass', () => {
                      return expect(true).toBe(true);
                    });
                  ",
"
                    test('should pass', async () => {
                      (sideEffect(), await expect.poll(() => element).toBeInTheDocument());
                    });
                  ",
"
                    test('should pass', async () => {
                      await (sideEffect(), expect.poll(() => element).toBeInTheDocument());
                    });
                  ",
"
                    test('should pass', async () => {
                      await (sideEffect(), (sideEffect(), (sideEffect(), expect.poll(() => element).toBeInTheDocument())));
                    });
                  ",
"
                    test('should pass', () => {
                      return (sideEffect(), expect.poll(() => element).toBeInTheDocument());
                    });
                  "
    ];

    let fail = vec![
        "
                    test('should fail', () => {
                      expect.poll(() => element).toBeInTheDocument();
                    });
                  ",
"
                    test('should fail', () => {
                      expect.element(element).toBeInTheDocument();
                    });
                  ",
"
                    test('should fail', () => {
                      expect['poll'](() => element).toBeInTheDocument();
                    });
                  ",
"
                    test('should fail', () => {
                      expect['element'](element).toBeInTheDocument();
                    });
                  ",
"
                    test('should fail', () => {
                      (expect.poll(() => element).toBeInTheDocument(), expect(true).toBe(true));
                    });
                  ",
"
                    test('should fail', () => {
                      (expect.element(() => element).toBeInTheDocument(), expect(true).toBe(true));
                    });
                  ",
"
                    test('should fail', () => {
                      return (expect.poll(() => element).toBeInTheDocument(), expect(true).toBe(true));
                    });
                  "
    ];

    Tester::new(RequireAwaitedExpectPoll::NAME, RequireAwaitedExpectPoll::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
