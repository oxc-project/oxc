use std::fmt::format;

use itertools::Itertools;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, IdentifierName, Statement},
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

#[derive(Debug, Eq, PartialEq)]
enum ExpectPairStates {
    WaitingOnce,
    WaitingWith,
    Paired,
}

#[derive(Debug)]
struct TrackingExpectPair {
    span_to_substitue: Span,
    span_to_remove: Span,
    identifier: CompactStr,
    args_to_be_expected: CompactStr,
    type_parameters: Option<CompactStr>,
    current_state: ExpectPairStates,
}

impl TrackingExpectPair {
    fn new_from_called_once(matcher_span: Span, identifier: CompactStr) -> Self {
        Self {
            span_to_substitue: matcher_span,
            span_to_remove: Span::empty(0),
            identifier,
            args_to_be_expected: CompactStr::new(""),
            type_parameters: None,
            current_state: ExpectPairStates::WaitingWith,
        }
    }

    fn new_from_called_with(
        matcher_span: Span,
        identifier: CompactStr,
        arguments: CompactStr,
        type_parameters: Option<CompactStr>,
    ) -> Self {
        Self {
            span_to_substitue: matcher_span,
            span_to_remove: Span::empty(0),
            identifier,
            args_to_be_expected: arguments,
            type_parameters,
            current_state: ExpectPairStates::WaitingOnce,
        }
    }

    fn update_tracking_with_called_once_information(&mut self, matcher_span: Span) {
        self.span_to_remove = matcher_span;
        self.current_state = ExpectPairStates::Paired;
    }

    fn update_tracking_with_called_with_information(
        &mut self,
        matcher_span: Span,
        identifier: CompactStr,
        arguments: CompactStr,
        type_parameters: Option<CompactStr>,
    ) {
        self.span_to_remove = matcher_span;
        self.identifier = identifier;
        self.args_to_be_expected = arguments;
        self.type_parameters = type_parameters;
        self.current_state = ExpectPairStates::Paired;
    }

    fn is_paired(&self) -> bool {
        self.current_state == ExpectPairStates::Paired
    }

    fn get_new_expect(&self) -> CompactStr {
        let type_params = self
            .type_parameters
            .as_ref()
            .map(|formatted| CompactStr::new(formatted.as_ref()))
            .unwrap_or(CompactStr::new(""));

        let expect = format!(
            "expect({}).toHaveBeenCalledExactlyOnceWith{}({})",
            self.identifier, type_params, self.args_to_be_expected
        );
        CompactStr::new(expect.as_ref())
    }

    fn is_expected_matcher(&self, matcher: &str) -> bool {
        if self.is_paired() {
            return false;
        }

        if self.current_state == ExpectPairStates::WaitingOnce && matcher == "toHaveBeenCalledOnce"
        {
            return false;
        }

        if self.current_state == ExpectPairStates::WaitingWith && matcher == "toHaveBeenCalledWith"
        {
            return false;
        }

        return true;
    }
}

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
            //TODO CAPACITY
            let mut set_matchers = FxHashSet::default();
            set_matchers.insert(CompactStr::new("toHaveBeenCalledOnce"));
            set_matchers.insert(CompactStr::new("toHaveBeenCalledWith"));

            set_matchers
        };

        let mock_reset_methods = {
            //TODO CAPACITY
            let mut mock_reset_methods_set = FxHashSet::default();
            mock_reset_methods_set.insert(CompactStr::new("mockClear"));
            mock_reset_methods_set.insert(CompactStr::new("mockReset"));
            mock_reset_methods_set.insert(CompactStr::new("mockRestore"));

            mock_reset_methods_set
        };

        let mut variables_expected: FxHashMap<CompactStr, TrackingExpectPair> =
            FxHashMap::default();

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
                .map(|expects| expects.is_expected_matcher(matcher_name.as_ref()))
                .unwrap_or(false);

            if duplicate_entry {
                variables_expected.remove(&variable_expected_name);
                continue;
            }

            // TODO MatcherKindEnum
            match matcher_name.as_ref() {
                "toHaveBeenCalledOnce" => {
                    if let Some(expect) = variables_expected.get_mut(&variable_expected_name) {
                        let statement_span = GetSpan::span(statement);
                        let mut start_remove = statement_span.start;

                        while !ctx
                            .source_range(Span::new(start_remove, statement_span.end + 1))
                            .starts_with('\n')
                        {
                            start_remove = start_remove - 1;
                        }

                        let next_line_statemen_span =
                            Span::new(start_remove + 1, statement_span.end + 1);

                        expect
                            .update_tracking_with_called_once_information(next_line_statemen_span);
                    } else {
                        variables_expected.insert(
                            variable_expected_name.clone(),
                            TrackingExpectPair::new_from_called_once(
                                call_expr.span,
                                variable_expected_name.clone(),
                            ),
                        );
                    };
                }

                "toHaveBeenCalledWith" => {
                    let to_be_arguments = expect_call
                        .matcher_arguments
                        .map(|arguments| {
                            arguments
                                .iter()
                                .map(|arg| ctx.source_range(GetSpan::span(arg)))
                                .join(", ")
                        })
                        .map(|arg_str| CompactStr::new(arg_str.as_ref()))
                        .unwrap_or(CompactStr::new(""));

                    let type_notation = call_expr
                        .type_arguments
                        .as_ref()
                        .map(|type_notation| CompactStr::new(ctx.source_range(type_notation.span)));

                    if let Some(expect) = variables_expected.get_mut(&variable_expected_name) {
                        let statement_span = GetSpan::span(statement);

                        let mut start_remove = statement_span.start;

                        while !ctx
                            .source_range(Span::new(start_remove, statement_span.end + 1))
                            .starts_with('\n')
                        {
                            start_remove = start_remove - 1;
                        }

                        let next_line_statemen_span =
                            Span::new(start_remove + 1, statement_span.end + 1);

                        expect.update_tracking_with_called_with_information(
                            next_line_statemen_span,
                            variable_expected_name,
                            to_be_arguments,
                            type_notation,
                        );
                    } else {
                        variables_expected.insert(
                            variable_expected_name.clone(),
                            TrackingExpectPair::new_from_called_with(
                                call_expr.span,
                                variable_expected_name.clone(),
                                to_be_arguments,
                                type_notation,
                            ),
                        );
                    };
                }
                _ => {}
            }
        }

        for expects in variables_expected.values() {
            if !expects.is_paired() {
                continue;
            }

            ctx.diagnostic_with_fix(
                prefer_called_exactly_once_with_diagnostic(Span::empty(0)),
                |fixer| {
                    let mut multiple_fixes = fixer.new_fix_with_capacity(2);
                    multiple_fixes.push(fixer.delete_range(expects.span_to_remove));
                    let substitute = expects.get_new_expect();
                    multiple_fixes.push(fixer.replace(expects.span_to_substitue, substitute));

                    multiple_fixes.with_message("Successfully fixed")
                },
            );
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
        "
			      expect(x).toHaveBeenCalledWith('hoge');
			      expect(x).toHaveBeenCalledOnce();
			      ",
        "
			      expect(x).toHaveBeenCalledWith('hoge', 123);
			      expect(x).toHaveBeenCalledOnce();
			      ",
        "
			      expect(x).toHaveBeenCalledWith('hoge', 123);
			      expect(x).toHaveBeenCalledOnce();
			      expect(y).toHaveBeenCalledWith('foo', 456);
			      expect(y).toHaveBeenCalledOnce();
			      ",
        "
			      expect(x).toHaveBeenCalledWith('hoge', 123);
			      const hoge = 'foo';
			      expect(x).toHaveBeenCalledOnce();
			      ",
        "
			      expect(x).toHaveBeenCalledOnce();
			      y.mockClear();
			      expect(x).toHaveBeenCalledWith('hoge');
			      ",
        "
			      expect(x).toHaveBeenCalledOnce();
			      expect(x).toHaveBeenCalledWith<[string]>('hoge');
			      ",
        "
			      expect(x).toHaveBeenCalledWith<[string]>('hoge');
			      expect(x).toHaveBeenCalledOnce();
			      ",
        "
			      expect(x).toHaveBeenCalledOnce<[number]>();
			      expect(x).toHaveBeenCalledWith<[string]>('hoge');
			      ",
        "
			      expect(x).toHaveBeenCalledOnce();
			      expect(x).toHaveBeenCalledWith<
			        [
			          {
			            id: number
			          }
			        ]
			      >('hoge');
			      ",
        "
			      expect(x).toHaveBeenCalledWith<[string, number]>('hoge', 123);
			      expect(x).toHaveBeenCalledOnce();
			      ",
        "
			      expect(x).toHaveBeenCalledWith<[string, number]>('hoge', 123);
			      expect(x).toHaveBeenCalledOnce();
			      expect(y).toHaveBeenCalledWith('foo', 456);
			      expect(y).toHaveBeenCalledOnce();
			      ",
        "
			      expect(x).toHaveBeenCalledOnce();
			      y.mockClear();
			      expect(x).toHaveBeenCalledWith<[string]>('hoge');
			      ",
    ];

    let fix = vec![
        (
            "
			      expect(x).toHaveBeenCalledOnce();
			      expect(x).toHaveBeenCalledWith('hoge');
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith('hoge');
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledWith('hoge');
			      expect(x).toHaveBeenCalledOnce();
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith('hoge');
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledWith('hoge', 123);
			      expect(x).toHaveBeenCalledOnce();
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith('hoge', 123);
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledWith('hoge', 123);
			      expect(x).toHaveBeenCalledOnce();
			      expect(y).toHaveBeenCalledWith('foo', 456);
			      expect(y).toHaveBeenCalledOnce();
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith('hoge', 123);
			      expect(y).toHaveBeenCalledExactlyOnceWith('foo', 456);
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledWith('hoge', 123);
			      const hoge = 'foo';
			      expect(x).toHaveBeenCalledOnce();
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith('hoge', 123);
			      const hoge = 'foo';
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledOnce();
			      y.mockClear();
			      expect(x).toHaveBeenCalledWith('hoge');
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith('hoge');
			      y.mockClear();
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledOnce();
			      expect(x).toHaveBeenCalledWith<[string]>('hoge');
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith<[string]>('hoge');
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledWith<[string]>('hoge');
			      expect(x).toHaveBeenCalledOnce();
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith<[string]>('hoge');
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledOnce<[number]>();
			      expect(x).toHaveBeenCalledWith<[string]>('hoge');
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith<[string]>('hoge');
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledOnce();
			      expect(x).toHaveBeenCalledWith<
			        [
			          {
			            id: number
			          }
			        ]
			      >('hoge');
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith<
			        [
			          {
			            id: number
			          }
			        ]
			      >('hoge');
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledWith<[string, number]>('hoge', 123);
			      expect(x).toHaveBeenCalledOnce();
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith<[string, number]>('hoge', 123);
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledWith<[string, number]>('hoge', 123);
			      expect(x).toHaveBeenCalledOnce();
			      expect(y).toHaveBeenCalledWith('foo', 456);
			      expect(y).toHaveBeenCalledOnce();
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith<[string, number]>('hoge', 123);
			      expect(y).toHaveBeenCalledExactlyOnceWith('foo', 456);
			      ",
            None,
        ),
        (
            "
			      expect(x).toHaveBeenCalledOnce();
			      y.mockClear();
			      expect(x).toHaveBeenCalledWith<[string]>('hoge');
			      ",
            "
			      expect(x).toHaveBeenCalledExactlyOnceWith<[string]>('hoge');
			      y.mockClear();
			      ",
            None,
        ),
    ];
    Tester::new(PreferCalledExactlyOnceWith::NAME, PreferCalledExactlyOnceWith::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
