use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn complexity_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(untagged, rename_all = "camelCase")]
enum ConfigElement0 {
    Unlabeled1(i32),
    Unlabeled2(ConfigElement00),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
struct ConfigElement00 {
    max: i32,
    maximum: i32,
    variant: Variant,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(untagged, rename_all = "camelCase")]
enum Variant {
    #[default]
    Classic,
    Modified,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Complexity(ConfigElement0);

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
    Complexity,
    eslint,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending, // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
    config = Complexity,
);

impl Rule for Complexity {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function a(x) {}", None),
        ("function b(x) {}", Some(serde_json::json!([1]))),
        ("function a(x) {if (true) {return x;}}", Some(serde_json::json!([2]))),
        ("function a(x) {if (true) {return x;} else {return x+1;}}", Some(serde_json::json!([2]))),
        (
            "function a(x) {if (true) {return x;} else if (false) {return x+1;} else {return 4;}}",
            Some(serde_json::json!([3])),
        ),
        (
            "function a(x) {for(var i = 0; i < 5; i ++) {x ++;} return x;}",
            Some(serde_json::json!([2])),
        ),
        ("function a(obj) {for(var i in obj) {obj[i] = 3;}}", Some(serde_json::json!([2]))),
        (
            "function a(x) {for(var i = 0; i < 5; i ++) {if(i % 2 === 0) {x ++;}} return x;}",
            Some(serde_json::json!([3])),
        ),
        (
            "function a(obj) {if(obj){ for(var x in obj) {try {x.getThis();} catch (e) {x.getThat();}}} else {return false;}}",
            Some(serde_json::json!([4])),
        ),
        (
            "function a(x) {try {x.getThis();} catch (e) {x.getThat();}}",
            Some(serde_json::json!([2])),
        ),
        ("function a(x) {return x === 4 ? 3 : 5;}", Some(serde_json::json!([2]))),
        ("function a(x) {return x === 4 ? 3 : (x === 3 ? 2 : 1);}", Some(serde_json::json!([3]))),
        ("function a(x) {return x || 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x && 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x ?? 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x ||= 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x &&= 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x ??= 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x = 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x |= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x &= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x += 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x >>= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x >>>= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x == 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x === 4;}", Some(serde_json::json!([1]))),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: 3;}}",
            Some(serde_json::json!([3])),
        ),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: if(x == 'foo') {5;};}}",
            Some(serde_json::json!([4])),
        ),
        ("function a(x) {while(true) {'foo';}}", Some(serde_json::json!([2]))),
        ("function a(x) {do {'foo';} while (true)}", Some(serde_json::json!([2]))),
        ("if (foo) { bar(); }", Some(serde_json::json!([3]))),
        ("var a = (x) => {do {'foo';} while (true)}", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: 3;}}",
            Some(serde_json::json!([{ "max": 2, "variant": "modified" }])),
        ),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: if(x == 'foo') {5;};}}",
            Some(serde_json::json!([{ "max": 3, "variant": "modified" }])),
        ),
        ("function foo() { class C { x = a || b; y = c || d; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { static x = a || b; static y = c || d; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { x = a || b; y = c || d; } e || f; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { a || b; class C { x = c || d; y = e || f; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("function foo() { class C { [x || y] = a || b; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b; y() { c || d; } z = e || f; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x() { a || b; } y = c || d; z() { e || f; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = (() => { a || b }) || (() => { c || d }) }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = () => { a || b }; y = () => { c || d } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || (() => { b || c }); }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = class { y = a || b; z = c || d; }; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || class { y = b || c; z = d || e; }; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x; y = a; static z; static q = b; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { static { a || b; } static { c || d; } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("function foo() { a || b; class C { static { c || d; } } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo() { class C { static { a || b; } } c || d; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { static { a || b; } } class D { static { c || d; } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; } static { c || d; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } static { c || d; } static { e || f; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { () => a || b; c || d; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; () => c || d; } static { c || d; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a } }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { static { a } static { b } }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } } class D { static { c || d; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; } static c = d || e; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static a = b || c; static { c || d; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; } c = d || e; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { a = b || c; static { d || e; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; c || d; } }", Some(serde_json::json!([3]))), // { "ecmaVersion": 2022 },
        ("class C { static { if (a || b) c = d || e; } }", Some(serde_json::json!([4]))), // { "ecmaVersion": 2022 },
        ("function b(x) {}", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { b?.c; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b = '') {}", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { const { c = '' } = b; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { const [ c = '' ] = b; }", Some(serde_json::json!([{ "max": 2 }]))),
    ];

    let fail = vec![
        ("function a(x) {}", Some(serde_json::json!([0]))),
        (
            "function foo(x) {if (x > 10) {return 'x is greater than 10';} else if (x > 5) {return 'x is greater than 5';} else {return 'x is less than 5';}}",
            Some(serde_json::json!([2])),
        ),
        ("var func = function () {}", Some(serde_json::json!([0]))),
        ("var obj = { a(x) {} }", Some(serde_json::json!([0]))), // { "ecmaVersion": 6 },
        ("class Test { a(x) {} }", Some(serde_json::json!([0]))), // { "ecmaVersion": 6 },
        ("var a = (x) => {if (true) {return x;}}", Some(serde_json::json!([1]))), // { "ecmaVersion": 6 },
        ("function a(x) {if (true) {return x;}}", Some(serde_json::json!([1]))),
        ("function a(x) {if (true) {return x;} else {return x+1;}}", Some(serde_json::json!([1]))),
        (
            "function a(x) {if (true) {return x;} else if (false) {return x+1;} else {return 4;}}",
            Some(serde_json::json!([2])),
        ),
        (
            "function a(x) {for(var i = 0; i < 5; i ++) {x ++;} return x;}",
            Some(serde_json::json!([1])),
        ),
        ("function a(obj) {for(var i in obj) {obj[i] = 3;}}", Some(serde_json::json!([1]))),
        ("function a(obj) {for(var i of obj) {obj[i] = 3;}}", Some(serde_json::json!([1]))), // { "ecmaVersion": 6 },
        (
            "function a(x) {for(var i = 0; i < 5; i ++) {if(i % 2 === 0) {x ++;}} return x;}",
            Some(serde_json::json!([2])),
        ),
        (
            "function a(obj) {if(obj){ for(var x in obj) {try {x.getThis();} catch (e) {x.getThat();}}} else {return false;}}",
            Some(serde_json::json!([3])),
        ),
        (
            "function a(x) {try {x.getThis();} catch (e) {x.getThat();}}",
            Some(serde_json::json!([1])),
        ),
        ("function a(x) {return x === 4 ? 3 : 5;}", Some(serde_json::json!([1]))),
        ("function a(x) {return x === 4 ? 3 : (x === 3 ? 2 : 1);}", Some(serde_json::json!([2]))),
        ("function a(x) {return x || 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x && 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x ?? 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x ||= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x &&= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x ??= 4;}", Some(serde_json::json!([1]))),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: 3;}}",
            Some(serde_json::json!([2])),
        ),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: if(x == 'foo') {5;};}}",
            Some(serde_json::json!([3])),
        ),
        ("function a(x) {while(true) {'foo';}}", Some(serde_json::json!([1]))),
        ("function a(x) {do {'foo';} while (true)}", Some(serde_json::json!([1]))),
        (
            "function a(x) {(function() {while(true){'foo';}})(); (function() {while(true){'bar';}})();}",
            Some(serde_json::json!([1])),
        ),
        (
            "function a(x) {(function() {while(true){'foo';}})(); (function() {'bar';})();}",
            Some(serde_json::json!([1])),
        ),
        ("var obj = { a(x) { return x ? 0 : 1; } };", Some(serde_json::json!([1]))), // { "ecmaVersion": 6 },
        ("var obj = { a: function b(x) { return x ? 0 : 1; } };", Some(serde_json::json!([1]))),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: 3;}}",
            Some(serde_json::json!([{ "max": 1, "variant": "modified" }])),
        ),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: if(x == 'foo') {5;};}}",
            Some(serde_json::json!([{ "max": 2, "variant": "modified" }])),
        ),
        ("function foo () { a || b; class C { x; } c || d; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { a || b; class C { x = c; } d || e; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { a || b; class C { [x || y]; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { a || b; class C { [x || y] = c; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { class C { [x || y]; } a || b; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { class C { [x || y] = a; } b || c; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { class C { [x || y]; [z || q]; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo () { class C { [x || y] = a; [z || q] = b; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo () { a || b; class C { x = c || d; } e || f; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { x(){ a || b; } y = c || d || e; z() { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { x = a || b; y() { c || d || e; } z = f || g; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x; y() { c || d || e; } z; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("(class { x = a || b; })", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { static x = a || b; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("(class { x = a ? b : c; })", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b || c; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b; y = b || c || d; z = e || f; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b || c; y = d || e; z = f || g || h; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = () => a || b || c; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = (() => a || b || c) || d; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = () => a || b || c; y = d || e; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = () => a || b || c; y = d || e || f; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "class C { x = function () { a || b }; y = function () { c || d }; }",
            Some(serde_json::json!([1])),
        ), // { "ecmaVersion": 2022 },
        ("class C { x = class { [y || z]; }; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { x = class { [y || z] = a; }; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { x = class { y = a || b; }; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("function foo () { a || b; class C { static {} } c || d; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo () { a || b; class C { static { c || d; } } e || f; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; }  }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b || c; }  }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; c || d; }  }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; c || d; e || f; }  }", Some(serde_json::json!([3]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; c || d; { e || f; } }  }", Some(serde_json::json!([3]))), // { "ecmaVersion": 2022 },
        ("class C { static { if (a || b) c = d || e; } }", Some(serde_json::json!([3]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { if (a || b) c = (d => e || f)() || (g => h || i)(); } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { x(){ a || b; } static { c || d || e; } z() { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { x = a || b; static { c || d || e; } y = f || g; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static x = a || b; static { c || d || e; } static y = f || g; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } static(){ c || d || e; } static { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } static static(){ c || d || e; } static { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } static x = c || d || e; static { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b || c || d; } static { e || f || g; } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b || c; } static { d || e || f || g; } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b || c || d; } static { e || f || g || h; } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 2022 },
        ("class C { x = () => a || b || c; y = f || g || h; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function a(x) {}", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "const obj = { b: (a) => a?.b?.c, c: function (a) { return a?.b?.c; } };",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        ("function a(b) { b?.c; }", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { b?.['c']; }", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { b?.c; d || e; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c?.d; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.['c']?.['d']; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c?.['d']; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c.d?.e; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c?.(); }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c?.()?.(); }", Some(serde_json::json!([{ "max": 3 }]))),
        ("function a(b = '') {}", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { const { c = '' } = b; }", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { const [ c = '' ] = b; }", Some(serde_json::json!([{ "max": 1 }]))),
        (
            "function a(b) { const [ { c: d = '' } = {} ] = b; }",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
    ];

    Tester::new(Complexity::NAME, Complexity::PLUGIN, pass, fail).test_and_snapshot();
}
