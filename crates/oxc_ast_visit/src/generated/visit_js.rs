// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/visit_js.rs`.

//! JavaScript-only visitor
//!
//! `VisitJs` traverses only the JavaScript parts of the AST, skipping TypeScript type-space
//! nodes. See [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html).
//!
//! This visitor is for visiting the JavaScript parts of a TypeScript AST — not for ASTs
//! where TypeScript has already been transformed out. TS constructs carrying runtime
//! JavaScript (enum initializers, namespace bodies, decorators, `x as T` casts,
//! `export =`, `import x = require(..)`) are still walked, so their walk code remains
//! in the binary; only the pure type grammar is pruned.

#![expect(unused_variables)]
#![allow(
    clippy::match_same_arms,
    clippy::semicolon_if_nothing_returned,
    clippy::needless_pass_by_ref_mut,
    clippy::trivially_copy_pass_by_ref,
    clippy::match_wildcard_for_single_variants,
    clippy::single_match_else
)]

use std::cell::Cell;

use oxc_allocator::ArenaVec;
use oxc_syntax::scope::{ScopeFlags, ScopeId};

use oxc_ast::ast::*;
use oxc_ast::ast_kind::AstKind;

use walk_js::*;

/// JavaScript-only syntax tree traversal.
///
/// Like [`Visit`], but skips TypeScript type-space nodes. Still descends into JavaScript
/// nested inside TS wrapper nodes (`x as T`, decorators, enum initializers, namespace
/// bodies, `export = expr`, `import x = require(..)`).
///
/// This trait is for visiting the JavaScript parts of a TypeScript AST — not for ASTs
/// where TypeScript has already been transformed out. The walks for those JS-carrying
/// TS nodes stay in the binary; only the pure type grammar is pruned.
///
/// Pruning is by grammar (node kind), not by TypeScript's erasure semantics: type-only
/// imports/exports (`import type`, `export type`) and `declare`d items are JS-grammar
/// nodes and are still visited — filter on `import_kind` / `declare` in the visitor if
/// needed, exactly as with [`Visit`].
///
/// [`Visit`]: crate::Visit
pub trait VisitJs<'a>: Sized {
    #[inline]
    fn enter_node(&mut self, kind: AstKind<'a>) {}
    #[inline]
    fn leave_node(&mut self, kind: AstKind<'a>) {}

    #[inline]
    fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {}
    #[inline]
    fn leave_scope(&mut self) {}

    #[inline]
    fn alloc<T>(&self, t: &T) -> &'a T {
        // SAFETY:
        // This should be safe as long as `src` is an reference from the allocator.
        // But honestly, I'm not really sure if this is safe.
        unsafe { std::mem::transmute(t) }
    }

    #[inline]
    fn visit_program(&mut self, it: &Program<'a>) {
        walk_program(self, it);
    }

    #[inline]
    fn visit_expression(&mut self, it: &Expression<'a>) {
        walk_expression(self, it);
    }

    #[inline]
    fn visit_identifier_name(&mut self, it: &IdentifierName<'a>) {
        walk_identifier_name(self, it);
    }

    #[inline]
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        walk_identifier_reference(self, it);
    }

    #[inline]
    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        walk_binding_identifier(self, it);
    }

    #[inline]
    fn visit_label_identifier(&mut self, it: &LabelIdentifier<'a>) {
        walk_label_identifier(self, it);
    }

    #[inline]
    fn visit_this_expression(&mut self, it: &ThisExpression) {
        walk_this_expression(self, it);
    }

    #[inline]
    fn visit_array_expression(&mut self, it: &ArrayExpression<'a>) {
        walk_array_expression(self, it);
    }

    #[inline]
    fn visit_array_expression_element(&mut self, it: &ArrayExpressionElement<'a>) {
        walk_array_expression_element(self, it);
    }

    #[inline]
    fn visit_elision(&mut self, it: &Elision) {
        walk_elision(self, it);
    }

    #[inline]
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        walk_object_expression(self, it);
    }

    #[inline]
    fn visit_object_property_kind(&mut self, it: &ObjectPropertyKind<'a>) {
        walk_object_property_kind(self, it);
    }

    #[inline]
    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        walk_object_property(self, it);
    }

    #[inline]
    fn visit_property_key(&mut self, it: &PropertyKey<'a>) {
        walk_property_key(self, it);
    }

    #[inline]
    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        walk_template_literal(self, it);
    }

    #[inline]
    fn visit_tagged_template_expression(&mut self, it: &TaggedTemplateExpression<'a>) {
        walk_tagged_template_expression(self, it);
    }

    #[inline]
    fn visit_template_element(&mut self, it: &TemplateElement<'a>) {
        walk_template_element(self, it);
    }

    #[inline]
    fn visit_member_expression(&mut self, it: &MemberExpression<'a>) {
        walk_member_expression(self, it);
    }

    #[inline]
    fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
        walk_computed_member_expression(self, it);
    }

    #[inline]
    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        walk_static_member_expression(self, it);
    }

    #[inline]
    fn visit_private_field_expression(&mut self, it: &PrivateFieldExpression<'a>) {
        walk_private_field_expression(self, it);
    }

    #[inline]
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        walk_call_expression(self, it);
    }

    #[inline]
    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        walk_new_expression(self, it);
    }

    #[inline]
    fn visit_meta_property(&mut self, it: &MetaProperty<'a>) {
        walk_meta_property(self, it);
    }

    #[inline]
    fn visit_spread_element(&mut self, it: &SpreadElement<'a>) {
        walk_spread_element(self, it);
    }

    #[inline]
    fn visit_argument(&mut self, it: &Argument<'a>) {
        walk_argument(self, it);
    }

    #[inline]
    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        walk_update_expression(self, it);
    }

    #[inline]
    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        walk_unary_expression(self, it);
    }

    #[inline]
    fn visit_binary_expression(&mut self, it: &BinaryExpression<'a>) {
        walk_binary_expression(self, it);
    }

    #[inline]
    fn visit_private_in_expression(&mut self, it: &PrivateInExpression<'a>) {
        walk_private_in_expression(self, it);
    }

    #[inline]
    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        walk_logical_expression(self, it);
    }

    #[inline]
    fn visit_conditional_expression(&mut self, it: &ConditionalExpression<'a>) {
        walk_conditional_expression(self, it);
    }

    #[inline]
    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        walk_assignment_expression(self, it);
    }

    #[inline]
    fn visit_assignment_target(&mut self, it: &AssignmentTarget<'a>) {
        walk_assignment_target(self, it);
    }

    #[inline]
    fn visit_simple_assignment_target(&mut self, it: &SimpleAssignmentTarget<'a>) {
        walk_simple_assignment_target(self, it);
    }

    #[inline]
    fn visit_assignment_target_pattern(&mut self, it: &AssignmentTargetPattern<'a>) {
        walk_assignment_target_pattern(self, it);
    }

    #[inline]
    fn visit_array_assignment_target(&mut self, it: &ArrayAssignmentTarget<'a>) {
        walk_array_assignment_target(self, it);
    }

    #[inline]
    fn visit_object_assignment_target(&mut self, it: &ObjectAssignmentTarget<'a>) {
        walk_object_assignment_target(self, it);
    }

    #[inline]
    fn visit_assignment_target_rest(&mut self, it: &AssignmentTargetRest<'a>) {
        walk_assignment_target_rest(self, it);
    }

    #[inline]
    fn visit_assignment_target_maybe_default(&mut self, it: &AssignmentTargetMaybeDefault<'a>) {
        walk_assignment_target_maybe_default(self, it);
    }

    #[inline]
    fn visit_assignment_target_with_default(&mut self, it: &AssignmentTargetWithDefault<'a>) {
        walk_assignment_target_with_default(self, it);
    }

    #[inline]
    fn visit_assignment_target_property(&mut self, it: &AssignmentTargetProperty<'a>) {
        walk_assignment_target_property(self, it);
    }

    #[inline]
    fn visit_assignment_target_property_identifier(
        &mut self,
        it: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        walk_assignment_target_property_identifier(self, it);
    }

    #[inline]
    fn visit_assignment_target_property_property(
        &mut self,
        it: &AssignmentTargetPropertyProperty<'a>,
    ) {
        walk_assignment_target_property_property(self, it);
    }

    #[inline]
    fn visit_sequence_expression(&mut self, it: &SequenceExpression<'a>) {
        walk_sequence_expression(self, it);
    }

    #[inline]
    fn visit_super(&mut self, it: &Super) {
        walk_super(self, it);
    }

    #[inline]
    fn visit_await_expression(&mut self, it: &AwaitExpression<'a>) {
        walk_await_expression(self, it);
    }

    #[inline]
    fn visit_chain_expression(&mut self, it: &ChainExpression<'a>) {
        walk_chain_expression(self, it);
    }

    #[inline]
    fn visit_chain_element(&mut self, it: &ChainElement<'a>) {
        walk_chain_element(self, it);
    }

    #[inline]
    fn visit_parenthesized_expression(&mut self, it: &ParenthesizedExpression<'a>) {
        walk_parenthesized_expression(self, it);
    }

    #[inline]
    fn visit_statement(&mut self, it: &Statement<'a>) {
        walk_statement(self, it);
    }

    #[inline]
    fn visit_directive(&mut self, it: &Directive<'a>) {
        walk_directive(self, it);
    }

    #[inline]
    fn visit_hashbang(&mut self, it: &Hashbang<'a>) {
        walk_hashbang(self, it);
    }

    #[inline]
    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        walk_block_statement(self, it);
    }

    #[inline]
    fn visit_declaration(&mut self, it: &Declaration<'a>) {
        walk_declaration(self, it);
    }

    #[inline]
    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        walk_variable_declaration(self, it);
    }

    #[inline]
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        walk_variable_declarator(self, it);
    }

    #[inline]
    fn visit_empty_statement(&mut self, it: &EmptyStatement) {
        walk_empty_statement(self, it);
    }

    #[inline]
    fn visit_expression_statement(&mut self, it: &ExpressionStatement<'a>) {
        walk_expression_statement(self, it);
    }

    #[inline]
    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        walk_if_statement(self, it);
    }

    #[inline]
    fn visit_do_while_statement(&mut self, it: &DoWhileStatement<'a>) {
        walk_do_while_statement(self, it);
    }

    #[inline]
    fn visit_while_statement(&mut self, it: &WhileStatement<'a>) {
        walk_while_statement(self, it);
    }

    #[inline]
    fn visit_for_statement(&mut self, it: &ForStatement<'a>) {
        walk_for_statement(self, it);
    }

    #[inline]
    fn visit_for_statement_init(&mut self, it: &ForStatementInit<'a>) {
        walk_for_statement_init(self, it);
    }

    #[inline]
    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        walk_for_in_statement(self, it);
    }

    #[inline]
    fn visit_for_statement_left(&mut self, it: &ForStatementLeft<'a>) {
        walk_for_statement_left(self, it);
    }

    #[inline]
    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        walk_for_of_statement(self, it);
    }

    #[inline]
    fn visit_continue_statement(&mut self, it: &ContinueStatement<'a>) {
        walk_continue_statement(self, it);
    }

    #[inline]
    fn visit_break_statement(&mut self, it: &BreakStatement<'a>) {
        walk_break_statement(self, it);
    }

    #[inline]
    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        walk_return_statement(self, it);
    }

    #[inline]
    fn visit_with_statement(&mut self, it: &WithStatement<'a>) {
        walk_with_statement(self, it);
    }

    #[inline]
    fn visit_switch_statement(&mut self, it: &SwitchStatement<'a>) {
        walk_switch_statement(self, it);
    }

    #[inline]
    fn visit_switch_case(&mut self, it: &SwitchCase<'a>) {
        walk_switch_case(self, it);
    }

    #[inline]
    fn visit_labeled_statement(&mut self, it: &LabeledStatement<'a>) {
        walk_labeled_statement(self, it);
    }

    #[inline]
    fn visit_throw_statement(&mut self, it: &ThrowStatement<'a>) {
        walk_throw_statement(self, it);
    }

    #[inline]
    fn visit_try_statement(&mut self, it: &TryStatement<'a>) {
        walk_try_statement(self, it);
    }

    #[inline]
    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        walk_catch_clause(self, it);
    }

    #[inline]
    fn visit_catch_parameter(&mut self, it: &CatchParameter<'a>) {
        walk_catch_parameter(self, it);
    }

    #[inline]
    fn visit_debugger_statement(&mut self, it: &DebuggerStatement) {
        walk_debugger_statement(self, it);
    }

    #[inline]
    fn visit_binding_pattern(&mut self, it: &BindingPattern<'a>) {
        walk_binding_pattern(self, it);
    }

    #[inline]
    fn visit_assignment_pattern(&mut self, it: &AssignmentPattern<'a>) {
        walk_assignment_pattern(self, it);
    }

    #[inline]
    fn visit_object_pattern(&mut self, it: &ObjectPattern<'a>) {
        walk_object_pattern(self, it);
    }

    #[inline]
    fn visit_binding_property(&mut self, it: &BindingProperty<'a>) {
        walk_binding_property(self, it);
    }

    #[inline]
    fn visit_array_pattern(&mut self, it: &ArrayPattern<'a>) {
        walk_array_pattern(self, it);
    }

    #[inline]
    fn visit_binding_rest_element(&mut self, it: &BindingRestElement<'a>) {
        walk_binding_rest_element(self, it);
    }

    #[inline]
    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        walk_function(self, it, flags);
    }

    #[inline]
    fn visit_formal_parameters(&mut self, it: &FormalParameters<'a>) {
        walk_formal_parameters(self, it);
    }

    #[inline]
    fn visit_formal_parameter(&mut self, it: &FormalParameter<'a>) {
        walk_formal_parameter(self, it);
    }

    #[inline]
    fn visit_formal_parameter_rest(&mut self, it: &FormalParameterRest<'a>) {
        walk_formal_parameter_rest(self, it);
    }

    #[inline]
    fn visit_function_body(&mut self, it: &FunctionBody<'a>) {
        walk_function_body(self, it);
    }

    #[inline]
    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        walk_arrow_function_expression(self, it);
    }

    #[inline]
    fn visit_yield_expression(&mut self, it: &YieldExpression<'a>) {
        walk_yield_expression(self, it);
    }

    #[inline]
    fn visit_class(&mut self, it: &Class<'a>) {
        walk_class(self, it);
    }

    #[inline]
    fn visit_class_body(&mut self, it: &ClassBody<'a>) {
        walk_class_body(self, it);
    }

    #[inline]
    fn visit_class_element(&mut self, it: &ClassElement<'a>) {
        walk_class_element(self, it);
    }

    #[inline]
    fn visit_method_definition(&mut self, it: &MethodDefinition<'a>) {
        walk_method_definition(self, it);
    }

    #[inline]
    fn visit_property_definition(&mut self, it: &PropertyDefinition<'a>) {
        walk_property_definition(self, it);
    }

    #[inline]
    fn visit_private_identifier(&mut self, it: &PrivateIdentifier<'a>) {
        walk_private_identifier(self, it);
    }

    #[inline]
    fn visit_static_block(&mut self, it: &StaticBlock<'a>) {
        walk_static_block(self, it);
    }

    #[inline]
    fn visit_module_declaration(&mut self, it: &ModuleDeclaration<'a>) {
        walk_module_declaration(self, it);
    }

    #[inline]
    fn visit_accessor_property(&mut self, it: &AccessorProperty<'a>) {
        walk_accessor_property(self, it);
    }

    #[inline]
    fn visit_import_expression(&mut self, it: &ImportExpression<'a>) {
        walk_import_expression(self, it);
    }

    #[inline]
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        walk_import_declaration(self, it);
    }

    #[inline]
    fn visit_import_declaration_specifier(&mut self, it: &ImportDeclarationSpecifier<'a>) {
        walk_import_declaration_specifier(self, it);
    }

    #[inline]
    fn visit_import_specifier(&mut self, it: &ImportSpecifier<'a>) {
        walk_import_specifier(self, it);
    }

    #[inline]
    fn visit_import_default_specifier(&mut self, it: &ImportDefaultSpecifier<'a>) {
        walk_import_default_specifier(self, it);
    }

    #[inline]
    fn visit_import_namespace_specifier(&mut self, it: &ImportNamespaceSpecifier<'a>) {
        walk_import_namespace_specifier(self, it);
    }

    #[inline]
    fn visit_with_clause(&mut self, it: &WithClause<'a>) {
        walk_with_clause(self, it);
    }

    #[inline]
    fn visit_import_attribute(&mut self, it: &ImportAttribute<'a>) {
        walk_import_attribute(self, it);
    }

    #[inline]
    fn visit_import_attribute_key(&mut self, it: &ImportAttributeKey<'a>) {
        walk_import_attribute_key(self, it);
    }

    #[inline]
    fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
        walk_export_named_declaration(self, it);
    }

    #[inline]
    fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
        walk_export_default_declaration(self, it);
    }

    #[inline]
    fn visit_export_all_declaration(&mut self, it: &ExportAllDeclaration<'a>) {
        walk_export_all_declaration(self, it);
    }

    #[inline]
    fn visit_export_specifier(&mut self, it: &ExportSpecifier<'a>) {
        walk_export_specifier(self, it);
    }

    #[inline]
    fn visit_export_default_declaration_kind(&mut self, it: &ExportDefaultDeclarationKind<'a>) {
        walk_export_default_declaration_kind(self, it);
    }

    #[inline]
    fn visit_module_export_name(&mut self, it: &ModuleExportName<'a>) {
        walk_module_export_name(self, it);
    }

    #[inline]
    fn visit_v8_intrinsic_expression(&mut self, it: &V8IntrinsicExpression<'a>) {
        walk_v8_intrinsic_expression(self, it);
    }

    #[inline]
    fn visit_boolean_literal(&mut self, it: &BooleanLiteral) {
        walk_boolean_literal(self, it);
    }

    #[inline]
    fn visit_null_literal(&mut self, it: &NullLiteral) {
        walk_null_literal(self, it);
    }

    #[inline]
    fn visit_numeric_literal(&mut self, it: &NumericLiteral<'a>) {
        walk_numeric_literal(self, it);
    }

    #[inline]
    fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
        walk_string_literal(self, it);
    }

    #[inline]
    fn visit_big_int_literal(&mut self, it: &BigIntLiteral<'a>) {
        walk_big_int_literal(self, it);
    }

    #[inline]
    fn visit_reg_exp_literal(&mut self, it: &RegExpLiteral<'a>) {
        walk_reg_exp_literal(self, it);
    }

    #[inline]
    fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
        walk_jsx_element(self, it);
    }

    #[inline]
    fn visit_jsx_opening_element(&mut self, it: &JSXOpeningElement<'a>) {
        walk_jsx_opening_element(self, it);
    }

    #[inline]
    fn visit_jsx_closing_element(&mut self, it: &JSXClosingElement<'a>) {
        walk_jsx_closing_element(self, it);
    }

    #[inline]
    fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
        walk_jsx_fragment(self, it);
    }

    #[inline]
    fn visit_jsx_opening_fragment(&mut self, it: &JSXOpeningFragment) {
        walk_jsx_opening_fragment(self, it);
    }

    #[inline]
    fn visit_jsx_closing_fragment(&mut self, it: &JSXClosingFragment) {
        walk_jsx_closing_fragment(self, it);
    }

    #[inline]
    fn visit_jsx_element_name(&mut self, it: &JSXElementName<'a>) {
        walk_jsx_element_name(self, it);
    }

    #[inline]
    fn visit_jsx_namespaced_name(&mut self, it: &JSXNamespacedName<'a>) {
        walk_jsx_namespaced_name(self, it);
    }

    #[inline]
    fn visit_jsx_member_expression(&mut self, it: &JSXMemberExpression<'a>) {
        walk_jsx_member_expression(self, it);
    }

    #[inline]
    fn visit_jsx_member_expression_object(&mut self, it: &JSXMemberExpressionObject<'a>) {
        walk_jsx_member_expression_object(self, it);
    }

    #[inline]
    fn visit_jsx_expression_container(&mut self, it: &JSXExpressionContainer<'a>) {
        walk_jsx_expression_container(self, it);
    }

    #[inline]
    fn visit_jsx_expression(&mut self, it: &JSXExpression<'a>) {
        walk_jsx_expression(self, it);
    }

    #[inline]
    fn visit_jsx_empty_expression(&mut self, it: &JSXEmptyExpression) {
        walk_jsx_empty_expression(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_item(&mut self, it: &JSXAttributeItem<'a>) {
        walk_jsx_attribute_item(self, it);
    }

    #[inline]
    fn visit_jsx_attribute(&mut self, it: &JSXAttribute<'a>) {
        walk_jsx_attribute(self, it);
    }

    #[inline]
    fn visit_jsx_spread_attribute(&mut self, it: &JSXSpreadAttribute<'a>) {
        walk_jsx_spread_attribute(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_name(&mut self, it: &JSXAttributeName<'a>) {
        walk_jsx_attribute_name(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_value(&mut self, it: &JSXAttributeValue<'a>) {
        walk_jsx_attribute_value(self, it);
    }

    #[inline]
    fn visit_jsx_identifier(&mut self, it: &JSXIdentifier<'a>) {
        walk_jsx_identifier(self, it);
    }

    #[inline]
    fn visit_jsx_child(&mut self, it: &JSXChild<'a>) {
        walk_jsx_child(self, it);
    }

    #[inline]
    fn visit_jsx_spread_child(&mut self, it: &JSXSpreadChild<'a>) {
        walk_jsx_spread_child(self, it);
    }

    #[inline]
    fn visit_jsx_text(&mut self, it: &JSXText<'a>) {
        walk_jsx_text(self, it);
    }

    #[inline]
    fn visit_ts_enum_declaration(&mut self, it: &TSEnumDeclaration<'a>) {
        walk_ts_enum_declaration(self, it);
    }

    #[inline]
    fn visit_ts_enum_body(&mut self, it: &TSEnumBody<'a>) {
        walk_ts_enum_body(self, it);
    }

    #[inline]
    fn visit_ts_enum_member(&mut self, it: &TSEnumMember<'a>) {
        walk_ts_enum_member(self, it);
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        walk_ts_enum_member_name(self, it);
    }

    #[inline]
    fn visit_ts_type_name(&mut self, it: &TSTypeName<'a>) {
        walk_ts_type_name(self, it);
    }

    #[inline]
    fn visit_ts_qualified_name(&mut self, it: &TSQualifiedName<'a>) {
        walk_ts_qualified_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration(&mut self, it: &TSModuleDeclaration<'a>) {
        walk_ts_module_declaration(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_name(&mut self, it: &TSModuleDeclarationName<'a>) {
        walk_ts_module_declaration_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_body(&mut self, it: &TSModuleDeclarationBody<'a>) {
        walk_ts_module_declaration_body(self, it);
    }

    #[inline]
    fn visit_ts_global_declaration(&mut self, it: &TSGlobalDeclaration<'a>) {
        walk_ts_global_declaration(self, it);
    }

    #[inline]
    fn visit_ts_module_block(&mut self, it: &TSModuleBlock<'a>) {
        walk_ts_module_block(self, it);
    }

    #[inline]
    fn visit_ts_as_expression(&mut self, it: &TSAsExpression<'a>) {
        walk_ts_as_expression(self, it);
    }

    #[inline]
    fn visit_ts_satisfies_expression(&mut self, it: &TSSatisfiesExpression<'a>) {
        walk_ts_satisfies_expression(self, it);
    }

    #[inline]
    fn visit_ts_type_assertion(&mut self, it: &TSTypeAssertion<'a>) {
        walk_ts_type_assertion(self, it);
    }

    #[inline]
    fn visit_ts_import_equals_declaration(&mut self, it: &TSImportEqualsDeclaration<'a>) {
        walk_ts_import_equals_declaration(self, it);
    }

    #[inline]
    fn visit_ts_module_reference(&mut self, it: &TSModuleReference<'a>) {
        walk_ts_module_reference(self, it);
    }

    #[inline]
    fn visit_ts_external_module_reference(&mut self, it: &TSExternalModuleReference<'a>) {
        walk_ts_external_module_reference(self, it);
    }

    #[inline]
    fn visit_ts_non_null_expression(&mut self, it: &TSNonNullExpression<'a>) {
        walk_ts_non_null_expression(self, it);
    }

    #[inline]
    fn visit_decorator(&mut self, it: &Decorator<'a>) {
        walk_decorator(self, it);
    }

    #[inline]
    fn visit_ts_export_assignment(&mut self, it: &TSExportAssignment<'a>) {
        walk_ts_export_assignment(self, it);
    }

    #[inline]
    fn visit_ts_instantiation_expression(&mut self, it: &TSInstantiationExpression<'a>) {
        walk_ts_instantiation_expression(self, it);
    }

    #[inline]
    fn visit_span(&mut self, it: &Span) {
        walk_span(self, it);
    }

    #[inline]
    fn visit_directives(&mut self, it: &ArenaVec<'a, Directive<'a>>) {
        walk_directives(self, it);
    }

    #[inline]
    fn visit_statements(&mut self, it: &ArenaVec<'a, Statement<'a>>) {
        walk_statements(self, it);
    }

    #[inline]
    fn visit_array_expression_elements(&mut self, it: &ArenaVec<'a, ArrayExpressionElement<'a>>) {
        walk_array_expression_elements(self, it);
    }

    #[inline]
    fn visit_object_property_kinds(&mut self, it: &ArenaVec<'a, ObjectPropertyKind<'a>>) {
        walk_object_property_kinds(self, it);
    }

    #[inline]
    fn visit_template_elements(&mut self, it: &ArenaVec<'a, TemplateElement<'a>>) {
        walk_template_elements(self, it);
    }

    #[inline]
    fn visit_expressions(&mut self, it: &ArenaVec<'a, Expression<'a>>) {
        walk_expressions(self, it);
    }

    #[inline]
    fn visit_arguments(&mut self, it: &ArenaVec<'a, Argument<'a>>) {
        walk_arguments(self, it);
    }

    #[inline]
    fn visit_assignment_target_properties(
        &mut self,
        it: &ArenaVec<'a, AssignmentTargetProperty<'a>>,
    ) {
        walk_assignment_target_properties(self, it);
    }

    #[inline]
    fn visit_variable_declarators(&mut self, it: &ArenaVec<'a, VariableDeclarator<'a>>) {
        walk_variable_declarators(self, it);
    }

    #[inline]
    fn visit_switch_cases(&mut self, it: &ArenaVec<'a, SwitchCase<'a>>) {
        walk_switch_cases(self, it);
    }

    #[inline]
    fn visit_binding_properties(&mut self, it: &ArenaVec<'a, BindingProperty<'a>>) {
        walk_binding_properties(self, it);
    }

    #[inline]
    fn visit_formal_parameter_list(&mut self, it: &ArenaVec<'a, FormalParameter<'a>>) {
        walk_formal_parameter_list(self, it);
    }

    #[inline]
    fn visit_decorators(&mut self, it: &ArenaVec<'a, Decorator<'a>>) {
        walk_decorators(self, it);
    }

    #[inline]
    fn visit_class_elements(&mut self, it: &ArenaVec<'a, ClassElement<'a>>) {
        walk_class_elements(self, it);
    }

    #[inline]
    fn visit_import_declaration_specifiers(
        &mut self,
        it: &ArenaVec<'a, ImportDeclarationSpecifier<'a>>,
    ) {
        walk_import_declaration_specifiers(self, it);
    }

    #[inline]
    fn visit_import_attributes(&mut self, it: &ArenaVec<'a, ImportAttribute<'a>>) {
        walk_import_attributes(self, it);
    }

    #[inline]
    fn visit_export_specifiers(&mut self, it: &ArenaVec<'a, ExportSpecifier<'a>>) {
        walk_export_specifiers(self, it);
    }

    #[inline]
    fn visit_jsx_children(&mut self, it: &ArenaVec<'a, JSXChild<'a>>) {
        walk_jsx_children(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_items(&mut self, it: &ArenaVec<'a, JSXAttributeItem<'a>>) {
        walk_jsx_attribute_items(self, it);
    }

    #[inline]
    fn visit_ts_enum_members(&mut self, it: &ArenaVec<'a, TSEnumMember<'a>>) {
        walk_ts_enum_members(self, it);
    }

    #[inline]
    fn visit_spans(&mut self, it: &ArenaVec<'a, Span>) {
        walk_spans(self, it);
    }
}

pub mod walk_js {
    use super::*;

    #[inline]
    pub fn walk_program<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Program<'a>) {
        let kind = AstKind::Program(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(
            {
                let mut flags = ScopeFlags::Top;
                if it.source_type.is_strict() || it.has_use_strict_directive() {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &it.scope_id,
        );
        visitor.visit_span(&it.span);
        if let Some(hashbang) = &it.hashbang {
            visitor.visit_hashbang(hashbang);
        }
        visitor.visit_directives(&it.directives);
        visitor.visit_statements(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    pub fn walk_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Expression<'a>) {
        // No `AstKind` for this type
        match it {
            Expression::BooleanLiteral(it) => visitor.visit_boolean_literal(it),
            Expression::NullLiteral(it) => visitor.visit_null_literal(it),
            Expression::NumericLiteral(it) => visitor.visit_numeric_literal(it),
            Expression::BigIntLiteral(it) => visitor.visit_big_int_literal(it),
            Expression::RegExpLiteral(it) => visitor.visit_reg_exp_literal(it),
            Expression::StringLiteral(it) => visitor.visit_string_literal(it),
            Expression::TemplateLiteral(it) => visitor.visit_template_literal(it),
            Expression::Identifier(it) => visitor.visit_identifier_reference(it),
            Expression::MetaProperty(it) => visitor.visit_meta_property(it),
            Expression::Super(it) => visitor.visit_super(it),
            Expression::ArrayExpression(it) => visitor.visit_array_expression(it),
            Expression::ArrowFunctionExpression(it) => visitor.visit_arrow_function_expression(it),
            Expression::AssignmentExpression(it) => visitor.visit_assignment_expression(it),
            Expression::AwaitExpression(it) => visitor.visit_await_expression(it),
            Expression::BinaryExpression(it) => visitor.visit_binary_expression(it),
            Expression::CallExpression(it) => visitor.visit_call_expression(it),
            Expression::ChainExpression(it) => visitor.visit_chain_expression(it),
            Expression::ClassExpression(it) => visitor.visit_class(it),
            Expression::ConditionalExpression(it) => visitor.visit_conditional_expression(it),
            Expression::FunctionExpression(it) => {
                let flags = ScopeFlags::Function;
                visitor.visit_function(it, flags)
            }
            Expression::ImportExpression(it) => visitor.visit_import_expression(it),
            Expression::LogicalExpression(it) => visitor.visit_logical_expression(it),
            Expression::NewExpression(it) => visitor.visit_new_expression(it),
            Expression::ObjectExpression(it) => visitor.visit_object_expression(it),
            Expression::ParenthesizedExpression(it) => visitor.visit_parenthesized_expression(it),
            Expression::SequenceExpression(it) => visitor.visit_sequence_expression(it),
            Expression::TaggedTemplateExpression(it) => {
                visitor.visit_tagged_template_expression(it)
            }
            Expression::ThisExpression(it) => visitor.visit_this_expression(it),
            Expression::UnaryExpression(it) => visitor.visit_unary_expression(it),
            Expression::UpdateExpression(it) => visitor.visit_update_expression(it),
            Expression::YieldExpression(it) => visitor.visit_yield_expression(it),
            Expression::PrivateInExpression(it) => visitor.visit_private_in_expression(it),
            Expression::JSXElement(it) => visitor.visit_jsx_element(it),
            Expression::JSXFragment(it) => visitor.visit_jsx_fragment(it),
            Expression::TSAsExpression(it) => visitor.visit_ts_as_expression(it),
            Expression::TSSatisfiesExpression(it) => visitor.visit_ts_satisfies_expression(it),
            Expression::TSTypeAssertion(it) => visitor.visit_ts_type_assertion(it),
            Expression::TSNonNullExpression(it) => visitor.visit_ts_non_null_expression(it),
            Expression::TSInstantiationExpression(it) => {
                visitor.visit_ts_instantiation_expression(it)
            }
            Expression::V8IntrinsicExpression(it) => visitor.visit_v8_intrinsic_expression(it),
            match_member_expression!(Expression) => {
                visitor.visit_member_expression(it.to_member_expression())
            }
        }
    }

    #[inline]
    pub fn walk_identifier_name<'a, V: VisitJs<'a>>(visitor: &mut V, it: &IdentifierName<'a>) {
        let kind = AstKind::IdentifierName(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_identifier_reference<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &IdentifierReference<'a>,
    ) {
        let kind = AstKind::IdentifierReference(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_identifier<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &BindingIdentifier<'a>,
    ) {
        let kind = AstKind::BindingIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_label_identifier<'a, V: VisitJs<'a>>(visitor: &mut V, it: &LabelIdentifier<'a>) {
        let kind = AstKind::LabelIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_this_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ThisExpression) {
        let kind = AstKind::ThisExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ArrayExpression<'a>) {
        let kind = AstKind::ArrayExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_array_expression_elements(&it.elements);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_expression_element<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArrayExpressionElement<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            ArrayExpressionElement::SpreadElement(it) => visitor.visit_spread_element(it),
            ArrayExpressionElement::Elision(it) => visitor.visit_elision(it),
            match_expression!(ArrayExpressionElement) => {
                visitor.visit_expression(it.to_expression())
            }
        }
    }

    #[inline]
    pub fn walk_elision<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Elision) {
        let kind = AstKind::Elision(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_object_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ObjectExpression<'a>) {
        let kind = AstKind::ObjectExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_object_property_kinds(&it.properties);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_object_property_kind<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ObjectPropertyKind<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            ObjectPropertyKind::ObjectProperty(it) => visitor.visit_object_property(it),
            ObjectPropertyKind::SpreadProperty(it) => visitor.visit_spread_element(it),
        }
    }

    #[inline]
    pub fn walk_object_property<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ObjectProperty<'a>) {
        let kind = AstKind::ObjectProperty(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_property_key(&it.key);
        visitor.visit_expression(&it.value);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_property_key<'a, V: VisitJs<'a>>(visitor: &mut V, it: &PropertyKey<'a>) {
        // No `AstKind` for this type
        match it {
            PropertyKey::StaticIdentifier(it) => visitor.visit_identifier_name(it),
            PropertyKey::PrivateIdentifier(it) => visitor.visit_private_identifier(it),
            match_expression!(PropertyKey) => visitor.visit_expression(it.to_expression()),
        }
    }

    #[inline]
    pub fn walk_template_literal<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TemplateLiteral<'a>) {
        let kind = AstKind::TemplateLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_template_elements(&it.quasis);
        visitor.visit_expressions(&it.expressions);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_tagged_template_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TaggedTemplateExpression<'a>,
    ) {
        let kind = AstKind::TaggedTemplateExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.tag);
        visitor.visit_template_literal(&it.quasi);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_template_element<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TemplateElement<'a>) {
        let kind = AstKind::TemplateElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_member_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &MemberExpression<'a>) {
        // No `AstKind` for this type
        match it {
            MemberExpression::ComputedMemberExpression(it) => {
                visitor.visit_computed_member_expression(it)
            }
            MemberExpression::StaticMemberExpression(it) => {
                visitor.visit_static_member_expression(it)
            }
            MemberExpression::PrivateFieldExpression(it) => {
                visitor.visit_private_field_expression(it)
            }
        }
    }

    #[inline]
    pub fn walk_computed_member_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ComputedMemberExpression<'a>,
    ) {
        let kind = AstKind::ComputedMemberExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.object);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_static_member_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &StaticMemberExpression<'a>,
    ) {
        let kind = AstKind::StaticMemberExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.object);
        visitor.visit_identifier_name(&it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_private_field_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &PrivateFieldExpression<'a>,
    ) {
        let kind = AstKind::PrivateFieldExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.object);
        visitor.visit_private_identifier(&it.field);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_call_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &CallExpression<'a>) {
        let kind = AstKind::CallExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.callee);
        visitor.visit_arguments(&it.arguments);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_new_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &NewExpression<'a>) {
        let kind = AstKind::NewExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.callee);
        visitor.visit_arguments(&it.arguments);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_meta_property<'a, V: VisitJs<'a>>(visitor: &mut V, it: &MetaProperty<'a>) {
        let kind = AstKind::MetaProperty(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_identifier_name(&it.meta);
        visitor.visit_identifier_name(&it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_spread_element<'a, V: VisitJs<'a>>(visitor: &mut V, it: &SpreadElement<'a>) {
        let kind = AstKind::SpreadElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_argument<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Argument<'a>) {
        // No `AstKind` for this type
        match it {
            Argument::SpreadElement(it) => visitor.visit_spread_element(it),
            match_expression!(Argument) => visitor.visit_expression(it.to_expression()),
        }
    }

    #[inline]
    pub fn walk_update_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &UpdateExpression<'a>) {
        let kind = AstKind::UpdateExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_simple_assignment_target(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_unary_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &UnaryExpression<'a>) {
        let kind = AstKind::UnaryExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binary_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &BinaryExpression<'a>) {
        let kind = AstKind::BinaryExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_private_in_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &PrivateInExpression<'a>,
    ) {
        let kind = AstKind::PrivateInExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_private_identifier(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_logical_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &LogicalExpression<'a>,
    ) {
        let kind = AstKind::LogicalExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_conditional_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ConditionalExpression<'a>,
    ) {
        let kind = AstKind::ConditionalExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.test);
        visitor.visit_expression(&it.consequent);
        visitor.visit_expression(&it.alternate);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentExpression<'a>,
    ) {
        let kind = AstKind::AssignmentExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_assignment_target(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target<'a, V: VisitJs<'a>>(visitor: &mut V, it: &AssignmentTarget<'a>) {
        // No `AstKind` for this type
        match it {
            match_simple_assignment_target!(AssignmentTarget) => {
                visitor.visit_simple_assignment_target(it.to_simple_assignment_target())
            }
            match_assignment_target_pattern!(AssignmentTarget) => {
                visitor.visit_assignment_target_pattern(it.to_assignment_target_pattern())
            }
        }
    }

    pub fn walk_simple_assignment_target<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &SimpleAssignmentTarget<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(it) => {
                visitor.visit_identifier_reference(it)
            }
            SimpleAssignmentTarget::TSAsExpression(it) => visitor.visit_ts_as_expression(it),
            SimpleAssignmentTarget::TSSatisfiesExpression(it) => {
                visitor.visit_ts_satisfies_expression(it)
            }
            SimpleAssignmentTarget::TSNonNullExpression(it) => {
                visitor.visit_ts_non_null_expression(it)
            }
            SimpleAssignmentTarget::TSTypeAssertion(it) => visitor.visit_ts_type_assertion(it),
            match_member_expression!(SimpleAssignmentTarget) => {
                visitor.visit_member_expression(it.to_member_expression())
            }
        }
    }

    #[inline]
    pub fn walk_assignment_target_pattern<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetPattern<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            AssignmentTargetPattern::ArrayAssignmentTarget(it) => {
                visitor.visit_array_assignment_target(it)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(it) => {
                visitor.visit_object_assignment_target(it)
            }
        }
    }

    #[inline]
    pub fn walk_array_assignment_target<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArrayAssignmentTarget<'a>,
    ) {
        let kind = AstKind::ArrayAssignmentTarget(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        for el in it.elements.iter().flatten() {
            visitor.visit_assignment_target_maybe_default(el);
        }
        if let Some(rest) = &it.rest {
            visitor.visit_assignment_target_rest(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_object_assignment_target<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ObjectAssignmentTarget<'a>,
    ) {
        let kind = AstKind::ObjectAssignmentTarget(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_assignment_target_properties(&it.properties);
        if let Some(rest) = &it.rest {
            visitor.visit_assignment_target_rest(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_rest<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetRest<'a>,
    ) {
        let kind = AstKind::AssignmentTargetRest(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_assignment_target(&it.target);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_maybe_default<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetMaybeDefault<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(it) => {
                visitor.visit_assignment_target_with_default(it)
            }
            match_assignment_target!(AssignmentTargetMaybeDefault) => {
                visitor.visit_assignment_target(it.to_assignment_target())
            }
        }
    }

    #[inline]
    pub fn walk_assignment_target_with_default<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetWithDefault<'a>,
    ) {
        let kind = AstKind::AssignmentTargetWithDefault(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_assignment_target(&it.binding);
        visitor.visit_expression(&it.init);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_property<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetProperty<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(it) => {
                visitor.visit_assignment_target_property_identifier(it)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(it) => {
                visitor.visit_assignment_target_property_property(it)
            }
        }
    }

    #[inline]
    pub fn walk_assignment_target_property_identifier<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        let kind = AstKind::AssignmentTargetPropertyIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_identifier_reference(&it.binding);
        if let Some(init) = &it.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_property_property<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetPropertyProperty<'a>,
    ) {
        let kind = AstKind::AssignmentTargetPropertyProperty(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_property_key(&it.name);
        visitor.visit_assignment_target_maybe_default(&it.binding);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_sequence_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &SequenceExpression<'a>,
    ) {
        let kind = AstKind::SequenceExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expressions(&it.expressions);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_super<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Super) {
        let kind = AstKind::Super(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_await_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &AwaitExpression<'a>) {
        let kind = AstKind::AwaitExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_chain_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ChainExpression<'a>) {
        let kind = AstKind::ChainExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_chain_element(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_chain_element<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ChainElement<'a>) {
        // No `AstKind` for this type
        match it {
            ChainElement::CallExpression(it) => visitor.visit_call_expression(it),
            ChainElement::TSNonNullExpression(it) => visitor.visit_ts_non_null_expression(it),
            match_member_expression!(ChainElement) => {
                visitor.visit_member_expression(it.to_member_expression())
            }
        }
    }

    #[inline]
    pub fn walk_parenthesized_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ParenthesizedExpression<'a>,
    ) {
        let kind = AstKind::ParenthesizedExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Statement<'a>) {
        // No `AstKind` for this type
        match it {
            Statement::BlockStatement(it) => visitor.visit_block_statement(it),
            Statement::BreakStatement(it) => visitor.visit_break_statement(it),
            Statement::ContinueStatement(it) => visitor.visit_continue_statement(it),
            Statement::DebuggerStatement(it) => visitor.visit_debugger_statement(it),
            Statement::DoWhileStatement(it) => visitor.visit_do_while_statement(it),
            Statement::EmptyStatement(it) => visitor.visit_empty_statement(it),
            Statement::ExpressionStatement(it) => visitor.visit_expression_statement(it),
            Statement::ForInStatement(it) => visitor.visit_for_in_statement(it),
            Statement::ForOfStatement(it) => visitor.visit_for_of_statement(it),
            Statement::ForStatement(it) => visitor.visit_for_statement(it),
            Statement::IfStatement(it) => visitor.visit_if_statement(it),
            Statement::LabeledStatement(it) => visitor.visit_labeled_statement(it),
            Statement::ReturnStatement(it) => visitor.visit_return_statement(it),
            Statement::SwitchStatement(it) => visitor.visit_switch_statement(it),
            Statement::ThrowStatement(it) => visitor.visit_throw_statement(it),
            Statement::TryStatement(it) => visitor.visit_try_statement(it),
            Statement::WhileStatement(it) => visitor.visit_while_statement(it),
            Statement::WithStatement(it) => visitor.visit_with_statement(it),
            match_declaration!(Statement) => visitor.visit_declaration(it.to_declaration()),
            match_module_declaration!(Statement) => {
                visitor.visit_module_declaration(it.to_module_declaration())
            }
        }
    }

    #[inline]
    pub fn walk_directive<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Directive<'a>) {
        let kind = AstKind::Directive(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_string_literal(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_hashbang<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Hashbang<'a>) {
        let kind = AstKind::Hashbang(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_block_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &BlockStatement<'a>) {
        let kind = AstKind::BlockStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_span(&it.span);
        visitor.visit_statements(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    pub fn walk_declaration<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Declaration<'a>) {
        // No `AstKind` for this type
        match it {
            Declaration::VariableDeclaration(it) => visitor.visit_variable_declaration(it),
            Declaration::FunctionDeclaration(it) => {
                let flags = ScopeFlags::Function;
                visitor.visit_function(it, flags)
            }
            Declaration::ClassDeclaration(it) => visitor.visit_class(it),
            Declaration::TSEnumDeclaration(it) => visitor.visit_ts_enum_declaration(it),
            Declaration::TSModuleDeclaration(it) => visitor.visit_ts_module_declaration(it),
            Declaration::TSGlobalDeclaration(it) => visitor.visit_ts_global_declaration(it),
            Declaration::TSImportEqualsDeclaration(it) => {
                visitor.visit_ts_import_equals_declaration(it)
            }
            _ => {}
        }
    }

    #[inline]
    pub fn walk_variable_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &VariableDeclaration<'a>,
    ) {
        let kind = AstKind::VariableDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_variable_declarators(&it.declarations);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_variable_declarator<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &VariableDeclarator<'a>,
    ) {
        let kind = AstKind::VariableDeclarator(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_pattern(&it.id);
        if let Some(init) = &it.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_empty_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &EmptyStatement) {
        let kind = AstKind::EmptyStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_expression_statement<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ExpressionStatement<'a>,
    ) {
        let kind = AstKind::ExpressionStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_if_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &IfStatement<'a>) {
        let kind = AstKind::IfStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.test);
        visitor.visit_statement(&it.consequent);
        if let Some(alternate) = &it.alternate {
            visitor.visit_statement(alternate);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_do_while_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_statement(&it.body);
        visitor.visit_expression(&it.test);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_while_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.test);
        visitor.visit_statement(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ForStatement<'a>) {
        let kind = AstKind::ForStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_span(&it.span);
        if let Some(init) = &it.init {
            visitor.visit_for_statement_init(init);
        }
        if let Some(test) = &it.test {
            visitor.visit_expression(test);
        }
        if let Some(update) = &it.update {
            visitor.visit_expression(update);
        }
        visitor.visit_statement(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_statement_init<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ForStatementInit<'a>) {
        // No `AstKind` for this type
        match it {
            ForStatementInit::VariableDeclaration(it) => visitor.visit_variable_declaration(it),
            match_expression!(ForStatementInit) => visitor.visit_expression(it.to_expression()),
        }
    }

    #[inline]
    pub fn walk_for_in_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_span(&it.span);
        visitor.visit_for_statement_left(&it.left);
        visitor.visit_expression(&it.right);
        visitor.visit_statement(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_statement_left<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ForStatementLeft<'a>) {
        // No `AstKind` for this type
        match it {
            ForStatementLeft::VariableDeclaration(it) => visitor.visit_variable_declaration(it),
            match_assignment_target!(ForStatementLeft) => {
                visitor.visit_assignment_target(it.to_assignment_target())
            }
        }
    }

    #[inline]
    pub fn walk_for_of_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_span(&it.span);
        visitor.visit_for_statement_left(&it.left);
        visitor.visit_expression(&it.right);
        visitor.visit_statement(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_continue_statement<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ContinueStatement<'a>,
    ) {
        let kind = AstKind::ContinueStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        if let Some(label) = &it.label {
            visitor.visit_label_identifier(label);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_break_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &BreakStatement<'a>) {
        let kind = AstKind::BreakStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        if let Some(label) = &it.label {
            visitor.visit_label_identifier(label);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_return_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ReturnStatement<'a>) {
        let kind = AstKind::ReturnStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        if let Some(argument) = &it.argument {
            visitor.visit_expression(argument);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_with_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &WithStatement<'a>) {
        let kind = AstKind::WithStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.object);
        visitor.enter_scope(ScopeFlags::With, &it.scope_id);
        visitor.visit_statement(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_switch_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.discriminant);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_switch_cases(&it.cases);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_switch_case<'a, V: VisitJs<'a>>(visitor: &mut V, it: &SwitchCase<'a>) {
        let kind = AstKind::SwitchCase(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        if let Some(test) = &it.test {
            visitor.visit_expression(test);
        }
        visitor.visit_statements(&it.consequent);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_labeled_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &LabeledStatement<'a>) {
        let kind = AstKind::LabeledStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_label_identifier(&it.label);
        visitor.visit_statement(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_throw_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ThrowStatement<'a>) {
        let kind = AstKind::ThrowStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_try_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TryStatement<'a>) {
        let kind = AstKind::TryStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_block_statement(&it.block);
        if let Some(handler) = &it.handler {
            visitor.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &it.finalizer {
            visitor.visit_block_statement(finalizer);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_catch_clause<'a, V: VisitJs<'a>>(visitor: &mut V, it: &CatchClause<'a>) {
        let kind = AstKind::CatchClause(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::CatchClause, &it.scope_id);
        visitor.visit_span(&it.span);
        if let Some(param) = &it.param {
            visitor.visit_catch_parameter(param);
        }
        visitor.visit_block_statement(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_catch_parameter<'a, V: VisitJs<'a>>(visitor: &mut V, it: &CatchParameter<'a>) {
        let kind = AstKind::CatchParameter(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_pattern(&it.pattern);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_debugger_statement<'a, V: VisitJs<'a>>(visitor: &mut V, it: &DebuggerStatement) {
        let kind = AstKind::DebuggerStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_pattern<'a, V: VisitJs<'a>>(visitor: &mut V, it: &BindingPattern<'a>) {
        // No `AstKind` for this type
        match it {
            BindingPattern::BindingIdentifier(it) => visitor.visit_binding_identifier(it),
            BindingPattern::ObjectPattern(it) => visitor.visit_object_pattern(it),
            BindingPattern::ArrayPattern(it) => visitor.visit_array_pattern(it),
            BindingPattern::AssignmentPattern(it) => visitor.visit_assignment_pattern(it),
        }
    }

    #[inline]
    pub fn walk_assignment_pattern<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &AssignmentPattern<'a>,
    ) {
        let kind = AstKind::AssignmentPattern(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_pattern(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_object_pattern<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ObjectPattern<'a>) {
        let kind = AstKind::ObjectPattern(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_properties(&it.properties);
        if let Some(rest) = &it.rest {
            visitor.visit_binding_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_property<'a, V: VisitJs<'a>>(visitor: &mut V, it: &BindingProperty<'a>) {
        let kind = AstKind::BindingProperty(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_property_key(&it.key);
        visitor.visit_binding_pattern(&it.value);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_pattern<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ArrayPattern<'a>) {
        let kind = AstKind::ArrayPattern(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        for el in it.elements.iter().flatten() {
            visitor.visit_binding_pattern(el);
        }
        if let Some(rest) = &it.rest {
            visitor.visit_binding_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_rest_element<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &BindingRestElement<'a>,
    ) {
        let kind = AstKind::BindingRestElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_pattern(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_function<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &Function<'a>,
        flags: ScopeFlags,
    ) {
        let kind = AstKind::Function(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(
            {
                let mut flags = flags;
                if it.has_use_strict_directive() {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &it.scope_id,
        );
        visitor.visit_span(&it.span);
        if let Some(id) = &it.id {
            visitor.visit_binding_identifier(id);
        }
        visitor.visit_formal_parameters(&it.params);
        if let Some(body) = &it.body {
            visitor.visit_function_body(body);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_formal_parameters<'a, V: VisitJs<'a>>(visitor: &mut V, it: &FormalParameters<'a>) {
        let kind = AstKind::FormalParameters(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_formal_parameter_list(&it.items);
        if let Some(rest) = &it.rest {
            visitor.visit_formal_parameter_rest(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_formal_parameter<'a, V: VisitJs<'a>>(visitor: &mut V, it: &FormalParameter<'a>) {
        let kind = AstKind::FormalParameter(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_decorators(&it.decorators);
        visitor.visit_binding_pattern(&it.pattern);
        if let Some(initializer) = &it.initializer {
            visitor.visit_expression(initializer);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_formal_parameter_rest<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &FormalParameterRest<'a>,
    ) {
        let kind = AstKind::FormalParameterRest(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_decorators(&it.decorators);
        visitor.visit_binding_rest_element(&it.rest);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_function_body<'a, V: VisitJs<'a>>(visitor: &mut V, it: &FunctionBody<'a>) {
        let kind = AstKind::FunctionBody(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_directives(&it.directives);
        visitor.visit_statements(&it.statements);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_arrow_function_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArrowFunctionExpression<'a>,
    ) {
        let kind = AstKind::ArrowFunctionExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(
            {
                let mut flags = ScopeFlags::Function | ScopeFlags::Arrow;
                if it.has_use_strict_directive() {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &it.scope_id,
        );
        visitor.visit_span(&it.span);
        visitor.visit_formal_parameters(&it.params);
        visitor.visit_function_body(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_yield_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &YieldExpression<'a>) {
        let kind = AstKind::YieldExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        if let Some(argument) = &it.argument {
            visitor.visit_expression(argument);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_class<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Class<'a>) {
        let kind = AstKind::Class(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_decorators(&it.decorators);
        if let Some(id) = &it.id {
            visitor.visit_binding_identifier(id);
        }
        visitor.enter_scope(ScopeFlags::StrictMode, &it.scope_id);
        if let Some(super_class) = &it.super_class {
            visitor.visit_expression(super_class);
        }
        visitor.visit_class_body(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_class_body<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ClassBody<'a>) {
        let kind = AstKind::ClassBody(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_class_elements(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_class_element<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ClassElement<'a>) {
        // No `AstKind` for this type
        match it {
            ClassElement::StaticBlock(it) => visitor.visit_static_block(it),
            ClassElement::MethodDefinition(it) => visitor.visit_method_definition(it),
            ClassElement::PropertyDefinition(it) => visitor.visit_property_definition(it),
            ClassElement::AccessorProperty(it) => visitor.visit_accessor_property(it),
            _ => {}
        }
    }

    #[inline]
    pub fn walk_method_definition<'a, V: VisitJs<'a>>(visitor: &mut V, it: &MethodDefinition<'a>) {
        let kind = AstKind::MethodDefinition(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_decorators(&it.decorators);
        visitor.visit_property_key(&it.key);
        {
            let flags = match it.kind {
                MethodDefinitionKind::Get => ScopeFlags::Function | ScopeFlags::GetAccessor,
                MethodDefinitionKind::Set => ScopeFlags::Function | ScopeFlags::SetAccessor,
                MethodDefinitionKind::Constructor => ScopeFlags::Function | ScopeFlags::Constructor,
                MethodDefinitionKind::Method => ScopeFlags::Function,
            };
            visitor.visit_function(&it.value, flags);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_property_definition<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &PropertyDefinition<'a>,
    ) {
        let kind = AstKind::PropertyDefinition(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_decorators(&it.decorators);
        visitor.visit_property_key(&it.key);
        if let Some(value) = &it.value {
            visitor.visit_expression(value);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_private_identifier<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &PrivateIdentifier<'a>,
    ) {
        let kind = AstKind::PrivateIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_static_block<'a, V: VisitJs<'a>>(visitor: &mut V, it: &StaticBlock<'a>) {
        let kind = AstKind::StaticBlock(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::ClassStaticBlock, &it.scope_id);
        visitor.visit_span(&it.span);
        visitor.visit_statements(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_module_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ModuleDeclaration<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            ModuleDeclaration::ImportDeclaration(it) => visitor.visit_import_declaration(it),
            ModuleDeclaration::ExportAllDeclaration(it) => visitor.visit_export_all_declaration(it),
            ModuleDeclaration::ExportDefaultDeclaration(it) => {
                visitor.visit_export_default_declaration(it)
            }
            ModuleDeclaration::ExportNamedDeclaration(it) => {
                visitor.visit_export_named_declaration(it)
            }
            ModuleDeclaration::TSExportAssignment(it) => visitor.visit_ts_export_assignment(it),
            _ => {}
        }
    }

    #[inline]
    pub fn walk_accessor_property<'a, V: VisitJs<'a>>(visitor: &mut V, it: &AccessorProperty<'a>) {
        let kind = AstKind::AccessorProperty(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_decorators(&it.decorators);
        visitor.visit_property_key(&it.key);
        if let Some(value) = &it.value {
            visitor.visit_expression(value);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ImportExpression<'a>) {
        let kind = AstKind::ImportExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.source);
        if let Some(options) = &it.options {
            visitor.visit_expression(options);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ImportDeclaration<'a>,
    ) {
        let kind = AstKind::ImportDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        if let Some(specifiers) = &it.specifiers {
            visitor.visit_import_declaration_specifiers(specifiers);
        }
        visitor.visit_string_literal(&it.source);
        if let Some(with_clause) = &it.with_clause {
            visitor.visit_with_clause(with_clause);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_declaration_specifier<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ImportDeclarationSpecifier<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            ImportDeclarationSpecifier::ImportSpecifier(it) => visitor.visit_import_specifier(it),
            ImportDeclarationSpecifier::ImportDefaultSpecifier(it) => {
                visitor.visit_import_default_specifier(it)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(it) => {
                visitor.visit_import_namespace_specifier(it)
            }
        }
    }

    #[inline]
    pub fn walk_import_specifier<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ImportSpecifier<'a>) {
        let kind = AstKind::ImportSpecifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_module_export_name(&it.imported);
        visitor.visit_binding_identifier(&it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_default_specifier<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ImportDefaultSpecifier<'a>,
    ) {
        let kind = AstKind::ImportDefaultSpecifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_identifier(&it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_namespace_specifier<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ImportNamespaceSpecifier<'a>,
    ) {
        let kind = AstKind::ImportNamespaceSpecifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_identifier(&it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_with_clause<'a, V: VisitJs<'a>>(visitor: &mut V, it: &WithClause<'a>) {
        let kind = AstKind::WithClause(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_import_attributes(&it.with_entries);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_attribute<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ImportAttribute<'a>) {
        let kind = AstKind::ImportAttribute(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_import_attribute_key(&it.key);
        visitor.visit_string_literal(&it.value);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_attribute_key<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ImportAttributeKey<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            ImportAttributeKey::Identifier(it) => visitor.visit_identifier_name(it),
            ImportAttributeKey::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_export_named_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ExportNamedDeclaration<'a>,
    ) {
        let kind = AstKind::ExportNamedDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        if let Some(declaration) = &it.declaration {
            visitor.visit_declaration(declaration);
        }
        visitor.visit_export_specifiers(&it.specifiers);
        if let Some(source) = &it.source {
            visitor.visit_string_literal(source);
        }
        if let Some(with_clause) = &it.with_clause {
            visitor.visit_with_clause(with_clause);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_export_default_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ExportDefaultDeclaration<'a>,
    ) {
        let kind = AstKind::ExportDefaultDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_export_default_declaration_kind(&it.declaration);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_export_all_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ExportAllDeclaration<'a>,
    ) {
        let kind = AstKind::ExportAllDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        if let Some(exported) = &it.exported {
            visitor.visit_module_export_name(exported);
        }
        visitor.visit_string_literal(&it.source);
        if let Some(with_clause) = &it.with_clause {
            visitor.visit_with_clause(with_clause);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_export_specifier<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ExportSpecifier<'a>) {
        let kind = AstKind::ExportSpecifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_module_export_name(&it.local);
        visitor.visit_module_export_name(&it.exported);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_export_default_declaration_kind<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ExportDefaultDeclarationKind<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            ExportDefaultDeclarationKind::FunctionDeclaration(it) => {
                let flags = ScopeFlags::Function;
                visitor.visit_function(it, flags)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(it) => visitor.visit_class(it),
            match_expression!(ExportDefaultDeclarationKind) => {
                visitor.visit_expression(it.to_expression())
            }
            _ => {}
        }
    }

    #[inline]
    pub fn walk_module_export_name<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ModuleExportName<'a>) {
        // No `AstKind` for this type
        match it {
            ModuleExportName::IdentifierName(it) => visitor.visit_identifier_name(it),
            ModuleExportName::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            ModuleExportName::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_v8_intrinsic_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &V8IntrinsicExpression<'a>,
    ) {
        let kind = AstKind::V8IntrinsicExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_identifier_name(&it.name);
        visitor.visit_arguments(&it.arguments);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_boolean_literal<'a, V: VisitJs<'a>>(visitor: &mut V, it: &BooleanLiteral) {
        let kind = AstKind::BooleanLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_null_literal<'a, V: VisitJs<'a>>(visitor: &mut V, it: &NullLiteral) {
        let kind = AstKind::NullLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_numeric_literal<'a, V: VisitJs<'a>>(visitor: &mut V, it: &NumericLiteral<'a>) {
        let kind = AstKind::NumericLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_string_literal<'a, V: VisitJs<'a>>(visitor: &mut V, it: &StringLiteral<'a>) {
        let kind = AstKind::StringLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_big_int_literal<'a, V: VisitJs<'a>>(visitor: &mut V, it: &BigIntLiteral<'a>) {
        let kind = AstKind::BigIntLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_reg_exp_literal<'a, V: VisitJs<'a>>(visitor: &mut V, it: &RegExpLiteral<'a>) {
        let kind = AstKind::RegExpLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_element<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXElement<'a>) {
        let kind = AstKind::JSXElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_jsx_opening_element(&it.opening_element);
        visitor.visit_jsx_children(&it.children);
        if let Some(closing_element) = &it.closing_element {
            visitor.visit_jsx_closing_element(closing_element);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_opening_element<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &JSXOpeningElement<'a>,
    ) {
        let kind = AstKind::JSXOpeningElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_jsx_element_name(&it.name);
        visitor.visit_jsx_attribute_items(&it.attributes);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_closing_element<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &JSXClosingElement<'a>,
    ) {
        let kind = AstKind::JSXClosingElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_jsx_element_name(&it.name);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_fragment<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXFragment<'a>) {
        let kind = AstKind::JSXFragment(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_jsx_opening_fragment(&it.opening_fragment);
        visitor.visit_jsx_children(&it.children);
        visitor.visit_jsx_closing_fragment(&it.closing_fragment);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_opening_fragment<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXOpeningFragment) {
        let kind = AstKind::JSXOpeningFragment(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_closing_fragment<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXClosingFragment) {
        let kind = AstKind::JSXClosingFragment(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_element_name<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXElementName<'a>) {
        // No `AstKind` for this type
        match it {
            JSXElementName::Identifier(it) => visitor.visit_jsx_identifier(it),
            JSXElementName::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            JSXElementName::NamespacedName(it) => visitor.visit_jsx_namespaced_name(it),
            JSXElementName::MemberExpression(it) => visitor.visit_jsx_member_expression(it),
            JSXElementName::ThisExpression(it) => visitor.visit_this_expression(it),
        }
    }

    #[inline]
    pub fn walk_jsx_namespaced_name<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &JSXNamespacedName<'a>,
    ) {
        let kind = AstKind::JSXNamespacedName(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_jsx_identifier(&it.namespace);
        visitor.visit_jsx_identifier(&it.name);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_member_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &JSXMemberExpression<'a>,
    ) {
        let kind = AstKind::JSXMemberExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_jsx_member_expression_object(&it.object);
        visitor.visit_jsx_identifier(&it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_member_expression_object<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &JSXMemberExpressionObject<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            JSXMemberExpressionObject::IdentifierReference(it) => {
                visitor.visit_identifier_reference(it)
            }
            JSXMemberExpressionObject::MemberExpression(it) => {
                visitor.visit_jsx_member_expression(it)
            }
            JSXMemberExpressionObject::ThisExpression(it) => visitor.visit_this_expression(it),
        }
    }

    #[inline]
    pub fn walk_jsx_expression_container<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &JSXExpressionContainer<'a>,
    ) {
        let kind = AstKind::JSXExpressionContainer(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_jsx_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXExpression<'a>) {
        // No `AstKind` for this type
        match it {
            JSXExpression::EmptyExpression(it) => visitor.visit_jsx_empty_expression(it),
            match_expression!(JSXExpression) => visitor.visit_expression(it.to_expression()),
        }
    }

    #[inline]
    pub fn walk_jsx_empty_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXEmptyExpression) {
        let kind = AstKind::JSXEmptyExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_attribute_item<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXAttributeItem<'a>) {
        // No `AstKind` for this type
        match it {
            JSXAttributeItem::Attribute(it) => visitor.visit_jsx_attribute(it),
            JSXAttributeItem::SpreadAttribute(it) => visitor.visit_jsx_spread_attribute(it),
        }
    }

    #[inline]
    pub fn walk_jsx_attribute<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXAttribute<'a>) {
        let kind = AstKind::JSXAttribute(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_jsx_attribute_name(&it.name);
        if let Some(value) = &it.value {
            visitor.visit_jsx_attribute_value(value);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_spread_attribute<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &JSXSpreadAttribute<'a>,
    ) {
        let kind = AstKind::JSXSpreadAttribute(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_attribute_name<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXAttributeName<'a>) {
        // No `AstKind` for this type
        match it {
            JSXAttributeName::Identifier(it) => visitor.visit_jsx_identifier(it),
            JSXAttributeName::NamespacedName(it) => visitor.visit_jsx_namespaced_name(it),
        }
    }

    #[inline]
    pub fn walk_jsx_attribute_value<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &JSXAttributeValue<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            JSXAttributeValue::StringLiteral(it) => visitor.visit_string_literal(it),
            JSXAttributeValue::ExpressionContainer(it) => {
                visitor.visit_jsx_expression_container(it)
            }
            JSXAttributeValue::Element(it) => visitor.visit_jsx_element(it),
            JSXAttributeValue::Fragment(it) => visitor.visit_jsx_fragment(it),
        }
    }

    #[inline]
    pub fn walk_jsx_identifier<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXIdentifier<'a>) {
        let kind = AstKind::JSXIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_child<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXChild<'a>) {
        // No `AstKind` for this type
        match it {
            JSXChild::Text(it) => visitor.visit_jsx_text(it),
            JSXChild::Element(it) => visitor.visit_jsx_element(it),
            JSXChild::Fragment(it) => visitor.visit_jsx_fragment(it),
            JSXChild::ExpressionContainer(it) => visitor.visit_jsx_expression_container(it),
            JSXChild::Spread(it) => visitor.visit_jsx_spread_child(it),
        }
    }

    #[inline]
    pub fn walk_jsx_spread_child<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXSpreadChild<'a>) {
        let kind = AstKind::JSXSpreadChild(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_text<'a, V: VisitJs<'a>>(visitor: &mut V, it: &JSXText<'a>) {
        let kind = AstKind::JSXText(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_enum_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSEnumDeclaration<'a>,
    ) {
        let kind = AstKind::TSEnumDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_identifier(&it.id);
        visitor.visit_ts_enum_body(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_enum_body<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TSEnumBody<'a>) {
        let kind = AstKind::TSEnumBody(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_span(&it.span);
        visitor.visit_ts_enum_members(&it.members);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_enum_member<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TSEnumMember<'a>) {
        let kind = AstKind::TSEnumMember(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_ts_enum_member_name(&it.id);
        if let Some(initializer) = &it.initializer {
            visitor.visit_expression(initializer);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_enum_member_name<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSEnumMemberName<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            TSEnumMemberName::Identifier(it) => visitor.visit_identifier_name(it),
            TSEnumMemberName::String(it) => visitor.visit_string_literal(it),
            TSEnumMemberName::ComputedString(it) => visitor.visit_string_literal(it),
            TSEnumMemberName::ComputedTemplateString(it) => visitor.visit_template_literal(it),
        }
    }

    #[inline]
    pub fn walk_ts_type_name<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TSTypeName<'a>) {
        // No `AstKind` for this type
        match it {
            TSTypeName::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            TSTypeName::QualifiedName(it) => visitor.visit_ts_qualified_name(it),
            TSTypeName::ThisExpression(it) => visitor.visit_this_expression(it),
        }
    }

    #[inline]
    pub fn walk_ts_qualified_name<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TSQualifiedName<'a>) {
        let kind = AstKind::TSQualifiedName(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_ts_type_name(&it.left);
        visitor.visit_identifier_name(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_module_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSModuleDeclaration<'a>,
    ) {
        let kind = AstKind::TSModuleDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_ts_module_declaration_name(&it.id);
        visitor.enter_scope(
            {
                let mut flags = ScopeFlags::TsModuleBlock;
                if it.body.as_ref().is_some_and(TSModuleDeclarationBody::has_use_strict_directive) {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &it.scope_id,
        );
        if let Some(body) = &it.body {
            visitor.visit_ts_module_declaration_body(body);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_module_declaration_name<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSModuleDeclarationName<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            TSModuleDeclarationName::Identifier(it) => visitor.visit_binding_identifier(it),
            TSModuleDeclarationName::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_ts_module_declaration_body<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSModuleDeclarationBody<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            TSModuleDeclarationBody::TSModuleDeclaration(it) => {
                visitor.visit_ts_module_declaration(it)
            }
            TSModuleDeclarationBody::TSModuleBlock(it) => visitor.visit_ts_module_block(it),
        }
    }

    #[inline]
    pub fn walk_ts_global_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSGlobalDeclaration<'a>,
    ) {
        let kind = AstKind::TSGlobalDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::TsModuleBlock, &it.scope_id);
        visitor.visit_span(&it.span);
        visitor.visit_span(&it.global_span);
        visitor.visit_ts_module_block(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_module_block<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TSModuleBlock<'a>) {
        let kind = AstKind::TSModuleBlock(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_directives(&it.directives);
        visitor.visit_statements(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_as_expression<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TSAsExpression<'a>) {
        let kind = AstKind::TSAsExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_satisfies_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSSatisfiesExpression<'a>,
    ) {
        let kind = AstKind::TSSatisfiesExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_assertion<'a, V: VisitJs<'a>>(visitor: &mut V, it: &TSTypeAssertion<'a>) {
        let kind = AstKind::TSTypeAssertion(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_import_equals_declaration<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSImportEqualsDeclaration<'a>,
    ) {
        let kind = AstKind::TSImportEqualsDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_binding_identifier(&it.id);
        visitor.visit_ts_module_reference(&it.module_reference);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_module_reference<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSModuleReference<'a>,
    ) {
        // No `AstKind` for this type
        match it {
            TSModuleReference::ExternalModuleReference(it) => {
                visitor.visit_ts_external_module_reference(it)
            }
            TSModuleReference::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            TSModuleReference::QualifiedName(it) => visitor.visit_ts_qualified_name(it),
        }
    }

    #[inline]
    pub fn walk_ts_external_module_reference<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSExternalModuleReference<'a>,
    ) {
        let kind = AstKind::TSExternalModuleReference(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_string_literal(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_non_null_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSNonNullExpression<'a>,
    ) {
        let kind = AstKind::TSNonNullExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_decorator<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Decorator<'a>) {
        let kind = AstKind::Decorator(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_export_assignment<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSExportAssignment<'a>,
    ) {
        let kind = AstKind::TSExportAssignment(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_instantiation_expression<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &TSInstantiationExpression<'a>,
    ) {
        let kind = AstKind::TSInstantiationExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_span(&it.span);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_span<'a, V: VisitJs<'a>>(visitor: &mut V, it: &Span) {
        // No `AstKind` for this type
    }

    #[inline]
    pub fn walk_directives<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ArenaVec<'a, Directive<'a>>) {
        for el in it {
            visitor.visit_directive(el);
        }
    }

    #[inline]
    pub fn walk_statements<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ArenaVec<'a, Statement<'a>>) {
        for el in it {
            visitor.visit_statement(el);
        }
    }

    #[inline]
    pub fn walk_array_expression_elements<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, ArrayExpressionElement<'a>>,
    ) {
        for el in it {
            visitor.visit_array_expression_element(el);
        }
    }

    #[inline]
    pub fn walk_object_property_kinds<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, ObjectPropertyKind<'a>>,
    ) {
        for el in it {
            visitor.visit_object_property_kind(el);
        }
    }

    #[inline]
    pub fn walk_template_elements<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, TemplateElement<'a>>,
    ) {
        for el in it {
            visitor.visit_template_element(el);
        }
    }

    #[inline]
    pub fn walk_expressions<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, Expression<'a>>,
    ) {
        for el in it {
            visitor.visit_expression(el);
        }
    }

    #[inline]
    pub fn walk_arguments<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ArenaVec<'a, Argument<'a>>) {
        for el in it {
            match el {
                oxc_ast::ast::Argument::SpreadElement(spread) => {
                    visitor.visit_spread_element(spread);
                }
                _ => {
                    visitor.visit_expression(el.to_expression());
                }
            }
        }
    }

    #[inline]
    pub fn walk_assignment_target_properties<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, AssignmentTargetProperty<'a>>,
    ) {
        for el in it {
            visitor.visit_assignment_target_property(el);
        }
    }

    #[inline]
    pub fn walk_variable_declarators<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, VariableDeclarator<'a>>,
    ) {
        for el in it {
            visitor.visit_variable_declarator(el);
        }
    }

    #[inline]
    pub fn walk_switch_cases<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, SwitchCase<'a>>,
    ) {
        for el in it {
            visitor.visit_switch_case(el);
        }
    }

    #[inline]
    pub fn walk_binding_properties<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, BindingProperty<'a>>,
    ) {
        for el in it {
            visitor.visit_binding_property(el);
        }
    }

    #[inline]
    pub fn walk_formal_parameter_list<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, FormalParameter<'a>>,
    ) {
        for el in it {
            visitor.visit_formal_parameter(el);
        }
    }

    #[inline]
    pub fn walk_decorators<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ArenaVec<'a, Decorator<'a>>) {
        for el in it {
            visitor.visit_decorator(el);
        }
    }

    #[inline]
    pub fn walk_class_elements<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, ClassElement<'a>>,
    ) {
        for el in it {
            visitor.visit_class_element(el);
        }
    }

    #[inline]
    pub fn walk_import_declaration_specifiers<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, ImportDeclarationSpecifier<'a>>,
    ) {
        for el in it {
            visitor.visit_import_declaration_specifier(el);
        }
    }

    #[inline]
    pub fn walk_import_attributes<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, ImportAttribute<'a>>,
    ) {
        for el in it {
            visitor.visit_import_attribute(el);
        }
    }

    #[inline]
    pub fn walk_export_specifiers<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, ExportSpecifier<'a>>,
    ) {
        for el in it {
            visitor.visit_export_specifier(el);
        }
    }

    #[inline]
    pub fn walk_jsx_children<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ArenaVec<'a, JSXChild<'a>>) {
        for el in it {
            visitor.visit_jsx_child(el);
        }
    }

    #[inline]
    pub fn walk_jsx_attribute_items<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, JSXAttributeItem<'a>>,
    ) {
        for el in it {
            visitor.visit_jsx_attribute_item(el);
        }
    }

    #[inline]
    pub fn walk_ts_enum_members<'a, V: VisitJs<'a>>(
        visitor: &mut V,
        it: &ArenaVec<'a, TSEnumMember<'a>>,
    ) {
        for el in it {
            visitor.visit_ts_enum_member(el);
        }
    }

    #[inline]
    pub fn walk_spans<'a, V: VisitJs<'a>>(visitor: &mut V, it: &ArenaVec<'a, Span>) {
        for el in it {
            visitor.visit_span(el);
        }
    }
}
