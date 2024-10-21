use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::PossibleJestNode};

fn add_type_parameter_to_module_mock_diagnostic(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "`jest.mock()` factories should not be used without an explicit type parameter.",
    )
    .with_help(format!("Add a type parameter to the mock factory such as `typeof import({x0:?})`"))
    .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct NoUntypedMockFactory;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers a warning if `mock()` or `doMock()` is used without a generic
    /// type parameter or return type.
    ///
    /// ### Why is this bad?
    ///
    /// By default, `jest.mock` and `jest.doMock` allow any type to be returned by a
    /// mock factory. A generic type parameter can be used to enforce that the factory
    /// returns an object with the same shape as the original module, or some other
    /// strict type. Requiring a type makes it easier to use TypeScript to catch changes
    /// needed in test mocks when the source module changes.
    ///
    /// ### Example
    ///
    /// // invalid
    /// ```typescript
    /// jest.mock('../moduleName', () => {
    ///     return jest.fn(() => 42);
    /// });
    ///
    /// jest.mock('./module', () => ({
    ///     ...jest.requireActual('./module'),
    ///     foo: jest.fn(),
    /// }));
    ///
    /// jest.mock('random-num', () => {
    ///     return jest.fn(() => 42);
    /// });
    /// ```
    ///
    /// // valid
    /// ```typescript
    ///
    /// // Uses typeof import()
    /// jest.mock<typeof import('../moduleName')>('../moduleName', () => {
    ///     return jest.fn(() => 42);
    /// });
    ///
    /// jest.mock<typeof import('./module')>('./module', () => ({
    ///     ...jest.requireActual('./module'),
    ///     foo: jest.fn(),
    /// }));
    ///
    /// // Uses custom type
    /// jest.mock<() => number>('random-num', () => {
    ///     return jest.fn(() => 42);
    /// });
    ///
    /// // No factory
    /// jest.mock('random-num');
    ///
    /// // Virtual mock
    /// jest.mock(
    ///     '../moduleName',
    ///     () => {
    ///         return jest.fn(() => 42);
    ///     },
    ///     { virtual: true },
    /// );
    /// ```
    ///
    NoUntypedMockFactory,
    style,
    conditional_fix
);

impl Rule for NoUntypedMockFactory {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl NoUntypedMockFactory {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(mem_expr) = call_expr.callee.as_member_expression() else {
            return;
        };
        let Some((property_span, property_name)) = mem_expr.static_property_info() else {
            return;
        };

        if call_expr.arguments.len() != 2 || (property_name != "mock" && property_name != "doMock")
        {
            return;
        }

        let Some(factory_node) = call_expr.arguments.get(1) else {
            return;
        };

        if call_expr.type_parameters.is_some() || Self::has_return_type(factory_node) {
            return;
        }

        let Some(name_node) = call_expr.arguments.first() else {
            return;
        };
        let Some(expr) = name_node.as_expression() else {
            return;
        };

        if let Expression::StringLiteral(string_literal) = expr {
            ctx.diagnostic_with_fix(
                add_type_parameter_to_module_mock_diagnostic(
                    string_literal.value.as_str(),
                    property_span,
                ),
                |fixer| {
                    let mut content = fixer.codegen();
                    content.print_str("<typeof import('");
                    content.print_str(string_literal.value.as_str());
                    content.print_str("')>(");
                    let span = Span::sized(string_literal.span.start - 1, 1);

                    fixer.replace(span, content)
                },
            );
        } else if let Expression::Identifier(ident) = expr {
            ctx.diagnostic(add_type_parameter_to_module_mock_diagnostic(
                ident.name.as_str(),
                property_span,
            ));
        }
    }

    fn has_return_type(argument: &Argument) -> bool {
        let Some(expr) = argument.as_expression() else {
            return false;
        };

        match expr {
            Expression::FunctionExpression(func_expr) => func_expr.return_type.is_some(),
            Expression::ArrowFunctionExpression(arrow_func_expr) => {
                arrow_func_expr.return_type.is_some()
            }
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("jest.mock('random-number');", None),
        (
            "
                jest.mock<typeof import('../moduleName')>('../moduleName', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.mock<typeof import('./module')>('./module', () => ({
                    ...jest.requireActual('./module'),
                    foo: jest.fn()
                }));
            ",
            None,
        ),
        (
            "
                jest.mock<typeof import('foo')>('bar', () => ({
                    ...jest.requireActual('bar'),
                    foo: jest.fn()
                }));
            ",
            None,
        ),
        (
            "
                jest.doMock('./module', (): typeof import('./module') => ({
                    ...jest.requireActual('./module'),
                    foo: jest.fn()
                }));
            ",
            None,
        ),
        (
            "
                jest.mock('../moduleName', function (): typeof import('../moduleName') {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.mock('../moduleName', function (): (() => number) {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.mock<() => number>('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest['doMock']<() => number>('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.mock<any>('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.mock(
                    '../moduleName',
                    () => {
                        return jest.fn(() => 42)
                    },
                    {virtual: true},
                );
            ",
            None,
        ),
        // Should not match
        (
            "
                mockito<() => number>('foo', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        ("test.skip('basic', async () => {});", None),
    ];

    let fail = vec![
        (
            "
                jest.mock('../moduleName', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.mock(\"./module\", () => ({
                    ...jest.requireActual('./module'),
                    foo: jest.fn()
                }));
            ",
            None,
        ),
        (
            "
                jest.mock('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.doMock('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest['mock']('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                const moduleToMock = 'random-num';
                jest.mock(moduleToMock, () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
    ];

    let fix = vec![
        (
            "
                jest.mock('../moduleName', () => {
                    return jest.fn(() => 42);
                });
            ",
            "
                jest.mock<typeof import('../moduleName')>('../moduleName', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.mock('./module', () => ({
                    ...jest.requireActual('./module'),
                    foo: jest.fn()
                }));
            ",
            "
                jest.mock<typeof import('./module')>('./module', () => ({
                    ...jest.requireActual('./module'),
                    foo: jest.fn()
                }));
            ",
            None,
        ),
        (
            "
                jest.mock('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            "
                jest.mock<typeof import('random-num')>('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest.doMock('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            "
                jest.doMock<typeof import('random-num')>('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
        (
            "
                jest['mock']('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            "
                jest['mock']<typeof import('random-num')>('random-num', () => {
                    return jest.fn(() => 42);
                });
            ",
            None,
        ),
    ];

    Tester::new(NoUntypedMockFactory::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
