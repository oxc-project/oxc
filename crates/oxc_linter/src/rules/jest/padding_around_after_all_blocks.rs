use std::ops::Deref;

use oxc_ast::{AstKind, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        JestGeneralFnKind, ParsedGeneralJestFnCall, PossibleJestNode, parse_general_jest_fn_call,
    },
};

fn padding_around_after_all_blocks_diagnostic(span: Span, name: &CompactStr) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Missing padding before {name} block"))
        .with_help(format!("Make sure there is an empty new line before the {name} block"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterAllBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a line of padding before and after 1 or more
    /// `afterAll` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent formatting of code can make the code more difficult to read
    /// and follow. This rule helps ensure that `afterAll` blocks are visually
    /// separated from the rest of the code, making them easier to identify while
    /// looking through test files.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const thing = 123;
    /// afterAll(() => {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const thing = 123;
    ///
    /// afterAll(() => {});
    /// ```
    PaddingAroundAfterAllBlocks,
    jest,
    style,
    fix
);

impl Rule for PaddingAroundAfterAllBlocks {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, jest_node, ctx) else {
            return;
        };
        let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
        let Some(kind) = kind.to_general() else {
            return;
        };
        if kind != JestGeneralFnKind::Hook {
            return;
        }
        let name_str = name.deref();
        if name_str != "afterAll" {
            return;
        }
        let scope_node = ctx.nodes().get_node(ctx.scoping().get_node_id(node.scope_id()));
        let prev_statement_span = match scope_node.kind() {
            AstKind::Program(program) => {
                get_statement_span_before_node(*node, program.body.as_slice())
            }
            AstKind::ArrowFunctionExpression(arrow_func_expr) => {
                get_statement_span_before_node(*node, arrow_func_expr.body.statements.as_slice())
            }
            AstKind::Function(function) => {
                let Some(body) = &function.body else {
                    return;
                };
                get_statement_span_before_node(*node, body.statements.as_slice())
            }
            _ => None,
        };
        let Some(prev_statement_span) = prev_statement_span else {
            return;
        };
        let mut comments_range = ctx.comments_range(prev_statement_span.end..node.span().start);
        let mut span_between_start = prev_statement_span.end;
        let mut span_between_end = node.span().start;
        if let Some(last_comment_span) = comments_range.next_back().map(|comment| comment.span) {
            let space_after_last_comment =
                ctx.source_range(Span::new(last_comment_span.end, node.span().start));
            let space_before_last_comment =
                ctx.source_range(Span::new(prev_statement_span.end, last_comment_span.start));
            if space_after_last_comment.matches('\n').count() > 1
                || space_before_last_comment.matches('\n').count() == 0
            {
                span_between_start = last_comment_span.end;
            } else {
                span_between_end = last_comment_span.start;
            }
        }
        let span_between = Span::new(span_between_start, span_between_end);
        let content = ctx.source_range(span_between);
        if content.matches('\n').count() < 2 {
            ctx.diagnostic_with_fix(
                padding_around_after_all_blocks_diagnostic(
                    Span::new(span_between_end, span_between_end),
                    &name.deref().into(),
                ),
                |fixer| {
                    let whitespace_after_last_line =
                        content.rfind('\n').map_or("", |index| content.split_at(index + 1).1);
                    fixer.replace(span_between, format!("\n\n{whitespace_after_last_line}"))
                },
            );
        }
    }
}

fn get_statement_span_before_node(node: AstNode, statements: &[Statement]) -> Option<Span> {
    statements
        .iter()
        .filter_map(|statement| {
            if statement.span().end <= node.span().start { Some(statement.span()) } else { None }
        })
        .next_back()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "afterAll(() => {});",
        "const thing = 123;\n\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\n});",
    ];

    let fail = vec!["const thing = 123;\nafterAll(() => {});"];

    let fix = vec![(
        "const thing = 123;\nafterAll(() => {});",
        "const thing = 123;\n\nafterAll(() => {});",
    )];

    Tester::new(PaddingAroundAfterAllBlocks::NAME, PaddingAroundAfterAllBlocks::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
