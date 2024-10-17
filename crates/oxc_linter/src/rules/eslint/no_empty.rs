use oxc_ast::{ast::BlockStatement, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_empty_diagnostic(stmt_kind: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected empty block statements")
        .with_help(format!("Remove this {stmt_kind} or add a comment inside it"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmpty {
    allow_empty_catch: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallows empty block statements
    ///
    /// ### Why is this bad?
    /// Empty block statements, while not technically errors, usually occur due to refactoring that wasn’t completed.
    /// They can cause confusion when reading code.
    ///
    /// ### Example
    /// ```javascript
    /// if (condition) {
    ///
    /// }
    /// ```
    NoEmpty,
    restriction,
    suggestion
);

impl Rule for NoEmpty {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        Self {
            allow_empty_catch: obj
                .and_then(|v| v.get("allowEmptyCatch"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BlockStatement(block) if block.body.is_empty() => {
                let parent = ctx.nodes().parent_kind(node.id());
                if self.allow_empty_catch && matches!(parent, Some(AstKind::CatchClause(_))) {
                    return;
                }

                if ctx.semantic().has_comments_between(block.span) {
                    return;
                }
                ctx.diagnostic_with_suggestion(no_empty_diagnostic("block", block.span), |fixer| {
                    if let Some(parent) = parent {
                        if matches!(parent, AstKind::CatchClause(_)) {
                            fixer.noop()
                        } else {
                            fixer.delete(&parent)
                        }
                    } else {
                        fixer.noop()
                    }
                });
            }
            // The visitor does not visit the `BlockStatement` inside the `FinallyClause`.
            // See `Visit::visit_finally_clause`.
            AstKind::FinallyClause(finally_clause) if finally_clause.body.is_empty() => {
                if ctx.semantic().has_comments_between(finally_clause.span) {
                    return;
                }
                ctx.diagnostic_with_suggestion(
                    no_empty_diagnostic("block", finally_clause.span),
                    |fixer| {
                        let parent = ctx
                            .nodes()
                            .parent_kind(node.id())
                            .expect("finally clauses must have a parent node");

                        let AstKind::TryStatement(parent) = parent else {
                            unreachable!("finally clauses must be children of a try statement");
                        };

                        // if there's no `catch`, we can't remove the `finally` block
                        if parent.handler.is_none() {
                            return fixer.noop();
                        }

                        if let Some(finally_kw_start) = find_finally_start(ctx, finally_clause) {
                            fixer.delete_range(Span::new(finally_kw_start, finally_clause.span.end))
                        } else {
                            fixer.noop()
                        }
                    },
                );
            }
            AstKind::SwitchStatement(switch) if switch.cases.is_empty() => {
                ctx.diagnostic_with_suggestion(
                    no_empty_diagnostic("switch", switch.span),
                    |fixer| fixer.delete(switch),
                );
            }
            _ => {}
        }
    }
}

fn find_finally_start(ctx: &LintContext, finally_clause: &BlockStatement) -> Option<u32> {
    let src = ctx.source_text();
    let finally_start = finally_clause.span.start as usize - 1;
    let mut start = finally_start;

    let src_chars: Vec<char> = src.chars().collect();

    while start > 0 {
        if let Some(&ch) = src_chars.get(start) {
            if !ch.is_whitespace() {
                if ch == 'y'
                    && "finally".chars().rev().skip(1).all(|c| {
                        start -= 1;
                        src_chars.get(start) == Some(&c)
                    })
                {
                    #[allow(clippy::cast_possible_truncation)]
                    return Some(start as u32);
                }
                return None;
            }
        }
        start = start.saturating_sub(1);
    }

    None
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("if (foo) { bar() }", None),
        ("while (foo) { bar() }", None),
        ("for (;foo;) { bar() }", None),
        ("try { foo() } catch (ex) { foo() }", None),
        ("switch(foo) {case 'foo': break;}", None),
        ("(function() { }())", None),
        ("var foo = () => {};", None),
        ("function foo() { }", None),
        ("if (foo) {/* empty */}", None),
        ("while (foo) {/* empty */}", None),
        ("for (;foo;) {/* empty */}", None),
        ("try { foo() } catch (ex) {/* empty */}", None),
        ("try { foo() } catch (ex) {// empty\n}", None),
        ("try { foo() } finally {// empty\n}", None),
        ("try { foo() } finally {// test\n}", None),
        ("try { foo() } finally {\n \n // hi i am off no use\n}", None),
        ("try { foo() } catch (ex) {/* test111 */}", None),
        ("if (foo) { bar() } else { // nothing in me \n}", None),
        ("if (foo) { bar() } else { /**/ \n}", None),
        ("if (foo) { bar() } else { // \n}", None),
        ("try { foo(); } catch (ex) {}", Some(json!([ { "allowEmptyCatch": true }]))),
        (
            "try { foo(); } catch (ex) {} finally { bar(); }",
            Some(json!([ { "allowEmptyCatch": true }])),
        ),
    ];

    let fail = vec![
        ("try {} catch (ex) {throw ex}", None),
        ("try { foo() } catch (ex) {throw ex} finally {}", None),
        ("try { foo() } catch (ex) {}", None),
        ("if (foo) {}", None),
        ("while (foo) {}", None),
        ("for (;foo;) {}", None),
        ("switch(foo) {}", None),
        ("switch (foo) { /* empty */ }", None),
        ("try {} catch (ex) {}", Some(json!([ { "allowEmptyCatch": true }]))),
        ("try { foo(); } catch (ex) {} finally {}", Some(json!([ { "allowEmptyCatch": true }]))),
        ("try {} catch (ex) {} finally {}", Some(json!([ { "allowEmptyCatch": true }]))),
        ("try { foo(); } catch (ex) {} finally {}", None),
    ];

    let fix = vec![
        ("try {} catch (ex) {throw ex}", "", None),
        (
            "try { foo() } catch (ex) {throw ex} finally {}",
            "try { foo() } catch (ex) {throw ex} ",
            None,
        ),
        // we can't fix this because removing the `catch` block would change the semantics of the code
        ("try { foo() } catch (ex) {}", "try { foo() } catch (ex) {}", None),
        ("if (foo) {}", "", None),
        ("while (foo) {}", "", None),
        ("for (;foo;) {}", "", None),
        ("switch(foo) {}", "", None),
        ("switch (foo) { /* empty */ }", "", None),
        ("try {} catch (ex) {}", "", Some(json!([ { "allowEmptyCatch": true }]))),
        (
            "try { foo(); } catch (ex) {} finally {}",
            "try { foo(); } catch (ex) {} ",
            Some(json!([ { "allowEmptyCatch": true }])),
        ),
        ("try {} catch (ex) {} finally {}", "", Some(json!([ { "allowEmptyCatch": true }]))),
        ("try { foo(); } catch (ex) {} finally {}", "try { foo(); } catch (ex) {} ", None),
    ];

    Tester::new(NoEmpty::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
