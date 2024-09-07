use oxc_ast::ast::{
    BindingRestElement, Expression, FormalParameterKind, TSTypeAnnotation,
    TSTypeParameterDeclaration, TSTypeParameterInstantiation,
};
use oxc_ast::AstBuilder;
use oxc_span::SPAN;

pub fn create_promise_resolve_require<'a>(
    target: Expression<'a>,
    builder: &'a AstBuilder,
) -> Expression<'a> {
    builder.expression_call(
        SPAN,
        builder.expression_member(builder.member_expression_static(
            SPAN,
            builder.expression_call(
                SPAN,
                builder.expression_member(builder.member_expression_static(
                    SPAN,
                    builder.expression_identifier_reference(SPAN, "Promise"),
                    builder.identifier_name(SPAN, "resolve"),
                    false,
                )),
                None::<TSTypeParameterInstantiation>,
                builder.vec(),
                false,
            ),
            builder.identifier_name(SPAN, "then"),
            false,
        )),
        None::<TSTypeParameterInstantiation>,
        builder.vec1(builder.argument_expression(builder.expression_arrow_function(
            SPAN,
            true,
            false,
            None::<TSTypeParameterDeclaration>,
            builder.formal_parameters(
                SPAN,
                FormalParameterKind::ArrowFormalParameters,
                builder.vec(),
                None::<BindingRestElement>,
            ),
            None::<TSTypeAnnotation>,
            builder.function_body(
                SPAN,
                builder.vec(),
                builder.vec1(builder.statement_expression(
                    SPAN,
                    builder.expression_call(
                        SPAN,
                        builder.expression_identifier_reference(SPAN, "require"),
                        None::<TSTypeParameterInstantiation>,
                        builder.vec1(builder.argument_expression(target)),
                        false,
                    ),
                )),
            ),
        ))),
        false,
    )
}
