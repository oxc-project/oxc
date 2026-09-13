use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{Expression, ExpressionStatement, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::DefaultRuleConfig,
    utils::{PossibleJestNode, iter_possible_jest_call_node, parse_expect_jest_fn_call},
};

fn no_snapshot(line_count: u32, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Snapshot is too long.")
        .with_help(format!(
            "Expected to not encounter a Jest or Vitest snapshot but one was found that is {line_count} lines long"
        ))
        .with_label(span)
}

fn too_long_snapshot(line_limit: u32, line_count: u32, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Snapshot is too long.")
        .with_help(format!(
            "Expected Jest or Vitest snapshot to be no longer than {line_limit} lines but it was {line_count} lines long"
        ))
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Disallow large snapshots.

### Why is this bad?

When using Jest's snapshot capability one should be mindful of the size of
created snapshots. As a general best practice snapshots should be limited in
size in order to be more manageable and reviewable. A stored snapshot is only as
good as its review and as such keeping it short, sweet, and readable is
important to allow for thorough reviews.

### Examples

Examples of **incorrect** code for this rule:
```javascript
exports[`a large snapshot 1`] = `
line 1
line 2
line 3
line 4
line 5
line 6
line 7
line 8
line 9
line 10
line 11
line 12
line 13
line 14
line 15
line 16
line 17
line 18
line 19
line 20
line 21
line 22
line 23
line 24
line 25
line 26
line 27
line 28
line 29
line 30
line 31
line 32
line 33
line 34
line 35
line 36
line 37
line 38
line 39
line 40
line 41
line 42
line 43
line 44
line 45
line 46
line 47
line 48
line 49
line 50
line 51
`;
```

Examples of **correct** code for this rule:
```js
exports[`a more manageable and readable snapshot 1`] = `
line 1
line 2
line 3
line 4
`;
```
";

#[derive(Debug, Clone)]
enum AllowedSnapshotMatcher {
    Pattern(Regex),
    Exact(CompactStr),
}

impl AllowedSnapshotMatcher {
    fn new(pattern: CompactStr) -> Self {
        Regex::new(&pattern).map_or(Self::Exact(pattern), Self::Pattern)
    }

    fn is_match(&self, snapshot_name: &str) -> bool {
        match self {
            Self::Pattern(pattern) => pattern.is_match(snapshot_name),
            Self::Exact(name) => name == snapshot_name,
        }
    }
}

impl<'de> Deserialize<'de> for AllowedSnapshotMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        CompactStr::deserialize(deserializer).map(Self::new)
    }
}

impl Serialize for AllowedSnapshotMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Pattern(pattern) => serializer.serialize_str(pattern.as_str()),
            Self::Exact(name) => serializer.serialize_str(name),
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoLargeSnapshotsConfig {
    /// Maximum number of lines allowed for external snapshot files.
    pub max_size: u32,
    /// Maximum number of lines allowed for inline snapshots.
    inline_max_size: Option<u32>,
    /// A map of snapshot file paths to arrays of snapshot names that are allowed to exceed the size limit.
    /// Each snapshot name is interpreted as a Rust regular expression. If it is not a valid regular
    /// expression, it is matched as an exact literal string instead.
    #[schemars(with = "FxHashMap<CompactStr, Vec<CompactStr>>")]
    allowed_snapshots: FxHashMap<CompactStr, Vec<AllowedSnapshotMatcher>>,
}

impl Default for NoLargeSnapshotsConfig {
    fn default() -> Self {
        Self { max_size: 50, inline_max_size: None, allowed_snapshots: FxHashMap::default() }
    }
}

impl NoLargeSnapshotsConfig {
    pub fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn inline_max_size(&self) -> u32 {
        self.inline_max_size.unwrap_or(self.max_size)
    }

    pub fn run_once(&self, ctx: &LintContext) {
        let is_snap = ctx.file_extension().is_some_and(|ext| ext.eq_ignore_ascii_case("snap"));

        if is_snap {
            for node in ctx.nodes().iter() {
                if let AstKind::ExpressionStatement(expr_stmt) = node.kind() {
                    self.report_in_expr_stmt(expr_stmt, ctx);
                }
            }
        } else {
            for possible_jest_node in iter_possible_jest_call_node(ctx.semantic()) {
                self.run(&possible_jest_node, ctx);
            }
        }
    }

    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        if !jest_fn_call.args.is_empty() {
            let Some(snapshot_matcher) = jest_fn_call.members.iter().find(|member| {
                member.is_name_equal("toMatchInlineSnapshot")
                    || member.is_name_equal("toThrowErrorMatchingInlineSnapshot")
            }) else {
                return;
            };
            let Some(first_arg) = jest_fn_call.args.first() else {
                return;
            };
            let Some(first_arg_expr) = first_arg.as_expression() else {
                return;
            };

            self.report_in_span(snapshot_matcher.span, first_arg_expr.span(), ctx);
        }
    }

    fn report_in_expr_stmt(&self, expr_stmt: &ExpressionStatement, ctx: &LintContext) {
        let line_count = Self::get_line_count(expr_stmt.span, ctx);
        let allowed = match &expr_stmt.expression {
            Expression::AssignmentExpression(assignment_expr) => {
                let Some(member_expr) = assignment_expr.left.as_member_expression() else {
                    return;
                };
                self.check_allowed_in_snapshots(member_expr, ctx)
            }
            Expression::BinaryExpression(binary_expr) => {
                let Some(member_expr) = binary_expr.left.as_member_expression() else {
                    return;
                };
                self.check_allowed_in_snapshots(member_expr, ctx)
            }
            Expression::LogicalExpression(logical_expr) => {
                let Some(member_expr) = logical_expr.left.as_member_expression() else {
                    return;
                };
                self.check_allowed_in_snapshots(member_expr, ctx)
            }
            _ => false,
        };

        if !allowed && line_count > self.max_size {
            if line_count == 0 {
                ctx.diagnostic(no_snapshot(line_count, expr_stmt.span));
            } else {
                ctx.diagnostic(too_long_snapshot(self.max_size, line_count, expr_stmt.span));
            }
        }
    }

    fn report_in_span(&self, snapshot_matcher_span: Span, first_arg_span: Span, ctx: &LintContext) {
        let line_count = Self::get_line_count(first_arg_span, ctx);
        let inline_max_size = self.inline_max_size();

        if line_count > inline_max_size {
            if inline_max_size == 0 {
                ctx.diagnostic(no_snapshot(line_count, snapshot_matcher_span));
            } else {
                ctx.diagnostic(too_long_snapshot(
                    inline_max_size,
                    line_count,
                    snapshot_matcher_span,
                ));
            }
        }
    }

    fn check_allowed_in_snapshots(
        &self,
        member_expr: &MemberExpression,
        ctx: &LintContext,
    ) -> bool {
        let Some(snapshot_name) = member_expr.static_property_name() else {
            return false;
        };
        let Some(file_name) = ctx.file_path().to_str() else {
            return false;
        };

        let Some(allowed_snapshots_in_file) = self.allowed_snapshots.get(file_name) else {
            return false;
        };

        allowed_snapshots_in_file.iter().any(|matcher| matcher.is_match(snapshot_name))
    }

    #[expect(clippy::cast_possible_truncation)] // the line count can't be over u32::MAX, because the source code is already limited by u32::MAX.
    fn get_line_count(span: Span, ctx: &LintContext) -> u32 {
        let start = span.start as usize;
        let end = span.end as usize;
        ctx.source_text()[start..=end].lines().count() as u32 - 1
    }
}

#[cfg(test)]
mod test {
    use super::NoLargeSnapshotsConfig;

    #[test]
    fn allowed_snapshot_matchers() {
        let config = NoLargeSnapshotsConfig::from_configuration(serde_json::json!([{
            "allowedSnapshots": {
                "/test.snap": [r"^snapshot \d+$", "snapshot [literal"]
            }
        }]))
        .unwrap();
        let matchers = &config.allowed_snapshots["/test.snap"];

        assert!(matchers[0].is_match("snapshot 42"));
        assert!(!matchers[0].is_match("other snapshot 42"));
        assert!(matchers[1].is_match("snapshot [literal"));
        assert!(!matchers[1].is_match("snapshot literal"));
    }

    #[test]
    fn inline_max_size_defaults_to_max_size() {
        let config = NoLargeSnapshotsConfig::from_configuration(serde_json::json!([{
            "maxSize": 10
        }]))
        .unwrap();

        assert_eq!(config.inline_max_size(), 10);
    }
}
