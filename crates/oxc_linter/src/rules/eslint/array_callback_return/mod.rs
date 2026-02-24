pub mod return_checker;

use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{Expression, FunctionBody, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use self::return_checker::{StatementReturnStatus, check_function_body, is_void_arrow_return};
use crate::{
    AstNode,
    ast_util::{get_enclosing_function, outermost_paren},
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

#[derive(Debug, Clone, Copy)]
enum MissingReturnHint {
    SwitchWithoutDefault,
    IfWithoutElse,
}

fn guess_missing_return_hint(
    function_body: &FunctionBody<'_>,
) -> Option<(Span, MissingReturnHint)> {
    let last_statement = function_body.statements.last()?;
    match last_statement {
        Statement::SwitchStatement(stmt) => {
            let has_default = stmt.cases.iter().any(oxc_ast::ast::SwitchCase::is_default_case);
            (!has_default).then_some((stmt.span, MissingReturnHint::SwitchWithoutDefault))
        }
        Statement::IfStatement(stmt) => {
            (stmt.alternate.is_none()).then_some((stmt.span, MissingReturnHint::IfWithoutElse))
        }
        _ => None,
    }
}

fn expect_return(
    method_name: &str,
    array_method_span: Span,
    function_body: &FunctionBody<'_>,
    allow_implicit: bool,
) -> OxcDiagnostic {
    let (span, hint) = guess_missing_return_hint(function_body)
        .map_or((function_body.span, None), |(span, hint)| (span, Some(hint)));

    let value_requirement = if allow_implicit {
        ""
    } else {
        "\nReturn a value on each path (or enable `allowImplicit` to allow `return;`)."
    };

    let (message, help) = match hint {
        Some(MissingReturnHint::SwitchWithoutDefault) => (
            format!(
                "Callback for array method {method_name:?} may fall through a `switch` without returning"
            ),
            format!(
                "This `switch` has no `default` case, so the callback may reach the end without a `return`. Add a `default` that returns/throws, or add a final `return` after the `switch`.{value_requirement}"
            ),
        ),
        Some(MissingReturnHint::IfWithoutElse) => (
            format!(
                "Callback for array method {method_name:?} may reach the end of an `if` without returning"
            ),
            format!(
                "This `if` has no `else` branch, so the callback may reach the end without a `return`. Add an `else`, or add a final `return`/`throw` after the `if`.{value_requirement}"
            ),
        ),
        None => (
            format!("Callback for array method {method_name:?} does not return on all code paths"),
            format!(
                "{method_name:?} uses the callback's return value. Add a `return` on every possible code path.{value_requirement}"
            ),
        ),
    };

    let mut diagnostic = OxcDiagnostic::warn(message).with_help(help).with_label(span);
    if allow_implicit {
        diagnostic = diagnostic.with_note("With `allowImplicit`, callbacks that don't explicitly return a value are considered to return `undefined`.");
    }
    if hint.is_some() {
        diagnostic = diagnostic
            .and_label(array_method_span.label(format!("{method_name:?} is called here.")));
    }

    diagnostic
}

fn expect_no_return(method_name: &str, call_span: Span, return_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected return value in callback for {method_name:?}"))
        .with_help(format!(
            "{method_name:?} ignores the callback's return value. Remove the returned value (use `return;` or no `return`), or use `map`/`flatMap` if you meant to produce a new array."
        ))
        .with_labels([
            call_span.label(format!("{method_name:?} is called here.")),
            return_span.label("This returned value is ignored."),
        ])
}

fn expect_void_return(method_name: &str, call_span: Span, return_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected return value in callback for {method_name:?}"))
        .with_help("Expected the return expression to be started with `void`")
        .with_labels([
            call_span.label(format!("{method_name:?} is called here.")),
            return_span.label("Prepend `void` to the expression."),
        ])
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ArrayCallbackReturn {
    /// When set to true, rule will also report forEach callbacks that return a value.
    check_for_each: bool,
    /// When set to true, allows callbacks of methods that require a return value to
    /// implicitly return undefined with a return statement containing no expression.
    allow_implicit: bool,
    /// When set to true, rule will not report the return value with a void operator.
    /// Works only if `checkForEach` option is set to true.
    allow_void: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce return statements in callbacks of array methods
    ///
    /// ### Why is this bad?
    ///
    /// Array has several methods for filtering, mapping, and folding.
    /// If we forget to write return statement in a callback of those, it’s probably a mistake.
    /// If you don’t want to use a return or don’t need the returned results,
    /// consider using .forEach instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let foo = [1, 2, 3, 4];
    /// foo.map((a) => {
    ///   console.log(a)
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let foo = [1, 2, 3, 4];
    /// foo.map((a) => {
    ///   console.log(a)
    ///   return a
    /// });
    /// ```
    ArrayCallbackReturn,
    eslint,
    pedantic,
    pending,
    config = ArrayCallbackReturn
);

impl Rule for ArrayCallbackReturn {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (function_body, always_explicit_return, is_async_empty) = match node.kind() {
            // Async, generator, and single expression arrow functions
            // always have explicit return value
            AstKind::ArrowFunctionExpression(arrow) => (
                &arrow.body,
                arrow.r#async || arrow.expression,
                arrow.r#async && arrow.body.statements.is_empty(),
            ),
            AstKind::Function(function) => {
                if let Some(body) = &function.body {
                    (
                        body,
                        function.r#async || function.generator,
                        function.r#async && body.statements.is_empty(),
                    )
                } else {
                    return;
                }
            }
            _ => return,
        };

        // Filter on target methods on Arrays
        if let Some((array_method_span, array_method)) = get_array_method_info(node, ctx) {
            let return_status = if always_explicit_return {
                StatementReturnStatus::AlwaysExplicit
            } else {
                check_function_body(function_body)
            };

            match (array_method, self.check_for_each, self.allow_void, self.allow_implicit) {
                ("forEach", false, _, _) => (),
                ("forEach", true, false, _) => {
                    if return_status.may_return_explicit() {
                        let return_spans = return_checker::get_explicit_return_spans(function_body)
                            .into_iter()
                            .next()
                            .unwrap_or_else(|| function_body.span());

                        ctx.diagnostic(expect_no_return(
                            &full_array_method_name(array_method),
                            array_method_span,
                            return_spans,
                        ));
                    }
                }
                ("forEach", true, true, _) => {
                    if !return_status.may_return_explicit() {
                        return;
                    }

                    if is_void_arrow_return(&function_body.statements) {
                        return;
                    }

                    let (return_spans, has_void) =
                        return_checker::get_no_voided_return_spans(function_body, self.allow_void);

                    if has_void && return_spans.is_empty() {
                        return;
                    }

                    let diagnostic_span =
                        return_spans.into_iter().next().unwrap_or_else(|| function_body.span());
                    ctx.diagnostic(expect_void_return(
                        &full_array_method_name(array_method),
                        array_method_span,
                        diagnostic_span,
                    ));
                }
                ("fromAsync", _, _, false) => {
                    if !return_status.must_return()
                        || return_status.may_return_implicit()
                        || is_async_empty
                    {
                        ctx.diagnostic(expect_return(
                            &full_array_method_name(array_method),
                            array_method_span,
                            function_body,
                            self.allow_implicit,
                        ));
                    }
                }
                (_, _, _, true) => {
                    if !return_status.must_return() {
                        ctx.diagnostic(expect_return(
                            &full_array_method_name(array_method),
                            array_method_span,
                            function_body,
                            self.allow_implicit,
                        ));
                    }
                }
                (_, _, _, false) => {
                    if !return_status.must_return() || return_status.may_return_implicit() {
                        ctx.diagnostic(expect_return(
                            &full_array_method_name(array_method),
                            array_method_span,
                            function_body,
                            self.allow_implicit,
                        ));
                    }
                }
            }
        }
    }
}

/// Code ported from [eslint](https://github.com/eslint/eslint/blob/v9.9.1/lib/rules/array-callback-return.js)
/// We're currently on a `Function` or `ArrowFunctionExpression`, findout if it is an argument
/// to the target array methods we're interested in.
pub fn get_array_method_info<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<(Span, &'a str)> {
    let mut current_node = node;
    loop {
        let parent = ctx.nodes().parent_node(current_node.id());
        match parent.kind() {
            // foo.every(nativeFoo || function foo() { ... })
            AstKind::LogicalExpression(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::ParenthesizedExpression(_)
            | AstKind::ChainExpression(_) => {
                current_node = parent;
            }

            // If the function is instantly invoked function expression (IIFE)
            // E.g.
            // foo.every(function() {
            //  return function() {}
            // }())
            AstKind::ReturnStatement(_) => {
                let Some(func_node) = get_enclosing_function(parent, ctx) else { break };
                let func_node = outermost_paren(func_node, ctx);

                // the node that calls func_node
                let func_parent = ctx.nodes().parent_node(func_node.id());

                if let AstKind::CallExpression(call) = func_parent.kind() {
                    let expected_callee = &call.callee;
                    if expected_callee.span() == func_node.kind().span() {
                        current_node = func_parent;
                        continue;
                    }
                }

                return None;
            }

            AstKind::CallExpression(call) => {
                let callee = call.callee.get_inner_expression();
                let callee = if let Some(member) = callee.as_member_expression() {
                    member
                } else if let Expression::ChainExpression(chain) = callee {
                    chain.expression.as_member_expression()?
                } else {
                    return None;
                };

                // Array.from
                if callee.is_specific_member_access("Array", "from") {
                    // Check that current node is parent's second argument
                    if call.arguments.len() == 2
                        && let Some(call_arg) = call.arguments[1].as_expression()
                        && call_arg.span() == current_node.kind().span()
                    {
                        return Some((callee.span(), "from"));
                    }
                }

                // Array.fromAsync
                if callee.is_specific_member_access("Array", "fromAsync") {
                    // Check that current node is parent's second argument
                    if call.arguments.len() == 2
                        && let Some(call_arg) = call.arguments[1].as_expression()
                        && call_arg.span() == current_node.kind().span()
                    {
                        return Some((callee.span(), "fromAsync"));
                    }
                }

                // "methods",
                let (array_method_span, array_method) = callee.static_property_info()?;

                if TARGET_METHODS.contains(&array_method)
                    // Check that current node is parent's first argument
                    && call.arguments.len() == 1
                    && let Some(call_arg) = call.arguments.first()
                        && call_arg
                            .as_expression()
                            .is_some_and(|arg| arg.span() == current_node.kind().span())
                {
                    return Some((array_method_span, array_method));
                }

                return None;
            }

            _ => return None,
        }
    }

    None
}

const TARGET_METHODS: [&str; 14] = [
    "every",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "flatMap",
    "forEach",
    "map",
    "reduce",
    "reduceRight",
    "some",
    "sort",
    "toSorted",
];

fn full_array_method_name(array_method: &str) -> Cow<'static, str> {
    match array_method {
        "from" => Cow::Borrowed("Array.from"),
        "fromAsync" => Cow::Borrowed("Array.fromAsync"),
        s => Cow::Owned(format!("Array.prototype.{s}")),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo.every(function(){ return 1; })", None),
        ("foo.every(function(){}())", None),
        ("foo.every(function(){ return function() { return true; }; }())", None),
        ("foo.every(function(){ return function() { return; }; })", None),
        ("foo.forEach(bar || function(x) { var a=0; })", None),
        ("foo.forEach(bar || function(x) { return a; })", None),
        ("foo.forEach(function() {return function() { var a = 0;}}())", None),
        ("foo.forEach(function(x) { var a=0; })", None),
        ("foo.forEach(function(x) { return a;})", None),
        ("foo.forEach(function(x) { return; })", None),
        ("foo.forEach(function(x) { if (a === b) { return;} var a=0; })", None),
        ("foo.forEach(function(x) { if (a === b) { return x;} var a=0; })", None),
        ("foo.bar().forEach(function(x) { return; })", None),
        ("[\"foo\",\"bar\",\"baz\"].forEach(function(x) { return x; })", None),
        ("foo.forEach(x => { var a=0; })", None),
        ("foo.forEach(x => { if (a === b) { return;} var a=0; })", None),
        ("foo.forEach(x => x)", None),
        ("foo.forEach(val => y += val)", None),
        ("foo.map(async function(){})", None),
        ("foo.map(async () => {})", None),
        ("foo.map(function* () {})", None),
        (
            "Array.from(x, function() { return true; })",
            Some(serde_json::json!([{ "allowImplicit": false }])),
        ),
        // (
        //     "Int32Array.from(x, function() { return true; })",
        //     Some(serde_json::json!([{ "allowImplicit": false }])),
        // ),
        ("foo.every(function() { return true; })", None),
        ("foo.filter(function() { return true; })", None),
        ("foo.find(function() { return true; })", None),
        ("foo.findIndex(function() { return true; })", None),
        ("foo.findLast(function() { return true; })", None),
        ("foo.findLastIndex(function() { return true; })", None),
        ("foo.flatMap(function() { return true; })", None),
        ("foo.forEach(function() { return; })", None),
        ("foo.map(function() { return true; })", None),
        ("foo.reduce(function() { return true; })", None),
        ("foo.reduceRight(function() { return true; })", None),
        ("foo.some(function() { return true; })", None),
        ("foo.sort(function() { return 0; })", None),
        ("foo.toSorted(function() { return 0; })", None),
        ("foo.every(() => { return true; })", None),
        ("foo.every(function() { if (a) return true; else return false; })", None),
        ("foo.every(function() { switch (a) { case 0: bar(); default: return true; } })", None),
        (
            "foo.every(function() { try { bar(); return true; } catch (err) { return false; } })",
            None,
        ),
        ("foo.every(function() { try { bar(); } finally { return true; } })", None),
        ("foo.every(function() { switch (a) { default: case0: return true; } })", None),
        (
            "Array.from(x, function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        // (
        //     "Int32Array.from(x, function() { return; })",
        //     Some(serde_json::json!([{"allowImplicit": true}])),
        // ),
        ("foo.every(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.filter(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.find(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.findIndex(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.findLast(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.findLastIndex(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.flatMap(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.forEach(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.map(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.reduce(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.reduceRight(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.some(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.sort(function() { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.toSorted(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.every(() => { return; })", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.every(function() { if (a) return; else return a; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.every(function() { switch (a) { case 0: bar(); default: return; } })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.every(function() { try { bar(); return; } catch (err) { return; } })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.every(function() { try { bar(); } finally { return; } })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.forEach(function(x) { return; })", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach(function(x) { var a=0; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function(x) { if (a === b) { return;} var a=0; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function() {return function() { if (a == b) { return; }}}())",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.forEach(x => { var a=0; })", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach(x => { if (a === b) { return;} var a=0; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.forEach(x => { x })", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach(bar || function(x) { return; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "Array.from(x, function() { return true; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        // (
        //     "Int32Array.from(x, function() { return true; })",
        //     Some(serde_json::json!([{"checkForEach": true}])),
        // ),
        ("foo.every(() => { return true; })", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.every(function() { if (a) return 1; else return a; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.every(function() { switch (a) { case 0: return bar(); default: return a; } })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.every(function() { try { bar(); return 1; } catch (err) { return err; } })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.every(function() { try { bar(); } finally { return 1; } })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.every(function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => void x)",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => void bar(x))",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(function (x) { return void bar(x); })",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { return void bar(x); })",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { if (a === b) { return void a; } bar(x) })",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        ("Arrow.from(x, function() {})", None),
        ("foo.abc(function() {})", None),
        ("every(function() {})", None),
        ("foo[every](function() {})", None),
        ("var every = function() {}", None),
        ("foo[`${every}`](function() {})", None),
        ("foo.every(() => true)", None),
        ("return function() {}", None),
        (
            "array.map((node) => { if (isTaskNode(node)) { return someObj; } else if (isOtherNode(node)) { return otherObj; } else { throw new Error('Unsupported'); } })",
            None,
        ),
        ("Array.fromAsync(x, function() { return true; })", None),
        ("Array.fromAsync(x, async function() { return true; })", None),
        (
            "Array.fromAsync(x, function() { return; })",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("Array.fromAsync(x, async () => true)", None),
        ("Array.fromAsync(x, function * () {})", None),
        ("Float64Array.fromAsync(x, function() {})", None),
        ("Array.fromAsync(function() {})", None),
    ];

    let fail = vec![
        ("Array.from(x, function() {})", None),
        ("Array.from(x, function foo() {})", None),
        // ("Int32Array.from(x, function() {})", None),
        // ("Int32Array.from(x, function foo() {})", None),
        ("foo.every(function() {})", None),
        ("foo.every(function foo() {})", None),
        ("foo.filter(function() {})", None),
        ("foo.filter(function foo() {})", None),
        ("foo.find(function() {})", None),
        ("foo.find(function foo() {})", None),
        ("foo.findLast(function() {})", None),
        ("foo.findLast(function foo() {})", None),
        ("foo.findIndex(function() {})", None),
        ("foo.findIndex(function foo() {})", None),
        ("foo.findLastIndex(function() {})", None),
        ("foo.findLastIndex(function foo() {})", None),
        ("foo.flatMap(function() {})", None),
        ("foo.flatMap(function foo() {})", None),
        ("foo.map(function() {})", None),
        ("foo.map(function foo() {})", None),
        (
            r#"const fruits = [{ name: "apple" }, { name: "banana" }] as const;

const _test = fruits.map((fruit) => {
  switch (fruit.name) {
    case "apple": {
      return "a"
    }

    case "banana": {
      return "b"
    }
  }
});"#,
            None,
        ),
        ("foo.reduce(function() {})", None),
        ("foo.reduce(function foo() {})", None),
        ("foo.reduceRight(function() {})", None),
        ("foo.reduceRight(function foo() {})", None),
        ("foo.some(function() {})", None),
        ("foo.some(function foo() {})", None),
        ("foo.sort(function() {})", None),
        ("foo.sort(function foo() {})", None),
        ("foo.toSorted(function() {})", None),
        ("foo.toSorted(function foo() {})", None),
        ("foo.bar.baz.every(function() {})", None),
        ("foo.bar.baz.every(function foo() {})", None),
        ("foo[\"every\"](function() {})", None),
        ("foo[\"every\"](function foo() {})", None),
        ("foo[`every`](function() {})", None),
        ("foo[`every`](function foo() {})", None),
        ("foo.every(() => {})", None),
        ("foo.every(function() { if (a) return true; })", None),
        ("foo.every(function cb() { if (a) return true; })", None),
        ("foo.every(function() { switch (a) { case 0: break; default: return true; } })", None),
        ("foo.every(function foo() { switch (a) { case 0: break; default: return true; } })", None),
        ("foo.every(function() { try { bar(); } catch (err) { return true; } })", None),
        ("foo.every(function foo() { try { bar(); } catch (err) { return true; } })", None),
        ("foo.every(function() { return; })", None),
        ("foo.every(function foo() { return; })", None),
        ("foo.every(function() { if (a) return; })", None),
        ("foo.every(function foo() { if (a) return; })", None),
        ("foo.every(function() { if (a) return; else return; })", None),
        ("foo.every(function foo() { if (a) return; else return; })", None),
        ("foo.every(cb || function() {})", None),
        ("foo.every(cb || function foo() {})", None),
        ("foo.every(a ? function() {} : function() {})", None),
        ("foo.every(a ? function foo() {} : function bar() {})", None),
        ("foo.every(function(){ return function() {}; }())", None),
        ("foo.every(function(){ return function foo() {}; }())", None),
        ("foo.every(() => {})", Some(serde_json::json!([{ "allowImplicit": false }]))),
        ("foo.every(() => {})", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Array.from(x, function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.every(function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.filter(function foo() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.find(function foo() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.map(function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.reduce(function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("foo.reduceRight(function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "foo.bar.baz.every(function foo() {})",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        ("foo.every(cb || function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        (
            "[\"foo\",\"bar\"].sort(function foo() {})",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "[\"foo\",\"bar\"].toSorted(function foo() {})",
            Some(serde_json::json!([{"allowImplicit": true}])),
        ),
        (
            "foo.forEach(x => x)",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => !x)",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => (x))",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { return x; })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { return !x; })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { return(x); })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { return (x + 1); })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { if (a === b) { return x; } })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { if (a === b) { return !x; } })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { if (a === b) { return (x + a); } })",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(function(x) { if (a == b) {return x;}})",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"allowImplicit": true, "checkForEach": true}])),
        ),
        ("foo.forEach(x => x)", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.forEach(val => y += val)", Some(serde_json::json!([{"checkForEach": true}]))),
        ("[\"foo\",\"bar\"].forEach(x => ++x)", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.bar().forEach(x => x === y)", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach(function() {return function() { if (a == b) { return a; }}}())",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function(x) { if (a == b) {return x;}})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function(x) { if (a == b) {return undefined;}})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.bar().forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "[\"foo\",\"bar\"].forEach(function bar(x) { return x;})",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.forEach((x) => { return x;})", Some(serde_json::json!([{"checkForEach": true}]))),
        ("Array.from(x, function() {})", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.every(function() {})", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.filter(function foo() {})", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.filter(function foo() { return; })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.every(cb || function() {})", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.filter(bar => { baz(); } )", None),
        ("foo.filter(\n() => {} )", None),
        ("foo.filter(bar || ((baz) => {}) )", None),
        ("foo.filter(bar => { return; })", None),
        ("Array.from(foo, bar => { bar })", None),
        ("foo.forEach(bar => bar)", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach((function () { return (bar) => bar; })())",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach((() => {\n return bar => bar; })())",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach((bar) => { if (bar) { return; } else { return bar ; } })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.forEach(x => (x))", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.forEach((x) => void x)", Some(serde_json::json!([{"checkForEach": true}]))),
        ("foo.forEach((x) => void bar(x))", Some(serde_json::json!([{"checkForEach": true}]))),
        (
            "foo.forEach((x) => { return void bar(x); })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            "foo.forEach((x) => { if (a === b) { return void a; } bar(x) })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        ("foo.filter(function(){})", None),
        ("foo.filter(function (){})", None),
        ("foo.filter(function\n(){})", None),
        ("foo.filter(function bar(){})", None),
        ("foo.filter(function bar  (){})", None),
        ("foo.filter(function\n bar() {})", None),
        ("Array.from(foo, function bar(){})", None),
        ("Array.from(foo, bar ? function (){} : baz)", None),
        ("foo.filter(function bar() { return \n })", None),
        (
            "foo.forEach(function () { \nif (baz) return bar\nelse return\n })",
            Some(serde_json::json!([{"checkForEach": true}])),
        ),
        (
            r"Array.fromAsync(x,
            async	function \\u0066oo // bar
               () {})",
            None,
        ),
        ("foo?.filter(() => { console.log('hello') })", None),
        ("(foo?.filter)(() => { console.log('hello') })", None),
        ("Array?.from([], () => { console.log('hello') })", None),
        ("(Array?.from)([], () => { console.log('hello') })", None),
        ("foo?.filter((function() { return () => { console.log('hello') } })?.())", None),
        ("Array.fromAsync(x, function() {})", None),
        ("Array.fromAsync(x, function() {})", Some(serde_json::json!([{"allowImplicit": true}]))),
        ("Array.fromAsync(x, () => {})", None),
        ("Array.fromAsync(x, function foo() {})", None),
        ("Array.fromAsync(x, async function() {})", None),
        ("Array.fromAsync(x, async () => {})", None),
        (
            "foo.forEach(x => x);",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => !x);",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => (x));",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => { return x; });",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => { return !x; });",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => { return (x); });",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => { return x + 1; });",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => { if (a === b) { return x; } });",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => { if (a === b) { return !x; } });",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
        (
            "foo.forEach(x => { if (a === b) { return (x + a); } });",
            Some(serde_json::json!([{"allowVoid": true, "checkForEach": true}])),
        ),
    ];

    Tester::new(ArrayCallbackReturn::NAME, ArrayCallbackReturn::PLUGIN, pass, fail)
        .test_and_snapshot();
}
