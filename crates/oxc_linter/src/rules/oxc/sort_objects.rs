use std::cmp::Ordering;

use cow_utils::CowUtils;
use oxc_ast::{AstKind, ast::ObjectExpression};
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

fn sort_objects_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Object keys should be sorted alphabetically.").with_label(span)
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
pub struct SortObjectsConfig {
    ignore_case: bool,
    order: SortOrder,
}

impl Default for SortObjectsConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortObjects(Box<SortObjectsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted keys in object literals.
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted object keys make objects harder to scan, diff, and maintain.
    /// Consistent ordering reduces merge conflicts and improves readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const obj = { z: 1, a: 2, m: 3 };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const obj = { a: 2, m: 3, z: 1 };
    /// ```
    SortObjects,
    oxc,
    style,
    conditional_fix,
    config = SortObjectsConfig
);

impl Rule for SortObjects {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortObjectsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj) = node.kind() else {
            return;
        };

        if obj.properties.len() < 2 {
            return;
        }

        // Collect only keyed properties (skip spread elements)
        let keyed: Vec<(usize, String)> = obj
            .properties
            .iter()
            .enumerate()
            .filter_map(|(i, prop)| {
                let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) = prop else {
                    return None;
                };
                let key = property_key_text(&p.key, ctx)?;
                Some((i, key))
            })
            .collect();

        if keyed.len() < 2 {
            return;
        }

        let sorted_keys = sorted_indices(&keyed, &self.0);
        let Some(first_unsorted) = first_unsorted_position(&sorted_keys) else {
            return;
        };

        let (orig_idx, _) = &keyed[first_unsorted];
        let span = obj.properties[*orig_idx].span();
        if let Some(replacement) = build_fix(obj, ctx, &keyed, &sorted_keys) {
            ctx.diagnostic_with_fix(sort_objects_diagnostic(span), |fixer| {
                fixer.replace(obj.span, replacement)
            });
            return;
        }
        ctx.diagnostic(sort_objects_diagnostic(span));
    }
}

fn property_key_text(key: &oxc_ast::ast::PropertyKey<'_>, ctx: &LintContext<'_>) -> Option<String> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
        oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
        oxc_ast::ast::PropertyKey::NumericLiteral(lit) => lit.raw.as_ref().map(ToString::to_string),
        _ => {
            // Computed keys — use source text as fallback
            Some(ctx.source_range(key.span()).to_string())
        }
    }
}

fn sorted_indices(keyed: &[(usize, String)], config: &SortObjectsConfig) -> Vec<usize> {
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

fn build_fix(
    obj: &ObjectExpression<'_>,
    ctx: &LintContext<'_>,
    keyed: &[(usize, String)],
    sorted_keys: &[usize],
) -> Option<String> {
    if ctx.has_comments_between(obj.span) {
        return None;
    }

    // Only fix if all properties are keyed (no spreads)
    if keyed.len() != obj.properties.len() {
        return None;
    }

    let pieces: Vec<&str> = sorted_keys
        .iter()
        .map(|&pos| {
            let (orig_idx, _) = &keyed[pos];
            ctx.source_range(obj.properties[*orig_idx].span()).trim()
        })
        .collect();

    let obj_text = ctx.source_range(obj.span);
    let is_multiline = obj_text.contains('\n');

    if is_multiline {
        // Preserve multiline structure
        let indent = detect_indent(obj_text);
        let mut result = String::from("{\n");
        for (i, piece) in pieces.iter().enumerate() {
            result.push_str(&indent);
            result.push_str(piece);
            if i < pieces.len() - 1 || obj_text.contains(",\n}") || obj_text.contains(",\n }") {
                result.push(',');
            }
            result.push('\n');
        }
        result.push('}');
        Some(result)
    } else {
        let inner = pieces.join(", ");
        Some(format!("{{ {inner} }}"))
    }
}

fn detect_indent(text: &str) -> String {
    for line in text.lines().skip(1) {
        if let Some(first_non_ws) = line.find(|c: char| !c.is_whitespace())
            && first_non_ws > 0
        {
            return line[..first_non_ws].to_string();
        }
    }
    "  ".to_string()
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("const obj = { a: 1, b: 2, c: 3 };", None),
        ("const obj = { };", None),
        ("const obj = { a: 1 };", None),
        ("const obj = { A: 1, b: 2 };", None),
    ];

    let fail = vec![
        ("const obj = { z: 1, a: 2 };", None),
        ("const obj = { b: 1, a: 2, c: 3 };", None),
        ("const obj = { a: 1, b: 2 };", Some(json!([{ "order": "desc" }]))),
    ];

    let fix = vec![
        ("const obj = { z: 1, a: 2 };", "const obj = { a: 2, z: 1 };", None),
        ("const obj = { b: 1, a: 2, c: 3 };", "const obj = { a: 2, b: 1, c: 3 };", None),
    ];

    Tester::new(SortObjects::NAME, SortObjects::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
