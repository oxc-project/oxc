use fast_glob::glob_match;
use oxc_ast::{AstKind, ast::JSXElementName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn jsx_pascal_case_diagnostic(
    span: Span,
    component_name: &str,
    allow_all_caps: bool,
) -> OxcDiagnostic {
    let message = if allow_all_caps {
        format!("JSX component {component_name} must be in PascalCase or SCREAMING_SNAKE_CASE")
    } else {
        format!("JSX component {component_name} must be in PascalCase")
    };

    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsxPascalCase(Box<JsxPascalCaseConfig>);

#[derive(Debug, Default, Clone)]
pub struct JsxPascalCaseConfig {
    pub allow_all_caps: bool,
    pub allow_namespace: bool,
    pub allow_leading_underscore: bool,
    pub ignore: Vec<CompactStr>,
}

impl std::ops::Deref for JsxPascalCase {
    type Target = JsxPascalCaseConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce PascalCase for user-defined JSX components
    ///
    /// ### Why is this bad?
    ///
    /// It enforces coding style that user-defined JSX components are defined and referenced in PascalCase. Note that since React's JSX uses the upper vs. lower case convention
    /// to distinguish between local component classes and HTML tags this rule will not warn on components that start with a lower case letter.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Test_component />
    /// <TEST_COMPONENT />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div />
    ///
    /// <TestComponent />
    ///
    /// <TestComponent>
    ///     <div />
    /// </TestComponent>
    ///
    /// <CSSTransitionGroup />
    /// ```
    ///
    /// Examples of **correct** code for the "allowAllCaps" option:
    /// ```jsx
    /// <ALLOWED />
    ///
    /// <TEST_COMPONENT />
    /// ```
    ///
    /// Examples of **correct** code for the "allowNamespace" option:
    /// ```jsx
    /// <Allowed.div />
    ///
    /// <TestComponent.p />
    /// ```
    ///
    /// Examples of **correct** code for the "allowLeadingUnderscore" option:
    /// ```jsx
    /// <_AllowedComponent />
    ///
    /// <_AllowedComponent>
    ///     <div />
    /// </_AllowedComponent>
    /// ```
    ///
    /// ### Options
    ///
    /// #### allowAllCaps
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// Optional boolean set to true to allow components name in all caps
    ///
    /// #### allowLeadingUnderscore
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// Optional boolean set to true to allow components name with that starts with an underscore
    ///
    /// #### allowNamespace
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// Optional boolean set to true to ignore namespaced components
    ///
    /// #### ignore
    ///
    /// `{ type: Array<string | RegExp>, default: [] }`
    ///
    /// Optional string-array of component names to ignore during validation
    ///
    JsxPascalCase,
    react,
    style
);

impl Rule for JsxPascalCase {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        let allow_all_caps = config
            .and_then(|v| v.get("allowAllCaps"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();

        let allow_namespace = config
            .and_then(|v| v.get("allowNamespace"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();

        let allow_leading_underscore = config
            .and_then(|v| v.get("allowLeadingUnderscore"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();

        let ignore = config
            .and_then(|v| v.get("ignore"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(JsxPascalCaseConfig {
            allow_all_caps,
            allow_namespace,
            allow_leading_underscore,
            ignore,
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_elem) = node.kind() else {
            return;
        };

        let mut is_namespaced_name = false;
        let mut is_member_expression = false;

        let name = match &jsx_elem.name {
            JSXElementName::IdentifierReference(id) => id.name.as_str(),
            JSXElementName::NamespacedName(namespaced) => {
                is_namespaced_name = true;
                &namespaced.to_string()
            }
            JSXElementName::MemberExpression(member_expr) => {
                is_member_expression = true;
                &member_expr.to_string()
            }
            JSXElementName::Identifier(id) if !id.name.chars().next().unwrap().is_lowercase() => {
                id.name.as_str()
            }
            _ => return,
        };

        if name.chars().nth(0).is_some_and(char::is_lowercase) {
            return;
        }

        let check_names: Vec<&str> = if is_namespaced_name {
            name.split(':').collect()
        } else if is_member_expression {
            name.split('.').collect()
        } else {
            vec![name]
        };

        for split_name in check_names {
            if split_name.len() == 1 {
                return;
            }

            let is_ignored = check_ignore(&self.ignore, split_name);

            let check_name = if self.allow_leading_underscore && split_name.starts_with('_') {
                split_name.strip_prefix('_').unwrap_or(split_name)
            } else {
                split_name
            };

            let is_pascal_case = check_pascal_case(check_name);
            let is_allowed_all_caps = self.allow_all_caps && check_all_caps(check_name);

            if !is_pascal_case && !is_allowed_all_caps && !is_ignored {
                ctx.diagnostic(jsx_pascal_case_diagnostic(
                    jsx_elem.span,
                    split_name,
                    self.allow_all_caps,
                ));
            }

            // if namespaces allowed check only first part of component name
            if self.allow_namespace {
                return;
            }
        }
    }
}

fn check_all_caps(check_name: &str) -> bool {
    let len = check_name.len();

    for (idx, letter) in check_name.chars().enumerate() {
        if idx == 0 || idx == len - 1 {
            if !(letter.is_uppercase() || letter.is_ascii_digit()) {
                return false;
            }
        } else if !(letter.is_uppercase() || letter.is_ascii_digit() || letter == '_') {
            return false;
        }
    }

    true
}

fn check_pascal_case(check_name: &str) -> bool {
    let mut chars = check_name.chars();

    match chars.next() {
        Some(c) if c.is_uppercase() => (),
        _ => return false,
    }

    let mut has_lower_or_digit = false;

    for c in chars {
        if !c.is_alphanumeric() {
            return false;
        }
        if c.is_lowercase() || c.is_ascii_digit() {
            has_lower_or_digit = true;
        }
    }

    has_lower_or_digit
}

fn check_ignore(ignore: &[CompactStr], check_name: &str) -> bool {
    ignore.iter().any(|entry| entry == check_name || glob_match(entry.as_str(), check_name))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<div />", None),
        ("<div></div>", None),
        ("<testcomponent />", None),
        ("<testComponent />", None),
        ("<test_component />", None),
        ("<TestComponent />", None),
        ("<CSSTransitionGroup />", None),
        ("<BetterThanCSS />", None),
        ("<TestComponent><div /></TestComponent>", None),
        ("<Test1Component />", None),
        ("<TestComponent1 />", None),
        ("<T3StComp0Nent />", None),
        ("<Éurströmming />", None),
        ("<Año />", None),
        ("<Søknad />", None),
        ("<T />", None),
        ("<YMCA />", Some(serde_json::json!([{ "allowAllCaps": true }]))),
        ("<TEST_COMPONENT />", Some(serde_json::json!([{ "allowAllCaps": true }]))),
        ("<Modal.Header />", None),
        ("<qualification.T3StComp0Nent />", None),
        ("<Modal:Header />", None),
        ("<IGNORED />", Some(serde_json::json!([{ "ignore": ["IGNORED"] }]))),
        ("<Foo_DEPRECATED />", Some(serde_json::json!([{ "ignore": ["*_DEPRECATED"] }]))),
        (
            "<Foo_DEPRECATED />",
            Some(serde_json::json!([{ "ignore": ["*_*[DEPRECATED,IGNORED]"] }])),
        ),
        ("<$ />", None),
        ("<_ />", None),
        ("<H1>Hello!</H1>", None),
        ("<Typography.P />", None),
        ("<Styled.h1 />", Some(serde_json::json!([{ "allowNamespace": true }]))),
        ("<Styled.H1.H2 />", None),
        (
            "<_TEST_COMPONENT />",
            Some(serde_json::json!([{ "allowAllCaps": true, "allowLeadingUnderscore": true }])),
        ),
        ("<_TestComponent />", Some(serde_json::json!([{ "allowLeadingUnderscore": true }]))),
        ("<Component_ />", Some(serde_json::json!([{ "ignore": ["Component_"] }]))),
    ];

    let fail = vec![
        ("<Test_component />", None),
        ("<TEST_COMPONENT />", None),
        ("<YMCA />", None),
        ("<_TEST_COMPONENT />", Some(serde_json::json!([{ "allowAllCaps": true }]))),
        ("<TEST_COMPONENT_ />", Some(serde_json::json!([{ "allowAllCaps": true }]))),
        ("<TEST-COMPONENT />", Some(serde_json::json!([{ "allowAllCaps": true }]))),
        ("<__ />", Some(serde_json::json!([{ "allowAllCaps": true }]))),
        ("<_div />", Some(serde_json::json!([{ "allowLeadingUnderscore": true }]))),
        (
            "<__ />",
            Some(serde_json::json!([{ "allowAllCaps": true, "allowLeadingUnderscore": true }])),
        ),
        ("<$a />", None),
        ("<Foo_DEPRECATED />", Some(serde_json::json!([{ "ignore": ["*_FOO"] }]))),
        ("<Styled.h1 />", None),
        ("<Styled.H1.h2 />", None),
        ("<Styled.h1.H2 />", None),
        ("<$Typography.P />", None),
        ("<STYLED.h1 />", Some(serde_json::json!([{ "allowNamespace": true }]))),
        ("<_camelCase />", Some(serde_json::json!([{ "allowLeadingUnderscore": true }]))),
        ("<_Test_Component />", Some(serde_json::json!([{ "allowLeadingUnderscore": true }]))),
    ];

    Tester::new(JsxPascalCase::NAME, JsxPascalCase::PLUGIN, pass, fail).test_and_snapshot();
}
