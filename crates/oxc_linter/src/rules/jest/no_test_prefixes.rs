use std::borrow::Borrow;

use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::Fix,
    jest_ast_util::{parse_jest_fn_call, JestFnKind, ParsedJestFnCall},
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-test-prefixes): Use {0:?} instead.")]
#[diagnostic(severity(warning))]
struct NoTestPrefixesDiagnostic(Atom, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoTestPrefixes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require using `.only` and `.skip` over `f` and `x`.
    ///
    /// ### Why is this bad?
    ///
    /// Jest allows you to choose how you want to define focused and skipped tests,
    /// with multiple permutations for each:
    /// - only & skip: it.only, test.only, describe.only, it.skip, test.skip, describe.skip.
    /// - 'f' & 'x': fit, fdescribe, xit, xtest, xdescribe.
    ///
    /// This rule enforces usages from the only & skip list.
    ///
    /// ### Example
    /// ```javascript
    /// fit('foo'); // invalid
    /// fdescribe('foo'); // invalid
    /// xit('foo'); // invalid
    /// xtest('foo'); // invalid
    /// xdescribe('foo'); // invalid
    /// ```
    NoTestPrefixes,
    nursery
);

fn get_preferred_node_names(jest_fn_call: &ParsedJestFnCall) -> Atom {
    let ParsedJestFnCall { members, raw, .. } = jest_fn_call;

    let preferred_modifier = if raw.starts_with('f') { "only" } else { "skip" };
    let member_names = members.iter().map(Borrow::borrow).collect::<Vec<&str>>().join(".");
    let name_slice = &raw[1..];

    if member_names.is_empty() {
        Atom::from(format!("{name_slice}.{preferred_modifier}"))
    } else {
        Atom::from(format!("{name_slice}.{preferred_modifier}.{member_names}"))
    }
}

impl Rule for NoTestPrefixes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if let Some(jest_fn_call) = parse_jest_fn_call(call_expr, ctx) {
                let ParsedJestFnCall { kind, raw, .. } = &jest_fn_call;

                if !matches!(kind, JestFnKind::Describe | JestFnKind::Test) {
                    return;
                }

                if !raw.starts_with('f') && !raw.starts_with('x') {
                    return;
                }

                let span = match &call_expr.callee {
                    Expression::TaggedTemplateExpression(tagged_template_expr) => {
                        tagged_template_expr.tag.span()
                    }
                    Expression::CallExpression(child_call_expr) => child_call_expr.callee.span(),
                    _ => call_expr.callee.span(),
                };

                let preferred_node_name = get_preferred_node_names(&jest_fn_call);
                let preferred_node_name_cloned = preferred_node_name.clone();

                ctx.diagnostic_with_fix(
                    NoTestPrefixesDiagnostic(preferred_node_name, span),
                    || Fix::new(preferred_node_name_cloned.to_string(), span),
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("describe('foo', function () {})", None),
        ("it('foo', function () {})", None),
        ("it.concurrent('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("describe.only('foo', function () {})", None),
        ("it.only('foo', function () {})", None),
        ("it.each()('foo', function () {})", None),
        ("it.each``('foo', function () {})", None),
        ("test.only('foo', function () {})", None),
        ("test.each()('foo', function () {})", None),
        ("test.each``('foo', function () {})", None),
        ("describe.skip('foo', function () {})", None),
        ("it.skip('foo', function () {})", None),
        ("test.skip('foo', function () {})", None),
        ("foo()", None),
        ("[1,2,3].forEach()", None),
    ];

    let fail = vec![
        ("fdescribe('foo', function () {})", None),
        ("xdescribe.each([])('foo', function () {})", None),
        ("fit('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xit.each``('foo', function () {})", None),
        ("xtest.each``('foo', function () {})", None),
        ("xit.each([])('foo', function () {})", None),
        ("xtest.each([])('foo', function () {})", None), // TODO: Continue work on it when [#510](https://github.com/Boshen/oxc/issues/510) solved
                                                         // (r#"import { xit } from '@jest/globals';
                                                         // xit("foo", function () {})"#, None),
                                                         // (r#"import { xit as skipThis } from '@jest/globals';
                                                         // skipThis("foo", function () {})"#, None),
                                                         // (r#"import { fit as onlyThis } from '@jest/globals';
                                                         // onlyThis("foo", function () {})"#, None)
    ];

    Tester::new(NoTestPrefixes::NAME, pass, fail).test_and_snapshot();
}
