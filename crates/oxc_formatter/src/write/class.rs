use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use crate::{
    format_args,
    formatter::{Buffer, FormatResult, Formatter, prelude::*, trivia::DanglingIndentMode},
    generated::ast_nodes::{AstNode, AstNodes},
    write,
};

use super::{
    FormatWrite,
    type_parameters::{FormatTsTypeParameters, FormatTsTypeParametersOptions},
};

impl<'a> FormatWrite<'a> for AstNode<'a, Class<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let decorators = self.decorators();
        let id = self.id();
        let type_parameters = self.type_parameters();
        let super_class = self.super_class();
        let implements = self.implements();
        let body = self.body();
        let r#abstract = self.r#abstract();

        let group_mode = should_group(self, f);

        if !matches!(
            self.parent,
            AstNodes::ExportNamedDeclaration(_) | AstNodes::ExportDefaultDeclaration(_)
        ) {
            write!(f, decorators)?;
        }

        if r#abstract {
            write!(f, ["abstract", space()])?;
        }

        write!(f, "class")?;

        let indent_only_heritage = (implements.is_empty() || super_class.is_none())
            && type_parameters.as_ref().is_some_and(|type_parameters| {
                // TODO: https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js#L447-L499
                // !f.comments().has_trailing_line_comment(type_parameters.span().end)
                true
            });

        let type_parameters_id = if indent_only_heritage && !implements.is_empty() {
            Some(f.group_id("type_parameters"))
        } else {
            None
        };

        let head = format_with(|f| {
            if let Some(id) = &id {
                write!(f, [space(), id])?;
            }

            if let Some(type_parameters) = &type_parameters {
                write!(
                    f,
                    FormatTsTypeParameters::new(
                        type_parameters,
                        FormatTsTypeParametersOptions {
                            group_id: type_parameters_id,
                            is_type_or_interface_decl: false
                        }
                    )
                )?;
            }

            Ok(())
        });

        let format_heritage_clauses = format_with(|f| {
            if let Some(extends) = super_class {
                let extends = format_once(|f| write!(f, [space(), "extends", space(), extends]));
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

                write!(f, implements)?;
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
            write!(f, body)
        }
    }
}

fn should_group<'a>(class: &Class<'a>, f: &Formatter<'_, 'a>) -> bool {
    // TODO: https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js#L447-L499

    // let comments = f.comments();
    // if let Some(id) = &class.id {
    //     if comments.has_trailing_comments(id.span.end) {
    //         return true;
    //     }
    // }

    // if let Some(type_parameters) = &class.type_parameters {
    //     if comments.has_trailing_comments(type_parameters.span.end) {
    //         return true;
    //     }
    // }

    // if let Some(extends) = &class.super_class {
    //     if comments.has_trailing_comments(extends.span().end) {
    //         return true;
    //     }
    // }

    if !class.implements.is_empty() {
        return true;
    }

    false
}
