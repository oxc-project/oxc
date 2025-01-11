use oxc_ast::{ast::AssignmentTarget, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_rest_spread_properties_diagnostic(
    span: Span,
    spread_kind: &str,
    message_suffix: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{spread_kind} are not allowed. {message_suffix}")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRestSpreadProperties(Box<NoRestSpreadPropertiesOptions>);

#[derive(Debug, Default, Clone)]
pub struct NoRestSpreadPropertiesOptions {
    object_spread_message: String,
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
    /// ### Example
    ///
    /// ```javascript
    /// let { x, ...y } = z;
    /// let z = { x, ...y };
    /// ```
    ///
    /// ### Options
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///     "no-rest-spread-properties": [
    ///         "error",
    ///         {
    ///             "objectSpreadMessage": "Object spread properties are not allowed.",
    ///             "objectRestMessage": "Object rest properties are not allowed."
    ///         }
    ///     ]
    ///   }
    /// }
    /// ```
    ///
    /// - `objectSpreadMessage`: A message to display when object spread properties are found.
    /// - `objectRestMessage`: A message to display when object rest properties are found.
    ///
    NoRestSpreadProperties,
    oxc,
    restriction,
);

impl Rule for NoRestSpreadProperties {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);
        let object_spread_message = config
            .and_then(|v| v.get("objectSpreadMessage"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let object_rest_message = config
            .and_then(|v| v.get("objectRestMessage"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();

        Self(Box::new(NoRestSpreadPropertiesOptions {
            object_spread_message: object_spread_message.to_string(),
            object_rest_message: object_rest_message.to_string(),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::SpreadElement(spread_element) => {
                if ctx
                    .nodes()
                    .parent_kind(node.id())
                    .is_some_and(|parent| matches!(parent, AstKind::ObjectExpression(_)))
                {
                    ctx.diagnostic(no_rest_spread_properties_diagnostic(
                        spread_element.span,
                        "object spread property",
                        self.object_spread_message.as_str(),
                    ));
                }
            }
            AstKind::BindingRestElement(rest_element) => {
                if ctx
                    .nodes()
                    .parent_kind(node.id())
                    .is_some_and(|parent| matches!(parent, AstKind::ObjectPattern(_)))
                {
                    ctx.diagnostic(no_rest_spread_properties_diagnostic(
                        rest_element.span,
                        "object rest property",
                        self.object_rest_message.as_str(),
                    ));
                }
            }
            AstKind::AssignmentTarget(assign_target) => {
                let AssignmentTarget::ObjectAssignmentTarget(object_assign) = assign_target else {
                    return;
                };
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
