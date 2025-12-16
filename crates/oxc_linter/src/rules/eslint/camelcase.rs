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
    /// Note: ignoreDestructuring does NOT apply to array destructuring because
    /// array elements have no "property name" to compare against (only indices).
    /// ESLint always checks array destructuring bindings regardless of ignoreDestructuring.
    fn check_binding_pattern(&self, pattern: &oxc_ast::ast::BindingPattern, ctx: &LintContext) {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
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
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "readonly",				},			},
        ("a_global_variable = foo", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "writable" } },
        ("a_global_variable = foo", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "globals": { "a_global_variable": "readonly" } },
        ("({ a_global_variable } = foo)", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        (
            "({ snake_cased: a_global_variable } = foo)",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        (
            "({ snake_cased: a_global_variable = foo } = bar)",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        ("[a_global_variable] = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        ("[a_global_variable = foo] = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        ("foo[a_global_variable] = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"globals": {										"a_global_variable": "readonly",				},			},
        (
            "var foo = { [a_global_variable]: bar }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "readonly",				},			},
        (
            "var { [a_global_variable]: foo } = bar",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "readonly",				},			},
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
        ), // {				"ecmaVersion": 2025,				"globals": {					"my_type": true, 				},			}
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
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        (
            "class Foo { a_global_variable() {} }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        ("a_global_variable: for (;;);", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"globals": {										"a_global_variable": "writable",				},			},
        (
            "if (foo) { let a_global_variable; a_global_variable = bar; }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        (
            "function foo(a_global_variable) { foo = a_global_variable; }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        ("var a_global_variable", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "ecmaVersion": 6 },
        ("function a_global_variable () {}", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // { "ecmaVersion": 6 },
        (
            "const a_global_variable = foo; bar = a_global_variable",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        (
            "bar = a_global_variable; var a_global_variable;",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "writable",				},			},
        ("var foo = { a_global_variable }", Some(serde_json::json!([{ "ignoreGlobals": true }]))), // {				"ecmaVersion": 6,				"globals": {										"a_global_variable": "readonly",				},			},
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
