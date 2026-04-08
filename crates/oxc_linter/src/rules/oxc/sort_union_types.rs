use std::cmp::Ordering;

use cow_utils::CowUtils;
use oxc_ast::{AstKind, ast::TSUnionType};
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

fn sort_union_types_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Union types should be sorted alphabetically.").with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Desc,
    #[default]
    Asc,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortType {
    #[default]
    Alphabetical,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SortUnionTypesConfig {
    ignore_case: bool,
    order: SortOrder,
    r#type: SortType,
}

impl Default for SortUnionTypesConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc, r#type: SortType::Alphabetical }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortUnionTypes(Box<SortUnionTypesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted TypeScript union members.
    ///
    /// ### Why is this bad?
    ///
    /// Keeping union members in a predictable order makes type declarations
    /// easier to scan, diff, and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Status = "pending" | "active" | "archived";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Status = "active" | "archived" | "pending";
    /// ```
    SortUnionTypes,
    oxc,
    style,
    conditional_fix,
    config = SortUnionTypesConfig
);

impl Rule for SortUnionTypes {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortUnionTypesConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSUnionType(union_type) = node.kind() else {
            return;
        };

        if union_type.types.len() < 2 || self.0.r#type != SortType::Alphabetical {
            return;
        }

        let indices = sorted_type_indices(union_type, ctx, &self.0);
        let Some(first_unsorted_index) = first_unsorted_position(&indices) else {
            return;
        };

        let diagnostic_span = union_type.types[first_unsorted_index].span();
        if let Some(replacement) = build_fix(union_type, ctx, &indices) {
            ctx.diagnostic_with_fix(sort_union_types_diagnostic(diagnostic_span), |fixer| {
                fixer.replace(union_type.span, replacement)
            });
            return;
        }

        ctx.diagnostic(sort_union_types_diagnostic(diagnostic_span));
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
    union_type: &TSUnionType<'_>,
    ctx: &LintContext<'_>,
    config: &SortUnionTypesConfig,
) -> Vec<usize> {
    let mut members = union_type
        .types
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            let raw = ctx.source_range(ty.span());
            let key = normalized_sort_key(raw, config.ignore_case);
            (index, key)
        })
        .collect::<Vec<_>>();

    members.sort_by(|(left_index, left_key), (right_index, right_key)| {
        let ordering = left_key.cmp(right_key);
        let ordering = match config.order {
            SortOrder::Asc => ordering,
            SortOrder::Desc => ordering.reverse(),
        };
        if ordering == Ordering::Equal { left_index.cmp(right_index) } else { ordering }
    });

    members.into_iter().map(|(index, _)| index).collect()
}

fn normalized_sort_key(raw: &str, ignore_case: bool) -> String {
    let compact = raw.split_whitespace().collect::<String>();
    if ignore_case { compact.cow_to_ascii_lowercase().into_owned() } else { compact }
}

fn build_fix(
    union_type: &TSUnionType<'_>,
    ctx: &LintContext<'_>,
    indices: &[usize],
) -> Option<String> {
    if ctx.has_comments_between(union_type.span) {
        return None;
    }

    for window in union_type.types.windows(2) {
        let between = Span::new(window[0].span().end, window[1].span().start);
        if ctx.has_comments_between(between) {
            return None;
        }
    }

    let pieces = indices
        .iter()
        .map(|&index| ctx.source_range(union_type.types[index].span()).trim().to_string())
        .collect::<Vec<_>>();

    let union_text = ctx.source_range(union_type.span);
    let replacement = if union_text.contains('\n') && union_text.trim_start().starts_with('|') {
        build_vertical_union_fix(&pieces, union_text, ctx.source_text(), union_type.span)
    } else {
        pieces.join(" | ")
    };

    Some(replacement)
}

fn build_vertical_union_fix(
    pieces: &[String],
    union_text: &str,
    source_text: &str,
    span: Span,
) -> String {
    let line_start = source_text[..span.start as usize].rfind('\n').map_or(0, |index| index + 1);
    let indent = &source_text[line_start..span.start as usize];
    if !indent.chars().all(char::is_whitespace) {
        return pieces.join(" | ");
    }

    let trailing_newline = union_text.ends_with('\n');
    let mut replacement = String::new();
    for (index, piece) in pieces.iter().enumerate() {
        if index > 0 {
            replacement.push('\n');
            replacement.push_str(indent);
        }
        replacement.push_str("| ");
        replacement.push_str(piece);
    }
    if trailing_newline {
        replacement.push('\n');
    }
    replacement
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("type Status = 'active' | 'archived' | 'pending';", None),
        ("type Status = 'Active' | 'archived' | 'pending';", None),
        ("type Result = Error | Promise<string> | string;", None),
        ("type Status =\n  | 'active'\n  | 'archived'\n  | 'pending';", None),
        (
            "type Status = 'Active' | 'archived' | 'pending';",
            Some(json!([{ "ignoreCase": false }])),
        ),
    ];

    let fail = vec![
        ("type Status = 'pending' | 'active' | 'archived';", None),
        ("type Result = string | Error | Promise<string>;", None),
        ("type Status =\n  | 'pending'\n  | 'active'\n  | 'archived';", None),
        ("type Status = 'active' | 'archived' | 'pending';", Some(json!([{ "order": "desc" }]))),
        (
            "type Status = 'pending' | 'archived' | 'Active';",
            Some(json!([{ "ignoreCase": false }])),
        ),
    ];

    let fix = vec![
        (
            "type Status = 'pending' | 'active' | 'archived';",
            "type Status = 'active' | 'archived' | 'pending';",
            None,
        ),
        (
            "type Result = string | Error | Promise<string>;",
            "type Result = Error | Promise<string> | string;",
            None,
        ),
        (
            "type Status =\n  | 'pending'\n  | 'active'\n  | 'archived';",
            "type Status =\n  | 'active'\n  | 'archived'\n  | 'pending';",
            None,
        ),
        (
            "type Status = 'active' | 'archived' | 'pending';",
            "type Status = 'pending' | 'archived' | 'active';",
            Some(json!([{ "order": "desc" }])),
        ),
        (
            "type Status = 'pending' | 'archived' | 'Active';",
            "type Status = 'Active' | 'archived' | 'pending';",
            Some(json!([{ "ignoreCase": false }])),
        ),
    ];

    Tester::new(SortUnionTypes::NAME, SortUnionTypes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
