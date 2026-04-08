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

fn sort_named_exports_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Named export specifiers should be sorted alphabetically.").with_label(span)
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
pub struct SortNamedExportsConfig {
    ignore_case: bool,
    order: SortOrder,
}

impl Default for SortNamedExportsConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortNamedExports(Box<SortNamedExportsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted named export specifiers.
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted named exports are harder to scan and lead to unnecessary merge
    /// conflicts when multiple developers add exports.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export { z, a, m };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// export { a, m, z };
    /// ```
    SortNamedExports,
    oxc,
    style,
    conditional_fix,
    config = SortNamedExportsConfig
);

impl Rule for SortNamedExports {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortNamedExportsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportNamedDeclaration(export) = node.kind() else {
            return;
        };

        if export.specifiers.len() < 2 {
            return;
        }

        let names: Vec<(usize, String)> = export
            .specifiers
            .iter()
            .enumerate()
            .map(|(i, spec)| {
                let name = spec.local.name().as_str().to_string();
                (i, name)
            })
            .collect();

        for window in names.windows(2) {
            let a_key = if self.0.ignore_case {
                window[0].1.cow_to_ascii_lowercase().into_owned()
            } else {
                window[0].1.clone()
            };
            let b_key = if self.0.ignore_case {
                window[1].1.cow_to_ascii_lowercase().into_owned()
            } else {
                window[1].1.clone()
            };

            let ord = a_key.cmp(&b_key);
            let ord = match self.0.order {
                SortOrder::Asc => ord,
                SortOrder::Desc => ord.reverse(),
            };

            if ord == Ordering::Greater {
                let span = export.specifiers[window[1].0].span();
                if let Some(fix_text) = self.build_fix(export, &names, ctx) {
                    ctx.diagnostic_with_fix(sort_named_exports_diagnostic(span), |fixer| {
                        fixer.replace(export.span, fix_text)
                    });
                } else {
                    ctx.diagnostic(sort_named_exports_diagnostic(span));
                }
                return;
            }
        }
    }
}

impl SortNamedExports {
    fn build_fix(
        &self,
        export: &oxc_ast::ast::ExportNamedDeclaration<'_>,
        names: &[(usize, String)],
        ctx: &LintContext<'_>,
    ) -> Option<String> {
        if ctx.has_comments_between(export.span) {
            return None;
        }

        let mut sorted: Vec<&str> = names.iter().map(|(_, name)| name.as_str()).collect();
        sorted.sort_by(|a, b| {
            let ak = if self.0.ignore_case {
                a.cow_to_ascii_lowercase().into_owned()
            } else {
                a.to_string()
            };
            let bk = if self.0.ignore_case {
                b.cow_to_ascii_lowercase().into_owned()
            } else {
                b.to_string()
            };
            let ord = ak.cmp(&bk);
            match self.0.order {
                SortOrder::Asc => ord,
                SortOrder::Desc => ord.reverse(),
            }
        });

        let specifier_text = sorted.join(", ");
        if let Some(source) = &export.source {
            let source_text = ctx.source_range(source.span());
            Some(format!("export {{ {specifier_text} }} from {source_text}"))
        } else {
            Some(format!("export {{ {specifier_text} }}"))
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass =
        vec!["export { a, b, c };", "export { a };", "const a = 1; const b = 2; export { a, b };"];

    let fail = vec!["export { z, a, m };", "export { b, a };"];

    let fix = vec![
        ("export { z, a, m };", "export { a, m, z }", None),
        ("export { b, a };", "export { a, b }", None),
    ];

    Tester::new(SortNamedExports::NAME, SortNamedExports::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
