use oxc_ast::{
    ast::{
        AssignmentExpression, AssignmentTarget, CallExpression, Expression, MemberExpression,
        SimpleAssignmentTarget,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, Fix};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-jasmine-globals): {0:?}")]
#[diagnostic(severity(warning), help("{1:?}"))]
struct NoJasmineGlobalsDiagnostic(pub &'static str, pub &'static str, #[label] pub Span);

/// <https://github.com/jest-community/eslint-plugin-jest/blob/main/docs/rules/no-jasmine-globals.md>
#[derive(Debug, Default, Clone)]
pub struct NoJasmineGlobals;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule reports on any usage of Jasmine globals, which is not ported to Jest, and suggests alternatives from Jest's own API.
    ///
    /// ### Example
    /// ```javascript
    /// jasmine.DEFAULT_TIMEOUT_INTERVAL = 5000;
    /// test('my test', () => {
    ///     pending();
    /// });
    /// test('my test', () => {
    ///     jasmine.createSpy();
    /// });
    /// ```
    NoJasmineGlobals,
    restriction
);

const NON_JASMINE_PROPERTY_NAMES: [&str; 4] = ["spyOn", "spyOnProperty", "fail", "pending"];

impl Rule for NoJasmineGlobals {
    fn run_once(&self, ctx: &LintContext) {
        let symbol_table = ctx.symbols();
        let jasmine_references = ctx
            .scopes()
            .root_unresolved_references()
            .iter()
            .filter(|(key, _)| NON_JASMINE_PROPERTY_NAMES.contains(&key.as_str()));

        for (name, reference_ids) in jasmine_references {
            for &reference_id in reference_ids {
                let reference = symbol_table.get_reference(reference_id);
                if let Some((error, help)) = get_non_jasmine_property_messages(name) {
                    ctx.diagnostic(NoJasmineGlobalsDiagnostic(error, help, reference.span()));
                }
            }
        }
    }

    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::AssignmentExpression(assign_expr) = node.kind() {
            diagnostic_assign_expr(assign_expr, ctx);
        } else if let AstKind::CallExpression(call_expr) = node.kind() {
            diagnostic_call_expr(call_expr, ctx);
        }
    }
}

fn diagnostic_assign_expr<'a>(expr: &'a AssignmentExpression<'a>, ctx: &LintContext) {
    if let AssignmentTarget::SimpleAssignmentTarget(
        SimpleAssignmentTarget::MemberAssignmentTarget(member_expr),
    ) = &expr.left
    {
        let (span, property_name) = match get_jasmine_property_name(member_expr) {
            Some(value) => value,
            None => return,
        };

        if property_name == "DEFAULT_TIMEOUT_INTERVAL" {
            // `jasmine.DEFAULT_TIMEOUT_INTERVAL = 5000` we can fix it to `jest.setTimeout(5000)`
            if let Expression::NumberLiteral(number_literal) = &expr.right {
                ctx.diagnostic_with_fix(
                    NoJasmineGlobalsDiagnostic(COMMON_ERROR_TEXT, COMMON_HELP_TEXT, span),
                    || {
                        let content = format!("jest.setTimeout({})", number_literal.value);
                        Fix::new(content, expr.span)
                    },
                );
                return;
            }
        }

        ctx.diagnostic(NoJasmineGlobalsDiagnostic(COMMON_ERROR_TEXT, COMMON_HELP_TEXT, span));
    }
}

fn diagnostic_call_expr<'a>(expr: &'a CallExpression<'a>, ctx: &LintContext) {
    if let Expression::MemberExpression(member_expr) = &expr.callee {
        let (span, property_name) = match get_jasmine_property_name(member_expr) {
            Some(value) => value,
            None => return,
        };

        JasmineProperty::from_str(property_name).map_or_else(
            || {
                ctx.diagnostic(NoJasmineGlobalsDiagnostic(
                    COMMON_ERROR_TEXT,
                    COMMON_HELP_TEXT,
                    span,
                ));
            },
            |jasmine_property| {
                let (error, help) = jasmine_property.details();
                if jasmine_property.available_in_jest_expect() {
                    ctx.diagnostic_with_fix(NoJasmineGlobalsDiagnostic(error, help, span), || {
                        Fix::new("expect", member_expr.object().span())
                    });
                } else {
                    ctx.diagnostic(NoJasmineGlobalsDiagnostic(error, help, span));
                }
            },
        );
    }
}

fn get_jasmine_property_name<'a>(member_expr: &'a MemberExpression<'a>) -> Option<(Span, &'a str)> {
    let name = match member_expr.object() {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        _ => None,
    };
    let is_jasmine_object = name.is_some_and(|name| name == "jasmine");
    if !is_jasmine_object {
        return None;
    }
    let Some((span, property_name)) = member_expr.static_property_info() else {
        return None;
    };
    Some((span, property_name))
}

const COMMON_ERROR_TEXT: &str = "Illegal usage of jasmine global";
const COMMON_HELP_TEXT: &str = "prefer use Jest own API";

fn get_non_jasmine_property_messages(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "spyOn" => Some(("Illegal usage of global spyOn", "prefer use Jest own API `jest.spyOn`")),
        "spyOnProperty" => {
            Some(("Illegal usage of global spyOnProperty", "prefer use Jest own API `jest.spyOn`"))
        }
        "fail" => Some((
            "Illegal usage of `fail`",
            "prefer throwing an error, or the `done.fail` callback",
        )),
        "pending" => Some((
            "Illegal usage of `pending`,",
            "prefer explicitly skipping a test using `test.skip`",
        )),
        _ => None,
    }
}

enum JasmineProperty {
    Any,
    Anything,
    ArrayContaining,
    ObjectContaining,
    StringMatching,
    AddMatchers,
    CreateSpy,
}

impl JasmineProperty {
    fn from_str(name: &str) -> Option<Self> {
        match name {
            "any" => Some(Self::Any),
            "anything" => Some(Self::Anything),
            "arrayContaining" => Some(Self::ArrayContaining),
            "objectContaining" => Some(Self::ObjectContaining),
            "stringMatching" => Some(Self::StringMatching),
            "addMatchers" => Some(Self::AddMatchers),
            "createSpy" => Some(Self::CreateSpy),
            _ => None,
        }
    }
    fn details(&self) -> (&'static str, &'static str) {
        match self {
            Self::Any => ("Illegal usage of `any`", "prefer use Jest own API `expect.any`"),
            Self::Anything => {
                ("Illegal usage of `anything`", "prefer use Jest own API `expect.anything`")
            }
            Self::ArrayContaining => (
                "Illegal usage of `arrayContaining`",
                "prefer use Jest own API `expect.arrayContaining`",
            ),
            Self::ObjectContaining => (
                "Illegal usage of `objectContaining`",
                "prefer use Jest own API `expect.objectContaining`",
            ),
            Self::StringMatching => (
                "Illegal usage of `stringMatching`",
                "prefer use Jest own API `expect.stringMatching`",
            ),
            Self::AddMatchers => {
                ("Illegal usage of `addMatchers`", "prefer use Jest own API `expect.extend`")
            }
            Self::CreateSpy => {
                ("Illegal usage of `createSpy`", "prefer use Jest own API `jest.fn`")
            }
        }
    }
    fn available_in_jest_expect(&self) -> bool {
        matches!(
            self,
            Self::Any
                | Self::Anything
                | Self::ArrayContaining
                | Self::ObjectContaining
                | Self::StringMatching
        )
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("jest.spyOn()", None),
        ("jest.fn()", None),
        ("expect.extend()", None),
        ("expect.any()", None),
        ("it('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("foo()", None),
        ("require('foo')('bar')", None),
        ("(function(){})()", None),
        ("function callback(fail) { fail() }", None),
        ("var spyOn = require('actions'); spyOn('foo')", None),
        ("function callback(pending) { pending() }", None),
    ];

    let fail = vec![
        ("spyOn(some, 'object')", None),
        ("spyOnProperty(some, 'object')", None),
        ("fail()", None),
        ("pending()", None),
        ("jasmine.DEFAULT_TIMEOUT_INTERVAL = 5000;", None),
        ("jasmine.DEFAULT_TIMEOUT_INTERVAL = function() {}", None),
        ("jasmine.addMatchers(matchers)", None),
        ("jasmine.createSpy()", None),
        ("jasmine.any()", None),
        ("jasmine.anything()", None),
        ("jasmine.arrayContaining()", None),
        ("jasmine.objectContaining()", None),
        ("jasmine.stringMatching()", None),
        ("jasmine.getEnv()", None),
        ("jasmine.empty()", None),
        ("jasmine.falsy()", None),
        ("jasmine.truthy()", None),
        ("jasmine.arrayWithExactContents()", None),
        ("jasmine.clock()", None),
        ("jasmine.MAX_PRETTY_PRINT_ARRAY_LENGTH = 42", None),
    ];

    let fix = vec![
        ("jasmine.DEFAULT_TIMEOUT_INTERVAL = 5", "jest.setTimeout(5)", None),
        (
            "jasmine.DEFAULT_TIMEOUT_INTERVAL = ()=>{}",
            "jasmine.DEFAULT_TIMEOUT_INTERVAL = ()=>{}",
            None,
        ),
        ("jasmine.any()", "expect.any()", None),
        ("jasmine.anything()", "expect.anything()", None),
        ("jasmine.arrayContaining()", "expect.arrayContaining()", None),
        ("jasmine.objectContaining()", "expect.objectContaining()", None),
        ("jasmine.stringMatching()", "expect.stringMatching()", None),
        ("jasmine.addMatchers(matchers)", "jasmine.addMatchers(matchers)", None),
        ("jasmine.getEnv()", "jasmine.getEnv()", None),
        ("jasmine.empty()", "jasmine.empty()", None),
        ("jasmine.falsy()", "jasmine.falsy()", None),
        ("jasmine.truthy()", "jasmine.truthy()", None),
        ("jasmine.arrayWithExactContents()", "jasmine.arrayWithExactContents()", None),
        ("jasmine.clock()", "jasmine.clock()", None),
        (
            "jasmine.MAX_PRETTY_PRINT_ARRAY_LENGTH = 42",
            "jasmine.MAX_PRETTY_PRINT_ARRAY_LENGTH = 42",
            None,
        ),
    ];

    Tester::new(NoJasmineGlobals::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
