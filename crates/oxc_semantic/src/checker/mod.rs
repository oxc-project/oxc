use oxc_ast::{
    AstKind,
    ast::{DoWhileStatement, ForStatement, WhileStatement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::builder::SemanticBuilder;

mod javascript;
mod typescript;
use javascript as js;
use typescript as ts;

pub use javascript::is_function_part_of_if_statement;

pub fn check<'a>(kind: AstKind<'a>, ctx: &SemanticBuilder<'a>) {
    match kind {
        AstKind::Program(_) => {
            js::check_duplicate_class_elements(ctx);
        }
        AstKind::BindingIdentifier(ident) => {
            js::check_identifier(&ident.name, ident.span, ctx);
            js::check_binding_identifier(ident, ctx);
        }
        AstKind::IdentifierReference(ident) => {
            js::check_identifier(&ident.name, ident.span, ctx);
            js::check_identifier_reference(ident, ctx);
        }
        AstKind::LabelIdentifier(ident) => js::check_identifier(&ident.name, ident.span, ctx),
        AstKind::PrivateIdentifier(ident) => js::check_private_identifier_outside_class(ident, ctx),
        AstKind::NumericLiteral(lit) => js::check_number_literal(lit, ctx),
        AstKind::StringLiteral(lit) => js::check_string_literal(lit, ctx),

        AstKind::Directive(dir) => js::check_directive(dir, ctx),
        m if m.is_module_declaration() => {
            if let Some(mod_decl_kind) = m.as_module_declaration_kind() {
                js::check_module_declaration(&mod_decl_kind, ctx);
            }
        }
        AstKind::MetaProperty(prop) => js::check_meta_property(prop, ctx),

        AstKind::WithStatement(stmt) => {
            js::check_function_declaration(&stmt.body, false, ctx);
            js::check_with_statement(stmt, ctx);
        }
        AstKind::SwitchStatement(stmt) => js::check_switch_statement(stmt, ctx),
        AstKind::BreakStatement(stmt) => js::check_break_statement(stmt, ctx),
        AstKind::ContinueStatement(stmt) => js::check_continue_statement(stmt, ctx),
        AstKind::LabeledStatement(stmt) => {
            js::check_labeled_statement(stmt, ctx);
            js::check_function_declaration_in_labeled_statement(&stmt.body, ctx);
        }
        AstKind::ForInStatement(stmt) => {
            js::check_function_declaration(&stmt.body, false, ctx);
            js::check_for_statement_left(&stmt.left, true, ctx);
            ts::check_for_statement_left(&stmt.left, true, ctx);
        }
        AstKind::ForOfStatement(stmt) => {
            js::check_function_declaration(&stmt.body, false, ctx);
            js::check_for_statement_left(&stmt.left, false, ctx);
            ts::check_for_statement_left(&stmt.left, false, ctx);
        }
        AstKind::WhileStatement(WhileStatement { body, .. })
        | AstKind::DoWhileStatement(DoWhileStatement { body, .. })
        | AstKind::ForStatement(ForStatement { body, .. }) => {
            js::check_function_declaration(body, false, ctx);
        }
        AstKind::IfStatement(stmt) => {
            js::check_function_declaration(&stmt.consequent, true, ctx);
            if let Some(alternate) = &stmt.alternate {
                js::check_function_declaration(alternate, true, ctx);
            }
        }
        AstKind::Class(class) => {
            js::check_class(class, ctx);
            if !ctx.source_type.is_typescript() {
                js::check_class_redeclaration(class, ctx);
            }
            ts::check_class(class, ctx);
        }
        AstKind::Function(func) if !ctx.source_type.is_typescript() => {
            js::check_function_redeclaration(func, ctx);
        }
        AstKind::MethodDefinition(method) => {
            ts::check_method_definition(method, ctx);
        }
        AstKind::PropertyDefinition(prop) => ts::check_property_definition(prop, ctx),
        AstKind::ObjectProperty(prop) => {
            ts::check_object_property(prop, ctx);
        }
        AstKind::Super(sup) => js::check_super(sup, ctx),

        AstKind::FormalParameters(params) => {
            ts::check_formal_parameters(params, ctx);
        }
        AstKind::ArrayPattern(pat) => {
            ts::check_array_pattern(pat, ctx);
        }

        AstKind::AssignmentExpression(expr) => js::check_assignment_expression(expr, ctx),
        AstKind::AwaitExpression(expr) => js::check_await_expression(expr, ctx),
        AstKind::PrivateFieldExpression(expr) => js::check_private_field_expression(expr, ctx),
        AstKind::ObjectExpression(expr) => js::check_object_expression(expr, ctx),
        AstKind::UnaryExpression(expr) => js::check_unary_expression(expr, ctx),
        AstKind::YieldExpression(expr) => js::check_yield_expression(expr, ctx),
        AstKind::VariableDeclarator(decl) => {
            if !ctx.source_type.is_typescript() {
                js::check_variable_declarator_redeclaration(decl, ctx);
            }
        }
        AstKind::TSTypeAnnotation(annot) => ts::check_ts_type_annotation(annot, ctx),
        AstKind::TSInterfaceDeclaration(decl) => ts::check_ts_interface_declaration(decl, ctx),
        AstKind::TSTypeParameter(param) => ts::check_ts_type_parameter(param, ctx),
        AstKind::TSModuleDeclaration(decl) => ts::check_ts_module_declaration(decl, ctx),
        AstKind::TSEnumDeclaration(decl) => ts::check_ts_enum_declaration(decl, ctx),
        AstKind::TSTypeAliasDeclaration(decl) => ts::check_ts_type_alias_declaration(decl, ctx),
        AstKind::TSImportEqualsDeclaration(decl) => {
            ts::check_ts_import_equals_declaration(decl, ctx);
        }
        AstKind::JSXExpressionContainer(container) => {
            ts::check_jsx_expression_container(container, ctx);
        }
        _ => {}
    }
}

#[cold]
fn undefined_export(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Export '{x0}' is not defined")).with_label(span1)
}

/// It is a Syntax Error if any element of the ExportedBindings of ModuleItemList
/// does not also occur in either the VarDeclaredNames of ModuleItemList, or the LexicallyDeclaredNames of ModuleItemList.
pub fn check_unresolved_exports(ctx: &SemanticBuilder<'_>) {
    for reference_ids in ctx.unresolved_references.root().values() {
        for reference_id in reference_ids {
            let reference = ctx.scoping.get_reference(*reference_id);
            let node = ctx.nodes.get_node(reference.node_id());
            if node.flags().has_export_specifier() {
                if let AstKind::IdentifierReference(ident) = node.kind() {
                    ctx.errors.borrow_mut().push(undefined_export(&ident.name, ident.span));
                }
            }
        }
    }
}
