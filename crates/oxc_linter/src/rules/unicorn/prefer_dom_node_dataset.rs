use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-dom-node-dataset): Prefer using `dataset` over `{1}`")]
#[diagnostic(severity(warning))]
struct PreferDomNodeDatasetDiagnostic(#[label] pub Span, pub String);

#[derive(Debug, Default, Clone)]
pub struct PreferDomNodeDataset;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Use [`.dataset`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dataset) on DOM elements over `getAttribute(…)`, `.setAttribute(…)`, `.removeAttribute(…)` and `.hasAttribute(…)`.
    ///
    /// ### Why is this bad?
    ///
    /// The `dataset` property is a map of strings that contains all the `data-*` attributes from the element. It is a convenient way to access all of them at once.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// element.setAttribute('data-unicorn', '🦄');
    ///
    /// // Good
    /// element.dataset.unicorn = '🦄';
    /// ```
    PreferDomNodeDataset,
    pedantic
);

impl Rule for PreferDomNodeDataset {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else { return };

        if member_expr.is_computed() {
            return;
        }

        let Some(method_name) = member_expr.static_property_name() else { return };

        match method_name {
            "setAttribute" => {
                if call_expr.arguments.len() != 2 {
                    return;
                }
            }
            "getAttribute" | "removeAttribute" | "hasAttribute" => {
                if call_expr.arguments.len() != 1 {
                    return;
                }
            }
            _ => return,
        }

        let Argument::Expression(Expression::StringLiteral(string_lit)) = &call_expr.arguments[0]
        else {
            return;
        };

        if !string_lit.value.to_lowercase().starts_with("data-") {
            return;
        }

        ctx.diagnostic(PreferDomNodeDatasetDiagnostic(string_lit.span, method_name.to_string()));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"element.dataset.unicorn = '🦄';"#,
        r#"element.dataset['unicorn'] = '🦄';"#,
        r#"new element.setAttribute('data-unicorn', '🦄');"#,
        r#"setAttribute('data-unicorn', '🦄');"#,
        r#"element['setAttribute']('data-unicorn', '🦄');"#,
        r#"element[setAttribute]('data-unicorn', '🦄');"#,
        r#"element.foo('data-unicorn', '🦄');"#,
        r#"element.setAttribute('data-unicorn', '🦄', 'extra');"#,
        r#"element.setAttribute('data-unicorn');"#,
        r#"element.setAttribute(...argumentsArray, ...argumentsArray2)"#,
        r#"element.setAttribute(`data-unicorn`, '🦄');"#,
        r#"element.setAttribute(0, '🦄');"#,
        r#"element.setAttribute('foo-unicorn', '🦄');"#,
        r#"element.setAttribute('data', '🦄');"#,
        r#"delete element.dataset.unicorn;"#,
        r#"delete element.dataset["unicorn"];"#,
        r#"new element.removeAttribute("data-unicorn");"#,
        r#"removeAttribute("data-unicorn");"#,
        r#"element["removeAttribute"]("data-unicorn");"#,
        r#"element[removeAttribute]("data-unicorn");"#,
        r#"element.foo("data-unicorn");"#,
        r#"element.removeAttribute("data-unicorn", "extra");"#,
        r#"element.removeAttribute();"#,
        r#"element.removeAttribute(...argumentsArray, ...argumentsArray2)"#,
        r#"element.removeAttribute(`data-unicorn`);"#,
        r#"element.removeAttribute(0);"#,
        r#"element.removeAttribute("foo-unicorn");"#,
        r#"element.removeAttribute("data");"#,
        r#""unicorn" in element.dataset"#,
        r#"element.dataset.hasOwnProperty("unicorn")"#,
        r#"Object.prototype.hasOwnProperty.call(element.dataset, "unicorn")"#,
        r#"Object.hasOwn(element.dataset, "unicorn")"#,
        r#"Reflect.has(element.dataset, "unicorn")"#,
        r#"new element.hasAttribute("data-unicorn");"#,
        r#"hasAttribute("data-unicorn");"#,
        r#"element["hasAttribute"]("data-unicorn");"#,
        r#"element[hasAttribute]("data-unicorn");"#,
        r#"element.foo("data-unicorn");"#,
        r#"element.hasAttribute("data-unicorn", "extra");"#,
        r#"element.hasAttribute();"#,
        r#"element.hasAttribute(...argumentsArray, ...argumentsArray2)"#,
        r#"element.hasAttribute(`data-unicorn`);"#,
        r#"element.hasAttribute(0);"#,
        r#"element.hasAttribute("foo-unicorn");"#,
        r#"element.hasAttribute("data");"#,
        r#"element.dataset.unicorn"#,
        r#"new element.getAttribute("data-unicorn");"#,
        r#"getAttribute("data-unicorn");"#,
        r#"element["getAttribute"]("data-unicorn");"#,
        r#"element[getAttribute]("data-unicorn");"#,
        r#"element.foo("data-unicorn");"#,
        r#"element.getAttribute("data-unicorn", "extra");"#,
        r#"element.getAttribute();"#,
        r#"element.getAttribute(...argumentsArray, ...argumentsArray2)"#,
        r#"element.getAttribute(`data-unicorn`);"#,
        r#"element.getAttribute(0);"#,
        r#"element.getAttribute("foo-unicorn");"#,
        r#"element.getAttribute("data");"#,
    ];

    let fail = vec![
        r#"element.setAttribute('data-unicorn', '🦄');"#,
        r#"element.setAttribute('data-🦄', '🦄');"#,
        r#"element.setAttribute('data-ゆ', 'ゆ');"#,
        r#"element.setAttribute('data-foo2', '🦄');"#,
        r#"element.setAttribute('data-foo:bar', 'zaz');"#,
        r#"element.setAttribute("data-foo:bar", "zaz");"#,
        r#"element.setAttribute('data-foo.bar', 'zaz');"#,
        r#"element.setAttribute('data-foo-bar', 'zaz');"#,
        r#"element.setAttribute('data-foo', /* comment */ 'bar');"#,
        r#"element.querySelector('#selector').setAttribute('data-AllowAccess', true);"#,
        r#"element.setAttribute("data-", "🦄");"#,
        r#"element.setAttribute("data--foo", "🦄");"#,
        r#"element.setAttribute("DATA--FOO", "🦄");"#,
        r#"element.setAttribute("DATA- ", "🦄");"#,
        r#"element.setAttribute("DATA-Foo-bar", "🦄");"#,
        r#"optional?.element.setAttribute("data-unicorn", "🦄");"#,
        r#"console.log(element.setAttribute("data-unicorn", "🦄"))"#,
        r#"element.removeAttribute('data-unicorn');"#,
        r#"element.removeAttribute("data-unicorn");"#,
        r#"element.removeAttribute("data-unicorn",);"#,
        r#"element.removeAttribute("data-🦄");"#,
        r#"element.removeAttribute("data-ゆ");"#,
        r#"element.removeAttribute("data-foo2");"#,
        r#"element.removeAttribute("data-foo:bar");"#,
        r#"element.removeAttribute("data-foo:bar");"#,
        r#"element.removeAttribute("data-foo.bar");"#,
        r#"element.removeAttribute("data-foo-bar");"#,
        r#"element.removeAttribute("data-foo");"#,
        r##"element.querySelector("#selector").removeAttribute("data-AllowAccess");"##,
        r#"element.removeAttribute("data-");"#,
        r#"optional?.element.removeAttribute("data-unicorn");"#,
        r#"element.removeAttribute("data-unicorn")?.property"#,
        r#"element.hasAttribute('data-unicorn');"#,
        r#"element.hasAttribute("data-unicorn");"#,
        r#"element.hasAttribute("data-unicorn",);"#,
        r#"element.hasAttribute("data-🦄");"#,
        r#"element.hasAttribute("data-ゆ");"#,
        r#"element.hasAttribute("data-foo2");"#,
        r#"element.hasAttribute("data-foo:bar");"#,
        r#"element.hasAttribute("data-foo:bar");"#,
        r#"element.hasAttribute("data-foo.bar");"#,
        r#"element.hasAttribute("data-foo-bar");"#,
        r#"element.hasAttribute("data-foo");"#,
        r##"element.querySelector("#selector").hasAttribute("data-AllowAccess");"##,
        r#"optional?.element.hasAttribute("data-unicorn");"#,
        r#"element.hasAttribute("data-unicorn").toString()"#,
        r#"element.getAttribute('data-unicorn');"#,
        r#"element.getAttribute("data-unicorn");"#,
        r#"element.getAttribute("data-unicorn",);"#,
        r#"element.getAttribute("data-🦄");"#,
        r#"element.getAttribute("data-ゆ");"#,
        r#"element.getAttribute("data-foo2");"#,
        r#"element.getAttribute("data-foo:bar");"#,
        r#"element.getAttribute("data-foo:bar");"#,
        r#"element.getAttribute("data-foo.bar");"#,
        r#"element.getAttribute("data-foo-bar");"#,
        r#"element.getAttribute("data-foo");"#,
        r##"element.querySelector("#selector").getAttribute("data-AllowAccess");"##,
        r#"optional?.element.getAttribute("data-unicorn");"#,
        r#"element.getAttribute("data-unicorn").toString()"#,
    ];

    Tester::new_without_config(PreferDomNodeDataset::NAME, pass, fail).test_and_snapshot();
}
