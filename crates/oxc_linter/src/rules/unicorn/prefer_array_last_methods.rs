use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression, StaticMemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_array_last_methods_diagnostic(
    span: Span,
    reversing_method: &str,
    method: &str,
    replacement: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Prefer `Array#{replacement}()` over `Array#{reversing_method}().{method}()`."
    ))
    .with_help(format!("Replace `.{reversing_method}().{method}()` with `.{replacement}()`."))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrayLastMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer last-oriented array methods over `Array#reverse()` or `Array#toReversed()`
    /// followed by `find()`, `findIndex()`, `indexOf()` or `reduce()`.
    ///
    /// ### Why is this bad?
    ///
    /// `reverse()` mutates the array and `toReversed()` copies it, only to search or fold it
    /// from the end. `findLast()`, `findLastIndex()`, `lastIndexOf()` and `reduceRight()` do
    /// that directly.
    ///
    /// Note that the suggested replacement is not always equivalent: `findIndex()` and
    /// `indexOf()` on a reversed array return an index into the reversed array, while
    /// `findLastIndex()` and `lastIndexOf()` return an index into the original one, and
    /// dropping `reverse()` no longer mutates the array.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const last = array.toReversed().find(item => item.ok);
    /// const total = array.reverse().reduce((sum, item) => sum + item, 0);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const last = array.findLast(item => item.ok);
    /// const total = array.reduceRight((sum, item) => sum + item, 0);
    /// ```
    PreferArrayLastMethods,
    unicorn,
    style,
    dangerous_suggestion,
    version = "next",
    short_description = "Prefer last-oriented array methods over `Array#reverse()` or `Array#toReversed()` followed by a method.",
);

impl Rule for PreferArrayLastMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else {
            return;
        };
        let Some(member) = get_static_method(call) else {
            return;
        };
        let method = member.property.name.as_str();
        let replacement = match method {
            "find" => "findLast",
            "findIndex" => "findLastIndex",
            "indexOf" => "lastIndexOf",
            "reduce" => "reduceRight",
            _ => return,
        };
        let Expression::CallExpression(reversing_call) = member.object.get_inner_expression()
        else {
            return;
        };
        if !reversing_call.arguments.is_empty() {
            return;
        }
        let Some(reversing_member) = get_static_method(reversing_call) else {
            return;
        };
        let reversing_method = reversing_member.property.name.as_str();
        if !matches!(reversing_method, "reverse" | "toReversed") {
            return;
        }

        let diagnostic = prefer_array_last_methods_diagnostic(
            member.property.span,
            reversing_method,
            method,
            replacement,
        );
        // `.reverse()` / `.toReversed()`, from the end of the array to the end of the call.
        let removed = Span::new(reversing_member.object.span().end, reversing_call.span.end);
        if ctx.has_comments_between(removed) {
            ctx.diagnostic(diagnostic);
            return;
        }
        ctx.diagnostic_with_dangerous_suggestion(diagnostic, |fixer| {
            let fixer = fixer.for_multifix();
            let mut fix = fixer
                .new_fix_with_capacity(2)
                .with_message(format!("Replace with `.{replacement}()`"));
            fix.push(fixer.delete_range(removed));
            fix.push(fixer.replace(member.property.span, replacement));
            fix
        });
    }
}

/// `object.method(…)`, without optional chaining.
fn get_static_method<'a, 'b>(
    call: &'b CallExpression<'a>,
) -> Option<&'b StaticMemberExpression<'a>> {
    if call.optional {
        return None;
    }
    match call.callee.get_inner_expression().as_member_expression()? {
        MemberExpression::StaticMemberExpression(member) if !member.optional => Some(member),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.findLast(fn)",
        "array.find(fn)",
        "array.reverse()",
        "array.reverse().map(fn)",
        "array.reverse().findLast(fn)",
        "array.reverse(foo).find(fn)",
        "array.toSorted().find(fn)",
        "array.reverse?.().find(fn)",
        "array?.reverse().find(fn)",
        "array.reverse()?.find(fn)",
        "array.reverse().find?.(fn)",
        "array[reverse]().find(fn)",
        "array.reverse()[find](fn)",
        "array['reverse']().find(fn)",
        "reverse().find(fn)",
    ];

    let fail = vec![
        "array.reverse().find(fn)",
        "array.toReversed().find(fn)",
        "array.reverse().findIndex(fn)",
        "array.toReversed().indexOf(foo)",
        "array.reverse().reduce(fn, 0)",
        "(array.reverse()).find(fn)",
        "(array).toReversed().find(fn)",
        "foo.bar().toReversed().find(fn)",
        "array\n  .toReversed()\n  .find(fn)",
        "array./* comment */toReversed().find(fn)",
        "array.toReversed(/* comment */).find(fn)",
        "array.toReversed()./* comment */find(fn)",
    ];

    let fix = vec![
        ("array.reverse().find(fn)", "array.findLast(fn)", None),
        ("array.toReversed().find(fn)", "array.findLast(fn)", None),
        ("array.reverse().findIndex(fn)", "array.findLastIndex(fn)", None),
        ("array.toReversed().indexOf(foo)", "array.lastIndexOf(foo)", None),
        ("array.reverse().reduce(fn, 0)", "array.reduceRight(fn, 0)", None),
        ("(array.reverse()).find(fn)", "(array).findLast(fn)", None),
        ("(array).toReversed().find(fn)", "(array).findLast(fn)", None),
        ("foo.bar().toReversed().find(fn)", "foo.bar().findLast(fn)", None),
        ("array\n  .toReversed()\n  .find(fn)", "array\n  .findLast(fn)", None),
        ("array.toReversed()./* comment */find(fn)", "array./* comment */findLast(fn)", None),
    ];

    Tester::new(PreferArrayLastMethods::NAME, PreferArrayLastMethods::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
