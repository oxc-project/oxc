use oxc_ast::{AstKind, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{ContentEq, GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_duplicate_if_branches_diagnostic(branch: Span, previous_branch: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This branch has the same body as the previous branch.")
        .with_help("Combine the conditions or remove the duplicate branch.")
        .with_labels([
            previous_branch.label("the previous branch is here"),
            branch.label("this branch has the same body"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateIfBranches;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate adjacent branches in `if` chains.
    ///
    /// ### Why is this bad?
    ///
    /// When two adjacent branches in an `if`/`else if`/`else` chain have the same body,
    /// the condition that chooses between them is redundant. This is usually a
    /// copy-paste bug or an incomplete refactor.
    ///
    /// Only adjacent branches are compared, since non-adjacent duplicate bodies can be
    /// legitimate fallback behavior. Comments, formatting and trailing semicolons are
    /// ignored. Empty branches are ignored.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (isAdmin) {
    ///     showDashboard();
    ///     loadStats();
    /// } else {
    ///     showDashboard();
    ///     loadStats();
    /// }
    ///
    /// if (value === 'a') {
    ///     doThing();
    /// } else if (value === 'b') {
    ///     doThing();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (isAdmin) {
    ///     showDashboard();
    ///     loadStats();
    /// } else {
    ///     showLogin();
    /// }
    ///
    /// if (value === 'a' || value === 'b') {
    ///     doThing();
    /// }
    /// ```
    NoDuplicateIfBranches,
    unicorn,
    suspicious,
    version = "next",
    short_description = "Disallow duplicate adjacent branches in `if` chains.",
);

impl Rule for NoDuplicateIfBranches {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };

        // The whole chain is walked from its root, so an `else if` is skipped here.
        if let AstKind::IfStatement(parent_if) = ctx.nodes().parent_kind(node.id())
            && parent_if
                .alternate
                .as_ref()
                .is_some_and(|alternate| alternate.span() == if_stmt.span)
        {
            return;
        }

        let mut previous = &if_stmt.consequent;
        let mut alternate = if_stmt.alternate.as_ref();
        while let Some(branch) = alternate {
            let (body, next) = match branch {
                Statement::IfStatement(else_if) => {
                    (&else_if.consequent, else_if.alternate.as_ref())
                }
                _ => (branch, None),
            };

            if has_same_body(previous, body) {
                ctx.diagnostic(no_duplicate_if_branches_diagnostic(body.span(), previous.span()));
            }

            previous = body;
            alternate = next;
        }
    }
}

fn branch_statements<'a>(body: &'a Statement<'a>) -> &'a [Statement<'a>] {
    match body {
        Statement::BlockStatement(block) => &block.body,
        _ => std::slice::from_ref(body),
    }
}

// Upstream compares token streams. `ContentEq` ignores spans, so it likewise ignores
// comments, whitespace and trailing semicolons; it additionally ignores how a literal
// is spelled, so `'a'` and `"a"` or `0x10` and `16` compare equal.
fn has_same_body(left: &Statement, right: &Statement) -> bool {
    let left = branch_statements(left);
    let right = branch_statements(right);

    // A branch holding nothing but `;` has no tokens to duplicate.
    left.iter().any(|stmt| !matches!(stmt, Statement::EmptyStatement(_)))
        && left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| left.content_eq(right))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (foo) {
                bar();
            } else {
                baz();
            }",
        "if (foo) {
                bar();
            }",
        "if (foo) {
            } else {
            }",
        "if (foo) {
                ;
            } else {
                ;
            }",
        "if (foo) {
            } else {
                ;
            }",
        "if (foo) {
                // TODO
            } else {
                // TODO
            }",
        "if (foo) {
                bar();
            } else if (baz) {
                qux();
            } else {
                quux();
            }",
        "if (foo) {
                bar();
            } else if (baz) {
                bar(1);
            }",
        "if (format === 'json') {
                parseJson();
            } else if (format === 'yaml') {
                parseYaml();
            } else {
                parseJson();
            }",
        "if (foo) {
                foo.bar();
            } else {
                foo['bar']();
            }",
        "class Unicorn {
                #value;
                method() {
                    if (foo) {
                        this.#value;
                    } else {
                        this.value;
                    }
                }
            }",
        "if (foo) {
                a++;
                b;
            } else {
                a;
                ++b;
            }",
        "if (foo) {
                if (bar) {
                    baz();
                }
            } else if (qux) {
                baz();
            }",
        "function unicorn() {
                if (foo) {
                    return;
                }
                if (bar) {
                    return;
                }
            }",
    ];

    let fail = vec![
        "if (isAdmin) {
                showDashboard();
                loadStats();
            } else {
                showDashboard();
                loadStats();
            }",
        "if (foo) {
                bar();
            } else if (baz) {
                bar();
            }",
        "if (foo) {
                bar();
            } else if (baz) {
                bar();
            } else {
                bar();
            }",
        "if (foo) {
                bar();
            } else if (baz) {
                qux();
            } else {
                qux();
            }",
        "function unicorn() {
                if (foo)
                    return bar();
                else
                    return bar();
            }",
        "if (foo)
                bar();
            else {
                bar();
            }",
        "if (foo)
                bar();
            else
                bar()",
        "if (foo) {
                // Comment does not make the branch different.
                bar();
            } else {
                bar();
            }",
        "if (foo) {
                bar(
                    baz
                );
            } else {
                bar(baz);
            }",
        "function unicorn() {
                if (foo) {
                    return bar();
                } else {
                    return bar()
                }
            }",
        "function unicorn() {
                if (foo) {
                    do {
                        bar();
                    } while (baz);
                } else {
                    do {
                        bar();
                    } while (baz)
                }
            }",
        "if (foo) {
                const bar = value as string;
                baz(bar);
            } else {
                const bar = value as string;
                baz(bar);
            }", // {"parser": parsers.typescript},
        "if (foo) {
                const bar = value satisfies string;
                baz(bar!);
            } else {
                const bar = value satisfies string;
                baz(bar!);
            }", // {"parser": parsers.typescript}
        // Upstream compares tokens, so it treats this as valid. `ContentEq` ignores
        // literal spelling, so it is reported here.
        r#"if (foo) {
                const value = 'unicorn';
            } else {
                const value = "unicorn";
            }"#,
    ];

    Tester::new(NoDuplicateIfBranches::NAME, NoDuplicateIfBranches::PLUGIN, pass, fail)
        .test_and_snapshot();
}
