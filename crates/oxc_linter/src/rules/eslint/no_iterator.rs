use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_iterator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Reserved name '__iterator__'")
        .with_help("Consider using [Symbol.iterator] instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoIterator;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow the use of the `__iterator__` property
    ///
    /// ### Why is this bad?
    /// The `__iterator__` property was a SpiderMonkey extension to JavaScript
    /// that could be used to create custom iterators that are compatible with
    /// JavaScript’s for in and for each constructs. However, this property is
    /// now obsolete, so it should not be used. Here’s an example of how this
    /// used to work:
    ///
    /// ```js
    /// Foo.prototype.__iterator__ = function() {
    ///     return new FooIterator(this);
    /// }
    /// ```
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// Foo.prototype.__iterator__ = function() {
    ///     return new FooIterator(this);
    /// };
    ///
    /// foo.__iterator__ = function () {};
    ///
    /// foo["__iterator__"] = function () {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const __iterator__ = 42; // not using the __iterator__ property
    ///
    /// Foo.prototype[Symbol.iterator] = function() {
    ///    return new FooIterator(this);
    /// };
    /// ```
    NoIterator,
    eslint,
    restriction,
    pending // TODO: suggestion
);

impl Rule for NoIterator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MemberExpression(member_expression) = node.kind() else {
            return;
        };
        if let Some(static_property_name) = member_expression.static_property_name() {
            if static_property_name == "__iterator__" {
                ctx.diagnostic(no_iterator_diagnostic(Span::new(
                    member_expression.span().start,
                    member_expression.span().end,
                )));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = test[__iterator__];",
        "var __iterator__ = null;",
        "foo[`__iterator`] = null;",
        "foo[`__iterator__
			`] = null;",
    ];

    let fail = vec![
        "var a = test.__iterator__;",
        "Foo.prototype.__iterator__ = function() {};",
        "var a = test['__iterator__'];",
        "var a = test[`__iterator__`];",
        "test[`__iterator__`] = function () {};",
    ];

    Tester::new(NoIterator::NAME, NoIterator::PLUGIN, pass, fail).test_and_snapshot();
}
