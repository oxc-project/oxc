use lazy_regex::Regex;
use oxc_ast::AstKind;
use oxc_ast::ast::{AssignmentTarget, BindingPatternKind, Expression, PropertyKey};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{AstNode, context::LintContext, rule::Rule};

fn camelcase_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier '{name}' is not in camel case."))
        .with_help("Rename this identifier to use camelCase.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum PropertiesOption {
    #[default]
    Always,
    Never,
}

/// Pre-compiled allow pattern (either literal string or regex)
#[derive(Debug, Clone)]
enum AllowPattern {
    Literal(String),
    Regex(Regex),
}

impl AllowPattern {
    fn matches(&self, name: &str) -> bool {
        match self {
            AllowPattern::Literal(s) => s == name,
            AllowPattern::Regex(re) => re.is_match(name),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct CamelcaseConfig {
    /// When set to "never", the rule will not check property names.
    properties: PropertiesOption,
    /// When set to true, the rule will not check destructuring identifiers.
    ignore_destructuring: bool,
    /// When set to true, the rule will not check import identifiers.
    ignore_imports: bool,
    /// An array of names or regex patterns to allow.
    /// Patterns starting with `^` or ending with `$` are treated as regular expressions.
    #[serde(default)]
    allow: Vec<String>,
}

/// Runtime configuration with pre-compiled patterns
#[derive(Debug, Clone, Default)]
struct CamelcaseRuntime {
    properties: PropertiesOption,
    ignore_destructuring: bool,
    ignore_imports: bool,
    allow_patterns: Vec<AllowPattern>,
}

impl From<CamelcaseConfig> for CamelcaseRuntime {
    fn from(config: CamelcaseConfig) -> Self {
        let allow_patterns = config
            .allow
            .into_iter()
            .map(|pattern| {
                if pattern.starts_with('^') || pattern.ends_with('$') {
                    Regex::new(&pattern)
                        .map_or_else(|_| AllowPattern::Literal(pattern), AllowPattern::Regex)
                } else {
                    AllowPattern::Literal(pattern)
                }
            })
            .collect();

        Self {
            properties: config.properties,
            ignore_destructuring: config.ignore_destructuring,
            ignore_imports: config.ignore_imports,
            allow_patterns,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Camelcase(Box<CamelcaseRuntime>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces camelCase naming convention.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent naming conventions make code harder to read and maintain.
    /// The camelCase convention is widely used in JavaScript and helps maintain
    /// a consistent codebase.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var my_variable = 1;
    /// function do_something() {}
    /// obj.my_prop = 2;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var myVariable = 1;
    /// function doSomething() {}
    /// obj.myProp = 2;
    /// var CONSTANT_VALUE = 1; // all caps allowed
    /// var _privateVar = 1; // leading underscore allowed
    /// ```
    Camelcase,
    eslint,
    style,
    config = CamelcaseConfig,
);

impl Rule for Camelcase {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config: CamelcaseConfig =
            value.get(0).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
        Self(Box::new(config.into()))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // Variable declarations: var foo_bar = 1;
            AstKind::VariableDeclarator(decl) => {
                // Only check simple binding identifiers, not destructuring patterns
                // Destructuring (ObjectPattern/ArrayPattern) is handled by BindingProperty
                if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Destructuring patterns in variable declarations
            AstKind::BindingProperty(prop) => {
                // Skip if ignoreDestructuring is enabled
                if self.0.ignore_destructuring {
                    return;
                }

                // Check the value (the local binding name)
                match &prop.value.kind {
                    BindingPatternKind::BindingIdentifier(ident) => {
                        self.check_name(&ident.name, ident.span, ctx);
                    }
                    // Handle destructuring with default value: { category_id = 1 }
                    BindingPatternKind::AssignmentPattern(pattern) => {
                        if let BindingPatternKind::BindingIdentifier(ident) = &pattern.left.kind {
                            self.check_name(&ident.name, ident.span, ctx);
                        }
                    }
                    _ => {}
                }
            }

            // Function declarations: function foo_bar() {}
            AstKind::Function(func) => {
                if let Some(ident) = &func.id {
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Function/method parameters
            AstKind::FormalParameter(param) => {
                if let BindingPatternKind::BindingIdentifier(ident) = &param.pattern.kind {
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Object property definitions: { foo_bar: 1 }
            AstKind::ObjectProperty(prop) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }

                // Check property key if it's an identifier
                if let PropertyKey::StaticIdentifier(ident) = &prop.key {
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Class property definitions
            AstKind::PropertyDefinition(prop) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }

                if let PropertyKey::StaticIdentifier(ident) = &prop.key {
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Private identifiers in classes: #foo_bar
            AstKind::PrivateIdentifier(ident) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }
                self.check_name(&ident.name, ident.span, ctx);
            }

            // Method definitions
            AstKind::MethodDefinition(method) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }

                if let PropertyKey::StaticIdentifier(ident) = &method.key {
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Assignment expressions
            AstKind::AssignmentExpression(assign) => {
                match &assign.left {
                    // Simple identifier assignment: foo_bar = 1
                    AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                        self.check_name(&ident.name, ident.span, ctx);
                    }
                    // Member expression assignment: obj.foo_bar = 1 or bar_baz.foo = 1
                    AssignmentTarget::StaticMemberExpression(member) => {
                        // Check the object identifier (bar_baz in bar_baz.foo)
                        if let Expression::Identifier(obj_ident) = &member.object {
                            self.check_name(&obj_ident.name, obj_ident.span, ctx);
                        }
                        // Check the property if properties option is "always"
                        if self.0.properties != PropertiesOption::Never {
                            self.check_name(&member.property.name, member.property.span, ctx);
                        }
                    }
                    _ => {}
                }
            }

            // Import declarations
            AstKind::ImportSpecifier(specifier) => {
                if self.0.ignore_imports {
                    return;
                }

                // Always check the local name (the name used in current scope)
                self.check_name(&specifier.local.name, specifier.local.span, ctx);
            }

            AstKind::ImportDefaultSpecifier(specifier) => {
                if self.0.ignore_imports {
                    return;
                }
                self.check_name(&specifier.local.name, specifier.local.span, ctx);
            }

            AstKind::ImportNamespaceSpecifier(specifier) => {
                if self.0.ignore_imports {
                    return;
                }
                self.check_name(&specifier.local.name, specifier.local.span, ctx);
            }

            // Export all: export * as foo_bar from 'mod'
            AstKind::ExportAllDeclaration(export) => {
                if let Some(exported) = &export.exported
                    && let Some(name) = exported.identifier_name()
                {
                    self.check_name(name.as_str(), exported.span(), ctx);
                }
            }

            // Destructuring assignment (not declaration): ({ foo_bar } = obj)
            // For shorthand: { foo_bar } = obj
            AstKind::AssignmentTargetPropertyIdentifier(ident) => {
                if self.0.ignore_destructuring {
                    return;
                }
                self.check_name(&ident.binding.name, ident.binding.span, ctx);
            }

            // For renamed: { key: foo_bar } = obj - check foo_bar
            AstKind::AssignmentTargetPropertyProperty(prop) => {
                if self.0.ignore_destructuring {
                    return;
                }
                // Check the binding target if it's a simple identifier
                if let oxc_ast::ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(
                    ident,
                ) = &prop.binding
                {
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Labels
            AstKind::LabeledStatement(stmt) => {
                self.check_name(&stmt.label.name, stmt.label.span, ctx);
            }

            _ => {}
        }
    }
}

impl Camelcase {
    /// Check if a name violates the camelCase rule
    fn check_name(&self, name: &str, span: Span, ctx: &LintContext) {
        if self.is_good_name(name) {
            return;
        }
        ctx.diagnostic(camelcase_diagnostic(name, span));
    }

    /// Check if a name is acceptable (either camelCase or in the allow list)
    fn is_good_name(&self, name: &str) -> bool {
        // Check pre-compiled allow patterns first
        if self.0.allow_patterns.iter().any(|p| p.matches(name)) {
            return true;
        }

        !is_underscored(name)
    }
}

/// Check if a name contains underscores in the middle (not camelCase).
/// Leading and trailing underscores are allowed.
/// ALL_CAPS names (constants) are allowed.
fn is_underscored(name: &str) -> bool {
    // Strip leading underscores
    let name = name.trim_start_matches('_');
    // Strip trailing underscores
    let name = name.trim_end_matches('_');

    // Empty string or single char after stripping is fine
    if name.is_empty() {
        return false;
    }

    // Check if it's ALL_CAPS (constant style) - these are allowed
    if is_all_caps(name) {
        return false;
    }

    // Check for underscore in the middle
    name.contains('_')
}

/// Check if a name is in ALL_CAPS style (allowed for constants)
fn is_all_caps(name: &str) -> bool {
    // Must contain at least one letter
    let has_letter = name.chars().any(char::is_alphabetic);
    if !has_letter {
        return false;
    }

    // All letters must be uppercase, and underscores/digits are allowed
    name.chars().all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
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
        ("const { ['foo']: _foo } = obj;", None),
        ("const { [_foo_]: foo } = obj;", None),
        (
            "var { category_id } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "var { category_id: category_id } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "var { category_id = 1 } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("var { category_id: category } = query;", None),
        ("var { _leading } = query;", None),
        ("var { trailing_ } = query;", None),
        (r#"import { camelCased } from "external module";"#, None),
        (r#"import { _leading } from "external module";"#, None),
        (r#"import { trailing_ } from "external module";"#, None),
        (r#"import { no_camelcased as camelCased } from "external-module";"#, None),
        (r#"import { no_camelcased as _leading } from "external-module";"#, None),
        (r#"import { no_camelcased as trailing_ } from "external-module";"#, None),
        (
            r#"import { no_camelcased as camelCased, anotherCamelCased } from "external-module";"#,
            None,
        ),
        ("import { snake_cased } from 'mod'", Some(serde_json::json!([{ "ignoreImports": true }]))),
        ("function foo({ no_camelcased: camelCased }) {};", None),
        ("function foo({ no_camelcased: _leading }) {};", None),
        ("function foo({ no_camelcased: trailing_ }) {};", None),
        ("function foo({ camelCased = 'default value' }) {};", None),
        ("function foo({ _leading = 'default value' }) {};", None),
        ("function foo({ trailing_ = 'default value' }) {};", None),
        ("function foo({ camelCased }) {};", None),
        ("function foo({ _leading }) {}", None),
        ("function foo({ trailing_ }) {}", None),
        ("ignored_foo = 0;", Some(serde_json::json!([{ "allow": ["ignored_foo"] }]))),
        (
            "ignored_foo = 0; ignored_bar = 1;",
            Some(serde_json::json!([{ "allow": ["ignored_foo", "ignored_bar"] }])),
        ),
        ("user_id = 0;", Some(serde_json::json!([{ "allow": ["_id$"] }]))),
        ("__option_foo__ = 0;", Some(serde_json::json!([{ "allow": ["__option_foo__"] }]))),
        (
            "class C { camelCase; #camelCase; #camelCase2() {} }",
            Some(serde_json::json!([{ "properties": "always" }])),
        ),
        (
            "class C { snake_case; #snake_case; #snake_case2() {} }",
            Some(serde_json::json!([{ "properties": "never" }])),
        ),
        // ignoreDestructuring applies to destructuring assignments too
        ("({ foo_bar } = obj);", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
        ("({ key: bar_baz } = obj);", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
    ];

    let fail = vec![
        (r#"first_name = "Nicholas""#, None),
        (r#"__private_first_name = "Patrick""#, None),
        ("function foo_bar(){}", None),
        ("obj.foo_bar = function(){};", None),
        ("bar_baz.foo = function(){};", None),
        ("var foo = { bar_baz: boom.bam_pow }", None),
        ("var o = {bar_baz: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("obj.a_b = 2;", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var { category_id } = query;", None),
        ("var { category_id: category_id } = query;", None),
        ("var { category_id = 1 } = query;", None),
        (r#"import no_camelcased from "external-module";"#, None),
        (r#"import * as no_camelcased from "external-module";"#, None),
        (r#"import { no_camelcased } from "external-module";"#, None),
        (r#"import { no_camelcased as no_camel_cased } from "external module";"#, None),
        (r#"import { camelCased as no_camel_cased } from "external module";"#, None),
        (r#"import { camelCased, no_camelcased } from "external-module";"#, None),
        ("export * as snake_cased from 'mod'", None),
        ("function foo({ no_camelcased }) {};", None),
        ("function foo({ no_camelcased = 'default value' }) {};", None),
        ("const { bar: no_camelcased } = foo;", None),
        ("function foo({ value_1: my_default }) {}", None),
        ("function foo({ isCamelcased: no_camelcased }) {};", None),
        ("var { foo: bar_baz = 1 } = quz;", None),
        ("const { no_camelcased = false } = bar;", None),
        ("not_ignored_foo = 0;", Some(serde_json::json!([{ "allow": ["ignored_bar"] }]))),
        ("class C { snake_case; }", Some(serde_json::json!([{ "properties": "always" }]))),
        (
            "class C { #snake_case; foo() { this.#snake_case; } }",
            Some(serde_json::json!([{ "properties": "always" }])),
        ),
        ("class C { #snake_case() {} }", Some(serde_json::json!([{ "properties": "always" }]))),
        // Ensure var foo_bar = {} is NOT mistaken for destructuring
        ("var foo_bar = {};", None),
        ("var foo_bar = [];", None),
        // Destructuring assignments (not declarations)
        ("({ foo_bar } = obj);", None),
        ("({ key: bar_baz } = obj);", None),
    ];

    Tester::new(Camelcase::NAME, Camelcase::PLUGIN, pass, fail).test_and_snapshot();
}
