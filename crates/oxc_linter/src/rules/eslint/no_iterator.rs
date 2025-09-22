use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_iterator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Reserved name '__iterator__'")
        .with_help("Consider using [Symbol.iterator] instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoIterator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of the `__iterator__` property
    ///
    /// ### Why is this bad?
    ///
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
    /// ### Examples
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
    suggestion
);

impl Rule for NoIterator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(member_expression) = node.kind().as_member_expression_kind() else {
            return;
        };
        if let Some(static_property_name) = member_expression.static_property_name()
            && static_property_name == "__iterator__"
        {
            let mem_span = member_expression.span();
            let obj_span = member_expression.object().span();
            ctx.diagnostic_with_suggestion(no_iterator_diagnostic(mem_span), |fixer| {
                fixer.replace(Span::new(obj_span.end, mem_span.end), "[Symbol.iterator]")
            });
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
        "foo[`__iterator__\n`] = null;",
    ];

    let fail = vec![
        "var a = test.__iterator__;",
        "Foo.prototype.__iterator__ = function() {};",
        "var a = test['__iterator__'];",
        "var a = test[`__iterator__`];",
        "test[`__iterator__`] = function () {};",
    ];

    let fix = vec![
        ("var a = test.__iterator__;", "var a = test[Symbol.iterator];"),
        (
            "Foo.prototype.__iterator__ = function() {};",
            "Foo.prototype[Symbol.iterator] = function() {};",
        ),
        ("var a = test['__iterator__'];", "var a = test[Symbol.iterator];"),
        ("var a = test[`__iterator__`];", "var a = test[Symbol.iterator];"),
        ("test[`__iterator__`] = function () {};", "test[Symbol.iterator] = function () {};"),
    ];

    Tester::new(NoIterator::NAME, NoIterator::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
