use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

fn no_rest_spread_properties_diagnostic(
    span: Span,
    spread_kind: &str,
    message_suffix: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{spread_kind} are not allowed. {message_suffix}")).with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoRestSpreadProperties(Box<NoRestSpreadPropertiesOptions>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoRestSpreadPropertiesOptions {
    /// A message to display when object spread properties are found.
    object_spread_message: String,
    /// A message to display when object rest properties are found.
    object_rest_message: String,
}

impl std::ops::Deref for NoRestSpreadProperties {
    type Target = NoRestSpreadPropertiesOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow [Object Rest/Spread Properties](https://github.com/tc39/proposal-object-rest-spread#readme).
    ///
    /// ### Why is this bad?
    ///
    /// Object rest/spread properties are a relatively new JavaScript feature that may
    /// not be supported in all target environments. If you need to support older
    /// browsers or JavaScript engines that don't support these features, using them
    /// can cause runtime errors. This rule helps maintain compatibility with older
    /// environments by preventing the use of these modern syntax features.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let { x, ...y } = z;
    /// let z = { x, ...y };
    /// ```
    NoRestSpreadProperties,
    oxc,
    restriction,
    config = NoRestSpreadPropertiesOptions,
);

impl Rule for NoRestSpreadProperties {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoRestSpreadProperties>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::SpreadElement(spread_element) => {
                if matches!(ctx.nodes().parent_kind(node.id()), AstKind::ObjectExpression(_)) {
                    ctx.diagnostic(no_rest_spread_properties_diagnostic(
                        spread_element.span,
                        "object spread property",
                        self.object_spread_message.as_str(),
                    ));
                }
            }
            AstKind::BindingRestElement(rest_element) => {
                if matches!(ctx.nodes().parent_kind(node.id()), AstKind::ObjectPattern(_)) {
                    ctx.diagnostic(no_rest_spread_properties_diagnostic(
                        rest_element.span,
                        "object rest property",
                        self.object_rest_message.as_str(),
                    ));
                }
            }
            AstKind::ObjectAssignmentTarget(object_assign) => {
                let Some(rest) = &object_assign.rest else {
                    return;
                };

                ctx.diagnostic(no_rest_spread_properties_diagnostic(
                    rest.span,
                    "object rest property",
                    self.object_rest_message.as_str(),
                ));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // test case are copied from eslint-plugin-es:
    // https://github.com/mysticatea/eslint-plugin-es/blob/v1.4.1/tests/lib/rules/no-rest-spread-properties.js
    let pass = vec![
        ("[...a]", None),
        ("[...a] = foo", None),
        ("({a: [...b]})", None),
        ("({a: [...b]} = obj)", None),
        ("function f(...a) {}", None),
        ("f(...a)", None),
    ];

    let fail = vec![
        ("({...a})", None),
        ("({...a} = obj)", None),
        ("for ({...a} in foo) {}", None),
        ("function f({...a}) {}", None),
        (
            "({...a})",
            Some(serde_json::json!([{
                "objectSpreadMessage": "Our codebase does not allow object spread properties."
            }])),
        ),
        (
            "({...a} = obj)",
            Some(serde_json::json!([{
                "objectRestMessage": "Our codebase does not allow object rest properties."
            }])),
        ),
    ];

    Tester::new(NoRestSpreadProperties::NAME, NoRestSpreadProperties::PLUGIN, pass, fail)
        .test_and_snapshot();
}
