use itertools::Itertools;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, FunctionBody, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, KnownMemberExpressionProperty, PossibleJestNode,
        parse_expect_jest_fn_call, parse_jest_fn_call,
    },
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

#[derive(Debug, Eq, PartialEq, Hash)]
enum MatcherKind {
    ToHaveBeenCalledOnce,
    ToHaveBeenCalledWith,
    Unknown,
}

impl MatcherKind {
    pub fn from(name: &str) -> Self {
        match name {
            "toHaveBeenCalledOnce" => Self::ToHaveBeenCalledOnce,
            "toHaveBeenCalledWith" => Self::ToHaveBeenCalledWith,
            _ => Self::Unknown,
        }
    }
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

    fn is_expected_matcher(&self, matcher: &MatcherKind) -> bool {
        if self.is_paired() {
            return false;
        }

        if self.current_state == ExpectPairStates::WaitingOnce
            && matcher == &MatcherKind::ToHaveBeenCalledOnce
        {
            return false;
        }

        if self.current_state == ExpectPairStates::WaitingWith
            && matcher == &MatcherKind::ToHaveBeenCalledWith
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
            let mut set_matchers = FxHashSet::with_capacity_and_hasher(2, FxBuildHasher);
            set_matchers.insert(MatcherKind::ToHaveBeenCalledOnce);
            set_matchers.insert(MatcherKind::ToHaveBeenCalledWith);

            set_matchers
        };

        let mock_reset_methods = {
            let mut mock_reset_methods_set = FxHashSet::with_capacity_and_hasher(3, FxBuildHasher);
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

            if is_mock_reset_call_expression(call_expr, &mock_reset_methods) {
                let Some(Expression::Identifier(identify)) =
                    call_expr.callee.as_member_expression().map(|member| member.object())
                else {
                    continue;
                };

                variables_expected.remove(&CompactStr::new(identify.name.as_ref()));

                continue;
            }

            let Some(parsed_vitest_fn) =
                parse_jest_fn_call(call_expr, &PossibleJestNode { node, original: None }, ctx)
            else {
                continue;
            };

            if matches!(parsed_vitest_fn.kind(), JestFnKind::General(JestGeneralFnKind::Test)) {
                let Some(callback) = get_test_callback(call_expr) else {
                    return;
                };

                let Some(body) = get_callback_body(callback) else {
                    return;
                };

                self.check_block_body(&body.statements, node, ctx);
                continue;
            }

            let Some(expect_call) = parse_expect_jest_fn_call(
                call_expr,
                &PossibleJestNode { node, original: None },
                ctx,
            ) else {
                continue;
            };

            if expect_call.members.iter().any(is_not_modifier_member) {
                continue;
            }

            let Some(matcher_index) = expect_call.matcher_index else {
                continue;
            };

            let Some(matcher) = expect_call
                .members
                .get(matcher_index)
                .and_then(|matcher| matcher.name())
                .map(|matcher_name| MatcherKind::from(matcher_name.as_ref()))
            else {
                continue;
            };

            if !matchers_to_combine.contains(&matcher) {
                continue;
            };

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
                .map(|expects| expects.is_expected_matcher(&matcher))
                .unwrap_or(false);

            if duplicate_entry {
                variables_expected.remove(&variable_expected_name);
                continue;
            }

            match matcher {
                MatcherKind::ToHaveBeenCalledOnce => {
                    if let Some(expect) = variables_expected.get_mut(&variable_expected_name) {
                        let statement_span = GetSpan::span(statement);

                        expect.update_tracking_with_called_once_information(
                            get_source_code_line_span(statement_span, ctx),
                        );
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

                MatcherKind::ToHaveBeenCalledWith => {
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

                        expect.update_tracking_with_called_with_information(
                            get_source_code_line_span(statement_span, ctx),
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
                MatcherKind::Unknown => {}
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

fn is_not_modifier_member(member: &KnownMemberExpressionProperty<'_>) -> bool {
    member.is_name_equal("not")
}

fn is_mock_reset_call_expression<'a>(
    call_expr: &'a CallExpression<'a>,
    mock_reset_methods: &FxHashSet<CompactStr>,
) -> bool {
    let Some(callee) = call_expr.callee_name() else {
        return false;
    };

    if !mock_reset_methods.contains(callee) {
        return false;
    }

    return true;
}

/**
 * Eslint fix is based on deleting the complete line of code. Span currently ignores the
 * whitespaces, so the test were failing due the trailing whitespaces not being removed.
 * Currently I'm asuming after the end of the statement, the next span position is the following line.
 * Even doing it safely the end check, this fix will remain dangerous as it remove code.
 */
fn get_source_code_line_span(statement_span: Span, ctx: &LintContext<'_>) -> Span {
    let mut column_0_span_index = statement_span.start;

    while !ctx
        .source_range(Span::new(column_0_span_index - 1, statement_span.end + 1))
        .starts_with('\n')
    {
        column_0_span_index = column_0_span_index - 1;
    }

    Span::new(column_0_span_index, statement_span.end + 1)
}

fn get_test_callback<'a>(call_expr: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    let args = &call_expr.arguments;

    // Find the callback function (last function argument)
    for arg in args.iter().rev() {
        if let Some(expr) = arg.as_expression()
            && matches!(
                expr,
                Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
            )
        {
            return Some(expr);
        }
    }

    None
}

fn get_callback_body<'a>(callback: &'a Expression<'a>) -> Option<&'a FunctionBody<'a>> {
    match callback {
        Expression::FunctionExpression(func) => func.body.as_ref().map(|body| body.as_ref()),
        Expression::ArrowFunctionExpression(func) => Some(&func.body),
        _ => None,
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
			      test('example',() => {
			        expect(x).toHaveBeenCalledWith('hoge', 123);
			        expect(x).toHaveBeenCalledOnce();
			      });
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
			      test('example',() => {
			        expect(x).toHaveBeenCalledWith('hoge', 123);
			        expect(x).toHaveBeenCalledOnce();
			      });
			      ",
            "
			      test('example',() => {
			        expect(x).toHaveBeenCalledExactlyOnceWith('hoge', 123);
			      });
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
