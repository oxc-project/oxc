use oxc_ast::{ast::*, match_expression};

/// This enum tracks the arguments inside a call expressions and checks if they are
/// simple or not.
///
/// The heuristic changes based on its type and the depth of the expressions. For example
/// if we have expressions as arguments, having 2 or more them tags the first argument as "not simple".
///
/// Criteria are different:
/// - *complex*: if the chain of simple arguments exceeds the depth 2 or higher
/// - *complex*: if the argument is a [RegExpLiteral] with len() greater than 5
/// - *simple*: the argument is a literal
/// - *simple*: the argument is a [RegExpLiteral] with len() less than 5
/// - *simple*: the argument is a [`ThisExpression`]
/// - *simple*: the argument is a [`IdentifierReference`]
/// - *simple*: the argument is a [`Super`]
/// - *simple*: the argument is a [`UnaryExpression`], has a trivial operator (`!`, `-`, `~`, or `+`), and the argument is simple
/// - *simple*: the argument is a [`UpdateExpression`], with a trivial operator (`++` or `--`), and the argument is simple.
/// - *simple*: the argument is a [`TSNonNullExpression`] and the argument is simple
/// - if the argument is a template literal, check [is_simple_template_literal]
/// - if the argument is an object expression, all its members are checked if they are simple or not.
/// - if the argument is an array expression, all its elements are checked if they are simple or not.
///
/// This algorithm is inspired from [Prettier].
///
/// [ThisExpression]: [ThisExpression]
/// [IdentifierReference]: [IdentifierReference]
/// [Super]: [Super]
/// [UnaryExpression]: [UnaryExpression]
/// [UpdateExpression]: [UpdateExpression]
/// [TSNonNullExpression]: [TSNonNullExpression]
/// [Prettier]: https://github.com/prettier/prettier/blob/a9de2a128cc8eea84ddd90efdc210378a894ab6b/src/language-js/utils/index.js#L802-L886
#[derive(Debug)]
pub enum SimpleArgument<'a, 'b> {
    Expression(&'b Expression<'a>),
    Assignment(&'b SimpleAssignmentTarget<'a>),
    // TODO: Not found a use case for this
    // Name(AnyJsName),
    Spread,
}

impl<'a, 'b> SimpleArgument<'a, 'b> {
    pub fn new(node: &'b Argument<'a>) -> Self {
        match node {
            Argument::SpreadElement(_) => Self::Spread,
            match_expression!(Argument) => Self::from(node.to_expression()),
        }
    }

    pub fn is_simple(&self) -> bool {
        self.is_simple_impl(0)
    }

    fn is_simple_impl(&self, depth: u8) -> bool {
        if depth >= 2 {
            return false;
        }

        match self {
            Self::Expression(expr) => match expr {
                Expression::NullLiteral(_)
                | Expression::BooleanLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::BigIntLiteral(_)
                | Expression::ThisExpression(_)
                | Expression::Identifier(_)
                | Expression::Super(_) => true,
                Expression::RegExpLiteral(regex) => regex.regex.pattern.text.len() <= 5,
                Expression::TemplateLiteral(template) => {
                    is_simple_template_literal(template, depth + 1)
                }
                Expression::TaggedTemplateExpression(template) => {
                    is_simple_template_literal(&template.quasi, depth + 1)
                }
                Expression::ObjectExpression(object) => self.is_simple_object(object, depth),
                Expression::ArrayExpression(array) => self.is_simple_array(array, depth),
                Expression::UnaryExpression(unary_expression) => {
                    matches!(
                        unary_expression.operator,
                        UnaryOperator::LogicalNot
                            | UnaryOperator::UnaryNegation
                            | UnaryOperator::UnaryPlus
                            | UnaryOperator::BitwiseNot
                    ) && Self::from(&unary_expression.argument).is_simple_impl(depth)
                }
                Expression::UpdateExpression(update) => {
                    Self::Assignment(&update.argument).is_simple_impl(depth)
                }
                Expression::TSNonNullExpression(assertion) => {
                    Self::from(&assertion.expression).is_simple_impl(depth)
                }
                Expression::StaticMemberExpression(static_expression) => {
                    Self::from(&static_expression.object).is_simple_impl(depth)
                }
                Expression::ComputedMemberExpression(computed_expression) => {
                    Self::from(&computed_expression.expression).is_simple_impl(depth)
                        && Self::from(&computed_expression.object).is_simple_impl(depth)
                }
                Expression::NewExpression(expr) => {
                    self.is_simple_call_like(&expr.callee, &expr.arguments, depth)
                }
                Expression::CallExpression(expr) => {
                    self.is_simple_call_like(&expr.callee, &expr.arguments, depth)
                }
                Expression::ImportExpression(expr) => depth < 2 && expr.options.is_none(),
                Expression::ChainExpression(chain) => {
                    self.is_simple_chain_element(&chain.expression, depth)
                }
                _ => false,
            },
            Self::Assignment(SimpleAssignmentTarget::AssignmentTargetIdentifier(_)) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_simple_object(&self, object: &'b ObjectExpression<'a>, depth: u8) -> bool {
        object.properties.iter().all(|member| {
            if let ObjectPropertyKind::ObjectProperty(property) = member {
                if property.method {
                    return false;
                }

                if property.shorthand {
                    return true;
                }

                !property.computed && Self::from(&property.value).is_simple_impl(depth + 1)
            } else {
                false
            }
        })
    }

    #[inline]
    fn is_simple_array(&self, array: &'b ArrayExpression<'a>, depth: u8) -> bool {
        array.elements.iter().all(|element| match element {
            match_expression!(ArrayExpressionElement) => {
                Self::from(element.to_expression()).is_simple_impl(depth + 1)
            }
            ArrayExpressionElement::Elision(_) => true,
            ArrayExpressionElement::SpreadElement(_) => false,
        })
    }

    #[inline]
    fn is_simple_call_like(
        &self,
        callee: &'b Expression<'a>,
        arguments: &'b [Argument<'a>],
        depth: u8,
    ) -> bool {
        Self::from(callee).is_simple_impl(depth)
            && arguments.len() + usize::from(depth) <= 2
            && arguments.iter().all(|argument| Self::new(argument).is_simple_impl(depth + 1))
    }

    #[inline]
    fn is_simple_chain_element(&self, element: &'b ChainElement<'a>, depth: u8) -> bool {
        match element {
            ChainElement::CallExpression(call) => {
                self.is_simple_call_like(&call.callee, &call.arguments, depth)
            }
            ChainElement::TSNonNullExpression(assertion) => {
                Self::from(&assertion.expression).is_simple_impl(depth)
            }
            ChainElement::StaticMemberExpression(static_expression) => {
                Self::from(&static_expression.object).is_simple_impl(depth)
            }
            ChainElement::ComputedMemberExpression(computed_expression) => {
                Self::from(&computed_expression.expression).is_simple_impl(depth)
                    && Self::from(&computed_expression.object).is_simple_impl(depth)
            }
            ChainElement::PrivateFieldExpression(private_field) => {
                Self::from(&private_field.object).is_simple_impl(depth)
            }
        }
    }
}

impl<'a, 'b> From<&'b Expression<'a>> for SimpleArgument<'a, 'b> {
    fn from(expr: &'b Expression<'a>) -> Self {
        Self::Expression(expr)
    }
}

/// A template literal is simple when:
///
/// - all strings don't contain newlines
/// - the expressions contained in the template are considered as [`Argument`]. Check
///   [`SimpleArgument`].
pub fn is_simple_template_literal(template: &TemplateLiteral<'_>, depth: u8) -> bool {
    for quasi in &template.quasis {
        if quasi.value.raw.contains('\n') {
            return false;
        }
    }

    for expr in &template.expressions {
        if !SimpleArgument::Expression(expr).is_simple_impl(depth) {
            return false;
        }
    }

    true
}
