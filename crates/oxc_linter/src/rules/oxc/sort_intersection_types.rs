use std::cmp::Ordering;

use cow_utils::CowUtils;
use oxc_ast::{AstKind, ast::TSIntersectionType};
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

fn sort_intersection_types_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Intersection types should be sorted alphabetically.").with_label(span)
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
pub struct SortIntersectionTypesConfig {
    ignore_case: bool,
    order: SortOrder,
}

impl Default for SortIntersectionTypesConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortIntersectionTypes(Box<SortIntersectionTypesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted TypeScript intersection type members.
    ///
    /// ### Why is this bad?
    ///
    /// Keeping intersection type members sorted makes type declarations
    /// easier to scan, diff, and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Combined = Serializable & Comparable & Addressable;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Combined = Addressable & Comparable & Serializable;
    /// ```
    SortIntersectionTypes,
    oxc,
    style,
    conditional_fix,
    config = SortIntersectionTypesConfig
);

impl Rule for SortIntersectionTypes {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortIntersectionTypesConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSIntersectionType(intersection) = node.kind() else {
            return;
        };

        if intersection.types.len() < 2 {
            return;
        }

        let indices = sorted_type_indices(intersection, ctx, &self.0);
        let Some(first_unsorted) = first_unsorted_position(&indices) else {
            return;
        };

        let diagnostic_span = intersection.types[first_unsorted].span();
        if let Some(replacement) = build_fix(intersection, ctx, &indices) {
            ctx.diagnostic_with_fix(sort_intersection_types_diagnostic(diagnostic_span), |fixer| {
                fixer.replace(intersection.span, replacement)
            });
            return;
        }
        ctx.diagnostic(sort_intersection_types_diagnostic(diagnostic_span));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn first_unsorted_position(indices: &[usize]) -> Option<usize> {
    indices
        .iter()
        .enumerate()
        .find_map(|(position, index)| (*index != position).then_some(position))
}

fn sorted_type_indices(
    intersection: &TSIntersectionType<'_>,
    ctx: &LintContext<'_>,
    config: &SortIntersectionTypesConfig,
) -> Vec<usize> {
    let mut members: Vec<(usize, String)> = intersection
        .types
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            let raw = ctx.source_range(ty.span());
            let key = if config.ignore_case {
                raw.split_whitespace().collect::<String>().cow_to_ascii_lowercase().into_owned()
            } else {
                raw.split_whitespace().collect::<String>()
            };
            (index, key)
        })
        .collect();

    members.sort_by(|(ai, ak), (bi, bk)| {
        let ord = ak.cmp(bk);
        let ord = match config.order {
            SortOrder::Asc => ord,
            SortOrder::Desc => ord.reverse(),
        };
        if ord == Ordering::Equal { ai.cmp(bi) } else { ord }
    });

    members.into_iter().map(|(index, _)| index).collect()
}

fn build_fix(
    intersection: &TSIntersectionType<'_>,
    ctx: &LintContext<'_>,
    indices: &[usize],
) -> Option<String> {
    if ctx.has_comments_between(intersection.span) {
        return None;
    }

    let pieces: Vec<&str> = indices
        .iter()
        .map(|&index| ctx.source_range(intersection.types[index].span()).trim())
        .collect();

    Some(pieces.join(" & "))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["type T = A & B & C;", "type T = A;"];

    let fail = vec!["type T = C & A & B;", "type T = Z & A;"];

    let fix = vec![
        ("type T = C & A & B;", "type T = A & B & C;", None),
        ("type T = Z & A;", "type T = A & Z;", None),
    ];

    Tester::new(SortIntersectionTypes::NAME, SortIntersectionTypes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
