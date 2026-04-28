use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{Formatter, prelude::*},
    print::function::should_group_function_parameters,
    utils::format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    write,
};

use super::FormatWrite;

impl<'me, 'a> FormatWrite<'a> for AstNode<'me, 'a, TSFunctionType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        format_grouped_parameters_with_return_type(
            self.type_parameters(),
            self.this_param.as_deref(),
            self.params(),
            Some(self.return_type()),
            /* is_function_or_constructor_type */ true,
            f,
        );
    }
}

impl<'me, 'a> FormatWrite<'a> for AstNode<'me, 'a, TSConstructorType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let r#abstract = self.r#abstract();

        write!(
            f,
            group(&format_args!(
                r#abstract.then_some("abstract "),
                "new",
                space(),
                &format_with(|f| {
                    format_grouped_parameters_with_return_type(
                        self.type_parameters(),
                        None,
                        self.params(),
                        Some(self.return_type()),
                        /* is_function_or_constructor_type */ true,
                        f,
                    );
                })
            ))
        );
    }
}

impl<'me, 'a> FormatWrite<'a> for AstNode<'me, 'a, TSCallSignatureDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        format_grouped_parameters_with_return_type(
            self.type_parameters(),
            self.this_param.as_deref(),
            self.params(),
            self.return_type(),
            /* is_function_or_constructor_type */ false,
            f,
        );
    }
}

impl<'me, 'a> FormatWrite<'a> for AstNode<'me, 'a, TSMethodSignature<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let format_inner = format_with(|f| {
            match self.kind() {
                TSMethodSignatureKind::Method => {}
                TSMethodSignatureKind::Get => {
                    write!(f, ["get", space()]);
                }
                TSMethodSignatureKind::Set => {
                    write!(f, ["set", space()]);
                }
            }
            if self.computed() {
                write!(f, "[");
            }
            write!(f, self.key());
            if self.computed() {
                write!(f, "]");
            }
            if self.optional() {
                write!(f, "?");
            }

            let format_type_parameters = self.type_parameters().memoized();
            let format_parameters = self.params().memoized();
            format_type_parameters.inspect(f);
            format_parameters.inspect(f);

            let format_return_type = self.return_type().memoized();

            let should_group_parameters = should_group_function_parameters(
                self.type_parameters.as_deref(),
                self.params.parameters_count() + usize::from(self.this_param.is_some()),
                self.return_type.as_deref(),
                &format_return_type,
                f,
            );

            if should_group_parameters {
                write!(f, group(&format_args!(&format_type_parameters, &format_parameters)));
            } else {
                write!(f, [format_type_parameters, format_parameters]);
            }

            write!(f, group(&format_return_type));
        });

        write!(f, group(&format_inner));
    }
}

impl<'me, 'a> FormatWrite<'a> for AstNode<'me, 'a, TSConstructSignatureDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(
            f,
            group(&format_args!(
                "new",
                space(),
                &format_with(|f| {
                    format_grouped_parameters_with_return_type(
                        self.type_parameters(),
                        None,
                        self.params(),
                        self.return_type(),
                        /* is_function_or_constructor_type */ false,
                        f,
                    );
                })
            ))
        );
    }
}

/// Based on <https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/print/type-annotation.js#L274-L331>
pub fn format_grouped_parameters_with_return_type<'me, 'a>(
    type_parameters: Option<AstNode<'me, 'a, TSTypeParameterDeclaration<'a>>>,
    this_param: Option<&TSThisParameter<'a>>,
    params: AstNode<'me, 'a, FormalParameters<'a>>,
    return_type: Option<AstNode<'me, 'a, TSTypeAnnotation<'a>>>,
    is_function_or_constructor_type: bool,
    f: &mut Formatter<'_, 'a>,
) {
    group(&format_with(|f| {
        let format_type_parameters = type_parameters.memoized();
        let format_parameters = params.memoized();
        let format_return_type = return_type.map(FormatNodeWithoutTrailingComments).memoized();

        // Inspect early, in case the `return_type` is formatted before `parameters`
        // in `should_group_function_parameters`.
        format_type_parameters.inspect(f);
        format_parameters.inspect(f);

        let group_parameters = should_group_function_parameters(
            type_parameters.as_ref().map(AsRef::as_ref),
            params.parameters_count() + usize::from(this_param.is_some()),
            return_type.as_ref().map(AsRef::as_ref),
            &format_return_type,
            f,
        );

        if group_parameters {
            write!(f, [group(&format_args!(format_type_parameters, format_parameters))]);
        } else {
            write!(f, [format_type_parameters, format_parameters]);
        }

        write!(f, [is_function_or_constructor_type.then_some(space()), format_return_type]);
    }))
    .fmt(f);
}
