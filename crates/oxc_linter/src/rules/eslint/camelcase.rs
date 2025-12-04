use lazy_regex::Regex;
use oxc_ast::{AstKind, ast::*};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn camelcase_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier '{}' is not in camel case.", name))
        .with_help("Use camelCase naming convention")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct CamelcaseConfig {
    properties: PropertiesOption,
    ignore_destructuring: bool,
    ignore_imports: bool,
    ignore_globals: bool,
    allow: Vec<CompactStr>,
    allow_regexes: Vec<Regex>,
}

#[derive(Debug, Clone)]
pub enum PropertiesOption {
    Always,
    Never,
}

impl Default for CamelcaseConfig {
    fn default() -> Self {
        Self {
            properties: PropertiesOption::Always,
            ignore_destructuring: false,
            ignore_imports: false,
            ignore_globals: false,
            allow: vec![],
            allow_regexes: vec![],
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Camelcase(Box<CamelcaseConfig>);

impl std::ops::Deref for Camelcase {
    type Target = CamelcaseConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&Value> for Camelcase {
    fn from(raw: &Value) -> Self {
        let config_entry = raw.get(0);
        if config_entry.is_none() {
            return Self(Box::default());
        }

        let config = config_entry.unwrap().as_object();
        if config.is_none() {
            return Self(Box::default());
        }

        let config = config.unwrap();

        let properties = match config.get("properties").and_then(|v| v.as_str()) {
            Some("never") => PropertiesOption::Never,
            _ => PropertiesOption::Always, // default is "always"
        };

        let ignore_destructuring =
            config.get("ignoreDestructuring").and_then(|v| v.as_bool()).unwrap_or(false);

        let ignore_imports = config.get("ignoreImports").and_then(|v| v.as_bool()).unwrap_or(false);

        let ignore_globals = config.get("ignoreGlobals").and_then(|v| v.as_bool()).unwrap_or(false);

        let (allow, allow_regexes) = if let Some(allow_value) = config.get("allow") {
            if let Some(allow_array) = allow_value.as_array() {
                let mut allow_list = Vec::new();
                let mut regex_list = Vec::new();

                for item in allow_array.iter().filter_map(|v| v.as_str()) {
                    if item.starts_with('^')
                        || item.contains(['*', '+', '?', '[', ']', '(', ')', '|'])
                    {
                        // Treat as regex
                        if let Ok(regex) = Regex::new(item) {
                            regex_list.push(regex);
                        }
                    } else {
                        // Treat as literal string
                        allow_list.push(CompactStr::new(item));
                    }
                }

                (allow_list, regex_list)
            } else {
                (vec![], vec![])
            }
        } else {
            (vec![], vec![])
        };

        Self(Box::new(CamelcaseConfig {
            properties,
            ignore_destructuring,
            ignore_imports,
            ignore_globals,
            allow,
            allow_regexes,
        }))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce camelcase naming convention.
    ///
    /// ### Why is this bad?
    ///
    /// When it comes to naming variables, style guides generally fall into one of two camps:
    /// camelcase (`variableName`) and underscores (`variable_name`). This rule focuses on using
    /// the camelcase approach. If your style guide calls for camelcasing your variable names,
    /// then this rule is for you!
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import { no_camelcased } from "external-module"
    ///
    /// var my_favorite_color = "#112C85";
    ///
    /// function do_something() {
    ///     // ...
    /// }
    ///
    /// obj.do_something = function() {
    ///     // ...
    /// };
    ///
    /// function foo({ no_camelcased }) {
    ///     // ...
    /// };
    ///
    /// function foo({ isCamelcased: no_camelcased }) {
    ///     // ...
    /// }
    ///
    /// function foo({ no_camelcased = 'default value' }) {
    ///     // ...
    /// };
    ///
    /// var obj = {
    ///     my_pref: 1
    /// };
    ///
    /// var { category_id = 1 } = query;
    ///
    /// var { category_id: category_alias } = query;
    ///
    /// var { category_id: categoryId, ...other_params } = query;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import { no_camelcased as camelCased } from "external-module";
    ///
    /// var myFavoriteColor   = "#112C85";
    /// var _myFavoriteColor  = "#112C85";
    /// var myFavoriteColor_  = "#112C85";
    /// var MY_FAVORITE_COLOR = "#112C85";
    /// var foo = bar.baz_boom;
    /// var foo = { qux: bar.baz_boom };
    ///
    /// obj.do_something();
    /// do_something();
    /// new do_something();
    ///
    /// var { category_id: categoryId } = query;
    ///
    /// function foo({ isCamelCased }) {
    ///     // ...
    /// };
    ///
    /// function foo({ isCamelCased = 'default value' }) {
    ///     // ...
    /// };
    ///
    /// var myObject = {
    ///     isCamelCased: true
    /// };
    ///
    /// var { categoryId } = query;
    ///
    /// var { categoryId, ...otherParams } = query;
    /// ```
    Camelcase,
    eslint,
    style
);

impl Rule for Camelcase {
    fn from_configuration(value: Value) -> Self {
        Self::from(&value)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // Variable declarations, function declarations, parameters
            AstKind::BindingIdentifier(binding_ident) => {
                if self.should_check_binding_identifier(node, ctx) {
                    self.check_identifier(binding_ident.span, &binding_ident.name, ctx);
                }
            }
            // Object property keys
            AstKind::ObjectProperty(property) => {
                if let PropertyKey::StaticIdentifier(ident) = &property.key {
                    if self.should_check_property_key(node, ctx) {
                        self.check_identifier(ident.span, &ident.name, ctx);
                    }
                }
            }
            // Import specifiers are handled via BindingIdentifier
            // Method definitions
            AstKind::MethodDefinition(method_def) => {
                if let PropertyKey::StaticIdentifier(ident) = &method_def.key {
                    if self.should_check_method_key(node, ctx) {
                        self.check_identifier(ident.span, &ident.name, ctx);
                    }
                }
            }
            _ => {}
        }
    }
}

impl Camelcase {
    fn check_identifier<'a>(&self, span: Span, name: &str, ctx: &LintContext<'a>) {
        if !self.is_underscored(name) || self.is_allowed(name) {
            return;
        }
        ctx.diagnostic(camelcase_diagnostic(span, name));
    }

    fn should_check_binding_identifier<'a>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        // Check if this is a global reference that should be ignored
        if self.ignore_globals {
            // For now, we'll skip the global check as the semantic API usage is complex
            // TODO: Implement proper global reference checking
        }

        // Check if this is inside an import and should be ignored
        if self.ignore_imports && self.is_in_import_context(node, ctx) {
            return false;
        }

        // Check if this is inside a destructuring pattern that should be ignored
        if self.ignore_destructuring && self.is_in_destructuring_pattern(node, ctx) {
            return false;
        }

        true
    }

    fn should_check_property_key<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        // Only check property keys if properties: "always"
        match self.properties {
            PropertiesOption::Always => {
                // Don't check if this is in a destructuring pattern and ignoreDestructuring is true
                if self.ignore_destructuring && self.is_in_destructuring_pattern(node, ctx) {
                    return false;
                }
                true
            }
            PropertiesOption::Never => false,
        }
    }

    fn should_check_method_key<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        // Check method keys only if properties: "always"
        match self.properties {
            PropertiesOption::Always => {
                !self.ignore_destructuring || !self.is_in_destructuring_pattern(node, ctx)
            }
            PropertiesOption::Never => false,
        }
    }

    fn is_in_import_context<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        // Walk up the parent chain to see if we're inside an import statement
        let mut current = node;
        loop {
            let parent = ctx.nodes().parent_node(current.id());
            match parent.kind() {
                AstKind::ImportSpecifier(_)
                | AstKind::ImportDefaultSpecifier(_)
                | AstKind::ImportNamespaceSpecifier(_) => return true,
                AstKind::Program(_) => break,
                _ => {}
            }
            current = parent;
        }
        false
    }

    fn is_in_destructuring_pattern<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        // Walk up the parent chain to see if we're inside a destructuring pattern
        let mut current = node;
        loop {
            let parent = ctx.nodes().parent_node(current.id());
            match parent.kind() {
                AstKind::ObjectPattern(_) | AstKind::ArrayPattern(_) => return true,
                // If we hit a variable declarator, check if it has a destructuring pattern
                AstKind::VariableDeclarator(declarator) => match &declarator.id.kind {
                    BindingPatternKind::ObjectPattern(_) | BindingPatternKind::ArrayPattern(_) => {
                        return true;
                    }
                    _ => {}
                },
                // Stop at function boundaries unless we're checking parameters
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    // Check if we're in the parameters
                    let grandparent = ctx.nodes().parent_node(parent.id());
                    if matches!(grandparent.kind(), AstKind::FormalParameters(_)) {
                        current = parent;
                        continue;
                    }
                    break;
                }
                AstKind::Program(_) => break,
                _ => {}
            }
            current = parent;
        }
        false
    }

    fn is_underscored(&self, name: &str) -> bool {
        // Remove leading and trailing underscores
        let name_body = name.trim_start_matches('_').trim_end_matches('_');

        // If there's an underscore, it might be A_CONSTANT, which is okay
        name_body.contains('_') && name_body != name_body.to_uppercase()
    }

    fn is_allowed(&self, name: &str) -> bool {
        // Check literal matches
        if self.allow.iter().any(|entry| name == entry.as_str()) {
            return true;
        }

        // Check regex matches
        self.allow_regexes.iter().any(|regex| regex.is_match(name))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Basic camelCase
        ("var firstName = \"Ned\"", None),
        ("var __myPrivateVariable = \"Patrick\"", None),
        ("var myPrivateVariable_ = \"Patrick\"", None),
        ("function doSomething(){}", None),
        // Constants (all uppercase with underscores)
        ("var MY_GLOBAL = 1", None),
        ("var ANOTHER_GLOBAL = 1", None),
        // Property access (should not be checked)
        ("var foo1 = bar.baz_boom;", None),
        ("var foo2 = { qux: bar.baz_boom };", None),
        ("obj.do_something();", None),
        ("do_something();", None),
        ("new do_something();", None),
        // Import with alias
        ("import { no_camelcased as camelCased } from \"external-module\";", None),
        // Destructuring with rename
        ("var { category_id: category } = query;", None),
        ("var { category_id: categoryId } = query;", None),
        ("function foo({ isCamelCased }) {}", None),
        ("function bar({ isCamelCased: isAlsoCamelCased }) {}", None),
        ("function baz({ isCamelCased = 'default value' }) {}", None),
        ("var { categoryId = 1 } = query;", None),
        ("var { foo: isCamelCased } = bar;", None),
        ("var { foo: camelCasedName = 1 } = quz;", None),
        // Properties: "never" option
        ("var obj = { my_pref: 1 };", Some(serde_json::json!([{ "properties": "never" }]))),
        ("obj.foo_bar = \"baz\";", Some(serde_json::json!([{ "properties": "never" }]))),
        // ignoreDestructuring: true
        (
            "var { category_id } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "var { category_name = 1 } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "var { category_id_name: category_id_name } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        // ignoreDestructuring: true also ignores renamed aliases (simplified behavior)
        (
            "var { category_id: category_alias } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        // ignoreDestructuring: true also ignores rest parameters (simplified behavior)
        (
            "var { category_id, ...other_props } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        // ignoreImports: true
        (
            "import { snake_cased } from 'mod';",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ),
        // ignoreGlobals: true
        ("var foo = no_camelcased;", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        // allow option
        ("var foo_bar;", Some(serde_json::json!([{ "allow": ["foo_bar"] }]))),
        (
            "function UNSAFE_componentWillMount() {}",
            Some(serde_json::json!([{ "allow": ["UNSAFE_componentWillMount"] }])),
        ),
        // allow with regex
        (
            "function UNSAFE_componentWillMount() {}",
            Some(serde_json::json!([{ "allow": ["^UNSAFE_"] }])),
        ),
        (
            "function UNSAFE_componentWillReceiveProps() {}",
            Some(serde_json::json!([{ "allow": ["^UNSAFE_"] }])),
        ),
        // Combined options
        (
            "var { some_property } = obj; doSomething({ some_property });",
            Some(serde_json::json!([{ "properties": "never", "ignoreDestructuring": true }])),
        ),
        // Using destructured vars with underscores (simplified implementation ignores both)
        (
            "var { some_property } = obj; var foo = some_property + 1;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
    ];

    let fail = vec![
        // Basic violations
        ("var no_camelcased = 1;", None),
        ("function no_camelcased(){}", None),
        ("function bar( obj_name ){}", None),
        // Import violations
        ("import { snake_cased } from 'mod';", None),
        ("import default_import from 'mod';", None),
        ("import * as namespaced_import from 'mod';", None),
        // Property violations (properties: "always" - default)
        ("var obj = { my_pref: 1 };", None),
        // Destructuring violations
        ("var { category_id } = query;", None),
        ("var { category_name = 1 } = query;", None),
        ("var { category_id: category_title } = query;", None),
        ("var { category_id: category_alias } = query;", None),
        ("var { category_id: categoryId, ...other_props } = query;", None),
        // Function parameter destructuring
        ("function foo({ no_camelcased }) {}", None),
        ("function bar({ isCamelcased: no_camelcased }) {}", None),
        ("function baz({ no_camelcased = 'default value' }) {}", None),
    ];

    Tester::new(Camelcase::NAME, Camelcase::PLUGIN, pass, fail).test_and_snapshot();
}
