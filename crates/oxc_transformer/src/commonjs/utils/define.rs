//! It is not `amd define`, but for `Object.defineProperty` related codes.

use oxc_ast::ast::{
    BindingRestElement, Expression, FormalParameterKind, FunctionType, ModuleExportName,
    PropertyKind, TSThisParameter, TSTypeAnnotation, TSTypeParameterDeclaration,
    TSTypeParameterInstantiation,
};
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use oxc_syntax::identifier;
use std::borrow::Cow;

/// Generates `Object.defineProperty` call for a given target.
///
/// Useful when re-exporting:
///
/// ```js
/// export { foo as bar } from './foobar.js'
/// ```
///
/// It can be transformed from:
///
/// ```js
/// Object.defineProperty(exports, 'bar', {
///   enumerable: true,
///   get: function () {
///     return foobar.foo;
///   }
/// })
/// ```
pub fn create_object_define_property<'a>(
    target: ModuleExportName<'a>,
    expression: Expression<'a>,
    builder: &'a AstBuilder,
) -> Expression<'a> {
    builder.expression_call(
        SPAN,
        builder.expression_member(builder.member_expression_static(
            SPAN,
            builder.expression_identifier_reference(SPAN, "Object"),
            builder.identifier_name(SPAN, "defineProperty"),
            false,
        )),
        None::<TSTypeParameterInstantiation>,
        {
            let mut items = builder.vec();
            items.push(
                builder
                    .argument_expression(builder.expression_identifier_reference(SPAN, "exports")),
            );
            items.push(builder.argument_expression(match target {
                ModuleExportName::IdentifierReference(id) => {
                    builder.expression_from_identifier_reference(id)
                }
                ModuleExportName::StringLiteral(id) => builder.expression_from_string_literal(id),
                ModuleExportName::IdentifierName(id) => {
                    builder.expression_string_literal(SPAN, id.name.as_str())
                }
            }));
            items.push(builder.argument_expression(builder.expression_object(
                SPAN,
                {
                    let mut items = builder.vec();
                    items.push(builder.object_property_kind_object_property(
                        SPAN,
                        PropertyKind::Init,
                        builder.property_key_identifier_name(SPAN, "enumerable"),
                        builder.expression_boolean_literal(SPAN, true),
                        None,
                        false,
                        false,
                        false,
                    ));
                    items.push(builder.object_property_kind_object_property(
                        SPAN,
                        PropertyKind::Init,
                        builder.property_key_identifier_name(SPAN, "get"),
                        builder.expression_function(
                            FunctionType::FunctionExpression,
                            SPAN,
                            None,
                            false,
                            false,
                            false,
                            None::<TSTypeParameterDeclaration>,
                            None::<TSThisParameter>,
                            builder.formal_parameters(
                                SPAN,
                                FormalParameterKind::FormalParameter,
                                builder.vec(),
                                None::<BindingRestElement>,
                            ),
                            None::<TSTypeAnnotation>,
                            Some(builder.function_body(
                                SPAN,
                                builder.vec(),
                                builder.vec1(builder.statement_return(SPAN, Some(expression))),
                            )),
                        ),
                        None,
                        false,
                        false,
                        false,
                    ));
                    items
                },
                None,
            )));
            items
        },
        false,
    )
}

/// Port from `rolldown`: https://github.com/rolldown/rolldown/blob/main/crates/rolldown_utils/src/ecma_script.rs#L16-L51.
pub fn legitimize_identifier_name(name: &str) -> Cow<str> {
    let mut legitimized = String::new();
    let mut chars_indices = name.char_indices();

    let mut first_invalid_char_index = None;

    if let Some((idx, first_char)) = chars_indices.next() {
        if identifier::is_identifier_start(first_char) {
            // Nothing we need to do
        } else {
            first_invalid_char_index = Some(idx);
        }
    }

    if first_invalid_char_index.is_none() {
        first_invalid_char_index = chars_indices
            .find(|(_idx, char)| !identifier::is_identifier_part(*char))
            .map(|(idx, _)| idx);
    }

    if let Some(first_invalid_char_index) = first_invalid_char_index {
        let (first_valid_part, rest_part) = name.split_at(first_invalid_char_index);
        legitimized.push_str(first_valid_part);
        for char in rest_part.chars() {
            if identifier::is_identifier_part(char) {
                legitimized.push(char);
            } else {
                legitimized.push('_');
            }
        }

        return Cow::Owned(legitimized);
    }

    Cow::Borrowed(name)
}
