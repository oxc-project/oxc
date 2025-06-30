use oxc_ast::ast::*;

use crate::{
    format_args,
    formatter::{
        Buffer, Comments, Format, FormatError, FormatResult, Formatter,
        buffer::RemoveSoftLinesBuffer, prelude::*, trivia::format_trailing_comments,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::FormatTrailingCommas,
    utils::assignment_like::AssignmentLikeLayout,
    write,
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
    pub body_cache_mode: FunctionBodyCacheMode,
}

#[derive(Debug, Clone, Copy)]
pub enum GroupedCallArgumentLayout {
    /// Group the first call argument.
    GroupedFirstArgument,

    /// Group the last call argument.
    GroupedLastArgument,
}

#[derive(Default, Clone, Copy)]
pub enum FunctionBodyCacheMode {
    /// Format the body without caching it or retrieving it from the cache.
    #[default]
    NoCache,

    /// The body has been cached before, try to retrieve the body from the cache.
    Cached,

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
                            format_signature(arrow, self.options.call_arg_layout.is_some(), true),
                            space(),
                            "=>"
                        ]
                    )
                });

                let format_body = FormatMaybeCachedFunctionBody {
                    body,
                    expression: arrow.expression(),
                    mode: self.options.body_cache_mode,
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
                let body_has_soft_line_break = match arrow.get_expression() {
                    None
                    | Some(
                        Expression::ArrowFunctionExpression(_)
                        | Expression::ArrayExpression(_)
                        | Expression::ObjectExpression(_),
                    ) => {
                        // TODO: It seems no difference whether check there is a leading comment or not.
                        // !f.comments().has_leading_own_line_comment(body.span().start)
                        true
                    }
                    _ => false,
                };
                // TODO:
                // let body_has_soft_line_break = match &body {
                // AnyJsExpression(JsxTagExpression(_)) => true,
                // AnyJsExpression(JsTemplateExpression(template)) => {
                // is_multiline_template_starting_on_same_line(template)
                // }
                // AnyJsExpression(JsSequenceExpression(sequence)) => {
                // let has_comment = f.context().comments().has_comments(sequence.syntax());
                // if has_comment {
                // return write!(
                // f,
                // [group(&format_args![
                // formatted_signature,
                // group(&format_args!(indent(&format_args!(
                // hard_line_break(),
                // "(",
                // soft_block_indent(&format_body),
                // ")"
                // ))))
                // ])]
                // );
                // }
                // return write!(
                // f,
                // group(&format_args!(
                // formatted_signature,
                // group(&format_args!(
                // space(),
                // "(",
                // soft_block_indent(&format_body),
                // ")"
                // ))
                // ))
                // );
                // }
                // _ => false,
                // };
                // TODO
                let body_is_condition_type = false; // matches!(body, AnyJsExpression(JsConditionalExpression(_)));
                if body_has_soft_line_break {
                    write!(f, [formatted_signature, space(), format_body])
                } else {
                    let should_add_parens = should_add_parens(body);

                    let is_last_call_arg = matches!(
                        self.options.call_arg_layout,
                        Some(GroupedCallArgumentLayout::GroupedLastArgument)
                    );

                    let should_add_soft_line = (
                        is_last_call_arg
                        // if it's inside a JSXExpression (e.g. an attribute) we should align the expression's closing } with the line with the opening {.
                        /*|| matches!(node.syntax().parent.kind(), Some(JsSyntaxKind::JSX_EXPRESSION_CHILD | JsSyntaxKind::JSX_EXPRESSION_ATTRIBUTE_VALUE))*/
                    );
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

        let has_parameters = parameters.has_parameter();
        // TODO
        // let has_parameters = match &parameters {
        // AnyJsArrowFunctionParameters::AnyJsBinding(_) => true,
        // AnyJsArrowFunctionParameters::JsParameters(parameters) => {
        // // This matches Prettier, which allows type annotations when
        // // grouping arrow expressions, but disallows them when grouping
        // // normal function expressions.
        // if !has_only_simple_parameters(parameters, true) {
        // return Ok(true);
        // }
        // !parameters.items().is_empty()
        // }
        // };

        let has_type_and_parameters = arrow.return_type.is_some() && has_parameters;

        has_type_and_parameters || has_rest_object_or_array_parameter(parameters)
    }
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

        // let head_parent = head.syntax().parent;
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
        // TODO
        let is_callee = false;
        // let is_callee = head_parent.as_ref().is_some_and(|parent| {
        // matches!(
        // parent.kind(),
        // JsSyntaxKind::JS_CALL_EXPRESSION | JsSyntaxKind::JS_NEW_EXPRESSION
        // )
        // });

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
        let body_on_separate_line = false;
        // !matches!(
        // tail_body,
        // AnyJsFunctionBody::JsFunctionBody(_)
        // | AnyJsFunctionBody::AnyJsExpression(
        // AnyJsExpression::JsObjectExpression(_)
        // | AnyJsExpression::JsArrayExpression(_)
        // | AnyJsExpression::JsSequenceExpression(_)
        // | AnyJsExpression::JsxTagExpression(_)
        // )
        // );

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
                    // TODO: maybe this is unneeded in the current oxc implementation?
                    // let should_format_comments = !is_first_in_chain
                    // && f.context().comments().has_leading_comments(arrow.syntax());
                    let should_format_comments = false;
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

                        write!(f, [format_signature(arrow, is_grouped_call_arg_layout, is_first)])
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

        // TODO
        let has_comment = false; //
        // matches!(
        // &tail_body,
        // AnyJsFunctionBody::AnyJsExpression(AnyJsExpression::JsSequenceExpression(sequence))
        // if f.context().comments().has_comments(sequence.syntax())
        // );

        let format_tail_body_inner = format_with(|f| {
            let format_tail_body = FormatMaybeCachedFunctionBody {
                body: tail_body,
                expression: tail.expression(),
                mode: self.options.body_cache_mode,
            };

            // Ensure that the parens of sequence expressions end up on their own line if the
            // body breaks
            // if matches!(
            // tail_body,
            // AnyJsFunctionBody::AnyJsExpression(AnyJsExpression::JsSequenceExpression(_))
            // ) {
            // if has_comment {
            // write!(
            // f,
            // group(&format_args!(indent(&format_args!(
            // hard_line_break(),
            // text("("),
            // soft_block_indent(&format_tail_body),
            // text(")")
            // ))))
            // )?;
            // } else {
            // write!(
            // f,
            // group(&format_args!("(", soft_block_indent(&format_tail_body), ")"))
            // )?;
            // }
            // } else {
            // let should_add_parens = should_add_parens(&tail_body);
            // if should_add_parens {
            // write!(
            // f,
            // [
            // if_group_fits_on_line(&text("(")),
            // format_tail_body,
            // if_group_fits_on_line(&text(")"))
            // ]
            // )?;
            // } else {
            write!(f, format_tail_body)?;
            // }
            // }

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
            let should_add_soft_line = false; //matches!(
            // head_parent.kind(),
            // Some(
            // JsSyntaxKind::JSX_EXPRESSION_CHILD
            // | JsSyntaxKind::JSX_EXPRESSION_ATTRIBUTE_VALUE
            // )
            // );

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

fn should_add_parens(body: &FunctionBody) -> bool {
    // TODO
    false
    // Add parentheses to avoid confusion between `a => b ? c : d` and `a <= b ? c : d`
    // but only if the body isn't an object/function or class expression because parentheses are always required in that
    // case and added by the object expression itself
    // match &body {
    // AnyJsFunctionBody::AnyJsExpression(
    // expression @ AnyJsExpression::JsConditionalExpression(_),
    // ) => {
    // let var_name = matches!(
    // AnyJsExpressionLeftSide::leftmost(expression.clone()),
    // AnyJsExpressionLeftSide::AnyJsExpression(
    // AnyJsExpression::JsObjectExpression(_)
    // | AnyJsExpression::JsFunctionExpression(_)
    // | AnyJsExpression::JsClassExpression(_)
    // )
    // );
    // let are_parentheses_mandatory = var_name;

    // !are_parentheses_mandatory
    // }
    // _ => false,
    // }
}

fn has_rest_object_or_array_parameter(params: &FormalParameters) -> bool {
    params.rest.is_some()
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
) -> impl Format<'a> + 'b {
    format_with(move |f| {
        let formatted_async_token =
            format_with(|f| if arrow.r#async() { write!(f, ["async", space()]) } else { Ok(()) });

        let formatted_parameters = format_with(|f| {
            write!(f, arrow.type_parameters())?;

            // match arrow.params {
            // AnyJsArrowFunctionParameters::AnyJsBinding(binding) => {
            // let should_hug =
            // is_test_call_argument(arrow.syntax())? || is_first_or_last_call_argument;
            // let parentheses_not_needed = can_avoid_parentheses(arrow, f);

            // if !parentheses_not_needed {
            // write!(f, "(")?;
            // }

            // if should_hug || parentheses_not_needed {
            // write!(f, [binding.format()])?;
            // } else {
            // write!(
            // f,
            // [&soft_block_indent(&format_args![binding, FormatTrailingCommas::All])]
            // )?
            // }

            // if !parentheses_not_needed {
            // write!(f, ")")?;
            // }
            // }
            // AnyJsArrowFunctionParameters::JsParameters(params) => {
            write!(f, arrow.params())?;
            // }
            // };

            Ok(())
        });

        let format_return_type = format_with(|f| {
            if let Some(return_type) = &arrow.return_type() {
                write!(f, return_type)?;
            }
            Ok(())
        });

        if is_first_or_last_call_argument {
            let mut buffer = RemoveSoftLinesBuffer::new(f);
            let mut recording = buffer.start_recording();

            write!(
                recording,
                [group(&format_args!(
                    maybe_space(!is_first_in_chain),
                    formatted_async_token,
                    group(&formatted_parameters),
                    group(&format_return_type)
                ))]
            )?;

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
                    group(&format_args!(
                        formatted_async_token,
                        formatted_parameters,
                        group(&format_return_type)
                    ))
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
            FunctionBodyCacheMode::Cached => {
                match f.context().get_cached_function_body(self.body) {
                    Some(cached) => f.write_element(cached),
                    None => {
                        // This can happen in the unlikely event where a function has a parameter with
                        // an initializer that contains a call expression with a first or last function/arrow
                        // ```javascript
                        // test((
                        //   problematic = test(() => body)
                        // ) => {});
                        // ```
                        // This case should be rare as it requires very specific syntax (and is rather messy to write)
                        // which is why it's fine to just fallback to formatting the body again in this case.
                        self.format(f)
                    }
                }
            }
            FunctionBodyCacheMode::Cache => match f.intern(&self.body)? {
                Some(interned) => {
                    f.context_mut().set_cached_function_body(self.body, interned.clone());
                    f.write_element(interned)
                }
                None => Ok(()),
            },
        }
    }
}
