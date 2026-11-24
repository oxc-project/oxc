use oxc_allocator::ArenaVec;
use oxc_ast::{
    AstKind,
    ast::{ArrayAssignmentTarget, AssignmentTargetMaybeDefault, SimpleAssignmentTarget},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unreadable_array_destructuring_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Array destructuring may not contain consecutive ignored values.")
        .with_label(span)
}

fn property_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not assign destructured values to object properties.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnreadableArrayDestructuring;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows destructuring values from an array in ways that are difficult to read.
    ///
    /// ### Why is this bad?
    ///
    /// Destructuring can be very useful, but it can also make some code harder to read.
    /// This rule prevents ignoring consecutive values (e.g. `let [,,foo] = array`)
    /// and assigning destructured values to object properties (e.g. `[object.property] = parts`)
    /// when destructuring from an array.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const [,, foo] = parts;
    /// const [,,...rest] = parts;
    /// [object.property] = parts;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const [foo] = parts;
    /// const foo = parts[3];
    /// const rest = parts.slice(2);
    /// const [value] = parts;
    /// object.property = value;
    ///
    /// // One is fine
    /// const [, foo] = parts;
    /// ```
    NoUnreadableArrayDestructuring,
    unicorn,
    style,
    recommended,
    pending,
    version = "0.0.19",
    short_description =
        "Disallows destructuring values from an array in ways that are difficult to read.",
);

fn is_unreadable_array_destructuring<T, U>(
    elements: &ArenaVec<'_, Option<T>>,
    rest: Option<&U>,
) -> bool {
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
            AstKind::ArrayAssignmentTarget(array_pattern) => {
                if is_unreadable_array_destructuring(
                    &array_pattern.elements,
                    array_pattern.rest.as_ref(),
                ) {
                    ctx.diagnostic(no_unreadable_array_destructuring_diagnostic(
                        array_pattern.span,
                    ));
                }
                check_property_assignments(array_pattern, ctx);
            }
            _ => {}
        }
    }
}

fn member_expression_target_span(target: &SimpleAssignmentTarget<'_>) -> Option<Span> {
    if let Some(member_expr) = target.as_member_expression() {
        return Some(member_expr.span());
    }
    target.get_expression()?.get_inner_expression().as_member_expression().map(GetSpan::span)
}

fn check_property_assignments(array_pattern: &ArrayAssignmentTarget<'_>, ctx: &LintContext<'_>) {
    for element in array_pattern.elements.iter().flatten() {
        let simple_target = match element {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                with_default.binding.as_simple_assignment_target()
            }
            _ => element.as_simple_assignment_target(),
        };
        if let Some(span) = simple_target.and_then(member_expression_target_span) {
            ctx.diagnostic(property_assignment_diagnostic(span));
        }
    }

    if let Some(rest) = &array_pattern.rest
        && let Some(span) =
            rest.target.as_simple_assignment_target().and_then(member_expression_target_span)
    {
        ctx.diagnostic(property_assignment_diagnostic(span));
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
        "[value] = array;",
        "[value = object.property] = array;",
        "[{property: object.property}] = array;",
        "[{[object.key]: value}] = array;",
        "({property: object.property} = source);",
        "[...{property: object.property}] = array;",
        "const foo = {property: object.property};",
        "[...rest] = array;",
        "[value!] = array;",
        "[value as any] = array;",
        "[value satisfies unknown] = array;",
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
        "[this.property] = array;",
        "[object.property] = array;",
        "[object[key]] = array;",
        "[(object.property)] = array;",
        "[object.property = defaultValue] = array;",
        "[object.property! = defaultValue] = array;",
        "[(condition ? first : second).property] = array;",
        "[this.property.value] = array;",
        "[...object.property] = array;",
        "[...[object.property]] = array;",
        "[object.a, ...object.b] = array;",
        "[object.property!] = array;",
        "[object.property as any] = array;",
        "[(object.property as any)!] = array;",
        "[object.property satisfies unknown] = array;",
        "[...object.property!] = array;",
        "class Foo { #prop; bar(array: unknown[]) { [this.#prop] = array; } }",
        "for ([this.property] of arrays) {}",
        "for ([this.property] in obj) {}",
        "({parts: [this.property]} = object);",
        "[[object.property]] = array;",
        "[,, object.property] = array;",
    ];

    Tester::new(
        NoUnreadableArrayDestructuring::NAME,
        NoUnreadableArrayDestructuring::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();

    let ts_pass = vec!["[<any>value] = array;"];
    let ts_fail = vec!["[<any>object.property] = array;"];
    Tester::new(
        NoUnreadableArrayDestructuring::NAME,
        NoUnreadableArrayDestructuring::PLUGIN,
        ts_pass,
        ts_fail,
    )
    .change_rule_path_extension("ts")
    .test();
}
