use crate::LintContext;
use oxc_ast::ast::{ArrowFunctionExpression, Expression, Function, Statement};

/// Checks if a function call is a Higher-Order Component (HOC)
pub fn is_hoc_call(callee_name: &str, ctx: &LintContext) -> bool {
    // Check built-in HOCs
    if matches!(callee_name, "memo" | "forwardRef")
        || callee_name.ends_with("memo")
        || callee_name.ends_with("forwardRef")
    {
        return true;
    }

    // Check component wrapper functions from settings
    ctx.settings().react.is_component_wrapper_function(callee_name)
}

/// Finds the innermost function with JSX in a chain of HOC calls
#[derive(Debug)]
pub enum InnermostFunction<'a> {
    Function(&'a Function<'a>),
    ArrowFunction(&'a ArrowFunctionExpression<'a>),
}

pub fn find_innermost_function_with_jsx<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'_>,
) -> Option<InnermostFunction<'a>> {
    match expr {
        Expression::CallExpression(call) => {
            // Check if this is a HOC call
            if let Some(callee_name) = call.callee_name() {
                if is_hoc_call(callee_name, ctx) {
                    // This is a HOC, recursively check the first argument
                    if let Some(first_arg) = call.arguments.first() {
                        if let Some(inner_expr) = first_arg.as_expression() {
                            return find_innermost_function_with_jsx(inner_expr, ctx);
                        }
                    }
                }
            }
            None
        }
        Expression::FunctionExpression(func) => {
            // Check if this function contains JSX
            if crate::utils::jsx::function_contains_jsx(func) {
                Some(InnermostFunction::Function(func))
            } else {
                None
            }
        }
        Expression::ArrowFunctionExpression(arrow_func) => {
            // Check if this arrow function contains JSX
            if crate::utils::jsx::function_like_contains_jsx(expr) {
                Some(InnermostFunction::ArrowFunction(arrow_func))
            } else {
                // Check if this arrow function returns another function that contains JSX
                if arrow_func.expression {
                    // Expression-bodied arrow function: () => () => <div />
                    if arrow_func.body.statements.len() == 1 {
                        if let Statement::ExpressionStatement(expr_stmt) =
                            &arrow_func.body.statements[0]
                        {
                            return find_innermost_function_with_jsx(&expr_stmt.expression, ctx);
                        }
                    }
                } else {
                    // Block-bodied arrow function: () => { return () => <div /> }
                    for stmt in &arrow_func.body.statements {
                        if let Statement::ReturnStatement(ret_stmt) = stmt {
                            if let Some(expr) = &ret_stmt.argument {
                                return find_innermost_function_with_jsx(expr, ctx);
                            }
                        }
                    }
                }
                None
            }
        }
        _ => None,
    }
}
