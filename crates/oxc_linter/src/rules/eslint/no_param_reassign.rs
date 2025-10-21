use lazy_regex::Regex;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;

use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentTargetPropertyIdentifier, AssignmentTargetPropertyProperty,
        CallExpression, ChainExpression, ComputedMemberExpression, ForInStatement, ForOfStatement,
        ObjectProperty, ParenthesizedExpression, StaticMemberExpression, TSAsExpression,
        TSNonNullExpression, TSSatisfiesExpression, TSTypeAssertion, UnaryExpression,
        UpdateExpression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId, Reference};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn assignment_to_param_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Assignment to function parameter '{name}'.")).with_label(span)
}

fn assignment_to_param_property_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Assignment to property of function parameter '{name}'."))
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct NoParamReassignConfig {
    /// When true, also check for modifications to properties of parameters.
    props: bool,
    /// An array of parameter names whose property modifications should be ignored.
    ignore_property_modifications_for: FxHashSet<String>,
    /// An array of regex patterns (as strings) for parameter names whose property modifications should be ignored.
    /// Note that this uses [Rust regex syntax](https://docs.rs/regex/latest/regex/) and so may not have all features
    /// available to JavaScript regexes.
    #[schemars(with = "Vec<String>", default)]
    ignore_property_modifications_for_regex: Vec<Regex>,
}

impl NoParamReassignConfig {
    fn is_ignored(&self, name: &str) -> bool {
        self.ignore_property_modifications_for.contains(name)
            || self.ignore_property_modifications_for_regex.iter().any(|regex| regex.is_match(name))
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoParamReassign(Box<NoParamReassignConfig>);

// doc: https://github.com/eslint/eslint/blob/v9.9.1/docs/src/rules/no-param-reassign.md
// code: https://github.com/eslint/eslint/blob/v9.9.1/lib/rules/no-param-reassign.js
// test: https://github.com/eslint/eslint/blob/v9.9.1/tests/lib/rules/no-param-reassign.js
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow reassigning function parameters or, optionally, their properties.
    ///
    /// ### Why is this bad?
    ///
    /// Reassigning parameters can lead to unexpected behavior, especially when relying on the
    /// original arguments passed into the function. Mutating parameter properties can be similarly
    /// surprising and harder to reason about.
    ///
    /// ### Examples
    ///
    /// ```javascript
    /// function foo(bar) {
    ///   bar = 1;
    /// }
    ///
    /// function baz(qux) {
    ///   qux.prop = 2; // when `props` option is enabled
    /// }
    /// ```
    NoParamReassign,
    eslint,
    restriction,
    config = NoParamReassignConfig,
);

impl Rule for NoParamReassign {
    fn from_configuration(value: Value) -> Self {
        let mut rule = Self::default();
        let config = &mut *rule.0;
        let Value::Array(array) = value else { return rule };
        let Some(Value::Object(options)) = array.first() else { return rule };

        if let Some(Value::Bool(props)) = options.get("props") {
            config.props = *props;
        }

        if !config.props {
            return rule;
        }

        if let Some(Value::Array(items)) = options.get("ignorePropertyModificationsFor") {
            for item in items {
                if let Value::String(value) = item {
                    config.ignore_property_modifications_for.insert(value.clone());
                }
            }
        }

        if let Some(Value::Array(items)) = options.get("ignorePropertyModificationsForRegex") {
            for item in items {
                if let Value::String(value) = item
                    && let Ok(regex) = Regex::new(value)
                {
                    config.ignore_property_modifications_for_regex.push(regex);
                }
            }
        }

        rule
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::FormalParameter(param) = node.kind() else {
            return;
        };

        let symbol_table = ctx.scoping();
        for ident in param.pattern.get_binding_identifiers() {
            let Some(symbol_id) = ident.symbol_id.get() else {
                continue;
            };

            let declaration_id = symbol_table.symbol_declaration(symbol_id);
            let name = symbol_table.symbol_name(symbol_id);

            let mut seen_nodes: FxHashSet<NodeId> = FxHashSet::default();

            for reference in symbol_table.get_resolved_references(symbol_id) {
                let node_id = reference.node_id();
                if !seen_nodes.insert(node_id) {
                    continue;
                }

                if ctx.nodes().ancestor_ids(node_id).any(|ancestor| ancestor == declaration_id) {
                    continue;
                }

                let span = ctx.semantic().reference_span(reference);

                if reference.is_write() {
                    ctx.diagnostic(assignment_to_param_diagnostic(name, span));
                    continue;
                }

                if self.0.props && !self.0.is_ignored(name) && is_modifying_property(reference, ctx)
                {
                    ctx.diagnostic(assignment_to_param_property_diagnostic(name, span));
                }
            }
        }
    }
}

fn is_modifying_property(reference: &Reference, ctx: &LintContext<'_>) -> bool {
    let nodes = ctx.nodes();
    let mut current_id = reference.node_id();
    let mut current_span = nodes.get_node(current_id).span();

    loop {
        let parent_id = nodes.parent_id(current_id);
        if parent_id == NodeId::ROOT {
            return false;
        }

        let parent_node = nodes.get_node(parent_id);
        match parent_node.kind() {
            AstKind::AssignmentExpression(AssignmentExpression { left, .. }) => {
                return left.span().contains_inclusive(current_span);
            }
            AstKind::UpdateExpression(UpdateExpression { argument, .. }) => {
                return argument.span().contains_inclusive(current_span);
            }
            AstKind::UnaryExpression(UnaryExpression {
                operator: UnaryOperator::Delete,
                argument,
                ..
            }) => {
                return argument.span().contains_inclusive(current_span);
            }
            AstKind::UnaryExpression(_) => {
                return false;
            }
            AstKind::ForInStatement(ForInStatement { left, .. })
            | AstKind::ForOfStatement(ForOfStatement { left, .. }) => {
                return left.span().contains_inclusive(current_span);
            }
            AstKind::StaticMemberExpression(StaticMemberExpression { object, .. })
            | AstKind::ComputedMemberExpression(ComputedMemberExpression { object, .. }) => {
                if object.span() != current_span {
                    return false;
                }
            }
            AstKind::ObjectProperty(ObjectProperty { key, .. }) => {
                if key.span() == current_span {
                    return false;
                }
            }
            AstKind::AssignmentTargetPropertyIdentifier(AssignmentTargetPropertyIdentifier {
                binding,
                ..
            }) => {
                if binding.span == current_span {
                    return false;
                }
            }
            AstKind::AssignmentTargetPropertyProperty(AssignmentTargetPropertyProperty {
                name,
                ..
            }) => {
                if name.span().contains_inclusive(current_span) {
                    return false;
                }
            }
            AstKind::ConditionalExpression(conditional) => {
                if conditional.test.span() == current_span {
                    return false;
                }
            }
            AstKind::ParenthesizedExpression(ParenthesizedExpression { expression, .. })
            | AstKind::TSAsExpression(TSAsExpression { expression, .. })
            | AstKind::TSNonNullExpression(TSNonNullExpression { expression, .. })
            | AstKind::TSSatisfiesExpression(TSSatisfiesExpression { expression, .. })
            | AstKind::TSTypeAssertion(TSTypeAssertion { expression, .. }) => {
                if expression.span() != current_span {
                    return false;
                }
            }
            AstKind::ChainExpression(ChainExpression { expression, .. }) => {
                if expression.span() != current_span {
                    return false;
                }
            }
            AstKind::CallExpression(CallExpression { callee, .. }) => {
                if callee.span() != current_span {
                    return false;
                }
            }
            kind if kind.is_statement() || kind.is_declaration() => {
                return false;
            }
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) | AstKind::Program(_) => {
                return false;
            }
            _ => {}
        }

        current_id = parent_id;
        current_span = parent_node.span();
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo(a) { var b = a; }", None),
        ("function foo(a) { for (b in a); }", None),
        ("function foo(a) { for (b of a); }", None), // { "ecmaVersion": 6 },
        ("function foo(a) { a.prop = 'value'; }", None),
        ("function foo(a) { for (a.prop in obj); }", None),
        ("function foo(a) { for (a.prop of arr); }", None), // { "ecmaVersion": 6 },
        ("function foo(a) { (function() { var a = 12; a++; })(); }", None),
        ("function foo() { someGlobal = 13; }", None),
        ("function foo() { someGlobal = 13; }", None), // { "globals": { "someGlobal": false } },
        ("function foo(a) { a.b = 0; }", None),
        ("function foo(a) { delete a.b; }", None),
        ("function foo(a) { ++a.b; }", None),
        ("function foo(a) { [a.b] = []; }", None), // { "ecmaVersion": 6 },
        ("function foo(a) { bar(a.b).c = 0; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { data[a.b] = 0; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { +a.b; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { (a ? [] : [])[0] = 1; }", Some(serde_json::json!([{ "props": true }]))),
        (
            "function foo(a) { (a.b ? [] : [])[0] = 1; }",
            Some(serde_json::json!([{ "props": true }])),
        ),
        (
            "function foo(a) { a.b = 0; }",
            Some(serde_json::json!([{ "props": true, "ignorePropertyModificationsFor": ["a"] }])),
        ),
        (
            "function foo(a) { ++a.b; }",
            Some(serde_json::json!([{ "props": true, "ignorePropertyModificationsFor": ["a"] }])),
        ),
        (
            "function foo(a) { delete a.b; }",
            Some(serde_json::json!([{ "props": true, "ignorePropertyModificationsFor": ["a"] }])),
        ),
        (
            "function foo(a) { for (a.b in obj); }",
            Some(serde_json::json!([{ "props": true, "ignorePropertyModificationsFor": ["a"] }])),
        ),
        (
            "function foo(a) { for (a.b of arr); }",
            Some(serde_json::json!([{ "props": true, "ignorePropertyModificationsFor": ["a"] }])),
        ), // { "ecmaVersion": 6 },
        (
            "function foo(a, z) { a.b = 0; x.y = 0; }",
            Some(
                serde_json::json!([				{ "props": true, "ignorePropertyModificationsFor": ["a", "x"] },			]),
            ),
        ),
        (
            "function foo(a) { a.b.c = 0;}",
            Some(serde_json::json!([{ "props": true, "ignorePropertyModificationsFor": ["a"] }])),
        ),
        (
            "function foo(aFoo) { aFoo.b = 0; }",
            Some(
                serde_json::json!([				{ "props": true, "ignorePropertyModificationsForRegex": ["^a.*$"] },			]),
            ),
        ),
        (
            "function foo(aFoo) { ++aFoo.b; }",
            Some(
                serde_json::json!([				{ "props": true, "ignorePropertyModificationsForRegex": ["^a.*$"] },			]),
            ),
        ),
        (
            "function foo(aFoo) { delete aFoo.b; }",
            Some(
                serde_json::json!([				{ "props": true, "ignorePropertyModificationsForRegex": ["^a.*$"] },			]),
            ),
        ),
        (
            "function foo(a, z) { aFoo.b = 0; x.y = 0; }",
            Some(
                serde_json::json!([				{					"props": true,					"ignorePropertyModificationsForRegex": ["^a.*$", "^x.*$"],				},			]),
            ),
        ),
        (
            "function foo(aFoo) { aFoo.b.c = 0;}",
            Some(
                serde_json::json!([				{ "props": true, "ignorePropertyModificationsForRegex": ["^a.*$"] },			]),
            ),
        ),
        (
            "function foo(a) { ({ [a]: variable } = value) }",
            Some(serde_json::json!([{ "props": true }])),
        ), // { "ecmaVersion": 6 },
        ("function foo(a) { ([...a.b] = obj); }", Some(serde_json::json!([{ "props": false }]))), // { "ecmaVersion": 2015 },
        ("function foo(a) { ({...a.b} = obj); }", Some(serde_json::json!([{ "props": false }]))), // { "ecmaVersion": 2018 },
        (
            "function foo(a) { for (obj[a.b] in obj); }",
            Some(serde_json::json!([{ "props": true }])),
        ),
        (
            "function foo(a) { for (obj[a.b] of arr); }",
            Some(serde_json::json!([{ "props": true }])),
        ), // { "ecmaVersion": 6 },
        ("function foo(a) { for (bar in a.b); }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { for (bar of a.b); }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 6 },
        ("function foo(a) { for (bar in baz) a.b; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { for (bar of baz) a.b; }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 6 },
        (
            "function foo(bar, baz) { bar.a = true; baz.b = false; }",
            Some(
                serde_json::json!([				{					"props": true,					"ignorePropertyModificationsForRegex": ["^(foo|bar)$"],					"ignorePropertyModificationsFor": ["baz"],				},			]),
            ),
        ),
    ];

    let fail = vec![
        ("function foo(a) { a = 1; }", None),
        ("function foo(a) { a += 1; }", None),
        ("function foo(a) { ({...a} = obj); }", None),
        ("function foo(a) { for (a in obj); }", None),
        (
            "function foo(a) { for ({bar: a.b} of arr); }",
            Some(serde_json::json!([{ "props": true }])),
        ),
        ("function foo(a) { a.b = 0; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { ++a.b; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { ({bar: a.b} = obj); }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { delete a.b; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(a) { a &&= b; }", None),
        ("function foo(a) { function bar() { a = 2; } }", None),
        ("function foo(bar) { bar = 13; }", None),
        ("function foo(bar) { bar += 13; }", None),
        ("function foo(bar) { (function() { bar = 13; })(); }", None),
        ("function foo(bar) { ++bar; }", None),
        ("function foo(bar) { bar++; }", None),
        ("function foo(bar) { --bar; }", None),
        ("function foo(bar) { bar--; }", None),
        ("function foo({bar}) { bar = 13; }", None), // { "ecmaVersion": 6 },
        ("function foo([, {bar}]) { bar = 13; }", None), // { "ecmaVersion": 6 },
        ("function foo(bar) { ({bar} = {}); }", None), // { "ecmaVersion": 6 },
        ("function foo(bar) { ({x: [, bar = 0]} = {}); }", None), // { "ecmaVersion": 6 },
        ("function foo(bar) { for (bar in baz); }", None),
        ("function foo(bar) { for (bar of baz); }", None), // { "ecmaVersion": 6 },
        ("function foo(bar) { bar.a = 0; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(bar) { bar.get(0).a = 0; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(bar) { delete bar.a; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(bar) { ++bar.a; }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(bar) { for (bar.a in {}); }", Some(serde_json::json!([{ "props": true }]))),
        ("function foo(bar) { for (bar.a of []); }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 6 },
        (
            "function foo(bar) { (bar ? bar : [])[0] = 1; }",
            Some(serde_json::json!([{ "props": true }])),
        ),
        ("function foo(bar) { [bar.a] = []; }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 6 },
        (
            "function foo(bar) { [bar.a] = []; }",
            Some(serde_json::json!([{ "props": true, "ignorePropertyModificationsFor": ["a"] }])),
        ), // { "ecmaVersion": 6 },
        (
            "function foo(bar) { [bar.a] = []; }",
            Some(
                serde_json::json!([				{ "props": true, "ignorePropertyModificationsForRegex": ["^a.*$"] },			]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "function foo(bar) { [bar.a] = []; }",
            Some(
                serde_json::json!([				{ "props": true, "ignorePropertyModificationsForRegex": ["^B.*$"] },			]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "function foo(bar) { ({foo: bar.a} = {}); }",
            Some(serde_json::json!([{ "props": true }])),
        ), // { "ecmaVersion": 6 },
        ("function foo(a) { ({a} = obj); }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 6 },
        ("function foo(a) { ([...a] = obj); }", None), // { "ecmaVersion": 2015 },
        ("function foo(a) { ({...a} = obj); }", None), // { "ecmaVersion": 2018 },
        ("function foo(a) { ([...a.b] = obj); }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 2015 },
        ("function foo(a) { ({...a.b} = obj); }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 2018 },
        (
            "function foo(a) { for ({bar: a.b} in {}); }",
            Some(serde_json::json!([{ "props": true }])),
        ), // { "ecmaVersion": 6 },
        ("function foo(a) { for ([a.b] of []); }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 6 },
        ("function foo(a) { a &&= b; }", None), // { "ecmaVersion": 2021 },
        ("function foo(a) { a ||= b; }", None), // { "ecmaVersion": 2021 },
        ("function foo(a) { a ??= b; }", None), // { "ecmaVersion": 2021 },
        ("function foo(a) { a.b &&= c; }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 2021 },
        ("function foo(a) { a.b.c ||= d; }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 2021 },
        ("function foo(a) { a[b] ??= c; }", Some(serde_json::json!([{ "props": true }]))), // { "ecmaVersion": 2021 }
    ];

    Tester::new(NoParamReassign::NAME, NoParamReassign::PLUGIN, pass, fail).test_and_snapshot();
}
