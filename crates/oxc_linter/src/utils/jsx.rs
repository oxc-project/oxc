use oxc_ast::ast::{Expression, Function, Statement};

/// Checks if an expression contains JSX elements
pub fn contains_jsx(expr: &Expression) -> bool {
    match expr {
        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
        Expression::CallExpression(call) => {
            if crate::utils::is_create_element_call(call) {
                return true;
            }
            call.arguments.iter().any(|arg| arg.as_expression().is_some_and(contains_jsx))
        }
        Expression::ParenthesizedExpression(inner) => contains_jsx(&inner.expression),
        Expression::StaticMemberExpression(member) => contains_jsx(&member.object),
        Expression::ConditionalExpression(cond) => {
            contains_jsx(&cond.consequent) || contains_jsx(&cond.alternate)
        }
        Expression::LogicalExpression(logical) => {
            contains_jsx(&logical.left) || contains_jsx(&logical.right)
        }
        Expression::SequenceExpression(seq) => seq.expressions.iter().any(contains_jsx),
        _ => false,
    }
}

/// Checks if a function contains JSX in its return statements
pub fn function_contains_jsx(func: &Function) -> bool {
    if let Some(body) = &func.body {
        for stmt in &body.statements {
            if let Statement::ReturnStatement(ret_stmt) = stmt {
                if let Some(expr) = &ret_stmt.argument {
                    if contains_jsx(expr) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Checks if a function-like expression (function or arrow function) contains JSX
pub fn function_like_contains_jsx(expr: &Expression) -> bool {
    match expr {
        Expression::FunctionExpression(func) => function_contains_jsx(func),
        Expression::ArrowFunctionExpression(arrow_func) => {
            if arrow_func.expression {
                // Expression-bodied arrow function: () => <div />
                if arrow_func.body.statements.len() == 1 {
                    if let Statement::ExpressionStatement(expr_stmt) =
                        &arrow_func.body.statements[0]
                    {
                        return contains_jsx(&expr_stmt.expression);
                    }
                }
            } else {
                // Block-bodied arrow function: () => { return <div /> }
                for stmt in &arrow_func.body.statements {
                    if let Statement::ReturnStatement(ret_stmt) = stmt {
                        if let Some(expr) = &ret_stmt.argument {
                            if contains_jsx(expr) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}
