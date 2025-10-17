use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{Argument, BindingPatternKind, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use oxc_syntax::identifier::is_identifier_name;

use crate::{AstNode, context::LintContext, rule::Rule};

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
    ignore: Vec<Regex>,
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
    /// This rule enforces consistent and descriptive naming for error variables
    /// in `catch` statements, preventing the use of vague names like `badName`
    /// or `_` when the error is used.
    ///
    /// ### Why is this bad?
    ///
    /// Using non-descriptive names like `badName` or `_` makes the code harder
    /// to read and understand, especially when debugging. It's important to use
    /// clear, consistent names to represent errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// try { } catch (badName) { }
    ///
    /// // `_` is not allowed if it's used
    /// try {} catch (_) { console.log(_) }
    ///
    /// promise.catch(badName => {});
    ///
    /// promise.then(undefined, badName => {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// try { } catch (error) { }
    ///
    /// // `_` is allowed if it's not used
    /// try {} catch (_) { console.log(123) }
    ///
    /// promise.catch(error => {});
    ///
    /// promise.then(undefined, error => {});
    /// ```
    ///
    /// ### Options
    ///
    /// #### name
    ///
    /// `{ type: string, default: "error" }`
    ///
    /// The name to use for error variables in `catch` blocks. You can customize it
    /// to something other than `'error'` (e.g., `'exception'`).
    ///
    /// Example:
    /// ```json
    /// "unicorn/catch-error-name": [
    ///   "error",
    ///   { "name": "exception" }
    /// ]
    /// ```
    ///
    /// #### ignore
    ///
    /// `{ type: Array<string | RegExp>, default: [] }`
    ///
    /// A list of patterns to ignore when checking `catch` variable names. The pattern
    /// can be a string or regular expression.
    ///
    /// Example:
    /// ```json
    /// "unicorn/catch-error-name": [
    ///   "error",
    ///   {
    ///     "ignore": [
    ///       "^error\\d*$"
    ///     ]
    ///   }
    /// ]
    /// ```
    CatchErrorName,
    unicorn,
    style,
    fix
);

impl Rule for CatchErrorName {
    fn from_configuration(value: serde_json::Value) -> Self {
        let ignored_names = value
            .get(0)
            .and_then(|v| v.get("ignore"))
            .and_then(serde_json::Value::as_array)
            .unwrap_or(&vec![])
            .iter()
            .filter_map(serde_json::Value::as_str)
            .filter_map(|x| Regex::new(x).ok())
            .collect::<Vec<Regex>>();

        let allowed_name = CompactStr::from(
            value
                .get(0)
                .and_then(|v| v.get("name"))
                .and_then(serde_json::Value::as_str)
                .and_then(|name| {
                    let name = name.trim();
                    is_identifier_name(name).then_some(name)
                })
                .unwrap_or("error"),
        );

        Self(Box::new(CatchErrorNameConfig { ignore: ignored_names, name: allowed_name }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CatchParameter(catch_param) => {
                self.check_binding_identifier(ctx, &catch_param.pattern.kind);
            }
            AstKind::CallExpression(call_expr) => {
                if let Some(member_expr) = call_expr.callee.as_member_expression() {
                    if member_expr.static_property_name() == Some("catch")
                        && let Some(arg) = call_expr.arguments.first()
                    {
                        self.check_function_arguments(arg, ctx);
                    }

                    if member_expr.static_property_name() == Some("then")
                        && let Some(arg) = call_expr.arguments.get(1)
                    {
                        self.check_function_arguments(arg, ctx);
                    }
                }
            }
            _ => {}
        }
    }
}

impl CatchErrorName {
    fn is_name_allowed(&self, name: &str) -> bool {
        self.name == name || self.ignore.iter().any(|s| s.is_match(name))
    }

    fn check_function_arguments(&self, arg: &Argument, ctx: &LintContext) {
        let Some(expr) = arg.as_expression() else { return };

        let first_arg = match expr.without_parentheses() {
            Expression::ArrowFunctionExpression(arrow_expr) => arrow_expr.params.items.first(),
            Expression::FunctionExpression(fn_expr) => fn_expr.params.items.first(),
            _ => return,
        };

        let Some(arg) = first_arg else { return };
        self.check_binding_identifier(ctx, &arg.pattern.kind);
    }

    fn check_binding_identifier(
        &self,
        ctx: &LintContext,
        binding_pattern_kind: &BindingPatternKind,
    ) {
        if let BindingPatternKind::BindingIdentifier(binding_ident) = binding_pattern_kind {
            if self.is_name_allowed(&binding_ident.name) {
                return;
            }

            let symbol_id = binding_ident.symbol_id();
            let mut iter = ctx.semantic().symbol_references(symbol_id).peekable();
            if binding_ident.name.starts_with('_') && iter.peek().is_none() {
                return;
            }

            ctx.diagnostic_with_fix(
                catch_error_name_diagnostic(
                    binding_ident.name.as_str(),
                    &self.name,
                    binding_ident.span,
                ),
                |fixer| {
                    let basic_fix = fixer.replace(binding_ident.span, self.name.clone());
                    if iter.peek().is_none() {
                        return basic_fix;
                    }

                    let fixer = fixer.for_multifix();
                    let capacity = ctx.scoping().get_resolved_reference_ids(symbol_id).len() + 1;

                    let mut declaration_fix = fixer.new_fix_with_capacity(capacity);

                    declaration_fix.push(basic_fix);
                    for reference in iter {
                        let node = ctx.nodes().get_node(reference.node_id()).kind();
                        let Some(id) = node.as_identifier_reference() else { continue };

                        declaration_fix.push(fixer.replace(id.span, self.name.clone()));
                    }

                    declaration_fix
                        .with_message(format!("Rename `{}` to `{}`", binding_ident.name, self.name))
                },
            );
        }
    }
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
            Some(serde_json::json!([{"ignore": ["_"]}])),
        ),
        ("try { } catch (error) { }", None),
        ("promise.catch(unicorn => { })", Some(serde_json::json!([{"ignore": ["unicorn"]}]))),
        //   https://github.com/oxc-project/oxc/issues/12430
        (
            "try {
  // some codes
} catch (error: unknown) {
  try {
    // some codes
  } catch (error2: unknown) {
    // some codes
  }
}",
            Some(serde_json::json!([{"ignore": [ "^error\\d*$"]}])),
        ),
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
        ("try { } catch (err) { console.error(err) }", None),
        ("promise.catch(notMatching => { })", Some(serde_json::json!([{"ignore": ["unicorn"]}]))),
        ("promise.catch((foo) => { })", None),
        ("promise.catch(function (foo) { })", None),
        ("promise.catch((function (foo) { }))", None),
        ("promise.then(function (foo) { }).catch((foo) => { })", None),
        ("promise.then(undefined, function (foo) { })", None),
        ("promise.then(undefined, (foo) => { })", None),
    ];

    let fix = vec![
        (
            "try { } catch (descriptiveError) { }",
            "try { } catch (exception) { }",
            Some(serde_json::json!([{"name": "exception"}])),
        ),
        (
            "try { } catch (e) { }",
            "try { } catch (has_space_after) { }",
            Some(serde_json::json!([{"name": "has_space_after "}])),
        ),
        (
            "try { } catch (e) { }",
            "try { } catch (error) { }",
            Some(serde_json::json!([{"name": "1_start_with_a_number"}])),
        ),
        (
            "try { } catch (e) { }",
            "try { } catch (error) { }",
            Some(serde_json::json!([{"name": "_){ } evilCode; if(false"}])),
        ),
        (
            "try { } catch (notMatching) { }",
            "try { } catch (error) { }",
            Some(serde_json::json!([{"ignore": []}])),
        ),
        (
            "try { } catch (notMatching) { }",
            "try { } catch (error) { }",
            Some(serde_json::json!([{"ignore": ["unicorn"]}])),
        ),
        (
            "try { } catch (notMatching) { }",
            "try { } catch (error) { }",
            Some(serde_json::json!([{"ignore": ["unicorn"]}])),
        ),
        (
            "try { } catch (_) { console.log(_) }",
            "try { } catch (error) { console.log(error) }",
            None,
        ),
        (
            "try { } catch (err) { console.error(err) }",
            "try { } catch (error) { console.error(error) }",
            None,
        ),
        (
            "promise.catch(notMatching => { })",
            "promise.catch(error => { })",
            Some(serde_json::json!([{"ignore": ["unicorn"]}])),
        ),
        ("promise.catch((foo) => { })", "promise.catch((error) => { })", None),
        ("promise.catch(function (foo) { })", "promise.catch(function (error) { })", None),
        ("promise.catch((function (foo) { }))", "promise.catch((function (error) { }))", None),
        (
            "promise.then(function (foo) { }).catch((foo) => { })",
            "promise.then(function (foo) { }).catch((error) => { })",
            None,
        ),
        (
            "promise.then(undefined, function (foo) { })",
            "promise.then(undefined, function (error) { })",
            None,
        ),
        ("promise.then(undefined, (foo) => { })", "promise.then(undefined, (error) => { })", None),
    ];

    Tester::new(CatchErrorName::NAME, CatchErrorName::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
