use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

fn consistent_date_clone_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary `.getTime()` call")
        .with_help("Prefer passing `Date` directly to the constructor when cloning")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentDateClone;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The Date constructor can clone a `Date` object directly when passed as an argument,
    /// making timestamp conversion unnecessary. This rule enforces the use of the
    /// direct `Date` cloning instead of using `.getTime()` for conversion.
    ///
    /// ### Why is this bad?
    ///
    /// Using `.getTime()` to convert a `Date` object to a timestamp and then back to a
    /// `Date` is redundant and unnecessary. Simply passing the `Date` object to the
    /// `Date` constructor is cleaner and more efficient.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// new Date(date.getTime());
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// new Date(date);
    /// ```
    ConsistentDateClone,
    unicorn,
    style,
    fix
);

impl Rule for ConsistentDateClone {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else {
            return;
        };

        if !(expr.callee.is_specific_id("Date")
            && expr.arguments.len() == 1
            && expr.type_arguments.is_none())
        {
            return;
        }

        let Argument::CallExpression(expr) = &expr.arguments[0] else {
            return;
        };

        let Expression::StaticMemberExpression(callee) = &expr.callee else {
            return;
        };

        if callee.property.name.as_str() == "getTime"
            && expr.arguments.is_empty()
            && !expr.optional
            && !callee.optional
        {
            ctx.diagnostic_with_fix(consistent_date_clone_diagnostic(expr.span), |fixer| {
                fixer.delete_range(Span::new(callee.object.span().end, expr.span.end))
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "new Date(date)",
        "date.getTime()",
        "new Date(...date.getTime())",
        "new Date(getTime())",
        "new Date(date.getTime(), extraArgument)",
        "new Date(date.not_getTime())",
        "new Date(date?.getTime())",
        "new NotDate(date.getTime())",
        "new Date(date[getTime]())",
        "new Date(date.getTime(extraArgument))",
        "Date(date.getTime())",
        // TODO: We may support these cases in future
        "new Date(
				date.getFullYear(),
				date.getMonth(),
				date.getDate(),
				date.getHours(),
				date.getMinutes(),
				date.getSeconds(),
				date.getMilliseconds(),
			);",
        "new Date(
				date.getFullYear(),
				date.getMonth(),
				date.getDate(),
				date.getHours(),
				date.getMinutes(),
				date.getSeconds(),
			);",
    ];

    let fail = vec![
        "new Date(date.getTime())",
        "new Date(date.getTime(),)",
        "new Date((0, date).getTime())",
        "new Date(date.getTime(/* comment */))",
        "new Date(date./* comment */getTime())",
    ];

    let fix = vec![
        ("new Date(date.getTime())", "new Date(date)"),
        ("new Date(date.getTime(),)", "new Date(date,)"),
        ("new Date((0, date).getTime())", "new Date((0, date))"),
        ("new Date(date.getTime(/* comment */))", "new Date(date)"),
        ("new Date(date./* comment */getTime())", "new Date(date)"),
    ];

    Tester::new(ConsistentDateClone::NAME, ConsistentDateClone::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
