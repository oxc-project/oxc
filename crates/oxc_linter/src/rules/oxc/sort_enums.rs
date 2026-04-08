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

fn sort_enums_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enum members should be sorted alphabetically.").with_label(span)
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
pub struct SortEnumsConfig {
    ignore_case: bool,
    order: SortOrder,
}

impl Default for SortEnumsConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortEnums(Box<SortEnumsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted members in TypeScript enums.
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted enum members make enums harder to scan and maintain.
    /// Consistent ordering reduces merge conflicts and improves readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// enum Color { Red, Green, Blue }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// enum Color { Blue, Green, Red }
    /// ```
    SortEnums,
    oxc,
    style,
    none,
    config = SortEnumsConfig
);

impl Rule for SortEnums {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortEnumsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSEnumDeclaration(enum_decl) = node.kind() else {
            return;
        };

        if enum_decl.body.members.len() < 2 {
            return;
        }

        let members: Vec<(usize, String)> = enum_decl
            .body
            .members
            .iter()
            .enumerate()
            .map(|(i, member)| {
                let key = match &member.id {
                    oxc_ast::ast::TSEnumMemberName::Identifier(ident) => ident.name.to_string(),
                    oxc_ast::ast::TSEnumMemberName::String(lit) => lit.value.to_string(),
                    _ => ctx.source_range(member.id.span()).to_string(),
                };
                (i, key)
            })
            .collect();

        let sorted = sorted_indices(&members, &self.0);
        if let Some(first_unsorted) = first_unsorted_position(&sorted) {
            let (orig_idx, _) = &members[first_unsorted];
            ctx.diagnostic(sort_enums_diagnostic(enum_decl.body.members[*orig_idx].span()));
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn sorted_indices(members: &[(usize, String)], config: &SortEnumsConfig) -> Vec<usize> {
    let mut entries: Vec<(usize, String)> = members
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
        "enum Color { Blue, Green, Red }",
        "enum Single { A }",
        "enum Empty {}",
        "enum Status { Active = 'active', Inactive = 'inactive', Pending = 'pending' }",
    ];

    let fail = vec!["enum Color { Red, Green, Blue }", "enum Status { Pending, Active, Inactive }"];

    Tester::new(SortEnums::NAME, SortEnums::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
