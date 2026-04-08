use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn detect_no_csrf_before_method_override_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("CSRF middleware found before method-override")
        .with_help("Ensure that CSRF protection middleware is applied after method-override, not before. Otherwise method-override can bypass CSRF checks.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectNoCsrfBeforeMethodOverride;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects Express applications where CSRF middleware (`csrf()`) is applied
    /// before `methodOverride()`.
    ///
    /// ### Why is this bad?
    ///
    /// If `method-override` middleware is applied after CSRF protection, it can
    /// be used to change the HTTP method of a request after the CSRF check has
    /// already passed, effectively bypassing the protection.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// app.use(csrf());
    /// app.use(methodOverride());
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// app.use(methodOverride());
    /// app.use(csrf());
    /// ```
    DetectNoCsrfBeforeMethodOverride,
    oxc,
    suspicious,
    none
);

/// Extract the middleware name from `app.use(name())` pattern.
fn get_middleware_name<'a>(expr: &'a Expression<'a>) -> Option<(&'a str, Span)> {
    let Expression::CallExpression(call_expr) = expr else {
        return None;
    };

    let Expression::StaticMemberExpression(member) = &call_expr.callee else {
        return None;
    };

    if member.property.name != "use" {
        return None;
    }

    let arg = call_expr.arguments.first()?.as_expression()?;
    let Expression::CallExpression(inner_call) = arg else {
        return None;
    };

    let Expression::Identifier(callee) = &inner_call.callee else {
        return None;
    };

    Some((callee.name.as_str(), call_expr.span))
}

impl Rule for DetectNoCsrfBeforeMethodOverride {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut found_csrf = false;

        for stmt in &ctx.semantic().nodes().program().body {
            let oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) = stmt else {
                continue;
            };

            let Some((name, span)) = get_middleware_name(&expr_stmt.expression) else {
                continue;
            };

            if name == "csrf" {
                found_csrf = true;
            }

            if name == "methodOverride" && found_csrf {
                ctx.diagnostic(detect_no_csrf_before_method_override_diagnostic(span));
                return;
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "app.use(methodOverride()); app.use(csrf());",
        "app.use(helmet()); app.use(methodOverride());",
        "app.use(bodyParser());",
    ];

    let fail = vec!["app.use(csrf()); app.use(methodOverride());"];

    Tester::new(
        DetectNoCsrfBeforeMethodOverride::NAME,
        DetectNoCsrfBeforeMethodOverride::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
