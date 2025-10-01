use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{IsGlobalReference, SymbolId};
use oxc_span::Span;

use crate::{
    AstNode, ast_util::is_method_call, context::LintContext, rule::Rule, utils::BUILT_IN_ERRORS,
};

fn no_useless_error_capture_stack_trace_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `Error.captureStackTrace(…)` in the constructor of an Error subclass")
        .with_help("The Error constructor already calls `captureStackTrace` internally, so calling it again is unnecessary.")
        .with_label(span)
}

#[derive(Debug)]
enum ClassInfo {
    None,
    Named(SymbolId),
    Anonymous,
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessErrorCaptureStackTrace;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows unnecessary `Error.captureStackTrace(…)` in error constructors.
    ///
    /// ### Why is this bad?
    ///
    /// Calling `Error.captureStackTrace(…)` inside the constructor of a built-in `Error` subclass
    /// is unnecessary, since the `Error` constructor calls it automatically.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class MyError extends Error {
    ///     constructor() {
    ///         Error.captureStackTrace(this, MyError);
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class MyError extends Error {
    ///     constructor() {
    ///         // No need to call Error.captureStackTrace
    ///     }
    /// }
    /// ```
    NoUselessErrorCaptureStackTrace,
    unicorn,
    restriction,
    pending
);

impl Rule for NoUselessErrorCaptureStackTrace {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, Some(&["Error"]), Some(&["captureStackTrace"]), None, None) {
            return;
        }

        if let Some(member) = call_expr.callee.as_member_expression()
            && let Expression::Identifier(error_ident) = member.object()
            && !error_ident.is_global_reference(ctx.scoping())
        {
            return;
        }

        match get_error_subclass_if_in_constructor(node, ctx) {
            ClassInfo::None => return,
            ClassInfo::Named(class_id) => {
                if !is_referencing_class(call_expr, Some(class_id), ctx) {
                    return;
                }
            }
            ClassInfo::Anonymous => {
                if !is_referencing_class(call_expr, None, ctx) {
                    return;
                }
            }
        }

        ctx.diagnostic(no_useless_error_capture_stack_trace_diagnostic(call_expr.span));
    }
}

fn get_error_subclass_if_in_constructor<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> ClassInfo {
    let mut found_constructor = false;

    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::StaticBlock(_) => return ClassInfo::None,
            AstKind::Function(func) if func.id.is_some() => return ClassInfo::None,
            AstKind::MethodDefinition(method) if method.kind.is_constructor() => {
                found_constructor = true;
            }
            AstKind::Class(class) => {
                if found_constructor && is_error_class(class, ctx) {
                    if let Some(id) = class.id.as_ref()
                        && let Some(symbol_id) = id.symbol_id.get()
                    {
                        return ClassInfo::Named(symbol_id);
                    }
                    return ClassInfo::Anonymous;
                }
                break;
            }
            _ => {}
        }
    }

    ClassInfo::None
}

fn is_error_class(class: &oxc_ast::ast::Class, ctx: &LintContext) -> bool {
    // Check if the class extends a built-in Error type
    let Some(super_class) = &class.super_class else {
        return false;
    };

    // Check if super_class is one of the built-in error types
    if let Expression::Identifier(ident) = super_class.get_inner_expression() {
        // Check that the name is a built-in error type
        if !BUILT_IN_ERRORS.contains(&ident.name.as_str()) {
            return false;
        }

        if !ident.is_global_reference(ctx.scoping()) {
            return false;
        }

        return true;
    }

    false
}

fn is_referencing_class(
    call_expr: &CallExpression,
    class_id: Option<SymbolId>,
    ctx: &LintContext,
) -> bool {
    let Some(expr) = call_expr
        .arguments
        .get(1)
        .and_then(|arg| arg.as_expression())
        .map(Expression::get_inner_expression)
    else {
        return false;
    };

    match expr {
        Expression::Identifier(ident) => {
            if let Some(expected_symbol) = class_id
                && let Some(reference_id) = ident.reference_id.get()
                && let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id()
            {
                return symbol_id == expected_symbol;
            }

            false
        }
        Expression::MetaProperty(meta)
            if meta.meta.name == "new" && meta.property.name == "target" =>
        {
            true
        }
        _ => {
            if let Some(member) = expr.as_member_expression()
                && let Expression::ThisExpression(_) = member.object().get_inner_expression()
                && let Some(prop_name) = member.static_property_name()
                && prop_name == "constructor"
            {
                return true;
            }

            false
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class MyError {constructor() {Error.captureStackTrace(this, MyError)}}",
        "class MyError extends NotABuiltinError {constructor() {Error.captureStackTrace(this, MyError)}}",
        "class MyError extends Error {
				notConstructor() {
					Error.captureStackTrace(this, MyError)
				}
			}",
        "class MyError extends Error {
				constructor() {
					function foo() {
						Error.captureStackTrace(this, MyError)
					}
				}
			}",
        "class MyError extends Error {
				constructor(MyError) {
					Error.captureStackTrace(this, MyError)
				}
			}",
        "class MyError extends Error {
				static {
					Error.captureStackTrace(this, MyError)
					function foo() {
						Error.captureStackTrace(this, MyError)
					}
				}
			}",
        "class MyError extends Error {
				constructor() {
					class NotAErrorSubclass {
						constructor() {
							Error.captureStackTrace(this, new.target)
						}
					}
				}
			}",
        "class Error {}
			class MyError extends Error {
				constructor() {
					Error.captureStackTrace(this, MyError)
				}
			}",
        "class Error {}
			class MyError extends RangeError {
				constructor() {
					Error.captureStackTrace(this, MyError)
				}
			}",
        "class MyError extends Error {
				constructor(): void;
				static {
					Error.captureStackTrace(this, MyError)
					function foo() {
						Error.captureStackTrace(this, MyError)
					}
				}
			}",
    ];

    let fail = vec![
        "class MyError extends Error {
				constructor() {
					const foo = () => {
						Error.captureStackTrace(this, MyError)
					}
				}
			}",
        "class MyError extends Error {
				constructor() {
					if (a) Error.captureStackTrace(this, MyError)
				}
			}",
        "class MyError extends Error {
				constructor() {
					const x = () => Error.captureStackTrace(this, MyError)
				}
			}",
        "class MyError extends Error {
				constructor() {
					void Error.captureStackTrace(this, MyError)
				}
			}",
        "export default class extends Error {
				constructor() {
					Error.captureStackTrace(this, new.target)
				}
			}",
        "export default (
				class extends Error {
					constructor() {
						Error.captureStackTrace(this, new.target)
					}
				}
			)",
    ];

    Tester::new(
        NoUselessErrorCaptureStackTrace::NAME,
        NoUselessErrorCaptureStackTrace::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
