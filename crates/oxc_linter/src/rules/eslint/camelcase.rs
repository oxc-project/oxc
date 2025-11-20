use oxc_ast::AstKind;
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
    allow: Vec<CompactStr>,
}

impl Default for CamelcaseConfig {
    fn default() -> Self {
        Self { allow: vec![] }
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

        let allow = if let Some(allow_value) = config.get("allow") {
            if let Some(allow_array) = allow_value.as_array() {
                allow_array.iter().filter_map(|v| v.as_str()).map(CompactStr::new).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        Self(Box::new(CamelcaseConfig { allow }))
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
            AstKind::BindingIdentifier(binding_ident) => {
                let name = &binding_ident.name;
                let atom_str = name.as_str();

                if !self.is_underscored(atom_str) || self.is_allowed(atom_str) {
                    return;
                }

                ctx.diagnostic(camelcase_diagnostic(binding_ident.span, atom_str));
            }
            _ => {}
        }
    }
}

impl Camelcase {
    fn is_underscored(&self, name: &str) -> bool {
        // Remove leading and trailing underscores
        let name_body = name.trim_start_matches('_').trim_end_matches('_');

        // If there's an underscore, it might be A_CONSTANT, which is okay
        name_body.contains('_') && name_body != name_body.to_uppercase()
    }

    fn is_allowed(&self, name: &str) -> bool {
        self.allow.iter().any(|entry| {
            name == entry.as_str() || {
                // Try to match as regex - simplified, just exact match for now
                false
            }
        })
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var firstName = \"Ned\"", None),
        ("var __myPrivateVariable = \"Patrick\"", None),
        ("var myPrivateVariable_ = \"Patrick\"", None),
        ("function doSomething(){}", None),
        ("var MY_GLOBAL = 1", None),
        ("var ANOTHER_GLOBAL = 1", None),
        ("var foo_bar", Some(serde_json::json!([{ "allow": ["foo_bar"] }]))),
    ];

    let fail = vec![
        ("var no_camelcased = 1;", None),
        ("function no_camelcased(){}", None),
        ("function bar( obj_name ){}", None),
    ];

    Tester::new(Camelcase::NAME, Camelcase::PLUGIN, pass, fail).test_and_snapshot();
}
