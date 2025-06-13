use std::mem::transmute_copy;

use oxc_allocator::{Address, CloneIn, GetAddress};
use oxc_ast::{ast::*, precedence};
use oxc_span::GetSpan;
use oxc_syntax::precedence::{GetPrecedence, Precedence};

use crate::{
    Format,
    formatter::{FormatResult, Formatter},
    generated::ast_nodes::{AstNode, AstNodes},
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

impl BinaryLikeOperator {
    fn as_str(self) -> &'static str {
        match self {
            Self::BinaryOperator(op) => op.as_str(),
            Self::LogicalOperator(op) => op.as_str(),
        }
    }

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

#[derive(Clone, Copy)]
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
        match &logical.right().as_ref() {
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
            // JsSyntaxKind::JSX_EXPRESSION_ATTRIBUTE_VALUE => true,
            AstNodes::ArrowFunctionExpression(arrow) => arrow.body().span() == self.span(),
            AstNodes::ConditionalExpression(conditional) => {
                matches!(
                    parent.parent(),
                    AstNodes::ReturnStatement(_)
                        | AstNodes::ThrowStatement(_)
                        | AstNodes::CallExpression(_)
                        | AstNodes::ImportExpression(_)
                        | AstNodes::Argument(_)
                        | AstNodes::MetaProperty(_)
                )
            }
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
            AstNodes::MemberExpression(_) | AstNodes::UnaryExpression(_) => true,
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

        let inline_logical_expression = self.should_inline_logical_expression();
        let should_indent_if_inlines = should_indent_if_parent_inlines(self.parent());
        let should_not_indent = self.should_not_indent_if_parent_indents(self.parent());

        let flattened = parts.len() > 2;

        if should_not_indent
            || (inline_logical_expression && !flattened)
            || (!inline_logical_expression && should_indent_if_inlines)
        {
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

        /// Indicates if the comments of the parent should be printed or not.
        /// Must be true if `parent` isn't the root `BinaryLikeExpression` for which `format` is called.
        print_parent_comments: bool,

        /// Indicates if the parent has the same kind as the current binary expression.
        parent_has_same_kind: bool,
    },
}

impl<'a> Format<'a> for BinaryLeftOrRightSide<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Left { parent } => write!(f, [group(parent.left())]),
            Self::Right {
                parent: binary_like_expression,
                inside_condition: inside_parenthesis,
                print_parent_comments,
                parent_has_same_kind,
            } => {
                // // It's only possible to suppress the formatting of the whole binary expression formatting OR
                // // the formatting of the right hand side value but not of a nested binary expression.
                // // This aligns with Prettier's behaviour.
                // f.context().comments().mark_suppression_checked(binary_like_expression.syntax());
                let right = binary_like_expression.right();
                let operator = binary_like_expression.operator();
                let operator_and_right_expression = format_with(|f| {
                    write!(f, [space(), operator.as_str()])?;

                    if binary_like_expression.should_inline_logical_expression() {
                        write!(f, [space()])?;
                    } else {
                        write!(f, [soft_line_break_or_space()])?;
                    }

                    write!(f, right)?;

                    Ok(())
                });

                // Doesn't match prettier that only distinguishes between logical and binary
                let left_has_same_kind = is_same_binary_expression_kind(
                    binary_like_expression,
                    binary_like_expression.left(),
                );

                let right_has_same_kind =
                    is_same_binary_expression_kind(binary_like_expression, right);

                // let should_break = f
                //     .context()
                //     .comments()
                //     .trailing_comments(binary_like_expression.left()?.syntax())
                //     .iter()
                //     .any(|comment| comment.kind().is_line());
                let should_break = false;

                let should_group = !(*parent_has_same_kind
                    || left_has_same_kind
                    || right_has_same_kind
                    || (*inside_parenthesis
                        && matches!(
                            binary_like_expression,
                            BinaryLikeExpression::LogicalExpression(_)
                        )));

                // if *print_parent_comments {
                //     write!(f, binary_like_expression)?;
                // }

                if should_group {
                    write!(f, [group(&operator_and_right_expression).should_expand(should_break)])?;
                } else {
                    write!(f, [operator_and_right_expression])?;
                }

                // if *print_parent_comments {
                //     write!(f, [format_trailing_comments(binary_like_expression.syntax())])?;
                // }

                Ok(())
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
    root: BinaryLikeExpression<'a, 'b>,
    inside_condition: bool,
) -> Vec<BinaryLeftOrRightSide<'a, 'b>> {
    fn split_into_left_and_right_sides_inner<'a, 'b>(
        binary: BinaryLikeExpression<'a, 'b>,
        inside_condition: bool,
        parent_has_same_kind: bool,
        items: &mut Vec<BinaryLeftOrRightSide<'a, 'b>>,
    ) {
        let left = binary.left();
        let right = binary.right();

        if binary.can_flatten() {
            // We can flatten the left hand side, so we need to check if we have a nested binary expression
            // that we can flatten.
            split_into_left_and_right_sides_inner(
                // SAFETY: `left` is guaranteed to be a valid binary like expression in `can_flatten()`.
                BinaryLikeExpression::try_from(left).unwrap(),
                inside_condition,
                is_same_binary_expression_kind(&binary, left),
                items,
            );
        } else {
            items.push(BinaryLeftOrRightSide::Left { parent: binary });
        }

        items.push(BinaryLeftOrRightSide::Right {
            parent: binary,
            inside_condition,
            // TODO:
            // print_parent_comments: expression.syntax() != root.syntax(),
            print_parent_comments: false,
            parent_has_same_kind,
        });
    }

    // Stores the left and right parts of the binary expression in sequence (rather than nested as they
    // appear in the tree).
    let mut items = Vec::new();

    split_into_left_and_right_sides_inner(root, inside_condition, false, &mut items);

    items
}

/// There are cases where the parent decides to inline the element; in
/// these cases the decide to actually break on a new line and indent it.
///
/// This function checks what the parents adheres to this behaviour
fn should_indent_if_parent_inlines(parent: &AstNodes<'_>) -> bool {
    if matches!(parent, AstNodes::AssignmentExpression(_) | AstNodes::ObjectProperty(_)) {
        return true;
    }

    match parent.parent() {
        AstNodes::VariableDeclarator(decl) => {
            decl.init().as_ref().is_some_and(|init| init.span() == parent.span())
        }
        AstNodes::PropertyDefinition(decl) => {
            decl.value().as_ref().is_some_and(|value| value.span() == parent.span())
        }
        _ => false,
    }
}

fn is_same_binary_expression_kind(
    binary: &BinaryLikeExpression<'_, '_>,
    other: &Expression<'_>,
) -> bool {
    match binary {
        BinaryLikeExpression::LogicalExpression(_) => {
            matches!(other, Expression::LogicalExpression(_))
        }
        BinaryLikeExpression::BinaryExpression(_) => {
            matches!(other, Expression::BinaryExpression(_))
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
