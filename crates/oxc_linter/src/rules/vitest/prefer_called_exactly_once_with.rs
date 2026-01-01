use itertools::Itertools;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, FunctionBody, MemberExpression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use std::collections::BTreeMap;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        KnownMemberExpressionProperty, ParsedExpectFnCall, ParsedJestFnCallNew, PossibleJestNode,
        parse_jest_fn_call,
    },
};

fn prefer_called_exactly_once_with_diagnostic(
    substitute_span: Span,
    remove_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `toHaveBeenCalledExactlyOnceWith` over `toHaveBeenCalledOnce` and `toHaveBeenCalledWith` on the same target.")
        .with_labels([
            substitute_span.label("Replace with `toHaveBeenCalledExactlyOnceWith`"),
            remove_span.label("Remove this expect"),
        ])
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

    /// Returns true if this matcher can be combined with its counterpart
    /// to form `toHaveBeenCalledExactlyOnceWith`.
    fn is_combinable(&self) -> bool {
        matches!(self, Self::ToHaveBeenCalledOnce | Self::ToHaveBeenCalledWith)
    }
}

#[derive(Debug)]
struct TrackingExpectPair {
    span_to_substitute: Span,
    span_to_remove: Span,
    identifier: CompactStr,
    args_to_be_expected: CompactStr,
    type_parameters: Option<CompactStr>,
    current_state: ExpectPairStates,
}

impl TrackingExpectPair {
    fn new_from_called_once(matcher_span: Span, identifier: CompactStr) -> Self {
        Self {
            span_to_substitute: matcher_span,
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
            span_to_substitute: matcher_span,
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
            .map_or(CompactStr::new(""), |formatted| CompactStr::new(formatted.as_ref()));

        let expect = format!(
            "expect({}).toHaveBeenCalledExactlyOnceWith{}({})",
            self.identifier, type_params, self.args_to_be_expected
        );
        CompactStr::new(expect.as_ref())
    }

    /// Returns true if this tracking pair can be completed by pairing with the given matcher.
    /// This is used to detect when we have both `toHaveBeenCalledOnce` and `toHaveBeenCalledWith`
    /// on the same target, which can be combined into `toHaveBeenCalledExactlyOnceWith`.
    fn can_pair_with(&self, matcher: &MatcherKind) -> bool {
        if self.is_paired() {
            return false;
        }

        // If we're waiting for `toHaveBeenCalledOnce` and get it, we can pair
        // If we're waiting for `toHaveBeenCalledWith` and get it, we can pair
        // Otherwise, we can't pair (e.g., getting the same matcher type we already have)
        matches!(
            (&self.current_state, matcher),
            (ExpectPairStates::WaitingOnce, MatcherKind::ToHaveBeenCalledOnce)
                | (ExpectPairStates::WaitingWith, MatcherKind::ToHaveBeenCalledWith)
        )
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// It checks when a target is expected with `toHaveBeenCalledOnce` and `toHaveBeenCalledWith` instead of
    /// `toHaveBeenCalledExactlyOnceWith`.
    ///
    /// ### Why is this bad?
    ///
    /// The user must deduct from both expects that the spy function is called once and with a specific arguments.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('foo', () => {
    ///   const mock = vi.fn()
    ///   mock('foo')
    ///   expect(mock).toHaveBeenCalledOnce()
    ///   expect(mock).toHaveBeenCalledWith('foo')
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('foo', () => {
    ///   const mock = vi.fn()
    ///   mock('foo')
    ///   expect(mock).toHaveBeenCalledExactlyOnceWith('foo')
    /// })
    /// ```
    PreferCalledExactlyOnceWith,
    vitest,
    style,
    dangerous_fix,
);

impl Rule for PreferCalledExactlyOnceWith {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(program) => {
                Self::check_block_body(&program.body, node, ctx);
            }
            AstKind::BlockStatement(block_statement) => {
                Self::check_block_body(&block_statement.body, node, ctx);
            }
            _ => {}
        }
    }
}

/// Mock reset methods that clear the mock call history.
const MOCK_RESET_METHODS: [&str; 3] = ["mockClear", "mockReset", "mockRestore"];

impl PreferCalledExactlyOnceWith {
    fn check_block_body<'a>(
        statements: &'a OxcVec<'a, Statement<'_>>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let mut variables_expected: BTreeMap<CompactStr, TrackingExpectPair> = BTreeMap::default();

        for statement in statements {
            let Statement::ExpressionStatement(statement_expression) = statement else {
                continue;
            };

            let Expression::CallExpression(call_expr) = &statement_expression.expression else {
                continue;
            };

            let Some(parsed_call_expression_statement) =
                parse_call_expression_statement(call_expr, node, ctx)
            else {
                continue;
            };

            match parsed_call_expression_statement {
                TestCallExpression::MockReset => {
                    let Some(Expression::Identifier(identify)) =
                        call_expr.callee.as_member_expression().map(MemberExpression::object)
                    else {
                        continue;
                    };

                    variables_expected.remove(&CompactStr::new(identify.name.as_ref()));
                }
                TestCallExpression::TestBlock(statements) => {
                    Self::check_block_body(statements, node, ctx);
                }
                TestCallExpression::ExpectFnCall(expect_call) => {
                    let Some((variable_expected_name, matcher)) =
                        get_identifier_and_matcher_to_be_expected(&expect_call, ctx)
                    else {
                        continue;
                    };

                    let duplicate_entry = variables_expected
                        .get(&variable_expected_name)
                        .is_some_and(|expects| !expects.can_pair_with(&matcher));

                    if duplicate_entry {
                        variables_expected.remove(&variable_expected_name);
                        continue;
                    }

                    match matcher {
                        MatcherKind::ToHaveBeenCalledOnce => {
                            if let Some(expect) =
                                variables_expected.get_mut(&variable_expected_name)
                            {
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
                            }
                        }

                        MatcherKind::ToHaveBeenCalledWith => {
                            let to_be_arguments = expect_call.matcher_arguments.map_or(
                                CompactStr::new(""),
                                |arguments| {
                                    let arguments_to_be_expected = arguments
                                        .iter()
                                        .map(|arg| ctx.source_range(GetSpan::span(arg)))
                                        .join(", ");
                                    CompactStr::new(arguments_to_be_expected.as_ref())
                                },
                            );

                            let type_notation =
                                call_expr.type_arguments.as_ref().map(|type_notation| {
                                    CompactStr::new(ctx.source_range(type_notation.span))
                                });

                            if let Some(expect) =
                                variables_expected.get_mut(&variable_expected_name)
                            {
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
                            }
                        }
                        MatcherKind::Unknown => {}
                    }
                }
            }
        }

        for expects in variables_expected.values() {
            if !expects.is_paired() {
                continue;
            }

            ctx.diagnostic_with_dangerous_fix(
                prefer_called_exactly_once_with_diagnostic(
                    expects.span_to_substitute,
                    expects.span_to_remove,
                ),
                |fixer| {
                    let fixer = fixer.for_multifix();
                    let substitute = expects.get_new_expect();
                    fixer
                        .new_fix_with_capacity(2)
                        .extend(fixer.replace(expects.span_to_substitute, substitute))
                        .extend(fixer.delete_range(expects.span_to_remove))
                        .with_message("Replace with `toHaveBeenCalledExactlyOnceWith` and remove redundant expect")
                },
            );
        }
    }
}

enum TestCallExpression<'a> {
    TestBlock(&'a oxc_allocator::Vec<'a, Statement<'a>>),
    MockReset,
    ExpectFnCall(ParsedExpectFnCall<'a>),
}

fn parse_call_expression_statement<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<TestCallExpression<'a>> {
    if is_mock_reset_call_expression(call_expr) {
        return Some(TestCallExpression::MockReset);
    }

    match parse_jest_fn_call(call_expr, &PossibleJestNode { node, original: None }, ctx) {
        Some(ParsedJestFnCallNew::GeneralJest(_)) => {
            let callback = get_test_callback(call_expr)?;

            let body = get_callback_body(callback)?;

            Some(TestCallExpression::TestBlock(&body.statements))
        }
        Some(ParsedJestFnCallNew::Expect(expect_vitest_call)) => {
            Some(TestCallExpression::ExpectFnCall(expect_vitest_call))
        }
        _ => None,
    }
}

fn get_identifier_and_matcher_to_be_expected<'a>(
    expect_call: &ParsedExpectFnCall<'a>,
    ctx: &LintContext<'a>,
) -> Option<(CompactStr, MatcherKind)> {
    if expect_call.members.iter().any(is_not_modifier_member) {
        return None;
    }

    let matcher_index = expect_call.matcher_index?;

    let matcher = expect_call
        .members
        .get(matcher_index)
        .and_then(KnownMemberExpressionProperty::name)
        .map(|matcher_name| MatcherKind::from(matcher_name.as_ref()))?;

    if !matcher.is_combinable() {
        return None;
    }

    let arguments = expect_call.expect_arguments?;

    let identifier_name =
        arguments.iter().map(|argument| ctx.source_range(GetSpan::span(argument))).join(", ");

    Some((CompactStr::new(identifier_name.as_ref()), matcher))
}

fn is_not_modifier_member(member: &KnownMemberExpressionProperty<'_>) -> bool {
    member.is_name_equal("not")
}

fn is_mock_reset_call_expression(call_expr: &CallExpression<'_>) -> bool {
    call_expr.callee_name().is_some_and(|callee| MOCK_RESET_METHODS.contains(&callee))
}

/**
 * Eslint fix is based on deleting the complete line of code. Span currently ignores the
 * whitespaces, so the test were failing due the trailing whitespaces not being removed.
 * Currently the method is asumming after the end of the statement, the next span position is the following line.
 * Even doing it safely the end check, this fix will remain dangerous as it removes code.
 */
fn get_source_code_line_span(statement_span: Span, ctx: &LintContext<'_>) -> Span {
    let mut column_0_span_index = statement_span.start;

    // Guard against underflow when statement is at the beginning of the file
    while column_0_span_index > 0
        && !ctx
            .source_range(Span::new(column_0_span_index - 1, statement_span.end + 1))
            .starts_with('\n')
    {
        column_0_span_index -= 1;
    }

    Span::new(column_0_span_index, statement_span.end + 1)
}

fn get_test_callback<'a>(call_expr: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    call_expr.arguments.iter().rev().filter_map(|arg| arg.as_expression()).find(|expr| {
        matches!(expr, Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_))
    })
}

fn get_callback_body<'a>(callback: &'a Expression<'a>) -> Option<&'a FunctionBody<'a>> {
    match callback {
        Expression::FunctionExpression(func) => func.body.as_ref().map(AsRef::as_ref),
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
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
