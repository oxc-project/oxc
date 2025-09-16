use std::ops::Deref;

use oxc_ast::ast::*;

use super::{
    FormatWrite,
    arrow_function_expression::{FunctionBodyCacheMode, GroupedCallArgumentLayout},
    block_statement::is_empty_block,
};
use crate::{
    format_args,
    formatter::{
        Buffer, FormatError, FormatResult, Formatter,
        buffer::RemoveSoftLinesBuffer,
        prelude::*,
        trivia::{DanglingIndentMode, FormatLeadingComments},
    },
    generated::ast_nodes::AstNode,
    write,
    write::arrow_function_expression::FormatMaybeCachedFunctionBody,
};

#[derive(Copy, Clone, Debug, Default)]
pub struct FormatFunctionOptions {
    pub call_argument_layout: Option<GroupedCallArgumentLayout>,
    // Determine whether the signature and body should be cached.
    pub cache_mode: FunctionBodyCacheMode,
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

impl<'a> FormatWrite<'a> for FormatFunction<'a, '_> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.declare() {
            write!(f, ["declare", space()])?;
        }

        if self.r#async() {
            write!(f, ["async", space()])?;
        }
        write!(
            f,
            [
                "function",
                self.generator().then_some("*"),
                space(),
                self.id(),
                group(&self.type_parameters()),
            ]
        );

        // The [`call_arguments`] will format the argument that can be grouped in different ways until
        // find the best layout. So we have to cache the parameters because it never be broken.
        let cached_signature = format_once(|f| {
            if matches!(self.options.cache_mode, FunctionBodyCacheMode::NoCache) {
                self.params().fmt(f)
            } else if let Some(grouped) = f.context().get_cached_element(&self.params.span) {
                f.write_element(grouped)
            } else {
                if let Ok(Some(grouped)) = f.intern(&self.params()) {
                    f.context_mut().cache_element(&self.params.span, grouped.clone());
                    f.write_element(grouped.clone());
                }
                Ok(())
            }
        });

        let format_parameters = format_with(|f: &mut Formatter<'_, 'a>| {
            if self.options.call_argument_layout.is_some() {
                let mut buffer = RemoveSoftLinesBuffer::new(f);

                let mut recording = buffer.start_recording();
                write!(recording, cached_signature)?;
                let recorded = recording.stop();

                if recorded.will_break() {
                    return Err(FormatError::PoorLayout);
                }

                Ok(())
            } else {
                cached_signature.fmt(f)
            }
        });

        write!(
            f,
            [group(&format_with(|f| {
                let params = &self.params;
                let mut format_return_type_annotation = self.return_type().memoized();
                let group_parameters = should_group_function_parameters(
                    self.type_parameters.as_deref(),
                    params.items.len() + usize::from(params.rest.is_some()),
                    self.return_type.as_deref(),
                    &mut format_return_type_annotation,
                    f,
                )?;

                if group_parameters {
                    write!(f, [group(&format_parameters)])?;
                } else {
                    write!(f, [format_parameters])?;
                }

                write!(f, [format_return_type_annotation])
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

        Ok(())
    }
}

impl<'a> FormatWrite<'a, FormatFunctionOptions> for AstNode<'a, Function<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatFunction { function: self, options: FormatFunctionOptions::default() }.write(f)
    }

    fn write_with_options(
        &self,
        options: FormatFunctionOptions,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<()> {
        FormatFunction { function: self, options }.write(f)
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
    let return_type = match return_type {
        Some(return_type) => &return_type.type_annotation,
        None => return Ok(false),
    };

    if let Some(type_parameters) = type_parameters {
        let mut params = type_parameters.params.iter();
        match params.next() {
            None => {} // fall through
            Some(first) if params.count() == 0 => {
                if first.constraint.is_some() || first.default.is_some() {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
    }

    Ok(parameter_count != 1
        && (matches!(return_type, TSType::TSLiteralType(_) | TSType::TSMappedType(_))
            || formatted_return_type.inspect(f)?.will_break()))
}
