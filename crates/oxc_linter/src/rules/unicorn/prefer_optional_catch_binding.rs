use oxc_ast::{AstKind, ast::BindingPattern};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_optional_catch_binding_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer omitting the catch binding parameter if it is unused")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferOptionalCatchBinding;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers omitting the catch binding parameter if it is unused
    ///
    /// ### Why is this bad?
    ///
    /// It is unnecessary to bind the error to a variable if it is not used.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// try {
    ///  // ...
    /// } catch (e) { }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// try {
    ///  // ...
    /// } catch { }
    /// ```
    PreferOptionalCatchBinding,
    unicorn,
    style,
    fix
);

impl Rule for PreferOptionalCatchBinding {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CatchParameter(catch_param) = node.kind() else {
            return;
        };
        let references_count = get_param_references_count(&catch_param.pattern, ctx);
        if references_count != 0 {
            return;
        }
        let AstKind::CatchClause(catch_clause) = ctx.nodes().parent_kind(node.id()) else {
            return;
        };
        ctx.diagnostic_with_fix(
            prefer_optional_catch_binding_diagnostic(catch_param.pattern.span()),
            |fixer| {
                let mut start = catch_clause.span().start + 5;
                let total_param = Span::new(start, catch_param.span().start);
                let total_param_value = ctx.source_range(total_param);
                let plus_space: u32 = total_param_value
                    .as_bytes()
                    .iter()
                    .position(|x| !x.is_ascii_whitespace())
                    .unwrap_or(0)
                    .try_into()
                    .unwrap();
                start += plus_space;
                let end = catch_clause.body.span().start;
                let span = Span::new(start, end);
                fixer.delete(&span)
            },
        );
    }
}

fn get_param_references_count(binding_pat: &BindingPattern, ctx: &LintContext) -> usize {
    match &binding_pat {
        BindingPattern::BindingIdentifier(binding_ident) => {
            ctx.semantic().symbol_references(binding_ident.symbol_id()).count()
        }
        BindingPattern::ObjectPattern(object_pat) => {
            let mut count = 0;

            for prop in &object_pat.properties {
                count += get_param_references_count(&prop.value, ctx);
            }

            if let Some(rest) = &object_pat.rest {
                count += get_param_references_count(&rest.argument, ctx);
            }

            count
        }
        BindingPattern::AssignmentPattern(_) => 1,
        BindingPattern::ArrayPattern(array_pat) => {
            let mut count = 0;

            for element in (&array_pat.elements).into_iter().flatten() {
                count += get_param_references_count(element, ctx);
            }

            count
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "try {} catch {}",
        "try {} catch {
                error
            }",
        "try {} catch(used) {
                console.error(used);
            }",
        "try {} catch(usedInADeeperScope) {
                function foo() {
                    function bar() {
                        console.error(usedInADeeperScope);
                    }
                }
            }",
        "try {} catch ({message}) {alert(message)}",
        "try {} catch ({cause: {message}}) {alert(message)}",
        "try {} catch({nonExistsProperty = thisWillExecute()}) {}",
    ];

    let fail = vec![
        "try {} catch (_) {}",
        "try {} catch (foo) {
                function bar(foo) {}
            }",
        "try {} catch (outer) {
                try {} catch (inner) {
                }
            }
            try {
                try {} catch (inTry) {
                }
            } catch (another) {
                try {} catch (inCatch) {
                }
            } finally {
                try {} catch (inFinally) {
                }
            }",
        "try {} catch (theRealErrorName) {}",
        "/* comment */
            try {
                /* comment */
                // comment
            } catch (
                /* comment */
                // comment
                unused
                /* comment */
                // comment
            ) {
                /* comment */
                // comment
            }
            /* comment */",
        "try    {    } catch    (e)
                {    }",
        "try {} catch(e) {}",
        "try {} catch (e){}",
        "try {} catch ({}) {}",
        "try {} catch ({message}) {}",
        "try {} catch ({message: notUsedMessage}) {}",
        "try {} catch ({cause: {message}}) {}",
    ];

    let fix = vec![
        ("try {} catch (_) {}", "try {} catch {}"),
        ("try {} catch (theRealErrorName) {}", "try {} catch {}"),
        (
            "try    {    } catch    (e)
                    {    }",
            "try    {    } catch    {    }",
        ),
        ("try {} catch(e) {}", "try {} catch{}"),
        ("try {} catch (e){}", "try {} catch {}"),
        ("try {} catch ({}) {}", "try {} catch {}"),
        ("try {} catch ({message}) {}", "try {} catch {}"),
        ("try {} catch ({message: notUsedMessage}) {}", "try {} catch {}"),
        ("try {} catch ({cause: {message}}) {}", "try {} catch {}"),
    ];

    Tester::new(PreferOptionalCatchBinding::NAME, PreferOptionalCatchBinding::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
