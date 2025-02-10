use oxc_ast::{
    ast::{Argument, BindingPatternKind, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn catch_error_name_diagnostic(
    caught_ident: &str,
    expected_name: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The catch parameter {caught_ident:?} should be named {expected_name:?}"
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct CatchErrorName(Box<CatchErrorNameConfig>);

#[derive(Debug, Clone)]
pub struct CatchErrorNameConfig {
    ignore: Vec<CompactStr>,
    name: CompactStr,
}

impl std::ops::Deref for CatchErrorName {
    type Target = CatchErrorNameConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for CatchErrorNameConfig {
    fn default() -> Self {
        Self { ignore: vec![], name: CompactStr::new_const("error") }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces naming conventions for catch statements.
    ///
    /// ### Why is this bad?
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// try { } catch (foo) { }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// try { } catch (error) { }
    /// ```
    CatchErrorName,
    unicorn,
    style,
    pending
);

impl Rule for CatchErrorName {
    fn from_configuration(value: serde_json::Value) -> Self {
        let ignored_names = value
            .get(0)
            .and_then(|v| v.get("ignored"))
            .and_then(serde_json::Value::as_array)
            .unwrap_or(&vec![])
            .iter()
            .map(serde_json::Value::as_str)
            .filter(Option::is_some)
            .map(|x| CompactStr::from(x.unwrap()))
            .collect::<Vec<CompactStr>>();

        let allowed_name = CompactStr::from(
            value
                .get(0)
                .and_then(|v| v.get("name"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("error"),
        );

        Self(Box::new(CatchErrorNameConfig { ignore: ignored_names, name: allowed_name }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CatchParameter(catch_param) = node.kind() {
            if let oxc_ast::ast::BindingPatternKind::BindingIdentifier(binding_ident) =
                &catch_param.pattern.kind
            {
                if self.is_name_allowed(&binding_ident.name) {
                    return;
                }

                if binding_ident.name.starts_with('_') {
                    if symbol_has_references(binding_ident.symbol_id(), ctx) {
                        ctx.diagnostic(catch_error_name_diagnostic(
                            binding_ident.name.as_str(),
                            &self.name,
                            binding_ident.span,
                        ));
                    }
                    return;
                }

                ctx.diagnostic(catch_error_name_diagnostic(
                    binding_ident.name.as_str(),
                    &self.name,
                    binding_ident.span,
                ));
            }
        }

        if let AstKind::CallExpression(call_expr) = node.kind() {
            if let Some(member_expr) = call_expr.callee.as_member_expression() {
                if member_expr.static_property_name() == Some("catch") {
                    if let Some(arg0) = call_expr.arguments.first() {
                        if let Some(diagnostic) = self.check_function_arguments(arg0, ctx) {
                            ctx.diagnostic(diagnostic);
                        }
                    }
                }

                if member_expr.static_property_name() == Some("then") {
                    if let Some(arg0) = call_expr.arguments.get(1) {
                        if let Some(diagnostic) = self.check_function_arguments(arg0, ctx) {
                            ctx.diagnostic(diagnostic);
                        }
                    }
                }
            }
        }
    }
}

impl CatchErrorName {
    fn is_name_allowed(&self, name: &str) -> bool {
        self.name == name || self.ignore.iter().any(|s| s.as_str() == name)
    }

    fn check_function_arguments(
        &self,
        arg0: &Argument,
        ctx: &LintContext,
    ) -> Option<OxcDiagnostic> {
        let expr = arg0.as_expression()?;
        let expr = expr.without_parentheses();

        if let Expression::ArrowFunctionExpression(arrow_expr) = expr {
            if let Some(arg0) = arrow_expr.params.items.first() {
                if let BindingPatternKind::BindingIdentifier(v) = &arg0.pattern.kind {
                    if self.is_name_allowed(&v.name) {
                        return None;
                    }

                    if v.name.starts_with('_') {
                        if symbol_has_references(v.symbol_id(), ctx) {
                            ctx.diagnostic(catch_error_name_diagnostic(
                                v.name.as_str(),
                                &self.name,
                                v.span,
                            ));
                        }

                        return None;
                    }

                    return Some(catch_error_name_diagnostic(v.name.as_str(), &self.name, v.span));
                }
            }
        }

        if let Expression::FunctionExpression(fn_expr) = expr {
            if let Some(arg0) = fn_expr.params.items.first() {
                if let BindingPatternKind::BindingIdentifier(binding_ident) = &arg0.pattern.kind {
                    if self.is_name_allowed(&binding_ident.name) {
                        return None;
                    }

                    if binding_ident.name.starts_with('_') {
                        if symbol_has_references(binding_ident.symbol_id(), ctx) {
                            ctx.diagnostic(catch_error_name_diagnostic(
                                binding_ident.name.as_str(),
                                &self.name,
                                binding_ident.span,
                            ));
                        }

                        return None;
                    }

                    return Some(catch_error_name_diagnostic(
                        binding_ident.name.as_str(),
                        &self.name,
                        binding_ident.span,
                    ));
                }
            }
        }

        None
    }
}

fn symbol_has_references(symbol_id: SymbolId, ctx: &LintContext) -> bool {
    ctx.semantic().symbol_references(symbol_id).next().is_some()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("try { } catch (error) { }", None),
        ("try { } catch (err) { }", Some(serde_json::json!([{"name": "err"}]))),
        ("obj.catch(error => { })", None),
        ("obj.then(undefined, error => { })", None),
        ("obj.then(result => { }, error => { })", None),
        ("obj.catch(() => { })", None),
        ("obj.catch(err => { })", Some(serde_json::json!([{"name": "err"}]))),
        ("obj.then(undefined, err => { })", Some(serde_json::json!([{"name": "err"}]))),
        ("obj.catch(function (error) { })", None),
        ("obj.then(undefined, function (error) { })", None),
        ("obj.catch(function onReject(error) { })", None),
        ("obj.then(undefined, function onReject(error) { })", None),
        ("obj.then(function onFulfilled(result) { }, function onReject(error) { })", None),
        ("obj.catch(function () { })", None),
        ("obj.catch(function (err) { })", Some(serde_json::json!([{"name": "err"}]))),
        ("obj.then(undefined, function (err) { })", Some(serde_json::json!([{"name": "err"}]))),
        ("obj.catch()", None),
        ("foo(function (error) { })", None),
        ("foo().then(function (error) { })", None),
        ("foo().catch(function (error) { })", None),
        ("try { } catch ({message}) { }", None),
        ("obj.catch(function ({message}) { })", None),
        ("obj.catch(({message}) => { })", None),
        ("obj.then(undefined, ({message}) => { })", None),
        ("obj.catch(error => { }, anotherArgument)", None),
        ("obj.then(undefined, error => { }, anotherArgument)", None),
        ("obj.catch(_ => { })", None),
        ("obj.catch((_) => { })", None),
        ("obj.catch((_) => { console.log(foo); })", None),
        ("try { } catch (_) { }", None),
        ("try { } catch (_) { console.log(foo); }", None),
        (
            "
							try {
							} catch (_) {
								console.log(_);
							}
						",
            Some(serde_json::json!([{"ignored": ["_"]}])),
        ),
        ("try { } catch (error) { }", None),
        ("promise.catch(unicorn => { })", Some(serde_json::json!([{"ignored": ["unicorn"]}]))),
        ("try { } catch (exception) { }", Some(serde_json::json!([{"name": "exception"}]))),
    ];

    let fail = vec![
        ("try { } catch (descriptiveError) { }", Some(serde_json::json!([{"name": "exception"}]))),
        ("try { } catch (e) { }", Some(serde_json::json!([{"name": "has_space_after "}]))),
        ("try { } catch (e) { }", Some(serde_json::json!([{"name": "1_start_with_a_number"}]))),
        ("try { } catch (e) { }", Some(serde_json::json!([{"name": "_){ } evilCode; if(false"}]))),
        ("try { } catch (notMatching) { }", Some(serde_json::json!([{"ignore": []}]))),
        ("try { } catch (notMatching) { }", Some(serde_json::json!([{"ignore": ["unicorn"]}]))),
        ("try { } catch (notMatching) { }", Some(serde_json::json!([{"ignore": ["unicorn"]}]))),
        ("try { } catch (_) { console.log(_) }", None),
        ("promise.catch(notMatching => { })", Some(serde_json::json!([{"ignore": ["unicorn"]}]))),
        ("promise.catch((foo) => { })", None),
        ("promise.catch(function (foo) { })", None),
        ("promise.catch((function (foo) { }))", None),
        ("promise.then(function (foo) { }).catch((foo) => { })", None),
        ("promise.then(undefined, function (foo) { })", None),
        ("promise.then(undefined, (foo) => { })", None),
    ];

    Tester::new(CatchErrorName::NAME, CatchErrorName::PLUGIN, pass, fail).test_and_snapshot();
}
