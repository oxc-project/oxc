use oxc_ast::{
    ast::{match_member_expression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn no_array_for_each_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `Array#forEach`")
        .with_help("Replace it with a for` loop. For loop is faster, more readable, and you can use `break` or `return` to exit early.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayForEach;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids the use of `Array#forEach` in favor of a for loop.
    ///
    /// ### Why is this bad?
    ///
    /// Benefits of [`for…of` statement](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...of) over the `forEach` method can include:
    ///
    /// - Faster
    /// - Better readability
    /// - Ability to exit early with `break` or `return`
    ///
    /// Additionally, using `for…of` has great benefits if you are using TypeScript, because it does not cause a function boundary to be crossed. This means that type-narrowing earlier on in the current scope will work properly while inside of the loop (without having to re-type-narrow). Furthermore, any mutated variables inside of the loop will picked up on for the purposes of determining if a variable is being used.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = [1, 2, 3];
    /// foo.forEach((element) => { /* ... */ });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = [1, 2, 3];
    /// for (const element of foo) { /* ... */ }
    /// ```
    NoArrayForEach,
    unicorn,
    restriction,
    pending
);

impl Rule for NoArrayForEach {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = (call_expr).callee.get_member_expr() else {
            return;
        };

        let Some((_span, _)) = member_expr.static_property_info() else {
            return;
        };

        if is_method_call(call_expr, None, Some(&["forEach"]), None, None)
            && !member_expr.is_computed()
        {
            let object = member_expr.object();

            match object {
                Expression::Identifier(ident) => {
                    if IGNORED_OBJECTS.contains(ident.name.as_str()) {
                        return;
                    }
                }
                match_member_expression!(Expression) => {
                    if let Some(name) = object.to_member_expression().static_property_name() {
                        if IGNORED_OBJECTS.contains(name) {
                            return;
                        }
                    }
                }
                _ => {}
            }

            let Some((span, _)) = member_expr.static_property_info() else {
                return;
            };

            ctx.diagnostic(no_array_for_each_diagnostic(span));
        }
    }
}

pub const IGNORED_OBJECTS: phf::Set<&'static str> = phf_set! {
    "Children",
    "r",
    "pIteration",
};

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"new foo.forEach(element => bar())",
        r"forEach(element => bar())",
        r"foo.notForEach(element => bar())",
        r"React.Children.forEach(children, (child) => {});",
        r"Children.forEach(children, (child) => {});",
    ];

    let fail = vec![
        r"foo.forEach?.(element => bar(element))",
        r"1?.forEach((a, b) => call(a, b))",
        r"array.forEach((arrayInArray) => arrayInArray.forEach(element => bar(element)));",
        r"array.forEach((arrayInArray) => arrayInArray?.forEach(element => bar(element)));",
        r"array.forEach((element, index = element) => {})",
        r"array.forEach(({foo}, index = foo) => {})",
        r"array.forEach((element, {bar = element}) => {})",
        r"array.forEach(({foo}, {bar = foo}) => {})",
        r"foo.forEach(function(element, element1) {})",
        r"foo.forEach(function element(element, element1) {})",
        r"this._listeners.forEach((listener: () => void) => listener());",
        r"return foo.forEach(element => {bar(element)});",
    ];

    Tester::new(NoArrayForEach::NAME, NoArrayForEach::PLUGIN, pass, fail).test_and_snapshot();
}
