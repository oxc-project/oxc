use oxc_ast::{AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-empty): Disallow empty block statements")]
#[diagnostic(severity(warning), help("Add comments inside {0} statement"))]
struct NoEmptyDiagnostic(&'static str, #[label("Empty {0} statement")] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEmpty;

impl Rule for NoEmpty {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.get().kind() {
            AstKind::BlockStatement(block) if block.body.is_empty() => {
                // TODO: check comment
                if !matches!(ctx.parent_kind(node), AstKind::CatchClause(_)) {
                    ctx.diagnostic(NoEmptyDiagnostic("block", block.span));
                }
            }
            AstKind::SwitchStatement(switch) if switch.cases.is_empty() => {
                ctx.diagnostic(NoEmptyDiagnostic("switch", switch.span));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::rules::RuleEnum;
    use crate::tester::Tester;

    let pass = vec![
        "if (foo) { bar() }",
        "while (foo) { bar() }",
        "for (;foo;) { bar() }",
        "try { foo() } catch (ex) { foo() }",
        "switch(foo) {case 'foo': break;}",
        "(function() { }())",
        "var foo = () => {};",
        "function foo() { }",
        "if (foo) {/* empty */}",
        "while (foo) {/* empty */}",
        "for (;foo;) {/* empty */}",
        "try { foo() } catch (ex) {/* empty */}",
        "try { foo() } catch (ex) {// empty\n}",
        "try { foo() } finally {// empty\n}",
        "try { foo() } finally {// test\n}",
        "try { foo() } finally {\n \n // hi i am off no use\n}",
        "try { foo() } catch (ex) {/* test111 */}",
        "if (foo) { bar() } else { // nothing in me \n}",
        "if (foo) { bar() } else { /**/ \n}",
        "if (foo) { bar() } else { // \n}",
        "try { foo(); } catch (ex) {}",
        "try { foo(); } catch (ex) {} finally { bar(); }",
    ];

    let fail = vec![
        "try {} catch (ex) {throw ex}",
        "try { foo() } catch (ex) {}",
        "try { foo() } catch (ex) {throw ex} finally {}",
        "if (foo) {}",
        "while (foo) {}",
        "for (;foo;) {}",
        "switch(foo) {}",
        "switch (foo) { /* empty */ }",
        "try {} catch (ex) {}",
        "try { foo(); } catch (ex) {} finally {}",
        "try {} catch (ex) {} finally {}",
        "try { foo(); } catch (ex) {} finally {}",
    ];

    Tester::new("no-empty", RuleEnum::NoEmpty(NoEmpty), pass, fail).test_and_snapshot();
}
