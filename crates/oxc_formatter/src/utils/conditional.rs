use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use crate::{
    Format, FormatResult, FormatWrite,
    formatter::{Formatter, prelude::*},
    generated::ast_nodes::{AstNode, AstNodes},
    write,
};

pub enum ConditionalLike<'a, 'b> {
    ConditionalExpression(&'b AstNode<'a, ConditionalExpression<'a>>),
    TSConditionalType(&'b AstNode<'a, TSConditionalType<'a>>),
}

impl<'a> ConditionalLike<'a, '_> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::ConditionalExpression(expr) => expr.span,
            Self::TSConditionalType(ty) => ty.span,
        }
    }

    #[inline]
    fn parent(&self) -> &AstNodes<'a> {
        match self {
            Self::ConditionalExpression(expr) => expr.parent,
            Self::TSConditionalType(ty) => ty.parent,
        }
    }

    #[inline]
    fn is_conditional_expression(&self) -> bool {
        matches!(self, Self::ConditionalExpression(_))
    }
}

/// Layout information for a conditional expression to determine formatting strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalLayout {
    /// This conditional isn't a child of another conditional.
    ///
    /// ```javascript
    /// return a ? b : c;
    /// ```
    Root {
        /// Whether this is part of a JSX conditional chain
        jsx_chain: bool,
    },
    /// Conditional that is the `test` of another conditional.
    ///
    /// ```javascript
    /// (
    ///     a              // <-- Note the extra indent here
    ///         ? b
    ///         : c
    ///  )
    ///     ? d
    ///     : e;
    /// ```
    NestedTest,
    /// Conditional that is the `consequent` of another conditional.
    ///
    /// ```javascript
    /// condition1
    ///     ? condition2
    ///         ? consequent2 // <-- consequent and alternate gets indented
    ///         : alternate2
    ///     : alternate1;
    /// ```
    NestedConsequent,
    /// Conditional that is the `alternate` of another conditional.
    ///
    /// The `test` condition of a nested alternated is aligned with the parent's `:`.
    ///
    /// ```javascript
    /// outerCondition
    ///     ? consequent
    ///     : nestedAlternate +
    ///       binary + // <- notice how the content is aligned to the `: `
    ///     ? consequentOfnestedAlternate
    ///     : alternateOfNestedAlternate;
    /// ```
    NestedAlternate,
}

impl ConditionalLayout {
    #[inline]
    fn is_root(self) -> bool {
        matches!(self, Self::Root { .. })
    }

    #[inline]
    fn is_nested_test(self) -> bool {
        matches!(self, Self::NestedTest)
    }

    #[inline]
    fn is_nested_consequent(self) -> bool {
        matches!(self, Self::NestedConsequent)
    }

    #[inline]
    fn is_nested_alternate(self) -> bool {
        matches!(self, Self::NestedAlternate)
    }

    #[inline]
    fn is_jsx_chain(self) -> bool {
        matches!(self, Self::Root { jsx_chain: true })
    }
}

impl<'a> ConditionalLike<'a, '_> {
    /// Determines the layout of this conditional based on its parent
    fn layout(&self) -> ConditionalLayout {
        let self_span = self.span();

        let (is_test, is_consequent) = match self.parent() {
            AstNodes::ConditionalExpression(parent) => {
                let parent_expr = parent.as_ref();
                (parent_expr.test.span() == self_span, parent_expr.consequent.span() == self_span)
            }
            AstNodes::TSConditionalType(parent) => {
                let parent_type = parent.as_ref();
                // For TS conditional types, both check_type and extends_type are part of the test
                let is_test = parent_type.check_type.span() == self_span
                    || parent_type.extends_type.span() == self_span;
                let is_consequent = parent_type.true_type.span() == self_span;
                (is_test, is_consequent)
            }
            _ => {
                let jsx_chain = self.is_conditional_expression() && self.is_jsx_conditional_chain();
                return ConditionalLayout::Root { jsx_chain };
            }
        };

        if is_test {
            ConditionalLayout::NestedTest
        } else if is_consequent {
            ConditionalLayout::NestedConsequent
        } else {
            ConditionalLayout::NestedAlternate
        }
    }

    /// Checks if this conditional expression contains JSX elements
    #[inline]
    fn is_jsx_conditional_chain(&self) -> bool {
        #[inline]
        fn has_jsx_expression(expr: &Expression) -> bool {
            matches!(expr, Expression::JSXElement(_) | Expression::JSXFragment(_))
        }

        let Self::ConditionalExpression(conditional) = self else {
            return false; // Types can't contain JSX
        };

        let conditional = conditional.as_ref();
        has_jsx_expression(&conditional.test)
            || has_jsx_expression(&conditional.consequent)
            || has_jsx_expression(&conditional.alternate)
    }

    /// It is desired to add an extra indent if this conditional is a ConditionalExpression and is directly inside
    /// of a member chain:
    ///
    /// ```javascript
    /// // Input
    /// return (a ? b : c).member
    ///
    /// // Default
    /// return (a
    ///     ? b
    ///     : c
    /// ).member
    ///
    /// // Preferred
    /// return (
    ///     a
    ///         ? b
    ///         : c
    /// ).member
    /// ```
    fn should_extra_indent(&self, layout: ConditionalLayout) -> bool {
        if !layout.is_root() {
            return false;
        }

        // Only check for ConditionalExpression, not TS types
        let Self::ConditionalExpression(expr) = self else {
            return false;
        };

        let mut expression_span = expr.span;
        let mut parent = expr.parent;

        // This tries to find the start of a member chain by iterating over all ancestors of the conditional.
        // The iteration "breaks" as soon as a non-member-chain node is found.
        loop {
            match parent {
                AstNodes::StaticMemberExpression(member) => {
                    if member.object.span() == expression_span {
                        expression_span = member.span();
                        parent = member.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::ComputedMemberExpression(member) => {
                    if member.object.span() == expression_span {
                        expression_span = member.span();
                        parent = member.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::CallExpression(call) => {
                    if call.callee.span() == expression_span {
                        expression_span = call.span();
                        parent = call.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::TSNonNullExpression(assertion) => {
                    if assertion.expression.span() == expression_span {
                        expression_span = assertion.span();
                        parent = assertion.parent;
                    } else {
                        break;
                    }
                }
                AstNodes::NewExpression(new_expr) => {
                    parent = new_expr.parent;
                    if new_expr.callee.span() == expression_span {
                        expression_span = new_expr.span();
                    }
                    break;
                }
                AstNodes::TSAsExpression(as_expr) => {
                    parent = as_expr.parent;
                    if as_expr.expression.span() == expression_span {
                        expression_span = as_expr.span();
                    }
                    break;
                }
                AstNodes::TSSatisfiesExpression(satisfies) => {
                    parent = satisfies.parent;
                    if satisfies.expression.span() == expression_span {
                        expression_span = satisfies.span();
                    }
                    break;
                }
                _ => break,
            }
        }

        // If we didn't find a member chain, no extra indent
        if expression_span == self.span() {
            return false;
        }

        // Check if the parent context requires extra indentation
        match parent {
            AstNodes::VariableDeclarator(decl) => {
                decl.init.as_ref().is_some_and(|init| init.span() == expression_span)
            }
            AstNodes::ReturnStatement(ret) => {
                ret.argument.as_ref().is_some_and(|arg| arg.span() == expression_span)
            }
            AstNodes::ThrowStatement(throw) => throw.argument.span() == expression_span,
            AstNodes::UnaryExpression(unary) => unary.argument.span() == expression_span,
            AstNodes::YieldExpression(yield_expr) => {
                yield_expr.argument.as_ref().is_some_and(|arg| arg.span() == expression_span)
            }
            AstNodes::AssignmentExpression(assign) => assign.right.span() == expression_span,
            _ => false,
        }
    }

    /// Checks if any part of the conditional has multiline comments
    #[inline]
    fn has_multiline_comment(&self, _f: &Formatter<'_, 'a>) -> bool {
        // TODO: Implement multiline comment detection
        false
    }

    /// Checks if the parent is a static member expression
    #[inline]
    fn is_parent_static_member_expression(&self, layout: ConditionalLayout) -> bool {
        layout.is_root()
            && self.is_conditional_expression()
            && matches!(self.parent(), AstNodes::StaticMemberExpression(_))
    }

    /// Formats the test part of the conditional
    fn format_test<'f>(
        &self,
        f: &mut Formatter<'f, 'a>,
        layout: ConditionalLayout,
    ) -> FormatResult<()> {
        let format_inner = format_with(|f| match self {
            Self::ConditionalExpression(conditional) => {
                write!(f, [conditional.test()])
            }
            Self::TSConditionalType(conditional) => {
                write!(
                    f,
                    [
                        conditional.check_type(),
                        space(),
                        "extends",
                        space(),
                        conditional.extends_type()
                    ]
                )
            }
        });

        if layout.is_nested_alternate() {
            // Align with parent's colon
            write!(f, [indent(&format_inner)])
        } else {
            format_inner.fmt(f)
        }
    }

    /// Formats the consequent and alternate with proper formatting
    fn format_consequent_and_alternate<'f>(
        &self,
        f: &mut Formatter<'f, 'a>,
        layout: ConditionalLayout,
    ) -> FormatResult<()> {
        write!(f, [soft_line_break_or_space(), "?", space()])?;

        let format_consequent = format_with(|f| match self {
            Self::ConditionalExpression(conditional) => {
                let is_consequent_nested = match self {
                    Self::ConditionalExpression(conditional) => {
                        matches!(conditional.consequent, Expression::ConditionalExpression(_))
                    }
                    Self::TSConditionalType(conditional) => {
                        matches!(conditional.true_type, TSType::TSConditionalType(_))
                    }
                };

                if is_consequent_nested && !layout.is_jsx_chain() {
                    // Add parentheses around the consequent if it is a conditional expression and fits on the same line
                    // so that it's easier to identify the parts that belong to a conditional expression.
                    // `a ? b ? c: d : e` -> `a ? (b ? c: d) : e`
                    write!(
                        f,
                        [
                            if_group_fits_on_line(&text("(")),
                            conditional.consequent(),
                            if_group_fits_on_line(&text(")"))
                        ]
                    )
                } else {
                    write!(f, [conditional.consequent()])
                }
            }
            Self::TSConditionalType(conditional) => {
                write!(f, [conditional.true_type()])
            }
        });

        write!(
            f,
            [
                indent(&format_consequent),
                soft_line_break_or_space(),
                ":",
                space(),
                indent(&format_with(|f| match self {
                    Self::ConditionalExpression(conditional) =>
                        write!(f, [conditional.alternate()]),
                    Self::TSConditionalType(conditional) => write!(f, [conditional.false_type()]),
                }))
            ]
        )
    }
}

impl<'a> Format<'a> for ConditionalLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let layout = self.layout();
        let should_extra_indent = self.should_extra_indent(layout);
        let has_multiline_comment = self.has_multiline_comment(f);

        let format_inner = format_with(|f| {
            self.format_test(f, layout)?;

            match &layout {
                ConditionalLayout::Root { jsx_chain: true } => match self {
                    Self::ConditionalExpression(conditional) => {
                        write!(
                            f,
                            [
                                space(),
                                "?",
                                space(),
                                format_jsx_chain_consequent(conditional.consequent()),
                                space(),
                                ":",
                                space(),
                                format_jsx_chain_alternate(conditional.alternate())
                            ]
                        )
                    }
                    Self::TSConditionalType(_) => {
                        unreachable!("TSConditionalType cannot be a JSX chain")
                    }
                },
                ConditionalLayout::Root { jsx_chain: false } | ConditionalLayout::NestedTest => {
                    write!(
                        f,
                        [indent(&format_with(|f| {
                            self.format_consequent_and_alternate(f, layout)
                        }))]
                    )
                }
                ConditionalLayout::NestedConsequent => {
                    write!(
                        f,
                        [dedent(&indent(&format_with(|f| {
                            self.format_consequent_and_alternate(f, layout)
                        })))]
                    )
                }
                ConditionalLayout::NestedAlternate => {
                    self.format_consequent_and_alternate(f, layout)
                }
            }?;

            // Add a soft line break in front of the closing `)` in case the parent is a static member expression
            // ```
            // (veryLongCondition
            //      ? a
            //      : b // <- enforce line break here if the conditional breaks
            // ).more
            // ```
            if self.is_parent_static_member_expression(layout)
                && !should_extra_indent
                && !layout.is_jsx_chain()
            {
                write!(f, [soft_line_break()])?;
            }

            Ok(())
        });

        let grouped = format_with(|f| {
            if layout.is_root() { write!(f, [group(&format_inner)]) } else { format_inner.fmt(f) }
        });

        if layout.is_nested_test() || should_extra_indent {
            write!(f, [group(&soft_block_indent(&grouped)).should_expand(has_multiline_comment)])
        } else {
            if has_multiline_comment {
                write!(f, [expand_parent()])?;
            }
            grouped.fmt(f)
        }
    }
}

/// Formats JSX consequent with conditional wrapping
fn format_jsx_chain_consequent<'a, 'b>(
    expression: &'b AstNode<'a, Expression<'a>>,
) -> impl Format<'a> + 'b {
    FormatJsxChainExpression { expression, alternate: false }
}

/// Formats JSX alternate with conditional wrapping
fn format_jsx_chain_alternate<'a, 'b>(
    expression: &'b AstNode<'a, Expression<'a>>,
) -> impl Format<'a> + 'b {
    FormatJsxChainExpression { expression, alternate: true }
}

/// A [ConditionalExpression] that itself or any of its parent's [ConditionalExpression] have a [JSXElement]
/// as its test, consequent or alternate.
///
/// Parenthesizes the `consequent` and `alternate` if the group breaks except if the expressions are
/// * `null`
/// * `undefined`
/// * or a nested ConditionalExpression in the alternate branch
///
/// ```javascript
/// abcdefgh? (
///   <Element>
///     <Sub />
///     <Sub />
///   </Element>
/// ) : (
///   <Element2>
///     <Sub />
///     <Sub />
///   </Element2>
/// );
/// ```
struct FormatJsxChainExpression<'a, 'b> {
    expression: &'b AstNode<'a, Expression<'a>>,
    alternate: bool,
}

impl<'a> Format<'a> for FormatJsxChainExpression<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let no_wrap = match self.expression.as_ref() {
            Expression::Identifier(ident) => ident.name == "undefined",
            Expression::NullLiteral(_) => true,
            Expression::ConditionalExpression(_) if self.alternate => true,
            _ => false,
        };

        if no_wrap {
            write!(f, [self.expression])
        } else {
            write!(
                f,
                [
                    if_group_breaks(&text("(")),
                    soft_block_indent(&self.expression),
                    if_group_breaks(&text(")"))
                ]
            )
        }
    }
}
