use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_proto_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The '__proto__' property is deprecated")
        .with_help("use `Object.getPrototypeOf` and `Object.setPrototypeOf` instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoProto;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of the `__proto__` property.
    ///
    /// ### Why is this bad?
    ///
    /// The `__proto__` property has been deprecated as of ECMAScript 3.1 and
    /// shouldn’t be used in new code. Use `Object.getPrototypeOf` and
    /// `Object.setPrototypeOf` instead.
    ///
    /// For more information, see
    /// [the MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/proto).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /*eslint no-proto: "error"*/
    ///
    /// var a = obj.__proto__;
    ///
    /// var a = obj["__proto__"];
    ///
    /// obj.__proto__ = b;
    ///
    /// obj["__proto__"] = b;
    /// ```
    NoProto,
    eslint,
    restriction,
    pending
);

impl Rule for NoProto {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(member_expr) = node.kind().as_member_expression_kind() else {
            return;
        };

        if member_expr.static_property_name().is_some_and(|name| name == "__proto__") {
            ctx.diagnostic(no_proto_diagnostic(member_expr.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = test[__proto__];",
        "var __proto__ = null;",
        "foo[`__proto`] = null;",
        "foo[`__proto__
			`] = null;",
        "class C { #__proto__; foo() { this.#__proto__; } }",
    ];

    let fail = vec![
        "var a = test.__proto__;",
        "var a = test['__proto__'];",
        "var a = test[`__proto__`];",
        "test[`__proto__`] = function () {};",
    ];

    Tester::new(NoProto::NAME, NoProto::PLUGIN, pass, fail).test_and_snapshot();
}
