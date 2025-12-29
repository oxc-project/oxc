use itertools::Itertools;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::{ParsedExpectFnCall, PossibleJestNode, parse_expect_and_typeof_vitest_fn_call},
};

fn prefer_called_exactly_once_with_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferCalledExactlyOnceWith;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    PreferCalledExactlyOnceWith,
    vitest,
    style,
    fix,
);

impl Rule for PreferCalledExactlyOnceWith {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(program) => {
                self.check_block_body(&program.body, node, ctx);
            }
            AstKind::BlockStatement(block_statement) => {
                self.check_block_body(&block_statement.body, node, ctx);
            }
            _ => {}
        }
    }
}

impl PreferCalledExactlyOnceWith {
    fn check_block_body<'a>(
        &self,
        statements: &'a OxcVec<'a, Statement<'_>>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let matchers_to_combine = {
            let mut set_matchers = FxHashSet::default();
            set_matchers.insert(CompactStr::new("toHaveBeenCalledOnce"));
            set_matchers.insert(CompactStr::new("toHaveBeenCalledWith"));

            set_matchers
        };

        let mock_reset_methods = {
            let mut mock_reset_methods_set = FxHashSet::default();
            mock_reset_methods_set.insert(CompactStr::new("mockClear"));
            mock_reset_methods_set.insert(CompactStr::new("mockReset"));
            mock_reset_methods_set.insert(CompactStr::new("mockRestore"));

            mock_reset_methods_set
        };

        let mut variables_expected: FxHashMap<
            CompactStr,
            Vec<(CompactStr, ParsedExpectFnCall<'_>)>,
        > = FxHashMap::default();

        for statement in statements {
            let Statement::ExpressionStatement(statement_expression) = statement else {
                continue;
            };

            let Expression::CallExpression(call_expr) = &statement_expression.expression else {
                continue;
            };

            let Some(expect_call) = parse_expect_and_typeof_vitest_fn_call(
                call_expr,
                &PossibleJestNode { node, original: None },
                ctx,
            ) else {
                // Wrap in a fn to be understood
                let Some(callee) = call_expr.callee_name() else {
                    continue;
                };

                if !mock_reset_methods.contains(callee) {
                    continue;
                }

                let Some(Expression::Identifier(identify)) =
                    call_expr.callee.as_member_expression().map(|member| member.object())
                else {
                    continue;
                };

                variables_expected.remove(&CompactStr::new(identify.name.as_ref()));

                continue;
            };

            if expect_call.members.iter().any(|member| member.is_name_equal("not")) {
                continue;
            }

            let Some(matcher_index) = expect_call.matcher_index else {
                continue;
            };

            let Some(matcher) = expect_call.members.get(matcher_index) else {
                continue;
            };

            let Some(matcher_name) = matcher.name() else {
                continue;
            };

            if !matchers_to_combine.contains(matcher_name.as_ref()) {
                continue;
            };

            // TODO CHANGE FOR IDENTITY REFERENCE
            let Some(arguments) = expect_call.expect_arguments else {
                continue;
            };

            let arguments_key = arguments
                .iter()
                .map(|argument| ctx.source_range(GetSpan::span(argument)))
                .join(", ");

            let variable_expected_name = CompactStr::new(arguments_key.as_ref());

            let duplicate_entry = variables_expected
                .get(&variable_expected_name)
                .map(|expects| {
                    expects
                        .iter()
                        .any(|(matcher_saved, _span)| matcher_saved == matcher_name.as_ref())
                })
                .unwrap_or(false);

            if duplicate_entry {
                variables_expected.remove(&variable_expected_name);
                continue;
            }

            if let Some(expects) = variables_expected.get_mut(&variable_expected_name) {
                expects.push((CompactStr::new(matcher_name.as_ref()), expect_call));
            } else {
                variables_expected.insert(
                    variable_expected_name,
                    vec![(CompactStr::new(matcher_name.as_ref()), expect_call)],
                );
            };
        }

        for (_variable, expects) in variables_expected.iter() {
            if expects.len() != 2 {
                continue;
            }

            ctx.diagnostic(prefer_called_exactly_once_with_diagnostic(Span::empty(0)));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect(fn).toHaveBeenCalledExactlyOnceWith();",
        "expect(x).toHaveBeenCalledExactlyOnceWith(args);",
        "expect(x).toHaveBeenCalledOnce();",
        "expect(x).toHaveBeenCalledWith('hoge');",
        "
			    expect(x).toHaveBeenCalledOnce();
			    expect(y).toHaveBeenCalledWith('hoge');
			    ",
        "
			    expect(x).toHaveBeenCalledWith('hoge');
			    expect(x).toHaveBeenCalledWith('foo');
			    ",
        "
			    expect(x).toHaveBeenCalledOnce();
			    expect(x).not.toHaveBeenCalledWith('hoge');
			    ",
        "
			    expect(x).not.toHaveBeenCalledOnce();
			    expect(x).toHaveBeenCalledWith('hoge');
			    ",
        "
			    expect(x).not.toHaveBeenCalledOnce();
			    expect(x).not.toHaveBeenCalledWith('hoge');
			    ",
        "
			    expect(x).toHaveBeenCalledOnce();
			    x.mockRestore();
			    expect(x).toHaveBeenCalledWith('hoge');
			    ",
        "
			    expect(x).toHaveBeenCalledOnce();
			    x.mockReset();
			    expect(x).toHaveBeenCalledWith('hoge');
			    ",
        "
			    expect(x).toHaveBeenCalledOnce();
			    x.mockClear();
			    expect(x).toHaveBeenCalledWith('hoge');
			    ",
        "
			    expect(x).toHaveBeenCalledOnce();
			    y.mockClear();
			    expect(y).toHaveBeenCalledWith('hoge');
			    ",
        "expect(fn).toHaveBeenCalledExactlyOnceWith<[{ id: number }]>()",
        "expect(fn).toHaveBeenCalledExactlyOnceWith<[{ id: number }]>({id: 1})",
    ];

    let fail = vec![
        "
			      expect(x).toHaveBeenCalledOnce();
			      expect(x).toHaveBeenCalledWith('hoge');
			      ",
    ];

    let fix = vec![(
        "
			      expect(x).toHaveBeenCalledOnce();
			      expect(x).toHaveBeenCalledWith('hoge');
			      ",
        "
			      expect(x).toHaveBeenCalledExactlyOnceWith('hoge');
			      ",
        None,
    )];
    Tester::new(PreferCalledExactlyOnceWith::NAME, PreferCalledExactlyOnceWith::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
