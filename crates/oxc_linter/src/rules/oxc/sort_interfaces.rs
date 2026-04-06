use std::cmp::Ordering;

use oxc_ast::{
    AstKind,
    ast::{TSInterfaceDeclaration, TSSignature},
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

fn sort_interfaces_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Interface members should be sorted alphabetically.").with_label(span)
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
pub struct SortInterfacesConfig {
    ignore_case: bool,
    order: SortOrder,
    r#type: SortType,
}

impl Default for SortInterfacesConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc, r#type: SortType::Alphabetical }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortInterfaces(Box<SortInterfacesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted interface members.
    ///
    /// ### Why is this bad?
    ///
    /// Sorting interface members makes shared contracts easier to scan and
    /// reduces noisy diffs when members are added over time.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// interface User {
    ///   name: string;
    ///   id: string;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// interface User {
    ///   id: string;
    ///   name: string;
    /// }
    /// ```
    SortInterfaces,
    oxc,
    style,
    conditional_fix,
    config = SortInterfacesConfig
);

impl Rule for SortInterfaces {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortInterfacesConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSInterfaceDeclaration(interface_decl) = node.kind() else {
            return;
        };

        if interface_decl.body.body.len() < 2 || self.0.r#type != SortType::Alphabetical {
            return;
        }

        let Some(indices) = sorted_member_indices(interface_decl, ctx, &self.0) else {
            return;
        };
        let Some(first_unsorted_index) = first_unsorted_position(&indices) else {
            return;
        };

        let diagnostic_span = interface_decl.body.body[first_unsorted_index].span();
        if let Some(replacement) = build_fix(interface_decl, ctx, &indices) {
            let replace_span = Span::new(
                interface_decl.body.body[0].span().start,
                interface_decl.body.body[interface_decl.body.body.len() - 1].span().end,
            );
            ctx.diagnostic_with_fix(sort_interfaces_diagnostic(diagnostic_span), |fixer| {
                fixer.replace(replace_span, replacement)
            });
            return;
        }

        ctx.diagnostic(sort_interfaces_diagnostic(diagnostic_span));
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

fn sorted_member_indices(
    interface_decl: &TSInterfaceDeclaration<'_>,
    _ctx: &LintContext<'_>,
    config: &SortInterfacesConfig,
) -> Option<Vec<usize>> {
    let mut members = interface_decl
        .body
        .body
        .iter()
        .enumerate()
        .map(|(index, member)| {
            let raw_name = member_sort_name(member)?;
            let key = if config.ignore_case { raw_name.to_ascii_lowercase() } else { raw_name };
            Some((index, key))
        })
        .collect::<Option<Vec<_>>>()?;

    members.sort_by(|(left_index, left_key), (right_index, right_key)| {
        let ordering = left_key.cmp(right_key);
        let ordering = match config.order {
            SortOrder::Asc => ordering,
            SortOrder::Desc => ordering.reverse(),
        };
        if ordering == Ordering::Equal { left_index.cmp(right_index) } else { ordering }
    });

    Some(members.into_iter().map(|(index, _)| index).collect())
}

fn member_sort_name(member: &TSSignature<'_>) -> Option<String> {
    match member {
        TSSignature::TSPropertySignature(signature) => {
            Some(signature.key.static_name()?.into_owned())
        }
        TSSignature::TSMethodSignature(signature) => {
            Some(signature.key.static_name()?.into_owned())
        }
        _ => None,
    }
}

fn build_fix(
    interface_decl: &TSInterfaceDeclaration<'_>,
    ctx: &LintContext<'_>,
    indices: &[usize],
) -> Option<String> {
    let members = interface_decl.body.body.as_slice();
    let first = members.first()?;
    let last = members.last()?;
    let full_span = Span::new(first.span().start, last.span().end);
    if ctx.has_comments_between(full_span) {
        return None;
    }

    for window in members.windows(2) {
        let between = Span::new(window[0].span().end, window[1].span().start);
        if ctx.has_comments_between(between) {
            return None;
        }
    }

    let texts = members
        .iter()
        .map(|member| ctx.source_range(member.span()).to_string())
        .collect::<Vec<_>>();
    let separators = members
        .windows(2)
        .map(|window| {
            ctx.source_range(Span::new(window[0].span().end, window[1].span().start)).to_string()
        })
        .collect::<Vec<_>>();

    let mut replacement = String::new();
    for (position, index) in indices.iter().enumerate() {
        replacement.push_str(&texts[*index]);
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
        ("interface User {\n  id: string;\n  name: string;\n}", None),
        ("interface User {\n  Id: string;\n  name: string;\n}", None),
        (
            "interface User {\n  Id: string;\n  name: string;\n}",
            Some(json!([{ "ignoreCase": false }])),
        ),
        ("interface User {\n  getId(): string;\n  name: string;\n}", None),
        ("interface User {\n  [key: string]: string;\n  name: string;\n}", None),
    ];

    let fail = vec![
        ("interface User {\n  name: string;\n  id: string;\n}", None),
        ("interface User {\n  name: string;\n  getId(): string;\n}", None),
        (
            "interface User {\n  id: string;\n  Id: string;\n}",
            Some(json!([{ "ignoreCase": false }])),
        ),
        (
            "interface User {\n  getId(): string;\n  id: string;\n}",
            Some(json!([{ "order": "desc" }])),
        ),
    ];

    let fix = vec![
        (
            "interface User {\n  name: string;\n  id: string;\n}",
            "interface User {\n  id: string;\n  name: string;\n}",
            None,
        ),
        (
            "interface User {\n  name: string;\n  getId(): string;\n}",
            "interface User {\n  getId(): string;\n  name: string;\n}",
            None,
        ),
        (
            "interface User {\n  id: string;\n  Id: string;\n}",
            "interface User {\n  Id: string;\n  id: string;\n}",
            Some(json!([{ "ignoreCase": false }])),
        ),
        (
            "interface User {\n  getId(): string;\n  id: string;\n}",
            "interface User {\n  id: string;\n  getId(): string;\n}",
            Some(json!([{ "order": "desc" }])),
        ),
    ];

    Tester::new(SortInterfaces::NAME, SortInterfaces::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
