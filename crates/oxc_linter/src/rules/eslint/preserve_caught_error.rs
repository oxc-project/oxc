use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrowFunctionExpression, BindingPattern, CatchClause, Expression, Function,
        IdentifierReference, ObjectExpression, ObjectPropertyKind, PropertyKey, StaticBlock,
        ThrowStatement, TryStatement,
    },
};
use oxc_ast_visit::VisitJs;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{IsGlobalReference, ScopeFlags};
use oxc_span::{GetSpan, Span};
use oxc_str::static_ident;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

const ADD_CAUSE_PROPERTY: &str = "Add cause property to the thrown error";
const REPLACE_CAUSE_PROPERTY: &str = "Replace cause property value with the caught error";

fn preserve_caught_error_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("There is no cause error attached to this new thrown error.")
        .with_help(
            "Preserve the original error by using the `cause` property when re-throwing errors.",
        )
        .with_label(span)
}

fn missing_catch_parameter_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The caught error is not accessible because the catch clause has no error parameter.",
    )
    .with_help("Add an error parameter to the catch clause to access the caught error.")
    .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct PreserveCaughtErrorOptions {
    /// When set to `true`, requires catch clauses that throw a new error to have a
    /// caught error parameter, so the caught error is not discarded when re-throwing.
    require_catch_parameter: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PreserveCaughtError(PreserveCaughtErrorOptions);

struct ThrowFinder<'a, 'ctx> {
    catch_param: Option<&'a BindingPattern<'a>>,
    ctx: &'ctx LintContext<'a>,
}

impl<'a> VisitJs<'a> for ThrowFinder<'a, '_> {
    fn visit_throw_statement(&mut self, throw_stmt: &ThrowStatement<'a>) {
        let (callee, type_arguments, args) = match &throw_stmt.argument {
            Expression::NewExpression(new_expr) => {
                (&new_expr.callee, &new_expr.type_arguments, &new_expr.arguments)
            }
            Expression::CallExpression(call_expr) => {
                (&call_expr.callee, &call_expr.type_arguments, &call_expr.arguments)
            }
            _ => return,
        };

        let Some(options_argument_index) = error_options_argument_index(callee, self.ctx) else {
            return;
        };

        let Some(catch_param) = self.catch_param else {
            self.ctx.diagnostic(missing_catch_parameter_diagnostic(throw_stmt.span));
            return;
        };

        let BindingPattern::BindingIdentifier(catch_ident) = catch_param else {
            self.ctx.diagnostic(preserve_caught_error_diagnostic(throw_stmt.span));
            return;
        };

        match args.get(options_argument_index) {
            Some(Argument::ObjectExpression(obj_expr)) => {
                if has_cause_property(obj_expr, catch_param, self.ctx) {
                    return;
                }
            }
            Some(_) => return,
            None => {}
        }

        // Allow spread arguments - they may contain cause property
        if args.iter().any(|arg| matches!(arg, Argument::SpreadElement(_))) {
            return;
        }

        self.ctx.diagnostic_with_fix(preserve_caught_error_diagnostic(throw_stmt.span), |fixer| {
            let ident = catch_ident.name;
            // Keep the catch binding unchanged as part of each fix, so removing or renaming it
            // conflicts with inserting a reference to it (e.g. `prefer-optional-catch-binding`).
            let preserve_catch_param = || fixer.replace_with(catch_param, catch_param);
            let cause_prop_text = format!("cause: {}", ident.as_str());

            match args.len() {
                0 => {
                    // find starting `(` of call after the callee and its type arguments
                    let args_start = match type_arguments {
                        Some(type_args) => type_args.span.end,
                        None => callee.span().end,
                    };
                    if let Some(offset) = fixer.find_next_token_within(
                        args_start,
                        throw_stmt.argument.span().end,
                        "(",
                    ) {
                        let mut fix = fixer.new_fix_with_capacity(4);
                        let span = Span::sized(args_start + offset, 1);
                        if let Expression::Identifier(ident) = callee
                            && is_aggregate_error(ident, self.ctx)
                        {
                            // AggregateError takes options as its third argument
                            fix.push(fixer.insert_text_after_range(span, "[], "));
                        }
                        fix.push(fixer.insert_text_after_range(span, "\"\", "));
                        fix.push(
                            fixer.insert_text_after_range(span, format!("{{ {cause_prop_text} }}")),
                        );
                        return fix.extend(preserve_catch_param()).with_message(ADD_CAUSE_PROPERTY);
                    }
                }
                1 => {
                    let span = args[0].span();
                    // insert comma
                    let mut fix = fixer.new_fix_with_capacity(4);
                    if let Expression::Identifier(ident) = callee
                        && is_aggregate_error(ident, self.ctx)
                    {
                        // AggregateError takes options as its third argument
                        fix.push(fixer.insert_text_after_range(span, ", \"\""));
                    }
                    fix.push(fixer.insert_text_after_range(span, ", "));
                    fix.push(
                        fixer.insert_text_after_range(span, format!("{{ {cause_prop_text} }}")),
                    );
                    return fix.extend(preserve_catch_param()).with_message(ADD_CAUSE_PROPERTY);
                }
                2 => {
                    if let Expression::Identifier(ident) = callee
                        && is_aggregate_error(ident, self.ctx)
                    {
                        // AggregateError takes options as its third argument
                        let span = args[1].span();
                        let mut fix = fixer.new_fix_with_capacity(3);
                        fix.push(fixer.insert_text_after_range(span, ", "));
                        fix.push(
                            fixer.insert_text_after_range(span, format!("{{ {cause_prop_text} }}")),
                        );
                        return fix.extend(preserve_catch_param()).with_message(ADD_CAUSE_PROPERTY);
                    }

                    // if the second argument is an existing object, merge into it
                    if let Argument::ObjectExpression(obj_expr) = &args[1] {
                        let cause_prop = obj_expr.properties.iter().find(|prop| match prop {
                            ObjectPropertyKind::ObjectProperty(prop) => {
                                let PropertyKey::StaticIdentifier(ident) = &prop.key else {
                                    return false;
                                };
                                ident.name == "cause"
                            }
                            ObjectPropertyKind::SpreadProperty(_) => true,
                        });

                        if let Some(cause_prop) = cause_prop {
                            // if the identifier name is not the catch parameter, replace it
                            if let ObjectPropertyKind::ObjectProperty(prop) = cause_prop
                                && !is_catch_parameter(&prop.value, catch_param, self.ctx)
                            {
                                if prop.shorthand || prop.method || prop.kind.is_accessor() {
                                    return fixer
                                        .replace(prop.span(), cause_prop_text)
                                        .extend(preserve_catch_param())
                                        .with_message(REPLACE_CAUSE_PROPERTY);
                                }
                                return fixer
                                    .replace(prop.value.span(), ident.as_str().to_string())
                                    .extend(preserve_catch_param())
                                    .with_message(REPLACE_CAUSE_PROPERTY);
                            }
                        } else if obj_expr.properties.is_empty() {
                            return fixer
                                .insert_text_after_range(
                                    obj_expr.span().shrink_right(1),
                                    format!(" {cause_prop_text} "),
                                )
                                .extend(preserve_catch_param())
                                .with_message(ADD_CAUSE_PROPERTY);
                        } else if let Some(last_prop) = obj_expr.properties.last() {
                            let mut fix = fixer.new_fix_with_capacity(3);
                            fix.push(fixer.insert_text_after_range(last_prop.span(), ", "));
                            fix.push(
                                fixer.insert_text_after_range(last_prop.span(), cause_prop_text),
                            );
                            return fix
                                .extend(preserve_catch_param())
                                .with_message(ADD_CAUSE_PROPERTY);
                        }
                    }
                }
                _ => {}
            }

            fixer.noop()
        });
    }

    // Do not traverse into nested functions/closures or static blocks within the catch block
    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}

    fn visit_static_block(&mut self, _block: &StaticBlock<'a>) {}

    // Do not traverse into nested catch clauses. They have their own caught error and will be
    // analyzed separately when visiting their associated `TryStatement`.
    fn visit_catch_clause(&mut self, _catch_clause: &CatchClause<'a>) {}
}

fn error_options_argument_index(expr: &Expression, ctx: &LintContext) -> Option<usize> {
    let Expression::Identifier(ident) = expr else {
        return None;
    };

    let index = match ident.name.as_str() {
        "Error" | "EvalError" | "RangeError" | "ReferenceError" | "SyntaxError" | "TypeError"
        | "URIError" => 1,
        "AggregateError" => 2,
        _ => return None,
    };
    ident.is_global_reference(ctx.scoping()).then_some(index)
}

fn is_aggregate_error(ident: &IdentifierReference, ctx: &LintContext) -> bool {
    ident.is_global_reference_name(static_ident!("AggregateError"), ctx.scoping())
}

fn has_cause_property(
    obj_expr: &ObjectExpression,
    catch_param: &BindingPattern,
    ctx: &LintContext,
) -> bool {
    for prop in &obj_expr.properties {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => {
                let PropertyKey::StaticIdentifier(ident) = &prop.key else {
                    continue;
                };
                if ident.name == "cause" {
                    return is_catch_parameter(&prop.value, catch_param, ctx);
                }
            }
            ObjectPropertyKind::SpreadProperty(_) => return true,
        }
    }
    false
}

fn is_catch_parameter(expr: &Expression, catch_param: &BindingPattern, ctx: &LintContext) -> bool {
    let BindingPattern::BindingIdentifier(binding) = &catch_param else {
        return false;
    };

    let catch_symbol_id = binding.symbol_id();

    let Expression::Identifier(ident) = expr else {
        return false;
    };

    let reference = ctx.scoping().get_reference(ident.reference_id());
    let Some(symbol_id) = reference.symbol_id() else {
        return false;
    };

    symbol_id == catch_symbol_id
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that when re-throwing an error in a catch block, the original error
    /// is preserved using the 'cause' property.
    ///
    /// ### Why is this bad?
    ///
    /// Re-throwing an error without preserving the original error loses important
    /// debugging information and makes it harder to trace the root cause of issues.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// try {
    ///     doSomething();
    /// } catch (err) {
    ///     throw new Error("Something failed");
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// try {
    ///     doSomething();
    /// } catch (err) {
    ///     throw new Error("Something failed", { cause: err });
    /// }
    /// ```
    PreserveCaughtError,
    eslint,
    suspicious,
    recommended,
    conditional_fix,
    config = PreserveCaughtErrorOptions,
    version = "1.16.0",
    short_description = "Enforces that when re-throwing an error in a catch block, the original error is preserved using the 'cause' property.",
);
impl PreserveCaughtError {
    fn check_try_statement<'a>(&self, try_stmt: &'a TryStatement<'a>, ctx: &LintContext<'a>) {
        if let Some(catch_clause) = &try_stmt.handler {
            self.check_catch_clause(catch_clause, ctx);
        }
    }

    fn check_catch_clause<'a>(&self, catch_clause: &'a CatchClause<'a>, ctx: &LintContext<'a>) {
        let catch_param = catch_clause.param.as_ref().map(|param| &param.pattern);
        if catch_param.is_none() && !self.0.require_catch_parameter {
            return;
        }
        let mut finder = ThrowFinder { catch_param, ctx };
        finder.visit_block_statement(catch_clause.body.as_ref());
    }
}

impl Rule for PreserveCaughtError {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TryStatement(try_stmt) = node.kind() else {
            return;
        };
        self.check_try_statement(try_stmt, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r#"try {
			        throw new Error("Original error");
			    } catch (error) {
			        throw new Error("Failed to perform error prone operations", { cause: error });
			    }"#,
            None,
        ),
        (
            "try {
			        doSomething();
			    } catch (e) {
			        console.error(e);
			    }",
            None,
        ),
        (
            r#"try {
			        doSomething();
			    } catch (err) {
			        throw new Error("Failed", { cause: err, extra: 42 });
			    }"#,
            None,
        ),
        (
            r#"try {
			        doSomething();
			    } catch (error) {
			        switch (error.code) {
			            case "A":
			                throw new Error("Type A", { cause: error });
			            case "B":
			                throw new Error("Type B", { cause: error });
			            default:
			                throw new Error("Other", { cause: error });
			        }
			    }"#,
            None,
        ),
        (
            r#"try {
					// ...
				} catch (err) {
					const opts = { cause: err }
					throw new Error("msg", { ...opts });
				}
				"#,
            None,
        ),
        (
            "try {
				} catch (error) {
					foo = {
						bar() {
							throw new Error();
						}
					};
				}",
            None,
        ),
        (
            "try {
							doSomething();
						} catch (error) {
							const args = [];
							throw new Error(...args);
					}",
            None,
        ),
        (
            r#"import { Error } from "./my-custom-error.js";
						try {
							doSomething();
						} catch (error) {
							throw Error("Failed to perform error prone operations");
						}"#,
            None,
        ),
        (
            r#"try {
					doSomething();
				} catch {
					throw new Error("Something went wrong");
				}"#,
            Some(serde_json::json!([{ "requireCatchParameter": false }])),
        ),
        (
            r#"try { doSomething(); } catch (errorA) { try { doSomethingElse(); } catch (errorB) { throw new Error( `The certificate key "${chalk.yellow(keyFile)}" is invalid.\n${errorA.message}`, { cause: errorB }); } }"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch (error) { throw new AggregateError([error], "aggregate", { cause: error }); }"#,
            None,
        ),
        (
            "try {
					return JSON.parse(text);
				} catch {
					return {};
				}",
            Some(serde_json::json!([{ "requireCatchParameter": true }])),
        ),
        (
            "try { doSomething(); } catch {}",
            Some(serde_json::json!([{ "requireCatchParameter": true }])),
        ),
        (
            r#"try { doSomething(); } catch { throw "failed"; }"#,
            Some(serde_json::json!([{ "requireCatchParameter": true }])),
        ),
        (
            r#"try {
					doSomething();
				} catch {
					foo = { bar() { throw new Error("Something went wrong"); } };
				}"#,
            Some(serde_json::json!([{ "requireCatchParameter": true }])),
        ),
        (
            r#"try { doSomething(); } catch (err) { const fail = () => { throw new Error("Something went wrong"); }; }"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch (err) { class Foo { static { throw new Error("Something went wrong"); } } }"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch { const fail = () => { throw new Error("Something went wrong"); }; }"#,
            Some(serde_json::json!([{ "requireCatchParameter": true }])),
        ),
        (
            r#"try { doSomething(); } catch { class Foo { static { throw new Error("Something went wrong"); } } }"#,
            Some(serde_json::json!([{ "requireCatchParameter": true }])),
        ),
        (
            r#"try { doSomething(); } catch (error) { throw new SyntaxError("Invalid JSON", { cause: error }); }"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch (error) { throw new EvalError("e", { cause: error }); }"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch (error) { throw new RangeError("e", { cause: error }); }"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch (error) { throw new ReferenceError("e", { cause: error }); }"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch (error) { throw new URIError("e", { cause: error }); }"#,
            None,
        ),
        (
            r#"try {
					return JSON.parse(text);
				} catch (error) {
					const options = { cause: error };
					throw new Error("Invalid JSON", options);
				}"#,
            None,
        ),
        (
            r#"try {
					doSomething();
				} catch (err) {
					const options = { cause: other };
					throw new Error("Something failed", options);
				}"#,
            None,
        ),
        (
            r#"try {
					doSomething();
				} catch (err) {
					throw new Error("Something failed", obj.options);
				}"#,
            None,
        ),
    ];

    let fail = vec![
        ("try { doSomething(); } catch (err) { throw new Error/* ( */(); }", None),
        ("try { doSomething(); } catch (err) { throw new Error<() => void>(); }", None),
        (
            r#"try {
			            doSomething();
			        } catch (err) {
			            throw new Error("Something failed");
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (err) {
			            const unrelated = new Error("other");
			            throw new Error("Something failed", { cause: unrelated });
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (err) {
			            const e = err;
			            throw new Error("Failed", { cause: e });
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (error) {
			            throw new Error("Failed", { cause: error.message });
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (error) {
			            if (shouldThrow) {
			                while (true) {
			                    if (Math.random() > 0.5) {
			                        throw new Error("Failed without cause");
			                    }
			                }
			            }
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (error) {
			            switch (error.code) {
			                case "A":
			                    throw new Error("Type A");
			                case "B":
			                    throw new Error("Type B", { cause: error });
			                default:
			                    throw new Error("Other", { cause: error });
			            }
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (error) {
			            throw new Error(`The certificate key "${chalk.yellow(keyFile)}" is invalid.
			${err.message}`);
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (error) {
			            const errorMessage = "Operation failed";
			            throw new Error(errorMessage);
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (error) {
			            const errorMessage = "Operation failed";
			            throw new Error(errorMessage, { existingOption: true, complexOption: { moreOptions: {} } });
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (err) {
			            if (err.code === "A") {
			                throw new Error("Type A");
			            }
			            throw new TypeError("Fallback error");
			        }"#,
            None,
        ),
        (
            r#"try {
			            doSomething();
			        } catch (err) {
			            throw Error("Something failed");
			        }"#,
            None,
        ),
        (
            r#"try {
			        } catch (err) {
			            my_label:
			            throw new Error("Failed without cause");
			        }"#,
            None,
        ),
        (
            r#"try {
			        } catch (err) {
			            {
			                throw new Error("Something went wrong");
			            }
			        }"#,
            None,
        ),
        (
            "try {
			        } catch (err) {
			            {
			                throw new Error();
			            }
			        }",
            None,
        ),
        (
            r#"try {
			        } catch (err) {
			            {
			                throw new AggregateError([], "Lorem ipsum");
			            }
			        }"#,
            None,
        ),
        (
            "try {
			        } catch (err) {
			            {
			                throw new AggregateError();
			            }
			        }",
            None,
        ),
        (
            "try {
			        } catch (err) {
			            {
			                throw new AggregateError([]);
			            }
			        }",
            None,
        ),
        (
            r#"try {
						doSomething();
					} catch {
						throw new Error("Something went wrong");
					}"#,
            Some(serde_json::json!([{ "requireCatchParameter": true }])),
        ),
        (
            r#"try {
			            doSomething();
			        } catch (err) {
			            throw new Error("Something failed", { cause });
			        }"#,
            None,
        ),
        (
            "try {
							doSomething();
						} catch ({ message }) {
							throw new Error(message);
						}",
            None,
        ),
        (
            "try {
							doSomethingElse();
						} catch ({ ...error }) {
							throw new Error(error.message);
						}",
            None,
        ),
        (
            r#"try {
							doSomething();
						} catch (error) {
							if (whatever) {
								const error = anotherError;
								throw new Error("Something went wrong", { cause: error });
							}
						}"#,
            None,
        ),
        (
            r#"try {
							doSomething();
						} catch (error) {
							throw new Error(
								"Something went wrong" // some comments
							);
						}"#,
            None,
        ),
        (
            r#"try {
							doSomething();
						} catch (err) {
							throw new Error("Something failed", {});
						}"#,
            None,
        ),
        (
            r#"try {
						doSomething();
					} catch (error) {
						const cause = "desc";
						throw new Error("Something failed", { [cause]: "Some error" });
					}"#,
            None,
        ),
        (
            r#"try {
						doSomething();
						} catch (error) {
						throw new Error("Something failed", { cause() { /* do something */ }  });
						}"#,
            None,
        ),
        (
            r#"try {} catch (error) {
				throw new Error("Something failed", {
					get cause() { return error; },
					set cause(value) { error = value; },
				});
			}"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch (error) { throw new AggregateError([error], "aggregate", { cause: unrelated }); }"#,
            None,
        ),
        (
            r#"try { doSomething(); } catch { if (shouldThrow) { throw new Error("Something went wrong"); } }"#,
            Some(serde_json::json!([{ "requireCatchParameter": true }])),
        ),
        (
            r#"try { doSomething(); } catch (error) { throw new SyntaxError("Invalid JSON"); }"#,
            None,
        ),
        (r#"try { doSomething(); } catch (error) { throw new EvalError("e"); }"#, None),
        (r#"try { doSomething(); } catch (error) { throw new RangeError("e"); }"#, None),
        (r#"try { doSomething(); } catch (error) { throw new ReferenceError("e"); }"#, None),
        (r#"try { doSomething(); } catch (error) { throw new URIError("e"); }"#, None),
        (
            r#"try { doSomething(); } catch (error) { throw new Error("e"); throw new EvalError("e"); throw new RangeError("e"); throw new ReferenceError("e"); throw new SyntaxError("e"); throw new URIError("e"); }"#,
            None,
        ),
        ("try { doSomething(); } catch ({ message }) { throw new Error(message, options); }", None),
        ("try { doSomething(); } catch ({ message }) { throw new Error(...args); }", None),
        (
            r#"try { doSomething(); } catch ([error]) { throw new Error("Something failed", { ...options }); }"#,
            None,
        ),
    ];

    let fix = vec![
        // the `(` inside the comment is not the start of the arguments
        (
            "try { doSomething(); } catch (err) { throw new Error/* ( */(); }",
            "try { doSomething(); } catch (err) { throw new Error/* ( */(\"\", { cause: err }); }",
            None,
        ),
        // the `(` inside the type arguments is not the start of the arguments
        (
            "try { doSomething(); } catch (err) { throw new Error<() => void>(); }",
            "try { doSomething(); } catch (err) { throw new Error<() => void>(\"\", { cause: err }); }",
            None,
        ),
        (
            r#"try {
                        doSomething();
                    } catch (err) {
                        throw new Error("Something failed");
                    }"#,
            r#"try {
                        doSomething();
                    } catch (err) {
                        throw new Error("Something failed", { cause: err });
                    }"#,
            None,
        ),
        (
            r#"try {
                        doSomething();
                    } catch (error) {
                        throw new Error("Failed");
                    }"#,
            r#"try {
                        doSomething();
                    } catch (error) {
                        throw new Error("Failed", { cause: error });
                    }"#,
            None,
        ),
        (
            r#"try {
                        doSomething();
                    } catch (error) {
                        throw new Error("Failed", {});
                    }"#,
            r#"try {
                        doSomething();
                    } catch (error) {
                        throw new Error("Failed", { cause: error });
                    }"#,
            None,
        ),
        (
            r#"try {
                        doSomething();
                    } catch (error) {
                        throw new Error("Failed", { existingOption: true, complexOption: { option: {} } });
                    }"#,
            r#"try {
                        doSomething();
                    } catch (error) {
                        throw new Error("Failed", { existingOption: true, complexOption: { option: {} }, cause: error });
                    }"#,
            None,
        ),
        (
            r#"try {
                        doSomething();
                    } catch (err) {
                        throw new Error("Something failed", {});
                    }"#,
            r#"try {
                        doSomething();
                    } catch (err) {
                        throw new Error("Something failed", { cause: err });
                    }"#,
            None,
        ),
        // Throwing a new error with unrelated cause
        (
            r#"try {
                        doSomething();
                    } catch (err) {
                        const unrelated = new Error("other");
                        throw new Error("Something failed", { cause: unrelated });
                    }"#,
            r#"try {
                        doSomething();
                    } catch (err) {
                        const unrelated = new Error("other");
                        throw new Error("Something failed", { cause: err });
                    }"#,
            None,
        ),
        // Throws a new Error, cause property is present but value is a different identifier
        (
            r#"try {
                        doSomething();
                    } catch (err) {
                        const e = err;
                        throw new Error("Something failed", { cause: e });
                    }"#,
            r#"try {
                        doSomething();
                    } catch (err) {
                        const e = err;
                        throw new Error("Something failed", { cause: err });
                    }"#,
            None,
        ),
        // Throws a new error, but is not using the full error as the cause
        (
            r#"try {
                        doSomething();
                    } catch (err) {
                        throw new Error("Something failed", { cause: err.message });
                    }"#,
            r#"try {
                        doSomething();
                    } catch (err) {
                        throw new Error("Something failed", { cause: err });
                    }"#,
            None,
        ),
        // Throw in a deeply nested catch block
        (
            r#"try {
                doSomething();
            } catch (error) {
                if (shouldThrow) {
                    while (true) {
                        if (Math.random() > 0.5) {
                            throw new Error("Failed without cause");
                        }
                    }
                }
            }"#,
            r#"try {
                doSomething();
            } catch (error) {
                if (shouldThrow) {
                    while (true) {
                        if (Math.random() > 0.5) {
                            throw new Error("Failed without cause", { cause: error });
                        }
                    }
                }
            }"#,
            None,
        ),
        // Throw deep inside a switch block
        (
            r#"try {
                doSomething();
            } catch (error) {
                switch (error.code) {
                    case "A":
                        throw new Error("Type A");
                    case "B":
                        throw new Error("Type B", { cause: error });
                    default:
                        throw new Error("Other", { cause: error });
                }
            }"#,
            r#"try {
                doSomething();
            } catch (error) {
                switch (error.code) {
                    case "A":
                        throw new Error("Type A", { cause: error });
                    case "B":
                        throw new Error("Type B", { cause: error });
                    default:
                        throw new Error("Other", { cause: error });
                }
            }"#,
            None,
        ),
        // Throw statement with template literal error message
        (
            r#"try {
                doSomething();
            } catch (error) {
                throw new Error(`The certificate key "${chalk.yellow(keyFile)}" is invalid.\n${err.message}`);
            }"#,
            r#"try {
                doSomething();
            } catch (error) {
                throw new Error(`The certificate key "${chalk.yellow(keyFile)}" is invalid.\n${err.message}`, { cause: error });
            }"#,
            None,
        ),
        // Throw statement with a variable error message
        (
            r#"try {
                doSomething();
            } catch (error) {
                const errorMessage = "Operation failed";
                throw new Error(errorMessage);
            }"#,
            r#"try {
                doSomething();
            } catch (error) {
                const errorMessage = "Operation failed";
                throw new Error(errorMessage, { cause: error });
            }"#,
            None,
        ),
        // Fixes sharing a catch binding overlap, so only the first throw is fixed per pass
        (
            r#"try {
                doSomething();
            } catch (err) {
                if (err.code === "A") {
                    throw new Error("Type A");
                }
                throw new TypeError("Fallback error");
            }"#,
            r#"try {
                doSomething();
            } catch (err) {
                if (err.code === "A") {
                    throw new Error("Type A", { cause: err });
                }
                throw new TypeError("Fallback error");
            }"#,
            None,
        ),
        // Fixes Error without new keyword
        (
            r#"try {
                doSomething();
            } catch (err) {
                throw Error("Something failed");
            }"#,
            r#"try {
                doSomething();
            } catch (err) {
                throw Error("Something failed", { cause: err });
            }"#,
            None,
        ),
        // Fixes labeled throw statement
        (
            r#"try {
                doSomething();
            } catch (err) {
                my_label:
                throw new Error("Failed without cause");
            }"#,
            r#"try {
                doSomething();
            } catch (err) {
                my_label:
                throw new Error("Failed without cause", { cause: err });
            }"#,
            None,
        ),
        // Fixes throw statement with empty error
        (
            r"try {
                doSomething();
            } catch (err) {
                {
                    throw new Error();
                }
            }",
            r#"try {
                doSomething();
            } catch (err) {
                {
                    throw new Error("", { cause: err });
                }
            }"#,
            None,
        ),
        // Fixes AggregateError, which accepts options as its third argument
        (
            r#"try {
                doSomething();
            } catch (err) {
                {
                    throw new AggregateError([], "Lorem ipsum");
                }
            }"#,
            r#"try {
                doSomething();
            } catch (err) {
                {
                    throw new AggregateError([], "Lorem ipsum", { cause: err });
                }
            }"#,
            None,
        ),
        // Fixes AggregateError with no message
        (
            r"try {
                doSomething();
            } catch (err) {
                {
                    throw new AggregateError();
                }
            }",
            r#"try {
                doSomething();
            } catch (err) {
                {
                    throw new AggregateError([], "", { cause: err });
                }
            }"#,
            None,
        ),
        // Fixes AggregateError with only an errors argument
        (
            r"try {
                doSomething();
            } catch (err) {
                {
                    throw new AggregateError([]);
                }
            }",
            r#"try {
                doSomething();
            } catch (err) {
                {
                    throw new AggregateError([], "", { cause: err });
                }
            }"#,
            None,
        ),
        // Throwing a new error with an unrelated variable named cause
        (
            r#"try {
                        doSomething();
                    } catch (err) {
                        throw new Error("Something failed", { cause });
                    }"#,
            r#"try {
                        doSomething();
                    } catch (err) {
                        throw new Error("Something failed", { cause: err });
                    }"#,
            None,
        ),
        // Make sure comments are preserved when fixing
        (
            r#"try {
				doSomething();
			} catch (error) {
				throw new Error(
					"Something went wrong" // some comments
				);
			}"#,
            r#"try {
				doSomething();
			} catch (error) {
				throw new Error(
					"Something went wrong", { cause: error } // some comments
				);
			}"#,
            None,
        ),
        // When an object property is computed, we cannot be sure it's not 'cause', so just add a new property
        (
            r#"try {
                doSomething();
            } catch (error) {
                const cause = "desc";
                throw new Error("Something failed", { [cause]: "Some error" });
            }"#,
            r#"try {
                doSomething();
            } catch (error) {
                const cause = "desc";
                throw new Error("Something failed", { [cause]: "Some error", cause: error });
            }"#,
            None,
        ),
        (
            r#"try {
                doSomething();
			} catch (error) {
                throw new Error("Something failed", { cause() { /* do something */ }  });
			}"#,
            r#"try {
                doSomething();
			} catch (error) {
                throw new Error("Something failed", { cause: error  });
			}"#,
            None,
        ),
        // Fix getters and setters as cause
        (
            r#"try {
                doSomething();
            } catch (error) {
                throw new Error("Something failed", { get cause() { } });
            }"#,
            r#"try {
                doSomething();
            } catch (error) {
                throw new Error("Something failed", { cause: error });
            }"#,
            None,
        ),
        (
            r#"try {
                doSomething();
            } catch (error) {
                throw new Error("Something failed", { set cause(value) { } });
            }"#,
            r#"try {
                doSomething();
            } catch (error) {
                throw new Error("Something failed", { cause: error });
            }"#,
            None,
        ),
        // A later pass fixes the remaining throw while preserving the same catch binding.
        (
            r#"try {} catch (err) { if (err.code === "A") { throw new Error("Type A", { cause: err }); } throw new TypeError("Fallback error"); }"#,
            r#"try {} catch (err) { if (err.code === "A") { throw new Error("Type A", { cause: err }); } throw new TypeError("Fallback error", { cause: err }); }"#,
            None,
        ),
        // Fixes built-in error constructors other than `Error`
        (
            "try { doSomething(); } catch (err) { throw new SyntaxError(\"Invalid JSON\"); }",
            "try { doSomething(); } catch (err) { throw new SyntaxError(\"Invalid JSON\", { cause: err }); }",
            None,
        ),
    ];

    Tester::new(PreserveCaughtError::NAME, PreserveCaughtError::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
