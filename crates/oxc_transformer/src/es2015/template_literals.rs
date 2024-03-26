use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, Span, SPAN};
use std::{mem, rc::Rc};

use crate::{context::TransformerCtx, TransformTarget};

/// ES2015: Template Literals
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-template-literals>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-template-literals>
pub struct TemplateLiterals<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> TemplateLiterals<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        (ctx.options.target < TransformTarget::ES2015 || ctx.options.template_literals)
            .then_some(Self { ast: ctx.ast })
    }

    pub fn transform_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        #[allow(clippy::single_match)]
        match expr {
            Expression::TemplateLiteral(template_literal) => {
                let quasis = mem::replace(&mut template_literal.quasis, self.ast.new_vec());
                let mut nodes = self.ast.new_vec_with_capacity(quasis.len());
                let mut expr_iter = template_literal.expressions.iter_mut();
                for quasi in quasis {
                    if let Some(cooked) = &quasi.value.cooked {
                        if cooked.as_str() != "" {
                            let string_literal = StringLiteral::new(SPAN, cooked.clone());
                            let string_literal = self.ast.literal_string_expression(string_literal);
                            nodes.push(string_literal);
                        }
                    }

                    if let Some(expr) = expr_iter.next() {
                        let expr = self.ast.move_expression(expr);
                        if is_non_empty_string_literal(&expr) {
                            nodes.push(expr);
                        }
                    }
                }

                // make sure the first node is a string
                if !matches!(nodes.first(), Some(Expression::StringLiteral(_))) {
                    let literal = StringLiteral::new(SPAN, Atom::from(""));
                    let string_literal = self.ast.literal_string_expression(literal);
                    nodes.insert(0, string_literal);
                }

                if let Some(call_expr) = build_concat_call_expr(nodes, &Rc::clone(&self.ast)) {
                    *expr = call_expr;
                }
            }
            // TODO: Expression::TaggedTemplateExpression
            _ => {}
        }
    }
}

/// This function groups the objects into multiple calls to `.concat()` in
///    order to preserve execution order of the primitive conversion
///
/// ```javascript
///      "".concat(obj.foo, "foo", obj2.foo, "foo2")
/// ```
///    The above code would evaluate both member expressions _first_ then, `concat` will
///    convert each one to a primitive, whereas
///
/// ```javascript
///      "".concat(obj.foo, "foo").concat(obj2.foo, "foo2")
/// ```
///    would evaluate the member, then convert it to a primitive, then evaluate
///    the second member and convert that one, which reflects the spec behavior
///    of template literals.
fn build_concat_call_expr<'a>(
    nodes: Vec<Expression<'a>>,
    ast: &Rc<AstBuilder<'a>>,
) -> Option<Expression<'a>> {
    // `1${"2"}${"3"}${a}${b}${"4"}${"5"}${c}` -> 1".concat("2", "3", a).concat(b, "4", "5").concat(c)
    let mut avail = false;
    nodes.into_iter().reduce(|mut left, right| {
        let mut can_be_inserted = matches!(right, Expression::StringLiteral(_));

        // for spec compatibility, we shouldn't keep two or more non-string node in one concat call.
        // but we want group multiple node in one concat call as much as possible
        // only the first encounter of non-string node can be inserted directly in the previous concat call
        // other concat call will contains non-string node already.
        if !can_be_inserted && avail {
            can_be_inserted = true;
            avail = false;
        }
        if can_be_inserted {
            if let Expression::CallExpression(call_expr) = &mut left {
                let argument = Argument::Expression(right);
                call_expr.arguments.push(argument);
                return left;
            }
        }

        let property = IdentifierName::new(Span::default(), "concat".into());
        let member_expr = ast.static_member_expression(Span::default(), left, property, false);
        let arguments = ast.new_vec_single(Argument::Expression(right));
        let call_expr = ast.call_expression(Span::default(), member_expr, arguments, false, None);
        call_expr
    })
}

fn is_non_empty_string_literal(expr: &Expression) -> bool {
    if let Expression::StringLiteral(string_literal) = expr {
        return string_literal.value.as_str() != "";
    }

    true
}
