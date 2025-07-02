use oxc_allocator::{Address, Vec as ArenaVec};
use oxc_ast::{ast::*, match_expression};
use oxc_span::GetSpan;

use crate::{
    Buffer, Format, FormatResult, FormatTrailingCommas, TrailingSeparator, format_args,
    formatter::{
        BufferExtensions, Comments, FormatElement, FormatError, Formatter, VecBuffer,
        format_element,
        prelude::{
            FormatElements, MemoizeFormat, Tag, empty_line, expand_parent, format_once,
            format_with, get_lines_before, group, soft_block_indent, soft_line_break_or_space,
            space,
        },
        separated::FormatSeparatedIter,
        trivia::{DanglingIndentMode, format_dangling_comments},
        write,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    utils::{
        is_long_curried_call, member_chain::simple_argument::SimpleArgument,
        write_arguments_multi_line,
    },
    write,
    write::{
        arrow_function_expression::is_multiline_template_starting_on_same_line,
        parameter_list::has_only_simple_parameters,
    },
};

use super::{
    array_element_list::can_concisely_print_array_list,
    arrow_function_expression::{
        FormatJsArrowFunctionExpression, FormatJsArrowFunctionExpressionOptions,
        FunctionBodyCacheMode, GroupedCallArgumentLayout,
    },
};

impl<'a> Format<'a> for AstNode<'a, ArenaVec<'a, Argument<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let l_paren_token = "(";
        let r_paren_token = ")";
        if self.is_empty() {
            return write!(
                f,
                [
                    l_paren_token,
                    // `call/* comment1 */(/* comment2 */)` Both comments are dangling comments.
                    format_dangling_comments(self.parent.span()).with_soft_block_indent(),
                    r_paren_token
                ]
            );
        }

        let (is_commonjs_or_amd_call, is_test_call) =
            if let AstNodes::CallExpression(call) = self.parent {
                (is_commonjs_or_amd_call(self, call), is_test_call_expression(call))
            } else {
                (false, false)
            };

        let is_first_arg_string_literal_or_template = self.len() != 2
            || matches!(
                self.as_ref().first(),
                Some(
                    Argument::StringLiteral(_)
                        | Argument::TemplateLiteral(_)
                        | Argument::TaggedTemplateExpression(_)
                )
            );

        if is_commonjs_or_amd_call
            || is_multiline_template_only_args(self, f.source_text())
            || is_react_hook_with_deps_array(self, f.comments())
            || (is_test_call && is_first_arg_string_literal_or_template)
        {
            return write!(
                f,
                [
                    l_paren_token,
                    format_with(|f| {
                        f.join_with(space())
                            .entries(
                                FormatSeparatedIter::new(self.iter(), ",")
                                    .with_trailing_separator(TrailingSeparator::Omit),
                            )
                            .finish()
                    }),
                    r_paren_token
                ]
            );
        }

        let last_index = self.len().saturating_sub(1);
        let mut has_empty_line = false;

        let arguments: Vec<_> = self
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let leading_lines = get_lines_before(element.span().start, f);
                has_empty_line = has_empty_line || leading_lines > 1;

                FormatCallArgument::Default { element, is_last: index == last_index, leading_lines }
            })
            .collect();

        if has_empty_line || is_function_composition_args(self) {
            return write!(
                f,
                [FormatAllArgsBrokenOut { args: &arguments, node: self, expand: true }]
            );
        }

        if let Some(group_layout) = arguments_grouped_layout(self, f) {
            write_grouped_arguments(self, arguments, group_layout, f)
        } else if is_long_curried_call(self.parent) {
            write!(
                f,
                [
                    l_paren_token,
                    soft_block_indent(&format_once(|f| {
                        write_arguments_multi_line(arguments.iter(), f)
                    })),
                    r_paren_token,
                ]
            )
        } else {
            write!(f, [FormatAllArgsBrokenOut { args: &arguments, node: self, expand: false }])
        }
    }
}

/// Helper for formatting a call argument
pub enum FormatCallArgument<'a, 'b> {
    /// Argument that has not been inspected if its formatted content breaks.
    Default {
        element: &'b AstNode<'a, Argument<'a>>,

        /// Whether this is the last element.
        is_last: bool,

        /// The number of lines before this node
        leading_lines: usize,
    },

    /// The argument has been formatted because a caller inspected if it [Self::will_break].
    ///
    /// Allows to re-use the formatted output rather than having to call into the formatting again.
    Inspected {
        /// The formatted element
        content: FormatResult<Option<FormatElement<'a>>>,

        /// The separated element
        element: &'b AstNode<'a, Argument<'a>>,

        /// The lines before this element
        leading_lines: usize,
    },
}

impl<'a> FormatCallArgument<'a, '_> {
    /// Returns `true` if this argument contains any content that forces a group to [`break`](FormatElements::will_break).
    fn will_break(&mut self, f: &mut Formatter<'_, 'a>) -> bool {
        match &self {
            Self::Default { element, leading_lines, .. } => {
                let interned = f.intern(&self);

                let breaks = match &interned {
                    Ok(Some(element)) => element.will_break(),
                    _ => false,
                };

                *self =
                    Self::Inspected { content: interned, element, leading_lines: *leading_lines };
                breaks
            }
            Self::Inspected { content: Ok(Some(result)), .. } => result.will_break(),
            Self::Inspected { .. } => false,
        }
    }

    /// Formats the node of this argument and caches the function body.
    ///
    /// See [crate::FormatContext::cached_function_body]
    ///
    /// # Panics
    ///
    /// If [`cache_function_body`](Self::cache_function_body) or [`will_break`](Self::will_break) has been called on this argument before.
    fn cache_function_body(&mut self, f: &mut Formatter<'_, 'a>) {
        match &self {
            Self::Default { element, leading_lines, .. } => {
                let interned = f.intern(&format_once(|f| {
                    self.fmt_with_cache_mode(FunctionBodyCacheMode::Cache, f)?;
                    Ok(())
                }));

                *self =
                    Self::Inspected { content: interned, element, leading_lines: *leading_lines };
            }
            Self::Inspected { .. } => {
                panic!("`cache` must be called before inspecting or formatting the element.");
            }
        }
    }

    fn fmt_with_cache_mode(
        &self,
        cache_mode: FunctionBodyCacheMode,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<()> {
        match self {
            // Re-use the cached formatted output if there is any.
            Self::Inspected { content, .. } => match content.clone()? {
                Some(element) => f.write_element(element),
                None => Ok(()),
            },
            Self::Default { element, is_last, .. } => {
                match element.as_ast_nodes() {
                    AstNodes::Function(function) => {
                        write!(
                            f,
                            // [function.format().with_options(FormatFunctionOptions {
                            //     body_cache_mode: cache_mode,
                            //     ..FormatFunctionOptions::default()
                            // })]
                            function
                        )?;
                    }
                    AstNodes::ArrowFunctionExpression(arrow) => write!(
                        f,
                        FormatJsArrowFunctionExpression::new_with_options(
                            arrow,
                            FormatJsArrowFunctionExpressionOptions {
                                body_cache_mode: cache_mode,
                                ..FormatJsArrowFunctionExpressionOptions::default()
                            },
                        )
                    )?,
                    _ => write!(f, element)?,
                }

                if *is_last { Ok(()) } else { write!(f, ",") }
            }
        }
    }

    /// Returns the number of leading lines before the argument's node
    fn leading_lines(&self) -> usize {
        match self {
            Self::Inspected { leading_lines, .. } | Self::Default { leading_lines, .. } => {
                *leading_lines
            }
        }
    }

    /// Returns an argument.
    fn element(&self) -> &AstNode<'a, Argument<'a>> {
        match self {
            Self::Inspected { element, .. } | Self::Default { element, .. } => element,
        }
    }
}

impl<'a> Format<'a> for FormatCallArgument<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.fmt_with_cache_mode(FunctionBodyCacheMode::default(), f)
    }
}

/// Tests if a call has multiple anonymous function like (arrow or function expression) arguments.
///
/// ## Examples
///
/// ```javascript
/// compose(sortBy(x => x), flatten, map(x => [x, x*2]));
/// ```
pub fn is_function_composition_args(args: &[Argument<'_>]) -> bool {
    if args.len() <= 1 {
        return false;
    }

    let mut has_seen_function_like = false;

    for arg in args {
        match arg {
            Argument::FunctionExpression(_) | Argument::ArrowFunctionExpression(_) => {
                if has_seen_function_like {
                    return true;
                }
                has_seen_function_like = true;
            }
            Argument::CallExpression(call) => {
                if call.arguments.iter().any(|arg| {
                    matches!(
                        arg,
                        Argument::FunctionExpression(_) | Argument::ArrowFunctionExpression(_)
                    )
                }) {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

pub struct FormatAllArgsBrokenOut<'a, 'b> {
    pub args: &'b [FormatCallArgument<'a, 'b>],
    pub expand: bool,
    pub node: &'b AstNode<'a, ArenaVec<'a, Argument<'a>>>,
}

impl<'a> Format<'a> for FormatAllArgsBrokenOut<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let is_inside_import = matches!(self.node.parent, AstNodes::ImportExpression(_));

        write!(
            f,
            [group(&format_args!(
                "(",
                soft_block_indent(&format_with(|f| {
                    for (index, entry) in self.args.iter().enumerate() {
                        if index > 0 {
                            match entry.leading_lines() {
                                0 | 1 => write!(f, [soft_line_break_or_space()])?,
                                _ => write!(f, [empty_line()])?,
                            }
                        }

                        write!(f, [entry])?;
                    }

                    if !is_inside_import {
                        write!(f, [FormatTrailingCommas::All])?;
                    }
                    Ok(())
                })),
                ")",
            ))
            .should_expand(self.expand)]
        )
    }
}

pub fn arguments_grouped_layout(
    args: &AstNode<ArenaVec<Argument>>,
    f: &Formatter<'_, '_>,
) -> Option<GroupedCallArgumentLayout> {
    if should_group_first_argument(args, f) {
        Some(GroupedCallArgumentLayout::GroupedFirstArgument)
    } else if should_group_last_argument(args, f) {
        Some(GroupedCallArgumentLayout::GroupedLastArgument)
    } else {
        None
    }
}

/// Checks if the first argument requires grouping
fn should_group_first_argument(args: &AstNode<ArenaVec<Argument>>, f: &Formatter<'_, '_>) -> bool {
    let mut iter = args.iter();
    match (iter.next().and_then(|a| a.as_expression()), iter.next().and_then(|a| a.as_expression()))
    {
        (Some(first), Some(second)) if iter.next().is_none() => {
            match &first {
                Expression::FunctionExpression(_) => {}
                // Arrow expressions that are a plain expression or are a chain
                // don't get grouped as the first argument, since they'll either
                // fit entirely on the line or break fully. Only a single arrow
                // with a block body can be grouped to collapse the braces.
                Expression::ArrowFunctionExpression(arrow) => {
                    if arrow.expression {
                        return false;
                    }
                }
                _ => return false,
            }

            if matches!(
                second,
                Expression::ArrowFunctionExpression(_)
                    | Expression::FunctionExpression(_)
                    | Expression::ConditionalExpression(_)
            ) {
                return false;
            }

            let call_like_span = args.parent.span();
            !f.comments().has_comments(call_like_span.start, first.span(), second.span().start)
                && !can_group_expression_argument(
                    first.span().end,
                    second,
                    // The span just for finding comment before the call like expression.
                    call_like_span.end,
                    f,
                )
                && is_relatively_short_argument(second)
        }
        _ => false,
    }
}

/// Checks if the last argument should be grouped.
fn should_group_last_argument(args: &AstNode<ArenaVec<Argument>>, f: &Formatter<'_, '_>) -> bool {
    let mut iter = args.as_ref().iter();
    let last = iter.next_back();

    match last.and_then(|arg| arg.as_expression()) {
        Some(last) => {
            let penultimate = iter.next_back();
            if let Some(penultimate) = &penultimate {
                // TODO: check if both last and penultimate are same kind of expression.
                // if penultimate.syntax().kind() == last.syntax().kind() {
                //     return Ok(false);
                // }
            }

            let call_like_span = args.parent.span();
            let previous_span = penultimate.map_or(call_like_span.start, |a| a.span().end);
            if f.comments().has_comments(previous_span, last.span(), call_like_span.end) {
                return false;
            }

            if !can_group_expression_argument(previous_span, last, call_like_span.end, f) {
                return false;
            }

            match last {
                Expression::ArrayExpression(array) if args.len() > 1 => {
                    // Not for `useEffect`
                    if args.len() == 2
                        && matches!(penultimate, Some(Argument::ArrowFunctionExpression(_)))
                    {
                        return false;
                    }

                    if can_concisely_print_array_list(array.span, &array.elements, f) {
                        return false;
                    }

                    true
                }
                _ => true,
            }
        }
        _ => false,
    }
}

/// Check if `ty` is a relatively simple type annotation, allowing a few
/// additional cases through. The simplicity is determined as
/// either being a keyword type or any reference type with no additional type
/// parameters. For example:
/// ```
///     number          => true
///     unknown         => true
///     HTMLElement     => true
///     string | object => false
///     Foo<string>     => false
/// ```
/// This function also introspects into array and generic types to extract the
/// core type, but only to a limited extent:
/// ```
///     string[]        => string
///     string[][]      => string
///     string[][][]    => string
///     Foo<string>[][] => string
///     Foo<string[]>[] => string[]
///     Foo<string[][]> => string[][]
/// ```
fn is_simple_ts_type(ty: &TSType<'_>) -> bool {
    // Reach up to two-levels deep into array types:
    //     string[]     => string
    //     string[][]   => string
    //     string[][][] => string[]
    let extracted_array_type = match ty {
        TSType::TSArrayType(array) => match &array.element_type {
            TSType::TSArrayType(inner_array) => Some(&inner_array.element_type),
            element_type => Some(element_type),
            _ => None,
        },
        _ => None,
    };

    // Then, extract the first type parameter of a Generic as long as it's the
    // only parameter:
    //     Foo<number> => number
    //     Foo<number, string> => Foo<number, string>
    let extracted_generic_type = match &extracted_array_type {
        Some(TSType::TSTypeReference(generic)) => {
            if let Some(type_arguments) = &generic.type_arguments {
                let argument_list = &type_arguments.params;
                if argument_list.len() == 1 { argument_list.first() } else { extracted_array_type }
            } else {
                extracted_array_type
            }
        }
        _ => extracted_array_type,
    };

    let resolved_type = extracted_generic_type.unwrap_or(ty);
    match resolved_type {
        // Any keyword or literal types
        TSType::TSLiteralType(_)
        | TSType::TSTemplateLiteralType(_)
        | TSType::TSThisType(_ | _)
        | TSType::TSBigIntKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSObjectKeyword(_)
        | TSType::TSAnyKeyword(_)
        | TSType::TSNeverKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSStringKeyword(_)
        | TSType::TSSymbolKeyword(_)
        | TSType::TSUndefinedKeyword(_)
        | TSType::TSUnknownKeyword(_)
        | TSType::TSVoidKeyword(_) => true,

        // Any reference with no generic type arguments
        TSType::TSTypeReference(reference) => reference.type_arguments.is_none(),

        _ => false,
    }
}

/// Checks if `argument` is "short" enough to be groupable. This aims to be
/// logically similar to Prettier's [`isHopefullyShortCallArgument`](https://github.com/prettier/prettier/blob/093745f0ec429d3db47c1edd823357e0ef24e226/src/language-js/print/call-arguments.js#L279),
fn is_relatively_short_argument(argument: &Expression<'_>) -> bool {
    match argument {
        Expression::BinaryExpression(binary) => {
            SimpleArgument::from(&binary.left).is_simple()
                && SimpleArgument::from(&binary.right).is_simple()
        }
        Expression::LogicalExpression(logical) => {
            SimpleArgument::from(&logical.left).is_simple()
                && SimpleArgument::from(&logical.right).is_simple()
        }
        Expression::TSAsExpression(expr) => {
            is_simple_ts_type(&expr.type_annotation)
                && SimpleArgument::from(&expr.expression).is_simple()
        }
        Expression::TSSatisfiesExpression(expr) => {
            is_simple_ts_type(&expr.type_annotation)
                && SimpleArgument::from(&expr.expression).is_simple()
        }
        Expression::RegExpLiteral(_) => true,
        Expression::CallExpression(call) => match call.arguments.len() {
            0 => true,
            1 => SimpleArgument::from(argument).is_simple(),
            _ => false,
        },
        _ => SimpleArgument::from(argument).is_simple(),
    }
}

/// Checks if `argument` benefits from grouping in call arguments.
fn can_group_expression_argument(
    previous_end: u32,
    argument: &Expression<'_>,
    following_start: u32,
    f: &Formatter<'_, '_>,
) -> bool {
    match argument {
        Expression::ObjectExpression(object_expression) => {
            !object_expression.properties.is_empty()
                || f.comments().has_comments(previous_end, object_expression.span, following_start)
        }

        Expression::ArrayExpression(array_expression) => {
            !array_expression.elements.is_empty()
                || f.comments().has_comments(previous_end, array_expression.span, following_start)
        }
        Expression::TSTypeAssertion(assertion_expression) => can_group_expression_argument(
            previous_end,
            &assertion_expression.expression,
            following_start,
            f,
        ),

        Expression::TSAsExpression(as_expression) => can_group_expression_argument(
            previous_end,
            &as_expression.expression,
            following_start,
            f,
        ),

        Expression::TSSatisfiesExpression(satisfies_expression) => can_group_expression_argument(
            previous_end,
            &satisfies_expression.expression,
            following_start,
            f,
        ),

        Expression::ArrowFunctionExpression(arrow_function) => {
            can_group_arrow_function_expression_argument(arrow_function, false, f)
        }
        Expression::FunctionExpression(_) => true,
        _ => false,
    }
}

fn can_group_arrow_function_expression_argument(
    arrow_function: &ArrowFunctionExpression,
    is_arrow_recursion: bool,
    f: &Formatter<'_, '_>,
) -> bool {
    let body = &arrow_function.body;
    let return_type_annotation = &arrow_function.return_type;

    // Handles cases like:
    //
    // app.get("/", (req, res): void => {
    //     res.send("Hello World!");
    // });
    //
    // export class Thing implements OtherThing {
    //   do: (type: Type) => Provider<Prop> = memoize(
    //     (type: ObjectType): Provider<Opts> => {}
    //   );
    // }
    let can_group_type = return_type_annotation.as_ref().is_none_or(|any_type| {
        match &any_type.type_annotation {
            TSType::TSTypeReference(_) => {
                if arrow_function.expression {
                    return false;
                }
                body.statements.iter().any(|statement| match statement {
                    Statement::EmptyStatement(s) => {
                        // When the body contains an empty statement, comments in
                        // the body will get attached to that statement rather than
                        // the body itself, so they need to be checked for comments
                        // as well to ensure that the body is still considered
                        // groupable when those empty statements are removed by the
                        // printer.
                        // TODO: it seems no difference if we comment out this line
                        // comments.has_comments(s.span)
                        true
                    }
                    _ => true,
                }) || (body.statements.is_empty() && f.comments().has_dangling_comments(body.span))
            }
            _ => true,
        }
    });

    if !can_group_type {
        return false;
    }

    arrow_function.get_expression().is_none_or(|expr| match expr {
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_) => true,
        Expression::ArrowFunctionExpression(inner_arrow_function) => {
            can_group_arrow_function_expression_argument(inner_arrow_function, true, f)
        }
        Expression::CallExpression(_) | Expression::ConditionalExpression(_) => !is_arrow_recursion,
        _ => false,
    })
}

fn write_grouped_arguments<'a, 'b>(
    call_arguments: &'b AstNode<'a, ArenaVec<'a, Argument<'a>>>,
    mut arguments: Vec<FormatCallArgument<'a, 'b>>,
    group_layout: GroupedCallArgumentLayout,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    let grouped_breaks = {
        let (grouped_arg, other_args) = match group_layout {
            GroupedCallArgumentLayout::GroupedFirstArgument => {
                let (first, tail) = arguments.split_at_mut(1);
                (&mut first[0], tail)
            }
            GroupedCallArgumentLayout::GroupedLastArgument => {
                let end_index = arguments.len().saturating_sub(1);
                let (head, last) = arguments.split_at_mut(end_index);
                (&mut last[0], head)
            }
        };

        let non_grouped_breaks = other_args.iter_mut().any(|arg| arg.will_break(f));

        // if any of the not grouped elements break, then fall back to the variant where
        // all arguments are printed in expanded mode.
        if non_grouped_breaks {
            return write!(
                f,
                [FormatAllArgsBrokenOut { args: &arguments, node: call_arguments, expand: true }]
            );
        }

        match grouped_arg.element().as_ast_nodes() {
            AstNodes::ArrowFunctionExpression(_) => {
                grouped_arg.cache_function_body(f);
            }
            AstNodes::Function(function)
                if !other_args.is_empty()
                    || function_has_only_simple_parameters(&function.params) =>
            {
                grouped_arg.cache_function_body(f);
            }
            _ => {
                // Node doesn't have a function body or its a function that doesn't get re-formatted.
            }
        }

        grouped_arg.will_break(f)
    };

    // We now cache the delimiter tokens. This is needed because `[crate::best_fitting]` will try to
    // print each version first
    let l_paren_token = "(".memoized();
    let r_paren_token = ")".memoized();

    // First write the most expanded variant because it needs `arguments`.
    let most_expanded = {
        let mut buffer = VecBuffer::new(f.state_mut());
        buffer.write_element(FormatElement::Tag(Tag::StartEntry))?;

        write!(
            buffer,
            [FormatAllArgsBrokenOut { args: &arguments, node: call_arguments, expand: true }]
        )?;
        buffer.write_element(FormatElement::Tag(Tag::EndEntry))?;

        buffer.into_vec()
    };

    // Now reformat the first or last argument if they happen to be a function or arrow function expression.
    // Function and arrow function expression apply a custom formatting that removes soft line breaks from the parameters,
    // type parameters, and return type annotation.
    //
    // This implementation caches the function body of the "normal" formatted function or arrow function expression
    // to avoid quadratic complexity if the functions' body contains another call expression with an arrow or function expression
    // as first or last argument.
    let last_index = arguments.len() - 1;
    let grouped = arguments
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            let layout = match group_layout {
                GroupedCallArgumentLayout::GroupedFirstArgument if index == 0 => {
                    Some(GroupedCallArgumentLayout::GroupedFirstArgument)
                }
                GroupedCallArgumentLayout::GroupedLastArgument if index == last_index => {
                    Some(GroupedCallArgumentLayout::GroupedLastArgument)
                }
                _ => None,
            };

            FormatGroupedArgument { argument, single_argument_list: last_index == 0, layout }
                .memoized()
        })
        .collect::<Vec<_>>();

    // Write the most flat variant with the first or last argument grouped.
    let most_flat = {
        // let snapshot = f.state_snapshot();
        let mut buffer = VecBuffer::new(f.state_mut());
        buffer.write_element(FormatElement::Tag(Tag::StartEntry))?;

        let result = write!(
            buffer,
            [
                l_paren_token,
                format_with(|f| {
                    f.join_with(soft_line_break_or_space()).entries(grouped.iter()).finish()
                }),
                r_paren_token,
            ]
        );

        // Turns out, using the grouped layout isn't a good fit because some parameters of the
        // grouped function or arrow expression break. In that case, fall back to the all args expanded
        // formatting.
        // This back tracking is required because testing if the grouped argument breaks would also return `true`
        // if any content of the function body breaks. But, as far as this is concerned, it's only interested if
        // any content in the signature breaks.
        if matches!(result, Err(FormatError::PoorLayout)) {
            drop(buffer);
            // f.restore_state_snapshot(snapshot);

            let mut most_expanded_iter = most_expanded.into_iter();
            // Skip over the Start/EndEntry items.
            most_expanded_iter.next();
            most_expanded_iter.next_back();

            return f.write_elements(most_expanded_iter);
        }

        buffer.write_element(FormatElement::Tag(Tag::EndEntry))?;

        buffer.into_vec().into_boxed_slice()
    };

    // Write the second variant that forces the group of the first/last argument to expand.
    let middle_variant = {
        let mut buffer = VecBuffer::new(f.state_mut());

        buffer.write_element(FormatElement::Tag(Tag::StartEntry))?;

        write!(
            buffer,
            [
                l_paren_token,
                format_with(|f| {
                    let mut joiner = f.join_with(soft_line_break_or_space());

                    match group_layout {
                        GroupedCallArgumentLayout::GroupedFirstArgument => {
                            joiner.entry(&group(&grouped[0]).should_expand(true));
                            joiner.entries(&grouped[1..]).finish()
                        }
                        GroupedCallArgumentLayout::GroupedLastArgument => {
                            let last_index = grouped.len() - 1;
                            joiner.entries(&grouped[..last_index]);
                            joiner.entry(&group(&grouped[last_index]).should_expand(true)).finish()
                        }
                    }
                }),
                r_paren_token
            ]
        )?;

        buffer.write_element(FormatElement::Tag(Tag::EndEntry))?;

        buffer.into_vec().into_boxed_slice()
    };

    // If the grouped content breaks, then we can skip the most_flat variant,
    // since we already know that it won't be fitting on a single line.
    let variants = if grouped_breaks {
        write!(f, [expand_parent()])?;
        vec![middle_variant, most_expanded.into_boxed_slice()]
    } else {
        vec![most_flat, middle_variant, most_expanded.into_boxed_slice()]
    };

    // SAFETY: Safe because variants is guaranteed to contain exactly 3 entries:
    // * most flat
    // * middle
    // * most expanded
    // ... and best fitting only requires the most flat/and expanded.
    unsafe {
        f.write_element(FormatElement::BestFitting(
            format_element::BestFittingElement::from_vec_unchecked(variants),
        ))
    }
}

/// Helper for formatting a grouped call argument (see [should_group_first_argument] and [should_group_last_argument]).
struct FormatGroupedArgument<'a, 'b> {
    argument: &'b FormatCallArgument<'a, 'b>,

    /// Whether this argument is the only argument in the argument list.
    single_argument_list: bool,

    /// The layout to use for this argument.
    layout: Option<GroupedCallArgumentLayout>,
}

impl<'a> Format<'a> for FormatGroupedArgument<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self.layout {
            Some(GroupedCallArgumentLayout::GroupedFirstArgument) => FormatGroupedFirstArgument {
                is_only: self.single_argument_list,
                argument: self.argument,
            }
            .fmt(f),
            Some(GroupedCallArgumentLayout::GroupedLastArgument) => FormatGroupedLastArgument {
                is_only: self.single_argument_list,
                argument: self.argument,
            }
            .fmt(f),
            None => self.argument.fmt(f),
        }
    }
}

/// Helper for formatting the first grouped argument (see [should_group_first_argument]).
struct FormatGroupedFirstArgument<'a, 'b> {
    argument: &'b FormatCallArgument<'a, 'b>,

    /// Whether this is the only argument in the argument list.
    is_only: bool,
}

impl<'a> Format<'a> for FormatGroupedFirstArgument<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let element = self.argument.element();

        match element.as_ast_nodes() {
            // Call the arrow function formatting but explicitly passes the call argument layout down
            // so that the arrow function formatting removes any soft line breaks between parameters and the return type.
            AstNodes::ArrowFunctionExpression(arrow) => with_token_tracking_disabled(f, |f| {
                write!(
                    f,
                    FormatJsArrowFunctionExpression::new_with_options(
                        arrow,
                        FormatJsArrowFunctionExpressionOptions {
                            body_cache_mode: FunctionBodyCacheMode::Cached,
                            call_arg_layout: Some(GroupedCallArgumentLayout::GroupedFirstArgument),
                            ..FormatJsArrowFunctionExpressionOptions::default()
                        },
                    )
                )?;

                write!(f, ",")
            }),

            // For all other nodes, use the normal formatting (which already has been cached)
            _ => self.argument.fmt(f),
        }
    }
}

/// Helper for formatting the last grouped argument (see [should_group_last_argument]).
struct FormatGroupedLastArgument<'a, 'b> {
    /// The argument to format
    argument: &'b FormatCallArgument<'a, 'b>,
    /// Is this the only argument in the arguments list
    is_only: bool,
}

impl<'a> Format<'a> for FormatGroupedLastArgument<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let element = self.argument.element();

        // For function and arrow expressions, re-format the node and pass the argument that it is the
        // last grouped argument. This changes the formatting of parameters, type parameters, and return types
        // to remove any soft line breaks.
        match element.as_ast_nodes() {
            AstNodes::Function(function)
                if !self.is_only || function_has_only_simple_parameters(&function.params) =>
            {
                with_token_tracking_disabled(f, |f| {
                    write!(
                        f,
                        // [function.format().with_options(FormatFunctionOptions {
                        //     body_cache_mode: FunctionBodyCacheMode::Cached,
                        //     call_argument_layout: Some(
                        //         GroupedCallArgumentLayout::GroupedLastArgument
                        //     ),
                        // })]
                        function
                    )?;

                    Ok(())
                })
            }

            AstNodes::ArrowFunctionExpression(arrow) => with_token_tracking_disabled(f, |f| {
                write!(
                    f,
                    FormatJsArrowFunctionExpression::new_with_options(
                        arrow,
                        FormatJsArrowFunctionExpressionOptions {
                            body_cache_mode: FunctionBodyCacheMode::Cached,
                            call_arg_layout: Some(GroupedCallArgumentLayout::GroupedLastArgument),
                            ..FormatJsArrowFunctionExpressionOptions::default()
                        },
                    )
                )?;

                Ok(())
            }),
            _ => self.argument.fmt(f),
        }
    }
}

/// Disable the token tracking because it is necessary to format function/arrow expressions slightly different.
fn with_token_tracking_disabled<'a, F: FnOnce(&mut Formatter<'_, 'a>) -> R, R>(
    f: &mut Formatter<'_, 'a>,
    callback: F,
) -> R {
    // let was_disabled = f.state().is_token_tracking_disabled();
    // f.state_mut().set_token_tracking_disabled(true);

    // f.state_mut().set_token_tracking_disabled(was_disabled);

    callback(f)
}

fn function_has_only_simple_parameters(params: &FormalParameters<'_>) -> bool {
    has_only_simple_parameters(params, false)
}

/// Tests if this is a call to commonjs [`require`](https://nodejs.org/api/modules.html#requireid)
/// or amd's [`define`](https://github.com/amdjs/amdjs-api/wiki/AMD#define-function-) function.
fn is_commonjs_or_amd_call(
    arguments: &[Argument<'_>],
    call: &AstNode<'_, CallExpression<'_>>,
) -> bool {
    let Expression::Identifier(ident) = &call.callee else {
        return false;
    };

    match ident.name.as_str() {
        "require" => {
            match arguments.len() {
                0 => false,
                // `require` can be called with any expression that resolves to a
                // string. This check is only an escape hatch to allow a complex
                // expression to break rather than group onto the previous line.
                //
                // EX: `require(path.join(__dirname, 'relative/path'))`
                // Without condition:
                //   require(path.join(
                //     __dirname,
                //     'relative/path'));
                // With condition:
                //   require(
                //     path.join(__dirname, 'relative/path')
                //   );
                1 => matches!(arguments.first(), Some(Argument::StringLiteral(_))),
                _ => true,
            }
        }
        "define" => {
            let in_statement = matches!(call.parent, AstNodes::ExpressionStatement(_));
            if in_statement {
                match arguments.len() {
                    1 => true,
                    2 => matches!(arguments.first(), Some(Argument::ArrayExpression(_))),
                    3 => {
                        let mut iter = arguments.iter();
                        let first = iter.next();
                        let second = iter.next();
                        matches!(
                            (first, second),
                            (Some(Argument::StringLiteral(_)), Some(Argument::ArrayExpression(_)))
                        )
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Returns `true` if `arguments` contains a single [multiline template literal argument that starts on its own ](is_multiline_template_starting_on_same_line).
fn is_multiline_template_only_args(arguments: &[Argument], source_text: &str) -> bool {
    if arguments.len() != 1 {
        return false;
    }

    match arguments.first().unwrap() {
        Argument::TemplateLiteral(template) => {
            is_multiline_template_starting_on_same_line(template.span.start, template, source_text)
        }
        Argument::TaggedTemplateExpression(template) => {
            is_multiline_template_starting_on_same_line(
                template.span.start,
                &template.quasi,
                source_text,
            )
        }
        _ => false,
    }
}

/// This function is used to check if the code is a hook-like code:
///
/// ```js
/// useMemo(() => {}, [])
/// ```
fn is_react_hook_with_deps_array(
    arguments: &AstNode<ArenaVec<Argument>>,
    comments: &Comments,
) -> bool {
    if arguments.len() > 3 || arguments.len() < 2 {
        return false;
    }

    let mut args = arguments.as_ref().iter();
    if arguments.len() == 3 {
        args.next();
    }

    match (args.next(), args.next()) {
        (
            Some(Argument::ArrowFunctionExpression(callback)),
            Some(Argument::ArrayExpression(deps)),
        ) => {
            if !callback.params.is_empty() {
                return false;
            }

            if callback.expression {
                return false;
            }

            // Is there a comment that isn't around the callback or deps?
            !comments.filter_comments_in_span(arguments.parent.span()).any(|comment| {
                !callback.span.contains_inclusive(comment.span)
                    && !deps.span.contains_inclusive(comment.span)
            })
        }
        _ => false,
    }
}

/// This is a specialized function that checks if the current [call expression]
/// resembles a call expression usually used by a testing frameworks.
///
/// If the [call expression] matches the criteria, a different formatting is applied.
///
/// To evaluate the eligibility of a  [call expression] to be a test framework like,
/// we need to check its [callee] and its [arguments].
///
/// 1. The [callee] must contain a name or a chain of names that belongs to the
///    test frameworks, for example: `test()`, `test.only()`, etc.
/// 2. The [arguments] should be at the least 2
/// 3. The first argument has to be a string literal
/// 4. The third argument, if present, has to be a number literal
/// 5. The second argument has to be an [arrow function expression] or [function expression]
/// 6. Both function must have zero or one parameters
///
/// [call expression]: CallExpression
/// [callee]: Expression
/// [arguments]: CallExpression::arguments
/// [arrow function expression]: ArrowFunctionExpression
/// [function expression]: Function
pub fn is_test_call_expression(call: &AstNode<CallExpression<'_>>) -> bool {
    let callee = &call.callee;
    let arguments = &call.arguments;

    let mut args = arguments.iter();

    match (args.next(), args.next(), args.next()) {
        (Some(argument), None, None) if arguments.len() == 1 => {
            if is_angular_test_wrapper(call) && {
                if let AstNodes::CallExpression(call) = call.parent.parent() {
                    is_test_call_expression(call)
                } else {
                    false
                }
            } {
                return matches!(
                    argument,
                    Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_)
                );
            }

            if is_unit_test_set_up_callee(callee) {
                return argument.as_expression().is_some_and(is_angular_test_wrapper_expression);
            }

            false
        }

        // it("description", ..)
        // it(Test.name, ..)
        (_, Some(second), third) if arguments.len() <= 3 && contains_a_test_pattern(callee) => {
            // it('name', callback, duration)
            if !matches!(third, None | Some(Argument::NumericLiteral(_))) {
                return false;
            }

            if second.as_expression().is_some_and(is_angular_test_wrapper_expression) {
                return true;
            }

            let (parameter_count, has_block_body) = match second {
                Argument::FunctionExpression(function) => {
                    (function.params.parameters_count(), true)
                }
                Argument::ArrowFunctionExpression(arrow) => {
                    (arrow.params.parameters_count(), !arrow.expression)
                }
                _ => return false,
            };

            parameter_count == 2 || (parameter_count <= 1 && has_block_body)
        }
        _ => false,
    }
}

/// Note: `inject` is used in AngularJS 1.x, `async` and `fakeAsync` in
/// Angular 2+, although `async` is deprecated and replaced by `waitForAsync`
/// since Angular 12.
///
/// example: <https://docs.angularjs.org/guide/unit-testing#using-beforeall->
///
/// @param {CallExpression} node
/// @returns {boolean}
///
fn is_angular_test_wrapper_expression(expression: &Expression) -> bool {
    matches!(expression, Expression::CallExpression(call) if is_angular_test_wrapper(call))
}

fn is_angular_test_wrapper(call: &CallExpression) -> bool {
    matches!(&call.callee,
        Expression::Identifier(ident) if
        matches!(ident.name.as_str(), "async" | "inject" | "fakeAsync" | "waitForAsync")
    )
}

/// Tests if the callee is a `beforeEach`, `beforeAll`, `afterEach` or `afterAll` identifier
/// that is commonly used in test frameworks.
fn is_unit_test_set_up_callee(callee: &Expression) -> bool {
    matches!(callee, Expression::Identifier(ident) if {
        matches!(ident.name.as_str(), "beforeEach" | "beforeAll" | "afterEach" | "afterAll")
    })
}

/// Iterator that returns the callee names in "top down order".
///
/// # Examples
///
/// ```javascript
/// it.only() -> [`only`, `it`]
/// ```
///
/// Same as <https://github.com/biomejs/biome/blob/4a5ef84930344ae54f3877da36888a954711f4a6/crates/biome_js_syntax/src/expr_ext.rs#L1402-L1438>.
pub fn callee_name_iterator<'b>(expr: &'b Expression<'_>) -> impl Iterator<Item = &'b str> {
    let mut current = Some(expr);
    std::iter::from_fn(move || match current {
        Some(Expression::Identifier(ident)) => {
            current = None;
            Some(ident.name.as_str())
        }
        Some(Expression::StaticMemberExpression(static_member)) => {
            current = Some(&static_member.object);
            Some(static_member.property.name.as_str())
        }
        _ => None,
    })
}

/// This function checks if a call expressions has one of the following members:
/// - `it`
/// - `it.only`
/// - `it.skip`
/// - `describe`
/// - `describe.only`
/// - `describe.skip`
/// - `test`
/// - `test.only`
/// - `test.skip`
/// - `test.step`
/// - `test.describe`
/// - `test.describe.only`
/// - `test.describe.parallel`
/// - `test.describe.parallel.only`
/// - `test.describe.serial`
/// - `test.describe.serial.only`
/// - `skip`
/// - `xit`
/// - `xdescribe`
/// - `xtest`
/// - `fit`
/// - `fdescribe`
/// - `ftest`
/// - `Deno.test`
///
/// Based on this [article]
///
/// [article]: https://craftinginterpreters.com/scanning-on-demand.html#tries-and-state-machines
pub fn contains_a_test_pattern(expr: &Expression<'_>) -> bool {
    let mut names = callee_name_iterator(expr);

    match names.next() {
        Some("it" | "describe" | "Deno") => match names.next() {
            None => true,
            Some("only" | "skip" | "test") => names.next().is_none(),
            _ => false,
        },
        Some("test") => match names.next() {
            None => true,
            Some("only" | "skip" | "step") => names.next().is_none(),
            Some("describe") => match names.next() {
                None => true,
                Some("only") => names.next().is_none(),
                Some("parallel" | "serial") => match names.next() {
                    None => true,
                    Some("only") => names.next().is_none(),
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        },
        Some("skip" | "xit" | "xdescribe" | "xtest" | "fit" | "fdescribe" | "ftest") => true,
        _ => false,
    }
}
