use cow_utils::CowUtils;
use oxc_allocator::Allocator;
use oxc_ast::{
    AstBuilder, AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_codegen::CodegenOptions;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, SPAN, Span};
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn set(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using `dataset` over `setAttribute`.").with_label(span)
}

fn get(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using `dataset` over `getAttribute`.").with_label(span)
}

fn has(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using `dataset` over `hasAttribute`.").with_label(span)
}

fn remove(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using `dataset` over `removeAttribute`.").with_label(span)
}

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// element.setAttribute('data-unicorn', '🦄');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// element.dataset.unicorn = '🦄';
    /// ```
    PreferDomNodeDataset,
    unicorn,
    pedantic,
    conditional_fix
);

impl Rule for PreferDomNodeDataset {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        if member_expr.is_computed() {
            return;
        }

        let Some((span, method_name)) = member_expr.static_property_info() else {
            return;
        };

        match method_name {
            "setAttribute" => {
                if call_expr.arguments.len() != 2 {
                    return;
                }
            }
            "removeAttribute" | "hasAttribute" => {
                if call_expr.arguments.len() != 1 {
                    return;
                }
            }
            "getAttribute" => {
                if call_expr.arguments.len() != 1 {
                    return;
                }

                // Playwright's `Locator#getAttribute()` returns a promise.
                // https://playwright.dev/docs/api/class-locator#locator-get-attribute
                // https://github.com/sindresorhus/eslint-plugin-unicorn/pull/2334
                if matches!(ctx.nodes().parent_node(node.id()).kind(), AstKind::AwaitExpression(_))
                {
                    return;
                }
            }
            _ => return,
        }

        let Argument::StringLiteral(string_lit) = &call_expr.arguments[0] else {
            return;
        };

        let Some(dataset_property_name) = strip_data_prefix(&string_lit.value) else {
            return;
        };

        match method_name {
            "setAttribute" => {
                ctx.diagnostic_with_fix(set(span), |fixer| {
                    if call_uses_optional_chain(call_expr) {
                        return fixer.noop();
                    }
                    if !is_value_not_usable(node, ctx) {
                        return fixer.noop();
                    }

                    let dataset_property_name_camel = dash_to_camel_case(dataset_property_name);
                    let object_span = member_expr.object().span();

                    let value_text = ctx.source_range(call_expr.arguments[1].span());
                    fix_to_dataset_assignment(
                        fixer,
                        call_expr.span,
                        object_span,
                        &dataset_property_name_camel,
                        value_text,
                        ctx,
                    )
                });
            }
            "getAttribute" => {
                ctx.diagnostic_with_fix(get(span), |fixer| {
                    let dataset_property_name_camel = dash_to_camel_case(dataset_property_name);
                    let object_span = member_expr.object().span();

                    fix_to_dataset_access(
                        fixer,
                        call_expr.span,
                        object_span,
                        &dataset_property_name_camel,
                        ctx,
                    )
                });
            }
            "removeAttribute" => {
                ctx.diagnostic_with_fix(remove(string_lit.span), |fixer| {
                    if !is_value_not_usable(node, ctx) {
                        return fixer.noop();
                    }
                    let dataset_property_name_camel = dash_to_camel_case(dataset_property_name);
                    let object_span = member_expr.object().span();

                    fix_to_dataset_delete(
                        fixer,
                        call_expr.span,
                        object_span,
                        &dataset_property_name_camel,
                        ctx,
                    )
                });
            }
            "hasAttribute" => {
                ctx.diagnostic_with_fix(has(span), |fixer| {
                    if call_uses_optional_chain(call_expr) {
                        return fixer.noop();
                    }

                    let dataset_property_name_camel = dash_to_camel_case(dataset_property_name);
                    let object_span = member_expr.object().span();

                    fix_to_has_own(
                        fixer,
                        call_expr.span,
                        object_span,
                        &dataset_property_name_camel,
                        ctx,
                    )
                });
            }
            _ => unreachable!(),
        }
    }
}

fn strip_data_prefix(s: &str) -> Option<&str> {
    s.strip_prefix("data-").or_else(|| s.strip_prefix("DATA-"))
}

/// converts a kebab-case string (after data- prefix is removed) to camelCase
fn dash_to_camel_case(s: &str) -> String {
    let s_lower = s.cow_to_lowercase();
    let mut result = String::with_capacity(s_lower.len());
    let mut capitalize_next = false;

    for c in s_lower.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn is_value_not_usable(node: &AstNode, ctx: &LintContext) -> bool {
    let parent_node = ctx.nodes().parent_node(node.id());
    let parent_kind = parent_node.kind();

    if matches!(parent_kind, AstKind::ExpressionStatement(_)) {
        return true;
    }

    if matches!(parent_kind, AstKind::ChainExpression(_)) {
        let grandparent = ctx.nodes().parent_node(parent_node.id());
        return matches!(grandparent.kind(), AstKind::ExpressionStatement(_));
    }

    false
}

fn call_uses_optional_chain(call_expr: &CallExpression) -> bool {
    call_expr.optional || expression_uses_optional_chain(&call_expr.callee)
}

fn expression_uses_optional_chain(expr: &Expression) -> bool {
    let expr = expr.get_inner_expression();

    if matches!(expr, Expression::ChainExpression(_)) {
        return true;
    }

    if let Some(member_expr) = expr.as_member_expression() {
        return member_expr.optional() || expression_uses_optional_chain(member_expr.object());
    }

    if let Expression::CallExpression(call_expr) = expr {
        return call_expr.optional || expression_uses_optional_chain(&call_expr.callee);
    }

    false
}

fn to_string_literal_text(fixer: RuleFixer, text: &str) -> String {
    let mut codegen = fixer.codegen().with_options(CodegenOptions::default());
    let alloc = Allocator::default();
    let ast = AstBuilder::new(&alloc);
    codegen.print_expression(&ast.expression_string_literal(SPAN, ast.atom(text), None));
    codegen.into_source_text()
}

fn dataset_property_text(fixer: RuleFixer, name: &str) -> String {
    if is_identifier_name(name) {
        format!(".{name}")
    } else {
        format!("[{}]", to_string_literal_text(fixer, name))
    }
}

fn fix_to_dataset_assignment(
    fixer: RuleFixer,
    call_span: Span,
    object_span: Span,
    property_name: &str,
    value: &str,
    ctx: &LintContext,
) -> RuleFix {
    let object_text = ctx.source_range(object_span);
    let property_access = dataset_property_text(fixer, property_name);
    let fixed = format!("{object_text}.dataset{property_access} = {value}");
    fixer.replace(call_span, fixed)
}

fn fix_to_dataset_access(
    fixer: RuleFixer,
    call_span: Span,
    object_span: Span,
    property_name: &str,
    ctx: &LintContext,
) -> RuleFix {
    let object_text = ctx.source_range(object_span);
    let property_access = dataset_property_text(fixer, property_name);
    let fixed = format!("{object_text}.dataset{property_access}");
    fixer.replace(call_span, fixed)
}

fn fix_to_dataset_delete(
    fixer: RuleFixer,
    call_span: Span,
    object_span: Span,
    property_name: &str,
    ctx: &LintContext,
) -> RuleFix {
    let object_text = ctx.source_range(object_span);
    let property_access = dataset_property_text(fixer, property_name);
    let fixed = format!("delete {object_text}.dataset{property_access}");
    fixer.replace(call_span, fixed)
}

fn fix_to_has_own(
    fixer: RuleFixer,
    call_span: Span,
    object_span: Span,
    property_name: &str,
    ctx: &LintContext,
) -> RuleFix {
    let object_text = ctx.source_range(object_span);
    let property_name_text = to_string_literal_text(fixer, property_name);
    let fixed = format!("Object.hasOwn({object_text}.dataset, {property_name_text})");
    fixer.replace(call_span, fixed)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"element.dataset.unicorn = '🦄';",
        r"element.dataset['unicorn'] = '🦄';",
        r"new element.setAttribute('data-unicorn', '🦄');",
        r"setAttribute('data-unicorn', '🦄');",
        r"element['setAttribute']('data-unicorn', '🦄');",
        r"element[setAttribute]('data-unicorn', '🦄');",
        r"element.foo('data-unicorn', '🦄');",
        r"element.setAttribute('data-unicorn', '🦄', 'extra');",
        r"element.setAttribute('data-unicorn');",
        r"element.setAttribute(...argumentsArray, ...argumentsArray2)",
        r"element.setAttribute(`data-unicorn`, '🦄');",
        r"element.setAttribute(0, '🦄');",
        r"element.setAttribute('foo-unicorn', '🦄');",
        r"element.setAttribute('data', '🦄');",
        r"delete element.dataset.unicorn;",
        r#"delete element.dataset["unicorn"];"#,
        r#"new element.removeAttribute("data-unicorn");"#,
        r#"removeAttribute("data-unicorn");"#,
        r#"element["removeAttribute"]("data-unicorn");"#,
        r#"element[removeAttribute]("data-unicorn");"#,
        r#"element.foo("data-unicorn");"#,
        r#"element.removeAttribute("data-unicorn", "extra");"#,
        r"element.removeAttribute();",
        r"element.removeAttribute(...argumentsArray, ...argumentsArray2)",
        r"element.removeAttribute(`data-unicorn`);",
        r"element.removeAttribute(0);",
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
        r"element.hasAttribute();",
        r"element.hasAttribute(...argumentsArray, ...argumentsArray2)",
        r"element.hasAttribute(`data-unicorn`);",
        r"element.hasAttribute(0);",
        r#"element.hasAttribute("foo-unicorn");"#,
        r#"element.hasAttribute("data");"#,
        r"element.dataset.unicorn",
        r#"new element.getAttribute("data-unicorn");"#,
        r#"getAttribute("data-unicorn");"#,
        r#"element["getAttribute"]("data-unicorn");"#,
        r#"element[getAttribute]("data-unicorn");"#,
        r#"element.foo("data-unicorn");"#,
        r#"element.getAttribute("data-unicorn", "extra");"#,
        r"element.getAttribute();",
        r"element.getAttribute(...argumentsArray, ...argumentsArray2)",
        r"element.getAttribute(`data-unicorn`);",
        r"element.getAttribute(0);",
        r#"element.getAttribute("foo-unicorn");"#,
        r#"element.getAttribute("data");"#,
        r#"element.getAttribute("stylý");"#,
        r#"await page.locator("text=Hello").getAttribute("data-foo")"#,
    ];

    let fail = vec![
        r"element.setAttribute('data-unicorn', '🦄');",
        r"element.setAttribute('data-🦄', '🦄');",
        r"element.setAttribute('data-ゆ', 'ゆ');",
        r"element.setAttribute('data-foo2', '🦄');",
        r"element.setAttribute('data-foo:bar', 'zaz');",
        r#"element.setAttribute("data-foo:bar", "zaz");"#,
        r"element.setAttribute('data-foo.bar', 'zaz');",
        r"element.setAttribute('data-foo-bar', 'zaz');",
        r"element.setAttribute('data-foo', /* comment */ 'bar');",
        r"element.querySelector('#selector').setAttribute('data-AllowAccess', true);",
        r#"element.setAttribute("data-", "🦄");"#,
        r#"element.setAttribute("data--foo", "🦄");"#,
        r#"element.setAttribute("DATA--FOO", "🦄");"#,
        r#"element.setAttribute("DATA- ", "🦄");"#,
        r#"element.setAttribute("DATA-Foo-bar", "🦄");"#,
        r#"element.setAttribute('data-a"b', "zaz");"#,
        r#"optional?.element.setAttribute("data-unicorn", "🦄");"#,
        r#"console.log(element.setAttribute("data-unicorn", "🦄"))"#,
        r"element.removeAttribute('data-unicorn');",
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
        r#"element.removeAttribute('data-a"b');"#,
        r##"element.querySelector("#selector").removeAttribute("data-AllowAccess");"##,
        r#"element.removeAttribute("data-");"#,
        r#"optional?.element.removeAttribute("data-unicorn");"#,
        r#"element.removeAttribute("data-unicorn")?.property"#,
        r"element.hasAttribute('data-unicorn');",
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
        r#"element.hasAttribute('data-a"b');"#,
        r#"optional?.element.hasAttribute("data-unicorn");"#,
        r#"element.hasAttribute("data-unicorn").toString()"#,
        r"element.getAttribute('data-unicorn');",
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
        r#"element.getAttribute('data-a"b');"#,
        r#"optional?.element.getAttribute("data-unicorn");"#,
        r#"element.getAttribute("data-unicorn").toString()"#,
        r#"(await promise).getAttribute("data-foo")"#,
    ];

    let fix = vec![
        (r"element.setAttribute('data-unicorn', '🦄');", r"element.dataset.unicorn = '🦄';"),
        (r"element.setAttribute('data-🦄', '🦄');", r#"element.dataset["🦄"] = '🦄';"#),
        (r"element.setAttribute('data-ゆ', 'ゆ');", r"element.dataset.ゆ = 'ゆ';"),
        (r"element.setAttribute('data-foo2', '🦄');", r"element.dataset.foo2 = '🦄';"),
        (r"element.setAttribute('data-foo:bar', 'zaz');", r#"element.dataset["foo:bar"] = 'zaz';"#),
        (
            r#"element.setAttribute("data-foo:bar", "zaz");"#,
            r#"element.dataset["foo:bar"] = "zaz";"#,
        ),
        (r"element.setAttribute('data-foo.bar', 'zaz');", r#"element.dataset["foo.bar"] = 'zaz';"#),
        (r"element.setAttribute('data-foo-bar', 'zaz');", r"element.dataset.fooBar = 'zaz';"),
        (
            r"element.setAttribute('data-foo', /* comment */ 'bar');",
            r"element.dataset.foo = 'bar';",
        ),
        (
            r"element.querySelector('#selector').setAttribute('data-AllowAccess', true);",
            r"element.querySelector('#selector').dataset.allowaccess = true;",
        ),
        (r#"element.setAttribute("data-", "🦄");"#, r#"element.dataset[""] = "🦄";"#),
        (r#"element.setAttribute("data--foo", "🦄");"#, r#"element.dataset.Foo = "🦄";"#),
        (r#"element.setAttribute("DATA--FOO", "🦄");"#, r#"element.dataset.Foo = "🦄";"#),
        (r#"element.setAttribute("DATA- ", "🦄");"#, r#"element.dataset[" "] = "🦄";"#),
        (r#"element.setAttribute("DATA-Foo-bar", "🦄");"#, r#"element.dataset.fooBar = "🦄";"#),
        (r#"element.setAttribute('data-a"b', "zaz");"#, r#"element.dataset["a\"b"] = "zaz";"#),
        (
            r#"optional?.element.setAttribute("data-unicorn", "🦄");"#,
            r#"optional?.element.setAttribute("data-unicorn", "🦄");"#,
        ),
        (r"element.removeAttribute('data-unicorn');", r"delete element.dataset.unicorn;"),
        (r#"element.removeAttribute("data-unicorn");"#, r"delete element.dataset.unicorn;"),
        (r#"element.removeAttribute("data-unicorn",);"#, r"delete element.dataset.unicorn;"),
        (r#"element.removeAttribute("data-🦄");"#, r#"delete element.dataset["🦄"];"#),
        (r#"element.removeAttribute("data-ゆ");"#, r"delete element.dataset.ゆ;"),
        (r#"element.removeAttribute("data-foo2");"#, r"delete element.dataset.foo2;"),
        (r#"element.removeAttribute("data-foo:bar");"#, r#"delete element.dataset["foo:bar"];"#),
        (r#"element.removeAttribute("data-foo.bar");"#, r#"delete element.dataset["foo.bar"];"#),
        (r#"element.removeAttribute("data-foo-bar");"#, r"delete element.dataset.fooBar;"),
        (r#"element.removeAttribute("data-foo");"#, r"delete element.dataset.foo;"),
        (r#"element.removeAttribute('data-a"b');"#, r#"delete element.dataset["a\"b"];"#),
        (
            r##"element.querySelector("#selector").removeAttribute("data-AllowAccess");"##,
            r##"delete element.querySelector("#selector").dataset.allowaccess;"##,
        ),
        (r#"element.removeAttribute("data-");"#, r#"delete element.dataset[""];"#),
        (
            r#"optional?.element.removeAttribute("data-unicorn");"#,
            r"delete optional?.element.dataset.unicorn;",
        ),
        (r"element.hasAttribute('data-unicorn');", r#"Object.hasOwn(element.dataset, "unicorn");"#),
        (
            r#"element.hasAttribute("data-unicorn");"#,
            r#"Object.hasOwn(element.dataset, "unicorn");"#,
        ),
        (
            r#"element.hasAttribute("data-unicorn",);"#,
            r#"Object.hasOwn(element.dataset, "unicorn");"#,
        ),
        (r#"element.hasAttribute("data-🦄");"#, r#"Object.hasOwn(element.dataset, "🦄");"#),
        (r#"element.hasAttribute("data-ゆ");"#, r#"Object.hasOwn(element.dataset, "ゆ");"#),
        (r#"element.hasAttribute("data-foo2");"#, r#"Object.hasOwn(element.dataset, "foo2");"#),
        (
            r#"element.hasAttribute("data-foo:bar");"#,
            r#"Object.hasOwn(element.dataset, "foo:bar");"#,
        ),
        (
            r#"element.hasAttribute("data-foo.bar");"#,
            r#"Object.hasOwn(element.dataset, "foo.bar");"#,
        ),
        (
            r#"element.hasAttribute("data-foo-bar");"#,
            r#"Object.hasOwn(element.dataset, "fooBar");"#,
        ),
        (r#"element.hasAttribute("data-foo");"#, r#"Object.hasOwn(element.dataset, "foo");"#),
        (
            r##"element.querySelector("#selector").hasAttribute("data-AllowAccess");"##,
            r##"Object.hasOwn(element.querySelector("#selector").dataset, "allowaccess");"##,
        ),
        (r#"element.hasAttribute('data-a"b');"#, r#"Object.hasOwn(element.dataset, "a\"b");"#),
        (
            r#"optional?.element.hasAttribute("data-unicorn");"#,
            r#"optional?.element.hasAttribute("data-unicorn");"#,
        ),
        (
            r#"element.hasAttribute("data-unicorn").toString()"#,
            r#"Object.hasOwn(element.dataset, "unicorn").toString()"#,
        ),
        (r"element.getAttribute('data-unicorn');", r"element.dataset.unicorn;"),
        (r#"element.getAttribute("data-unicorn");"#, r"element.dataset.unicorn;"),
        (r#"element.getAttribute("data-unicorn",);"#, r"element.dataset.unicorn;"),
        (r#"element.getAttribute("data-🦄");"#, r#"element.dataset["🦄"];"#),
        (r#"element.getAttribute("data-ゆ");"#, r"element.dataset.ゆ;"),
        (r#"element.getAttribute("data-foo2");"#, r"element.dataset.foo2;"),
        (r#"element.getAttribute("data-foo:bar");"#, r#"element.dataset["foo:bar"];"#),
        (r#"element.getAttribute("data-foo.bar");"#, r#"element.dataset["foo.bar"];"#),
        (r#"element.getAttribute("data-foo-bar");"#, r"element.dataset.fooBar;"),
        (r#"element.getAttribute("data-foo");"#, r"element.dataset.foo;"),
        (r#"element.getAttribute('data-a"b');"#, r#"element.dataset["a\"b"];"#),
        (
            r##"element.querySelector("#selector").getAttribute("data-AllowAccess");"##,
            r##"element.querySelector("#selector").dataset.allowaccess;"##,
        ),
        (
            r#"optional?.element.getAttribute("data-unicorn");"#,
            r"optional?.element.dataset.unicorn;",
        ),
        (
            r#"element.getAttribute("data-unicorn").toString()"#,
            r"element.dataset.unicorn.toString()",
        ),
        (r#"(await promise).getAttribute("data-foo")"#, r"(await promise).dataset.foo"),
    ];

    Tester::new(PreferDomNodeDataset::NAME, PreferDomNodeDataset::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("mjs")
        .test_and_snapshot();
}
