use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoUndefined;

fn no_undefined_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected use of `undefined`").with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow the use of `undefined` as an identifier
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example of bad code
    /// ```javascript
    ///
    /// var foo = undefined;
    ///
    /// var undefined = "foo";
    ///
    /// if (foo === undefined) {
    ///     // ...
    /// }
    ///
    /// function baz(undefined) {
    ///     // ...
    /// }
    ///
    /// bar(undefined, "lorem");
    /// ```
    ///
    /// ### Example of good code
    /// ```javascript
    /// var foo = void 0;
    ///
    /// var Undefined = "foo";
    ///
    /// if (typeof foo === "undefined") {
    ///     // ...
    /// }
    ///
    /// global.undefined = "foo";
    ///
    /// bar(void 0, "lorem");
    /// ```
    ///
    NoUndefined,
    eslint,
    restriction,
);

fn diagnostic_undefined_keyword(name: &str, span: Span, ctx: &LintContext) {
    if name == "undefined" {
        ctx.diagnostic(no_undefined_diagnostic(span));
    }
}

impl Rule for NoUndefined {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IdentifierReference(ident) => {
                diagnostic_undefined_keyword(ident.name.as_str(), ident.span, ctx);
            }
            AstKind::BindingIdentifier(ident) => {
                diagnostic_undefined_keyword(ident.name.as_str(), ident.span, ctx);
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "void 0",
        "void!0",
        "void-0",
        "void+0",
        "null",
        "undefine",
        "a.undefined",
        "this.undefined",
        "global['undefined']",
        "({ undefined: bar })",
        "({ undefined: bar } = foo)",
        "({ undefined() {} })",
        "class Foo { undefined() {} }",
        "(class { undefined() {} })",
        "import { undefined as a } from 'foo'", // ES6_MODULE,
        "export { undefined } from 'foo'",      // ES6_MODULE,
        "export { undefined as a } from 'foo'", // ES6_MODULE,
        "export { a as undefined } from 'foo'", // ES6_MODULE
    ];

    let fail = vec![
        "undefined",
        "undefined.a",
        "a[undefined]",
        "undefined[0]",
        "f(undefined)",
        "function f(undefined) {}",
        "function f() { var undefined; }",
        "function f() { undefined = true; }",
        "var undefined;",
        "try {} catch(undefined) {}",
        "function undefined() {}",
        "(function undefined(){}())",
        "var foo = function undefined() {}",
        "foo = function undefined() {}",
        "undefined = true",
        "var undefined = true",
        "({ undefined })",
        "({ [undefined]: foo })",
        "({ bar: undefined })",
        "({ bar: undefined } = foo)",
        "var { undefined } = foo",
        "var { bar: undefined } = foo",
        "({ undefined: function undefined() {} })",
        "({ foo: function undefined() {} })",
        "class Foo { [undefined]() {} }",
        "(class { [undefined]() {} })",
        "var undefined = true; undefined = false;",
        "import undefined from 'foo'",          // ES6_MODULE,
        "import * as undefined from 'foo'",     // ES6_MODULE,
        "import { undefined } from 'foo'",      // ES6_MODULE,
        "import { a as undefined } from 'foo'", // ES6_MODULE,
        "let a = [b, ...undefined]",
        "[a, ...undefined] = b",
        "[a = undefined] = b",
    ];

    Tester::new(NoUndefined::NAME, NoUndefined::PLUGIN, pass, fail).test_and_snapshot();
}
