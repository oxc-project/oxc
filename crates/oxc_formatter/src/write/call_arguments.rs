use crate::{
    Buffer, Format, FormatResult, FormatTrailingCommas, TrailingSeparator, format_args,
    formatter::{
        Argument, Comments, FormatElement, FormatError, Formatter,
        prelude::{
            FormatElements, empty_line, format_once, format_with, get_lines_before, group,
            soft_block_indent, soft_line_break_or_space, space,
        },
        separated::FormatSeparatedIter,
        write,
    },
    utils::{is_long_curried_call, write_arguments_multi_line},
    write,
};
use oxc_allocator::Vec;
use oxc_ast::ast::Argument as OxcArgument;
use oxc_span::GetSpan;

use super::arrow_function_expression::{FunctionBodyCacheMode, GroupedCallArgumentLayout};

impl<'a> Format<'a> for Vec<'a, OxcArgument<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let l_paren_token = "(";
        let r_paren_token = ")";
        if self.is_empty() {
            return write!(
                f,
                [
                    l_paren_token,
                    // format_dangling_comments(node.syntax()).with_soft_block_indent(),
                    r_paren_token
                ]
            );
        }

        // let (is_commonjs_or_amd_call, is_test_call) =
        //     call_expression.as_ref().map_or((Ok(false), Ok(false)), |call| {
        //         (is_commonjs_or_amd_call(node, call), call.is_test_call_expression())
        //     });
        let (is_commonjs_or_amd_call, is_test_call) = (false, false);

        let is_first_arg_string_literal_or_template = if self.len() != 2 {
            true
        } else {
            matches!(
                self.iter().next(),
                Some(OxcArgument::StringLiteral(_) | OxcArgument::TemplateLiteral(_))
            )
        };

        if is_commonjs_or_amd_call
            // || is_multiline_template_only_args(node)
            // || is_react_hook_with_deps_array(node, f.comments())
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

        let arguments: std::vec::Vec<_> = self
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let leading_lines = get_lines_before(element.span());
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

        if let Some(group_layout) = arguments_grouped_layout(self, f.comments()) {
            write_grouped_arguments(self, arguments, group_layout, f)
        } else if is_long_curried_call(f.parent_stack()) {
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
        element: &'b OxcArgument<'a>,

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
        content: FormatResult<Option<FormatElement>>,

        /// The separated element
        element: &'b OxcArgument<'a>,

        /// The lines before this element
        leading_lines: usize,
    },
}

impl<'a, 'b> FormatCallArgument<'a, 'b> {
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
    /// See [JsFormatContext::cached_function_body]
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
                Some(element) => {
                    f.write_element(element)?;
                    Ok(())
                }
                None => Ok(()),
            },
            Self::Default { element, is_last, .. } => {
                match element {
                    // OxcArgument::FunctionExpression(function) => {
                    //     write!(
                    //         f,
                    //         [function.format().with_options(FormatFunctionOptions {
                    //             body_cache_mode: cache_mode,
                    //             ..FormatFunctionOptions::default()
                    //         })]
                    //     )?;
                    // }
                    // OxcArgument::ArrowFunctionExpression(arrow) => {
                    //     write!(
                    //         f,
                    //         [arrow.format().with_options(FormatJsArrowFunctionExpressionOptions {
                    //             body_cache_mode: cache_mode,
                    //             ..FormatJsArrowFunctionExpressionOptions::default()
                    //         })]
                    //     )?;
                    // }
                    node => write!(f, node)?,
                }

                // if let Some(separator) = element.trailing_separator()? {
                //     if *is_last {
                //         write!(f, [format_removed(separator)])
                //     } else {
                //         write!(f, [separator.format()])
                //     }
                // } else if !is_last {
                //     Err(FormatError::SyntaxError)
                // } else {
                //     Ok(())
                // }
                if !*is_last { write!(f, [",", soft_line_break_or_space()]) } else { Ok(()) }
            }
        }
    }

    /// Returns the number of leading lines before the argument's node
    fn leading_lines(&self) -> usize {
        match self {
            Self::Default { leading_lines, .. } => *leading_lines,
            Self::Inspected { leading_lines, .. } => *leading_lines,
        }
    }

    /// Returns the [`separated element`](AstSeparatedElement) of this argument.
    fn element(&self) -> &OxcArgument<'a> {
        match self {
            Self::Default { element, .. } => element,
            Self::Inspected { element, .. } => element,
        }
    }
}

impl<'a, 'b> Format<'a> for FormatCallArgument<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.fmt_with_cache_mode(FunctionBodyCacheMode::default(), f)?;
        Ok(())
    }
}

/// Tests if a call has multiple anonymous function like (arrow or function expression) arguments.
///
/// ## Examples
///
/// ```javascript
/// compose(sortBy(x => x), flatten, map(x => [x, x*2]));
/// ```
pub fn is_function_composition_args(args: &[OxcArgument<'_>]) -> bool {
    if args.len() <= 1 {
        return false;
    }

    let mut has_seen_function_like = false;

    // for arg in args.iter().flatten() {
    for arg in args {
        match arg {
            OxcArgument::FunctionExpression(_) | OxcArgument::ArrowFunctionExpression(_) => {
                if has_seen_function_like {
                    return true;
                }
                has_seen_function_like = true;
            }
            OxcArgument::CallExpression(call) => {
                // TODO: flatten
                call.arguments.iter().any(|arg| {
                    matches!(
                        arg,
                        OxcArgument::FunctionExpression(_)
                            | OxcArgument::ArrowFunctionExpression(_)
                    )
                });
            }
            _ => {}
        }
    }

    false
}

pub struct FormatAllArgsBrokenOut<'a, 'b> {
    pub args: &'b [FormatCallArgument<'a, 'b>],
    pub expand: bool,
    pub node: &'b [OxcArgument<'a>],
}

impl<'a, 'b> Format<'a> for FormatAllArgsBrokenOut<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // let is_inside_import = self.node.parent::<JsImportCallExpression>().is_some();
        let is_inside_import = false;

        write!(
            f,
            [group(&format_args![
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
            ])
            .should_expand(self.expand)]
        )
    }
}

pub fn arguments_grouped_layout(
    args: &[OxcArgument<'_>],
    comments: &Comments,
) -> Option<GroupedCallArgumentLayout> {
    // if should_group_first_argument(args, comments).unwrap_or(false) {
    //     Some(GroupedCallArgumentLayout::GroupedFirstArgument)
    // } else if should_group_last_argument(args, comments).unwrap_or(false) {
    //     Some(GroupedCallArgumentLayout::GroupedLastArgument)
    // } else {
    //     None
    // }
    None
}

fn write_grouped_arguments(
    args: &[OxcArgument<'_>],
    arguments: std::vec::Vec<FormatCallArgument<'_, '_>>,
    group_layout: GroupedCallArgumentLayout,
    f: &mut Formatter<'_, '_>,
) -> FormatResult<()> {
    Ok(())
}
