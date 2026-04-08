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

fn sort_object_types_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("TypeScript object type members should be sorted alphabetically.")
        .with_label(span)
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
pub struct SortObjectTypesConfig {
    ignore_case: bool,
    order: SortOrder,
}

impl Default for SortObjectTypesConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortObjectTypes(Box<SortObjectTypesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted members in TypeScript object type literals.
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted type members make type definitions harder to scan and diff.
    /// Consistent ordering improves readability and reduces merge conflicts.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type User = { name: string; age: number; email: string };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type User = { age: number; email: string; name: string };
    /// ```
    SortObjectTypes,
    oxc,
    style,
    none,
    config = SortObjectTypesConfig
);

fn member_key(member: &oxc_ast::ast::TSSignature<'_>, ctx: &LintContext<'_>) -> Option<String> {
    match member {
        oxc_ast::ast::TSSignature::TSPropertySignature(prop) => match &prop.key {
            oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
            oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
            _ => Some(ctx.source_range(prop.key.span()).to_string()),
        },
        oxc_ast::ast::TSSignature::TSMethodSignature(method) => match &method.key {
            oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
            _ => Some(ctx.source_range(method.key.span()).to_string()),
        },
        _ => None,
    }
}

impl Rule for SortObjectTypes {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortObjectTypesConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSTypeLiteral(type_lit) = node.kind() else {
            return;
        };

        if type_lit.members.len() < 2 {
            return;
        }

        let keyed: Vec<(usize, String)> = type_lit
            .members
            .iter()
            .enumerate()
            .filter_map(|(i, member)| {
                let key = member_key(member, ctx)?;
                Some((i, key))
            })
            .collect();

        if keyed.len() < 2 {
            return;
        }

        let sorted = sorted_indices(&keyed, &self.0);
        if let Some(first_unsorted) = first_unsorted_position(&sorted) {
            let (orig_idx, _) = &keyed[first_unsorted];
            ctx.diagnostic(sort_object_types_diagnostic(type_lit.members[*orig_idx].span()));
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn sorted_indices(keyed: &[(usize, String)], config: &SortObjectTypesConfig) -> Vec<usize> {
    let mut entries: Vec<(usize, String)> = keyed
        .iter()
        .enumerate()
        .map(|(pos, (_, key))| {
            let k = if config.ignore_case {
                key.cow_to_ascii_lowercase().into_owned()
            } else {
                key.clone()
            };
            (pos, k)
        })
        .collect();

    entries.sort_by(|(ai, ak), (bi, bk)| {
        let ord = ak.cmp(bk);
        let ord = match config.order {
            SortOrder::Asc => ord,
            SortOrder::Desc => ord.reverse(),
        };
        if ord == Ordering::Equal { ai.cmp(bi) } else { ord }
    });

    entries.into_iter().map(|(pos, _)| pos).collect()
}

fn first_unsorted_position(indices: &[usize]) -> Option<usize> {
    indices
        .iter()
        .enumerate()
        .find_map(|(position, index)| (*index != position).then_some(position))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "type User = { age: number; email: string; name: string };",
        "type Empty = {};",
        "type Single = { a: number };",
    ];

    let fail = vec![
        "type User = { name: string; age: number; email: string };",
        "type Obj = { z: number; a: string };",
    ];

    Tester::new(SortObjectTypes::NAME, SortObjectTypes::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
