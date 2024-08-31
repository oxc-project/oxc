use oxc_ast::ast::{
    BindingPattern, BindingRestElement, Expression, ImportDeclarationSpecifier, ModuleExportName,
    PropertyKey, Statement, TSTypeAnnotation, TSTypeParameterInstantiation,
    VariableDeclarationKind,
};
use oxc_ast::AstBuilder;
use oxc_span::{Atom, SPAN};

fn create_require<'a>(target: &str, builder: &'a AstBuilder) -> Expression<'a> {
    builder.expression_call(
        SPAN,
        builder.expression_identifier_reference(SPAN, "require"),
        None::<TSTypeParameterInstantiation>,
        builder.vec1(builder.argument_expression(builder.expression_string_literal(SPAN, target))),
        false,
    )
}

/// Generate the `require` bond for a given target.
/// e.g. for `fs`:
/// ```js
/// require('fs')
/// ```
///
/// It can be transformed from:
///
/// ```js
/// import 'fs'
/// ```
pub fn create_empty_require<'a>(target: &str, builder: &'a AstBuilder) -> Statement<'a> {
    builder.statement_expression(SPAN, create_require(target, builder))
}

/// Generate a namespaced `require` bond for a given target and assignee.
/// e.g. for `fs`:
/// ```js
/// const fs = require('fs')
/// ```
///
/// It can be transformed from:
///
/// ```js
/// import * as fs from 'fs'
/// ```
pub fn create_namespaced_require<'a>(
    target: &str,
    assignee: &str,
    builder: &'a AstBuilder,
    const_bindings: bool,
) -> Statement<'a> {
    let bindings =
        if const_bindings { VariableDeclarationKind::Const } else { VariableDeclarationKind::Var };
    builder.statement_declaration(builder.declaration_variable(
        SPAN,
        bindings,
        builder.vec1(builder.variable_declarator(
            SPAN,
            bindings,
            builder.binding_pattern(
                builder.binding_pattern_kind_binding_identifier(SPAN, assignee),
                None::<TSTypeAnnotation>,
                false,
            ),
            Some(create_require(target, builder)),
            false,
        )),
        false,
    ))
}

pub fn create_general_require<'a>(
    target: &str,
    assignees: Vec<(PropertyKey<'a>, BindingPattern<'a>)>, // (property, imported)
    builder: &'a AstBuilder,
    const_bindings: bool,
) -> Statement<'a> {
    let bindings =
        if const_bindings { VariableDeclarationKind::Const } else { VariableDeclarationKind::Var };
    builder.statement_declaration(builder.declaration_variable(
        SPAN,
        bindings,
        builder.vec1(builder.variable_declarator(
            SPAN,
            bindings,
            builder.binding_pattern(
                builder.binding_pattern_kind_object_pattern(
                    SPAN,
                    {
                        let mut items = builder.vec();
                        for (property, imported) in assignees {
                            items.push(
                                builder.binding_property(SPAN, property, imported, false, false),
                            );
                        }
                        items
                    },
                    None::<BindingRestElement>,
                ),
                None::<TSTypeAnnotation>,
                false,
            ),
            Some(create_require(target, builder)),
            false,
        )),
        false,
    ))
}
