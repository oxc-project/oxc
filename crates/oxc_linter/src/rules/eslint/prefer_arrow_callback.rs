use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn prefer_arrow_callback_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrowCallback;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
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
    PreferArrowCallback,
    eslint,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for PreferArrowCallback {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo(a => a);", None),
        ("foo(function*() {});", None),
        ("foo(function() { this; });", None),
        ("foo(function bar() {});", Some(serde_json::json!([{ "allowNamedFunctions": true }]))),
        ("foo(function() { (() => this); });", None),
        ("foo(function() { this; }.bind(obj));", None),
        ("foo(function() { this; }.call(this));", None),
        ("foo(a => { (function() {}); });", None),
        ("var foo = function foo() {};", None),
        ("(function foo() {})();", None),
        ("foo(function bar() { bar; });", None),
        ("foo(function bar() { arguments; });", None),
        ("foo(function bar() { arguments; }.bind(this));", None),
        ("foo(function bar() { new.target; });", None),
        ("foo(function bar() { new.target; }.bind(this));", None),
        ("foo(function bar() { this; }.bind(this, somethingElse));", None),
        ("foo((function() {}).bind.bar)", None),
        ("foo((function() { this.bar(); }).bind(obj).bind(this))", None),
    ];

    let fail = vec![
        ("foo(function bar() {});", None),
("foo(function() {});", Some(serde_json::json!([{ "allowNamedFunctions": true }]))),
("foo(function bar() {});", Some(serde_json::json!([{ "allowNamedFunctions": false }]))),
("foo(function() {});", None),
("foo(nativeCb || function() {});", None),
("foo(bar ? function() {} : function() {});", None),
("foo(function() { (function() { this; }); });", None),
("foo(function() { this; }.bind(this));", None),
("foo(bar || function() { this; }.bind(this));", None),
("foo(function() { (() => this); }.bind(this));", None),
("foo(function bar(a) { a; });", None),
("foo(function(a) { a; });", None),
("foo(function(arguments) { arguments; });", None),
("foo(function() { this; });", Some(serde_json::json!([{ "allowUnboundThis": false }]))),
("foo(function() { (() => this); });", Some(serde_json::json!([{ "allowUnboundThis": false }]))),
("qux(function(foo, bar, baz) { return foo * 2; })", None),
("qux(function(foo, bar, baz) { return foo * bar; }.bind(this))", None),
("qux(function(foo, bar, baz) { return foo * this.qux; }.bind(this))", None),
("foo(function() {}.bind(this, somethingElse))", None),
("qux(function(foo = 1, [bar = 2] = [], {qux: baz = 3} = {foo: 'bar'}) { return foo + bar; });", None),
("qux(function(baz, baz) { })", None),
("qux(function( /* no params */ ) { })", None),
("qux(function( /* a */ foo /* b */ , /* c */ bar /* d */ , /* e */ baz /* f */ ) { return foo; })", None),
("qux(async function (foo = 1, bar = 2, baz = 3) { return baz; })", None),
("qux(async function (foo = 1, bar = 2, baz = 3) { return this; }.bind(this))", None),
("foo((bar || function() {}).bind(this))", None),
("foo(function() {}.bind(this).bind(obj))", None),
("foo?.(function() {});", None),
("foo?.(function() { return this; }.bind(this));", None),
("foo(function() { return this; }?.bind(this));", None),
("foo((function() { return this; }?.bind)(this));", None),
("
			            test(
			                function ()
			                { }
			            );
			            ", None),
("
			            test(
			                function (
			                    ...args
			                ) /* Lorem ipsum
			                dolor sit amet. */ {
			                    return args;
			                }
			            );
			            ", None)
    ];

    let fix = vec![
        ("foo(function bar() {});", "foo(() => {});", None),
("foo(function() {});", "foo(() => {});", Some(serde_json::json!([{ "allowNamedFunctions": true }]))),
("foo(function bar() {});", "foo(() => {});", Some(serde_json::json!([{ "allowNamedFunctions": false }]))),
("foo(function() {});", "foo(() => {});", None),
("foo(nativeCb || function() {});", "foo(nativeCb || (() => {}));", None),
("foo(bar ? function() {} : function() {});", "foo(bar ? () => {} : () => {});", None),
("foo(function() { (function() { this; }); });", "foo(() => { (function() { this; }); });", None),
("foo(function() { this; }.bind(this));", "foo(() => { this; });", None),
("foo(bar || function() { this; }.bind(this));", "foo(bar || (() => { this; }));", None),
("foo(function() { (() => this); }.bind(this));", "foo(() => { (() => this); });", None),
("foo(function bar(a) { a; });", "foo((a) => { a; });", None),
("foo(function(a) { a; });", "foo((a) => { a; });", None),
("foo(function(arguments) { arguments; });", "foo((arguments) => { arguments; });", None),
("qux(function(foo, bar, baz) { return foo * 2; })", "qux((foo, bar, baz) => { return foo * 2; })", None),
("qux(function(foo, bar, baz) { return foo * bar; }.bind(this))", "qux((foo, bar, baz) => { return foo * bar; })", None),
("qux(function(foo, bar, baz) { return foo * this.qux; }.bind(this))", "qux((foo, bar, baz) => { return foo * this.qux; })", None),
("foo(function() {}.bind(this, somethingElse))", "foo((() => {}).bind(this, somethingElse))", None),
("qux(function(foo = 1, [bar = 2] = [], {qux: baz = 3} = {foo: 'bar'}) { return foo + bar; });", "qux((foo = 1, [bar = 2] = [], {qux: baz = 3} = {foo: 'bar'}) => { return foo + bar; });", None),
("qux(function( /* no params */ ) { })", "qux(( /* no params */ ) => { })", None),
("qux(function( /* a */ foo /* b */ , /* c */ bar /* d */ , /* e */ baz /* f */ ) { return foo; })", "qux(( /* a */ foo /* b */ , /* c */ bar /* d */ , /* e */ baz /* f */ ) => { return foo; })", None),
("qux(async function (foo = 1, bar = 2, baz = 3) { return baz; })", "qux(async (foo = 1, bar = 2, baz = 3) => { return baz; })", None),
("qux(async function (foo = 1, bar = 2, baz = 3) { return this; }.bind(this))", "qux(async (foo = 1, bar = 2, baz = 3) => { return this; })", None),
("foo(function() {}.bind(this).bind(obj))", "foo((() => {}).bind(obj))", None),
("foo?.(function() {});", "foo?.(() => {});", None),
("foo?.(function() { return this; }.bind(this));", "foo?.(() => { return this; });", None),
("foo(function() { return this; }?.bind(this));", "foo(() => { return this; });", None),
("
			            test(
			                function ()
			                { }
			            );
			            ", "
			            test(
			                () =>
			                { }
			            );
			            ", None),
("
			            test(
			                function (
			                    ...args
			                ) /* Lorem ipsum
			                dolor sit amet. */ {
			                    return args;
			                }
			            );
			            ", "
			            test(
			                (
			                    ...args
			                ) => /* Lorem ipsum
			                dolor sit amet. */ {
			                    return args;
			                }
			            );
			            ", None)
    ];
    Tester::new(PreferArrowCallback::NAME, PreferArrowCallback::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
