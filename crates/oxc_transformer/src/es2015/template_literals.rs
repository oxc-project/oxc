use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, Span};

use std::rc::Rc;

/// ES2015: Template Literals
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-template-literals>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-template-literals>
pub struct TemplateLiteral<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> TemplateLiteral<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>) -> Self {
        Self { ast }
    }

    pub fn transform_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        match expr {
            Expression::TemplateLiteral(template_literal) => {
                let mut nodes = Vec::new();
                let mut expr_iter = template_literal.expressions.iter_mut();
                for quasis in &template_literal.quasis {
                    if let Some(cooked) = &quasis.value.cooked {
                        if cooked.as_str() != "" {
                            let string_literal =
                                self.ast.string_literal(Span::default(), cooked.clone());
                            let string_literal = self.ast.literal_string_expression(string_literal);
                            nodes.push(string_literal);
                        }
                    }

                    if let Some(expr) = expr_iter.next() {
                        let expr = self.ast.move_expression(expr);
                        if is_non_empty_string_literal(&expr) {
                            nodes.push(expr)
                        }
                    }
                }

                // make sure the first node is a string
                if !matches!(nodes.get(0), Some(Expression::StringLiteral(_))) {
                    let string_literal = self.ast.string_literal(Span::default(), Atom::from(""));
                    let string_literal = self.ast.literal_string_expression(string_literal);
                    nodes.insert(0, string_literal);
                }

                *expr = build_concat_call_expr(nodes)
            }
            Expression::TaggedTemplateExpression(_) => {
                // TODO
            }
            _ => {}
        }
    }
}

fn build_concat_call_expr(nodes: Vec<Expression>) -> Expression {
    todo!();
}

fn is_non_empty_string_literal(expr: &Expression) -> bool {
    if let Expression::StringLiteral(string_literal) = expr {
        if string_literal.value.as_str() != "" {
            return true;
        }
    }

    return false;
}
