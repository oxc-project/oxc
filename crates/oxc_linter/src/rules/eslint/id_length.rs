use std::ops::Deref;

use oxc_ast::AstKind;
use oxc_ast::ast::{
    BindingIdentifier, BindingPatternKind, BindingProperty, IdentifierName, PrivateIdentifier,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{ContentEq, GetSpan, Span};
use schemars::JsonSchema;

use crate::{AstNode, context::LintContext, rule::Rule};
use icu_segmenter::GraphemeClusterSegmenter;
use lazy_regex::Regex;
use serde_json::Value;

fn id_length_is_too_short_diagnostic(span: Span, config_min: u64) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier name is too short (< {config_min}).")).with_label(span)
}

fn id_length_is_too_long_diagnostic(span: Span, config_max: u64) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier name is too long (> {config_max}).")).with_label(span)
}

const DEFAULT_MAX_LENGTH: u64 = u64::MAX;
const DEFAULT_MIN_LENGTH: u64 = 2;

#[derive(Debug, Default, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum PropertyKind {
    #[default]
    Always,
    Never,
}

impl PropertyKind {
    pub fn from(raw: &str) -> Self {
        if raw == "never" { PropertyKind::Never } else { PropertyKind::default() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IdLength(Box<IdLengthConfig>);

impl Deref for IdLength {
    type Target = IdLengthConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct IdLengthConfig {
    /// An array of regex patterns for identifiers to exclude from the rule.
    /// For example, `["^x.*"]` would exclude all identifiers starting with "x".
    #[schemars(with = "Vec<String>")]
    exception_patterns: Vec<Regex>,
    /// An array of identifier names that are excluded from the rule.
    /// For example, `["x", "y", "z"]` would allow single-letter identifiers "x", "y", and "z".
    exceptions: Vec<String>,
    /// The maximum number of graphemes allowed in an identifier.
    /// Defaults to no maximum (effectively unlimited).
    max: u64,
    /// The minimum number of graphemes required in an identifier.
    min: u64,
    /// When set to `"never"`, property names are not checked for length.
    /// When set to `"always"` (default), property names are checked just like other identifiers.
    properties: PropertyKind,
}

impl Default for IdLengthConfig {
    fn default() -> Self {
        Self {
            exception_patterns: vec![],
            exceptions: vec![],
            max: DEFAULT_MAX_LENGTH,
            min: DEFAULT_MIN_LENGTH,
            properties: PropertyKind::default(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a minimum and/or maximum identifier length convention by counting the
    /// graphemes for a given identifier.
    ///
    /// ### Why is this bad?
    ///
    /// Very short identifier names like e, x, _t or very long ones like
    /// hashGeneratorResultOutputContainerObject can make code harder to read and potentially less
    /// maintainable. To prevent this, one may enforce a minimum and/or maximum identifier length.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint id-length: "error"*/     // default is minimum 2-chars ({ "min": 2 })
    ///
    /// const x = 5;
    /// obj.e = document.body;
    /// const foo = function (e) { };
    /// try {
    ///     dangerousStuff();
    /// } catch (e) {
    ///     // ignore as many do
    /// }
    /// const myObj = { a: 1 };
    /// (a) => { a * a };
    /// class y { }
    /// class Foo { x() {} }
    /// class Bar { #x() {} }
    /// class Baz { x = 1 }
    /// class Qux { #x = 1 }
    /// function bar(...x) { }
    /// function baz([x]) { }
    /// const [z] = arr;
    /// const { prop: [i]} = {};
    /// function qux({x}) { }
    /// const { j } = {};
    /// const { prop: a} = {};
    /// ({ prop: obj.x } = {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /*eslint id-length: "error"*/     // default is minimum 2-chars ({ "min": 2 })
    ///
    /// const num = 5;
    /// function _f() { return 42; }
    /// function _func() { return 42; }
    /// obj.el = document.body;
    /// const foo = function (evt) { /* do stuff */ };
    /// try {
    ///     dangerousStuff();
    /// } catch (error) {
    ///     // ignore as many do
    /// }
    /// const myObj = { apple: 1 };
    /// (num) => { num * num };
    /// function bar(num = 0) { }
    /// class MyClass { }
    /// class Foo { method() {} }
    /// class Bar { #method() {} }
    /// class Baz { field = 1 }
    /// class Qux { #field = 1 }
    /// function baz(...args) { }
    /// function qux([longName]) { }
    /// const { prop } = {};
    /// const { prop: [name] } = {};
    /// const [longName] = arr;
    /// function foobar({ prop }) { }
    /// function foobaz({ a: prop }) { }
    /// const { a: property } = {};
    /// ({ prop: obj.longName } = {});
    /// const data = { "x": 1 };  // excused because of quotes
    /// data["y"] = 3;  // excused because of calculated property access
    /// ```
    IdLength,
    eslint,
    style,
    config = IdLengthConfig
);

impl Rule for IdLength {
    fn from_configuration(value: Value) -> Self {
        let object = value.get(0).and_then(Value::as_object);

        Self(Box::new(IdLengthConfig {
            exception_patterns: object
                .and_then(|map| map.get("exceptionPatterns"))
                .and_then(Value::as_array)
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|val| val.as_str().and_then(|val| Regex::new(val).ok()))
                .collect(),
            exceptions: object
                .and_then(|map| map.get("exceptions"))
                .and_then(Value::as_array)
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|val| val.as_str())
                .map(ToString::to_string)
                .collect(),
            max: object
                .and_then(|map| map.get("max"))
                .and_then(Value::as_u64)
                .unwrap_or(DEFAULT_MAX_LENGTH),
            min: object
                .and_then(|map| map.get("min"))
                .and_then(Value::as_u64)
                .unwrap_or(DEFAULT_MIN_LENGTH),
            properties: object
                .and_then(|map| map.get("properties"))
                .and_then(Value::as_str)
                .map(PropertyKind::from)
                .unwrap_or_default(),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BindingIdentifier(ident) => {
                self.handle_binding_identifier(ident, node, ctx);
            }
            AstKind::IdentifierName(ident) => {
                self.handle_identifier_name(ident, node, ctx);
            }
            AstKind::PrivateIdentifier(ident) => {
                self.handle_private_identifier(ident, node, ctx);
            }
            _ => {}
        }
    }
}

impl IdLength {
    fn handle_binding_identifier(
        &self,
        ident: &BindingIdentifier,
        node: &AstNode,
        ctx: &LintContext,
    ) {
        let ident_name = ident.name;

        if self.is_exception(&ident_name) {
            return;
        }

        // If identifier is all ASCII, then we can use .len() instead of counting graphemes
        let (is_too_long, is_too_short) = if ident_name.is_ascii() {
            let ident_length = ident_name.len();
            (self.is_too_long(ident_length), self.is_too_short(ident_length))
        } else {
            let segmenter = GraphemeClusterSegmenter::new();
            let graphemes_length = segmenter.segment_str(&ident_name).count() - 1;
            (self.is_too_long(graphemes_length), self.is_too_short(graphemes_length))
        };

        if !is_too_long && !is_too_short {
            return;
        }

        let parent_node = ctx.nodes().parent_node(node.id());

        match parent_node.kind() {
            AstKind::ImportSpecifier(import_specifier) => {
                if import_specifier.imported.name() == import_specifier.local.name {
                    return;
                }
                if !import_specifier.local.content_eq(ident) {
                    return;
                }
            }
            AstKind::BindingProperty(_) => {
                if let AstKind::ObjectPattern(object_pattern) =
                    ctx.nodes().parent_kind(parent_node.id())
                {
                    let binding_property_option =
                        object_pattern.properties.iter().find(|x| x.span == ident.span);

                    if IdLength::is_binding_identifier_or_object_pattern(binding_property_option)
                        && self.properties == PropertyKind::Never
                    {
                        return;
                    }
                } else {
                    return;
                }
            }
            _ => {}
        }

        if is_too_long {
            ctx.diagnostic(id_length_is_too_long_diagnostic(node.span(), self.max));
        }
        if is_too_short {
            ctx.diagnostic(id_length_is_too_short_diagnostic(node.span(), self.min));
        }
    }

    fn handle_identifier_name(&self, ident: &IdentifierName, node: &AstNode, ctx: &LintContext) {
        let ident_name = ident.name;

        if self.is_exception(&ident_name) {
            return;
        }

        let (is_too_long, is_too_short) = if ident_name.is_ascii() {
            let ident_length = ident_name.len();
            (self.is_too_long(ident_length), self.is_too_short(ident_length))
        } else {
            let segmenter = GraphemeClusterSegmenter::new();
            let graphemes_length = segmenter.segment_str(&ident_name).count() - 1;
            (self.is_too_long(graphemes_length), self.is_too_short(graphemes_length))
        };

        if !is_too_long && !is_too_short {
            return;
        }

        let parent_node = ctx.nodes().parent_node(node.id());

        match parent_node.kind() {
            AstKind::ExportSpecifier(_)
            | AstKind::ImportAttribute(_)
            | AstKind::ImportSpecifier(_)
            | AstKind::WithClause(_) => {
                return;
            }
            AstKind::ComputedMemberExpression(_)
            | AstKind::PrivateFieldExpression(_)
            | AstKind::StaticMemberExpression(_) => {
                if !self.should_check_member_expression_property(parent_node, ctx) {
                    return;
                }
            }
            property_key if property_key.is_property_key() => {
                let property_key = property_key.as_property_key_kind().unwrap();
                if self.properties == PropertyKind::Never {
                    return;
                }

                let mut parent_parent_node = ctx.nodes().parent_node(parent_node.id());
                if matches!(parent_parent_node.kind(), AstKind::BindingProperty(_)) {
                    parent_parent_node = ctx.nodes().parent_node(parent_parent_node.id());
                }

                match parent_parent_node.kind() {
                    AstKind::ObjectPattern(object_pattern) => {
                        // TODO: Is there a better way to do this check?
                        let binding_property_option = object_pattern
                            .properties
                            .iter()
                            .find(|x| x.key.span().contains_inclusive(property_key.span()));

                        if IdLength::is_binding_identifier_or_object_pattern(
                            binding_property_option,
                        ) {
                            return;
                        }
                    }
                    AstKind::ObjectAssignmentTarget(_)
                    | AstKind::AssignmentTargetPropertyProperty(_) => {
                        return;
                    }
                    _ => {}
                }
            }
            AstKind::BindingProperty(binding_prop) => {
                if self.properties == PropertyKind::Never {
                    return;
                }
                // If this node is the original identifier in a binding property, we can skip it
                //
                // let {a: foo} = bar;
                //      ^
                if IdLength::is_binding_identifier_or_object_pattern(Some(binding_prop)) {
                    return;
                }
            }
            AstKind::ObjectProperty(_) => {
                if self.properties == PropertyKind::Never {
                    return;
                }
            }
            AstKind::AssignmentTargetPropertyProperty(assignment_target) => {
                // Skip node when it is the original identifier in an assignment target property
                //
                // ({x: a}) = {};
                //   ^
                if assignment_target.name.span() == ident.span {
                    return;
                }
            }
            _ => {}
        }

        if is_too_long {
            ctx.diagnostic(id_length_is_too_long_diagnostic(node.span(), self.max));
        }
        if is_too_short {
            ctx.diagnostic(id_length_is_too_short_diagnostic(node.span(), self.min));
        }
    }

    fn handle_private_identifier(
        &self,
        ident: &PrivateIdentifier,
        node: &AstNode,
        ctx: &LintContext,
    ) {
        let ident_name = ident.name;

        if self.is_exception(&ident_name) {
            return;
        }

        let (is_too_long, is_too_short) = if ident_name.is_ascii() {
            let ident_length = ident_name.len();
            (self.is_too_long(ident_length), self.is_too_short(ident_length))
        } else {
            let segmenter = GraphemeClusterSegmenter::new();
            let graphemes_length = segmenter.segment_str(&ident_name).count() - 1;
            (self.is_too_long(graphemes_length), self.is_too_short(graphemes_length))
        };

        if is_too_long {
            ctx.diagnostic(id_length_is_too_long_diagnostic(node.span(), self.max));
        }
        if is_too_short {
            ctx.diagnostic(id_length_is_too_short_diagnostic(node.span(), self.min));
        }
    }

    fn is_binding_identifier_or_object_pattern(
        binding_property_option: Option<&BindingProperty>,
    ) -> bool {
        let Some(binding_property) = binding_property_option else {
            return false;
        };

        matches!(
            &binding_property.value.kind,
            BindingPatternKind::BindingIdentifier(_) | BindingPatternKind::ObjectPattern(_)
        )
    }

    fn is_exception(&self, identifier: &str) -> bool {
        if self.exceptions.iter().any(|exc| exc == identifier) {
            return true;
        }
        if self.exception_patterns.iter().any(|regex| regex.is_match(identifier)) {
            return true;
        }

        false
    }

    fn is_too_long(&self, ident_length: usize) -> bool {
        ident_length > usize::try_from(self.max).unwrap_or(usize::MAX)
    }

    fn is_too_short(&self, ident_length: usize) -> bool {
        ident_length < usize::try_from(self.min).unwrap_or(0)
    }

    fn should_check_member_expression_property(&self, node: &AstNode, ctx: &LintContext) -> bool {
        // Only check property names in member expressions if properties == Always
        if self.properties != PropertyKind::Always {
            return false;
        }

        // Check if this member expression is on the left side of an assignment
        let parent_kind = ctx.nodes().parent_kind(node.id());

        let is_valid_context = if let AstKind::AssignmentExpression(assignment) = parent_kind {
            // Check if this node is the left operand (not the right)
            assignment.left.span() == node.span()
        } else {
            // Allow destructuring patterns
            matches!(parent_kind, AstKind::AssignmentTargetPropertyProperty(_))
        };

        if !is_valid_context {
            return false;
        }

        // Only check the rightmost property in a chain
        let grandparent_kind = ctx.nodes().parent_kind(node.id());
        !matches!(
            grandparent_kind,
            AstKind::StaticMemberExpression(_)
                | AstKind::ComputedMemberExpression(_)
                | AstKind::PrivateFieldExpression(_)
        )
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var xyz;", None),
        ("var xy = 1;", None),
        ("function xyz() {};", None),
        ("function xyz(abc, de) {};", None),
        ("var obj = { abc: 1, de: 2 };", None),
        ("var obj = { 'a': 1, bc: 2 };", None),
        ("var obj = {}; obj['a'] = 2;", None),
        ("abc = d;", None),
        ("try { blah(); } catch (err) { /* pass */ }", None),
        ("var handler = function ($e) {};", None),
        ("var _a = 2", None),
        ("var _ad$$ = new $;", None),
        ("var xyz = new ΣΣ();", None),
        ("unrelatedExpressionThatNeedsToBeIgnored();", None),
        ("var obj = { 'a': 1, bc: 2 }; obj.tk = obj.a;", None),
        ("var query = location.query.q || '';", None),
        ("var query = location.query.q ? location.query.q : ''", None),
        ("let {a: foo} = bar;", None),       // { "ecmaVersion": 6 },
        ("let foo = { [a]: 1 };", None),     // { "ecmaVersion": 6 },
        ("let foo = { [a + b]: 1 };", None), // { "ecmaVersion": 6 },
        ("var x = Foo(42)", Some(serde_json::json!([{ "min": 1 }]))),
        ("var x = Foo(42)", Some(serde_json::json!([{ "min": 0 }]))),
        ("foo.$x = Foo(42)", Some(serde_json::json!([{ "min": 1 }]))),
        ("var lalala = Foo(42)", Some(serde_json::json!([{ "max": 6 }]))),
        (
            "for (var q, h=0; h < 10; h++) { console.log(h); q++; }",
            Some(serde_json::json!([{ "exceptions": ["h", "q"] }])),
        ),
        ("(num) => { num * num };", None), // { "ecmaVersion": 6 },
        ("function foo(num = 0) { }", None), // { "ecmaVersion": 6 },
        ("class MyClass { }", None),       // { "ecmaVersion": 6 },
        ("class Foo { method() {} }", None), // { "ecmaVersion": 6 },
        ("function foo(...args) { }", None), // { "ecmaVersion": 6 },
        ("var { prop } = {};", None),      // { "ecmaVersion": 6 },
        ("var { [a]: prop } = {};", None), // { "ecmaVersion": 6 },
        ("var { a: foo } = {};", Some(serde_json::json!([{ "min": 3 }]))), // { "ecmaVersion": 6 },
        ("var { prop: foo } = {};", Some(serde_json::json!([{ "max": 3 }]))), // { "ecmaVersion": 6 },
        ("var { longName: foo } = {};", Some(serde_json::json!([{ "min": 3, "max": 5 }]))), // { "ecmaVersion": 6 },
        ("var { foo: a } = {};", Some(serde_json::json!([{ "exceptions": ["a"] }]))), // { "ecmaVersion": 6 },
        ("var { a: { b: { c: longName } } } = {};", None), // { "ecmaVersion": 6 },
        ("({ a: obj.x.y.z } = {});", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("import something from 'y';", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export var num = 0;", None),        // { "ecmaVersion": 6, "sourceType": "module" },
        ("import * as something from 'y';", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { x } from 'y';", None),     // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { x as x } from 'y';", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { 'x' as x } from 'y';", None), // { "ecmaVersion": 2022, "sourceType": "module" },
        ("import { x as foo } from 'y';", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { longName } from 'y';", Some(serde_json::json!([{ "max": 5 }]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { x as bar } from 'y';", Some(serde_json::json!([{ "max": 5 }]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("({ prop: obj.x.y.something } = {});", None), // { "ecmaVersion": 6 },
        ("({ prop: obj.longName } = {});", None),      // { "ecmaVersion": 6 },
        ("var obj = { a: 1, bc: 2 };", Some(serde_json::json!([{ "properties": "never" }]))),
        ("var obj = { [a]: 2 };", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        (
            "var obj = {}; obj.a = 1; obj.bc = 2;",
            Some(serde_json::json!([{ "properties": "never" }])),
        ),
        ("({ prop: obj.x } = {});", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("var obj = { aaaaa: 1 };", Some(serde_json::json!([{ "max": 4, "properties": "never" }]))),
        (
            "var obj = {}; obj.aaaaa = 1;",
            Some(serde_json::json!([{ "max": 4, "properties": "never" }])),
        ),
        (
            "({ a: obj.x.y.z } = {});",
            Some(serde_json::json!([{ "max": 4, "properties": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "({ prop: obj.xxxxx } = {});",
            Some(serde_json::json!([{ "max": 4, "properties": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("var arr = [i,j,f,b]", None),    // { "ecmaVersion": 6 },
        ("function foo([arr]) {}", None), // { "ecmaVersion": 6 },
        ("var {x} = foo;", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("var {x, y: {z}} = foo;", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("let foo = { [a]: 1 };", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 6 },
        ("let foo = { [a + b]: 1 };", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 6 },
        (
            "function BEFORE_send() {};",
            Some(serde_json::json!([{ "min": 3, "max": 5, "exceptionPatterns": ["^BEFORE_"] }])),
        ),
        (
            "function BEFORE_send() {};",
            Some(
                serde_json::json!([{ "min": 3, "max": 5, "exceptionPatterns": ["^BEFORE_", "send$"] },]),
            ),
        ),
        (
            "function BEFORE_send() {};",
            Some(
                serde_json::json!([{ "min": 3, "max": 5, "exceptionPatterns": ["^BEFORE_", "^A", "^Z"] }]),
            ),
        ),
        (
            "function BEFORE_send() {};",
            Some(
                serde_json::json!([{ "min": 3, "max": 5, "exceptionPatterns": ["^A", "^BEFORE_", "^Z"] }]),
            ),
        ),
        (
            "var x = 1 ;",
            Some(serde_json::json!([{ "min": 3, "max": 5, "exceptionPatterns": ["[x-z]"] }])),
        ),
        ("class Foo { #xyz() {} }", None), // { "ecmaVersion": 2022 },
        ("class Foo { xyz = 1 }", None),   // { "ecmaVersion": 2022 },
        ("class Foo { #xyz = 1 }", None),  // { "ecmaVersion": 2022 },
        ("class Foo { #abc() {} }", Some(serde_json::json!([{ "max": 3 }]))), // { "ecmaVersion": 2022 },
        ("class Foo { abc = 1 }", Some(serde_json::json!([{ "max": 3 }]))), // { "ecmaVersion": 2022 },
        ("class Foo { #abc = 1 }", Some(serde_json::json!([{ "max": 3 }]))), // { "ecmaVersion": 2022 },
        ("var 𠮟 = 2", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6 },
        ("var 葛󠄀 = 2", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6 },
        ("var a = { 𐌘: 1 };", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("(𐌘) => { 𐌘 * 𐌘 };", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("class 𠮟 { }", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("class F { 𐌘() {} }", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("class F { #𐌘() {} }", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 2022, },
        ("class F { 𐌘 = 1 }", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 2022, },
        ("class F { #𐌘 = 1 }", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 2022, },
        ("function f(...𐌘) { }", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("function f([𐌘]) { }", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("var [ 𐌘 ] = a;", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("var { p: [𐌘]} = {};", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("function f({𐌘}) { }", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("var { 𐌘 } = {};", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("var { p: 𐌘} = {};", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        ("({ prop: o.𐌘 } = {});", Some(serde_json::json!([{ "min": 1, "max": 1 }]))), // { "ecmaVersion": 6, },
        (
            "import foo from 'foo.json' with { type: 'json' }",
            Some(serde_json::json!([{ "min": 1, "max": 3, "properties": "always" }])),
        ), // { "ecmaVersion": 2025 },
        (
            "export * from 'foo.json' with { type: 'json' }",
            Some(serde_json::json!([{ "min": 1, "max": 3, "properties": "always" }])),
        ), // { "ecmaVersion": 2025 },
        (
            "export { default } from 'foo.json' with { type: 'json' }",
            Some(serde_json::json!([{ "min": 1, "max": 3, "properties": "always" }])),
        ), // { "ecmaVersion": 2025 },
                                                                                      // TODO:
                                                                                      // (
                                                                                      //     "import('foo.json', { with: { type: 'json' } })",
                                                                                      //     Some(serde_json::json!([{ "min": 1, "max": 3, "properties": "always" }])),
                                                                                      // ), // { "ecmaVersion": 2025 },
                                                                                      // (
                                                                                      //     "import('foo.json', { 'with': { type: 'json' } })",
                                                                                      //     Some(serde_json::json!([{ "min": 1, "max": 3, "properties": "always" }])),
                                                                                      // ), // { "ecmaVersion": 2025 },
                                                                                      // (
                                                                                      //     "import('foo.json', { with: { type } })",
                                                                                      //     Some(serde_json::json!([{ "min": 1, "max": 3, "properties": "always" }])),
                                                                                      // ), // { "ecmaVersion": 2025 }
    ];

    let fail = vec![
        ("var x = 1;", None),
        ("var x;", None),
        ("obj.e = document.body;", None),
        ("function x() {};", None),
        ("function xyz(a) {};", None),
        ("var obj = { a: 1, bc: 2 };", None),
        ("try { blah(); } catch (e) { /* pass */ }", None),
        ("var handler = function (e) {};", None),
        ("for (var i=0; i < 10; i++) { console.log(i); }", None),
        ("var j=0; while (j > -10) { console.log(--j); }", None),
        ("var [i] = arr;", None),                   // { "ecmaVersion": 6 },
        ("var [,i,a] = arr;", None),                // { "ecmaVersion": 6 },
        ("function foo([a]) {}", None),             // { "ecmaVersion": 6 },
        ("import x from 'module';", None),          // { "ecmaVersion": 6 },
        ("import { x as z } from 'module';", None), // { "ecmaVersion": 6 },
        ("import { foo as z } from 'module';", None), // { "ecmaVersion": 6 },
        ("import { 'foo' as z } from 'module';", None), // { "ecmaVersion": 2022 },
        ("import * as x from 'module';", None),     // { "ecmaVersion": 6 },
        ("import longName from 'module';", Some(serde_json::json!([{ "max": 5 }]))), // { "ecmaVersion": 6 },
        ("import * as longName from 'module';", Some(serde_json::json!([{ "max": 5 }]))), // { "ecmaVersion": 6 },
        ("import { foo as longName } from 'module';", Some(serde_json::json!([{ "max": 5 }]))), // { "ecmaVersion": 6 },
        ("var _$xt_$ = Foo(42)", Some(serde_json::json!([{ "min": 2, "max": 4 }]))),
        ("var _$x$_t$ = Foo(42)", Some(serde_json::json!([{ "min": 2, "max": 4 }]))),
        ("var toString;", Some(serde_json::json!([{ "max": 5 }]))),
        ("(a) => { a * a };", None),        // { "ecmaVersion": 6 },
        ("function foo(x = 0) { }", None),  // { "ecmaVersion": 6 },
        ("class x { }", None),              // { "ecmaVersion": 6 },
        ("class Foo { x() {} }", None),     // { "ecmaVersion": 6 },
        ("function foo(...x) { }", None),   // { "ecmaVersion": 6 },
        ("function foo({x}) { }", None),    // { "ecmaVersion": 6 },
        ("function foo({x: a}) { }", None), // { "ecmaVersion": 6 },
        ("function foo({x: a, longName}) { }", None), // { "ecmaVersion": 6 },
        ("function foo({ longName: a }) {}", Some(serde_json::json!([{ "min": 3, "max": 5 }]))), // { "ecmaVersion": 6 },
        ("function foo({ prop: longName }) {};", Some(serde_json::json!([{ "min": 3, "max": 5 }]))), // { "ecmaVersion": 6 },
        ("function foo({ a: b }) {};", Some(serde_json::json!([{ "exceptions": ["a"] }]))), // { "ecmaVersion": 6 },
        ("var hasOwnProperty;", Some(serde_json::json!([{ "max": 10, "exceptions": [] }]))),
        ("function foo({ a: { b: { c: d, e } } }) { }", None), // { "ecmaVersion": 6 },
        ("var { x} = {};", None),                              // { "ecmaVersion": 6 },
        ("var { x: a} = {};", None),                           // { "ecmaVersion": 6 },
        ("var { a: a} = {};", None),                           // { "ecmaVersion": 6 },
        ("var { prop: a } = {};", None),                       // { "ecmaVersion": 6 },
        ("var { longName: a } = {};", Some(serde_json::json!([{ "min": 3, "max": 5 }]))), // { "ecmaVersion": 6 },
        ("var { prop: [x] } = {};", None), // { "ecmaVersion": 6 },
        ("var { prop: [[x]] } = {};", None), // { "ecmaVersion": 6 },
        ("var { prop: longName } = {};", Some(serde_json::json!([{ "min": 3, "max": 5 }]))), // { "ecmaVersion": 6 },
        ("var { x: a} = {};", Some(serde_json::json!([{ "exceptions": ["x"] }]))), // { "ecmaVersion": 6 },
        ("var { a: { b: { c: d } } } = {};", None), // { "ecmaVersion": 6 },
        ("var { a: { b: { c: d, e } } } = {};", None), // { "ecmaVersion": 6 },
        ("var { a: { b: { c, e: longName } } } = {};", None), // { "ecmaVersion": 6 },
        ("var { a: { b: { c: d, e: longName } } } = {};", None), // { "ecmaVersion": 6 },
        ("var { a, b: { c: d, e: longName } } = {};", None), // { "ecmaVersion": 6 },
        ("import x from 'y';", None),               // { "ecmaVersion": 6, "sourceType": "module" },
        ("export var x = 0;", None),                // { "ecmaVersion": 6, "sourceType": "module" },
        ("({ a: obj.x.y.z } = {});", None),         // { "ecmaVersion": 6 },
        ("({ prop: obj.x } = {});", None),          // { "ecmaVersion": 6 },
        ("var x = 1;", Some(serde_json::json!([{ "properties": "never" }]))),
        ("var {prop: x} = foo;", Some(serde_json::json!([{ "properties": "never" }]))), // { "ecmaVersion": 6 },
        ("var foo = {x: prop};", Some(serde_json::json!([{ "properties": "always" }]))), // { "ecmaVersion": 6 },
        ("function BEFORE_send() {};", Some(serde_json::json!([{ "min": 3, "max": 5 }]))),
        (
            "function NOTMATCHED_send() {};",
            Some(serde_json::json!([{ "min": 3, "max": 5, "exceptionPatterns": ["^BEFORE_"] }])),
        ),
        (
            "function N() {};",
            Some(serde_json::json!([{ "min": 3, "max": 5, "exceptionPatterns": ["^BEFORE_"] }])),
        ),
        ("class Foo { #x() {} }", None), // { "ecmaVersion": 2022 },
        ("class Foo { x = 1 }", None),   // { "ecmaVersion": 2022 },
        ("class Foo { #x = 1 }", None),  // { "ecmaVersion": 2022 },
        ("class Foo { #abcdefg() {} }", Some(serde_json::json!([{ "max": 3 }]))), // { "ecmaVersion": 2022 },
        ("class Foo { abcdefg = 1 }", Some(serde_json::json!([{ "max": 3 }]))), // { "ecmaVersion": 2022 },
        ("class Foo { #abcdefg = 1 }", Some(serde_json::json!([{ "max": 3 }]))), // { "ecmaVersion": 2022 },
        ("var 𠮟 = 2", None),              // { "ecmaVersion": 6 },
        ("var 葛󠄀 = 2", None),              // { "ecmaVersion": 6 },
        ("var myObj = { 𐌘: 1 };", None),   // { "ecmaVersion": 6, },
        ("(𐌘) => { 𐌘 * 𐌘 };", None),       // { "ecmaVersion": 6, },
        ("class 𠮟 { }", None),            // { "ecmaVersion": 6, },
        ("class Foo { 𐌘() {} }", None),    // { "ecmaVersion": 6, },
        ("class Foo1 { #𐌘() {} }", None),  // { "ecmaVersion": 2022 },
        ("class Foo2 { 𐌘 = 1 }", None),    // { "ecmaVersion": 2022 },
        ("class Foo3 { #𐌘 = 1 }", None),   // { "ecmaVersion": 2022 },
        ("function foo1(...𐌘) { }", None), // { "ecmaVersion": 6, },
        ("function foo([𐌘]) { }", None),   // { "ecmaVersion": 6, },
        ("var [ 𐌘 ] = arr;", None),        // { "ecmaVersion": 6, },
        ("var { prop: [𐌘]} = {};", None),  // { "ecmaVersion": 6, },
        ("function foo({𐌘}) { }", None),   // { "ecmaVersion": 6, },
        ("var { 𐌘 } = {};", None),         // { "ecmaVersion": 6, },
        ("var { prop: 𐌘} = {};", None),    // { "ecmaVersion": 6, },
        ("({ prop: obj.𐌘 } = {});", None), // { "ecmaVersion": 6, }
    ];

    Tester::new(IdLength::NAME, IdLength::PLUGIN, pass, fail).test_and_snapshot();
}
