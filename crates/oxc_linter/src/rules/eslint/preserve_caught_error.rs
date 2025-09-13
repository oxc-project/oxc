use oxc_ast::AstKind;
use oxc_ast::ast::{
    Argument, BindingPattern, CatchClause, Expression, ObjectExpression, ObjectPropertyKind,
    PropertyKey, Statement, ThrowStatement, TryStatement,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AstNode, context::LintContext, rule::Rule};

fn preserve_caught_error_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("preserve-caught-error")
        .with_help(
            "When re-throwing an error, preserve the original error using the 'cause' property",
        )
        .with_label(span)
}

fn missing_catch_parameter_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("preserve-caught-error")
        .with_help("Catch clause must have a parameter when 'requireCatchParameter' is enabled")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct ConfigElement0 {
    require_catch_parameter: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PreserveCaughtError(ConfigElement0);

// Helper functions - global scope for better reusability and testing

// Recursively check statements for throw expressions
fn check_statement<'a>(
    stmt: &Statement<'a>,
    catch_param: &BindingPattern<'a>,
    ctx: &LintContext<'a>,
) {
    match stmt {
        Statement::ThrowStatement(throw_stmt) => {
            check_throw_statement(throw_stmt, catch_param, ctx);
        }
        Statement::IfStatement(if_stmt) => {
            check_statement(&if_stmt.consequent, catch_param, ctx);
            if let Some(alternate) = &if_stmt.alternate {
                check_statement(alternate, catch_param, ctx);
            }
        }
        Statement::WhileStatement(while_stmt) => {
            check_statement(&while_stmt.body, catch_param, ctx);
        }
        Statement::ForStatement(for_stmt) => {
            check_statement(&for_stmt.body, catch_param, ctx);
        }
        Statement::BlockStatement(block_stmt) => {
            for stmt in &block_stmt.body {
                check_statement(stmt, catch_param, ctx);
            }
        }
        Statement::SwitchStatement(switch_stmt) => {
            for case in &switch_stmt.cases {
                for stmt in &case.consequent {
                    check_statement(stmt, catch_param, ctx);
                }
            }
        }
        Statement::LabeledStatement(label_stmt) => {
            check_statement(&label_stmt.body, catch_param, ctx);
        }
        _ => {}
    }
}

// Check if a throw statement re-throws built-in errors without preserving the original
fn check_throw_statement<'a>(
    throw_stmt: &ThrowStatement<'a>,
    catch_param: &BindingPattern<'a>,
    ctx: &LintContext<'a>,
) {
    let (callee, args) = match &throw_stmt.argument {
        Expression::NewExpression(new_expr) => (&new_expr.callee, &new_expr.arguments),
        Expression::CallExpression(call_expr) => (&call_expr.callee, &call_expr.arguments),
        _ => return,
    };

    if !is_builtin_error_constructor(callee, ctx) {
        return;
    }

    // Check if second argument has proper cause property
    if let Some(Argument::ObjectExpression(obj_expr)) = args.get(1) {
        if has_cause_property(obj_expr, catch_param, ctx) {
            return;
        }
    }

    // Allow spread arguments - they may contain cause property
    if args.iter().any(|arg| matches!(arg, Argument::SpreadElement(_))) {
        return;
    }

    ctx.diagnostic(preserve_caught_error_diagnostic(throw_stmt.span));
}

// Check if expression is a built-in Error constructor
fn is_builtin_error_constructor(expr: &Expression, ctx: &LintContext) -> bool {
    let Expression::Identifier(ident) = expr else {
        return false;
    };

    ident.is_global_reference_name("Error", ctx.scoping())
        || ident.is_global_reference_name("TypeError", ctx.scoping())
        || ident.is_global_reference_name("AggregateError", ctx.scoping())
}

// Check if object expression has a 'cause' property with the catch parameter
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
            // Spread properties might contain cause
            ObjectPropertyKind::SpreadProperty(_) => return true,
        }
    }
    false
}

// Check if expression references the catch parameter
fn is_catch_parameter(expr: &Expression, catch_param: &BindingPattern, ctx: &LintContext) -> bool {
    // Only handle simple identifier catch parameters for now
    let oxc_ast::ast::BindingPatternKind::BindingIdentifier(binding) = &catch_param.kind else {
        return false; // Destructuring patterns not supported
    };

    let Some(catch_symbol_id) = binding.symbol_id.get() else {
        return false;
    };

    let Expression::Identifier(ident) = expr else {
        return false;
    };

    let Some(reference_id) = ident.reference_id.get() else {
        return false;
    };

    let reference = ctx.scoping().get_reference(reference_id);
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
    correctness,
    pending,
    config = ConfigElement0,
);
impl PreserveCaughtError {
    fn check_try_statement<'a>(&self, try_stmt: &TryStatement<'a>, ctx: &LintContext<'a>) {
        if let Some(catch_clause) = &try_stmt.handler {
            self.check_catch_clause(catch_clause, ctx);
        }
    }

    fn check_catch_clause<'a>(&self, catch_clause: &CatchClause<'a>, ctx: &LintContext<'a>) {
        if let Some(catch_param) = &catch_clause.param {
            for stmt in &catch_clause.body.body {
                check_statement(stmt, &catch_param.pattern, ctx);
            }
        } else if self.0.require_catch_parameter {
            ctx.diagnostic(missing_catch_parameter_diagnostic(catch_clause.span));
        }
    }
}

impl Rule for PreserveCaughtError {
    fn from_configuration(value: serde_json::Value) -> Self {
        if value.is_null() {
            return Self::default();
        }

        let Some(config_array) = value.as_array() else {
            return serde_json::from_value(value).unwrap_or_default();
        };

        let Some(config_obj) = config_array.first() else {
            return Self::default();
        };

        serde_json::from_value(config_obj.clone()).unwrap_or_default()
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
    ];

    let fail = vec![
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
    ];

    Tester::new(PreserveCaughtError::NAME, PreserveCaughtError::PLUGIN, pass, fail)
        .test_and_snapshot();
}
