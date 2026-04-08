use std::cmp::Ordering;

use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{Expression, Statement, SwitchCase, SwitchStatement},
};
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

fn sort_switch_case_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Switch cases should be sorted alphabetically.").with_label(span)
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
pub struct SortSwitchCaseConfig {
    ignore_case: bool,
    order: SortOrder,
    r#type: SortType,
}

impl Default for SortSwitchCaseConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc, r#type: SortType::Alphabetical }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortSwitchCase(Box<SortSwitchCaseConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted switch cases.
    ///
    /// ### Why is this bad?
    ///
    /// Keeping switch branches in a predictable order makes control flow easier
    /// to scan and reduces noisy diffs when cases are added.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// switch (status) {
    ///   case "pending":
    ///     return 1;
    ///   case "active":
    ///     return 2;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// switch (status) {
    ///   case "active":
    ///     return 2;
    ///   case "pending":
    ///     return 1;
    /// }
    /// ```
    SortSwitchCase,
    oxc,
    style,
    conditional_fix,
    config = SortSwitchCaseConfig
);

impl Rule for SortSwitchCase {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortSwitchCaseConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch_statement) = node.kind() else {
            return;
        };

        if switch_statement.cases.len() < 2
            || self.0.r#type != SortType::Alphabetical
            || matches!(&switch_statement.discriminant, Expression::BooleanLiteral(lit) if lit.value)
        {
            return;
        }

        let groups = build_groups(switch_statement);
        if groups.len() < 2 && groups.iter().all(|group| group.len() < 2) {
            return;
        }

        let mut has_unsorted_labels = false;
        for group in &groups {
            if let Some(span) = first_unsorted_label_span(group, switch_statement, &self.0, ctx) {
                has_unsorted_labels = true;
                ctx.diagnostic(sort_switch_case_diagnostic(span));
            }
            if let Some(span) = misplaced_default_span(group, switch_statement) {
                has_unsorted_labels = true;
                ctx.diagnostic(sort_switch_case_diagnostic(span));
            }
        }

        let sorted_group_indices = sorted_group_indices(&groups, switch_statement, &self.0, ctx);
        let Some(first_unsorted_group_position) = first_unsorted_position(&sorted_group_indices)
        else {
            return;
        };

        let group = &groups[first_unsorted_group_position];
        let diagnostic_span = switch_statement.cases[group[0]].span;
        if !has_unsorted_labels
            && let Some(replacement) =
                build_group_reorder_fix(switch_statement, &groups, &sorted_group_indices, ctx)
        {
            let replace_span = Span::new(
                switch_statement.cases[groups[0][0]].span.start,
                switch_statement.cases[*groups.last().and_then(|group| group.last()).unwrap()]
                    .span
                    .end,
            );
            ctx.diagnostic_with_fix(sort_switch_case_diagnostic(diagnostic_span), |fixer| {
                fixer.replace(replace_span, replacement)
            });
            return;
        }

        ctx.diagnostic(sort_switch_case_diagnostic(diagnostic_span));
    }
}

fn build_groups(switch_statement: &SwitchStatement<'_>) -> Vec<Vec<usize>> {
    let mut groups = vec![Vec::new()];

    for (index, case) in switch_statement.cases.iter().enumerate() {
        groups.last_mut().unwrap().push(index);
        if case_ends_group(case) && index + 1 < switch_statement.cases.len() {
            groups.push(Vec::new());
        }
    }

    groups
}

fn case_ends_group(case: &SwitchCase<'_>) -> bool {
    if case.consequent.is_empty() {
        return false;
    }

    let statements = match case.consequent.first() {
        Some(Statement::BlockStatement(block)) => block.body.as_slice(),
        _ => case.consequent.as_slice(),
    };

    statements.iter().any(|statement| {
        matches!(statement, Statement::BreakStatement(_) | Statement::ReturnStatement(_))
    })
}

fn first_unsorted_position(indices: &[usize]) -> Option<usize> {
    indices
        .iter()
        .enumerate()
        .find_map(|(position, index)| (*index != position).then_some(position))
}

fn misplaced_default_span(group: &[usize], switch_statement: &SwitchStatement<'_>) -> Option<Span> {
    let last_position = group.len().checked_sub(1)?;
    for (position, case_index) in group.iter().enumerate() {
        if switch_statement.cases[*case_index].test.is_none() && position != last_position {
            return Some(switch_statement.cases[*case_index].span);
        }
    }
    None
}

fn first_unsorted_label_span(
    group: &[usize],
    switch_statement: &SwitchStatement<'_>,
    config: &SortSwitchCaseConfig,
    ctx: &LintContext<'_>,
) -> Option<Span> {
    let labels = group
        .iter()
        .filter_map(|case_index| {
            let case = &switch_statement.cases[*case_index];
            let test = case.test.as_ref()?;
            Some((*case_index, normalized_case_name(case, test, config, ctx)))
        })
        .collect::<Vec<_>>();

    let mut sorted = labels.clone();
    sorted.sort_by(|(left_index, left_name), (right_index, right_name)| {
        let ordering = left_name.cmp(right_name);
        let ordering = match config.order {
            SortOrder::Asc => ordering,
            SortOrder::Desc => ordering.reverse(),
        };
        if ordering == Ordering::Equal { left_index.cmp(right_index) } else { ordering }
    });

    labels.iter().zip(sorted.iter()).find_map(|((actual_index, _), (expected_index, _))| {
        (actual_index != expected_index).then_some(switch_statement.cases[*actual_index].span)
    })
}

fn sorted_group_indices(
    groups: &[Vec<usize>],
    switch_statement: &SwitchStatement<'_>,
    config: &SortSwitchCaseConfig,
    ctx: &LintContext<'_>,
) -> Vec<usize> {
    let last_group_should_stay = groups
        .last()
        .is_some_and(|group| !case_ends_group(&switch_statement.cases[*group.last().unwrap()]));

    let mut sortable = groups
        .iter()
        .enumerate()
        .map(|(index, group)| {
            let is_default_group =
                group.iter().all(|case_index| switch_statement.cases[*case_index].test.is_none());
            let key = first_group_key(group, switch_statement, config, ctx);
            (index, key, is_default_group)
        })
        .collect::<Vec<_>>();

    sortable.sort_by(
        |(left_index, left_key, left_default), (right_index, right_key, right_default)| {
            if last_group_should_stay {
                let last_index = groups.len() - 1;
                if *left_index == last_index {
                    return Ordering::Greater;
                }
                if *right_index == last_index {
                    return Ordering::Less;
                }
            }
            if *left_default {
                return Ordering::Greater;
            }
            if *right_default {
                return Ordering::Less;
            }
            let ordering = left_key.cmp(right_key);
            let ordering = match config.order {
                SortOrder::Asc => ordering,
                SortOrder::Desc => ordering.reverse(),
            };
            if ordering == Ordering::Equal { left_index.cmp(right_index) } else { ordering }
        },
    );

    sortable.into_iter().map(|(index, _, _)| index).collect()
}

fn first_group_key(
    group: &[usize],
    switch_statement: &SwitchStatement<'_>,
    config: &SortSwitchCaseConfig,
    ctx: &LintContext<'_>,
) -> String {
    group
        .iter()
        .find_map(|case_index| {
            let case = &switch_statement.cases[*case_index];
            let test = case.test.as_ref()?;
            Some(normalized_case_name(case, test, config, ctx))
        })
        .unwrap_or_else(|| "default".to_string())
}

fn normalized_case_name(
    case: &SwitchCase<'_>,
    test: &Expression<'_>,
    config: &SortSwitchCaseConfig,
    ctx: &LintContext<'_>,
) -> String {
    let raw = ctx.source_range(test.span()).trim();
    let raw = if raw.is_empty() { ctx.source_range(case.span()).trim() } else { raw };
    if config.ignore_case { raw.cow_to_ascii_lowercase().into_owned() } else { raw.to_string() }
}

fn build_group_reorder_fix(
    switch_statement: &SwitchStatement<'_>,
    groups: &[Vec<usize>],
    sorted_group_indices: &[usize],
    ctx: &LintContext<'_>,
) -> Option<String> {
    let first_case_index = groups.first()?.first()?;
    let last_case_index = groups.last()?.last()?;
    let full_span = Span::new(
        switch_statement.cases[*first_case_index].span.start,
        switch_statement.cases[*last_case_index].span.end,
    );
    if ctx.has_comments_between(full_span) {
        return None;
    }

    for window in groups.windows(2) {
        let left = &switch_statement.cases[*window[0].last().unwrap()];
        let right = &switch_statement.cases[window[1][0]];
        if ctx.has_comments_between(Span::new(left.span.end, right.span.start)) {
            return None;
        }
    }

    let group_texts = groups
        .iter()
        .map(|group| {
            let start = switch_statement.cases[group[0]].span.start;
            let end = switch_statement.cases[*group.last().unwrap()].span.end;
            ctx.source_range(Span::new(start, end)).to_string()
        })
        .collect::<Vec<_>>();
    let separators = groups
        .windows(2)
        .map(|window| {
            let left = &switch_statement.cases[*window[0].last().unwrap()];
            let right = &switch_statement.cases[window[1][0]];
            ctx.source_range(Span::new(left.span.end, right.span.start)).to_string()
        })
        .collect::<Vec<_>>();

    let mut replacement = String::new();
    for (position, group_index) in sorted_group_indices.iter().enumerate() {
        replacement.push_str(&group_texts[*group_index]);
        if position < separators.len() {
            replacement.push_str(&separators[position]);
        }
    }
    Some(replacement)
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("switch (status) { case 'active': return 1; case 'pending': return 2; }", None),
        ("switch (status) { case 'Active': return 1; case 'pending': return 2; }", None),
        ("switch (true) { case a: return 1; case b: return 2; }", None),
        (
            "switch (status) { case 'archived': case 'pending': return 1; case 'zebra': return 2; }",
            None,
        ),
        ("switch (status) { case 'pending': return 1; default: return 2; }", None),
        (
            "switch (status) { case 'pending': return 1; case 'archived': return 2; }",
            Some(json!([{ "order": "desc" }])),
        ),
        (
            "switch (status) { case 'Active': return 1; case 'pending': return 2; }",
            Some(json!([{ "ignoreCase": false }])),
        ),
    ];

    let fail = vec![
        ("switch (status) { case 'pending': return 1; case 'active': return 2; }", None),
        (
            "switch (status) { case 'pending': return 1; case 'archived': return 2; case 'zebra': return 3; }",
            None,
        ),
        (
            "switch (status) { case 'pending': return 1; default: return 2; case 'zebra': return 3; }",
            None,
        ),
        (
            "switch (status) { case 'pending': case 'archived': return 1; case 'zebra': return 2; }",
            None,
        ),
        ("switch (status) { case 'b': case 'a': return 1; case 'z': return 2; }", None),
        (
            "switch (status) { case 'active': return 1; case 'pending': return 2; }",
            Some(json!([{ "order": "desc" }])),
        ),
        (
            "switch (status) { case 'pending': return 1; case 'Active': return 2; }",
            Some(json!([{ "ignoreCase": false }])),
        ),
    ];

    let fix = vec![
        (
            "switch (status) { case 'pending': return 1; case 'active': return 2; }",
            "switch (status) { case 'active': return 2; case 'pending': return 1; }",
            None,
        ),
        (
            "switch (status) { case 'pending': return 1; case 'archived': return 2; case 'zebra': return 3; }",
            "switch (status) { case 'archived': return 2; case 'pending': return 1; case 'zebra': return 3; }",
            None,
        ),
        (
            "switch (status) { case 'active': return 1; case 'pending': return 2; }",
            "switch (status) { case 'pending': return 2; case 'active': return 1; }",
            Some(json!([{ "order": "desc" }])),
        ),
        (
            "switch (status) { case 'pending': return 1; case 'Active': return 2; }",
            "switch (status) { case 'Active': return 2; case 'pending': return 1; }",
            Some(json!([{ "ignoreCase": false }])),
        ),
    ];

    Tester::new(SortSwitchCase::NAME, SortSwitchCase::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
