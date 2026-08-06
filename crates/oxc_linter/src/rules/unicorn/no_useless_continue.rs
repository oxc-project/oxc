// Ported from eslint-plugin-unicorn v72.0.0:
// https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v72.0.0/rules/no-useless-continue.js
// Copyright (c) Sindre Sorhus and contributors. Licensed under MIT; see THIRD-PARTY-LICENSE.

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_continue_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary `continue` statement.")
        .with_help("Remove this redundant `continue` statement.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessContinue;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows `continue` statements that are reached as the final action of
    /// the current loop iteration.
    ///
    /// ### Why is this bad?
    ///
    /// A trailing `continue` statement has no effect because the loop advances
    /// to its next iteration immediately afterward. Removing it makes the
    /// control flow easier to read without changing behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (const item of items) {
    ///     process(item);
    ///     continue;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// for (const item of items) {
    ///     if (shouldSkip(item)) {
    ///         continue;
    ///     }
    ///     process(item);
    /// }
    /// ```
    NoUselessContinue,
    unicorn,
    pedantic,
    fix,
    version = "next",
    short_description = "Disallows useless `continue` statements.",
);

impl Rule for NoUselessContinue {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ContinueStatement(continue_statement) = node.kind() else {
            return;
        };

        if continue_statement.label.is_some() || !is_useless_continue(node, ctx) {
            return;
        }

        ctx.diagnostic_with_fix(no_useless_continue_diagnostic(continue_statement.span), |fixer| {
            fixer.delete(continue_statement)
        });
    }
}

/// Returns whether removing this `continue` would leave the loop iteration's
/// behavior unchanged.
fn is_useless_continue(node: &AstNode, ctx: &LintContext) -> bool {
    let nodes = ctx.nodes();

    // Removing a bare loop body such as `while (condition) continue;` would
    // leave invalid syntax. Upstream intentionally treats these as valid.
    if !matches!(nodes.parent_kind(node.id()), AstKind::BlockStatement(_)) {
        return false;
    }

    let mut current_id = node.id();
    let mut current_span = node.kind().span();

    loop {
        let parent = nodes.parent_node(current_id);

        match parent.kind() {
            AstKind::BlockStatement(block) => {
                if block.body.last().is_none_or(|statement| statement.span() != current_span) {
                    return false;
                }

                current_id = parent.id();
                current_span = block.span;
            }
            AstKind::IfStatement(if_statement) => {
                let is_branch = if_statement.consequent.span() == current_span
                    || if_statement
                        .alternate
                        .as_ref()
                        .is_some_and(|alternate| alternate.span() == current_span);

                if !is_branch {
                    return false;
                }

                current_id = parent.id();
                current_span = if_statement.span;
            }
            AstKind::ForStatement(statement) => return statement.body.span() == current_span,
            AstKind::ForInStatement(statement) => return statement.body.span() == current_span,
            AstKind::ForOfStatement(statement) => return statement.body.span() == current_span,
            AstKind::WhileStatement(statement) => return statement.body.span() == current_span,
            AstKind::DoWhileStatement(statement) => return statement.body.span() == current_span,
            _ => return false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "for (const x of xs) {
                if (skip(x)) {
                    continue;
                }
                process(x);
            }",
        "for (const x of xs) {
                if (a) {
                    continue;
                }
                doMore();
            }",
        "for (const x of xs) {
                if (a) {
                    continue;
                } else {
                    doMore();
                }
                use(x);
            }",
        "while (cond) continue;",
        "for (;;) continue;",
        "for (const x of xs) continue;",
        "outer: for (const x of xs) {
                for (const y of ys) {
                    continue outer;
                }
            }",
        "loop: for (const x of xs) {
                doX();
                continue loop;
            }",
        "for (const x of xs) {
                switch (x) {
                    case 1:
                        continue;
                }
            }",
        "for (const x of xs) {
                try {
                    continue;
                } finally {
                    cleanup();
                }
            }",
        "for (const x of xs) {
                try {
                    doX();
                    continue;
                } catch {}
            }",
        "for (const x of xs) {
                try {
                    doX();
                } catch {
                    continue;
                }
            }",
        "for (const x of xs) {
                try {
                    doX();
                } finally {
                    continue;
                }
            }",
        "for (const x of xs) {
                for (const y of ys) {
                    if (skip(y)) {
                        continue;
                    }
                    process(y);
                }
            }",
        "for (const x of xs) {
                continue;
                doX();
            }",
        "for (const x of xs) {
                continue;
                function f() {}
            }",
        "for (const x of xs) {
                if (a) {
                    continue;
                }
                ;
            }",
        "for (const x of xs) {
                {
                    continue;
                }
                doMore();
            }",
        "for (const x of xs) {
                block: {
                    continue;
                }
            }",
        "for (const x of xs) {
                if (a) {
                    if (b) {
                        continue;
                    }
                }
                doMore();
            }",
    ];

    let fail = vec![
        "for (const x of xs) {
                process(x);
                continue;
            }",
        "for (const x of xs) {
                continue;
            }",
        "while (cond) {
                doX();
                continue;
            }",
        "do {
                doX();
                continue;
            } while (cond);",
        "for (const x in object) {
                doX();
                continue;
            }",
        "for (let i = 0; i < n; i++) {
                doX();
                continue;
            }",
        "for (const x of xs) if (a) { continue; }",
        "for (const x of xs) {
                if (a) {
                    continue;
                }
            }",
        "for (const x of xs) {
                if (a) {
                    if (b) {
                        continue;
                    }
                }
            }",
        "for (const x of xs) {
                if (a) {
                    doX();
                    continue;
                } else {
                    doY();
                }
            }",
        "for (const x of xs) {
                if (a) {
                    doX();
                } else {
                    doY();
                    continue;
                }
            }",
        "for (const x of xs) {
                if (a) {
                    doX();
                } else if (b) {
                    doY();
                    continue;
                }
            }",
        "async function run() {
                for await (const x of xs) {
                    process(x);
                    continue;
                }
            }",
        "for (const x of xs) {
                {
                    continue;
                }
            }",
        "for (const x of xs) {
                for (const y of ys) {
                    process(y);
                    continue;
                }
            }",
        "for (const x of xs) {
                for (const y of ys) {
                    continue;
                }
                continue;
            }",
        "outer: for (const x of xs) {
                doX();
                continue;
            }",
        "for (const x of xs) {
                continue;
                continue;
            }",
        "for (const x of xs) {
                if (a) {
                    doX();
                } else {
                    if (b) {
                        continue;
                    }
                }
            }",
        "do {
                if (a) {
                    continue;
                }
            } while (cond);",
        "for (const x of xs) {
                doX();
                continue; // trailing comment
            }",
        "for (const x of xs) {
                doX();
                // leading comment
                continue;
            }",
    ];

    let fix = vec![
        ("for (const x of xs) { process(x); continue; }", "for (const x of xs) { process(x);  }"),
        ("for (const x of xs) { continue; }", "for (const x of xs) {  }"),
        ("for (const x of xs) { if (a) { continue; } }", "for (const x of xs) { if (a) {  } }"),
        (
            "for (const x of xs) { if (a) { doX(); } else { continue; } }",
            "for (const x of xs) { if (a) { doX(); } else {  } }",
        ),
        (
            "for (const x of xs) { for (const y of ys) { continue; } continue; }",
            "for (const x of xs) { for (const y of ys) {  }  }",
        ),
        (
            "for (const x of xs) { doX(); continue; // trailing comment\n}",
            "for (const x of xs) { doX();  // trailing comment\n}",
        ),
        (
            "for (const x of xs) { doX(); // leading comment\ncontinue; }",
            "for (const x of xs) { doX(); // leading comment\n }",
        ),
    ];

    Tester::new(NoUselessContinue::NAME, NoUselessContinue::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
