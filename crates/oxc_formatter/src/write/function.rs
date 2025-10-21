use std::ops::Deref;

use oxc_ast::ast::*;

use super::{
    FormatWrite,
    arrow_function_expression::{FunctionCacheMode, GroupedCallArgumentLayout},
    block_statement::is_empty_block,
};
use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{
        Buffer, FormatError, FormatResult, Formatter,
        buffer::RemoveSoftLinesBuffer,
        prelude::*,
        trivia::{DanglingIndentMode, FormatLeadingComments},
    },
    write,
    write::{
        arrow_function_expression::FormatMaybeCachedFunctionBody, semicolon::OptionalSemicolon,
    },
};

#[derive(Copy, Clone, Debug, Default)]
pub struct FormatFunctionOptions {
    pub call_argument_layout: Option<GroupedCallArgumentLayout>,
    // Determine whether the signature and body should be cached.
    pub cache_mode: FunctionCacheMode,
}

pub struct FormatFunction<'a, 'b> {
    pub function: &'b AstNode<'a, Function<'a>>,
    pub options: FormatFunctionOptions,
}

impl<'a> Deref for FormatFunction<'a, '_> {
    type Target = AstNode<'a, Function<'a>>;

    fn deref(&self) -> &Self::Target {
        self.function
    }
}

impl<'a> Format<'a> for FormatFunction<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let head = format_once(|f| {
            write!(
                f,
                [
                    self.declare.then_some("declare "),
                    self.r#async.then_some("async "),
                    "function",
                    self.generator().then_some("*"),
                    space(),
                    self.id(),
                    group(&self.type_parameters()),
                ]
            )
        });
        FormatContentWithCacheMode::new(self.span, head, self.options.cache_mode).fmt(f)?;

        let format_parameters = FormatContentWithCacheMode::new(
            self.params.span,
            self.params(),
            self.options.cache_mode,
        )
        .memoized();

        let mut format_parameters = format_once(|f: &mut Formatter<'_, 'a>| {
            if self.options.call_argument_layout.is_some() {
                let mut buffer = RemoveSoftLinesBuffer::new(f);

                let mut recording = buffer.start_recording();
                write!(recording, format_parameters)?;
                let recorded = recording.stop();

                if recorded.will_break() {
                    return Err(FormatError::PoorLayout);
                }

                Ok(())
            } else {
                format_parameters.fmt(f)
            }
        })
        .memoized();

        let mut format_return_type = self
            .return_type()
            .map(|return_type| {
                let content = format_once(move |f| {
                    let needs_space =
                        f.context().comments().has_comment_before(return_type.span.start);
                    write!(f, [maybe_space(needs_space), return_type])
                });
                FormatContentWithCacheMode::new(return_type.span, content, self.options.cache_mode)
            })
            .memoized();

        write!(
            f,
            [group(&format_once(|f| {
                let params = &self.params;
                // Inspect early, in case the `return_type` is formatted before `parameters`
                // in `should_group_function_parameters`.
                format_parameters.inspect(f)?;

                let group_parameters = should_group_function_parameters(
                    self.type_parameters.as_deref(),
                    params.parameters_count() + usize::from(self.this_param.is_some()),
                    self.return_type.as_deref(),
                    &mut format_return_type,
                    f,
                )?;

                if group_parameters {
                    write!(f, [group(&format_parameters)])
                } else {
                    write!(f, [format_parameters])
                }?;

                write!(f, [format_return_type])
            }))]
        )?;

        if let Some(body) = self.body() {
            write!(
                f,
                [
                    space(),
                    FormatMaybeCachedFunctionBody {
                        body,
                        mode: self.options.cache_mode,
                        expression: false
                    }
                ]
            )?;
        }

        if self.is_ts_declare_function() {
            write!(f, [OptionalSemicolon])?;
        }

        Ok(())
    }
}

impl<'a> FormatWrite<'a, FormatFunctionOptions> for AstNode<'a, Function<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatFunction { function: self, options: FormatFunctionOptions::default() }.fmt(f)
    }

    fn write_with_options(
        &self,
        options: FormatFunctionOptions,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<()> {
        FormatFunction { function: self, options }.fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, FunctionBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().block_comments_before(self.span.start);
        write!(f, [space(), FormatLeadingComments::Comments(comments)])?;

        let statements = self.statements();
        let directives = self.directives();
        if is_empty_block(statements, f) && directives.is_empty() {
            write!(f, ["{", format_dangling_comments(self.span).with_block_indent(), "}"])
        } else {
            write!(f, ["{", block_indent(&format_args!(directives, statements)), "}"])
        }
    }
}

/// Returns `true` if the function parameters should be grouped.
/// Grouping the parameters has the effect that the return type will break first.
pub fn should_group_function_parameters<'a>(
    type_parameters: Option<&TSTypeParameterDeclaration<'a>>,
    parameter_count: usize,
    return_type: Option<&TSTypeAnnotation<'a>>,
    formatted_return_type: &mut Memoized<'a, impl Format<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<bool> {
    if let Some(type_parameters) = type_parameters {
        match type_parameters.params.len() {
            0 => {} // fall through
            1 => {
                let first = type_parameters.params.iter().next().unwrap();
                if first.constraint.is_some() || first.default.is_some() {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
    }

    let return_type = match return_type {
        Some(return_type) => &return_type.type_annotation,
        None => return Ok(false),
    };

    Ok(parameter_count == 1
        && (matches!(return_type, TSType::TSTypeLiteral(_) | TSType::TSMappedType(_))
            || formatted_return_type.inspect(f)?.will_break()))
}

/// A wrapper that formats content and caches the result based on the given cache mode.
///
/// It is useful in cases like in [`super::call_arguments`] because it allows printing a node
/// a few times to find a proper layout.
/// However, the current architecture of the formatter isn't able to do things like this,
/// because it will cause the comments printed after the first printing to be lost in the
/// subsequent printing, because comments only can be printed once.
/// This wrapper solves this problem by caching the result of the first printing
/// and reusing it in the subsequent printing.
pub struct FormatContentWithCacheMode<T> {
    key: Span,
    content: T,
    cache_mode: FunctionCacheMode,
}

impl<T> FormatContentWithCacheMode<T> {
    pub fn new(key: Span, content: T, cache_mode: FunctionCacheMode) -> Self {
        Self { key, content, cache_mode }
    }
}

impl<'a, T> Format<'a> for FormatContentWithCacheMode<T>
where
    T: Format<'a>,
{
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if matches!(self.cache_mode, FunctionCacheMode::NoCache) {
            self.content.fmt(f)
        } else if let Some(grouped) = f.context().get_cached_element(&self.key) {
            f.write_element(grouped)
        } else {
            if let Ok(Some(grouped)) = f.intern(&self.content) {
                f.context_mut().cache_element(&self.key, grouped.clone());
                f.write_element(grouped.clone());
            }
            Ok(())
        }
    }
}
