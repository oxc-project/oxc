use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn camelcase_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct Camelcase;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
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
    Camelcase,
    eslint,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for Camelcase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"firstName = "Nicholas""#, None),
        (r#"FIRST_NAME = "Nicholas""#, None),
        (r#"__myPrivateVariable = "Patrick""#, None),
        (r#"myPrivateVariable_ = "Patrick""#, None),
        ("function doSomething(){}", None),
        ("do_something()", None),
        ("new do_something", None),
        ("new do_something()", None),
        ("foo.do_something()", None),
        ("var foo = bar.baz_boom;", None),
        ("var foo = bar.baz_boom.something;", None),
        ("foo.boom_pow.qux = bar.baz_boom.something;", None),
        ("if (bar.baz_boom) {}", None),
        ("var obj = { key: foo.bar_baz };", None),
        ("var arr = [foo.bar_baz];", None),
        ("[foo.bar_baz]", None),
        ("var arr = [foo.bar_baz.qux];", None),
        ("[foo.bar_baz.nesting]", None),
        ("if (foo.bar_baz === boom.bam_pow) { [foo.baz_boom] }", None),
        ("var o = {key: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var o = {_leading: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var o = {trailing_: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var o = {bar_baz: 1}", Some(serde_json::json!([{ "properties": "never" }]))),
        ("var o = {_leading: 1}", Some(serde_json::json!([{ "properties": "never" }]))),
        ("var o = {trailing_: 1}", Some(serde_json::json!([{ "properties": "never" }]))),
        ("obj.a_b = 2;", Some(serde_json::json!([{ "properties": "never" }]))),
        ("obj._a = 2;", Some(serde_json::json!([{ "properties": "always" }]))),
        ("obj.a_ = 2;", Some(serde_json::json!([{ "properties": "always" }]))),
        ("obj._a = 2;", Some(serde_json::json!([{ "properties": "never" }]))),
        ("obj.a_ = 2;", Some(serde_json::json!([{ "properties": "never" }]))),
        (
            "var obj = {
			 a_a: 1 
			};
			 obj.a_b = 2;",
            Some(serde_json::json!([{ "properties": "never" }])),
        ),
        ("obj.foo_bar = function(){};", Some(serde_json::json!([{ "properties": "never" }]))),
        ("const { ['foo']: _foo } = obj;", None), // { "ecmaVersion": 6 },
        ("const { [_foo_]: foo } = obj;", None),  // { "ecmaVersion": 6 },
        (
            "var { category_id } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var { category_id: category_id } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var { category_id = 1 } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var { [{category_id} = query]: categoryId } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 6 },
        ("var { category_id: category } = query;", None), // { "ecmaVersion": 6 },
        ("var { _leading } = query;", None),      // { "ecmaVersion": 6 },
        ("var { trailing_ } = query;", None),     // { "ecmaVersion": 6 },
        (r#"import { camelCased } from "external module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import { _leading } from "external module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import { trailing_ } from "external module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import { no_camelcased as camelCased } from "external-module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import { no_camelcased as _leading } from "external-module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import { no_camelcased as trailing_ } from "external-module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            r#"import { no_camelcased as camelCased, anotherCamelCased } from "external-module";"#,
            None,
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { snake_cased } from 'mod'", Some(serde_json::json!([{ "ignoreImports": true }]))), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "import { snake_cased as snake_cased } from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ), // { "ecmaVersion": 2022, "sourceType": "module" },
        (
            "import { 'snake_cased' as snake_cased } from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ), // { "ecmaVersion": 2022, "sourceType": "module" },
        ("import { camelCased } from 'mod'", Some(serde_json::json!([{ "ignoreImports": false }]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { a as 'snake_cased' } from 'mod'", None), // { "ecmaVersion": 2022, "sourceType": "module" },
        ("export * as 'snake_cased' from 'mod'", None), // { "ecmaVersion": 2022, "sourceType": "module" },
        (
            "var _camelCased = aGlobalVariable",
            Some(serde_json::json!([{ "ignoreGlobals": false }])),
        ), // { "globals": { "aGlobalVariable": "readonly" } },
        (
            "var camelCased = _aGlobalVariable",
            Some(serde_json::json!([{ "ignoreGlobals": false }])),
        ), // { "globals": { _"aGlobalVariable": "readonly" } },
        (
            "var camelCased = a_global_variable",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // { "globals": { "a_global_variable": "readonly" } },
        ("a_global_variable.foo()", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "readonly" } },
        ("a_global_variable[undefined]", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "readonly" } },
        ("var foo = a_global_variable.bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "readonly" } },
        ("a_global_variable.foo = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "readonly" } },
        (
            "( { foo: a_global_variable.bar } = baz )",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "readonly",				},			},
        ("a_global_variable = foo", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "writable" } },
        ("a_global_variable = foo", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "readonly" } },
        ("({ a_global_variable } = foo)", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        (
            "({ snake_cased: a_global_variable } = foo)",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        (
            "({ snake_cased: a_global_variable = foo } = bar)",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        ("[a_global_variable] = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        ("[a_global_variable = foo] = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        ("foo[a_global_variable] = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "readonly",				},			},
        (
            "var foo = { [a_global_variable]: bar }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "readonly",				},			},
        (
            "var { [a_global_variable]: foo } = bar",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "readonly",				},			},
        ("function foo({ no_camelcased: camelCased }) {};", None), // { "ecmaVersion": 6 },
        ("function foo({ no_camelcased: _leading }) {};", None),   // { "ecmaVersion": 6 },
        ("function foo({ no_camelcased: trailing_ }) {};", None),  // { "ecmaVersion": 6 },
        ("function foo({ camelCased = 'default value' }) {};", None), // { "ecmaVersion": 6 },
        ("function foo({ _leading = 'default value' }) {};", None), // { "ecmaVersion": 6 },
        ("function foo({ trailing_ = 'default value' }) {};", None), // { "ecmaVersion": 6 },
        ("function foo({ camelCased }) {};", None),                // { "ecmaVersion": 6 },
        ("function foo({ _leading }) {}", None),                   // { "ecmaVersion": 6 },
        ("function foo({ trailing_ }) {}", None),                  // { "ecmaVersion": 6 },
        ("ignored_foo = 0;", Some(serde_json::json!([{ "allow": ["ignored_foo"] }]))),
        (
            "ignored_foo = 0; ignored_bar = 1;",
            Some(serde_json::json!([{ "allow": ["ignored_foo", "ignored_bar"] }])),
        ),
        ("user_id = 0;", Some(serde_json::json!([{ "allow": ["_id$"] }]))),
        ("__option_foo__ = 0;", Some(serde_json::json!([{ "allow": ["__option_foo__"] }]))),
        (
            "__option_foo__ = 0; user_id = 0; foo = 1",
            Some(serde_json::json!([{ "allow": ["__option_foo__", "_id$"] }])),
        ),
        ("fo_o = 0;", Some(serde_json::json!([{ "allow": ["__option_foo__", "fo_o"] }]))),
        ("user = 0;", Some(serde_json::json!([{ "allow": [] }]))),
        ("foo = { [computedBar]: 0 };", Some(serde_json::json!([{ "ignoreDestructuring": true }]))), // { "ecmaVersion": 6 },
        ("({ a: obj.fo_o } = bar);", Some(serde_json::json!([{ "allow": ["fo_o"] }]))), // { "ecmaVersion": 6 },
        ("({ a: obj.foo } = bar);", Some(serde_json::json!([{ "allow": ["fo_o"] }]))), // { "ecmaVersion": 6 },
        ("({ a: obj.fo_o } = bar);", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("({ a: obj.fo_o.b_ar } = bar);", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("({ a: { b: obj.fo_o } } = bar);", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("([obj.fo_o] = bar);", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("({ c: [ob.fo_o]} = bar);", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("([obj.fo_o.b_ar] = bar);", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("({obj} = baz.fo_o);", None), // { "ecmaVersion": 6 },
        ("([obj] = baz.fo_o);", None), // { "ecmaVersion": 6 },
        ("([obj.foo = obj.fo_o] = bar);", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 6 },
        (
            "class C { camelCase; #camelCase; #camelCase2() {} }",
            Some(serde_json::json!([{ "properties": "always" }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { snake_case; #snake_case; #snake_case2() {} }",
            Some(serde_json::json!([{ "properties": "never" }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
			            const { some_property } = obj;
			
			            const bar = { some_property };
			
			            obj.some_property = 10;
			
			            const xyz = { some_property: obj.some_property };
			
			            const foo = ({ some_property }) => {
			                console.log(some_property)
			            };
			            ",
            Some(serde_json::json!([{ "properties": "never", "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
			            const { some_property } = obj;
			            doSomething({ some_property });
			            ",
            Some(serde_json::json!([{ "properties": "never", "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "import foo from 'foo.json' with { my_type: 'json' }",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // { "ecmaVersion": 2025, "sourceType": "module" },
        (
            "export * from 'foo.json' with { my_type: 'json' }",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // { "ecmaVersion": 2025, "sourceType": "module" },
        (
            "export { default } from 'foo.json' with { my_type: 'json' }",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // { "ecmaVersion": 2025, "sourceType": "module" },
        (
            "import('foo.json', { my_with: { my_type: 'json' } })",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // { "ecmaVersion": 2025 },
        (
            "import('foo.json', { 'with': { my_type: 'json' } })",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // { "ecmaVersion": 2025 },
        (
            "import('foo.json', { my_with: { my_type } })",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // { "ecmaVersion": 2025 },
        (
            "import('foo.json', { my_with: { my_type } })",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // {				"ecmaVersion": 2025,				"globals": {					"my_type": true, // eslint-disable-line camelcase -- for testing				},			}
    ];

    let fail = vec![
        (r#"first_name = "Nicholas""#, None),
        (r#"__private_first_name = "Patrick""#, None),
        ("function foo_bar(){}", None),
        ("obj.foo_bar = function(){};", None),
        ("bar_baz.foo = function(){};", None),
        ("[foo_bar.baz]", None),
        ("if (foo.bar_baz === boom.bam_pow) { [foo_bar.baz] }", None),
        ("foo.bar_baz = boom.bam_pow", None),
        ("var foo = { bar_baz: boom.bam_pow }", None),
        (
            "var foo = { bar_baz: boom.bam_pow }",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("foo.qux.boom_pow = { bar: boom.bam_pow }", None),
        ("var o = {bar_baz: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("obj.a_b = 2;", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var { category_id: category_alias } = query;", None), // { "ecmaVersion": 6 },
        (
            "var { category_id: category_alias } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var { [category_id]: categoryId } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 6 },
        ("var { [category_id]: categoryId } = query;", None),   // { "ecmaVersion": 6 },
        (
            "var { category_id: categoryId, ...other_props } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 2018 },
        ("var { category_id } = query;", None),                 // { "ecmaVersion": 6 },
        ("var { category_id: category_id } = query;", None),    // { "ecmaVersion": 6 },
        ("var { category_id = 1 } = query;", None),             // { "ecmaVersion": 6 },
        (r#"import no_camelcased from "external-module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import * as no_camelcased from "external-module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import { no_camelcased } from "external-module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import { no_camelcased as no_camel_cased } from "external module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import { camelCased as no_camel_cased } from "external module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { 'snake_cased' as snake_cased } from 'mod'", None), // { "ecmaVersion": 2022, "sourceType": "module" },
        (
            "import { 'snake_cased' as another_snake_cased } from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ), // { "ecmaVersion": 2022, "sourceType": "module" },
        (r#"import { camelCased, no_camelcased } from "external-module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            r#"import { no_camelcased as camelCased, another_no_camelcased } from "external-module";"#,
            None,
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        (r#"import camelCased, { no_camelcased } from "external-module";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            r#"import no_camelcased, { another_no_camelcased as camelCased } from "external-module";"#,
            None,
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import snake_cased from 'mod'", Some(serde_json::json!([{ "ignoreImports": true }]))), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "import * as snake_cased from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import snake_cased from 'mod'", Some(serde_json::json!([{ "ignoreImports": false }]))), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "import * as snake_cased from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": false }])),
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var camelCased = snake_cased", Some(serde_json::json!([{ "ignoreGlobals": false }]))), // { "globals": { "snake_cased": "readonly" } },
        ("a_global_variable.foo()", Some(serde_json::json!([{ "ignoreGlobals": false }]))), // { "globals": { "snake_cased": "readonly" } },
        ("a_global_variable[undefined]", Some(serde_json::json!([{ "ignoreGlobals": false }]))), // { "globals": { "snake_cased": "readonly" } },
        ("var camelCased = snake_cased", None), // { "globals": { "snake_cased": "readonly" } },
        ("var camelCased = snake_cased", Some(serde_json::json!([{}]))), // { "globals": { "snake_cased": "readonly" } },
        ("foo.a_global_variable = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "writable" } },
        (
            "var foo = { a_global_variable: bar }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // { "globals": { "a_global_variable": "writable" } },
        (
            "var foo = { a_global_variable: a_global_variable }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // { "globals": { "a_global_variable": "writable" } },
        (
            "var foo = { a_global_variable() {} }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        (
            "class Foo { a_global_variable() {} }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        ("a_global_variable: for (;;);", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        (
            "if (foo) { let a_global_variable; a_global_variable = bar; }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        (
            "function foo(a_global_variable) { foo = a_global_variable; }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        ("var a_global_variable", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "ecmaVersion": 6 },
        ("function a_global_variable () {}", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "ecmaVersion": 6 },
        (
            "const a_global_variable = foo; bar = a_global_variable",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        (
            "bar = a_global_variable; var a_global_variable;",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "writable",				},			},
        ("var foo = { a_global_variable }", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"ecmaVersion": 6,				"globals": {					// eslint-disable-next-line camelcase -- Testing non-CamelCase					"a_global_variable": "readonly",				},			},
        ("undefined_variable;", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        ("implicit_global = 1;", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        ("export * as snake_cased from 'mod'", None), // { "ecmaVersion": 2020, "sourceType": "module" },
        ("function foo({ no_camelcased }) {};", None), // { "ecmaVersion": 6 },
        ("function foo({ no_camelcased = 'default value' }) {};", None), // { "ecmaVersion": 6 },
        ("const no_camelcased = 0; function foo({ camelcased_value = no_camelcased}) {}", None), // { "ecmaVersion": 6 },
        ("const { bar: no_camelcased } = foo;", None), // { "ecmaVersion": 6 },
        ("function foo({ value_1: my_default }) {}", None), // { "ecmaVersion": 6 },
        ("function foo({ isCamelcased: no_camelcased }) {};", None), // { "ecmaVersion": 6 },
        ("var { foo: bar_baz = 1 } = quz;", None),     // { "ecmaVersion": 6 },
        ("const { no_camelcased = false } = bar;", None), // { "ecmaVersion": 6 },
        ("const { no_camelcased = foo_bar } = bar;", None), // { "ecmaVersion": 6 },
        ("not_ignored_foo = 0;", Some(serde_json::json!([{ "allow": ["ignored_bar"] }]))),
        ("not_ignored_foo = 0;", Some(serde_json::json!([{ "allow": ["_id$"] }]))),
        (
            "foo = { [computed_bar]: 0 };",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 6 },
        ("({ a: obj.fo_o } = bar);", None), // { "ecmaVersion": 6 },
        ("({ a: obj.fo_o } = bar);", Some(serde_json::json!([{ "ignoreDestructuring": true }]))), // { "ecmaVersion": 6 },
        ("({ a: obj.fo_o.b_ar } = baz);", None), // { "ecmaVersion": 6 },
        ("({ a: { b: { c: obj.fo_o } } } = bar);", None), // { "ecmaVersion": 6 },
        ("({ a: { b: { c: obj.fo_o.b_ar } } } = baz);", None), // { "ecmaVersion": 6 },
        ("([obj.fo_o] = bar);", None),           // { "ecmaVersion": 6 },
        ("([obj.fo_o] = bar);", Some(serde_json::json!([{ "ignoreDestructuring": true }]))), // { "ecmaVersion": 6 },
        ("([obj.fo_o = 1] = bar);", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 6 },
        ("({ a: [obj.fo_o] } = bar);", None), // { "ecmaVersion": 6 },
        ("({ a: { b: [obj.fo_o] } } = bar);", None), // { "ecmaVersion": 6 },
        ("([obj.fo_o.ba_r] = baz);", None),   // { "ecmaVersion": 6 },
        ("({...obj.fo_o} = baz);", None),     // { "ecmaVersion": 9 },
        ("({...obj.fo_o.ba_r} = baz);", None), // { "ecmaVersion": 9 },
        ("({c: {...obj.fo_o }} = baz);", None), // { "ecmaVersion": 9 },
        ("obj.o_k.non_camelcase = 0", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 2020 },
        ("(obj?.o_k).non_camelcase = 0", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 2020 },
        ("class C { snake_case; }", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 2022 },
        (
            "class C { #snake_case; foo() { this.#snake_case; } }",
            Some(serde_json::json!([{ "properties": "always" }])),
        ), // { "ecmaVersion": 2022 },
        ("class C { #snake_case() {} }", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 2022 },
        (
            "
			            const { some_property } = obj;
			            doSomething({ some_property });
			            ",
            Some(serde_json::json!([{ "properties": "always", "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            r#"
			            const { some_property } = obj;
			            doSomething({ some_property });
			            doSomething({ [some_property]: "bar" });
			            "#,
            Some(serde_json::json!([{ "properties": "never", "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
			            const { some_property } = obj;
			
			            const bar = { some_property };
			
			            obj.some_property = 10;
			
			            const xyz = { some_property: obj.some_property };
			
			            const foo = ({ some_property }) => {
			                console.log(some_property)
			            };
			            ",
            Some(serde_json::json!([{ "properties": "always", "ignoreDestructuring": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "import('foo.json', { my_with: { [my_type]: 'json' } })",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // { "ecmaVersion": 2025 },
        (
            "import('foo.json', { my_with: { my_type: my_json } })",
            Some(
                serde_json::json!([				{					"properties": "always",					"ignoreImports": false,				},			]),
            ),
        ), // { "ecmaVersion": 2025 }
    ];

    Tester::new(Camelcase::NAME, Camelcase::PLUGIN, pass, fail).test_and_snapshot();
}
