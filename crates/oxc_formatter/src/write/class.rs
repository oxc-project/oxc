use std::ops::Deref;

use oxc_allocator::Vec;
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use crate::{
    TrailingSeparator, format_args,
    formatter::{
        Buffer, FormatResult, Formatter,
        prelude::*,
        separated::FormatSeparatedIter,
        trivia::{DanglingIndentMode, FormatLeadingComments, FormatTrailingComments},
    },
    generated::ast_nodes::{AstNode, AstNodes},
    parentheses::NeedsParentheses,
    utils::{
        assignment_like::AssignmentLike, expression::FormatExpressionWithoutTrailingComments,
        object::format_property_key,
    },
    write,
    write::{
        semicolon::{ClassPropertySemicolon, OptionalSemicolon},
        type_parameters,
    },
};

use super::{
    FormatWrite,
    type_parameters::{FormatTSTypeParameters, FormatTSTypeParametersOptions},
};

impl<'a> FormatWrite<'a> for AstNode<'a, ClassBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{", block_indent(&self.body()), "}"])
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ClassElement<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = f.source_text();
        // Join class elements with hard line breaks between them
        let mut join = f.join_nodes_with_hardline();
        // Iterate through pairs of consecutive elements to handle semicolons properly
        // Each element is paired with the next one (or None for the last element)
        for (e1, e2) in self.iter().zip(self.iter().skip(1).map(Some).chain(std::iter::once(None)))
        {
            join.entry(e1.span(), &(e1, e2));
        }
        join.finish()
    }
}

impl<'a> Format<'a> for (&AstNode<'a, ClassElement<'a>>, Option<&AstNode<'a, ClassElement<'a>>>) {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.0, ClassPropertySemicolon::new(self.0, self.1)])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, MethodDefinition<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.decorators()])?;
        if let Some(accessibility) = &self.accessibility() {
            write!(f, [accessibility.as_str(), space()])?;
        }
        if self.r#type().is_abstract() {
            write!(f, ["abstract", space()])?;
        }
        if self.r#static() {
            write!(f, ["static", space()])?;
        }
        match &self.kind() {
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
            MethodDefinitionKind::Get => {
                write!(f, ["get", space()])?;
            }
            MethodDefinitionKind::Set => {
                write!(f, ["set", space()])?;
            }
        }
        if self.value().r#async() {
            write!(f, ["async", space()])?;
        }
        if self.value().generator() {
            write!(f, "*")?;
        }
        if self.computed {
            write!(f, ["[", self.key(), "]"])?;
        } else {
            format_property_key(self.key(), f)?;
        }

        if self.optional() {
            write!(f, "?")?;
        }
        if let Some(type_parameters) = &self.value().type_parameters() {
            write!(f, type_parameters)?;
        }
        let value = self.value();
        // Handle comments between method name and parameters
        // Example: method /* comment */ (param) {}
        let comments = f.context().comments().comments_before(value.params().span.start);
        if !comments.is_empty() {
            write!(f, [space(), FormatTrailingComments::Comments(comments)])?;
        }
        write!(f, group(&value.params()))?;
        if let Some(return_type) = &value.return_type() {
            write!(f, return_type)?;
        }
        if let Some(body) = &value.body() {
            // Handle block comments between method signature and body
            // Example: method() /* comment */ {}
            let comments = f.context().comments().comments_before(body.span.start);
            if !comments.is_empty() {
                write!(f, [space(), FormatTrailingComments::Comments(comments)])?;
            }
            write!(f, [space(), body])?;
        }
        if self.r#type().is_abstract()
            || matches!(value.r#type, FunctionType::TSEmptyBodyFunctionExpression)
        {
            write!(f, OptionalSemicolon)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PropertyDefinition<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        AssignmentLike::PropertyDefinition(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PrivateIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#", dynamic_text(self.name().as_str())])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, StaticBlock<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["static", space(), "{"])?;

        if self.body.is_empty() {
            write!(f, [format_dangling_comments(self.span).with_block_indent()])?;
        } else {
            write!(f, [block_indent(&self.body())])?;
        }

        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AccessorProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.decorators())?;
        // Handle comments between decorators and the 'accessor' keyword
        // We look for comments before the first character 'a' of 'accessor'
        // This ensures proper placement of comments like: @decorator /* comment */ accessor x
        let comments = f.context().comments().comments_before_character(self.span.start, b'a');
        FormatLeadingComments::Comments(comments).fmt(f)?;

        if self.r#type().is_abstract() {
            write!(f, ["abstract", space()])?;
        }
        if let Some(accessibility) = self.accessibility() {
            write!(f, [accessibility.as_str(), space()])?;
        }
        if self.r#static() {
            write!(f, ["static", space()])?;
        }
        if self.r#override() {
            write!(f, ["override", space()])?;
        }
        write!(f, ["accessor", space()])?;
        if self.computed() {
            write!(f, "[")?;
        }
        write!(f, self.key())?;
        if self.computed() {
            write!(f, "]")?;
        }
        if let Some(type_annotation) = &self.type_annotation() {
            write!(f, type_annotation)?;
        }
        if let Some(value) = &self.value() {
            write!(f, [space(), "=", space(), value])?;
        }
        Ok(())
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSClassImplements<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                "implements",
                group(&indent(&format_args!(
                    soft_line_break_or_space(),
                    format_once(|f| {
                        // the grouping will be applied by the parent
                        f.join_with(&soft_line_break_or_space())
                            .entries_with_trailing_separator(
                                self.iter(),
                                ",",
                                TrailingSeparator::Disallowed,
                            )
                            .finish()
                    })
                )))
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSClassImplements<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression(), self.type_arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Class<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.r#type == ClassType::ClassExpression
            && (!self.decorators.is_empty() && self.needs_parentheses(f))
        {
            write!(
                f,
                [
                    indent(&format_args!(soft_line_break_or_space(), &FormatClass(self))),
                    soft_line_break_or_space()
                ]
            )
        } else {
            FormatClass(self).fmt(f)
        }
    }
}

struct FormatClass<'a, 'b>(pub &'b AstNode<'a, Class<'a>>);

impl<'a> Deref for FormatClass<'a, '_> {
    type Target = AstNode<'a, Class<'a>>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a> for FormatClass<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let decorators = self.decorators();
        let type_parameters = self.type_parameters();
        let super_class = self.super_class();
        let implements = self.implements();
        let body = self.body();

        // Decorators are handled differently depending on the parent context
        // When the class is exported, the export statement handles decorator formatting
        // to ensure proper placement relative to the export keyword
        if self.is_expression()
            || !matches!(
                self.parent,
                AstNodes::ExportNamedDeclaration(_) | AstNodes::ExportDefaultDeclaration(_)
            )
        {
            write!(f, decorators)?;
        }

        if self.declare {
            write!(f, ["declare", space()])?;
        }
        if self.r#abstract {
            write!(f, ["abstract", space()])?;
        }

        write!(f, "class")?;
        let indent_only_heritage = ((implements.is_empty() && super_class.is_some())
            || (!implements.is_empty() && super_class.is_none()))
            && type_parameters.as_ref().is_some_and(|type_parameters| {
                let current_node_end = type_parameters.span.end;
                let next_node_start = super_class
                    .map(GetSpan::span)
                    .or(implements.first().map(GetSpan::span))
                    .unwrap()
                    .start;
                !f.comments()
                    .comments_in_range(current_node_end, next_node_start)
                    .iter()
                    .any(|c| c.is_line())
            });

        let type_parameters_id = if indent_only_heritage && !implements.is_empty() {
            Some(f.group_id("type_parameters"))
        } else {
            None
        };

        let head = format_with(|f| {
            if let Some(id) = self.id() {
                write!(f, [space()])?;

                if self.type_parameters.is_some()
                    || self.super_class.is_some()
                    || !self.implements.is_empty()
                {
                    id.fmt(f)?;
                } else {
                    id.write(f)?;
                }
            }

            if let Some(type_parameters) = &type_parameters {
                write!(
                    f,
                    FormatTSTypeParameters::new(
                        type_parameters,
                        FormatTSTypeParametersOptions {
                            group_id: type_parameters_id,
                            is_type_or_interface_decl: false
                        }
                    )
                )?;
                if super_class.is_some() || !implements.is_empty() {
                    type_parameters.format_trailing_comments(f)?;
                }
            }

            // Handle comments that appear between the class name and the extends keyword.
            // This is specifically for preserving comments in patterns like:
            //   class A // comment 1
            //     // comment 2
            //     extends B {}
            //
            // The logic here ensures these comments are formatted as trailing comments
            // after the class name, maintaining their position before the extends clause.
            if let Some(super_class) = &super_class {
                let comments = f.context().comments().comments_before(super_class.span().start);
                if comments.iter().any(|c| c.is_line()) {
                    FormatTrailingComments::Comments(comments).fmt(f)?;
                }
            }

            Ok(())
        });

        let group_mode = should_group(self, f);

        let format_heritage_clauses = format_with(|f| {
            if let Some(extends) = super_class {
                // Format the extends clause with its expression and optional type arguments
                let format_super = format_with(|f| {
                    let type_arguments = self.super_type_arguments();

                    // Collect comments after the extends expression (and type arguments if present)
                    // These comments need careful handling to preserve their association
                    // with the extends clause rather than the class body
                    let comments = if type_arguments.is_some() || !implements.is_empty() {
                        &[]
                    } else {
                        f.context()
                            .comments()
                            .comments_in_range(extends.span().end, body.span.start)
                    };

                    // Check if there are trailing line comments after the extends clause
                    // These comments need special handling to ensure they're placed correctly
                    // relative to the extends expression and any type arguments
                    let has_trailing_comments = comments.iter().any(|comment| comment.is_line());

                    let content = format_with(|f| {
                        if let Some(type_arguments) = type_arguments {
                            write!(f, [extends]);
                            if implements.is_empty() {
                                type_arguments.write(f)
                            } else {
                                type_arguments.fmt(f)
                            }
                        } else if implements.is_empty() {
                            FormatExpressionWithoutTrailingComments(extends).fmt(f)?;
                            // Only add trailing comments if they're not line comments
                            // Line comments are handled separately to ensure proper placement
                            if !has_trailing_comments {
                                FormatTrailingComments::Comments(comments).fmt(f)?;
                            }
                            Ok(())
                        } else {
                            extends.fmt(f)
                        }
                    });

                    if matches!(extends.parent.parent(), AstNodes::AssignmentExpression(_)) {
                        if has_trailing_comments {
                            write!(f, [text("("), &content, text(")")])
                        } else {
                            let content = content.memoized();
                            write!(
                                f,
                                [
                                    if_group_breaks(&format_args!(
                                        text("("),
                                        &soft_block_indent(&content),
                                        text(")"),
                                    )),
                                    if_group_fits_on_line(&content)
                                ]
                            )
                        }
                    } else {
                        content.fmt(f)
                    }
                });

                let extends =
                    format_once(|f| write!(f, [space(), "extends", space(), group(&format_super)]));

                if group_mode {
                    write!(f, [soft_line_break_or_space(), group(&extends)])?;
                } else {
                    write!(f, extends)?;
                }
            }

            if !implements.is_empty() {
                if indent_only_heritage {
                    write!(
                        f,
                        [
                            if_group_breaks(&space()).with_group_id(type_parameters_id),
                            if_group_fits_on_line(&soft_line_break_or_space())
                                .with_group_id(type_parameters_id)
                        ]
                    )?;
                } else {
                    write!(f, [soft_line_break_or_space()])?;
                }

                let comments = f.context().comments().comments_before(implements[0].span().start);
                write!(f, [FormatLeadingComments::Comments(comments), implements])?;
            }

            Ok(())
        });

        if group_mode {
            let indented = format_with(|f| {
                if indent_only_heritage {
                    write!(f, [head, indent(&format_heritage_clauses)])
                } else {
                    write!(f, indent(&format_args!(head, format_heritage_clauses)))
                }
            });

            let heritage_id = f.group_id("heritageGroup");
            write!(f, [group(&indented).with_group_id(Some(heritage_id)), space()])?;

            if !body.body().is_empty() {
                write!(f, [if_group_breaks(&hard_line_break()).with_group_id(Some(heritage_id))])?;
            }
        } else {
            write!(f, [head, format_heritage_clauses, space()])?;
        }

        if body.body().is_empty() {
            write!(f, ["{", format_dangling_comments(self.span).with_block_indent(), "}"])
        } else {
            body.fmt(f)
        }
    }
}

/// Determines whether class heritage clauses (extends/implements) should be grouped.
///
/// Grouping affects how line breaks are handled - grouped elements try to fit
/// on the same line but break together if they don't fit.
///
/// Heritage clauses are grouped when:
/// 1. The class has an `implements` clause
/// 2. There are comments in the heritage clause area
/// 3. There are trailing line comments after type parameters
fn should_group<'a>(class: &Class<'a>, f: &Formatter<'_, 'a>) -> bool {
    let comments = f.comments();

    if !class.implements.is_empty() {
        return true;
    }

    let id_span = class.id.as_ref().map(GetSpan::span);
    let type_parameters_span = class.type_parameters.as_ref().map(|t| t.span);
    let super_class_span = class.super_class.as_ref().map(GetSpan::span);
    let super_type_arguments_span = class.super_type_arguments.as_ref().map(|t| t.span);
    let implements_span = class.implements.first().map(GetSpan::span);

    let spans = [
        id_span,
        type_parameters_span,
        super_class_span,
        super_type_arguments_span,
        implements_span,
    ];

    let mut spans_iter = spans.iter().flatten().peekable();

    while let Some(span) = spans_iter.next() {
        if let Some(next_span) = spans_iter.peek() {
            // Check if there are comments between the current span and the next one
            if comments.has_comment_in_range(span.end, next_span.start) {
                // If there are comments, we should group the heritage clauses
                return true;
            }
        } else {
            break;
        }
    }
    false
}
