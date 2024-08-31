use oxc_allocator::Vec;
use oxc_ast::ast::{BindingPattern, BindingPatternKind, BindingRestElement, Declaration, ExportDefaultDeclaration, Expression, ModuleExportName, PropertyKey, Statement, TSTypeAnnotation, TSTypeParameterInstantiation, VariableDeclarationKind};
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use oxc_syntax::operator::AssignmentOperator;

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
        ModuleExportName::IdentifierReference(_) => unreachable!(),
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

pub fn create_named_exports<'a>(
    declaration: Declaration<'a>,
    builder: &'a AstBuilder,
    kind: VariableDeclarationKind,
) -> Vec<'a, Statement<'a>> {
    match declaration {
        Declaration::VariableDeclaration(decls) => {
            let mut result = builder.vec();
            for &decl in decls.unbox().declarations.iter() {
                match &decl.id.kind {
                    BindingPatternKind::BindingIdentifier(id) => {
                        result.push(builder.statement_expression(
                            SPAN,
                            create_exports(
                                builder.module_export_name_identifier_name(SPAN, id.name.as_str()),
                                decl.init.unwrap_or(builder.void_0()),
                                builder,
                            ),
                        ))
                    }
                    _ => unreachable!()
                }
            }
            result
        }
        _ => todo!()
    }
}
