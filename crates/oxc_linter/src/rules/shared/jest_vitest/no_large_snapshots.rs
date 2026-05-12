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
use serde::Deserialize;

use crate::{
    context::LintContext,
    utils::{PossibleJestNode, iter_possible_jest_call_node, parse_expect_jest_fn_call},
};

fn no_snapshot(line_count: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Snapshot is too long.")
        .with_help(format!(
            "Expected to not encounter a Jest or Vitest snapshot but one was found that is {line_count} lines long"
        ))
        .with_label(span)
}

fn too_long_snapshot(line_limit: usize, line_count: usize, span: Span) -> OxcDiagnostic {
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

Examples of **incorrect** code for this rule:
```js
exports[`a more manageable and readable snapshot 1`] = `
line 1
line 2
line 3
line 4
`;
```
";

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoLargeSnapshotsConfig {
    /// Maximum number of lines allowed for external snapshot files.
    pub max_size: usize,
    /// Maximum number of lines allowed for inline snapshots.
    pub inline_max_size: usize,
    /// A map of snapshot file paths to arrays of snapshot names that are allowed to exceed the size limit.
    /// Snapshot names can be specified as regular expressions.
    pub allowed_snapshots: FxHashMap<CompactStr, Vec<CompactStr>>,
}

impl Default for NoLargeSnapshotsConfig {
    fn default() -> Self {
        Self { max_size: 50, inline_max_size: 50, allowed_snapshots: FxHashMap::default() }
    }
}

impl NoLargeSnapshotsConfig {
    #[expect(clippy::unnecessary_wraps)]
    pub fn from_configuration(value: &serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = value.get(0);

        let max_size = config
            .and_then(|c| c.get("maxSize"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
            .unwrap_or(50);

        let inline_max_size = config
            .and_then(|c| c.get("inlineMaxSize"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
            .unwrap_or(max_size);

        let allowed_snapshots = config
            .and_then(|c| c.get("allowedSnapshots"))
            .and_then(serde_json::Value::as_object)
            .map(Self::compile_allowed_snapshots)
            .unwrap_or_default();

        Ok(Self { max_size, inline_max_size, allowed_snapshots })
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

        if line_count > self.inline_max_size {
            if self.inline_max_size == 0 {
                ctx.diagnostic(no_snapshot(line_count, snapshot_matcher_span));
            } else {
                ctx.diagnostic(too_long_snapshot(
                    self.inline_max_size,
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

        allowed_snapshots_in_file.iter().any(|allowed_snapshot| {
            match Regex::new(allowed_snapshot) {
                Ok(regex) => regex.is_match(snapshot_name),
                Err(_) => snapshot_name == allowed_snapshot,
            }
        })
    }

    fn get_line_count(span: Span, ctx: &LintContext) -> usize {
        let start = span.start as usize;
        let end = span.end as usize;
        ctx.source_text()[start..=end].lines().count() - 1
    }

    pub fn compile_allowed_snapshots(
        matchers: &serde_json::Map<String, serde_json::Value>,
    ) -> FxHashMap<CompactStr, Vec<CompactStr>> {
        matchers
            .iter()
            .map(|(key, value)| {
                let serde_json::Value::Array(configs) = value else {
                    return (CompactStr::from(key.as_str()), vec![]);
                };

                let configs =
                    configs.iter().filter_map(|c| c.as_str().map(CompactStr::from)).collect();

                (CompactStr::from(key.as_str()), configs)
            })
            .collect()
    }
}
