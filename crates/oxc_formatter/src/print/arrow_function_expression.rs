use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Buffer, Format, Formatter, SourceText, buffer::RemoveSoftLinesBuffer, prelude::*,
        trivia::FormatTrailingComments,
    },
    options::FormatTrailingCommas,
    print::function::FormatContentWithCacheMode,
    utils::{
        assignment_like::AssignmentLikeLayout, expression::ExpressionLeftSide,
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
        suppressed::FormatSuppressedNode,
    },
    write,
};

use super::{FormatWrite, parameters::has_only_simple_parameters};

impl<'a> FormatWrite<'a, FormatJsArrowFunctionExpressionOptions>
    for AstNode<'a, ArrowFunctionExpression<'a>>
{
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        FormatJsArrowFunctionExpression::new(self).fmt(f);
    }

    fn write_with_options(
        &self,
        options: FormatJsArrowFunctionExpressionOptions,
        f: &mut Formatter<'_, 'a>,
    ) {
        FormatJsArrowFunctionExpression::new_with_options(self, options).fmt(f);
    }
}

#[derive(Clone, Copy)]
pub struct FormatJsArrowFunctionExpression<'a, 'b> {
    arrow: &'b AstNode<'a, ArrowFunctionExpression<'a>>,
    options: FormatJsArrowFunctionExpressionOptions,
}

#[derive(Default, Clone, Copy)]
pub struct FormatJsArrowFunctionExpressionOptions {
    pub assignment_layout: Option<AssignmentLikeLayout>,
    pub call_argument_layout: Option<GroupedCallArgumentLayout>,
    // Determine whether the signature and body should be cached.
    pub cache_mode: FunctionCacheMode,
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
pub enum FunctionCacheMode {
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

    #[inline]
    pub fn format(&self, f: &mut Formatter<'_, 'a>) {
        let layout = ArrowFunctionLayout::for_arrow(self.arrow, self.options);

        match layout {
            ArrowFunctionLayout::Chain(chain) => {
                write!(f, chain);
            }
            ArrowFunctionLayout::Single(arrow) => {
                let body = &arrow.body();

                let formatted_signature = format_with(|f| {
                    write!(
                        f,
                        [
                            format_signature(
                                arrow,
                                self.options.call_argument_layout.is_some(),
                                true,
                                self.options.cache_mode
                            ),
                            space(),
                            "=>"
                        ]
                    );
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
                    return if let Some(format_sequence) =
                        format_sequence_with_leading_comment(sequence.span(), &format_body, f)
                    {
                        write!(f, [group(&format_args!(formatted_signature, format_sequence))]);
                    } else {
                        write!(
                            f,
                            [group(&format_args!(
                                formatted_signature,
                                group(&format_args!(
                                    space(),
                                    token("("),
                                    soft_block_indent(&format_body),
                                    token(")")
                                ))
                            ))]
                        );
                    };
                }

                write!(f, formatted_signature);

                let body_has_soft_line_break =
                    arrow_expression.is_none_or(|expression| match expression {
                        Expression::ArrowFunctionExpression(_)
                        | Expression::ArrayExpression(_)
                        | Expression::ObjectExpression(_) => {
                            !f.comments().has_leading_own_line_comment(body.span().start)
                        }
                        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
                        _ => {
                            is_multiline_template_starting_on_same_line(expression, f.source_text())
                        }
                    });

                if body_has_soft_line_break {
                    write!(f, [space(), format_body]);
                } else {
                    let should_add_parens = arrow.expression && should_add_parens(body);

                    let is_last_call_arg = matches!(
                        self.options.call_argument_layout,
                        Some(GroupedCallArgumentLayout::GroupedLastArgument)
                    );

                    let should_add_soft_line = is_last_call_arg
                        // if it's inside a JSXExpression (e.g. an attribute) we should align the expression's closing } with the line with the opening {.
                        || (matches!(self.arrow.parent(), AstNodes::JSXExpressionContainer(container)
                            if !f.context().comments().has_comment_in_range(arrow.span.end, container.span.end)));

                    write!(
                        f,
                        group(&format_args!(
                            soft_line_indent_or_space(&format_with(|f| {
                                if should_add_parens {
                                    write!(f, if_group_fits_on_line(&"("));
                                }

                                write!(f, format_body);

                                if should_add_parens {
                                    write!(f, if_group_fits_on_line(&")"));
                                }
                            })),
                            is_last_call_arg.then_some(&FormatTrailingCommas::All),
                            should_add_soft_line.then_some(soft_line_break())
                        ))
                    );
                }
            }
        }
    }
}

impl<'a> Format<'a> for FormatJsArrowFunctionExpression<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        self.format(f);
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
        options: FormatJsArrowFunctionExpressionOptions,
    ) -> ArrowFunctionLayout<'a, 'b> {
        let mut head = None;
        let mut middle = Vec::new();
        let mut current = arrow;
        let mut should_break = false;
        let is_non_grouped_or_grouped_last_argument = matches!(
            options.call_argument_layout,
            None | Some(GroupedCallArgumentLayout::GroupedLastArgument)
        );

        loop {
            if is_non_grouped_or_grouped_last_argument
                && current.expression()
                && let Some(AstNodes::ExpressionStatement(expr_stmt)) =
                    current.body().statements().first().map(AstNode::<Statement>::as_ast_nodes)
                && let AstNodes::ArrowFunctionExpression(next) =
                    &expr_stmt.expression().as_ast_nodes()
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

        let has_parameters = !parameters.items.is_empty();
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
    expression: &Expression,
    source_text: SourceText,
) -> bool {
    let (start, template) = match expression {
        Expression::TemplateLiteral(template) => (template.span.start, template.as_ref()),
        Expression::TaggedTemplateExpression(tagged) => (tagged.span.start, &tagged.quasi),
        _ => return false,
    };

    template.quasis.iter().any(|quasi| source_text.contains_newline(quasi.span))
        && !source_text.has_newline_before(start)
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
    fn arrows(&self) -> impl Iterator<Item = &'b AstNode<'a, ArrowFunctionExpression<'a>>> {
        use std::iter::once;
        once(self.head).chain(self.middle.iter().copied()).chain(once(self.tail))
    }
}

impl<'a> Format<'a> for ArrowChain<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let ArrowChain { tail, expand_signatures, .. } = *self;

        let tail_body = tail.body();
        let is_grouped_call_arg_layout = self.options.call_argument_layout.is_some();

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
        let is_callee = self.head.is_call_like_callee();

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
        let has_initial_indent = is_callee
            || self
                .options
                .assignment_layout
                .is_some_and(|layout| layout != AssignmentLikeLayout::BreakAfterOperator);

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
                        && f.context().comments().has_comment_before(arrow.span.start);
                    let is_first = is_first_in_chain;

                    let formatted_signature = format_with(|f| {
                        let format_leading_comments = format_with(|f| {
                            if should_format_comments {
                                // A grouped layout implies that the arrow chain is trying to be rendered
                                // in a condensed, single-line format (at least the signatures, not
                                // necessarily the body). In that case, we _need_ to prevent the leading
                                // comments from inserting line breaks. But if it's _not_ a grouped layout,
                                // then we want to _force_ the line break so that the leading comments
                                // don't inadvertently end up on the previous line after the fat arrow.
                                if is_grouped_call_arg_layout {
                                    write!(f, [space(), format_leading_comments(arrow.span())]);
                                } else {
                                    write!(
                                        f,
                                        [
                                            soft_line_break_or_space(),
                                            format_leading_comments(arrow.span())
                                        ]
                                    );
                                }
                            }
                        });

                        let start = arrow.span().start;
                        write!(
                            f,
                            [
                                FormatContentWithCacheMode::new(
                                    Span::new(start, start),
                                    format_leading_comments,
                                    self.options.cache_mode,
                                ),
                                format_signature(
                                    arrow,
                                    is_grouped_call_arg_layout,
                                    is_first,
                                    self.options.cache_mode
                                )
                            ]
                        );
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
                        write!(f, [formatted_signature]);
                    } else {
                        write!(f, [indent(&formatted_signature)]);
                    }

                    // The arrow of the tail is formatted outside of the group to ensure it never
                    // breaks from the body
                    if !std::ptr::eq(arrow, tail) {
                        write!(f, [space(), "=>"]);
                    }
                }
            });

            group(&join_signatures).should_expand(expand_signatures).fmt(f);
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
                if let Some(format_sequence) =
                    format_sequence_with_leading_comment(sequence.span(), &format_tail_body, f)
                {
                    write!(f, format_sequence);
                } else {
                    write!(
                        f,
                        [group(&format_args!(
                            token("("),
                            soft_block_indent(&format_tail_body),
                            token(")")
                        ))]
                    );
                }
            } else {
                let should_add_parens = tail.expression && should_add_parens(tail_body);
                if should_add_parens {
                    write!(
                        f,
                        [
                            if_group_fits_on_line(&token("(")),
                            format_tail_body,
                            if_group_fits_on_line(&token(")"))
                        ]
                    );
                } else {
                    write!(f, [format_tail_body]);
                }
            }
        });

        let format_tail_body = format_with(|f| {
            // if it's inside a JSXExpression (e.g. an attribute) we should align the expression's closing } with the line with the opening {.
            let should_add_soft_line =
                matches!(self.head.parent(), AstNodes::JSXExpressionContainer(_));

            if body_on_separate_line {
                write!(
                    f,
                    [
                        soft_line_indent_or_space(&format_tail_body_inner),
                        should_add_soft_line.then_some(soft_line_break())
                    ]
                );
            } else {
                write!(f, [space(), format_tail_body_inner]);
            }
        });

        let group_id = f.group_id("arrow-chain");

        let format_inner = format_with(|f| {
            if has_initial_indent {
                write!(
                    f,
                    [group(&indent(&format_args!(soft_line_break(), format_arrow_signatures)))
                        .with_group_id(Some(group_id))
                        .should_expand(break_signatures)]
                );
            } else {
                write!(
                    f,
                    group(&format_arrow_signatures)
                        .with_group_id(Some(group_id))
                        .should_expand(break_signatures)
                );
            }

            write!(f, [space(), "=>"]);

            if is_grouped_call_arg_layout {
                write!(f, [group(&format_tail_body)]);
            } else {
                write!(f, [indent_if_group_breaks(&format_tail_body, group_id)]);
            }

            if is_callee {
                write!(f, [if_group_breaks(&soft_line_break()).with_group_id(Some(group_id))]);
            }
        });

        write!(f, [group(&format_inner)]);
    }
}

fn should_add_parens(body: &AstNode<'_, FunctionBody<'_>>) -> bool {
    let AstNodes::ExpressionStatement(stmt) = body.statements().first().unwrap().as_ast_nodes()
    else {
        unreachable!()
    };

    // Add parentheses to avoid confusion between `a => b ? c : d` and `a <= b ? c : d`
    // but only if the body isn't an object/function or class expression because parentheses are always required in that
    // case and added by the object expression itself
    if matches!(&stmt.expression, Expression::ConditionalExpression(_)) {
        !matches!(
            ExpressionLeftSide::leftmost(stmt.expression()).as_ref(),
            Expression::ObjectExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::ClassExpression(_)
        )
    } else {
        false
    }
}

fn has_rest_object_or_array_parameter(params: &FormalParameters) -> bool {
    params.rest.is_some()
        || params.items.iter().any(|param| param.pattern.is_destructuring_pattern())
}

/// Writes the arrow function type parameters, parameters, and return type annotation.
///
/// Formats the parameters and return type annotation without any soft line breaks if `is_grouped_call_argument` is `true`
/// so that the parameters and return type are kept on the same line.
fn format_signature<'a, 'b>(
    arrow: &'b AstNode<'a, ArrowFunctionExpression<'a>>,
    is_grouped_call_argument: bool,
    is_first_in_chain: bool,
    cache_mode: FunctionCacheMode,
) -> impl Format<'a> + 'b {
    format_with(move |f| {
        let content = format_with(|f| {
            group(&format_args!(
                maybe_space(!is_first_in_chain),
                arrow.r#async().then_some("async "),
                arrow.type_parameters(),
                arrow.params(),
                arrow.return_type().map(FormatNodeWithoutTrailingComments),
            ))
            .fmt(f);
        });
        let format_head = FormatContentWithCacheMode::new(arrow.params.span, content, cache_mode);

        if is_grouped_call_argument {
            // The first arrow's soft lines have already been removed in the CallArguments.
            if is_first_in_chain {
                write!(f, format_head);
            } else {
                let mut buffer = RemoveSoftLinesBuffer::new(f);
                write!(buffer, format_head);
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
                    format_head
                ]
            );
        }

        // Print comments before the fat arrow (`=>`)
        let comments_before_fat_arrow =
            f.context().comments().comments_before_character(arrow.params.span().end, b'=');
        let content = FormatTrailingComments::Comments(comments_before_fat_arrow);
        write!(f, [FormatContentWithCacheMode::new(arrow.span, content, cache_mode)]);
    })
}

/// Formats a function body with additional caching depending on [`mode`](Self::mode).
pub struct FormatMaybeCachedFunctionBody<'a, 'b> {
    /// The body to format.
    pub body: &'b AstNode<'a, FunctionBody<'a>>,

    /// Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    pub expression: bool,

    /// If the body should be cached or if the formatter should try to retrieve it from the cache.
    pub mode: FunctionCacheMode,
}

impl<'a> Format<'a> for FormatMaybeCachedFunctionBody<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let content = format_with(|f| {
            if self.expression
                && let AstNodes::ExpressionStatement(s) =
                    &self.body.statements().first().unwrap().as_ast_nodes()
            {
                return s.expression().fmt(f);
            }
            self.body.fmt(f);
        });
        FormatContentWithCacheMode::new(self.body.span, content, self.mode).fmt(f);
    }
}

/// Format a sequence expression in an arrow function body that has a leading comment.
///
/// When an arrow function body is a sequence expression (e.g., `() => (a, b, c)`) and has
/// a leading comment, special formatting is needed to place the comment correctly:
///
/// ```js
/// const f = () =>
///   // comment
///   (a, b, c);
/// ```
///
/// Returns `Some(formatter)` if the sequence has a leading comment, `None` otherwise.
/// When `None`, the caller should use normal formatting with `soft_block_indent`.
///
/// Handles `oxfmt-ignore` by preserving original source text when suppressed.
fn format_sequence_with_leading_comment<'a, 'b>(
    sequence_span: Span,
    format_body: &'b impl Format<'a>,
    f: &Formatter<'_, 'a>,
) -> Option<impl Format<'a> + 'b> {
    if !f.comments().has_comment_before(sequence_span.start) {
        return None;
    }

    let is_suppressed = f.comments().is_suppressed(sequence_span.start);

    let format_sequence = format_with(move |f| {
        write!(f, [format_leading_comments(sequence_span), "("]);
        if is_suppressed {
            write!(f, FormatSuppressedNode(sequence_span));
        } else {
            write!(f, format_body);
        }
        write!(f, [")"]);
    });

    Some(format_with(move |f| {
        write!(f, group(&indent(&format_args!(hard_line_break(), format_sequence))));
    }))
}
