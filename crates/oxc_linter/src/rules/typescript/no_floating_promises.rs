use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, ChainElement, Expression},
};
use oxc_checker::types::Ty;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    fixer::RuleFixer,
    native_type_aware::TypedApiContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{NameSpecifier, TypeOrValueSpecifier},
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoFloatingPromises(Box<NoFloatingPromisesConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoFloatingPromisesConfig {
    /// Allows specific calls to be ignored, specified as type or value specifiers.
    pub allow_for_known_safe_calls: Vec<TypeOrValueSpecifier>,
    /// Allows specific Promise types to be ignored, specified as type or value specifiers.
    pub allow_for_known_safe_promises: Vec<TypeOrValueSpecifier>,
    /// Check for thenable objects that are not necessarily Promises.
    pub check_thenables: bool,
    /// Ignore immediately invoked function expressions (IIFEs).
    #[serde(rename = "ignoreIIFE")]
    pub ignore_iife: bool,
    /// Ignore Promises that are void expressions.
    pub ignore_void: bool,
}

impl Default for NoFloatingPromisesConfig {
    fn default() -> Self {
        Self {
            allow_for_known_safe_calls: Vec::new(),
            allow_for_known_safe_promises: Vec::new(),
            check_thenables: false,
            ignore_iife: false,
            ignore_void: true,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows "floating" Promises in TypeScript code, which is a Promise that is created without any code to handle its resolution or rejection.
    ///
    /// This rule will report Promise-valued statements that are not treated in one of the following ways:
    ///
    /// - Calling its `.then()` with two arguments
    /// - Calling its `.catch()` with one argument
    /// - `await`ing it
    /// - `return`ing it
    /// - `void`ing it
    ///
    /// This rule also reports when an Array containing Promises is created and not properly handled. The main way to resolve this is by using one of the Promise concurrency methods to create a single Promise, then handling that according to the procedure above. These methods include:
    ///
    /// - `Promise.all()`
    /// - `Promise.allSettled()`
    /// - `Promise.any()`
    /// - `Promise.race()`
    ///
    /// ### Why is this bad?
    ///
    /// Floating Promises can cause several issues, such as improperly sequenced operations, ignored Promise rejections, and more.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const promise = new Promise((resolve, reject) => resolve('value'));
    /// promise;
    ///
    /// async function returnsPromise() {
    ///   return 'value';
    /// }
    /// returnsPromise().then(() => {});
    ///
    /// Promise.reject('value').catch();
    ///
    /// Promise.reject('value').finally();
    ///
    /// [1, 2, 3].map(async x => x + 1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const promise = new Promise((resolve, reject) => resolve('value'));
    /// await promise;
    ///
    /// async function returnsPromise() {
    ///   return 'value';
    /// }
    ///
    /// void returnsPromise();
    ///
    /// returnsPromise().then(
    ///   () => {},
    ///   () => {},
    /// );
    ///
    /// Promise.reject('value').catch(() => {});
    ///
    /// await Promise.reject('value').finally(() => {});
    ///
    /// await Promise.all([1, 2, 3].map(async x => x + 1));
    /// ```
    NoFloatingPromises,
    typescript,
    correctness,
    suggestion,
    config = NoFloatingPromisesConfig,
    version = "1.11.0",
    short_description = "Require Promise-like statements to be handled appropriately.",
);

impl Rule for NoFloatingPromises {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExpressionStatement(statement) = node.kind() else {
            return;
        };
        let Some(api) = ctx.type_aware() else { return };

        if self.0.ignore_iife && is_iife(&statement.expression) {
            return;
        }
        if is_known_safe_call(&statement.expression, api, &self.0.allow_for_known_safe_calls) {
            return;
        }

        let Some(unhandled) = find_unhandled(
            &statement.expression,
            api,
            self.0.check_thenables,
            self.0.ignore_void,
            &self.0.allow_for_known_safe_promises,
        ) else {
            return;
        };

        let type_name = api.type_name(unhandled.type_span, unhandled.ty).unwrap_or("unknown");
        let (message, help) = if unhandled.promise_array {
            (
                "An array of Promises may be unintentional.",
                if self.0.ignore_void {
                    "Consider handling the promises' fulfillment or rejection with Promise.all or similar, or explicitly marking the expression as ignored with the `void` operator."
                } else {
                    "Consider handling the promises' fulfillment or rejection with Promise.all or similar."
                },
            )
        } else if self.0.ignore_void {
            (
                "Promises must be awaited, add void operator to ignore.",
                "The promise must end with a call to .catch, or end with a call to .then with a rejection handler, or be explicitly marked as ignored with the `void` operator.",
            )
        } else {
            (
                "Promises must be awaited, add await operator.",
                "The promise must end with a call to .catch, or end with a call to .then with a rejection handler.",
            )
        };
        let help = if let Some(handler_span) = unhandled.non_function_handler {
            let handler_name = api
                .type_at_span(handler_span)
                .and_then(|ty| api.type_name(handler_span, ty))
                .unwrap_or("unknown");
            format!(
                "{help} The rejection handler has type `{handler_name}`, which is not callable."
            )
        } else {
            help.to_string()
        };
        let type_description = if unhandled.promise_array {
            "This array contains Promises and"
        } else {
            "This unhandled promise-like value"
        };
        let diagnostic = OxcDiagnostic::warn(message).with_help(help).with_label(
            unhandled.span.label(format!("{type_description} has type `{type_name}`.")),
        );
        if unhandled.promise_array {
            ctx.diagnostic(diagnostic);
            return;
        }

        let fixer = RuleFixer::new(crate::fixer::FixKind::Suggestion, ctx);
        let expression = &statement.expression;
        let await_replacement = suggestion_text("await", expression, ctx.source_text());
        let await_fix =
            fixer.replace(expression.span(), await_replacement).with_message("Add await operator.");
        if self.0.ignore_void {
            let void_fix = fixer
                .replace(expression.span(), suggestion_text("void", expression, ctx.source_text()))
                .with_message("Add void operator to ignore.");
            ctx.diagnostic_with_suggestions(diagnostic, [void_fix, await_fix]);
        } else {
            ctx.diagnostic_with_suggestions(diagnostic, [await_fix]);
        }
    }
}

#[derive(Clone, Copy)]
struct UnhandledPromise<'a> {
    span: Span,
    type_span: Span,
    ty: Ty<'a>,
    promise_array: bool,
    non_function_handler: Option<Span>,
}

fn find_unhandled<'a>(
    expression: &'a Expression<'a>,
    api: &TypedApiContext<'a>,
    check_thenables: bool,
    ignore_void: bool,
    safe_promises: &[TypeOrValueSpecifier],
) -> Option<UnhandledPromise<'a>> {
    let expression = expression.get_inner_expression();

    match expression {
        Expression::AssignmentExpression(_) => return None,
        Expression::SequenceExpression(sequence) => {
            return sequence.expressions.iter().find_map(|expression| {
                find_unhandled(expression, api, check_thenables, ignore_void, safe_promises)
            });
        }
        Expression::UnaryExpression(unary)
            if unary.operator == UnaryOperator::Void && !ignore_void =>
        {
            return find_unhandled(
                &unary.argument,
                api,
                check_thenables,
                ignore_void,
                safe_promises,
            );
        }
        Expression::ChainExpression(chain) => {
            if let ChainElement::CallExpression(call) = &chain.expression {
                return find_unhandled_call(
                    call,
                    chain.span,
                    api,
                    check_thenables,
                    ignore_void,
                    safe_promises,
                );
            }
        }
        _ => {}
    }

    let span = expression.span();
    let ty = api.type_at_span(span)?;
    if api.is_promise_array(span, check_thenables) {
        return Some(UnhandledPromise {
            span,
            type_span: span,
            ty,
            promise_array: true,
            non_function_handler: None,
        });
    }
    if matches!(expression, Expression::AwaitExpression(_)) {
        return None;
    }
    if !api.is_promise_like(span, check_thenables)
        || type_matches_specifier(api.type_name(span, ty), safe_promises)
    {
        return None;
    }

    match expression {
        Expression::CallExpression(call) => {
            find_unhandled_call(call, span, api, check_thenables, ignore_void, safe_promises)
        }
        Expression::ConditionalExpression(conditional) => {
            find_unhandled(&conditional.alternate, api, check_thenables, ignore_void, safe_promises)
                .or_else(|| {
                    find_unhandled(
                        &conditional.consequent,
                        api,
                        check_thenables,
                        ignore_void,
                        safe_promises,
                    )
                })
        }
        Expression::LogicalExpression(logical) => {
            find_unhandled(&logical.left, api, check_thenables, ignore_void, safe_promises).or_else(
                || find_unhandled(&logical.right, api, check_thenables, ignore_void, safe_promises),
            )
        }
        _ => Some(UnhandledPromise {
            span,
            type_span: span,
            ty,
            promise_array: false,
            non_function_handler: None,
        }),
    }
}

fn find_unhandled_call<'a>(
    call: &'a CallExpression<'a>,
    span: Span,
    api: &TypedApiContext<'a>,
    check_thenables: bool,
    ignore_void: bool,
    safe_promises: &[TypeOrValueSpecifier],
) -> Option<UnhandledPromise<'a>> {
    let ty = api.type_at_span(span)?;
    if !api.is_promise_like(span, check_thenables)
        || type_matches_specifier(api.type_name(span, ty), safe_promises)
    {
        return None;
    }
    let Some(member) = call.callee.get_member_expr() else {
        return Some(unhandled_call(span, ty));
    };
    match member.static_property_name() {
        Some("catch") if !call.arguments.is_empty() => {
            rejection_handler_result(call, 0, span, ty, api)
        }
        Some("then") if call.arguments.len() >= 2 => {
            rejection_handler_result(call, 1, span, ty, api)
        }
        Some("finally") => {
            let mut result =
                find_unhandled(member.object(), api, check_thenables, ignore_void, safe_promises)?;
            result.span = span;
            result.type_span = span;
            result.ty = ty;
            Some(result)
        }
        _ => Some(unhandled_call(span, ty)),
    }
}

fn rejection_handler_result<'a>(
    call: &'a CallExpression<'a>,
    index: usize,
    span: Span,
    ty: Ty<'a>,
    api: &TypedApiContext<'a>,
) -> Option<UnhandledPromise<'a>> {
    if call.arguments[..=index]
        .iter()
        .any(|argument| matches!(argument, Argument::SpreadElement(_)))
    {
        return Some(unhandled_call(span, ty));
    }
    let handler = call.arguments[index].as_expression()?;
    if api.is_callable(handler.span()) {
        None
    } else {
        Some(UnhandledPromise {
            non_function_handler: Some(handler.span()),
            ..unhandled_call(span, ty)
        })
    }
}

fn unhandled_call(span: Span, ty: Ty<'_>) -> UnhandledPromise<'_> {
    UnhandledPromise { span, type_span: span, ty, promise_array: false, non_function_handler: None }
}

fn is_iife(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return false;
    };
    matches!(
        call.callee.get_inner_expression(),
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

fn suggestion_text(operator: &str, expression: &Expression<'_>, source_text: &str) -> String {
    if operator == "await"
        && let Expression::UnaryExpression(unary) = expression.get_inner_expression()
        && unary.operator == UnaryOperator::Void
    {
        return format!("await {}", unary.argument.span().source_text(source_text));
    }
    let source = expression.span().source_text(source_text);
    if has_unary_operand_precedence(expression.get_inner_expression()) {
        format!("{operator} {source}")
    } else {
        format!("{operator} ({source})")
    }
}

fn has_unary_operand_precedence(expression: &Expression<'_>) -> bool {
    !matches!(
        expression,
        Expression::AssignmentExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::LogicalExpression(_)
            | Expression::SequenceExpression(_)
            | Expression::YieldExpression(_)
    )
}

fn is_known_safe_call(
    expression: &Expression<'_>,
    api: &TypedApiContext<'_>,
    specifiers: &[TypeOrValueSpecifier],
) -> bool {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return false;
    };
    let callee = call.callee.get_inner_expression();
    let value_name = match callee {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => {
            callee.get_member_expr().and_then(oxc_ast::ast::MemberExpression::static_property_name)
        }
    };
    value_name
        .is_some_and(|name| specifiers.iter().any(|specifier| specifier_has_name(specifier, name)))
        || api
            .type_at_span(callee.span())
            .is_some_and(|ty| type_matches_specifier(api.type_name(callee.span(), ty), specifiers))
}

fn type_matches_specifier(type_name: Option<&str>, specifiers: &[TypeOrValueSpecifier]) -> bool {
    let Some(type_name) = type_name else { return false };
    specifiers.iter().any(|specifier| {
        specifier_names(specifier).any(|name| {
            type_name == name
                || type_name.strip_prefix(name).is_some_and(|rest| rest.starts_with('<'))
        })
    })
}

fn specifier_has_name(specifier: &TypeOrValueSpecifier, expected: &str) -> bool {
    specifier_names(specifier).any(|name| name == expected)
}

fn specifier_names(specifier: &TypeOrValueSpecifier) -> impl Iterator<Item = &str> {
    let names = match specifier {
        TypeOrValueSpecifier::String(name) => {
            return std::slice::from_ref(name).iter().map(String::as_str);
        }
        TypeOrValueSpecifier::File(specifier) => &specifier.name,
        TypeOrValueSpecifier::Lib(specifier) => &specifier.name,
        TypeOrValueSpecifier::Package(specifier) => &specifier.name,
    };
    match names {
        NameSpecifier::Single(name) => std::slice::from_ref(name).iter().map(String::as_str),
        NameSpecifier::Multiple(names) => names.iter().map(String::as_str),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tester::Tester;
    use serde_json::json;

    #[test]
    fn test_rule() {
        let pass = vec![
            "await Promise.resolve();",
            "void Promise.resolve();",
            "Promise.resolve().catch(() => {});",
            "Promise.resolve().then(() => {}, () => {});",
            "const promise = Promise.resolve();",
        ];
        let fail = vec![
            "Promise.resolve();",
            "Promise.resolve().then(() => {});",
            "Promise.resolve().catch();",
            "Promise.resolve().catch(undefined);",
            "Promise.resolve().finally(() => {});",
            "[Promise.resolve()];",
        ];

        Tester::new(NoFloatingPromises::NAME, NoFloatingPromises::PLUGIN, pass, fail)
            .expect_fix(vec![("Promise.resolve();", "void Promise.resolve();")])
            .test_and_snapshot();
    }

    #[test]
    fn test_default_config() {
        let rule = NoFloatingPromises::default();
        let config = rule.to_configuration().unwrap().unwrap();

        // Verify the default values
        assert_eq!(config["allowForKnownSafeCalls"], json!([]));
        assert_eq!(config["allowForKnownSafePromises"], json!([]));
        assert_eq!(config["checkThenables"], json!(false));
        assert_eq!(config["ignoreIIFE"], json!(false));
        assert_eq!(config["ignoreVoid"], json!(true));
    }

    #[test]
    fn test_from_configuration() {
        let config_value = json!([{
            "allowForKnownSafeCalls": [{"from": "package", "name": "foo", "package": "some-package"}],
            "checkThenables": true,
            "ignoreVoid": false
        }]);

        let rule = NoFloatingPromises::from_configuration(config_value).unwrap();

        assert!(rule.0.check_thenables);
        assert!(!rule.0.ignore_void);
        assert_eq!(rule.0.allow_for_known_safe_calls.len(), 1);
    }

    #[test]
    fn test_round_trip() {
        let original_config = json!([{
            "allowForKnownSafeCalls": [{"from": "package", "name": "bar", "package": "test-pkg"}],
            "allowForKnownSafePromises": [{"from": "lib", "name": "Promise"}],
            "checkThenables": true,
            "ignoreIIFE": true,
            "ignoreVoid": false
        }]);

        let rule = NoFloatingPromises::from_configuration(original_config).unwrap();
        let serialized = rule.to_configuration().unwrap().unwrap();

        // Verify all fields are present in serialized output
        assert_eq!(
            serialized["allowForKnownSafeCalls"],
            json!([{"from": "package", "name": "bar", "package": "test-pkg"}])
        );
        assert_eq!(
            serialized["allowForKnownSafePromises"],
            json!([{"from": "lib", "name": "Promise"}])
        );
        assert_eq!(serialized["checkThenables"], json!(true));
        assert_eq!(serialized["ignoreIIFE"], json!(true));
        assert_eq!(serialized["ignoreVoid"], json!(false));
    }

    #[test]
    fn test_all_specifier_types() {
        let config_value = json!([{
            "allowForKnownSafeCalls": [
                "SomeType",  // string specifier
                {"from": "file", "name": "MyType", "path": "./types.ts"},  // file specifier with path
                {"from": "file", "name": ["Type1", "Type2"]},  // file specifier with multiple names
                {"from": "lib", "name": "Promise"},  // lib specifier
                {"from": "package", "name": "Observable", "package": "rxjs"}  // package specifier
            ],
            "checkThenables": false,
            "ignoreVoid": true
        }]);

        let rule = NoFloatingPromises::from_configuration(config_value).unwrap();

        assert_eq!(rule.0.allow_for_known_safe_calls.len(), 5);
        assert!(!rule.0.check_thenables);
        assert!(rule.0.ignore_void);

        // Verify serialization preserves all types
        let serialized = rule.to_configuration().unwrap().unwrap();
        assert_eq!(serialized["allowForKnownSafeCalls"].as_array().unwrap().len(), 5);
    }
}
