use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};
use oxc_syntax::line_terminator::is_line_terminator;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundBeforeEachBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a blank line before and after a Vitest `beforeEach` statement
    /// when another statement is adjacent to it.
    ///
    /// ### Why is this bad?
    ///
    /// Blank lines separate setup hooks from the surrounding test code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import { beforeEach, test } from 'vitest';
    /// beforeEach(resetState);
    /// test('starts empty', () => {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { beforeEach, test } from 'vitest';
    ///
    /// beforeEach(resetState);
    ///
    /// test('starts empty', () => {});
    /// ```
    PaddingAroundBeforeEachBlocks,
    vitest,
    style,
    fix,
    version = "next",
);

impl Rule for PaddingAroundBeforeEachBlocks {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (statements, mut previous_span) = match node.kind() {
            AstKind::Program(program) => {
                (program.body.as_slice(), program.directives.last().map(GetSpan::span))
            }
            AstKind::FunctionBody(body) => {
                (body.statements.as_slice(), body.directives.last().map(GetSpan::span))
            }
            AstKind::BlockStatement(block) => (block.body.as_slice(), None),
            AstKind::SwitchCase(case) => (case.consequent.as_slice(), None),
            _ => return,
        };

        let mut previous_is_hook = false;
        for statement in statements {
            let is_hook = is_before_each(statement, ctx);
            let span = statement.span();
            if let Some(previous_span) = previous_span
                && (previous_is_hook || is_hook)
            {
                check_padding(previous_span, span, ctx);
            }
            previous_span = Some(span);
            previous_is_hook = is_hook;
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

fn is_before_each<'a>(mut statement: &'a Statement<'a>, ctx: &LintContext<'a>) -> bool {
    while let Statement::LabeledStatement(label) = statement {
        statement = &label.body;
    }
    let Statement::ExpressionStatement(statement) = statement else { return false };
    let expression = match &statement.expression {
        Expression::AwaitExpression(await_expression) => &await_expression.argument,
        expression => expression,
    };
    let Expression::CallExpression(call) = expression else { return false };
    let Expression::Identifier(callee) = &call.callee else { return false };
    PossibleJestNode::from_call_expression(ctx.nodes().get_node(call.node_id()), call, ctx)
        .is_some_and(|node| node.original.unwrap_or(callee.name.as_str()) == "beforeEach")
}

fn check_padding(previous: Span, next: Span, ctx: &LintContext<'_>) {
    let mut cursor = previous.end;
    let mut insert_after = cursor;

    for comment in ctx.comments_range(previous.end..next.start) {
        let gap = ctx.source_range(Span::new(cursor, comment.span.start));
        if has_blank_line(gap) {
            return;
        }
        // Keep trailing comments attached to the preceding statement.
        if insert_after == cursor && !gap.contains(is_line_terminator) {
            insert_after = comment.span.end;
        }
        cursor = comment.span.end;
    }
    if has_blank_line(ctx.source_range(Span::new(cursor, next.start))) {
        return;
    }

    let gap = ctx.source_range(Span::new(insert_after, next.start));
    let insert = first_line_break(gap).map_or("\n\n", |(_, newline)| newline);
    ctx.diagnostic_with_fix(
        OxcDiagnostic::warn("Expected a blank line before this statement")
            .with_help("Add an empty line around the `beforeEach` hook.")
            .with_label(Span::empty(next.start)),
        |fixer| fixer.insert_text_after_range(Span::empty(insert_after), insert.to_owned()),
    );
}

fn first_line_break(text: &str) -> Option<(usize, &str)> {
    let (offset, newline) = text.match_indices(is_line_terminator).next()?;
    let newline = if text[offset..].starts_with("\r\n") { "\r\n" } else { newline };
    Some((offset, newline))
}

fn has_blank_line(text: &str) -> bool {
    first_line_break(text).is_some_and(|(offset, newline)| {
        text[offset + newline.len()..].contains(is_line_terminator)
    })
}

#[test]
fn upstream() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    // Fixtures from @vitest/eslint-plugin v1.6.28.
    let input = r"
const someText = 'abc';
beforeEach(() => {
});
describe('someText', () => {
  const something = 'abc';
  // A comment
  beforeEach(() => {
    // stuff
  });
  beforeEach(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';
  beforeEach(() => {
    // stuff
  });
});
";
    let output = r"
const someText = 'abc';

beforeEach(() => {
});

describe('someText', () => {
  const something = 'abc';

  // A comment
  beforeEach(() => {
    // stuff
  });

  beforeEach(() => {
    // other stuff
  });
});

describe('someText', () => {
  const something = 'abc';

  beforeEach(() => {
    // stuff
  });
});
";
    let paths = ["src/component.test.jsx", "src/component.test.js"];
    let fail = paths.map(|path| (input, None, None, Some(PathBuf::from(path))));
    let fix = paths.map(|path| (input, output, None, Some(PathBuf::from(path))));

    Tester::new(
        PaddingAroundBeforeEachBlocks::NAME,
        PaddingAroundBeforeEachBlocks::PLUGIN,
        vec![(output, None, None, None)],
        fail.to_vec(),
    )
    .with_vitest_plugin(true)
    .expect_fix(fix.to_vec())
    .test_and_snapshot();
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "beforeEach(() => {});",
        "if (enabled) beforeEach(() => {});\ntest('a', () => {});",
        "function beforeEach() {}\nbeforeEach();\nfoo();",
        "beforeEach(() => {});\n\n// Test comment\ntest('a', () => {});",
    ];
    let fix = [
        (
            "beforeEach(() => {});\ntest('a', () => {});",
            "beforeEach(() => {});\n\ntest('a', () => {});",
        ),
        (
            "import { beforeEach as setup } from 'vitest';\nsetup(() => {});",
            "import { beforeEach as setup } from 'vitest';\n\nsetup(() => {});",
        ),
        (
            "beforeEach(() => {}); // Setup comment\r\n// Test comment\r\ntest('a', () => {});",
            "beforeEach(() => {}); // Setup comment\r\n\r\n// Test comment\r\ntest('a', () => {});",
        ),
    ];

    Tester::new(
        PaddingAroundBeforeEachBlocks::NAME,
        PaddingAroundBeforeEachBlocks::PLUGIN,
        pass,
        fix.iter().map(|&(source, _)| source).collect(),
    )
    .with_vitest_plugin(true)
    .expect_fix(fix.to_vec())
    .test();
}
