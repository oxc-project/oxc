use std::{ops::Deref, path::Path};

use oxc_ast::{
    ast::{Expression, ExpressionStatement, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use regex::Regex;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{iter_possible_jest_call_node, parse_expect_jest_fn_call, PossibleJestNode},
};

// TODO: re-word diagnostic messages
fn no_snapshot(x0: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow large snapshots.")
        .with_help(format!("`{x0:?}`s should begin with lowercase"))
        .with_label(span)
}

fn too_long_snapshots(x0: usize, x1: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow large snapshots.")
        .with_help(format!(
            "Expected Jest snapshot to be smaller than {x0:?} lines but was {x1:?} lines long"
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLargeSnapshots(Box<NoLargeSnapshotsConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoLargeSnapshotsConfig {
    pub max_size: usize,
    pub inline_max_size: usize,
    pub allowed_snapshots: FxHashMap<CompactStr, Vec<CompactStr>>,
}

impl Deref for NoLargeSnapshots {
    type Target = NoLargeSnapshotsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When using Jest's snapshot capability one should be mindful of the size of
    /// created snapshots. As a general best practice snapshots should be limited in
    /// size in order to be more manageable and reviewable. A stored snapshot is only as
    /// good as its review and as such keeping it short, sweet, and readable is
    /// important to allow for thorough reviews.
    ///
    /// ### Example
    ///
    /// ```javascript
    ///
    /// // invalid
    /// exports[`a large snapshot 1`] = `
    /// line 1
    /// line 2
    /// line 3
    /// line 4
    /// line 5
    /// line 6
    /// line 7
    /// line 8
    /// line 9
    /// line 10
    /// line 11
    /// line 12
    /// line 13
    /// line 14
    /// line 15
    /// line 16
    /// line 17
    /// line 18
    /// line 19
    /// line 20
    /// line 21
    /// line 22
    /// line 23
    /// line 24
    /// line 25
    /// line 26
    /// line 27
    /// line 28
    /// line 29
    /// line 30
    /// line 31
    /// line 32
    /// line 33
    /// line 34
    /// line 35
    /// line 36
    /// line 37
    /// line 38
    /// line 39
    /// line 40
    /// line 41
    /// line 42
    /// line 43
    /// line 44
    /// line 45
    /// line 46
    /// line 47
    /// line 48
    /// line 49
    /// line 50
    /// line 51
    /// `;
    ///
    /// // valid
    /// exports[`a more manageable and readable snapshot 1`] = `
    /// line 1
    /// line 2
    /// line 3
    /// line 4
    /// `;
    /// ```
    ///
    NoLargeSnapshots,
    style,
);

impl Rule for NoLargeSnapshots {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        let max_size = config
            .and_then(|c| c.get("maxSize"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(50, |v| usize::try_from(v).unwrap_or(50));

        let inline_max_size = config
            .and_then(|c| c.get("inlineMaxSize"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(max_size, |v| usize::try_from(v).unwrap_or(max_size));

        let allowed_snapshots = config
            .and_then(|c| c.get("allowedSnapshots"))
            .and_then(serde_json::Value::as_object)
            .and_then(Self::compile_allowed_snapshots)
            .unwrap_or_default();

        Self(Box::new(NoLargeSnapshotsConfig { max_size, inline_max_size, allowed_snapshots }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let is_snap = ctx.file_path().to_str().map_or(false, |p| {
            Path::new(p).extension().map_or(false, |ext| ext.eq_ignore_ascii_case("snap"))
        });

        if is_snap {
            for node in ctx.nodes().iter().collect::<Vec<_>>() {
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
}

impl NoLargeSnapshots {
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        if jest_fn_call.args.len() > 0
            && jest_fn_call.members.iter().any(|member| {
                member.is_name_equal("toMatchInlineSnapshot")
                    || member.is_name_equal("toThrowErrorMatchingInlineSnapshot")
            })
        {
            let Some(first_arg) = jest_fn_call.args.first() else {
                return;
            };
            let Some(first_arg_expr) = first_arg.as_expression() else {
                return;
            };

            let span = first_arg_expr.span();
            self.report_in_span(span, ctx);
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
                ctx.diagnostic(too_long_snapshots(self.max_size, line_count, expr_stmt.span));
            }
        }
    }

    fn report_in_span(&self, span: Span, ctx: &LintContext) {
        let line_count = Self::get_line_count(span, ctx);

        if line_count > self.inline_max_size {
            if self.inline_max_size == 0 {
                ctx.diagnostic(no_snapshot(line_count, span));
            } else {
                ctx.diagnostic(too_long_snapshots(self.inline_max_size, line_count, span));
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

    #[allow(clippy::unnecessary_wraps)]
    pub fn compile_allowed_snapshots(
        matchers: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<FxHashMap<CompactStr, Vec<CompactStr>>> {
        Some(
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
                .collect(),
        )
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    // use std::path::PathBuf;

    fn generate_snapshot_lines(lines: usize) -> String {
        let repeated_lines = "line\n".repeat(lines);
        format!("`\n{repeated_lines}`")
    }

    // fn generate_exports_snapshot_string(lines: usize, title: Option<&str>) -> String {
    //     let title = title.unwrap_or("a big component 1");
    //     format!("exports[`{}`] = {};", title, generate_snapshot_lines(lines))
    // }

    fn generate_expect_inline_snaps_code(line: usize, matcher: &str) -> String {
        format!("expect(something).{}({});", matcher, generate_snapshot_lines(line))
    }

    fn generate_match_inline_snapshot(line: usize) -> String {
        generate_expect_inline_snaps_code(line, "toMatchInlineSnapshot")
    }

    fn generate_throw_error_matching_inline_snapshot(line: usize) -> String {
        generate_expect_inline_snaps_code(line, "toThrowErrorMatchingInlineSnapshot")
    }

    // Note: Currently oxlint didn't check `.snap` file
    //
    // #[cfg(target_os = "windows")]
    // let snap_path = "c:\\mock-component.jsx.snap";
    // #[cfg(target_os = "windows")]
    // let another_snap_path = "c:\\another-mock-component.jsx.snap";

    // #[cfg(not(target_os = "windows"))]
    // let snap_path = "/mock-component.jsx.snap";
    // #[cfg(not(target_os = "windows"))]
    // let another_snap_path = "/another-mock-component.jsx.snap";

    let tow_match_inline_cases = generate_match_inline_snapshot(2);
    let two_throw_error_match_cases = generate_throw_error_matching_inline_snapshot(2);
    let twenty_match_inline_cases = generate_match_inline_snapshot(20);
    let sixty_match_inline_cases = generate_match_inline_snapshot(60);
    let sixty_cases = format!(
        "
            expect(
                functionUnderTest(
                    arg1,
                    arg2,
                    arg3
                )
            ).toMatchInlineSnapshot({})
        ",
        generate_snapshot_lines(60)
    );
    // let twenty_exports_snapshot = generate_exports_snapshot_string(20, None);
    // let fifty_eight_exports_snapshot = generate_exports_snapshot_string(58, None);

    let pass = vec![
        ("expect(something)", None, None, None),
        ("expect(something).toBe(1)", None, None, None),
        ("expect(something).toMatchInlineSnapshot", None, None, None),
        ("expect(something).toMatchInlineSnapshot()", None, None, None),
        (tow_match_inline_cases.as_str(), None, None, None),
        (two_throw_error_match_cases.as_str(), None, None, None),
        (
            twenty_match_inline_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 19, "inlineMaxSize": 21 }])),
            None,
            None,
        ),
        (
            sixty_match_inline_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 61 }])),
            None,
            None,
        ),
        (sixty_cases.as_str(), Some(serde_json::json!([{ "maxSize": 61 }])), None, None),
        // '/mock-component.jsx.snap'
        // (twenty_exports_snapshot.as_str(), None, None, Some(PathBuf::from(snap_path))),
        // '/mock-component.jsx.snap'
        // (
        //     fifty_eight_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{
        //         "allowedSnapshots": {
        //             snap_path.to_string(): ["a big component 1"]
        //         }
        //     }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     twenty_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{ "maxSize": 21, "inlineMaxSize": 19 }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
    ];

    let fifty_match_inline_cases = generate_match_inline_snapshot(50);
    let fifty_throw_error_match_cases = generate_throw_error_matching_inline_snapshot(50);

    // let fifty_two_exports_snapshot = generate_exports_snapshot_string(58, None);
    // let one_hundred_exports_snapshot = generate_exports_snapshot_string(100, None);
    // let one_exports_snapshot = generate_exports_snapshot_string(1, None);
    // let fifty_eight_exports_snapshot = generate_exports_snapshot_string(58, None);
    // let vec_to_str = [
    //     generate_exports_snapshot_string(58, Some("a big component w/ text")),
    //     generate_exports_snapshot_string(58, Some("a big component 2")),
    // ]
    // .join("\n\n");

    let fail = vec![
        (fifty_match_inline_cases.as_str(), None, None, None),
        (fifty_throw_error_match_cases.as_str(), None, None, None),
        (
            fifty_throw_error_match_cases.as_str(),
            Some(serde_json::json!([{ "maxSize": 51, "inlineMaxSize": 50 }])),
            None,
            None,
        ),
        // '/mock-component.jsx.snap'
        // (fifty_two_exports_snapshot.as_str(), None, None, Some(PathBuf::from(snap_path))),
        // '/mock-component.jsx.snap'
        // (
        //     one_hundred_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{ "maxSize": 70 }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     one_hundred_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{ "maxSize": 70, "inlineMaxSize": 101 }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     one_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{ "maxSize": 0 }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     fifty_eight_exports_snapshot.as_str(),
        //     Some(serde_json::json!([{
        //         "allowedSnapshots": {
        //             another_snap_path.to_string(): [r"a big component \d+"]
        //         }
        //     }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     vec_to_str.as_str(),
        //     Some(serde_json::json!([{
        //         "allowedSnapshots": {
        //             snap_path.to_string(): [r"a big component \d+"],
        //         },
        //     }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
        // '/mock-component.jsx.snap'
        // (
        //     vec_to_str.as_str(),
        //     Some(serde_json::json!([{
        //         "allowedSnapshots": {
        //             snap_path.to_string(): ["a big component 2"],
        //         },
        //     }])),
        //     None,
        //     Some(PathBuf::from(snap_path)),
        // ),
    ];

    Tester::new(NoLargeSnapshots::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
