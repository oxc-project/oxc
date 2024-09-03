use crate::commonjs::utils::define::{create_object_define_property, legitimize_identifier_name};
use crate::commonjs::utils::import::create_require;
use oxc_allocator::{CloneIn, Vec};
use oxc_ast::ast::{
    BindingPatternKind, BindingRestElement, Declaration, ExportSpecifier, Expression,
    FormalParameterKind, FunctionType, ModuleExportName, Statement, StringLiteral, TSAccessibility,
    TSThisParameter, TSTypeAnnotation, TSTypeParameterDeclaration, TSTypeParameterInstantiation,
    VariableDeclarationKind,
};
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator};

fn create_exports<'a>(
    target: ModuleExportName<'a>,
    declaration: Expression<'a>,
    builder: &'a AstBuilder,
) -> Expression<'a> {
    let member_expression = match target {
        ModuleExportName::IdentifierName(name) => builder.member_expression_static(
            SPAN,
            builder.expression_identifier_reference(SPAN, "exports"),
            name,
            false,
        ),
        ModuleExportName::StringLiteral(literal) => builder.member_expression_computed(
            SPAN,
            builder.expression_identifier_reference(SPAN, "exports"),
            builder.expression_from_string_literal(literal),
            false,
        ),
        ModuleExportName::IdentifierReference(ident) => builder.member_expression_computed(
            SPAN,
            builder.expression_identifier_reference(SPAN, "exports"),
            builder.expression_from_identifier_reference(ident),
            false,
        ),
    };
    builder.expression_assignment(
        SPAN,
        AssignmentOperator::Assign,
        builder.assignment_target_simple(
            builder.simple_assignment_target_member_expression(member_expression),
        ),
        declaration,
    )
}

fn create_exports_with_assignment<'a>(
    assigns: Vec<(&str, ModuleExportName<'a>, Expression<'a>)>,
    kind: VariableDeclarationKind,
    builder: &'a AstBuilder,
) -> Statement<'a> {
    builder.statement_declaration(builder.declaration_variable(
        SPAN,
        kind,
        {
            let mut decls = builder.vec();
            for (assignee, target, declaration) in assigns {
                decls.push(builder.variable_declarator(
                    SPAN,
                    kind,
                    builder.binding_pattern(
                        builder.binding_pattern_kind_binding_identifier(SPAN, assignee),
                        None::<TSTypeAnnotation>,
                        false,
                    ),
                    Some(create_exports(target, declaration, builder)),
                    false,
                ))
            }
            decls
        },
        false,
    ))
}

/// Generate the default `exports` bond for a given declaration.
/// e.g. for `export default foo`:
/// ```js
/// exports.default = foo
/// ```
pub fn create_default_exports<'a>(
    declaration: Expression<'a>,
    builder: &'a AstBuilder,
) -> Statement<'a> {
    if declaration.is_identifier_reference() {
        builder.statement_expression(
            SPAN,
            create_exports(
                builder.module_export_name_identifier_name(SPAN, "default"),
                declaration,
                builder,
            ),
        )
    } else {
        create_exports_with_assignment(
            builder.vec1((
                "default",
                builder.module_export_name_identifier_name(SPAN, "_default"),
                declaration,
            )),
            VariableDeclarationKind::Var,
            builder,
        )
    }
}

pub fn create_declared_named_exports<'a>(
    declaration: Declaration<'a>,
    builder: &'a AstBuilder,
) -> Vec<'a, Statement<'a>> {
    match declaration {
        Declaration::VariableDeclaration(decls) => {
            let mut result = builder.vec();
            for decl in decls.declarations.iter() {
                match &decl.id.kind {
                    BindingPatternKind::BindingIdentifier(id) => {
                        result.push(builder.statement_expression(
                            SPAN,
                            create_exports(
                                builder.module_export_name_identifier_name(SPAN, id.name.as_str()),
                                match &decl.init {
                                    Some(init) => init.clone_in(builder.allocator),
                                    None => builder.void_0(),
                                },
                                builder,
                            ),
                        ))
                    }
                    _ => unreachable!(),
                }
            }
            result
        }
        Declaration::FunctionDeclaration(decls) => {
            let mut result = builder.vec();
            // 1. append the function declaration without export
            result.push(builder.statement_expression(
                SPAN,
                builder.expression_from_function(decls.clone_in(builder.allocator)),
            ));
            // 2. append the export statement
            let identifier = &decls.id;
            match identifier {
                Some(id) => result.push(builder.statement_expression(
                    SPAN,
                    create_exports(
                        builder.module_export_name_identifier_reference(SPAN, id.name.as_str()),
                        builder.expression_identifier_reference(SPAN, id.name.as_str()),
                        builder,
                    ),
                )),
                None => unreachable!(),
            }
            result
        }
        Declaration::ClassDeclaration(decls) => {
            let mut result = builder.vec();
            // 1. append the function declaration without export
            result.push(builder.statement_expression(
                SPAN,
                builder.expression_from_class(decls.clone_in(builder.allocator)),
            ));
            // 2. append the export statement
            let identifier = &decls.id;
            match identifier {
                Some(id) => result.push(builder.statement_expression(
                    SPAN,
                    create_exports(
                        builder.module_export_name_identifier_reference(SPAN, id.name.as_str()),
                        builder.expression_identifier_reference(SPAN, id.name.as_str()),
                        builder,
                    ),
                )),
                None => unreachable!(),
            }
            result
        }
        _ => unreachable!(),
    }
}

/// Generate the `exports` bond for all listed exports, which uses `export { foo, bar, bar_foo as foobar }`.
/// It should be transformed to:
///
/// ```js
/// exports.foo = foo
/// exports.bar = bar
/// exports.foobar = bar_foo
/// ```
pub fn create_listed_named_exports<'a>(
    specifiers: Vec<'a, ExportSpecifier<'a>>,
    builder: &'a AstBuilder,
) -> Vec<'a, Statement<'a>> {
    let mut result = builder.vec();
    for specifier in specifiers {
        result.push(builder.statement_expression(
            SPAN,
            create_exports(
                specifier.exported,
                match specifier.local {
                    ModuleExportName::IdentifierReference(id) => {
                        builder.expression_from_identifier_reference(id)
                    }
                    _ => unreachable!(),
                },
                builder,
            ),
        ));
    }
    result
}

/// Generate the `exports` bond for all renamed exports, which uses `export * as foo from 'bar'`.
/// It should be transformed to:
///
/// ```js
///
pub fn create_reexported_named_exports<'a>(
    specifiers: Vec<'a, ExportSpecifier<'a>>,
    source: StringLiteral<'a>,
    builder: &'a AstBuilder,
) -> Vec<'a, Statement<'a>> {
    let mut result = builder.vec();
    // TODO deconflict the name
    let ident = legitimize_identifier_name(source.value.as_str()).to_string();
    // 1. Generate require
    result.push(builder.statement_declaration(builder.declaration_variable(
        SPAN,
        VariableDeclarationKind::Const,
        builder.vec1(builder.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            builder.binding_pattern(
                builder.binding_pattern_kind_binding_identifier(SPAN, ident.as_str()),
                None::<TSTypeAnnotation>,
                false,
            ),
            Some(create_require(source.value.as_str(), builder)),
            false,
        )),
        false,
    )));
    for specifier in specifiers {
        result.push(builder.statement_expression(
            SPAN,
            create_object_define_property(
                match specifier.exported {
                    ModuleExportName::IdentifierReference(id) => id.clone_in(builder.allocator),
                    _ => unreachable!(),
                },
                builder.expression_member(match specifier.local {
                    ModuleExportName::IdentifierName(name) => builder.member_expression_static(
                        SPAN,
                        builder.expression_identifier_reference(SPAN, "exports"),
                        name,
                        false,
                    ),
                    ModuleExportName::StringLiteral(literal) => builder.member_expression_computed(
                        SPAN,
                        builder.expression_identifier_reference(SPAN, "exports"),
                        builder.expression_from_string_literal(literal),
                        false,
                    ),
                    ModuleExportName::IdentifierReference(ident) => {
                        builder.member_expression_computed(
                            SPAN,
                            builder.expression_identifier_reference(SPAN, "exports"),
                            builder.expression_from_identifier_reference(ident),
                            false,
                        )
                    }
                }),
                builder,
            ),
        ));
    }
    result
}

pub fn create_export_star_exports<'a>(
    source: &'a str,
    builder: &'a AstBuilder,
) -> Vec<'a, Statement<'a>> {
    let mut result = builder.vec();
    // TODO deconflict the name
    let ident = legitimize_identifier_name(source).to_string();
    // 1. Generate require
    result.push(builder.statement_declaration(builder.declaration_variable(
        SPAN,
        VariableDeclarationKind::Const,
        builder.vec1(builder.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            builder.binding_pattern(
                builder.binding_pattern_kind_binding_identifier(SPAN, ident.as_str()),
                None::<TSTypeAnnotation>,
                false,
            ),
            Some(create_require(source, builder)),
            false,
        )),
        false,
    )));
    // 2. Iterate the exports and assign them.

    result.push(builder.statement_expression(
        SPAN,
        builder.expression_call(
            SPAN,
            builder.expression_member(builder.member_expression_static(
                SPAN,
                builder.expression_call(
                    SPAN,
                    builder.expression_member(builder.member_expression_static(
                        SPAN,
                        builder.expression_identifier_reference(SPAN, "Object"),
                        builder.identifier_name(SPAN, "keys"),
                        false,
                    )),
                    None::<TSTypeParameterInstantiation>,
                    builder.vec1(builder.argument_expression(
                        builder.expression_identifier_reference(SPAN, ident.as_str()),
                    )),
                    false,
                ),
                builder.identifier_name(SPAN, "forEach"),
                false,
            )),
            None::<TSTypeParameterInstantiation>,
            builder.vec1(builder.argument_expression(builder.expression_function(
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
                    builder.vec1(builder.formal_parameter(
                        SPAN,
                        builder.vec(),
                        builder.binding_pattern(
                            builder.binding_pattern_kind_binding_identifier(SPAN, "key"),
                            None::<TSTypeAnnotation>,
                            false,
                        ),
                        None::<TSAccessibility>,
                        false,
                        false,
                    )),
                    None::<BindingRestElement>,
                ),
                None::<TSTypeAnnotation>,
                Some(builder.function_body(SPAN, builder.vec(), {
                    let mut items = builder.vec();
                    items.push(builder.statement_if(
                        SPAN,
                        builder.expression_logical(
                            SPAN,
                            builder.expression_binary(
                                SPAN,
                                builder.expression_identifier_reference(SPAN, "key"),
                                BinaryOperator::StrictEquality,
                                builder.expression_string_literal(SPAN, "default"),
                            ),
                            LogicalOperator::Or,
                            builder.expression_binary(
                                SPAN,
                                builder.expression_identifier_reference(SPAN, "key"),
                                BinaryOperator::StrictEquality,
                                builder.expression_string_literal(SPAN, "__esModule"),
                            ),
                        ),
                        builder.statement_return(SPAN, None),
                        None,
                    ));
                    items.push(builder.statement_if(
                        SPAN,
                        builder.expression_logical(
                            SPAN,
                            builder.expression_binary(
                                SPAN,
                                builder.expression_identifier_reference(SPAN, "key"),
                                BinaryOperator::In,
                                builder.expression_identifier_reference(SPAN, "exports"),
                            ),
                            LogicalOperator::And,
                            builder.expression_binary(
                                SPAN,
                                builder.expression_member(builder.member_expression_computed(
                                    SPAN,
                                    builder.expression_identifier_reference(SPAN, "exports"),
                                    builder.expression_identifier_reference(SPAN, "key"),
                                    false,
                                )),
                                BinaryOperator::StrictEquality,
                                builder.expression_member(builder.member_expression_computed(
                                    SPAN,
                                    builder.expression_identifier_reference(SPAN, ident.as_str()),
                                    builder.expression_identifier_reference(SPAN, "key"),
                                    false,
                                )),
                            ),
                        ),
                        builder.statement_return(SPAN, None),
                        None,
                    ));
                    items.push(builder.statement_expression(
                        SPAN,
                        create_object_define_property(
                            builder.identifier_reference(SPAN, ident.as_str()),
                            builder.expression_identifier_reference(SPAN, "key"),
                            builder,
                        ),
                    ));
                    items
                })),
            ))),
            false,
        ),
    ));
    result
}

pub fn create_renamed_export_star_exports<'a>(
    source: &'a str,
    specifier: ModuleExportName<'a>,
    builder: &'a AstBuilder,
) -> Vec<'a, Statement<'a>> {
    builder.vec1(builder.statement_expression(
        SPAN,
        create_exports(specifier, create_require(source, builder), builder),
    ))
}
