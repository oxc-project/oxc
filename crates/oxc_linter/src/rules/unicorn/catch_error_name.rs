use cow_utils::CowUtils;
use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{Argument, BindingPattern, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use oxc_syntax::{identifier::is_identifier_name, symbol::SymbolId};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::deserialize_regex_vec,
};

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

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CatchErrorNameConfig {
    /// A list of patterns to ignore when checking `catch` variable names. The pattern
    /// can be a string or regular expression.
    #[serde(default, deserialize_with = "deserialize_regex_vec")]
    ignore: Vec<Regex>,
    /// The name to use for error variables in `catch` blocks. You can customize it
    /// to something other than `'error'` (e.g., `'exception'`).
    #[serde(default = "default_error_name", deserialize_with = "deserialize_error_name")]
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
        Self { ignore: vec![], name: default_error_name() }
    }
}

fn default_error_name() -> CompactStr {
    CompactStr::new_const("error")
}

fn deserialize_error_name<'de, D>(deserializer: D) -> Result<CompactStr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    let name = name.trim();

    if is_identifier_name(name) { Ok(CompactStr::from(name)) } else { Ok(default_error_name()) }
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
    CatchErrorName,
    unicorn,
    style,
    fix,
    config = CatchErrorNameConfig,
    version = "0.0.14",
    short_description = "This rule enforces consistent and descriptive naming for error variables in `catch` statements, preventing the use of vague names like `badName` or `_` when the error is used.",
);

impl Rule for CatchErrorName {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<CatchErrorNameConfig>::from_value(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CatchParameter(catch_param) => {
                self.check_binding_identifier(ctx, &catch_param.pattern);
            }
            AstKind::CallExpression(call_expr) => {
                if let Some(member_expr) = call_expr.callee.as_member_expression() {
                    if member_expr.static_property_name().is_some_and(|name| name == "catch")
                        && let Some(arg) = call_expr.arguments.first()
                    {
                        self.check_function_arguments(arg, ctx);
                    }

                    if member_expr.static_property_name().is_some_and(|name| name == "then")
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
        if name
            .strip_prefix(self.name.as_str())
            .is_some_and(|suffix| suffix.bytes().all(|byte| byte == b'_'))
        {
            return true;
        }

        // strip all trailing underscores from the name.
        let stripped_name = name.trim_end_matches('_').cow_to_lowercase();

        if self.name == stripped_name.as_ref() {
            return true;
        }

        // Allow a qualified name, e.g. if we allow "error",
        // should also allow "diagnostic_error" or "diagnosticError".
        if stripped_name.ends_with(self.name.as_str().cow_to_lowercase().as_ref()) {
            return true;
        }

        // Check configured ignore patterns, do not use stripped name
        self.ignore.iter().any(|s| s.is_match(name))
    }

    fn check_function_arguments(&self, arg: &Argument, ctx: &LintContext) {
        let Some(expr) = arg.as_expression() else { return };

        let first_arg = match expr.without_parentheses() {
            Expression::ArrowFunctionExpression(arrow_expr) => arrow_expr.params.items.first(),
            Expression::FunctionExpression(fn_expr) => fn_expr.params.items.first(),
            _ => return,
        };

        let Some(arg) = first_arg else { return };
        self.check_binding_identifier(ctx, &arg.pattern);
    }

    fn check_binding_identifier(&self, ctx: &LintContext, binding_pattern_kind: &BindingPattern) {
        if let BindingPattern::BindingIdentifier(binding_ident) = binding_pattern_kind {
            if self.is_name_allowed(&binding_ident.name) {
                return;
            }

            let symbol_id = binding_ident.symbol_id();
            let mut iter = ctx.semantic().symbol_references(symbol_id).peekable();
            if binding_ident.name == "_" && iter.peek().is_none() {
                return;
            }

            let fixed_name = self.get_fixed_name(ctx, symbol_id);
            ctx.diagnostic_with_fix(
                catch_error_name_diagnostic(
                    binding_ident.name.as_str(),
                    fixed_name.as_str(),
                    binding_ident.span,
                ),
                |fixer| {
                    let basic_fix = fixer.replace(binding_ident.span, fixed_name.clone());
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

                        declaration_fix.push(fixer.replace(id.span, fixed_name.clone()));
                    }

                    declaration_fix
                        .with_message(format!("Rename `{}` to `{fixed_name}`", binding_ident.name))
                },
            );
        }
    }

    fn get_fixed_name(&self, ctx: &LintContext, symbol_id: SymbolId) -> CompactStr {
        let scoping = ctx.scoping();
        let scope_id = scoping.symbol_scope_id(symbol_id);
        let mut name = self.name.clone().into_compact_string();

        while scoping.find_binding(scope_id, name.as_str().into()).is_some()
            || scoping.get_resolved_references(symbol_id).any(|reference| {
                scoping.find_binding(reference.scope_id(), name.as_str().into()).is_some()
            })
            || scoping.root_unresolved_references().get(name.as_str()).is_some_and(|references| {
                references.iter().any(|&reference_id| {
                    let reference_scope_id = scoping.get_reference(reference_id).scope_id();
                    reference_scope_id == scope_id
                        || scoping.scope_is_descendant_of(reference_scope_id, scope_id)
                })
            })
        {
            name.push('_');
        }

        name.into()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("try {} catch (err) {}", Some(serde_json::json!([{"name": "err"}]))),
        (
            "try {
            } catch (outerError) {
                try {
                } catch (innerError) {}
            }",
            None,
        ),
        (
            "const handleError = error => {
                try {
                    doSomething();
                } catch (error_) {
                    console.log(error_);
                }
            }",
            None,
        ),
        (
            "const handleError = err => {
                try {
                    doSomething();
                } catch (err_) {
                    console.log(err_);
                }
            }",
            Some(serde_json::json!([{"name": "err"}])),
        ),
        (
            "const handleError = error => {
                const error_ = new Error('🦄');
                try {
                    doSomething();
                } catch (error__) {
                    console.log(error__);
                }
            }",
            None,
        ),
        ("obj.catch(error => {})", None),
        ("obj.catch?.(error => {})", None),
        ("obj.then(undefined, error => {})", None),
        ("obj.then(result => {}, error => {})", None),
        ("obj.then?.(undefined, error => {})", None),
        ("obj.then?.(result => {}, error => {})", None),
        (
            "const handleError = error => {
                obj.catch(error_ => { });
            }",
            None,
        ),
        (
            "const handleError = error => {
                obj.then(undefined, error_ => { });
            }",
            None,
        ),
        (
            "const handleError = err => {
                obj.catch(err_ => { });
            }",
            Some(serde_json::json!([{"name": "err"}])),
        ),
        (
            "const handleError = err => {
                obj.then(undefined, err_ => { });
            }",
            Some(serde_json::json!([{"name": "err"}])),
        ),
        (
            "const handleError = error => {
                const error_ = new Error('foo bar');
                obj.catch(error__ => { });
            }",
            None,
        ),
        (
            "const handleError = error => {
                const error_ = new Error('foo bar');
                obj.then(undefined, error__ => { });
            }",
            None,
        ),
        (
            "const handleError = error => {
                const error_ = new Error('foo bar');
                const error__ = new Error('foo bar');
                const error___ = new Error('foo bar');
                const error____ = new Error('foo bar');
                const error_____ = new Error('foo bar');
                const error______ = new Error('foo bar');
                const error_______ = new Error('foo bar');
                const error________ = new Error('foo bar');
                const error_________ = new Error('foo bar');
                obj.catch(error__________ => { });
            }",
            None,
        ),
        ("obj.catch(() => {})", None),
        ("obj.catch(err => {})", Some(serde_json::json!([{"name": "err"}]))),
        ("obj.then(undefined, err => {})", Some(serde_json::json!([{"name": "err"}]))),
        (
            "obj.catch(
                outerError => {
                    return obj2.catch(innerError => {})
                }
            )",
            None,
        ),
        ("obj.catch(function (error) {})", None),
        ("obj.then(undefined, function (error) {})", None),
        ("obj.catch(function onReject(error) {})", None),
        ("obj.then(undefined, function onReject(error) {})", None),
        ("obj.then(function onFulfilled(result) {}, function onReject(error) {})", None),
        ("obj.catch(function () {})", None),
        ("obj.catch(function (err) {})", Some(serde_json::json!([{"name": "err"}]))),
        ("obj.then(undefined, function (err) {})", Some(serde_json::json!([{"name": "err"}]))),
        (
            "obj.catch(function (outerError) {
                return obj2.catch(function (innerError) {
                })
            })",
            None,
        ),
        (
            "obj.then(undefined, function (outerError) {
                return obj2.then(undefined, function (innerError) {
                })
            })",
            None,
        ),
        (
            "obj.then(undefined, function (outerError) {
                return obj2.catch(function (innerError) {
                })
            })",
            None,
        ),
        (
            "obj.catch(function (outerError) {
                return obj2.then(undefined, function (innerError) {
                })
            })",
            None,
        ),
        ("obj.catch()", None),
        ("foo(function (error) {})", None),
        ("foo().then(function (error) {})", None),
        ("foo().catch(function (error) {})", None),
        (
            "try {
                throw new Error('message');
            } catch {
                console.log('failed');
            }",
            None,
        ),
        ("try {} catch ({message}) {}", None),
        ("obj.catch(function ({message}) {})", None),
        ("obj.catch(({message}) => {})", None),
        ("obj.then(undefined, ({message}) => {})", None),
        ("obj.catch(error => {}, anotherArgument)", None),
        ("obj.then(undefined, error => {}, anotherArgument)", None),
        ("obj.catch(_ => {})", None),
        ("obj.catch((_) => {})", None),
        ("obj.catch((_) => { console.log(foo); })", None),
        ("try {} catch (_) {}", None),
        ("try {} catch (_) { console.log(foo); }", None),
        (
            "try {
            } catch (_) {
                try {
                } catch (_) {}
            }",
            None,
        ),
        (
            "
                            try {
                            } catch (_) {
                                console.log(_);
                            }
                        ",
            Some(serde_json::json!([ { "ignore": ["^_$"] } ])),
        ),
        ("try {} catch (error) {}", None),
        ("try {} catch (error__) {}", None),
        ("try {} catch (descriptiveError) {}", None),
        ("try {} catch (descriptive_error) {}", None),
        ("try {} catch (descriptiveError__) {}", None),
        ("try {} catch (descriptive_error__) {}", None),
        ("try {} catch (skipThisNameCheck) {}", Some(serde_json::json!([{"ignore": ["^skip"]}]))),
        // This one needs to be modified from the upstream because there's no way to pass a regex with a flag (e.g. case insensitive flag).
        (
            "try {} catch (skipThisNameCheck) {}
            try {} catch (ignoreThisNameCheck) {}
            try {} catch (pleaseIgnoreThisNameCheck) {}",
            Some(serde_json::json!([{"ignore": ["^skip", "ignore", "Ignore"]}])),
        ),
        (
            "try {} catch (error1) {}
            try {} catch (error1__) {}
            try {} catch (error2) {}",
            Some(serde_json::json!([{"ignore": ["error\\d*"]}])),
        ),
        ("promise.catch(unicorn => {})", Some(serde_json::json!([{"ignore": ["unicorn"]}]))),
        // https://github.com/oxc-project/oxc/issues/12430
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
        ("promise.catch(exception__ => {})", Some(serde_json::json!([{"name": "exception_"}]))),
        ("try {} catch (_error) {}", None),
        ("try {} catch (_err) {}", Some(serde_json::json!([{"name": "_err"}]))),
        (
            "try {} catch (_err) { console.log(_err); }",
            Some(serde_json::json!([{"ignore": ["^_"]}])),
        ),
    ];

    let fail = vec![
        (
            "const handleError = error => {
                try {
                    doSomething();
                } catch (foo) {
                    console.log(foo);
                }
            }",
            None,
        ),
        (
            "const handleError = error => {
                const error9 = new Error('foo bar');
                try {
                    doSomething();
                } catch (foo) {
                    console.log(foo);
                }
            }",
            None,
        ),
        (
            "const handleError = error => {
                const error_ = new Error('foo bar');
                obj.catch(foo => { });
            }",
            None,
        ),
        (
            "obj.catch(err => {});
            obj.catch(err => {});
            obj.then(err => {}, err => {});
            obj.then(err => {}, err => {});",
            None,
        ),
        ("try {} catch (descriptiveError) {}", Some(serde_json::json!([{"name": "exception"}]))),
        ("try {} catch (e) {}", Some(serde_json::json!([{"name": "has_space_after "}]))),
        ("try {} catch (e) {}", Some(serde_json::json!([{"name": "1_start_with_a_number"}]))),
        ("try {} catch (e) {}", Some(serde_json::json!([{"name": "_){} evilCode; if(false"}]))),
        ("try {} catch (notMatching) {}", Some(serde_json::json!([{"ignore": []}]))),
        ("try {} catch (notMatching) {}", Some(serde_json::json!([{"ignore": ["unicorn"]}]))),
        // ("try {} catch (notMatching) {}", Some(serde_json::json!([{"ignore": [/unicorn/]}]))),
        (
            "try {} catch (error1) {}
            try {} catch (error1__) {}
            try {} catch (error2) {}
            try {} catch (unicorn) {}",
            Some(serde_json::json!([{"ignore": [r"error\d*"]}])),
        ),
        (
            "try {} catch (notMatching) {}
            try {} catch (unicorn) {}
            try {} catch (unicorn__) {}",
            Some(serde_json::json!([{"ignore": ["unicorn"]}])),
        ),
        ("promise.catch(notMatching => {})", Some(serde_json::json!([{"ignore": ["unicorn"]}]))),
        (
            "try {
            } catch (_) {
                console.log(_)
                try {
                } catch (_) {
                    console.log(_)
                }
            }",
            None,
        ),
        (
            "foo.then(() => {
                try {} catch (e) {}
            }).catch(err => err);",
            None,
        ),
        (
            "try {
                doSomething();
            } catch (anyName) { // Nesting of catch clauses disables the rule
                try {
                    doSomethingElse();
                } catch (anyOtherName) {
                    // ...
                }
            }",
            None,
        ),
        (
            r#"@DragSource({
                async endDrag(props, monitor, component) {
                    try {
                    } catch (e) {
                        alert("There was a problem moving these items: " + e);
                    }
                }
            })
            export default class A {}"#,
            None,
        ),
        (
            r#"@DragSource({
                async endDrag(props, monitor, component) {
                    try {
                    } catch (e) {
                        alert("1There was a problem moving these items: " + e);
                    }
                }
            })
            export default class A {}"#,
            None,
        ),
        ("try { y(); } catch (_err) {}", None),
        ("try { y(); } catch (_err) { console.log(_err); }", None),
        ("try {} catch (__) {}", None),
        ("promise.catch(_err => {})", None),
        ("promise.catch(function (_err) {})", None),
        ("promise.then(undefined, _err => {})", None),
        ("promise.then(undefined, function (_err) {})", None),
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
        (
            "try { run(); } catch (e) { const error = new Error('wrapped', { cause: e }); throw error; }",
            "try { run(); } catch (error_) { const error = new Error('wrapped', { cause: error_ }); throw error; }",
            None,
        ),
        (
            r#"const error = "outer"; try { throw new Error("inner"); } catch (e) { console.log(`${error} / ${e.message}`); }"#,
            r#"const error = "outer"; try { throw new Error("inner"); } catch (error_) { console.log(`${error} / ${error_.message}`); }"#,
            None,
        ),
        (
            "try {} catch (e) { const error = 1; }",
            "try {} catch (error_) { const error = 1; }",
            None,
        ),
        ("const error = 1; try {} catch (e) {}", "const error = 1; try {} catch (error_) {}", None),
        (
            "try {} catch (e) { const error = 1; const error_ = 2; console.log(e); }",
            "try {} catch (error__) { const error = 1; const error_ = 2; console.log(error__); }",
            None,
        ),
        (
            "try {} catch (e) { function f(error) { console.log(e, error); } }",
            "try {} catch (error_) { function f(error) { console.log(error_, error); } }",
            None,
        ),
        (
            "try {} catch (e) { console.log(error, e); }",
            "try {} catch (error_) { console.log(error, error_); }",
            None,
        ),
        (
            "try {} catch (e) { function f() { return error; } }",
            "try {} catch (error_) { function f() { return error; } }",
            None,
        ),
        (
            "promise.catch(e => { const error = 1; console.log(e); });",
            "promise.catch(error_ => { const error = 1; console.log(error_); });",
            None,
        ),
        (
            "promise.then(undefined, function (e, error) { console.log(e, error); });",
            "promise.then(undefined, function (error_, error) { console.log(error_, error); });",
            None,
        ),
        (
            "const exception = 1; promise.catch(e => console.log(exception, e));",
            "const exception = 1; promise.catch(exception_ => console.log(exception, exception_));",
            Some(serde_json::json!([{"name": "exception"}])),
        ),
        (
            "try {} catch (e) { console.log(e); } { const error = 1; }",
            "try {} catch (error) { console.log(error); } { const error = 1; }",
            None,
        ),
        (
            "try {} catch (e) {} console.log(error);",
            "try {} catch (error) {} console.log(error);",
            None,
        ),
        (
            "const exception_ = 1; promise.catch(e => console.log(exception_, e));",
            "const exception_ = 1; promise.catch(exception__ => console.log(exception_, exception__));",
            Some(serde_json::json!([{"name": "exception_"}])),
        ),
        (
            "const error = 1; try {} catch (e) { console.log(error_, e); }",
            "const error = 1; try {} catch (error__) { console.log(error_, error__); }",
            None,
        ),
        ("try { y(); } catch (_err) {}", "try { y(); } catch (error) {}", None),
        (
            "try { y(); } catch (_err) { console.log(_err); }",
            "try { y(); } catch (error) { console.log(error); }",
            None,
        ),
        ("promise.catch(_err => {})", "promise.catch(error => {})", None),
        (
            "promise.then(undefined, function (_err) {})",
            "promise.then(undefined, function (error) {})",
            None,
        ),
    ];

    Tester::new(CatchErrorName::NAME, CatchErrorName::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
