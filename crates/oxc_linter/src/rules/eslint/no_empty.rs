use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_empty_diagnostic(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-empty): Disallow empty block statements")
        .with_help(format!("Add comment inside empty {x0} statement"))
        .with_label(span1.label(format!("Empty {x0} statement")))
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
                if self.allow_empty_catch
                    && matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::CatchClause(_)))
                {
                    return;
                }

                if ctx.semantic().trivias().has_comments_between(block.span) {
                    return;
                }
                ctx.diagnostic(no_empty_diagnostic("block", block.span));
            }
            // The visitor does not visit the `BlockStatement` inside the `FinallyClause`.
            // See `Visit::visit_finally_clause`.
            AstKind::FinallyClause(finally_clause) if finally_clause.body.is_empty() => {
                if ctx.semantic().trivias().has_comments_between(finally_clause.span) {
                    return;
                }
                ctx.diagnostic(no_empty_diagnostic("block", finally_clause.span));
            }
            AstKind::SwitchStatement(switch) if switch.cases.is_empty() => {
                ctx.diagnostic(no_empty_diagnostic("switch", switch.span));
            }
            _ => {}
        }
    }
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

    Tester::new(NoEmpty::NAME, pass, fail).test_and_snapshot();
}
