use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{AstKind, ast::BlockStatement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn no_empty_diagnostic(stmt_kind: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected empty block statements")
        .with_help(format!("Remove this {stmt_kind} or add a comment inside it"))
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoEmpty {
    /// If set to `true`, allows an empty `catch` block without triggering the linter.
    allow_empty_catch: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows empty block statements.
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
    recommended,
    suggestion,
    config = NoEmpty,
    version = "0.0.3",
    short_description = "Disallows empty block statements.",
);

impl Rule for NoEmpty {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
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
                    match parent {
                        AstKind::TryStatement(try_stmt)
                            if try_stmt
                                .finalizer
                                .as_ref()
                                .is_some_and(|finalizer| finalizer.span == block.span) =>
                        {
                            if try_stmt.handler.is_some()
                                && let Some(finally_kw_start) = find_finally_start(ctx, block)
                            {
                                fixer.delete_range(Span::new(finally_kw_start, block.span.end))
                            } else {
                                fixer.noop()
                            }
                        }
                        AstKind::TryStatement(try_stmt)
                            if try_stmt
                                .finalizer
                                .as_ref()
                                .is_some_and(|finalizer| !finalizer.body.is_empty()) =>
                        {
                            fixer.noop()
                        }
                        AstKind::IfStatement(if_stmt) if if_stmt.alternate.is_some() => {
                            if if_stmt.consequent.span() == block.span {
                                fixer.noop()
                            } else {
                                delete_statement(node, ctx, fixer)
                            }
                        }
                        AstKind::CatchClause(_) => fixer.noop(),
                        AstKind::TryStatement(_)
                        | AstKind::IfStatement(_)
                        | AstKind::WhileStatement(_)
                        | AstKind::DoWhileStatement(_)
                        | AstKind::ForStatement(_)
                        | AstKind::ForInStatement(_)
                        | AstKind::ForOfStatement(_)
                        | AstKind::LabeledStatement(_)
                        | AstKind::WithStatement(_) => {
                            delete_statement(ctx.nodes().parent_node(node.id()), ctx, fixer)
                        }
                        _ => delete_statement(node, ctx, fixer),
                    }
                });
            }
            AstKind::SwitchStatement(switch) if switch.cases.is_empty() => {
                ctx.diagnostic_with_suggestion(
                    no_empty_diagnostic("switch", switch.span),
                    |fixer| delete_statement(node, ctx, fixer),
                );
            }
            _ => {}
        }
    }
}

/// Deletes the statement `node`, keeping the enclosing statement valid:
/// `if (a) b(); else {}` -> `if (a) b();` and `if (a) while (b) {}` -> `if (a) {}`.
fn delete_statement<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    fixer: RuleFixer<'_, 'a>,
) -> RuleFix {
    let span = node.span();
    match ctx.nodes().parent_kind(node.id()) {
        AstKind::IfStatement(if_stmt)
            if if_stmt.alternate.as_ref().is_some_and(|alternate| alternate.span() == span) =>
        {
            let else_span = Span::new(if_stmt.consequent.span().end, span.start);
            if ctx.has_comments_between(else_span) {
                fixer.noop()
            } else {
                fixer.delete_range(Span::new(else_span.start, span.end))
            }
        }
        AstKind::IfStatement(_)
        | AstKind::WhileStatement(_)
        | AstKind::DoWhileStatement(_)
        | AstKind::ForStatement(_)
        | AstKind::ForInStatement(_)
        | AstKind::ForOfStatement(_)
        | AstKind::LabeledStatement(_)
        | AstKind::WithStatement(_) => fixer.replace(span, "{}"),
        _ => fixer.delete_range(span),
    }
}

fn find_finally_start(ctx: &LintContext, finally_clause: &BlockStatement) -> Option<u32> {
    let src = &ctx.source_text()[..finally_clause.span.start as usize];
    let mut chars = src.char_indices().rev().skip_while(|(_, ch)| ch.is_whitespace());
    let mut start = 0;

    for expected in "finally".chars().rev() {
        let (offset, ch) = chars.next()?;
        if ch != expected {
            return None;
        }
        start = offset;
    }

    #[expect(clippy::cast_possible_truncation)]
    let start = start as u32;
    (!ctx.is_inside_comment(start)).then_some(start)
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
        (
            "/*😀😀😀😀😀😀😀😀😀😀😀*/\ntry{a()}finally{}/*finally                      */",
            "/*😀😀😀😀😀😀😀😀😀😀😀*/\ntry{a()}finally{}/*finally                      */",
            None,
        ),
        (
            "/* é中😀 */ try { foo(); } catch (ex) { throw ex; } finally {}",
            "/* é中😀 */ try { foo(); } catch (ex) { throw ex; } ",
            None,
        ),
        (
            "try { foo(); } catch (ex) { throw ex; } finally\u{2003}\u{2028}{}",
            "try { foo(); } catch (ex) { throw ex; } ",
            None,
        ),
        (
            "/*é*/try{a()}catch(e){b()}finally//finally\n{}",
            "/*é*/try{a()}catch(e){b()}finally//finally\n{}",
            None,
        ),
        (
            "try{a()}catch(e){b()}finally//finally\n{}",
            "try{a()}catch(e){b()}finally//finally\n{}",
            None,
        ),
        ("try{a()}catch(e){b()}//finally\nfinally{}", "try{a()}catch(e){b()}//finally\n", None),
        ("if (ready) { start(); } else {}", "if (ready) { start(); }", None),
        ("if (ready) start(); else {}", "if (ready) start();", None),
        ("if (a) {} else { b(); }", "if (a) {} else { b(); }", None),
        ("if (a) { b(); } else if (c) {}", "if (a) { b(); }", None),
        ("if (a) b(); /* c */ else {}", "if (a) b(); /* c */ else {}", None),
        ("if (a) while (b) {}", "if (a) {}", None),
        ("if (a) switch (b) {}", "if (a) {}", None),
        ("function f() { g(); {} }", "function f() { g();  }", None),
        ("{ g(); {} }", "{ g();  }", None),
        ("try {} finally { g(); }", "try {} finally { g(); }", None),
    ];

    Tester::new(NoEmpty::NAME, NoEmpty::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
