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

/// Pre-compiled allow pattern
/// ESLint treats each entry as both a literal string AND a regex pattern:
/// `allow.some(entry => name === entry || name.match(new RegExp(entry, "u")))`
#[derive(Debug, Clone)]
struct AllowPattern {
    literal: String,
    regex: Option<Regex>,
}

impl AllowPattern {
    fn new(pattern: String) -> Self {
        // Try to compile as regex (ESLint uses Unicode flag)
        let regex = Regex::new(&pattern).ok();
        Self { literal: pattern, regex }
    }

    fn matches(&self, name: &str) -> bool {
        // ESLint: name === entry || name.match(new RegExp(entry, "u"))
        if name == self.literal {
            return true;
        }
        if let Some(ref re) = self.regex
            && re.is_match(name)
        {
            return true;
        }
        false
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
        let allow_patterns = config.allow.into_iter().map(AllowPattern::new).collect();

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
                // Get the local binding name
                let local_name = match &prop.value.kind {
                    BindingPatternKind::BindingIdentifier(ident) => Some(&ident.name),
                    BindingPatternKind::AssignmentPattern(pattern) => {
                        if let BindingPatternKind::BindingIdentifier(ident) = &pattern.left.kind {
                            Some(&ident.name)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let Some(local_name) = local_name else {
                    return;
                };

                // ESLint ignoreDestructuring: only skip when local name equals property name
                // e.g., { category_id } or { category_id: category_id } -> skip
                // but { category_id: categoryId } -> still check categoryId
                if self.0.ignore_destructuring {
                    let key_name = match &prop.key {
                        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                        _ => None,
                    };
                    if key_name == Some(local_name.as_str()) {
                        return;
                    }
                }

                // Check the local binding name
                match &prop.value.kind {
                    BindingPatternKind::BindingIdentifier(ident) => {
                        self.check_name(&ident.name, ident.span, ctx);
                    }
                    BindingPatternKind::AssignmentPattern(pattern) => {
                        if let BindingPatternKind::BindingIdentifier(ident) = &pattern.left.kind {
                            self.check_name(&ident.name, ident.span, ctx);
                        }
                    }
                    _ => {}
                }
            }

            // Rest element in destructuring: { ...other_props } or [...rest]
            AstKind::BindingRestElement(rest) => {
                // Get the binding name from the rest element's argument
                if let BindingPatternKind::BindingIdentifier(ident) = &rest.argument.kind {
                    // Check if we should skip due to ignoreDestructuring
                    // Rest elements don't have a "property name" to compare, so they're
                    // only skipped when ignoreDestructuring is true (ESLint behavior)
                    if self.0.ignore_destructuring {
                        return;
                    }
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Array destructuring: const [foo_bar, bar_baz] = arr;
            AstKind::ArrayPattern(pattern) => {
                for element in pattern.elements.iter().flatten() {
                    self.check_binding_pattern(element, ctx);
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
                // ESLint ignoreImports: only skip when local name equals imported name
                // e.g., import { snake_case } -> skip (local === imported)
                // but import { snake_case as local_name } -> still check local_name
                if self.0.ignore_imports {
                    let imported_name = specifier.imported.name();
                    if specifier.local.name.as_str() == imported_name.as_str() {
                        return;
                    }
                }

                // Check the local name (the name used in current scope)
                self.check_name(&specifier.local.name, specifier.local.span, ctx);
            }

            // Default imports: import foo_bar from 'mod'
            // No "imported" name to compare, so ignoreImports doesn't apply
            AstKind::ImportDefaultSpecifier(specifier) => {
                self.check_name(&specifier.local.name, specifier.local.span, ctx);
            }

            // Namespace imports: import * as foo_bar from 'mod'
            // No "imported" name to compare, so ignoreImports doesn't apply
            AstKind::ImportNamespaceSpecifier(specifier) => {
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

            // Named exports: export { foo_bar } or export { foo as bar_baz }
            AstKind::ExportSpecifier(specifier) => {
                // Check the exported name (the name visible to importers)
                if let Some(name) = specifier.exported.identifier_name() {
                    self.check_name(name.as_str(), specifier.exported.span(), ctx);
                }
            }

            // Destructuring assignment (not declaration): ({ foo_bar } = obj)
            // For shorthand: { foo_bar } = obj - this is always local === key
            AstKind::AssignmentTargetPropertyIdentifier(ident) => {
                // Shorthand destructuring: local name always equals property name
                // ESLint ignoreDestructuring skips this case
                if self.0.ignore_destructuring {
                    return;
                }
                self.check_name(&ident.binding.name, ident.binding.span, ctx);
            }

            // For renamed: { key: foo_bar } = obj - check foo_bar
            AstKind::AssignmentTargetPropertyProperty(prop) => {
                // Get the local binding name
                let local_name =
                    if let oxc_ast::ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(
                        ident,
                    ) = &prop.binding
                    {
                        Some(ident.name.as_str())
                    } else {
                        None
                    };

                let Some(local_name) = local_name else {
                    return;
                };

                // ESLint ignoreDestructuring: only skip when local name equals property name
                if self.0.ignore_destructuring {
                    let key_name = match &prop.name {
                        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                        _ => None,
                    };
                    if key_name == Some(local_name) {
                        return;
                    }
                }

                // Check the binding target
                if let oxc_ast::ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(
                    ident,
                ) = &prop.binding
                {
                    self.check_name(&ident.name, ident.span, ctx);
                }
            }

            // Labels - both definition and references
            AstKind::LabeledStatement(stmt) => {
                self.check_name(&stmt.label.name, stmt.label.span, ctx);
            }

            // break label_name;
            AstKind::BreakStatement(stmt) => {
                if let Some(label) = &stmt.label {
                    self.check_name(&label.name, label.span, ctx);
                }
            }

            // continue label_name;
            AstKind::ContinueStatement(stmt) => {
                if let Some(label) = &stmt.label {
                    self.check_name(&label.name, label.span, ctx);
                }
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

    /// Check a binding pattern for camelCase violations (used for array destructuring)
    fn check_binding_pattern(&self, pattern: &oxc_ast::ast::BindingPattern, ctx: &LintContext) {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                // For array destructuring, there's no "property name" to compare
                // so ignoreDestructuring skips all array element bindings
                if self.0.ignore_destructuring {
                    return;
                }
                self.check_name(&ident.name, ident.span, ctx);
            }
            BindingPatternKind::AssignmentPattern(assign) => {
                // Handle default values: const [foo_bar = 1] = arr;
                self.check_binding_pattern(&assign.left, ctx);
            }
            // ObjectPattern and ArrayPattern are handled by their own AstKind handlers
            _ => {}
        }
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
        // ignoreDestructuring applies to destructuring assignments too (only shorthand/same-name)
        ("({ foo_bar } = obj);", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
        // ESLint: ignoreDestructuring only skips when local === property name
        // { category_id: camelCase } -> camelCase is valid, so pass
        (
            "var { category_id: camelCase } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        // ignoreImports only skips when local === imported (ESLint behavior)
        // import { snake_case as camelCase } -> camelCase is valid, so pass
        (
            r#"import { snake_case as camelCase } from "mod";"#,
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ),
        // allow patterns work as regex even without ^/$
        ("foo_bar = 0;", Some(serde_json::json!([{ "allow": ["foo.*"] }]))),
        ("get_user_id = 0;", Some(serde_json::json!([{ "allow": ["_id"] }]))),
        // Rest element in destructuring with ignoreDestructuring
        (
            "const { category_id, ...other_props } = obj;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        // Array destructuring with ignoreDestructuring
        (
            "const [foo_bar, bar_baz] = arr;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("const [foo_bar = 1] = arr;", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
    ];

    // Test cases to verify current gaps (should fail but currently pass - known limitations)
    // These are documented as ESLint compatibility gaps:
    // 1. Rest element without ignoreDestructuring - should report other_props
    // 2. Array destructuring - should report foo_bar
    // 3. ignoreDestructuring + later use - ESLint reports but we don't (scope analysis needed)

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
        // ESLint ignoreDestructuring: renamed destructuring should still report
        // { category_id: other_name } -> other_name should be checked (not equal to key)
        (
            "var { category_id: other_name } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "({ key: other_name } = obj);",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        // ESLint ignoreImports: renamed imports should still report
        // import { snake_case as other_snake } -> other_snake should be checked
        (
            r#"import { snake_case as other_snake } from "mod";"#,
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ),
        // Rest element in destructuring (without ignoreDestructuring) - should report
        ("const { foo, ...other_props } = obj;", None),
        // Array destructuring - should report
        ("const [foo_bar] = arr;", None),
        ("const [first, second_item] = arr;", None),
        ("const [foo_bar = 1] = arr;", None), // with default value
                                              // NOTE: Known ESLint compatibility gaps (not currently checked):
                                              // - ignoreDestructuring + later use: ESLint checks variable usage after destructuring
                                              // - ignoreGlobals option: not implemented
    ];

    Tester::new(Camelcase::NAME, Camelcase::PLUGIN, pass, fail).test_and_snapshot();
}
