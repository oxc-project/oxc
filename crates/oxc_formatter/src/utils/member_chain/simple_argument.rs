use unicode_width::UnicodeWidthStr;

use oxc_ast::{ast::*, match_expression};

use crate::{formatter::JsFormatter, utils::tailwindcss::tailwind_call_collapses_whitespace};

/// This enum tracks the arguments inside a call expressions and checks if they are
/// simple or not.
///
/// The heuristic changes based on its type and the depth of the expressions. For example
/// if we have expressions as arguments, having 2 or more them tags the first argument as "not simple".
///
/// Criteria are different:
/// - *complex*: if the chain of simple arguments exceeds the depth 2 or higher
/// - *complex*: if the argument is a [RegExpLiteral] with width greater than 5
/// - *simple*: the argument is a literal
/// - *simple*: the argument is a [RegExpLiteral] with width less than 5
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
    Spread,
}

impl<'a, 'b> SimpleArgument<'a, 'b> {
    pub fn new(node: &'b Argument<'a>) -> Self {
        match node {
            Argument::SpreadElement(_) => Self::Spread,
            match_expression!(Argument) => Self::from(node.to_expression()),
        }
    }

    /// `tailwind_collapses` tells whether the Tailwind context the argument is printed in
    /// collapses the line breaks of its templates, so they're judged as printed.
    /// - Mirrors the printer: a call hides the context for its callee and arguments,
    ///   or pushes one for its arguments when Tailwind (see [`tailwind_call_collapses_whitespace`]),
    ///   and `new` does neither
    pub fn is_simple(&self, tailwind_collapses: bool, f: &JsFormatter<'_, '_>) -> bool {
        self.is_simple_impl(0, tailwind_collapses, f)
    }

    pub fn is_simple_with_depth(
        &self,
        depth: u8,
        tailwind_collapses: bool,
        f: &JsFormatter<'_, '_>,
    ) -> bool {
        self.is_simple_impl(depth, tailwind_collapses, f)
    }

    fn is_simple_impl(&self, depth: u8, collapses: bool, f: &JsFormatter<'_, '_>) -> bool {
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
                Expression::RegExpLiteral(regex) => regex.regex.pattern.text.width() <= 5,
                Expression::TemplateLiteral(template) => {
                    is_simple_template_literal(template, depth + 1, collapses, f)
                }
                Expression::ObjectExpression(object) => {
                    Self::is_simple_object(object, depth, collapses, f)
                }
                Expression::ArrayExpression(array) => {
                    Self::is_simple_array(array, depth, collapses, f)
                }
                Expression::UnaryExpression(unary_expression) => {
                    matches!(
                        unary_expression.operator,
                        UnaryOperator::LogicalNot
                            | UnaryOperator::UnaryNegation
                            | UnaryOperator::UnaryPlus
                            | UnaryOperator::BitwiseNot
                    ) && Self::from(&unary_expression.argument).is_simple_impl(depth, collapses, f)
                }
                Expression::UpdateExpression(update) => {
                    Self::Assignment(&update.argument).is_simple_impl(depth, collapses, f)
                }
                Expression::TSNonNullExpression(assertion) => {
                    Self::from(&assertion.expression).is_simple_impl(depth, collapses, f)
                }
                Expression::StaticMemberExpression(static_expression) => {
                    Self::from(&static_expression.object).is_simple_impl(depth, collapses, f)
                }
                Expression::ComputedMemberExpression(computed_expression) => {
                    Self::from(&computed_expression.expression).is_simple_impl(depth, collapses, f)
                        && Self::from(&computed_expression.object)
                            .is_simple_impl(depth, collapses, f)
                }
                // In Prettier's default `typescript` parser (typescript-estree) AST,
                // `this.#v` is a `MemberExpression` with a `PrivateIdentifier` property.
                // In oxc's AST, it's a separate type.
                // The private field name is always simple, so only check the object.
                // https://github.com/prettier/prettier/blob/093745f0ec429d3db47c1edd823357e0ef24e226/src/language-js/utilities/index.js#L643-L648
                Expression::PrivateFieldExpression(private_field) => {
                    Self::from(&private_field.object).is_simple_impl(depth, collapses, f)
                }
                // Unlike a call, `new` neither pushes nor hides a Tailwind context
                Expression::NewExpression(expr) => Self::is_simple_call_like(
                    &expr.callee,
                    &expr.arguments,
                    depth,
                    collapses,
                    collapses,
                    f,
                ),
                Expression::CallExpression(expr) => Self::is_simple_call(expr, depth, collapses, f),
                Expression::ImportExpression(expr) => depth < 2 && expr.options.is_none(),
                Expression::ChainExpression(chain) => {
                    Self::is_simple_chain_element(&chain.expression, depth, collapses, f)
                }
                _ => false,
            },
            Self::Assignment(SimpleAssignmentTarget::AssignmentTargetIdentifier(_)) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_simple_object(
        object: &'b ObjectExpression<'a>,
        depth: u8,
        collapses: bool,
        f: &JsFormatter<'_, '_>,
    ) -> bool {
        object.properties.iter().all(|member| {
            if let ObjectPropertyKind::ObjectProperty(property) = member {
                if property.method {
                    return false;
                }

                if property.shorthand {
                    return true;
                }

                !property.computed
                    && Self::from(&property.value).is_simple_impl(depth + 1, collapses, f)
            } else {
                false
            }
        })
    }

    #[inline]
    fn is_simple_array(
        array: &'b ArrayExpression<'a>,
        depth: u8,
        collapses: bool,
        f: &JsFormatter<'_, '_>,
    ) -> bool {
        array.elements.iter().all(|element| match element {
            match_expression!(ArrayExpressionElement) => {
                Self::from(element.to_expression()).is_simple_impl(depth + 1, collapses, f)
            }
            ArrayExpressionElement::Elision(_) => true,
            ArrayExpressionElement::SpreadElement(_) => false,
        })
    }

    #[inline]
    fn is_simple_call(
        call: &'b CallExpression<'a>,
        depth: u8,
        collapses: bool,
        f: &JsFormatter<'_, '_>,
    ) -> bool {
        let arguments_collapse = tailwind_call_collapses_whitespace(&call.callee, f);
        Self::is_simple_call_like(
            &call.callee,
            &call.arguments,
            depth,
            collapses && arguments_collapse,
            arguments_collapse,
            f,
        )
    }

    #[inline]
    fn is_simple_call_like(
        callee: &'b Expression<'a>,
        arguments: &'b [Argument<'a>],
        depth: u8,
        callee_collapses: bool,
        arguments_collapse: bool,
        f: &JsFormatter<'_, '_>,
    ) -> bool {
        Self::from(callee).is_simple_impl(depth, callee_collapses, f)
            && arguments.len() + usize::from(depth) <= 2
            && arguments.iter().all(|argument| {
                Self::new(argument).is_simple_impl(depth + 1, arguments_collapse, f)
            })
    }

    #[inline]
    fn is_simple_chain_element(
        element: &'b ChainElement<'a>,
        depth: u8,
        collapses: bool,
        f: &JsFormatter<'_, '_>,
    ) -> bool {
        match element {
            ChainElement::CallExpression(call) => Self::is_simple_call(call, depth, collapses, f),
            ChainElement::TSNonNullExpression(assertion) => {
                Self::from(&assertion.expression).is_simple_impl(depth, collapses, f)
            }
            ChainElement::StaticMemberExpression(static_expression) => {
                Self::from(&static_expression.object).is_simple_impl(depth, collapses, f)
            }
            ChainElement::ComputedMemberExpression(computed_expression) => {
                Self::from(&computed_expression.expression).is_simple_impl(depth, collapses, f)
                    && Self::from(&computed_expression.object).is_simple_impl(depth, collapses, f)
            }
            ChainElement::PrivateFieldExpression(private_field) => {
                Self::from(&private_field.object).is_simple_impl(depth, collapses, f)
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
/// - all strings don't contain newlines, or Tailwind collapses them (`collapses`)
/// - the expressions contained in the template are considered as [`Argument`]. Check
///   [`SimpleArgument`].
fn is_simple_template_literal(
    template: &TemplateLiteral<'_>,
    depth: u8,
    collapses: bool,
    f: &JsFormatter<'_, '_>,
) -> bool {
    (collapses || template.quasis.iter().all(|quasi| !quasi.value.raw.contains('\n')))
        && template
            .expressions
            .iter()
            .all(|expr| SimpleArgument::Expression(expr).is_simple_impl(depth, collapses, f))
}
