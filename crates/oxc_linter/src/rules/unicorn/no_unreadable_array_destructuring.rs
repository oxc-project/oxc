use oxc_allocator::Vec;
use oxc_ast::{ast::AssignmentTarget, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_unreadable_array_destructuring_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Array destructuring may not contain consecutive ignored values.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnreadableArrayDestructuring;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unreadable array destructuring
    ///
    /// ### Why is this bad?
    ///
    /// Destructuring is very useful, but it can also make some code harder to read.
    /// This rule prevents ignoring consecutive values when destructuring from an array.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const [,, foo] = parts;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const [foo] = parts;
    /// ```
    NoUnreadableArrayDestructuring,
    unicorn,
    style
);

fn is_unreadable_array_destructuring<T, U>(elements: &Vec<Option<T>>, rest: Option<&U>) -> bool {
    if elements.len() >= 3 && elements.windows(2).any(|window| window.iter().all(Option::is_none)) {
        return true;
    }

    if elements.len() == 2 && elements.iter().all(std::option::Option::is_none) && rest.is_some() {
        return true;
    }

    false
}

impl Rule for NoUnreadableArrayDestructuring {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ArrayPattern(array_pattern) = node.kind() {
            if is_unreadable_array_destructuring(
                &array_pattern.elements,
                array_pattern.rest.as_ref(),
            ) {
                ctx.diagnostic(no_unreadable_array_destructuring_diagnostic(array_pattern.span));
            }
        }

        if let AstKind::AssignmentTarget(AssignmentTarget::ArrayAssignmentTarget(array_pattern)) =
            node.kind()
        {
            if is_unreadable_array_destructuring(
                &array_pattern.elements,
                array_pattern.rest.as_ref(),
            ) {
                ctx.diagnostic(no_unreadable_array_destructuring_diagnostic(array_pattern.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r" [, foo] = parts;", None),
        (r" [foo] = parts;", None),
        (r" [foo,,bar] = parts;", None),
        (r" [foo,   ,     bar] = parts;", None),
        (r" [foo,] = parts;", None),
        (r" [foo,,] = parts;", None),
        (r" [foo,, bar,, baz] = parts;", None),
        (r"[,foo] = bar;", None),
        (r"({parts: [,foo]} = bar);", None),
        (r"function foo([, bar]) {}", None),
        (r"function foo([bar]) {}", None),
        (r"function foo([bar,,baz]) {}", None),
        (r"function foo([bar,   ,     baz]) {}", None),
        (r"function foo([bar,]) {}", None),
        (r"function foo([bar,,]) {}", None),
        (r"function foo([bar,, baz,, qux]) {}", None),
        (r" [, ...rest] = parts;", None),
        // This is stupid, but valid code
        (r" [,,] = parts;", None),
    ];

    let fail = vec![
        (r"const [,, foo] = parts;", None),
        (r"const [foo,,, bar] = parts;", None),
        (r"const [foo,,,] = parts;", None),
        (r"const [foo, bar,, baz ,,, qux] = parts;", None),
        (r"[,, foo] = bar;", None),
        (r"({parts: [,, foo]} = bar);", None),
        (r"function foo([,, bar]) {}", None),
        (r"function foo([bar,,, baz]) {}", None),
        (r"function foo([bar,,,]) {}", None),
        (r"function foo([bar, baz,, qux ,,, quux]) {}", None),
        (r"const [,,...rest] = parts;", None),
        // This is stupid, but valid code
        (r"const [,,,] = parts;", None),
        // Should add parentheses to array
        (r"const [,,...rest] = new Array;", None),
        (r"const [,,...rest] = (0, foo);", None),
        (r"let [,,thirdElement] = new Array;", None),
        (r"var [,,thirdElement] = (((0, foo)));", None),
        // Variable is not `Identifier`
        (r"let [,,[,,thirdElementInThirdElement]] = foo", None),
        (r"let [,,{propertyOfThirdElement}] = foo", None),
        // Multiple declarations
        (r"let [,,thirdElement] = foo, anotherVariable = bar;", None),
        // Default value
        (r"let [,,thirdElement = {}] = foo;", None),
        (r"for (const [, , id] of shuffle(list)) {}", None),
        // Space after keyword
        (r"let[,,thirdElement] = foo;", None),
        (r"let[,,...thirdElement] = foo;", None),
        (r"const[,,thirdElement] = foo;", None),
        (r"const[,,...thirdElement] = foo;", None),
        (r"var[,,thirdElement] = foo;", None),
        (r"var[,,...thirdElement] = foo;", None),
        (r"let[]=[],[,,thirdElement] = foo;", None),
    ];

    Tester::new(
        NoUnreadableArrayDestructuring::NAME,
        NoUnreadableArrayDestructuring::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
