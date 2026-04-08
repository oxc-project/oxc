use std::cmp::Ordering;

use cow_utils::CowUtils;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn sort_jsx_props_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("JSX props should be sorted alphabetically.").with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Desc,
    #[default]
    Asc,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SortJsxPropsConfig {
    ignore_case: bool,
    order: SortOrder,
}

impl Default for SortJsxPropsConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortJsxProps(Box<SortJsxPropsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted JSX attributes (props).
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted JSX props make components harder to scan and lead to merge
    /// conflicts. Consistent ordering improves readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Component z="1" a="2" m="3" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Component a="2" m="3" z="1" />
    /// ```
    SortJsxProps,
    oxc,
    style,
    none,
    config = SortJsxPropsConfig
);

impl Rule for SortJsxProps {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortJsxPropsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(element) = node.kind() else {
            return;
        };

        if element.attributes.len() < 2 {
            return;
        }

        // Collect named attributes (skip spread)
        let named: Vec<(usize, &str)> = element
            .attributes
            .iter()
            .enumerate()
            .filter_map(|(i, attr)| {
                if let oxc_ast::ast::JSXAttributeItem::Attribute(a) = attr {
                    let name = a.name.as_identifier().map(|ident| ident.name.as_str())?;
                    Some((i, name))
                } else {
                    None
                }
            })
            .collect();

        if named.len() < 2 {
            return;
        }

        for window in named.windows(2) {
            let a_key = if self.0.ignore_case {
                window[0].1.cow_to_ascii_lowercase().into_owned()
            } else {
                window[0].1.to_string()
            };
            let b_key = if self.0.ignore_case {
                window[1].1.cow_to_ascii_lowercase().into_owned()
            } else {
                window[1].1.to_string()
            };

            let ord = a_key.cmp(&b_key);
            let ord = match self.0.order {
                SortOrder::Asc => ord,
                SortOrder::Desc => ord.reverse(),
            };

            if ord == Ordering::Greater {
                ctx.diagnostic(sort_jsx_props_diagnostic(element.attributes[window[1].0].span()));
                return;
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<Component a="1" b="2" c="3" />"#,
        "<Component />",
        r#"<Component a="1" />"#,
        r#"<Component A="1" b="2" />"#,
    ];

    let fail = vec![r#"<Component z="1" a="2" />"#, r#"<Component c="1" a="2" b="3" />"#];

    Tester::new(SortJsxProps::NAME, SortJsxProps::PLUGIN, pass, fail)
        .change_rule_path_extension("tsx")
        .test_and_snapshot();
}
