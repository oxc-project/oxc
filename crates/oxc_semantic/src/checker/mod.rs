use oxc_ast::{
    ast::{DoWhileStatement, ForStatement, WhileStatement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

mod javascript;
mod typescript;

use javascript as js;
use typescript as ts;

use crate::{builder::SemanticBuilder, AstNode};

pub fn check<'a>(node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
    let kind = node.kind();

    match kind {
        AstKind::Program(_) => {
            js::check_duplicate_class_elements(ctx);
        }
        AstKind::BindingIdentifier(ident) => {
            js::check_identifier(&ident.name, ident.span, node, ctx);
            js::check_binding_identifier(ident, node, ctx);
        }
        AstKind::IdentifierReference(ident) => {
            js::check_identifier(&ident.name, ident.span, node, ctx);
            js::check_identifier_reference(ident, node, ctx);
        }
        AstKind::LabelIdentifier(ident) => js::check_identifier(&ident.name, ident.span, node, ctx),
        AstKind::PrivateIdentifier(ident) => js::check_private_identifier_outside_class(ident, ctx),
        AstKind::NumericLiteral(lit) => js::check_number_literal(lit, ctx),
        AstKind::StringLiteral(lit) => js::check_string_literal(lit, ctx),
        AstKind::RegExpLiteral(lit) => js::check_regexp_literal(lit, ctx),

        AstKind::Directive(dir) => js::check_directive(dir, ctx),
        AstKind::ModuleDeclaration(decl) => {
            js::check_module_declaration(decl, node, ctx);
        }
        AstKind::MetaProperty(prop) => js::check_meta_property(prop, node, ctx),

        AstKind::WithStatement(stmt) => {
            js::check_function_declaration(&stmt.body, false, ctx);
            js::check_with_statement(stmt, ctx);
        }
        AstKind::SwitchStatement(stmt) => js::check_switch_statement(stmt, ctx),
        AstKind::BreakStatement(stmt) => js::check_break_statement(stmt, node, ctx),
        AstKind::ContinueStatement(stmt) => js::check_continue_statement(stmt, node, ctx),
        AstKind::LabeledStatement(stmt) => {
            js::check_labeled_statement(stmt, node, ctx);
            js::check_function_declaration(&stmt.body, true, ctx);
        }
        AstKind::ForInStatement(stmt) => {
            js::check_function_declaration(&stmt.body, false, ctx);
            js::check_for_statement_left(&stmt.left, true, node, ctx);
            ts::check_for_statement_left(&stmt.left, true, ctx);
        }
        AstKind::ForOfStatement(stmt) => {
            js::check_function_declaration(&stmt.body, false, ctx);
            js::check_for_statement_left(&stmt.left, false, node, ctx);
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
            js::check_class(class, node, ctx);
            ts::check_class(class, ctx);
        }
        AstKind::MethodDefinition(method) => {
            js::check_method_definition(method, ctx);
            ts::check_method_definition(method, ctx);
        }
        AstKind::PropertyDefinition(prop) => ts::check_property_definition(prop, ctx),
        AstKind::ObjectProperty(prop) => {
            js::check_object_property(prop, ctx);
            ts::check_object_property(prop, ctx);
        }
        AstKind::Super(sup) => js::check_super(sup, node, ctx),

        AstKind::FormalParameters(params) => {
            js::check_formal_parameters(params, node, ctx);
            ts::check_formal_parameters(params, ctx);
        }
        AstKind::ArrayPattern(pat) => {
            js::check_array_pattern(pat, ctx);
            ts::check_array_pattern(pat, ctx);
        }

        AstKind::AssignmentExpression(expr) => js::check_assignment_expression(expr, ctx),
        AstKind::AwaitExpression(expr) => js::check_await_expression(expr, node, ctx),
        AstKind::BinaryExpression(expr) => js::check_binary_expression(expr, ctx),
        AstKind::LogicalExpression(expr) => js::check_logical_expression(expr, ctx),
        AstKind::MemberExpression(expr) => js::check_member_expression(expr, ctx),
        AstKind::ObjectExpression(expr) => js::check_object_expression(expr, ctx),
        AstKind::UnaryExpression(expr) => js::check_unary_expression(expr, node, ctx),
        AstKind::YieldExpression(expr) => js::check_yield_expression(expr, node, ctx),
        AstKind::VariableDeclaration(decl) => ts::check_variable_declaration(decl, ctx),
        AstKind::VariableDeclarator(decl) => ts::check_variable_declarator(decl, ctx),
        AstKind::SimpleAssignmentTarget(target) => ts::check_simple_assignment_target(target, ctx),
        AstKind::TSInterfaceDeclaration(decl) => ts::check_ts_interface_declaration(decl, ctx),
        AstKind::TSTypeAnnotation(annot) => ts::check_ts_type_annotation(annot, ctx),
        AstKind::TSTypeParameterDeclaration(declaration) => {
            ts::check_ts_type_parameter_declaration(declaration, ctx);
        }
        AstKind::TSModuleDeclaration(decl) => ts::check_ts_module_declaration(decl, ctx),
        AstKind::TSEnumDeclaration(decl) => ts::check_ts_enum_declaration(decl, ctx),
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
            let reference = ctx.symbols.get_reference(*reference_id);
            let node = ctx.nodes.get_node(reference.node_id());
            if node.flags().has_export_specifier() {
                if let AstKind::IdentifierReference(ident) = node.kind() {
                    ctx.errors.borrow_mut().push(undefined_export(&ident.name, ident.span));
                }
            }
        }
    }
}
