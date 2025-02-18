use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashSet;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn id_denylist_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct IdDenylist(Box<IdDenylistConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows specified identifiers in assignments and function definitions.
    ///
    /// This rule will catch disallowed identifiers that are:
    ///
    ///  - variable declarations
    ///  - function declarations
    ///  - object properties assigned to during object creation
    ///  - class fields
    ///  - class methods
    ///
    /// It will not catch disallowed identifiers that are:
    ///
    ///  - function calls (so you can still use functions you do not have control over)
    ///  - object properties (so you can still use objects you do not have control over)
    ///
    /// ### Why is this bad?
    ///
    /// Generic names can lead to hard-to-decipher code. This rule allows you
    /// to specify a deny list of disallowed identifier names to avoid this
    /// practice.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    ///  /* id-denylist: ["error", "data", "callback"] */
    ///
    /// const data = { ...values };
    ///
    /// function callback() {
    ///     // ...
    /// }
    ///
    /// element.callback = function() {
    ///     // ...
    /// };
    ///
    /// const itemSet = {
    ///     data: [...values]
    /// };
    ///
    /// class Foo {
    ///     data = [];
    /// }
    ///
    /// class Bar {
    ///     #data = [];
    /// }
    ///
    /// class Baz {
    ///     callback() {}
    /// }
    ///
    /// class Qux {
    ///     #callback() {}
    /// }
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// /*eslint id-denylist: ["error", "data", "callback"] */
    ///
    /// const encodingOptions = {...values};
    ///
    /// function processFileResult() {
    ///     // ...
    /// }
    ///
    /// element.successHandler = function() {
    ///     // ...
    /// };
    ///
    /// const itemSet = {
    ///     entities: [...values]
    /// };
    ///
    /// callback(); // all function calls are ignored
    ///
    /// foo.callback(); // all function calls are ignored
    ///
    /// foo.data; // all property names that are not assignments are ignored
    ///
    /// class Foo {
    ///     items = [];
    /// }
    ///
    /// class Bar {
    ///     #items = [];
    /// }
    ///
    /// class Baz {
    ///     method() {}
    /// }
    ///
    /// class Qux {
    ///     #method() {}
    /// }
    IdDenylist,
    eslint,
    restriction,
);

#[derive(Debug, Default, Clone)]
pub struct IdDenylistConfig {
    id_denylist: FxHashSet<CompactStr>,
}

#[derive(PartialEq)]
enum IdDenylistResult {
    IdAllowed,
    IdDenied,
}

fn is_id_allowed(id: &CompactStr, deny_list: &FxHashSet<CompactStr>) -> IdDenylistResult {
    return if deny_list.contains(id) {
        IdDenylistResult::IdDenied
    } else {
        IdDenylistResult::IdAllowed
    };
}

fn check_expression_statement<'a>(expr: &'a Expression<'a>, id_denylist: &FxHashSet<CompactStr>) {
    match expr {
        Expression::FunctionExpression(fn_expr) => return,
        Expression::ArrowFunctionExpression(arrow_expr) => {
            return;
        }
        Expression::CallExpression(call_expr) => {
            return;
        }
        Expression::Identifier(ident) => {
            let res: IdDenylistResult = is_id_allowed(&ident.name.into_compact_str(), id_denylist);

            return;
        }
        Expression::AwaitExpression(expr) => {
            return;
        }
        _ => {
            return;
        }
    };

    return;
}

impl Rule for IdDenylist {
    fn from_configuration(value: serde_json::Value) -> Self {
        let options: Option<&serde_json::Value> = value.get(0);

        let id_denylist: FxHashSet<CompactStr> = options
            .and_then(|x| x.get("allowedNames"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(IdDenylistConfig { id_denylist }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExpressionStatement(expr) => {
                check_expression_statement(expr, &self.0.id_denylist);
                return;
            }
            _ => {
                return;
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"foo = "bar""#, Some(serde_json::json!(["bar"]))),
        (r#"bar = "bar""#, Some(serde_json::json!(["foo"]))),
        (r#"foo = "bar""#, Some(serde_json::json!(["f", "fo", "fooo", "bar"]))),
        ("function foo(){}", Some(serde_json::json!(["bar"]))),
        ("foo()", Some(serde_json::json!(["f", "fo", "fooo", "bar"]))),
        ("import { foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("foo.bar()", Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "baz"]))),
        (
            "var foo = bar.baz;",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz"])),
        ),
        (
            "var foo = bar.baz.bing;",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "foo.bar.baz = bing.bong.bash;",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "if (foo.bar) {}",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "var obj = { key: foo.bar };",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        ("const {foo: bar} = baz", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6 },
        ("const {foo: {bar: baz}} = qux", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("function foo({ bar: baz }) {}", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("function foo({ bar: {baz: qux} }) {}", Some(serde_json::json!(["bar", "baz"]))), // { "ecmaVersion": 6 },
        ("function foo({baz} = obj.qux) {}", Some(serde_json::json!(["qux"]))), // { "ecmaVersion": 6 },
        ("function foo({ foo: {baz} = obj.qux }) {}", Some(serde_json::json!(["qux"]))), // { "ecmaVersion": 6 },
        ("({a: bar = obj.baz});", Some(serde_json::json!(["baz"]))), // { "ecmaVersion": 6 },
        ("({foo: {a: bar = obj.baz}} = qux);", Some(serde_json::json!(["baz"]))), // { "ecmaVersion": 6 },
        (
            "var arr = [foo.bar];",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "[foo.bar]",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "[foo.bar.nesting]",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "if (foo.bar === bar.baz) { [foo.bar] }",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "var myArray = new Array(); var myDate = new Date();",
            Some(serde_json::json!(["array", "date", "mydate", "myarray", "new", "var"])),
        ),
        ("foo()", Some(serde_json::json!(["foo"]))),
        ("foo.bar()", Some(serde_json::json!(["bar"]))),
        ("foo.bar", Some(serde_json::json!(["bar"]))),
        ("({foo: obj.bar.bar.bar.baz} = {});", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("({[obj.bar]: a = baz} = qux);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("Number.parseInt()", Some(serde_json::json!(["Number"]))),
        ("x = Number.NaN;", Some(serde_json::json!(["Number"]))),
        ("var foo = undefined;", Some(serde_json::json!(["undefined"]))),
        ("if (foo === undefined);", Some(serde_json::json!(["undefined"]))),
        ("obj[undefined] = 5;", Some(serde_json::json!(["undefined"]))),
        ("foo = { [myGlobal]: 1 };", Some(serde_json::json!(["myGlobal"]))), // {                "ecmaVersion": 6,                "globals": { "myGlobal": "readonly" }            },
        ("({ myGlobal } = foo);", Some(serde_json::json!(["myGlobal"]))), // {                "ecmaVersion": 6,                "globals": { "myGlobal": "writable" }            },
        ("/* global myGlobal: readonly */ myGlobal = 5;", Some(serde_json::json!(["myGlobal"]))),
        ("var foo = [Map];", Some(serde_json::json!(["Map"]))), // {                "ecmaVersion": 6            },
        ("var foo = { bar: window.baz };", Some(serde_json::json!(["window"]))), // {                "globals": {                    "window": "readonly"                }            },
        ("class C { camelCase; #camelCase; #camelCase2() {} }", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 2022 },
        (
            "class C { snake_case; #snake_case; #snake_case2() {} }",
            Some(serde_json::json!(["foo"])),
        ), // { "ecmaVersion": 2022 },
        ("import foo from 'foo.json' with { type: 'json' }", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025, "sourceType": "module" },
        ("export * from 'foo.json' with { type: 'json' }", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025, "sourceType": "module" },
        (
            "export { default } from 'foo.json' with { type: 'json' }",
            Some(serde_json::json!(["type"])),
        ), // { "ecmaVersion": 2025, "sourceType": "module" },
        (
            "import('foo.json', { with: { type: 'json' } })",
            Some(serde_json::json!(["with", "type"])),
        ), // { "ecmaVersion": 2025 },
        ("import('foo.json', { 'with': { type: 'json' } })", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025 },
        ("import('foo.json', { with: { type } })", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025 }
    ];

    let fail = vec![
        (r#"foo = "bar""#, Some(serde_json::json!(["foo"]))),
        (r#"bar = "bar""#, Some(serde_json::json!(["bar"]))),
        (r#"foo = "bar""#, Some(serde_json::json!(["f", "fo", "foo", "bar"]))),
        ("function foo(){}", Some(serde_json::json!(["f", "fo", "foo", "bar"]))),
        ("import foo from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import * as foo from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export * as foo from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 2020, "sourceType": "module" },
        ("import { foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo as bar } from 'mod'", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo as bar } from 'mod'", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo as foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo, foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo as bar, foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import foo, { foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo as bar };", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo };", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo as bar };", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo as foo };", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo as bar };", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as bar } from 'mod'", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as bar } from 'mod'", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo, foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as bar, foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("foo.bar()", Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "baz"]))),
        ("foo[bar] = baz;", Some(serde_json::json!(["bar"]))),
        ("baz = foo[bar];", Some(serde_json::json!(["bar"]))),
        (
            "var foo = bar.baz;",
            Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "barr", "bazz"])),
        ),
        (
            "var foo = bar.baz;",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "bar", "bazz"])),
        ),
        (
            "if (foo.bar) {}",
            Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        ("var obj = { key: foo.bar };", Some(serde_json::json!(["obj"]))),
        ("var obj = { key: foo.bar };", Some(serde_json::json!(["key"]))),
        ("var obj = { key: foo.bar };", Some(serde_json::json!(["foo"]))),
        ("var arr = [foo.bar];", Some(serde_json::json!(["arr"]))),
        ("var arr = [foo.bar];", Some(serde_json::json!(["foo"]))),
        (
            "[foo.bar]",
            Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "if (foo.bar === bar.baz) { [bing.baz] }",
            Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "if (foo.bar === bar.baz) { [foo.bar] }",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "bar", "bazz", "bingg"])),
        ),
        (
            "var myArray = new Array(); var myDate = new Date();",
            Some(serde_json::json!(["array", "date", "myDate", "myarray", "new", "var"])),
        ),
        (
            "var myArray = new Array(); var myDate = new Date();",
            Some(serde_json::json!(["array", "date", "mydate", "myArray", "new", "var"])),
        ),
        ("foo.bar = 1", Some(serde_json::json!(["bar"]))),
        ("foo.bar.baz = 1", Some(serde_json::json!(["bar", "baz"]))),
        ("const {foo} = baz", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6 },
        ("const {foo: bar} = baz", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("const {[foo]: bar} = baz", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("const {foo: {bar: baz}} = qux", Some(serde_json::json!(["foo", "bar", "baz"]))), // { "ecmaVersion": 6 },
        ("const {foo: {[bar]: baz}} = qux", Some(serde_json::json!(["foo", "bar", "baz"]))), // { "ecmaVersion": 6 },
        ("const {[foo]: {[bar]: baz}} = qux", Some(serde_json::json!(["foo", "bar", "baz"]))), // { "ecmaVersion": 6 },
        ("function foo({ bar: baz }) {}", Some(serde_json::json!(["bar", "baz"]))), // { "ecmaVersion": 6 },
        ("function foo({ bar: {baz: qux} }) {}", Some(serde_json::json!(["bar", "baz", "qux"]))), // { "ecmaVersion": 6 },
        ("({foo: obj.bar} = baz);", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("({foo: obj.bar.bar.bar.baz} = {});", Some(serde_json::json!(["foo", "bar", "baz"]))), // { "ecmaVersion": 6 },
        ("({[foo]: obj.bar} = baz);", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("({foo: { a: obj.bar }} = baz);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("({a: obj.bar = baz} = qux);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        (
            "({a: obj.bar.bar.baz = obj.qux} = obj.qux);",
            Some(serde_json::json!(["a", "bar", "baz", "qux"])),
        ), // { "ecmaVersion": 6 },
        (
            "({a: obj[bar] = obj.qux} = obj.qux);",
            Some(serde_json::json!(["a", "bar", "baz", "qux"])),
        ), // { "ecmaVersion": 6 },
        ("({a: [obj.bar] = baz} = qux);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("({foo: { a: obj.bar = baz}} = qux);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("({foo: { [a]: obj.bar }} = baz);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("({...obj.bar} = baz);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 9 },
        ("([obj.bar] = baz);", Some(serde_json::json!(["bar"]))),    // { "ecmaVersion": 6 },
        ("const [bar] = baz;", Some(serde_json::json!(["bar"]))),    // { "ecmaVersion": 6 },
        ("foo.undefined = 1;", Some(serde_json::json!(["undefined"]))),
        ("var foo = { undefined: 1 };", Some(serde_json::json!(["undefined"]))),
        ("var foo = { undefined: undefined };", Some(serde_json::json!(["undefined"]))),
        ("var foo = { Number() {} };", Some(serde_json::json!(["Number"]))), // { "ecmaVersion": 6 },
        ("class Foo { Number() {} }", Some(serde_json::json!(["Number"]))), // { "ecmaVersion": 6 },
        ("myGlobal: while(foo) { break myGlobal; } ", Some(serde_json::json!(["myGlobal"]))), // {                "globals": { "myGlobal": "readonly" }            },
        ("const foo = 1; bar = foo;", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6 },
        ("let foo; foo = bar;", Some(serde_json::json!(["foo"]))),       // { "ecmaVersion": 6 },
        ("bar = foo; var foo;", Some(serde_json::json!(["foo"]))),
        ("function foo() {} var bar = foo;", Some(serde_json::json!(["foo"]))),
        ("class Foo {} var bar = Foo;", Some(serde_json::json!(["Foo"]))), // { "ecmaVersion": 6 },
        ("let undefined; undefined = 1;", Some(serde_json::json!(["undefined"]))), // { "ecmaVersion": 6 },
        ("foo = undefined; var undefined;", Some(serde_json::json!(["undefined"]))),
        ("function undefined(){} x = undefined;", Some(serde_json::json!(["undefined"]))),
        ("class Number {} x = Number.NaN;", Some(serde_json::json!(["Number"]))), // { "ecmaVersion": 6 },
        (
            "/* globals myGlobal */ window.myGlobal = 5; foo = myGlobal;",
            Some(serde_json::json!(["myGlobal"])),
        ), // {                "globals": {                    "window": "readonly"                }            },
        ("var foo = undefined;", Some(serde_json::json!(["undefined"]))), // {                "globals": { "undefined": "off" }            },
        ("/* globals Number: off */ Number.parseInt()", Some(serde_json::json!(["Number"]))),
        ("var foo = [Map];", Some(serde_json::json!(["Map"]))),
        ("if (foo) { let undefined; bar = undefined; }", Some(serde_json::json!(["undefined"]))), // { "ecmaVersion": 6 },
        ("function foo(Number) { var x = Number.NaN; }", Some(serde_json::json!(["Number"]))),
        ("function foo() { var myGlobal; x = myGlobal; }", Some(serde_json::json!(["myGlobal"]))), // {                "globals": { "myGlobal": "readonly" }            },
        (
            "function foo(bar) { return Number.parseInt(bar); } const Number = 1;",
            Some(serde_json::json!(["Number"])),
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "import Number from 'myNumber'; const foo = Number.parseInt(bar);",
            Some(serde_json::json!(["Number"])),
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo = function undefined() {};", Some(serde_json::json!(["undefined"]))),
        ("var foo = { undefined }", Some(serde_json::json!(["undefined"]))), // { "ecmaVersion": 6 },
        (
            "class C { camelCase; #camelCase; #camelCase2() {} }",
            Some(serde_json::json!(["camelCase"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { snake_case; #snake_case() {}; #snake_case2() {} }",
            Some(serde_json::json!(["snake_case"])),
        ), // { "ecmaVersion": 2022 },
        ("import('foo.json', { with: { [type]: 'json' } })", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025 },
        ("import('foo.json', { with: { type: json } })", Some(serde_json::json!(["json"]))), // { "ecmaVersion": 2025 }
    ];

    Tester::new(IdDenylist::NAME, IdDenylist::PLUGIN, pass, fail).test_and_snapshot();
}
