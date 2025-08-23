use oxc_ast::ast::*;
use oxc_span::GetSpan;
use rustc_hash::FxHashSet;

/// Simple collector for argument spans - focuses on the most common cases
pub struct ArgumentCollector;

impl ArgumentCollector {
    pub fn collect(program: &Program) -> FxHashSet<u32> {
        let mut spans = FxHashSet::default();
        Self::collect_from_program(&mut spans, program);
        spans
    }

    fn collect_from_program(spans: &mut FxHashSet<u32>, program: &Program) {
        for stmt in &program.body {
            Self::collect_from_statement(spans, stmt);
        }
    }

    fn collect_from_statement(spans: &mut FxHashSet<u32>, stmt: &Statement) {
        match stmt {
            Statement::ExpressionStatement(expr_stmt) => {
                Self::collect_from_expression(spans, &expr_stmt.expression);
            }
            Statement::BlockStatement(block) => {
                for stmt in &block.body {
                    Self::collect_from_statement(spans, stmt);
                }
            }
            Statement::IfStatement(if_stmt) => {
                Self::collect_from_expression(spans, &if_stmt.test);
                Self::collect_from_statement(spans, &if_stmt.consequent);
                if let Some(alt) = &if_stmt.alternate {
                    Self::collect_from_statement(spans, alt);
                }
            }
            Statement::ForStatement(for_stmt) => {
                if let Some(test) = &for_stmt.test {
                    Self::collect_from_expression(spans, test);
                }
                if let Some(update) = &for_stmt.update {
                    Self::collect_from_expression(spans, update);
                }
                Self::collect_from_statement(spans, &for_stmt.body);
            }
            Statement::WhileStatement(while_stmt) => {
                Self::collect_from_expression(spans, &while_stmt.test);
                Self::collect_from_statement(spans, &while_stmt.body);
            }
            Statement::ReturnStatement(ret_stmt) => {
                if let Some(arg) = &ret_stmt.argument {
                    Self::collect_from_expression(spans, arg);
                }
            }
            Statement::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(init) = &declarator.init {
                        Self::collect_from_expression(spans, init);
                    }
                }
            }
            Statement::FunctionDeclaration(func) => {
                if let Some(body) = &func.body {
                    for stmt in &body.statements {
                        Self::collect_from_statement(spans, stmt);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_from_expression(spans: &mut FxHashSet<u32>, expr: &Expression) {
        match expr {
            Expression::CallExpression(call) => {
                // Register all argument spans - this is the key optimization
                for arg in &call.arguments {
                    spans.insert(arg.span().start);
                }
                // Recurse into callee and arguments
                Self::collect_from_expression(spans, &call.callee);
                for arg in &call.arguments {
                    Self::collect_from_argument(spans, arg);
                }
            }
            Expression::NewExpression(new_expr) => {
                // Register all argument spans
                for arg in &new_expr.arguments {
                    spans.insert(arg.span().start);
                }
                // Recurse into callee and arguments
                Self::collect_from_expression(spans, &new_expr.callee);
                for arg in &new_expr.arguments {
                    Self::collect_from_argument(spans, arg);
                }
            }
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    Self::collect_from_array_element(spans, element);
                }
            }
            Expression::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    match prop {
                        ObjectPropertyKind::ObjectProperty(prop) => {
                            Self::collect_from_expression(spans, &prop.value);
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            Self::collect_from_expression(spans, &spread.argument);
                        }
                    }
                }
            }
            Expression::ArrowFunctionExpression(arrow) => {
                Self::collect_from_arrow_body(spans, &arrow.body);
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    for stmt in &body.statements {
                        Self::collect_from_statement(spans, stmt);
                    }
                }
            }
            Expression::BinaryExpression(binary) => {
                Self::collect_from_expression(spans, &binary.left);
                Self::collect_from_expression(spans, &binary.right);
            }
            Expression::UnaryExpression(unary) => {
                Self::collect_from_expression(spans, &unary.argument);
            }
            Expression::ConditionalExpression(cond) => {
                Self::collect_from_expression(spans, &cond.test);
                Self::collect_from_expression(spans, &cond.consequent);
                Self::collect_from_expression(spans, &cond.alternate);
            }
            Expression::AssignmentExpression(assign) => {
                Self::collect_from_expression(spans, &assign.right);
            }
            Expression::LogicalExpression(logical) => {
                Self::collect_from_expression(spans, &logical.left);
                Self::collect_from_expression(spans, &logical.right);
            }
            Expression::SequenceExpression(seq) => {
                for expr in &seq.expressions {
                    Self::collect_from_expression(spans, expr);
                }
            }
            Expression::AwaitExpression(await_expr) => {
                Self::collect_from_expression(spans, &await_expr.argument);
            }
            Expression::YieldExpression(yield_expr) => {
                if let Some(arg) = &yield_expr.argument {
                    Self::collect_from_expression(spans, arg);
                }
            }
            Expression::ParenthesizedExpression(paren) => {
                Self::collect_from_expression(spans, &paren.expression);
            }
            _ => {}
        }
    }

    fn collect_from_argument(spans: &mut FxHashSet<u32>, arg: &Argument) {
        match arg {
            Argument::SpreadElement(spread) => {
                Self::collect_from_expression(spans, &spread.argument);
            }
            _ => {
                // Arguments are expressions themselves
                Self::collect_from_expression(spans, arg.to_expression());
            }
        }
    }

    fn collect_from_array_element(spans: &mut FxHashSet<u32>, element: &ArrayExpressionElement) {
        if let ArrayExpressionElement::SpreadElement(spread) = element {
            Self::collect_from_expression(spans, &spread.argument);
        } else {
            // Most array elements are expressions
            let expr = element.to_expression();
            Self::collect_from_expression(spans, expr);
        }
    }

    fn collect_from_arrow_body(spans: &mut FxHashSet<u32>, body: &FunctionBody) {
        for stmt in &body.statements {
            Self::collect_from_statement(spans, stmt);
        }
        // Arrow functions can also have expression bodies
        // but we'll handle the common case of statement bodies
    }
}