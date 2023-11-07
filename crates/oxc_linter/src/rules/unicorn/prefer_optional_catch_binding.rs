use oxc_ast::{
    ast::{BindingPattern, BindingPatternKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-optional-catch-binding): Prefer omitting the catch binding parameter if it is unused")]
#[diagnostic(severity(warning))]
struct PreferOptionalCatchBindingDiagnostic(#[label] pub Span);

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
    /// ### Example
    /// ```javascript
    /// // Bad
    /// try {
    ///  // ...
    /// } catch (e) { }
    ///
    /// // Good
    /// try {
    ///  // ...
    /// } catch { }
    /// ```
    PreferOptionalCatchBinding,
    style
);

impl Rule for PreferOptionalCatchBinding {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CatchClause(catch_clause) = node.kind() else { return };

        let Some(catch_param) = &catch_clause.param else { return };

        let references_count = get_param_references_count(catch_param, ctx);

        if references_count != 0 {
            return;
        }

        ctx.diagnostic(PreferOptionalCatchBindingDiagnostic(catch_param.span()));
    }
}

fn get_param_references_count(binding_pat: &BindingPattern, ctx: &LintContext) -> usize {
    match &binding_pat.kind {
        BindingPatternKind::BindingIdentifier(binding_ident) => {
            ctx.semantic().symbol_references(binding_ident.symbol_id.get().unwrap()).count()
        }
        BindingPatternKind::ObjectPattern(object_pat) => {
            let mut count = 0;

            for prop in &object_pat.properties {
                count += get_param_references_count(&prop.value, ctx);
            }

            if let Some(rest) = &object_pat.rest {
                count += get_param_references_count(&rest.argument, ctx);
            }

            count
        }
        BindingPatternKind::AssignmentPattern(_) => 1,
        BindingPatternKind::ArrayPattern(array_pat) => {
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
        r#"try {} catch {}"#,
        r#"try {} catch ({message}) {alert(message)}"#,
        r#"try {} catch ({cause: {message}}) {alert(message)}"#,
        r#"try {} catch({nonExistsProperty = thisWillExecute()}) {}"#,
    ];

    let fail = vec![
        r#"try {} catch (_) {}"#,
        r#"try {} catch (theRealErrorName) {}"#,
        r#"try    {    } catch    (e)  
			  	  {    }"#,
        r#"try {} catch(e) {}"#,
        r#"try {} catch (e){}"#,
        r#"try {} catch ({}) {}"#,
        r#"try {} catch ({message}) {}"#,
        r#"try {} catch ({message: notUsedMessage}) {}"#,
        r#"try {} catch ({cause: {message}}) {}"#,
    ];

    Tester::new_without_config(PreferOptionalCatchBinding::NAME, pass, fail).test_and_snapshot();
}
