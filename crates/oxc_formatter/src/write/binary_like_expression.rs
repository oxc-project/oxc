use std::mem::transmute_copy;

use oxc_allocator::{Address, CloneIn, GetAddress};
use oxc_ast::{ast::*, precedence};
use oxc_span::GetSpan;
use oxc_syntax::precedence::{GetPrecedence, Precedence};

use crate::{
    Format,
    formatter::{FormatResult, Formatter, trivia::FormatTrailingComments},
    generated::ast_nodes::{AstNode, AstNodes},
    utils::format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
};

use crate::{format_args, formatter::prelude::*, write};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryLikeOperator {
    BinaryOperator(BinaryOperator),
    LogicalOperator(LogicalOperator),
}

impl From<BinaryOperator> for BinaryLikeOperator {
    fn from(value: BinaryOperator) -> Self {
        Self::BinaryOperator(value)
    }
}

impl From<LogicalOperator> for BinaryLikeOperator {
    fn from(value: LogicalOperator) -> Self {
        Self::LogicalOperator(value)
    }
}

impl Format<'_> for BinaryLikeOperator {
    fn fmt(&self, f: &mut Formatter<'_, '_>) -> FormatResult<()> {
        let operator = match self {
            Self::BinaryOperator(op) => op.as_str(),
            Self::LogicalOperator(op) => op.as_str(),
        };

        write!(f, operator)
    }
}

impl BinaryLikeOperator {
    pub fn precedence(self) -> Precedence {
        match self {
            Self::BinaryOperator(op) => op.precedence(),
            Self::LogicalOperator(op) => op.precedence(),
        }
    }

    pub fn is_remainder(self) -> bool {
        matches!(self, Self::BinaryOperator(BinaryOperator::Remainder))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryLikeExpression<'a, 'b> {
    LogicalExpression(&'b AstNode<'a, LogicalExpression<'a>>),
    BinaryExpression(&'b AstNode<'a, BinaryExpression<'a>>),
}

impl<'a, 'b> BinaryLikeExpression<'a, 'b> {
    /// Returns the left hand side of the binary expression.
    fn left(&self) -> &'b AstNode<'a, Expression<'a>> {
        match self {
            Self::LogicalExpression(expr) => expr.left(),
            Self::BinaryExpression(expr) => expr.left(),
        }
    }

    /// Returns the right hand side of the binary expression.
    pub fn right(&self) -> &'b AstNode<'a, Expression<'a>> {
        match self {
            Self::LogicalExpression(expr) => expr.right(),
            Self::BinaryExpression(expr) => expr.right(),
        }
    }

    pub fn parent(&self) -> &AstNodes<'a> {
        match self {
            Self::LogicalExpression(expr) => expr.parent,
            Self::BinaryExpression(expr) => expr.parent,
        }
    }

    /// Returns `true` if the expression is inside of a test condition of `parent`.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// if (a + b) {} // true
    /// if (true) { a + b } // false
    /// switch (a + b) {} // true
    /// ```
    fn is_inside_condition(&self, parent: &AstNodes<'_>) -> bool {
        match parent {
            AstNodes::IfStatement(stmt) => stmt.test().span() == self.span(),
            AstNodes::DoWhileStatement(stmt) => stmt.test().span() == self.span(),
            AstNodes::WhileStatement(stmt) => stmt.test().span() == self.span(),
            AstNodes::SwitchStatement(stmt) => stmt.discriminant().span() == self.span(),
            _ => false,
        }
    }

    pub fn operator(&self) -> BinaryLikeOperator {
        match self {
            Self::LogicalExpression(expr) => BinaryLikeOperator::from(expr.operator()),
            Self::BinaryExpression(expr) => BinaryLikeOperator::from(expr.operator()),
        }
    }

    /// Determines if a binary like expression should be flattened or not. As a rule of thumb, an expression
    /// can be flattened if its left hand side has the same operator-precedence
    fn can_flatten(&self) -> bool {
        let left_operator = match self.left().as_ref() {
            Expression::BinaryExpression(expr) => BinaryLikeOperator::from(expr.operator),
            Expression::LogicalExpression(expr) => BinaryLikeOperator::from(expr.operator),
            _ => return false,
        };

        should_flatten(self.operator(), left_operator)
    }

    fn should_inline_logical_expression(&self) -> bool {
        let Self::LogicalExpression(logical) = self else {
            return false;
        };
        Self::can_inline_logical_expr(logical)
    }

    pub fn can_inline_logical_expr(logical: &LogicalExpression) -> bool {
        match &logical.right {
            Expression::ObjectExpression(object) => !object.properties.is_empty(),
            Expression::ArrayExpression(array) => !array.elements.is_empty(),
            Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
            _ => false,
        }
    }

    /// This function checks whether the chain of logical/binary expressions **should not** be indented
    ///
    /// There are some cases where the indentation is done by the parent, so if the parent is already doing
    /// the indentation, then there's no need to do a second indentation.
    /// [Prettier applies]: <https://github.com/prettier/prettier/blob/b0201e01ef99db799eb3716f15b7dfedb0a2e62b/src/language-js/print/binaryish.js#L122-L125>
    pub fn should_not_indent_if_parent_indents(&self, parent: &AstNodes<'a>) -> bool {
        match parent {
            AstNodes::ReturnStatement(_)
            | AstNodes::ThrowStatement(_)
            | AstNodes::ForStatement(_)
            | AstNodes::TemplateLiteral(_) => true,
            AstNodes::JSXExpressionContainer(container) => {
                matches!(container.parent, AstNodes::JSXAttribute(_))
            }
            AstNodes::ExpressionStatement(statement) => {
                if let AstNodes::FunctionBody(arrow) = statement.parent {
                    arrow.span == self.span()
                } else {
                    false
                }
            }
            AstNodes::ConditionalExpression(conditional) => !matches!(
                conditional.parent,
                AstNodes::ReturnStatement(_)
                    | AstNodes::ThrowStatement(_)
                    | AstNodes::CallExpression(_)
                    | AstNodes::ImportExpression(_)
                    | AstNodes::MetaProperty(_)
            ),
            _ => false,
        }
    }
}

impl GetSpan for BinaryLikeExpression<'_, '_> {
    fn span(&self) -> oxc_span::Span {
        match self {
            Self::LogicalExpression(expr) => expr.span(),
            Self::BinaryExpression(expr) => expr.span(),
        }
    }
}

impl GetAddress for BinaryLikeExpression<'_, '_> {
    fn address(&self) -> Address {
        match self {
            Self::LogicalExpression(expr) => Address::from_ptr(*expr),
            Self::BinaryExpression(expr) => Address::from_ptr(*expr),
        }
    }
}

impl<'a, 'b> TryFrom<&'b AstNode<'a, Expression<'a>>> for BinaryLikeExpression<'a, 'b> {
    type Error = ();

    fn try_from(value: &'b AstNode<'a, Expression<'a>>) -> Result<Self, Self::Error> {
        match value.as_ast_nodes() {
            AstNodes::LogicalExpression(expr) => Ok(Self::LogicalExpression(expr)),
            AstNodes::BinaryExpression(expr) => Ok(Self::BinaryExpression(expr)),
            _ => Err(()),
        }
    }
}

impl<'a> Format<'a> for BinaryLikeExpression<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let parent = self.parent();
        let is_inside_condition = self.is_inside_condition(parent);
        let parts = split_into_left_and_right_sides(*self, is_inside_condition);

        // Don't indent inside of conditions because conditions add their own indent and grouping.
        if is_inside_condition {
            return write!(f, [&format_once(|f| { f.join().entries(parts).finish() })]);
        }

        // Add a group with a soft block indent in cases where it is necessary to parenthesize the binary expression.
        // For example, `(a+b)(call)`, `!(a + b)`, `(a + b).test`.
        let is_inside_parenthesis = match parent {
            AstNodes::StaticMemberExpression(_) | AstNodes::UnaryExpression(_) => true,
            AstNodes::CallExpression(call) => {
                call.callee().without_parentheses().span() == self.span()
            }
            AstNodes::NewExpression(new) => {
                new.callee().without_parentheses().span() == self.span()
            }
            _ => false,
        };

        if is_inside_parenthesis {
            return write!(
                f,
                [group(&soft_block_indent(&format_once(|f| { f.join().entries(parts).finish() })))]
            );
        }

        if self.should_not_indent_if_parent_indents(self.parent()) || {
            let flattened = parts.len() > 2;
            let inline_logical_expression = self.should_inline_logical_expression();
            let should_indent_if_inlines = should_indent_if_parent_inlines(self.parent());
            (inline_logical_expression && !flattened)
                || (!inline_logical_expression && should_indent_if_inlines)
        } {
            return write!(f, [group(&format_once(|f| { f.join().entries(parts).finish() }))]);
        }

        if let Some(first) = parts.first() {
            let last_is_jsx = parts.last().is_some_and(BinaryLeftOrRightSide::is_jsx);
            let tail_parts = if last_is_jsx { &parts[1..parts.len() - 1] } else { &parts[1..] };

            let group_id = f.group_id("logicalChain");

            let format_non_jsx_parts = format_with(|f| {
                write!(
                    f,
                    [group(&format_args!(
                        first,
                        indent(&format_once(|f| { f.join().entries(tail_parts.iter()).finish() }))
                    ))
                    .with_group_id(Some(group_id))]
                )
            });

            if last_is_jsx {
                // `last_is_jsx` is only true if parts is not empty
                let jsx_element = parts.last().unwrap();
                write!(
                    f,
                    [group(&format_args!(
                        format_non_jsx_parts,
                        indent_if_group_breaks(&jsx_element, group_id),
                    ))]
                )
            } else {
                write!(f, format_non_jsx_parts)
            }
        } else {
            // Empty, should never ever happen but let's gracefully recover.
            Ok(())
        }
    }
}

/// Represents the right or left hand side of a binary expression.
#[derive(Debug)]
enum BinaryLeftOrRightSide<'a, 'b> {
    /// A terminal left hand side of a binary expression.
    ///
    /// Formats the left hand side only.
    Left { parent: BinaryLikeExpression<'a, 'b> },

    /// The right hand side of a binary expression.
    /// Formats the operand together with the right hand side.
    Right {
        parent: BinaryLikeExpression<'a, 'b>,
        /// Is the parent the condition of a `if` / `while` / `do-while` / `for` statement?
        inside_condition: bool,
        /// It is the root of the expression.
        root: bool,
    },
}

impl<'a> Format<'a> for BinaryLeftOrRightSide<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Left { parent } => write!(f, group(parent.left())),
            Self::Right {
                parent: binary_like_expression,
                inside_condition: inside_parenthesis,
                root,
            } => {
                let mut binary_like_expression = *binary_like_expression;
                // // It's only possible to suppress the formatting of the whole binary expression formatting OR
                // // the formatting of the right hand side value but not of a nested binary expression.
                // // This aligns with Prettier's behaviour.
                // f.context().comments().mark_suppression_checked(binary_like_expression.syntax());

                let logical_operator = if let BinaryLikeExpression::LogicalExpression(logical) =
                    binary_like_expression
                {
                    Some(logical.operator())
                } else {
                    None
                };

                // `(longVariable === "long-string") && ((1 <= longVariable) && (longVariable <= 100000000));`
                //  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                //                            is a LogicalExpression with the `&&` operator
                //
                //                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                //                                        the right side of the parent `LogicalExpression` is
                //                                        also a `LogicalExpression` with the `&&` operator
                //
                // In this case, the both are `LogicalExpression`s and have the same operator, then we have to
                // flatten them into the same group, so these three parts, respectively,
                // `longVariable === "long-string"`, `1 <= longVariable` and `longVariable <= 100000000` should
                // be formatted in the same group.
                //
                // In the following logic, we will recursively find the right side of the LogicalExpression to
                // ensure that all parts are in the same group.
                //
                // Example output:
                // ```js
                // longVariable === "long-string" &&
                //  1 <= longVariable &&
                //  longVariable <= 100000000;
                // ```
                //
                // Based on Prettier's rebalancing logic for LogicalExpressions:
                // <https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/parse/postprocess/index.js#L64-L69>
                loop {
                    if let AstNodes::LogicalExpression(right_logical) =
                        binary_like_expression.right().as_ast_nodes()
                        && let Some(operator) = logical_operator
                        && operator == right_logical.operator()
                    {
                        write!(
                            f,
                            [
                                space(),
                                operator.as_str(),
                                soft_line_break_or_space(),
                                format_once(|f| {
                                    // If the left side of the right logical expression is still a logical expression with
                                    // the same operator, we need to recursively split it into left and right sides.
                                    // This way, we can ensure that all parts are in the same group.
                                    let left_child = right_logical.left();
                                    if let AstNodes::LogicalExpression(left_logical_child) =
                                        left_child.as_ast_nodes()
                                        && operator == left_logical_child.operator()
                                    {
                                        let left_parts = split_into_left_and_right_sides(
                                            BinaryLikeExpression::LogicalExpression(
                                                left_logical_child,
                                            ),
                                            *inside_parenthesis,
                                        );

                                        f.join().entries(left_parts).finish()
                                    } else {
                                        left_child.fmt(f)
                                    }
                                })
                            ]
                        )?;

                        binary_like_expression =
                            BinaryLikeExpression::LogicalExpression(right_logical);
                    } else {
                        break;
                    }
                }

                let right = binary_like_expression.right();

                let operator_and_right_expression = format_with(|f| {
                    write!(f, [space(), binary_like_expression.operator()])?;

                    if binary_like_expression.should_inline_logical_expression() {
                        write!(f, [space()])?;
                    } else {
                        write!(f, [soft_line_break_or_space()])?;
                    }

                    if *root {
                        write!(f, FormatNodeWithoutTrailingComments(right))
                    } else {
                        write!(f, right)
                    }
                });

                // Doesn't match prettier that only distinguishes between logical and binary
                let should_group = !(is_same_binary_expression_kind(
                    binary_like_expression,
                    binary_like_expression.parent(),
                ) || is_same_binary_expression_kind(
                    binary_like_expression,
                    binary_like_expression.left().as_ast_nodes(),
                ) || is_same_binary_expression_kind(
                    binary_like_expression,
                    right.as_ast_nodes(),
                ) || (*inside_parenthesis && logical_operator.is_some()));

                if should_group {
                    // `left` side has printed before `right` side, so that trailing comments of `left` side has been printed,
                    // so we need to find if there are any printed comments that are after the `left` side and it is line comment.
                    // If so, it should break the line.
                    // ```js
                    // a = b + // comment
                    // c
                    // ```
                    // // to
                    // ```js
                    // a =
                    //     b || // Comment
                    //     c;
                    let should_break = f
                        .comments()
                        .printed_comments()
                        .iter()
                        .rev()
                        .take_while(|comment| {
                            binary_like_expression.left().span().end < comment.span.start
                                && right.span().start > comment.span.end
                        })
                        .any(|comment| comment.is_line());

                    write!(f, [group(&operator_and_right_expression).should_expand(should_break)])
                } else {
                    write!(f, [operator_and_right_expression])
                }
            }
        }
    }
}

impl BinaryLeftOrRightSide<'_, '_> {
    fn is_jsx(&self) -> bool {
        match self {
            BinaryLeftOrRightSide::Left { parent } => {
                matches!(
                    parent.left().as_ref(),
                    Expression::JSXElement(_) | Expression::JSXFragment(_)
                )
            }
            BinaryLeftOrRightSide::Right { parent, .. } => {
                matches!(
                    parent.right().as_ref(),
                    Expression::JSXElement(_) | Expression::JSXFragment(_)
                )
            }
            _ => false,
        }
    }
}

/// Creates a [BinaryLeftOrRightSide::Left] for the first left hand side that:
/// * isn't a [BinaryLikeExpression]
/// * is a [BinaryLikeExpression] but it should be formatted as its own group (see [BinaryLikeExpression::can_flatten]).
///
/// It then traverses upwards from the left most node and creates [BinaryLeftOrRightSide::Right]s for
/// every [BinaryLikeExpression] until it reaches the root again.
fn split_into_left_and_right_sides<'a, 'b>(
    binary: BinaryLikeExpression<'a, 'b>,
    inside_condition: bool,
) -> Vec<BinaryLeftOrRightSide<'a, 'b>> {
    fn split_into_left_and_right_sides_inner<'a, 'b>(
        is_root: bool,
        binary: BinaryLikeExpression<'a, 'b>,
        inside_condition: bool,
        items: &mut Vec<BinaryLeftOrRightSide<'a, 'b>>,
    ) {
        let left = binary.left();

        if binary.can_flatten() {
            // We can flatten the left hand side, so we need to check if we have a nested binary expression
            // that we can flatten.
            split_into_left_and_right_sides_inner(
                false,
                // SAFETY: `left` is guaranteed to be a valid binary like expression in `can_flatten()`.
                BinaryLikeExpression::try_from(left).unwrap(),
                inside_condition,
                items,
            );
        } else {
            items.push(BinaryLeftOrRightSide::Left { parent: binary });
        }

        items.push(BinaryLeftOrRightSide::Right {
            parent: binary,
            inside_condition,
            root: is_root,
        });
    }

    // Stores the left and right parts of the binary expression in sequence (rather than nested as they
    // appear in the tree).
    // `with_capacity(2)` because we expect at least 2 items (left and right).
    let mut items = Vec::with_capacity(2);

    split_into_left_and_right_sides_inner(true, binary, inside_condition, &mut items);

    items
}

/// There are cases where the parent decides to inline the element; in
/// these cases the decide to actually break on a new line and indent it.
///
/// This function checks what the parents adheres to this behaviour
fn should_indent_if_parent_inlines(parent: &AstNodes<'_>) -> bool {
    matches!(
        parent,
        AstNodes::AssignmentExpression(_)
            | AstNodes::ObjectProperty(_)
            | AstNodes::VariableDeclarator(_)
            | AstNodes::PropertyDefinition(_)
    )
}

fn is_same_binary_expression_kind(
    binary: BinaryLikeExpression<'_, '_>,
    other: &AstNodes<'_>,
) -> bool {
    match binary {
        BinaryLikeExpression::LogicalExpression(_) => {
            matches!(other, AstNodes::LogicalExpression(_))
        }
        BinaryLikeExpression::BinaryExpression(_) => {
            matches!(other, AstNodes::BinaryExpression(_))
        }
    }
}

pub fn should_flatten(parent_operator: BinaryLikeOperator, operator: BinaryLikeOperator) -> bool {
    let parent_precedence = parent_operator.precedence();
    let precedence = operator.precedence();

    if parent_precedence != precedence {
        return false;
    }

    match (parent_operator.precedence(), operator.precedence()) {
        // `**` is right associative
        (Precedence::Exponentiation, _) |
        // `a == b == c` => `(a == b) == c`
        (Precedence::Equals, Precedence::Equals) |
        // `a << 3 << 4` -> `(a << 3) << 4`
        (Precedence::Shift, Precedence::Shift) => false,
        (Precedence::Multiply, Precedence::Multiply) => {
            // `a * 3 % 5` -> `(a * 3) % 5`
            if parent_operator.is_remainder() || operator.is_remainder() {
                return false;
            }

            // `a * 3 / 5` -> `(a * 3) / 5`
            parent_operator == operator
        }
        _ => true,
    }
}
