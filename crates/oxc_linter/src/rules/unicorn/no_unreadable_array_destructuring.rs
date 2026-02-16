use oxc_allocator::Vec;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

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
    /// ### Examples
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
    style,
    pending
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
        match node.kind() {
            AstKind::ArrayPattern(array_pattern)
                if is_unreadable_array_destructuring(
                    &array_pattern.elements,
                    array_pattern.rest.as_ref(),
                ) =>
            {
                ctx.diagnostic(no_unreadable_array_destructuring_diagnostic(array_pattern.span));
            }
            AstKind::ArrayAssignmentTarget(array_pattern)
                if is_unreadable_array_destructuring(
                    &array_pattern.elements,
                    array_pattern.rest.as_ref(),
                ) =>
            {
                ctx.diagnostic(no_unreadable_array_destructuring_diagnostic(array_pattern.span));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const [, foo] = parts;",
        "const [foo] = parts;",
        "const [foo,,bar] = parts;",
        "const [foo,   ,     bar] = parts;",
        "const [foo,] = parts;",
        "const [foo,,] = parts;",
        "const [foo,, bar,, baz] = parts;",
        "[,foo] = bar;",
        "({parts: [,foo]} = bar);",
        "function foo([, bar]) {}",
        "function foo([bar]) {}",
        "function foo([bar,,baz]) {}",
        "function foo([bar,   ,     baz]) {}",
        "function foo([bar,]) {}",
        "function foo([bar,,]) {}",
        "function foo([bar,, baz,, qux]) {}",
        "const [, ...rest] = parts;",
        // This is stupid, but valid code
        "const [,,] = parts;",
    ];

    let fail = vec![
        "const [,, foo] = parts;",
        "const [foo,,, bar] = parts;",
        "const [foo,,,] = parts;",
        "const [foo, bar,, baz ,,, qux] = parts;",
        "[,, foo] = bar;",
        "({parts: [,, foo]} = bar);",
        "function foo([,, bar]) {}",
        "function foo([bar,,, baz]) {}",
        "function foo([bar,,,]) {}",
        "function foo([bar, baz,, qux ,,, quux]) {}",
        "const [,,...rest] = parts;",
        // This is stupid, but valid code
        "const [,,,] = parts;",
        // Should add parentheses to array
        "const [,,...rest] = new Array;",
        "const [,,...rest] = (0, foo);",
        "let [,,thirdElement] = new Array;",
        "var [,,thirdElement] = (((0, foo)));",
        "let [,,[,,thirdElementInThirdElement]] = foo",
        // Variable is not `Identifier`
        "let [,,{propertyOfThirdElement}] = foo",
        // Multiple declarations
        "let [,,thirdElement] = foo, anotherVariable = bar;",
        // Default value
        "let [,,thirdElement = {}] = foo;",
        "for (const [, , id] of shuffle(list)) {}",
        // Space after keyword
        "let[,,thirdElement] = foo;",
        "let[,,...thirdElement] = foo;",
        "const[,,thirdElement] = foo;",
        "const[,,...thirdElement] = foo;",
        "var[,,thirdElement] = foo;",
        "var[,,...thirdElement] = foo;",
        "let[]=[],[,,thirdElement] = foo;",
    ];

    Tester::new(
        NoUnreadableArrayDestructuring::NAME,
        NoUnreadableArrayDestructuring::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
