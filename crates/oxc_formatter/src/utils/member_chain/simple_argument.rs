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
/// - if the argument is an object expression, all its members are checked if they are simple or not. Check [`SimpleArgument::is_simple_object_expression`]
/// - if the argument is an array expression, all its elements are checked if they are simple or not. Check [`SimpleArgument::is_simple_array_expression`]
///
/// This algorithm is inspired from [Prettier].
///
/// [ThisExpression]: [ThisExpression]
/// [IdentifierReference]: [IdentifierReference]
/// [Super]: [Super]
/// [SimpleArgument::is_simple_object_expression]: [SimpleArgument::is_simple_object_expression]
/// [SimpleArgument::is_simple_array_expression]: [SimpleArgument::is_simple_array_expression]
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

        if self.is_simple_literal() {
            return true;
        }

        self.is_simple_template(depth)
            || self.is_simple_object_expression(depth)
            || self.is_simple_array_expression(depth)
            || self.is_simple_unary_expression(depth)
            || self.is_simple_update_expression(depth)
            || self.is_simple_non_null_assertion_expression(depth)
            || self.is_simple_member_expression(depth)
            || self.is_simple_call_like_expression(depth)
            || self.is_simple_regex_expression()
    }

    fn is_simple_call_like_expression(&self, depth: u8) -> bool {
        if let Self::Expression(any_expression) = self {
            if any_expression.is_call_like_expression() {
                let mut is_import_call_expression = false;
                let mut is_simple_callee = false;
                let arguments = match any_expression {
                    Expression::NewExpression(expr) => {
                        let callee = &expr.callee;
                        is_simple_callee = Self::from(callee).is_simple_impl(depth);
                        &expr.arguments
                    }
                    Expression::CallExpression(expr) => {
                        let callee = &expr.callee;
                        is_simple_callee = Self::from(callee).is_simple_impl(depth);
                        &expr.arguments
                    }
                    // Expression::ImportExpression(expr) => {
                    //     is_import_call_expression = true;
                    //     expr.arguments
                    // }
                    _ => unreachable!("The check is done inside `is_call_like_expression`"),
                };

                if !is_import_call_expression && !is_simple_callee {
                    return false;
                }

                // This is a little awkward, but because we _increment_
                // depth, we need to add it to the left and compare to the
                // max we allow (2), versus just comparing `len <= depth`.
                arguments.len() + usize::from(depth) <= 2
                    && arguments
                        .iter()
                        .all(|argument| Self::new(argument).is_simple_impl(depth + 1))
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_simple_member_expression(&self, depth: u8) -> bool {
        if let Self::Expression(expr) = self {
            match expr {
                Expression::StaticMemberExpression(static_expression) => {
                    Self::from(&static_expression.object).is_simple_impl(depth)
                }
                Expression::ComputedMemberExpression(computed_expression) => {
                    Self::from(&computed_expression.expression).is_simple_impl(depth)
                        && Self::from(&computed_expression.object).is_simple_impl(depth)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn is_simple_non_null_assertion_expression(&self, depth: u8) -> bool {
        if let Self::Expression(Expression::TSNonNullExpression(assertion)) = self {
            Self::from(&assertion.expression).is_simple_impl(depth)
        } else {
            false
        }
    }

    fn is_simple_unary_expression(&self, depth: u8) -> bool {
        if let Self::Expression(Expression::UnaryExpression(unary_expression)) = self {
            if matches!(
                unary_expression.operator,
                UnaryOperator::LogicalNot
                    | UnaryOperator::UnaryNegation
                    | UnaryOperator::UnaryPlus
                    | UnaryOperator::BitwiseNot
            ) {
                Self::from(&unary_expression.argument).is_simple_impl(depth)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_simple_update_expression(&self, depth: u8) -> bool {
        // Both PreUpdate and PostUpdate expressions have Increment and Decrement
        // operators, but they are typed separately, so must be handled that way.
        // These arms should be equivalent.
        if let Self::Expression(Expression::UpdateExpression(update)) = self {
            Self::Assignment(&update.argument).is_simple_impl(depth)
        } else {
            false
        }
    }

    fn is_simple_regex_expression(&self) -> bool {
        if let Self::Expression(Expression::RegExpLiteral(regex)) = self {
            return regex.regex.pattern.text.len() <= 5;
        }

        false
    }

    fn is_simple_array_expression(&self, depth: u8) -> bool {
        if let Self::Expression(Expression::ArrayExpression(array)) = self {
            array.elements.iter().all(|element| match element {
                match_expression!(ArrayExpressionElement) => {
                    Self::from(element.to_expression()).is_simple_impl(depth + 1)
                }
                ArrayExpressionElement::Elision(_) => true,
                ArrayExpressionElement::SpreadElement(_) => false,
            })
        } else {
            false
        }
    }

    fn is_simple_template(&self, depth: u8) -> bool {
        match self {
            Self::Expression(Expression::TemplateLiteral(template)) => {
                is_simple_template_literal(template, depth + 1)
            }
            Self::Expression(Expression::TaggedTemplateExpression(template)) => {
                is_simple_template_literal(&template.quasi, depth + 1)
            }
            _ => false,
        }
    }

    const fn is_simple_literal(&self) -> bool {
        // if let Self::Name(AnyJsName::JsPrivateName(_)) = self {
        //     return true;
        // }

        if let Self::Expression(Expression::RegExpLiteral(_)) = self {
            return false;
        }

        matches!(
            self,
            Self::Expression(
                Expression::NullLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::StringLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BigIntLiteral(_)
                    | Expression::ThisExpression(_)
                    | Expression::Identifier(_)
                    | Expression::Super(_)
            ) | Self::Assignment(SimpleAssignmentTarget::AssignmentTargetIdentifier(_))
        )
    }

    fn is_simple_object_expression(&self, depth: u8) -> bool {
        if let Self::Expression(Expression::ObjectExpression(object)) = self {
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
        } else {
            false
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
/// - all strings dont contain newlines
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
