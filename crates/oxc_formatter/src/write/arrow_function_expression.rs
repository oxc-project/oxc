use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    format_args,
    formatter::{
        Buffer, Comments, Format, FormatError, FormatResult, Formatter,
        buffer::RemoveSoftLinesBuffer, comments::has_new_line_backward, prelude::*,
        trivia::format_trailing_comments, write,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::FormatTrailingCommas,
    utils::assignment_like::AssignmentLikeLayout,
    write,
    write::parameter_list::has_only_simple_parameters,
};

#[derive(Clone, Copy)]
pub struct FormatJsArrowFunctionExpression<'a, 'b> {
    arrow: &'b AstNode<'a, ArrowFunctionExpression<'a>>,
    options: FormatJsArrowFunctionExpressionOptions,
}

#[derive(Default, Clone, Copy)]
pub struct FormatJsArrowFunctionExpressionOptions {
    pub assignment_layout: Option<AssignmentLikeLayout>,
    pub call_arg_layout: Option<GroupedCallArgumentLayout>,
    // Determine whether the signature and body should be cached.
    pub cache_mode: FunctionBodyCacheMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupedCallArgumentLayout {
    /// Group the first call argument.
    GroupedFirstArgument,

    /// Group the last call argument.
    GroupedLastArgument,
}

impl GroupedCallArgumentLayout {
    pub fn is_grouped_first(self) -> bool {
        matches!(self, GroupedCallArgumentLayout::GroupedFirstArgument)
    }

    pub fn is_grouped_last(self) -> bool {
        matches!(self, GroupedCallArgumentLayout::GroupedLastArgument)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum FunctionBodyCacheMode {
    /// Format the body without caching it or retrieving it from the cache.
    #[default]
    NoCache,

    /// Cache the body during the next [formatting](Format::fmt).
    Cache,
}

impl<'a, 'b> FormatJsArrowFunctionExpression<'a, 'b> {
    pub fn new(arrow: &'b AstNode<'a, ArrowFunctionExpression<'a>>) -> Self {
        Self { arrow, options: FormatJsArrowFunctionExpressionOptions::default() }
    }

    pub fn new_with_options(
        arrow: &'b AstNode<'a, ArrowFunctionExpression<'a>>,
        options: FormatJsArrowFunctionExpressionOptions,
    ) -> Self {
        Self { arrow, options }
    }
}

impl<'a> Format<'a> for FormatJsArrowFunctionExpression<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let layout =
            ArrowFunctionLayout::for_arrow(self.arrow, f.context().comments(), self.options);

        match layout {
            ArrowFunctionLayout::Chain(chain) => {
                write!(f, chain)
            }
            ArrowFunctionLayout::Single(arrow) => {
                let body = &arrow.body();

                let formatted_signature = format_with(|f| {
                    write!(
                        f,
                        [
                            format_signature(
                                arrow,
                                self.options.call_arg_layout.is_some(),
                                true,
                                self.options.cache_mode
                            ),
                            space(),
                            "=>"
                        ]
                    )
                });

                let format_body = FormatMaybeCachedFunctionBody {
                    body,
                    expression: arrow.expression(),
                    mode: self.options.cache_mode,
                };

                // With arrays, arrow self and objects, they have a natural line breaking strategy:
                // Arrays and objects become blocks:
                //
                //    [
                //      100000,
                //      200000,
                //      300000
                //    ]
                //
                // Arrow self get line broken after the `=>`:
                //
                //  (foo) => (bar) =>
                //     (foo + bar) * (foo + bar)
                //
                // Therefore if our body is an arrow self, array, or object, we
                // do not have a soft line break after the arrow because the body is
                // going to get broken anyways.
                let arrow_expression = arrow.get_expression();

                if let Some(Expression::SequenceExpression(sequence)) = arrow_expression {
                    return if f.context().comments().has_comments_before(sequence.span().start) {
                        write!(
                            f,
                            [group(&format_args!(
                                formatted_signature,
                                group(&format_args!(indent(&format_args!(
                                    hard_line_break(),
                                    text("("),
                                    soft_block_indent(&format_body),
                                    text(")")
                                ))))
                            ))]
                        )
                    } else {
                        write!(
                            f,
                            [group(&format_args!(
                                formatted_signature,
                                group(&format_args!(
                                    space(),
                                    text("("),
                                    soft_block_indent(&format_body),
                                    text(")")
                                ))
                            ))]
                        )
                    };
                }

                #[expect(clippy::match_same_arms)]
                let body_has_soft_line_break = arrow_expression.is_none_or(|expression| {
                    match expression {
                        Expression::ArrowFunctionExpression(_)
                        | Expression::ArrayExpression(_)
                        | Expression::ObjectExpression(_) => {
                            // TODO: It seems no difference whether check there is a leading comment or not.
                            // !f.comments().has_leading_own_line_comment(body.span().start)
                            true
                        }
                        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
                        Expression::TemplateLiteral(template) => {
                            is_multiline_template_starting_on_same_line(
                                template.span.start,
                                template,
                                f.source_text(),
                            )
                        }
                        Expression::TaggedTemplateExpression(template) => {
                            is_multiline_template_starting_on_same_line(
                                template.span.start,
                                &template.quasi,
                                f.source_text(),
                            )
                        }
                        _ => false,
                    }
                });

                let body_is_condition_type =
                    matches!(arrow_expression, Some(Expression::ConditionalExpression(_)));
                if body_has_soft_line_break {
                    write!(f, [formatted_signature, space(), format_body])
                } else {
                    let should_add_parens = arrow.expression && should_add_parens(body);

                    let is_last_call_arg = matches!(
                        self.options.call_arg_layout,
                        Some(GroupedCallArgumentLayout::GroupedLastArgument)
                    );

                    let should_add_soft_line = (is_last_call_arg
                        // if it's inside a JSXExpression (e.g. an attribute) we should align the expression's closing } with the line with the opening {.
                        || matches!(self.arrow.parent, AstNodes::JSXExpressionContainer(_)));
                    // TODO: it seems no difference whether check there is a comment or not.
                    //&& !f.context().comments().has_comments(node.syntax());

                    if body_is_condition_type {
                        write!(
                            f,
                            [
                                formatted_signature,
                                group(&format_args!(
                                    soft_line_indent_or_hard_space(&format_with(|f| {
                                        if should_add_parens {
                                            write!(f, if_group_fits_on_line(&"("))?;
                                        }

                                        write!(f, format_body)?;

                                        if should_add_parens {
                                            write!(f, if_group_fits_on_line(&")"))?;
                                        }

                                        Ok(())
                                    })),
                                    is_last_call_arg
                                        .then_some(format_args!(FormatTrailingCommas::All,)),
                                    should_add_soft_line.then_some(format_args!(soft_line_break()))
                                ))
                            ]
                        )
                    } else {
                        write!(
                            f,
                            [
                                formatted_signature,
                                group(&format_args!(
                                    soft_line_indent_or_space(&format_with(|f| {
                                        if should_add_parens {
                                            write!(f, if_group_fits_on_line(&"("))?;
                                        }

                                        write!(f, format_body)?;

                                        if should_add_parens {
                                            write!(f, if_group_fits_on_line(&")"))?;
                                        }

                                        Ok(())
                                    })),
                                    is_last_call_arg
                                        .then_some(format_args!(FormatTrailingCommas::All,)),
                                    should_add_soft_line.then_some(format_args!(soft_line_break()))
                                ))
                            ]
                        )
                    }
                }
            }
        }
    }
}

enum ArrowFunctionLayout<'a, 'b> {
    /// Arrow function with a non-arrow function body
    Single(&'b AstNode<'a, ArrowFunctionExpression<'a>>),

    /// A chain of at least two arrow functions.
    ///
    /// An arrow function is part of the chain when it is the body of the parent arrow function.
    ///
    /// The idea of arrow chains is that they break after the `=>` token
    ///
    /// ```javascript
    /// const x =
    ///   (a): string =>
    ///   (b) =>
    ///   (c) =>
    ///   (d) =>
    ///   (e) =>
    ///     f;
    /// ```
    Chain(ArrowChain<'a, 'b>),
}

impl<'a, 'b> ArrowFunctionLayout<'a, 'b> {
    /// Determines the layout for the passed arrow function. See [ArrowFunctionLayout] for a description
    /// of the different layouts.
    fn for_arrow(
        arrow: &'b AstNode<'a, ArrowFunctionExpression<'a>>,
        comments: &Comments<'a>,
        options: FormatJsArrowFunctionExpressionOptions,
    ) -> ArrowFunctionLayout<'a, 'b> {
        let mut head = None;
        let mut middle = Vec::new();
        let mut current = arrow;
        let mut should_break = false;

        loop {
            if current.expression() {
                if let Some(AstNodes::ExpressionStatement(expr_stmt)) =
                    current.body().statements().first().map(AstNode::<Statement>::as_ast_nodes)
                {
                    if let AstNodes::ArrowFunctionExpression(next) =
                        &expr_stmt.expression().as_ast_nodes()
                    {
                        if matches!(
                            options.call_arg_layout,
                            None | Some(GroupedCallArgumentLayout::GroupedLastArgument)
                        )
                        // TODO: Unsupported yet
                        //  && !comments.is_suppressed(next.span())
                        {
                            should_break = should_break || Self::should_break_chain(current);

                            should_break = should_break || Self::should_break_chain(next);

                            if head.is_none() {
                                head = Some(current);
                            } else {
                                middle.push(current);
                            }

                            current = next;
                            continue;
                        }
                    }
                }
            }
            break match head {
                None => ArrowFunctionLayout::Single(current),
                Some(head) => ArrowFunctionLayout::Chain(ArrowChain {
                    head,
                    middle,
                    tail: current,
                    expand_signatures: should_break,
                    options,
                }),
            };
        }
    }

    /// Returns a `true` result if the arrow function contains any elements which
    /// should force the chain to break onto multiple lines. This includes any kind
    /// of return type annotation if the function also takes parameters (e.g.,
    /// `(a, b): bool => ...`), any kind of rest/object/array binding parameter
    /// (e.g., `({a, b: foo}) => ...`), and any kind of initializer for a parameter
    /// (e.g., `(a = 2) => ...`).
    ///
    /// The complexity of these expressions limits their legibility when printed
    /// inline, so they force the chain to break to preserve clarity. Any other
    /// cases are considered simple enough to print in a single line.
    fn should_break_chain(arrow: &ArrowFunctionExpression<'a>) -> bool {
        if arrow.type_parameters.is_some() {
            return true;
        }

        let parameters = &arrow.params;

        // This matches Prettier, which allows type annotations when
        // grouping arrow expressions, but disallows them when grouping
        // normal function expressions.
        if !has_only_simple_parameters(parameters, true) {
            return true;
        }

        let has_parameters = parameters.items.is_empty();
        let has_type_and_parameters = arrow.return_type.is_some() && has_parameters;
        has_type_and_parameters || has_rest_object_or_array_parameter(parameters)
    }
}

/// Returns `true` for a template that starts on the same line as the previous token and contains a line break.
///
///
/// # Examples
///
/// ```javascript
/// "test" + `
///   some content
/// `;
/// ```
///
/// Returns `true` because the template starts on the same line as the `+` token and its text contains a line break.
///
/// ```javascript
/// "test" + `no line break`
/// ```
///
/// Returns `false` because the template text contains no line break.
///
/// ```javascript
/// "test" +
///     `template
///     with line break`;
/// ```
///
/// Returns `false` because the template isn't on the same line as the '+' token.
pub fn is_multiline_template_starting_on_same_line(
    start: u32,
    template: &TemplateLiteral,
    source_text: &str,
) -> bool {
    template.quasis.iter().any(|quasi| quasi.value.raw.contains('\n'))
        && !has_new_line_backward(&source_text[..start as usize])
}

struct ArrowChain<'a, 'b> {
    /// The top most arrow function in the chain
    head: &'b AstNode<'a, ArrowFunctionExpression<'a>>,

    /// The arrow functions in the chain that are neither the first nor the last.
    /// Empty for chains consisting only of two arrow functions.
    middle: Vec<&'b AstNode<'a, ArrowFunctionExpression<'a>>>,

    /// The last arrow function in the chain
    tail: &'b AstNode<'a, ArrowFunctionExpression<'a>>,

    options: FormatJsArrowFunctionExpressionOptions,

    /// Whether the group wrapping the signatures should be expanded or not.
    expand_signatures: bool,
}

impl<'a, 'b> ArrowChain<'a, 'b> {
    /// Returns an iterator over all arrow functions in this chain
    fn arrows(&self) -> impl Iterator<Item = &&'b AstNode<'a, ArrowFunctionExpression<'a>>> {
        use std::iter::once;
        once(&self.head).chain(self.middle.iter()).chain(once(&self.tail))
    }
}

impl<'a> Format<'a> for ArrowChain<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let ArrowChain { tail, expand_signatures, .. } = self;

        let head_parent = self.head.parent;
        let tail_body = tail.body();
        let is_assignment_rhs = self.options.assignment_layout.is_some();
        let is_grouped_call_arg_layout = self.options.call_arg_layout.is_some();

        // If this chain is the callee in a parent call expression, then we
        // want it to break onto a new line to clearly show that the arrow
        // chain is distinct and the _result_ is what's being called.
        // Example:
        //      (() => () => a)()
        // becomes
        //      (
        //        () => () =>
        //          a
        //      )();
        let is_callee =
            matches!(head_parent, AstNodes::CallExpression(_) | AstNodes::NewExpression(_));

        // With arrays, objects, sequence expressions, and block function bodies,
        // the opening brace gives a convenient boundary to insert a line break,
        // allowing that token to live immediately after the last arrow token
        // and save a line from being printed with just the punctuation.
        //
        // (foo) => (bar) => [a, b]
        //
        // (foo) => (bar) => [
        //   a,
        //   b
        // ]
        //
        // If the body is _not_ one of those kinds, then we'll want to insert a
        // soft line break before the body so that it prints on a separate line
        // in its entirety.
        let body_on_separate_line = !tail.get_expression().is_none_or(|expression| {
            matches!(
                expression,
                Expression::ObjectExpression(_)
                    | Expression::ArrayExpression(_)
                    | Expression::SequenceExpression(_)
                    | Expression::JSXElement(_)
                    | Expression::JSXFragment(_)
            )
        });

        // If the arrow chain will break onto multiple lines, either because
        // it's a callee or because the body is printed on its own line, then
        // the signatures should be expanded first.
        let break_signatures = (is_callee && body_on_separate_line)
            || matches!(
                self.options.assignment_layout,
                Some(AssignmentLikeLayout::ChainTailArrowFunction)
            );

        // Arrow chains as callees or as the right side of an assignment
        // indent the entire signature chain a single level and do _not_
        // indent a second level for additional signatures after the first:
        //   const foo =
        //     (a) =>
        //     (b) =>
        //     (c) =>
        //       0;
        // This tracks that state and is used to prevent the insertion of
        // additional indents under `format_arrow_signatures`, then also to
        // add the outer indent under `format_inner`.
        let has_initial_indent = is_callee || is_assignment_rhs;

        let format_arrow_signatures = format_with(|f| {
            let join_signatures = format_with(|f| {
                let mut is_first_in_chain = true;
                for arrow in self.arrows() {
                    // The first comment in the chain gets formatted by the
                    // parent (the FormatJsArrowFunctionExpression), but the
                    // rest of the arrows in the chain need to format their
                    // comments manually, since they won't have their own
                    // Format node to handle it.
                    let should_format_comments = !is_first_in_chain
                        && f.context().comments().has_comments_before(arrow.span.start);
                    let is_first = is_first_in_chain;

                    let formatted_signature = format_with(|f| {
                        if should_format_comments {
                            // A grouped layout implies that the arrow chain is trying to be rendered
                            // in a condensed, single-line format (at least the signatures, not
                            // necessarily the body). In that case, we _need_ to prevent the leading
                            // comments from inserting line breaks. But if it's _not_ a grouped layout,
                            // then we want to _force_ the line break so that the leading comments
                            // don't inadvertently end up on the previous line after the fat arrow.
                            if is_grouped_call_arg_layout {
                                write!(f, [space(), format_leading_comments(arrow.span())])?;
                            } else {
                                write!(
                                    f,
                                    [
                                        soft_line_break_or_space(),
                                        format_leading_comments(arrow.span())
                                    ]
                                )?;
                            }
                        }

                        write!(
                            f,
                            [format_signature(
                                arrow,
                                is_grouped_call_arg_layout,
                                is_first,
                                self.options.cache_mode
                            )]
                        )
                    });

                    // Arrow chains indent a second level for every item other than the first:
                    //   (a) =>
                    //     (b) =>
                    //     (c) =>
                    //       0
                    // Because the chain is printed as a flat list, each entry needs to set
                    // its own indention. This ensures that the first item keeps the same
                    // level as the surrounding content, and then each subsequent item has
                    // one additional level, as shown above.
                    if is_first_in_chain || has_initial_indent {
                        is_first_in_chain = false;
                        write!(f, [formatted_signature])?;
                    } else {
                        write!(f, [indent(&formatted_signature)])?;
                    }

                    // The arrow of the tail is formatted outside of the group to ensure it never
                    // breaks from the body
                    if !std::ptr::eq(arrow, tail) {
                        write!(f, [space(), "=>"])?;
                    }
                }

                Ok(())
            });

            write!(f, [group(&join_signatures).should_expand(*expand_signatures)])
        });

        let format_tail_body_inner = format_with(|f| {
            let format_tail_body = FormatMaybeCachedFunctionBody {
                body: tail_body,
                expression: tail.expression(),
                mode: self.options.cache_mode,
            };

            // Ensure that the parens of sequence expressions end up on their own line if the
            // body breaks
            if let Some(Expression::SequenceExpression(sequence)) = tail.get_expression() {
                if f.context().comments().has_comments_before(sequence.span().start) {
                    write!(
                        f,
                        [group(&format_args!(indent(&format_args!(
                            hard_line_break(),
                            text("("),
                            soft_block_indent(&format_tail_body),
                            text(")")
                        ))))]
                    )?;
                } else {
                    write!(
                        f,
                        [group(&format_args!(
                            text("("),
                            soft_block_indent(&format_tail_body),
                            text(")")
                        ))]
                    )?;
                }
            } else {
                let should_add_parens = tail.expression && should_add_parens(tail_body);
                if should_add_parens {
                    write!(
                        f,
                        [
                            if_group_fits_on_line(&text("(")),
                            format_tail_body,
                            if_group_fits_on_line(&text(")"))
                        ]
                    )?;
                } else {
                    write!(f, [format_tail_body])?;
                }
            }

            // Format the trailing comments of all arrow function EXCEPT the first one because
            // the comments of the head get formatted as part of the `FormatJsArrowFunctionExpression` call.
            // TODO: It seems unneeded in the current oxc implementation?
            // for arrow in self.arrows().skip(1) {
            //     write!(f, format_trailing_comments(arrow.span().end))?;
            // }

            Ok(())
        });

        let format_tail_body = format_with(|f| {
            // if it's inside a JSXExpression (e.g. an attribute) we should align the expression's closing } with the line with the opening {.
            let should_add_soft_line = matches!(head_parent, AstNodes::JSXExpressionContainer(_));

            if body_on_separate_line {
                write!(
                    f,
                    [
                        indent(&format_args!(soft_line_break_or_space(), format_tail_body_inner)),
                        should_add_soft_line.then_some(soft_line_break())
                    ]
                )
            } else {
                write!(f, [space(), format_tail_body_inner])
            }
        });

        let group_id = f.group_id("arrow-chain");

        let format_inner = format_once(|f| {
            if has_initial_indent {
                write!(
                    f,
                    [group(&indent(&format_args!(soft_line_break(), format_arrow_signatures)))
                        .with_group_id(Some(group_id))
                        .should_expand(break_signatures)]
                )?;
            } else {
                write!(
                    f,
                    group(&format_arrow_signatures)
                        .with_group_id(Some(group_id))
                        .should_expand(break_signatures)
                )?;
            }

            write!(f, [space(), "=>"])?;

            if is_grouped_call_arg_layout {
                write!(f, [group(&format_tail_body)])?;
            } else {
                write!(f, [indent_if_group_breaks(&format_tail_body, group_id)])?;
            }

            if is_callee {
                write!(f, [if_group_breaks(&soft_line_break()).with_group_id(Some(group_id))])?;
            }

            Ok(())
        });

        write!(f, [group(&format_inner)])
    }
}

#[derive(Debug)]
pub enum ExpressionLeftSide<'a, 'b> {
    Expression(&'b Expression<'a>),
    AssignmentTarget(&'b AssignmentTarget<'a>),
    SimpleAssignmentTarget(&'b SimpleAssignmentTarget<'a>),
}

impl<'a, 'b> From<&'b Expression<'a>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: &'b Expression<'a>) -> Self {
        Self::Expression(value)
    }
}

impl<'a, 'b> From<&'b AssignmentTarget<'a>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: &'b AssignmentTarget<'a>) -> Self {
        Self::AssignmentTarget(value)
    }
}

impl<'a, 'b> From<&'b SimpleAssignmentTarget<'a>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: &'b SimpleAssignmentTarget<'a>) -> Self {
        Self::SimpleAssignmentTarget(value)
    }
}

impl<'a, 'b> ExpressionLeftSide<'a, 'b> {
    pub fn leftmost(expression: &'b Expression<'a>) -> Self {
        let mut current: Self = expression.into();
        loop {
            match current.left_expression() {
                None => {
                    break current;
                }
                Some(left) => {
                    current = left;
                }
            }
        }
    }

    /// Returns the left side of an expression (an expression where the first child is a `Node` or [None]
    /// if the expression has no left side.
    pub fn left_expression(&self) -> Option<Self> {
        match self {
            Self::Expression(expression) => match expression {
                Expression::SequenceExpression(expr) => expr.expressions.first().map(Into::into),
                Expression::StaticMemberExpression(expr) => Some((&expr.object).into()),
                Expression::ComputedMemberExpression(expr) => Some((&expr.object).into()),
                Expression::PrivateFieldExpression(expr) => Some((&expr.object).into()),
                Expression::TaggedTemplateExpression(expr) => Some((&expr.tag).into()),
                Expression::NewExpression(expr) => Some((&expr.callee).into()),
                Expression::CallExpression(expr) => Some((&expr.callee).into()),
                Expression::ConditionalExpression(expr) => Some((&expr.test).into()),
                Expression::TSAsExpression(expr) => Some((&expr.expression).into()),
                Expression::TSSatisfiesExpression(expr) => Some((&expr.expression).into()),
                Expression::TSNonNullExpression(expr) => Some((&expr.expression).into()),
                Expression::AssignmentExpression(expr) => Some(Self::AssignmentTarget(&expr.left)),
                Expression::UpdateExpression(expr) => {
                    if expr.prefix {
                        None
                    } else {
                        Some(Self::SimpleAssignmentTarget(&expr.argument))
                    }
                }
                Expression::BinaryExpression(binary) => Some((&binary.left).into()),
                Expression::LogicalExpression(logical) => Some((&logical.left).into()),
                Expression::ChainExpression(chain) => match &chain.expression {
                    ChainElement::CallExpression(expr) => Some((&expr.callee).into()),
                    ChainElement::TSNonNullExpression(expr) => Some((&expr.expression).into()),
                    ChainElement::ComputedMemberExpression(expr) => Some((&expr.object).into()),
                    ChainElement::StaticMemberExpression(expr) => Some((&expr.object).into()),
                    ChainElement::PrivateFieldExpression(expr) => Some((&expr.object).into()),
                },
                _ => None,
            },
            Self::AssignmentTarget(target) => match target {
                match_simple_assignment_target!(AssignmentTarget) => {
                    Self::SimpleAssignmentTarget(target.to_simple_assignment_target())
                        .left_expression()
                }
                _ => None,
            },
            Self::SimpleAssignmentTarget(target) => match target {
                SimpleAssignmentTarget::TSAsExpression(expr) => Some((&expr.expression).into()),
                SimpleAssignmentTarget::TSSatisfiesExpression(expr) => {
                    Some((&expr.expression).into())
                }
                SimpleAssignmentTarget::TSNonNullExpression(expr) => {
                    Some((&expr.expression).into())
                }
                SimpleAssignmentTarget::TSTypeAssertion(expr) => Some((&expr.expression).into()),
                SimpleAssignmentTarget::ComputedMemberExpression(expr) => {
                    Some((&expr.object).into())
                }
                SimpleAssignmentTarget::StaticMemberExpression(expr) => Some((&expr.object).into()),
                SimpleAssignmentTarget::PrivateFieldExpression(expr) => Some((&expr.object).into()),
                SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier_reference) => None,
            },
        }
    }

    pub fn span(&self) -> Span {
        match self {
            ExpressionLeftSide::Expression(expression) => expression.span(),
            ExpressionLeftSide::AssignmentTarget(target) => target.span(),
            ExpressionLeftSide::SimpleAssignmentTarget(target) => target.span(),
        }
    }
}

fn should_add_parens(body: &FunctionBody) -> bool {
    let Statement::ExpressionStatement(stmt) = body.statements.first().unwrap() else {
        unreachable!()
    };

    // Add parentheses to avoid confusion between `a => b ? c : d` and `a <= b ? c : d`
    // but only if the body isn't an object/function or class expression because parentheses are always required in that
    // case and added by the object expression itself
    if matches!(&stmt.expression, Expression::ConditionalExpression(_)) {
        !matches!(
            ExpressionLeftSide::leftmost(&stmt.expression),
            ExpressionLeftSide::Expression(
                Expression::ObjectExpression(_)
                    | Expression::FunctionExpression(_)
                    | Expression::ClassExpression(_)
            )
        )
    } else {
        false
    }
}

fn has_rest_object_or_array_parameter(params: &FormalParameters) -> bool {
    params.rest.is_some()
        || params.items.iter().any(|param| param.pattern.kind.is_destructuring_pattern())
}

/// Writes the arrow function type parameters, parameters, and return type annotation.
///
/// Formats the parameters and return type annotation without any soft line breaks if `is_first_or_last_call_argument` is `true`
/// so that the parameters and return type are kept on the same line.
///
/// # Errors
///
/// Returns [`FormatError::PoorLayout`] if `is_first_or_last_call_argument` is `true` but the parameters
/// or return type annotation contain any content that forces a [*group to break](FormatElements::will_break).
///
/// This error gets captured by FormatJsCallArguments.
fn format_signature<'a, 'b>(
    arrow: &'b AstNode<'a, ArrowFunctionExpression<'a>>,
    is_first_or_last_call_argument: bool,
    is_first_in_chain: bool,
    cache_mode: FunctionBodyCacheMode,
) -> impl Format<'a> + 'b {
    format_with(move |f| {
        let formatted_async_token =
            format_with(|f| if arrow.r#async() { write!(f, ["async", space()]) } else { Ok(()) });

        let formatted_parameters =
            format_with(|f| write!(f, [arrow.type_parameters(), arrow.params()]));

        let format_return_type = format_with(|f| write!(f, arrow.return_type()));

        let signatures = format_once(|f| {
            write!(
                f,
                [group(&format_args!(
                    maybe_space(!is_first_in_chain),
                    formatted_async_token,
                    group(&formatted_parameters),
                    group(&format_return_type)
                ))]
            )
        });

        // The [`call_arguments`] will format the argument that can be grouped in different ways until
        // find the best layout. So we have to cache the parameters because it never be broken.
        let cached_signature = format_once(|f| {
            if matches!(cache_mode, FunctionBodyCacheMode::NoCache) {
                signatures.fmt(f)
            } else if let Some(grouped) = f.context().get_cached_element(&arrow.params.span) {
                f.write_element(grouped)
            } else {
                if let Ok(Some(grouped)) = f.intern(&signatures) {
                    f.context_mut().cache_element(&arrow.params.span, grouped.clone());
                    f.write_element(grouped.clone());
                }
                Ok(())
            }
        });

        if is_first_or_last_call_argument {
            let mut buffer = RemoveSoftLinesBuffer::new(f);
            let mut recording = buffer.start_recording();

            write!(recording, cached_signature)?;

            if recording.stop().will_break() {
                return Err(FormatError::PoorLayout);
            }
        } else {
            write!(
                f,
                [
                    // This soft break is placed outside of the group to ensure
                    // that the parameter group only tries to write on a single
                    // line and can't break pre-emptively without also causing
                    // the parent (i.e., this ArrowChain) to break first.
                    (!is_first_in_chain).then_some(soft_line_break_or_space()),
                    cached_signature
                ]
            )?;
        }

        // TODO: for case `a = (x: any): x is string /* comment */ => {}`
        // if f.comments().has_dangling_comments(arrow.span()) {
        //     write!(f, [space(), format_dangling_comments(arrow.span())])?;
        // }

        Ok(())
    })
}

/// Formats a function body with additional caching depending on [`mode`](Self::mode).
pub struct FormatMaybeCachedFunctionBody<'a, 'b> {
    /// The body to format.
    pub body: &'b AstNode<'a, FunctionBody<'a>>,

    /// Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    pub expression: bool,

    /// If the body should be cached or if the formatter should try to retrieve it from the cache.
    pub mode: FunctionBodyCacheMode,
}

impl<'a> FormatMaybeCachedFunctionBody<'a, '_> {
    fn format(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.expression {
            if let AstNodes::ExpressionStatement(s) =
                &self.body.statements().first().unwrap().as_ast_nodes()
            {
                return s.expression().fmt(f);
            }
        }
        self.body.fmt(f)
    }
}

impl<'a> Format<'a> for FormatMaybeCachedFunctionBody<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self.mode {
            FunctionBodyCacheMode::NoCache => self.format(f),
            FunctionBodyCacheMode::Cache => {
                if let Some(cached) = f.context().get_cached_element(&self.body.span) {
                    f.write_element(cached)
                } else {
                    match f.intern(&format_once(|f| self.format(f)))? {
                        Some(interned) => {
                            f.context_mut().cache_element(&self.body.span, interned.clone());
                            f.write_element(interned)
                        }
                        None => Ok(()),
                    }
                }
            }
        }
    }
}
