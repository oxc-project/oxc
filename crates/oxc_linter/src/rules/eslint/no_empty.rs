use oxc_ast::{AstKind, ast::BlockStatement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_empty_diagnostic(stmt_kind: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected empty block statements")
        .with_help(format!("Remove this {stmt_kind} or add a comment inside it"))
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoEmpty {
    /// If set to `true`, allows an empty `catch` block without triggering the linter.
    allow_empty_catch: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows empty block statements
    ///
    /// ### Why is this bad?
    ///
    /// Empty block statements, while not technically errors, usually occur due to refactoring that wasn’t completed.
    /// They can cause confusion when reading code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (condition) {
    ///
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (condition) {
    ///   throw new Error("condition should be false")
    /// }
    /// ```
    NoEmpty,
    eslint,
    restriction,
    suggestion,
    config = NoEmpty,
);

impl Rule for NoEmpty {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoEmpty>>(value).unwrap_or_default().into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BlockStatement(block) if block.body.is_empty() => {
                let parent = ctx.nodes().parent_kind(node.id());
                if self.allow_empty_catch && matches!(parent, AstKind::CatchClause(_)) {
                    return;
                }

                if ctx.has_comments_between(block.span) {
                    return;
                }
                ctx.diagnostic_with_suggestion(no_empty_diagnostic("block", block.span), |fixer| {
                    if let AstKind::TryStatement(try_stmt) = parent
                        && let Some(try_block_stmt) = &try_stmt.finalizer
                        && try_block_stmt.span == block.span
                    {
                        return if let Some(finally_kw_start) = find_finally_start(ctx, block) {
                            fixer.delete_range(Span::new(finally_kw_start, block.span.end))
                        } else {
                            fixer.noop()
                        };
                    }
                    if matches!(parent, AstKind::CatchClause(_)) {
                        return fixer.noop();
                    }
                    fixer.delete(&parent)
                });
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
        if let Some(&ch) = src_chars.get(start)
            && !ch.is_whitespace()
        {
            if ch == 'y'
                && "finally".chars().rev().skip(1).all(|c| {
                    start -= 1;
                    src_chars.get(start) == Some(&c)
                })
            {
                #[expect(clippy::cast_possible_truncation)]
                return Some(start as u32);
            }
            return None;
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

    Tester::new(NoEmpty::NAME, NoEmpty::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
