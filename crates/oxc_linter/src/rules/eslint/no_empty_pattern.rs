use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_empty_array_pattern_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty array binding pattern")
        .with_help("Passing non-iterable values (null, undefined, numbers, booleans, etc.) will result in a runtime error because these values are not iterable.")
        .with_label(span)
}

fn no_empty_object_pattern_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty object binding pattern")
        .with_help("Passing `null` or `undefined` will result in runtime error because `null` and `undefined` cannot be destructured.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoEmptyPattern {
    /// When set to `true`, this rule allows empty object patterns used directly as function
    /// parameters, including parameters defaulted to an empty object literal.
    allow_object_patterns_as_parameters: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow empty destructuring patterns.
    ///
    /// ### Why is this bad?
    ///
    /// When using destructuring, it’s possible to create a pattern that has no effect.
    /// This happens when empty curly braces are used to the right of
    /// an embedded object destructuring pattern, such as:
    ///
    /// ```JavaScript
    /// // doesn't create any variables
    /// var {a: {}} = foo;
    /// ```
    /// In this code, no new variables are created because a is just a location helper
    /// while the `{}` is expected to contain the variables to create, such as:
    ///
    /// ```JavaScript
    /// // creates variable b
    /// var {a: { b }} = foo;
    /// ```
    ///
    /// In many cases, the empty object pattern is a mistake
    /// where the author intended to use a default value instead, such as:
    ///
    /// ```JavaScript
    /// // creates variable a
    /// var {a = {}} = foo;
    /// ```
    ///
    /// The difference between these two patterns is subtle,
    /// especially because the problematic empty pattern looks just like an object literal.
    ///
    /// ### Examples of **incorrect** code for this rule:
    ///
    /// ```JavaScript
    /// var {} = foo;
    /// var [] = foo;
    /// var {a: {}} = foo;
    /// var {a: []} = foo;
    /// function foo({}) {}
    /// function foo([]) {}
    /// function foo({a: {}}) {}
    /// function foo({a: []}) {}
    /// ```
    ///
    /// ### Examples of **correct** code for this rule:
    ///
    /// ```JavaScript
    /// var {a = {}} = foo;
    /// var {a = []} = foo;
    /// function foo({a = {}}) {}
    /// function foo({a = []}) {}
    /// ```
    NoEmptyPattern,
    eslint,
    correctness,
    config = NoEmptyPattern,
    version = "0.0.3",
);

impl Rule for NoEmptyPattern {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ArrayPattern(array) => {
                if array.is_empty() {
                    ctx.diagnostic(no_empty_array_pattern_diagnostic(array.span));
                }
            }
            AstKind::ObjectPattern(object) => {
                if object.is_empty() && !self.is_allowed_empty_parameter_object_pattern(node, ctx) {
                    ctx.diagnostic(no_empty_object_pattern_diagnostic(object.span));
                }
            }
            _ => {}
        }
    }
}

impl NoEmptyPattern {
    fn is_allowed_empty_parameter_object_pattern<'a>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        if self.allow_object_patterns_as_parameters
            && let AstKind::FormalParameter(parameter) = ctx.nodes().parent_kind(node.id())
        {
            return parameter
                .initializer
                .as_ref()
                .is_none_or(|expr| matches!(&**expr, Expression::ObjectExpression(expr) if expr.properties.is_empty()));
        }
        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var {a = {}} = foo;", None),       // { "ecmaVersion": 6 },
        ("var {a, b = {}} = foo;", None),    // { "ecmaVersion": 6 },
        ("var {a = []} = foo;", None),       // { "ecmaVersion": 6 },
        ("function foo({a = {}}) {}", None), // { "ecmaVersion": 6 },
        ("function foo({a = []}) {}", None), // { "ecmaVersion": 6 },
        ("var [a] = foo", None),             // { "ecmaVersion": 6 },
        (
            "function foo({}) {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function({}) {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = ({}) => {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "function foo({} = {}) {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function({} = {}) {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = ({} = {}) => {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 }
    ];

    let fail = vec![
        ("var {} = foo", None),             // { "ecmaVersion": 6 },
        ("var [] = foo", None),             // { "ecmaVersion": 6 },
        ("var {a: {}} = foo", None),        // { "ecmaVersion": 6 },
        ("var {a, b: {}} = foo", None),     // { "ecmaVersion": 6 },
        ("var {a: []} = foo", None),        // { "ecmaVersion": 6 },
        ("function foo({}) {}", None),      // { "ecmaVersion": 6 },
        ("function foo([]) {}", None),      // { "ecmaVersion": 6 },
        ("function foo({a: {}}) {}", None), // { "ecmaVersion": 6 },
        ("function foo({a: []}) {}", None), // { "ecmaVersion": 6 },
        ("function foo({}) {}", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("var foo = function({}) {}", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("var foo = ({}) => {}", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("function foo({} = {}) {}", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("var foo = function({} = {}) {}", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("var foo = ({} = {}) => {}", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        (
            "var foo = ({a: {}}) => {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = ({} = bar) => {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = ({} = { bar: 1 }) => {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = ([]) => {}",
            Some(serde_json::json!([{ "allowObjectPatternsAsParameters": true }])),
        ), // { "ecmaVersion": 6 }
    ];

    Tester::new(NoEmptyPattern::NAME, NoEmptyPattern::PLUGIN, pass, fail).test_and_snapshot();
}
