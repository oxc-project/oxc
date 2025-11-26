use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Buffer, Format, FormatTrailingCommas, TrailingSeparator,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Comments, FormatElement, Formatter, SourceText, VecBuffer,
        buffer::RemoveSoftLinesBuffer,
        format_element,
        prelude::{
            FormatElements, Tag, empty_line, expand_parent, format_once, format_with, group,
            soft_block_indent, soft_line_break_or_space, space,
        },
        trivia::format_dangling_comments,
    },
    utils::{
        call_expression::is_test_call_expression, is_long_curried_call,
        member_chain::simple_argument::SimpleArgument,
    },
    write,
    write::{
        FormatFunctionOptions,
        arrow_function_expression::is_multiline_template_starting_on_same_line,
    },
};

use super::{
    FormatJsArrowFunctionExpression,
    array_element_list::can_concisely_print_array_list,
    arrow_function_expression::{
        FormatJsArrowFunctionExpressionOptions, FunctionCacheMode, GroupedCallArgumentLayout,
    },
    function::FormatFunction,
    parameters::has_only_simple_parameters,
};

impl<'a> Format<'a> for AstNode<'a, ArenaVec<'a, Argument<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let l_paren_token = "(";
        let r_paren_token = ")";

        let arguments = self.as_ref();

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

        let is_simple_module_import = is_simple_module_import(self, f.comments());

        let call_expression =
            if !is_simple_module_import && let AstNodes::CallExpression(call) = self.parent {
                Some(call)
            } else {
                None
            };

        if is_simple_module_import
            || call_expression.is_some_and(|call| {
                is_commonjs_or_amd_call(self, call, f)
                    || ((self.len() != 2
                        || matches!(
                            arguments.first(),
                            Some(
                                Argument::StringLiteral(_)
                                    | Argument::TemplateLiteral(_)
                                    | Argument::TaggedTemplateExpression(_)
                            )
                        ))
                        && is_test_call_expression(call))
            })
            || is_multiline_template_only_args(self, f.source_text())
            || is_react_hook_with_deps_array(self, f.comments())
        {
            return write!(
                f,
                [
                    l_paren_token,
                    format_with(|f| {
                        f.join_with(space()).entries_with_trailing_separator(
                            self.iter(),
                            ",",
                            TrailingSeparator::Omit,
                        );
                    }),
                    r_paren_token
                ]
            );
        }

        // Check if there's an empty line (2+ newlines) between any consecutive arguments.
        // This is used to preserve intentional blank lines in the original source.
        let has_empty_line = arguments.windows(2).any(|window| {
            let (cur_arg, next_arg) = (&window[0], &window[1]);

            // Count newlines between arguments, short-circuiting at 2 for performance
            // Check if there are at least two newlines between arguments
            f.source_text()
                .bytes_range(cur_arg.span().end, next_arg.span().start)
                .iter()
                .filter(|&&b| b == b'\n')
                .nth(1)
                .is_some()
        });

        if has_empty_line
            || (!matches!(self.grand_parent(), AstNodes::Decorator(_))
                && is_function_composition_args(self))
        {
            return format_all_args_broken_out(self, true, f);
        }

        if let Some(group_layout) = arguments_grouped_layout(self, f) {
            write_grouped_arguments(self, group_layout, f);
        } else if call_expression.is_some_and(|call| is_long_curried_call(call)) {
            let trailing_operator = FormatTrailingCommas::All.trailing_separator(f.options());
            write!(
                f,
                [
                    l_paren_token,
                    soft_block_indent(&format_with(|f| {
                        f.join_with(soft_line_break_or_space()).entries_with_trailing_separator(
                            self.iter(),
                            ",",
                            trailing_operator,
                        );
                    })),
                    r_paren_token,
                ]
            );
        } else {
            format_all_args_broken_out(self, false, f);
        }
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
    let is_call_expression_with_arrow_or_function = |call: &CallExpression| {
        call.arguments.iter().any(|arg| {
            matches!(arg, Argument::FunctionExpression(_) | Argument::ArrowFunctionExpression(_))
        })
    };

    for arg in args {
        match arg {
            Argument::FunctionExpression(_) | Argument::ArrowFunctionExpression(_) => {
                if has_seen_function_like {
                    return true;
                }
                has_seen_function_like = true;
            }
            Argument::ChainExpression(chain) => {
                return if let ChainElement::CallExpression(call) = &chain.expression {
                    is_call_expression_with_arrow_or_function(call)
                } else {
                    false
                };
            }
            Argument::CallExpression(call) => {
                if is_call_expression_with_arrow_or_function(call) {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

fn format_all_elements_broken_out<'a, 'b>(
    node: &'b AstNode<'a, ArenaVec<'a, Argument<'a>>>,
    elements: impl Iterator<Item = (Option<FormatElement<'a>>, usize)>,
    expand: bool,
    mut buffer: impl Buffer<'a>,
) {
    write!(
        buffer,
        [group(&format_args!(
            "(",
            soft_block_indent(&format_once(move |f| {
                for (index, (element, lines_before)) in elements.into_iter().enumerate() {
                    if let Some(element) = element {
                        if index > 0 {
                            match lines_before {
                                0 | 1 => write!(f, [soft_line_break_or_space()]),
                                _ => write!(f, [empty_line()]),
                            }
                        }

                        f.write_element(element);
                    }
                }

                write!(
                    f,
                    [(!matches!(node.parent, AstNodes::ImportExpression(_)))
                        .then_some(FormatTrailingCommas::All)]
                );
            })),
            ")",
        ))
        .should_expand(expand)]
    );
}

fn format_all_args_broken_out<'a, 'b>(
    node: &'b AstNode<'a, ArenaVec<'a, Argument<'a>>>,
    expand: bool,
    mut buffer: impl Buffer<'a>,
) {
    let last_index = node.len() - 1;
    write!(
        buffer,
        [group(&format_args!(
            "(",
            soft_block_indent(&format_with(move |f| {
                for (index, argument) in node.iter().enumerate() {
                    if index > 0 {
                        match f.source_text().get_lines_before(argument.span(), f.comments()) {
                            0 | 1 => write!(f, [soft_line_break_or_space()]),
                            _ => write!(f, [empty_line()]),
                        }
                    }

                    write!(f, [argument, (index != last_index).then_some(",")]);
                }

                write!(
                    f,
                    [(!matches!(node.parent, AstNodes::ImportExpression(_)))
                        .then_some(FormatTrailingCommas::All)]
                );
            })),
            ")",
        ))
        .should_expand(expand)]
    );
}

pub fn arguments_grouped_layout(
    args: &[Argument],
    f: &Formatter<'_, '_>,
) -> Option<GroupedCallArgumentLayout> {
    // For exactly 2 arguments, we need to check both grouping strategies.
    // To avoid redundant `can_group_expression_argument` calls, we handle this case specially.
    if args.len() == 2 {
        let [first, second] = args else { unreachable!("args.len() == 2 guarantees two elements") };
        let first = first.as_expression()?;
        let second = second.as_expression()?;

        // Call `can_group_expression_argument` only once for the second argument
        let second_can_group = can_group_expression_argument(second, f);
        let second_can_group_fn = || second_can_group;

        // Check if we should group the last argument (second)
        if should_group_last_argument_impl(Some(first), second, second_can_group_fn, f) {
            return Some(GroupedCallArgumentLayout::GroupedLastArgument);
        }

        // Check if we should group the first argument instead
        // Reuse the already-computed `second_can_group` value
        should_group_first_argument(first, second, second_can_group, f)
            .then_some(GroupedCallArgumentLayout::GroupedFirstArgument)
    } else {
        // For other cases (not exactly 2 arguments), only check last argument grouping
        should_group_last_argument(args, f)
            .then_some(GroupedCallArgumentLayout::GroupedLastArgument)
    }
}

/// Checks if the first argument requires grouping
fn should_group_first_argument(
    first: &Expression,
    second: &Expression,
    second_can_group: bool,
    f: &Formatter<'_, '_>,
) -> bool {
    match first {
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

    let first_span = first.span();

    // Don't group if there are comments around the first argument:
    // - Before the first argument
    // - After the first argument (next non-whitespace is not a comma)
    // - End-of-line comments between the first and second argument
    if f.comments().has_comment_before(first_span.start)
        || !f.source_text().next_non_whitespace_byte_is(first_span.end, b',')
        || f.comments()
            .comments_in_range(first_span.end, second.span().start)
            .iter()
            .any(|c| f.comments().is_end_of_line_comment(c))
    {
        return false;
    }

    !second_can_group && is_relatively_short_argument(second)
}

/// Core logic for checking if the last argument should be grouped.
/// Takes the penultimate argument as an Expression for the 2-argument case,
/// or extracts it from the arguments array for other cases.
fn should_group_last_argument_impl(
    penultimate: Option<&Expression>,
    last: &Expression,
    last_can_group_fn: impl FnOnce() -> bool,
    f: &Formatter<'_, '_>,
) -> bool {
    // Check if penultimate and last are the same type (both Object or both Array)
    if let Some(penultimate) = penultimate
        && matches!(
            (penultimate, last),
            (Expression::ObjectExpression(_), Expression::ObjectExpression(_))
                | (Expression::ArrayExpression(_), Expression::ArrayExpression(_))
                | (Expression::TSAsExpression(_), Expression::TSAsExpression(_))
                | (Expression::TSSatisfiesExpression(_), Expression::TSSatisfiesExpression(_))
                | (Expression::ArrowFunctionExpression(_), Expression::ArrowFunctionExpression(_))
                | (Expression::FunctionExpression(_), Expression::FunctionExpression(_))
        )
    {
        return false;
    }

    let last_span = last.span();

    // Don't group if there are comments around the last argument
    let has_comment_before_last = if let Some(penultimate) = penultimate {
        // Check for comments between the penultimate and last argument
        f.comments().comments_in_range(penultimate.span().end, last_span.start).last().is_some_and(
            |c| {
                // Exclude end-of-line comments (treated as previous node's comment)
                // and comments followed by a comma
                !f.comments().is_end_of_line_comment(c)
                    && !f.source_text().next_non_whitespace_byte_is(c.span.end, b',')
            },
        )
    } else {
        f.comments().has_comment_before(last_span.start)
    };

    if has_comment_before_last
        // Check for comments after the last argument
        || f.comments().comments_after(last_span.end).first().is_some_and(|c|
        !f.source_text().bytes_contain(last_span.end, c.span.start, b')'))
    {
        return false;
    }

    if !last_can_group_fn() {
        return false;
    }

    match last {
        Expression::ArrayExpression(array) if penultimate.is_some() => {
            // Not for `useEffect`
            if matches!(penultimate, Some(Expression::ArrowFunctionExpression(_))) {
                return false;
            }

            !can_concisely_print_array_list(array.span, &array.elements, f)
        }
        _ => true,
    }
}

/// Checks if the last argument should be grouped.
fn should_group_last_argument(args: &[Argument], f: &Formatter<'_, '_>) -> bool {
    let mut iter = args.iter();
    let Some(last) = iter.next_back().unwrap().as_expression() else {
        return false;
    };

    let penultimate = iter.next_back().and_then(|arg| arg.as_expression());
    let last_can_group_fn = || can_group_expression_argument(last, f);
    should_group_last_argument_impl(penultimate, last, last_can_group_fn, f)
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
        | TSType::TSThisType(_)
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
            SimpleArgument::from(&binary.left).is_simple_with_depth(1)
                && SimpleArgument::from(&binary.right).is_simple_with_depth(1)
        }
        Expression::LogicalExpression(logical) => {
            SimpleArgument::from(&logical.left).is_simple_with_depth(1)
                && SimpleArgument::from(&logical.right).is_simple_with_depth(1)
        }
        Expression::TSAsExpression(expr) => {
            is_simple_ts_type(&expr.type_annotation)
                && SimpleArgument::from(&expr.expression).is_simple_with_depth(1)
        }
        Expression::TSSatisfiesExpression(expr) => {
            is_simple_ts_type(&expr.type_annotation)
                && SimpleArgument::from(&expr.expression).is_simple_with_depth(1)
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
fn can_group_expression_argument(argument: &Expression<'_>, f: &Formatter<'_, '_>) -> bool {
    match argument {
        Expression::ObjectExpression(object_expression) => {
            !object_expression.properties.is_empty()
                || f.comments().has_comment_in_span(object_expression.span)
        }
        Expression::ArrayExpression(array_expression) => {
            !array_expression.elements.is_empty()
                || f.comments().has_comment_in_span(array_expression.span)
        }
        Expression::TSTypeAssertion(assertion_expression) => {
            can_group_expression_argument(&assertion_expression.expression, f)
        }
        Expression::TSAsExpression(as_expression) => {
            can_group_expression_argument(&as_expression.expression, f)
        }
        Expression::TSSatisfiesExpression(satisfies_expression) => {
            can_group_expression_argument(&satisfies_expression.expression, f)
        }
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
                    #[expect(clippy::match_same_arms)]
                    Statement::EmptyStatement(_) => {
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
                }) || (body.statements.is_empty() && f.comments().has_comment_before(body.span.end))
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
        Expression::ChainExpression(chain) => {
            matches!(chain.expression, ChainElement::CallExpression(_)) && !is_arrow_recursion
        }
        Expression::CallExpression(_) | Expression::ConditionalExpression(_) => !is_arrow_recursion,
        _ => false,
    })
}

fn write_grouped_arguments<'a>(
    node: &AstNode<'a, ArenaVec<'a, Argument<'a>>>,
    group_layout: GroupedCallArgumentLayout,
    f: &mut Formatter<'_, 'a>,
) {
    let last_index = node.len() - 1;
    let only_one_argument = last_index == 0;
    let mut non_grouped_breaks = false;
    let mut grouped_breaks = false;
    let mut has_cached = false;

    // Pre-format the arguments to determine if they can be grouped.
    let elements = node
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            let is_grouped_argument = (group_layout.is_grouped_first() && index == 0)
                || (group_layout.is_grouped_last() && index == last_index);

            // We have to get the lines before the argument has been formatted, because it relies on
            // the comments before the argument. After formatting, the comments might marked as printed,
            // which would lead to a wrong line count.
            let lines_before = f.source_text().get_lines_before(argument.span(), f.comments());
            let comma = (last_index != index).then_some(",");

            let interned = f.intern(&format_once(|f| {
                if is_grouped_argument {
                    match argument.as_ast_nodes() {
                        AstNodes::Function(function)
                            if !group_layout.is_grouped_first()
                                && (!only_one_argument
                                    || function_has_only_simple_parameters(&function.params)) =>
                        {
                            has_cached = true;
                            return write!(
                                f,
                                [
                                    FormatFunction::new_with_options(
                                        function,
                                        FormatFunctionOptions {
                                            cache_mode: FunctionCacheMode::Cache,
                                            ..FormatFunctionOptions::default()
                                        },
                                    ),
                                    comma
                                ]
                            );
                        }
                        AstNodes::ArrowFunctionExpression(arrow) => {
                            has_cached = true;
                            return write!(
                                f,
                                [
                                    FormatJsArrowFunctionExpression::new_with_options(
                                        arrow,
                                        FormatJsArrowFunctionExpressionOptions {
                                            cache_mode: FunctionCacheMode::Cache,
                                            ..FormatJsArrowFunctionExpressionOptions::default()
                                        },
                                    ),
                                    comma
                                ]
                            );
                        }
                        _ => {}
                    }
                }
                write!(f, [argument, comma]);
            }));

            let break_type =
                if is_grouped_argument { &mut grouped_breaks } else { &mut non_grouped_breaks };

            *break_type = *break_type || interned.as_ref().is_some_and(FormatElement::will_break);

            (interned, lines_before)
        })
        .collect::<Vec<_>>();

    // If any of the not grouped elements break, then fall back to the variant where
    // all arguments are printed in expanded mode.
    if non_grouped_breaks {
        return format_all_elements_broken_out(node, elements.into_iter(), true, f);
    }

    // First write the most expanded variant because it needs `arguments`.
    let most_expanded = {
        let mut buffer = VecBuffer::new(f.state_mut());
        buffer.write_element(FormatElement::Tag(Tag::StartEntry));

        format_all_elements_broken_out(node, elements.iter().cloned(), true, &mut buffer);

        buffer.write_element(FormatElement::Tag(Tag::EndEntry));

        buffer.into_vec().into_bump_slice()
    };

    // Now reformat the first or last argument if they happen to be a function or arrow function expression.
    // Function and arrow function expression apply a custom formatting that removes soft line breaks from the parameters,
    // type parameters, and return type annotation.
    //
    // This implementation caches the function body of the "normal" formatted function or arrow function expression
    // to avoid quadratic complexity if the functions' body contains another call expression with an arrow or function expression
    // as first or last argument.
    let mut grouped = elements;
    if has_cached {
        let (argument, grouped_element) = match group_layout {
            GroupedCallArgumentLayout::GroupedFirstArgument => {
                (node.first().unwrap(), &mut grouped.first_mut().unwrap().0)
            }
            GroupedCallArgumentLayout::GroupedLastArgument => {
                (node.last().unwrap(), &mut grouped.last_mut().unwrap().0)
            }
        };

        let function_params = match argument.as_ast_nodes() {
            AstNodes::ArrowFunctionExpression(arrow) => Some(&arrow.params),
            AstNodes::Function(function) => Some(&function.params),
            _ => None,
        };

        // Turns out, using the grouped layout isn't a good fit because some parameters of the
        // grouped function or arrow expression break. In that case, fall back to the all args expanded
        // formatting.
        // This back tracking is required because testing if the grouped argument breaks would also return `true`
        // if any content of the function body breaks. But, as far as this is concerned, it's only interested if
        // any content in the signature breaks.
        //
        // <https://github.com/biomejs/biome/blob/98ca2ae9f3b9b25a14d63b243223583aba6e4907/crates/biome_js_formatter/src/js/expressions/call_arguments.rs#L466-L482>
        if let Some(params) = function_params {
            let Some(cached_element) = f.context().get_cached_element(params.as_ref()) else {
                unreachable!(
                    "The parameters should have already been formatted and cached in the `FormatFunction` or `FormatJsArrowFunctionExpression`"
                );
            };

            // Remove soft lines from the cached parameters and check if they would break.
            // If they break even without soft lines, we need to use the expanded layout.
            let interned = f.intern(&format_once(|f| {
                RemoveSoftLinesBuffer::new(f).write_element(cached_element);
            }));

            if let Some(interned) = interned {
                if interned.will_break() {
                    return format_all_elements_broken_out(node, grouped.into_iter(), true, f);
                }

                // No break; it should print the element without soft lines.
                // It would be used in the `FormatFunction` or `FormatJsArrowFunctionExpression`.
                f.context_mut().cache_element(params.as_ref(), interned);
            }
        }

        *grouped_element = if group_layout.is_grouped_first() {
            f.intern(&format_with(|f| {
                FormatGroupedFirstArgument { argument }.fmt(f);
                write!(f, (last_index != 0).then_some(","));
            }))
        } else {
            f.intern(&FormatGroupedLastArgument { argument, is_only: only_one_argument })
        }
    }

    // Write the second variant that forces the group of the first/last argument to expand.
    let middle_variant = {
        let mut buffer = VecBuffer::new(f.state_mut());

        buffer.write_element(FormatElement::Tag(Tag::StartEntry));

        write!(
            buffer,
            [
                "(",
                format_with(|f| {
                    let mut joiner = f.join_with(soft_line_break_or_space());

                    for (i, (element, _)) in grouped.iter().enumerate() {
                        if (group_layout.is_grouped_first() && i == 0)
                            || (group_layout.is_grouped_last() && i == last_index)
                        {
                            joiner.entry(
                                &group(&format_with(|f| {
                                    if let Some(arg_element) = element.clone() {
                                        f.write_element(arg_element);
                                    }
                                }))
                                .should_expand(true),
                            );
                        } else {
                            joiner.entry(&format_with(|f| {
                                if let Some(arg_element) = element.clone() {
                                    f.write_element(arg_element);
                                }
                            }));
                        }
                    }
                }),
                ")"
            ]
        );

        buffer.write_element(FormatElement::Tag(Tag::EndEntry));

        buffer.into_vec().into_bump_slice()
    };

    // If the grouped content breaks, then we can skip the most_flat variant,
    // since we already know that it won't be fitting on a single line.
    let variants = if grouped_breaks {
        write!(f, [expand_parent()]);
        ArenaVec::from_array_in([middle_variant, most_expanded], f.context().allocator())
    } else {
        // Write the most flat variant with the first or last argument grouped.
        let most_flat = {
            let mut buffer = VecBuffer::new(f.state_mut());
            buffer.write_element(FormatElement::Tag(Tag::StartEntry));

            write!(
                buffer,
                [
                    "(",
                    format_once(|f| {
                        f.join_with(soft_line_break_or_space()).entries(grouped.into_iter().map(
                            |(element, _)| {
                                format_once(move |f| {
                                    if let Some(element) = element {
                                        f.write_element(element);
                                    }
                                })
                            },
                        ));
                    }),
                    ")",
                ]
            );

            buffer.write_element(FormatElement::Tag(Tag::EndEntry));

            buffer.into_vec().into_bump_slice()
        };

        ArenaVec::from_array_in([most_flat, middle_variant, most_expanded], f.context().allocator())
    };

    // SAFETY: Safe because variants is guaranteed to contain exactly 3 entries:
    // * most flat
    // * middle
    // * most expanded
    // ... and best fitting only requires the most flat/and expanded.
    unsafe {
        f.write_element(FormatElement::BestFitting(
            format_element::BestFittingElement::from_vec_unchecked(variants),
        ));
    }
}

/// Helper for formatting the first grouped argument (see [should_group_first_argument]).
struct FormatGroupedFirstArgument<'a, 'b> {
    argument: &'b AstNode<'a, Argument<'a>>,
}

impl<'a> Format<'a> for FormatGroupedFirstArgument<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match self.argument.as_ast_nodes() {
            // Call the arrow function formatting but explicitly passes the call argument layout down
            // so that the arrow function formatting removes any soft line breaks between parameters and the return type.
            AstNodes::ArrowFunctionExpression(arrow) => {
                FormatJsArrowFunctionExpression::new_with_options(
                    arrow,
                    FormatJsArrowFunctionExpressionOptions {
                        cache_mode: FunctionCacheMode::Cache,
                        call_argument_layout: Some(GroupedCallArgumentLayout::GroupedFirstArgument),
                        ..FormatJsArrowFunctionExpressionOptions::default()
                    },
                )
                .fmt(f);
            }

            // For all other nodes, use the normal formatting (which already has been cached)
            _ => self.argument.fmt(f),
        }
    }
}

/// Helper for formatting the last grouped argument (see [should_group_last_argument]).
struct FormatGroupedLastArgument<'a, 'b> {
    /// The argument to format
    argument: &'b AstNode<'a, Argument<'a>>,
    /// Is this the only argument in the arguments list
    is_only: bool,
}

impl<'a> Format<'a> for FormatGroupedLastArgument<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        // For function and arrow expressions, re-format the node and pass the argument that it is the
        // last grouped argument. This changes the formatting of parameters, type parameters, and return types
        // to remove any soft line breaks.
        match self.argument.as_ast_nodes() {
            AstNodes::Function(function)
                if !self.is_only || function_has_only_simple_parameters(&function.params) =>
            {
                FormatFunction::new_with_options(
                    function,
                    FormatFunctionOptions {
                        cache_mode: FunctionCacheMode::Cache,
                        call_argument_layout: Some(GroupedCallArgumentLayout::GroupedLastArgument),
                    },
                )
                .fmt(f);
            }
            AstNodes::ArrowFunctionExpression(arrow) => {
                FormatJsArrowFunctionExpression::new_with_options(
                    arrow,
                    FormatJsArrowFunctionExpressionOptions {
                        cache_mode: FunctionCacheMode::Cache,
                        call_argument_layout: Some(GroupedCallArgumentLayout::GroupedLastArgument),
                        ..FormatJsArrowFunctionExpressionOptions::default()
                    },
                )
                .fmt(f);
            }
            _ => self.argument.fmt(f),
        }
    }
}

fn function_has_only_simple_parameters(params: &FormalParameters<'_>) -> bool {
    has_only_simple_parameters(params, false)
}

/// Tests if this a simple module import like `import("module-name")` or `require("module-name")`.
pub fn is_simple_module_import(
    arguments: &AstNode<'_, ArenaVec<'_, Argument<'_>>>,
    comments: &Comments,
) -> bool {
    if arguments.len() != 1 {
        return false;
    }

    match arguments.parent {
        AstNodes::ImportExpression(_) => {}
        AstNodes::CallExpression(call) => {
            match &call.callee {
                Expression::StaticMemberExpression(member) => match member.property.name.as_str() {
                    "resolve" => {
                        match &member.object {
                            Expression::Identifier(ident) if ident.name.as_str() == "require" => {
                                // `require.resolve("foo")`
                            }
                            Expression::MetaProperty(_) => {
                                // `import.meta.resolve("foo")`
                            }
                            _ => return false,
                        }
                    }
                    "paths" => {
                        if !matches!(
                        &member.object, Expression::StaticMemberExpression(member)
                        if matches!(&member.object, Expression::Identifier(ident)
                            if ident.name == "require") && member.property.name.as_str() == "resolve"
                        ) {
                            return false;
                        }
                    }
                    _ => return false,
                },
                _ => {
                    return false;
                }
            }
        }
        _ => return false,
    }

    matches!(arguments.as_ref()[0], Argument::StringLiteral(_))
        && !comments.has_comment_before(arguments.parent.span().end)
}

/// Tests if amd's [`define`](https://github.com/amdjs/amdjs-api/wiki/AMD#define-function-) function.
fn is_commonjs_or_amd_call(
    arguments: &[Argument<'_>],
    call: &AstNode<'_, CallExpression<'_>>,
    f: &Formatter<'_, '_>,
) -> bool {
    let Expression::Identifier(ident) = &call.callee else {
        return false;
    };

    match ident.name.as_str() {
        "require" => {
            let first_argument = &arguments[0];
            if f.comments().has_comment_before(first_argument.span().start) {
                return false;
            }
            match arguments.len() {
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
                1 => {
                    matches!(first_argument, Argument::StringLiteral(_))
                }
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
fn is_multiline_template_only_args(arguments: &[Argument], source_text: SourceText) -> bool {
    if arguments.len() != 1 {
        return false;
    }

    arguments
        .first()
        .unwrap()
        .as_expression()
        .is_some_and(|expr| is_multiline_template_starting_on_same_line(expr, source_text))
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
            !comments.comments_before(arguments.parent.span().end).iter().any(|comment| {
                !callback.span.contains_inclusive(comment.span)
                    && !deps.span.contains_inclusive(comment.span)
            })
        }
        _ => false,
    }
}
