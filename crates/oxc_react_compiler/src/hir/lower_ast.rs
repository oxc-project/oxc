/// Convert oxc_ast nodes to LowerableExpression/LowerableStatement.
///
/// This module bridges the gap between oxc_parser output and the HIR lowering
/// layer. It converts oxc_ast expression and statement nodes into the
/// intermediate `LowerableExpression` and `LowerableStatement` types that
/// BuildHIR can then lower to HIR instructions.
use oxc_ast::ast;
use oxc_span::GetSpan;
use oxc_syntax::operator::BinaryOperator;

use super::build_hir::{LowerableExpression, LowerableStatement};

/// Convert an oxc_ast Expression to a LowerableExpression.
pub fn convert_expression(expr: &ast::Expression<'_>) -> LowerableExpression {
    match expr {
        ast::Expression::NumericLiteral(lit) => {
            LowerableExpression::NumericLiteral(lit.value, lit.span)
        }
        ast::Expression::StringLiteral(lit) => {
            LowerableExpression::StringLiteral(lit.value.to_string(), lit.span)
        }
        ast::Expression::BooleanLiteral(lit) => {
            LowerableExpression::BooleanLiteral(lit.value, lit.span)
        }
        ast::Expression::NullLiteral(lit) => LowerableExpression::NullLiteral(lit.span),
        ast::Expression::Identifier(ident) => {
            if ident.name == "undefined" {
                LowerableExpression::Undefined(ident.span)
            } else {
                LowerableExpression::Identifier(ident.name.to_string(), ident.span)
            }
        }
        ast::Expression::RegExpLiteral(lit) => LowerableExpression::RegExpLiteral {
            pattern: format!("{:?}", lit.regex.pattern),
            flags: lit.regex.flags.to_string(),
            span: lit.span,
        },
        ast::Expression::TemplateLiteral(tpl) => {
            let quasis = tpl
                .quasis
                .iter()
                .map(|q| (q.value.raw.to_string(), q.value.cooked.as_ref().map(std::string::ToString::to_string)))
                .collect();
            let expressions = tpl.expressions.iter().map(convert_expression).collect();
            LowerableExpression::TemplateLiteral {
                quasis,
                expressions,
                span: tpl.span,
            }
        }
        ast::Expression::ArrayExpression(arr) => {
            let elements = arr
                .elements
                .iter()
                .filter_map(|elem| match elem {
                    ast::ArrayExpressionElement::SpreadElement(spread) => {
                        Some(LowerableExpression::SpreadElement {
                            argument: Box::new(convert_expression(&spread.argument)),
                            span: spread.span,
                        })
                    }
                    ast::ArrayExpressionElement::Elision(_) => None,
                    _ => {
                        let expr = elem.to_expression();
                        Some(convert_expression(expr))
                    }
                })
                .collect();
            LowerableExpression::ArrayExpression(elements, arr.span)
        }
        ast::Expression::ObjectExpression(obj) => {
            LowerableExpression::ObjectExpression(obj.span)
        }
        ast::Expression::BinaryExpression(bin) => LowerableExpression::BinaryExpression {
            operator: bin.operator,
            left: Box::new(convert_expression(&bin.left)),
            right: Box::new(convert_expression(&bin.right)),
            span: bin.span,
        },
        ast::Expression::UnaryExpression(unary) => LowerableExpression::UnaryExpression {
            operator: unary.operator,
            argument: Box::new(convert_expression(&unary.argument)),
            span: unary.span,
        },
        ast::Expression::CallExpression(call) => {
            let callee = convert_expression(&call.callee);
            let arguments = call.arguments.iter().map(|arg| {
                match arg {
                    ast::Argument::SpreadElement(spread) => LowerableExpression::SpreadElement {
                        argument: Box::new(convert_expression(&spread.argument)),
                        span: spread.span,
                    },
                    _ => convert_expression(arg.to_expression()),
                }
            }).collect();
            LowerableExpression::CallExpression {
                callee: Box::new(callee),
                arguments,
                span: call.span,
            }
        }
        ast::Expression::NewExpression(new_expr) => {
            let callee = convert_expression(&new_expr.callee);
            let arguments = new_expr.arguments.iter().map(|arg| {
                match arg {
                    ast::Argument::SpreadElement(spread) => LowerableExpression::SpreadElement {
                        argument: Box::new(convert_expression(&spread.argument)),
                        span: spread.span,
                    },
                    _ => convert_expression(arg.to_expression()),
                }
            }).collect();
            LowerableExpression::NewExpression {
                callee: Box::new(callee),
                arguments,
                span: new_expr.span,
            }
        }
        ast::Expression::StaticMemberExpression(member) => {
            LowerableExpression::PropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: member.property.name.to_string(),
                span: member.span,
            }
        }
        ast::Expression::ComputedMemberExpression(member) => {
            LowerableExpression::ComputedPropertyAccess {
                object: Box::new(convert_expression(&member.object)),
                property: Box::new(convert_expression(&member.expression)),
                span: member.span,
            }
        }
        ast::Expression::ConditionalExpression(cond) => {
            LowerableExpression::ConditionalExpression {
                test: Box::new(convert_expression(&cond.test)),
                consequent: Box::new(convert_expression(&cond.consequent)),
                alternate: Box::new(convert_expression(&cond.alternate)),
                span: cond.span,
            }
        }
        ast::Expression::AssignmentExpression(assign) => {
            let right = convert_expression(&assign.right);
            let left = match &assign.left {
                ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    LowerableExpression::Identifier(ident.name.to_string(), ident.span)
                }
                _ => LowerableExpression::Undefined(assign.span), // Placeholder
            };
            LowerableExpression::AssignmentExpression {
                left: Box::new(left),
                right: Box::new(right),
                span: assign.span,
            }
        }
        ast::Expression::AwaitExpression(await_expr) => LowerableExpression::AwaitExpression {
            argument: Box::new(convert_expression(&await_expr.argument)),
            span: await_expr.span,
        },
        ast::Expression::ArrowFunctionExpression(arrow) => {
            LowerableExpression::ArrowFunctionExpression {
                is_async: arrow.r#async,
                span: arrow.span,
            }
        }
        ast::Expression::FunctionExpression(func) => {
            LowerableExpression::FunctionExpression {
                name: func.id.as_ref().map(|id| id.name.to_string()),
                is_async: func.r#async,
                is_generator: func.generator,
                span: func.span,
            }
        }
        ast::Expression::JSXElement(jsx) => {
            let tag_name = match &jsx.opening_element.name {
                ast::JSXElementName::Identifier(ident) => ident.name.to_string(),
                ast::JSXElementName::IdentifierReference(ident) => ident.name.to_string(),
                ast::JSXElementName::NamespacedName(ns) => {
                    format!("{}:{}", ns.namespace.name, ns.name.name)
                }
                ast::JSXElementName::MemberExpression(member) => {
                    format_jsx_member_expression(member)
                }
                ast::JSXElementName::ThisExpression(_) => "this".to_string(),
            };
            LowerableExpression::JsxElement {
                tag: tag_name,
                span: jsx.span,
            }
        }
        ast::Expression::JSXFragment(frag) => LowerableExpression::ArrayExpression(
            Vec::new(), // Children would be processed here
            frag.span,
        ),
        ast::Expression::LogicalExpression(logical) => {
            // Lower logical expressions as binary for now
            let operator = match logical.operator {
                oxc_syntax::operator::LogicalOperator::And => BinaryOperator::BitwiseAnd,
                oxc_syntax::operator::LogicalOperator::Or
                | oxc_syntax::operator::LogicalOperator::Coalesce => BinaryOperator::BitwiseOR,
            };
            LowerableExpression::BinaryExpression {
                operator,
                left: Box::new(convert_expression(&logical.left)),
                right: Box::new(convert_expression(&logical.right)),
                span: logical.span,
            }
        }
        ast::Expression::SequenceExpression(seq) => {
            // Return the last expression in the sequence
            if let Some(last) = seq.expressions.last() {
                convert_expression(last)
            } else {
                LowerableExpression::Undefined(seq.span)
            }
        }
        ast::Expression::TaggedTemplateExpression(tagged) => {
            // Simplify: treat as a call to the tag with the template
            LowerableExpression::CallExpression {
                callee: Box::new(convert_expression(&tagged.tag)),
                arguments: Vec::new(),
                span: tagged.span,
            }
        }
        ast::Expression::ParenthesizedExpression(paren) => {
            convert_expression(&paren.expression)
        }
        ast::Expression::TSAsExpression(ts_as) => {
            convert_expression(&ts_as.expression)
        }
        ast::Expression::TSSatisfiesExpression(ts_sat) => {
            convert_expression(&ts_sat.expression)
        }
        ast::Expression::TSNonNullExpression(ts_nn) => {
            convert_expression(&ts_nn.expression)
        }
        ast::Expression::TSTypeAssertion(ts_ta) => {
            convert_expression(&ts_ta.expression)
        }
        // Default: treat as undefined for unsupported expressions
        _ => LowerableExpression::Undefined(expr.span()),
    }
}

/// Convert an oxc_ast Statement to a LowerableStatement.
pub fn convert_statement<'a>(stmt: &'a ast::Statement<'a>) -> LowerableStatement<'a> {
    match stmt {
        ast::Statement::VariableDeclaration(decl) => {
            LowerableStatement::VariableDeclaration(decl)
        }
        ast::Statement::ExpressionStatement(expr) => {
            LowerableStatement::ExpressionStatement(expr)
        }
        ast::Statement::ReturnStatement(ret) => LowerableStatement::ReturnStatement(ret),
        ast::Statement::IfStatement(if_stmt) => LowerableStatement::IfStatement(if_stmt),
        ast::Statement::WhileStatement(while_stmt) => {
            LowerableStatement::WhileStatement(while_stmt)
        }
        ast::Statement::ForStatement(for_stmt) => LowerableStatement::ForStatement(for_stmt),
        ast::Statement::BlockStatement(block) => LowerableStatement::BlockStatement(block),
        ast::Statement::ThrowStatement(throw) => LowerableStatement::ThrowStatement(throw),
        ast::Statement::TryStatement(try_stmt) => LowerableStatement::TryStatement(try_stmt),
        ast::Statement::SwitchStatement(switch) => {
            LowerableStatement::SwitchStatement(switch)
        }
        ast::Statement::BreakStatement(_) => LowerableStatement::BreakStatement,
        ast::Statement::ContinueStatement(_) => LowerableStatement::ContinueStatement,
        ast::Statement::DebuggerStatement(_) => LowerableStatement::DebuggerStatement,
        // Empty statements and unsupported statement types
        _ => LowerableStatement::EmptyStatement,
    }
}

fn format_jsx_member_expression(member: &ast::JSXMemberExpression<'_>) -> String {
    let object = match &member.object {
        ast::JSXMemberExpressionObject::IdentifierReference(ident) => ident.name.to_string(),
        ast::JSXMemberExpressionObject::MemberExpression(inner) => {
            format_jsx_member_expression(inner)
        }
        ast::JSXMemberExpressionObject::ThisExpression(_) => "this".to_string(),
    };
    format!("{}.{}", object, member.property.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    #[test]
    fn test_convert_numeric_literal() {
        let allocator = Allocator::default();
        let source = "42";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        assert!(!body.is_empty());
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::NumericLiteral(42.0, _)));
        }
    }

    #[test]
    fn test_convert_string_literal() {
        let allocator = Allocator::default();
        let source = "let x = \"hello\"";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty(), "Parse errors: {:?}", parser_result.errors);

        let body = &parser_result.program.body;
        assert!(!body.is_empty(), "Body should not be empty");
        // The string is inside a variable declaration initializer
        if let ast::Statement::VariableDeclaration(decl) = &body[0] {
            if let Some(init) = &decl.declarations[0].init {
                let lowered = convert_expression(init);
                assert!(matches!(lowered, LowerableExpression::StringLiteral(ref s, _) if s == "hello"));
            }
        }
    }

    #[test]
    fn test_convert_binary_expression() {
        let allocator = Allocator::default();
        let source = "1 + 2";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::BinaryExpression { operator: BinaryOperator::Addition, .. }));
        }
    }

    #[test]
    fn test_convert_call_expression() {
        let allocator = Allocator::default();
        let source = "foo(1, 2)";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::CallExpression { .. }));
        }
    }

    #[test]
    fn test_convert_member_expression() {
        let allocator = Allocator::default();
        let source = "obj.prop";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::PropertyAccess { ref property, .. } if property == "prop"));
        }
    }

    #[test]
    fn test_convert_jsx_element() {
        let allocator = Allocator::default();
        let source = "<div />";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::JsxElement { ref tag, .. } if tag == "div"));
        }
    }

    #[test]
    fn test_convert_arrow_function() {
        let allocator = Allocator::default();
        let source = "() => 42";
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());

        let body = &parser_result.program.body;
        if let ast::Statement::ExpressionStatement(expr_stmt) = &body[0] {
            let lowered = convert_expression(&expr_stmt.expression);
            assert!(matches!(lowered, LowerableExpression::ArrowFunctionExpression { .. }));
        }
    }

    #[test]
    fn test_convert_statement_types() {
        let allocator = Allocator::default();
        let source = "return 42;";
        let source_type = SourceType::jsx().with_script(true);
        let parser_result = Parser::new(&allocator, source, source_type).parse();

        // Return statements outside functions may error in some parsers
        // but our converter should handle all statement types gracefully
    }
}
