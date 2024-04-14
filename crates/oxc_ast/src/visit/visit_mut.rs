//! Visit Mut Pattern

use oxc_allocator::Vec;
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use crate::{ast::*, AstType};

#[allow(clippy::wildcard_imports)]
use self::walk_mut::*;

/// Syntax tree traversal to mutate an exclusive borrow of a syntax tree in place.
pub trait VisitMut<'a>: Sized {
    fn enter_node(&mut self, _kind: AstType) {}
    fn leave_node(&mut self, _kind: AstType) {}

    fn enter_scope(&mut self, _flags: ScopeFlags) {}
    fn leave_scope(&mut self) {}

    fn visit_program(&mut self, program: &mut Program<'a>) {
        walk_program_mut(self, program);
    }

    /* ----------  Statement ---------- */

    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        walk_statements_mut(self, stmts);
    }

    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        walk_statement_mut(self, stmt);
    }

    fn visit_block_statement(&mut self, stmt: &mut BlockStatement<'a>) {
        walk_block_statement_mut(self, stmt);
    }

    fn visit_break_statement(&mut self, stmt: &mut BreakStatement<'a>) {
        walk_break_statement_mut(self, stmt);
    }

    fn visit_continue_statement(&mut self, stmt: &mut ContinueStatement<'a>) {
        walk_continue_statement_mut(self, stmt);
    }

    fn visit_debugger_statement(&mut self, stmt: &mut DebuggerStatement) {
        walk_debugger_statement_mut(self, stmt);
    }

    fn visit_do_while_statement(&mut self, stmt: &mut DoWhileStatement<'a>) {
        walk_do_while_statement_mut(self, stmt);
    }

    fn visit_empty_statement(&mut self, stmt: &mut EmptyStatement) {
        walk_empty_statement_mut(self, stmt);
    }

    fn visit_expression_statement(&mut self, stmt: &mut ExpressionStatement<'a>) {
        walk_expression_statement_mut(self, stmt);
    }

    fn visit_for_statement(&mut self, stmt: &mut ForStatement<'a>) {
        walk_for_statement_mut(self, stmt);
    }

    fn visit_for_statement_init(&mut self, init: &mut ForStatementInit<'a>) {
        walk_for_statement_init_mut(self, init);
    }

    fn visit_for_in_statement(&mut self, stmt: &mut ForInStatement<'a>) {
        walk_for_in_statement_mut(self, stmt);
    }

    fn visit_for_of_statement(&mut self, stmt: &mut ForOfStatement<'a>) {
        walk_for_of_statement_mut(self, stmt);
    }

    fn visit_for_statement_left(&mut self, left: &mut ForStatementLeft<'a>) {
        walk_for_statement_left_mut(self, left);
    }

    fn visit_if_statement(&mut self, stmt: &mut IfStatement<'a>) {
        walk_if_statement_mut(self, stmt);
    }

    fn visit_labeled_statement(&mut self, stmt: &mut LabeledStatement<'a>) {
        walk_labeled_statement_mut(self, stmt);
    }

    fn visit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>) {
        walk_return_statement_mut(self, stmt);
    }

    fn visit_switch_statement(&mut self, stmt: &mut SwitchStatement<'a>) {
        walk_switch_statement_mut(self, stmt);
    }

    fn visit_switch_case(&mut self, case: &mut SwitchCase<'a>) {
        walk_switch_case_mut(self, case);
    }

    fn visit_throw_statement(&mut self, stmt: &mut ThrowStatement<'a>) {
        walk_throw_statement_mut(self, stmt);
    }

    fn visit_try_statement(&mut self, stmt: &mut TryStatement<'a>) {
        walk_try_statement_mut(self, stmt);
    }

    fn visit_catch_clause(&mut self, clause: &mut CatchClause<'a>) {
        walk_catch_clause_mut(self, clause);
    }

    fn visit_finally_clause(&mut self, clause: &mut BlockStatement<'a>) {
        walk_finally_clause_mut(self, clause);
    }

    fn visit_while_statement(&mut self, stmt: &mut WhileStatement<'a>) {
        walk_while_statement_mut(self, stmt);
    }

    fn visit_with_statement(&mut self, stmt: &mut WithStatement<'a>) {
        walk_with_statement_mut(self, stmt);
    }

    fn visit_directive(&mut self, directive: &mut Directive<'a>) {
        walk_directive_mut(self, directive);
    }

    /* ----------  Declaration ---------- */

    fn visit_variable_declaration(&mut self, decl: &mut VariableDeclaration<'a>) {
        walk_variable_declaration_mut(self, decl);
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        walk_variable_declarator_mut(self, declarator);
    }

    fn visit_using_declaration(&mut self, declaration: &mut UsingDeclaration<'a>) {
        walk_using_declaration_mut(self, declaration);
    }

    /* ----------  Function ---------- */

    fn visit_function(&mut self, func: &mut Function<'a>, flags: Option<ScopeFlags>) {
        walk_function_mut(self, func, flags);
    }

    fn visit_function_body(&mut self, body: &mut FunctionBody<'a>) {
        walk_function_body_mut(self, body);
    }

    fn visit_formal_parameters(&mut self, params: &mut FormalParameters<'a>) {
        walk_formal_parameters_mut(self, params);
    }

    fn visit_formal_parameter(&mut self, param: &mut FormalParameter<'a>) {
        walk_formal_parameter_mut(self, param);
    }

    /* ----------  Class ---------- */

    fn visit_decorator(&mut self, decorator: &mut Decorator<'a>) {
        walk_decorator_mut(self, decorator);
    }

    fn visit_class(&mut self, class: &mut Class<'a>) {
        walk_class_mut(self, class);
    }

    fn visit_class_heritage(&mut self, expr: &mut Expression<'a>) {
        walk_class_heritage_mut(self, expr);
    }

    fn visit_class_body(&mut self, body: &mut ClassBody<'a>) {
        walk_class_body_mut(self, body);
    }

    fn visit_class_element(&mut self, elem: &mut ClassElement<'a>) {
        walk_class_element_mut(self, elem);
    }

    fn visit_static_block(&mut self, block: &mut StaticBlock<'a>) {
        walk_static_block_mut(self, block);
    }

    fn visit_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        walk_method_definition_mut(self, def);
    }

    fn visit_property_definition(&mut self, def: &mut PropertyDefinition<'a>) {
        walk_property_definition_mut(self, def);
    }

    /* ----------  Expression ---------- */

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        walk_expression_mut(self, expr);
    }

    fn visit_meta_property(&mut self, meta: &mut MetaProperty<'a>) {
        walk_meta_property_mut(self, meta);
    }

    fn visit_array_expression(&mut self, expr: &mut ArrayExpression<'a>) {
        walk_array_expression_mut(self, expr);
    }

    fn visit_array_expression_element(&mut self, arg: &mut ArrayExpressionElement<'a>) {
        walk_array_expression_element_mut(self, arg);
    }

    fn visit_argument(&mut self, arg: &mut Argument<'a>) {
        walk_argument_mut(self, arg);
    }

    fn visit_spread_element(&mut self, elem: &mut SpreadElement<'a>) {
        walk_spread_element_mut(self, elem);
    }

    fn visit_expression_array_element(&mut self, expr: &mut Expression<'a>) {
        walk_expression_array_element_mut(self, expr);
    }

    fn visit_elision(&mut self, span: Span) {
        walk_elision_mut(self, span);
    }

    fn visit_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        walk_assignment_expression_mut(self, expr);
    }

    fn visit_arrow_expression(&mut self, expr: &mut ArrowFunctionExpression<'a>) {
        walk_arrow_expression_mut(self, expr);
    }

    fn visit_await_expression(&mut self, expr: &mut AwaitExpression<'a>) {
        walk_await_expression_mut(self, expr);
    }

    fn visit_binary_expression(&mut self, expr: &mut BinaryExpression<'a>) {
        walk_binary_expression_mut(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &mut CallExpression<'a>) {
        walk_call_expression_mut(self, expr);
    }

    fn visit_chain_expression(&mut self, expr: &mut ChainExpression<'a>) {
        walk_chain_expression_mut(self, expr);
    }

    fn visit_chain_element(&mut self, elem: &mut ChainElement<'a>) {
        walk_chain_element_mut(self, elem);
    }

    fn visit_conditional_expression(&mut self, expr: &mut ConditionalExpression<'a>) {
        walk_conditional_expression_mut(self, expr);
    }

    fn visit_import_expression(&mut self, expr: &mut ImportExpression<'a>) {
        walk_import_expression_mut(self, expr);
    }

    fn visit_logical_expression(&mut self, expr: &mut LogicalExpression<'a>) {
        walk_logical_expression_mut(self, expr);
    }

    fn visit_member_expression(&mut self, expr: &mut MemberExpression<'a>) {
        walk_member_expression_mut(self, expr);
    }

    fn visit_computed_member_expression(&mut self, expr: &mut ComputedMemberExpression<'a>) {
        walk_computed_member_expression_mut(self, expr);
    }

    fn visit_static_member_expression(&mut self, expr: &mut StaticMemberExpression<'a>) {
        walk_static_member_expression_mut(self, expr);
    }

    fn visit_private_field_expression(&mut self, expr: &mut PrivateFieldExpression<'a>) {
        walk_private_field_expression_mut(self, expr);
    }

    fn visit_new_expression(&mut self, expr: &mut NewExpression<'a>) {
        walk_new_expression_mut(self, expr);
    }

    fn visit_object_expression(&mut self, expr: &mut ObjectExpression<'a>) {
        walk_object_expression_mut(self, expr);
    }

    fn visit_object_property_kind(&mut self, prop: &mut ObjectPropertyKind<'a>) {
        walk_object_property_kind_mut(self, prop);
    }

    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        walk_object_property_mut(self, prop);
    }

    fn visit_property_key(&mut self, key: &mut PropertyKey<'a>) {
        walk_property_key_mut(self, key);
    }

    fn visit_parenthesized_expression(&mut self, expr: &mut ParenthesizedExpression<'a>) {
        walk_parenthesized_expression_mut(self, expr);
    }

    fn visit_private_in_expression(&mut self, expr: &mut PrivateInExpression<'a>) {
        walk_private_in_expression_mut(self, expr);
    }

    fn visit_sequence_expression(&mut self, expr: &mut SequenceExpression<'a>) {
        walk_sequence_expression_mut(self, expr);
    }

    fn visit_tagged_template_expression(&mut self, expr: &mut TaggedTemplateExpression<'a>) {
        walk_tagged_template_expression_mut(self, expr);
    }

    fn visit_this_expression(&mut self, expr: &mut ThisExpression) {
        walk_this_expression_mut(self, expr);
    }

    fn visit_unary_expression(&mut self, expr: &mut UnaryExpression<'a>) {
        walk_unary_expression_mut(self, expr);
    }

    fn visit_update_expression(&mut self, expr: &mut UpdateExpression<'a>) {
        walk_update_expression_mut(self, expr);
    }

    fn visit_yield_expression(&mut self, expr: &mut YieldExpression<'a>) {
        walk_yield_expression_mut(self, expr);
    }

    fn visit_super(&mut self, expr: &mut Super) {
        walk_super_mut(self, expr);
    }

    fn visit_assignment_target(&mut self, target: &mut AssignmentTarget<'a>) {
        walk_assignment_target_mut(self, target);
    }

    fn visit_simple_assignment_target(&mut self, target: &mut SimpleAssignmentTarget<'a>) {
        walk_simple_assignment_target_mut(self, target);
    }

    fn visit_assignment_target_pattern(&mut self, pat: &mut AssignmentTargetPattern<'a>) {
        walk_assignment_target_pattern_mut(self, pat);
    }

    fn visit_array_assignment_target(&mut self, target: &mut ArrayAssignmentTarget<'a>) {
        walk_array_assignment_target_mut(self, target);
    }

    fn visit_assignment_target_maybe_default(
        &mut self,
        target: &mut AssignmentTargetMaybeDefault<'a>,
    ) {
        walk_assignment_target_maybe_default_mut(self, target);
    }

    fn visit_assignment_target_with_default(
        &mut self,
        target: &mut AssignmentTargetWithDefault<'a>,
    ) {
        walk_assignment_target_with_default_mut(self, target);
    }

    fn visit_object_assignment_target(&mut self, target: &mut ObjectAssignmentTarget<'a>) {
        walk_object_assignment_target_mut(self, target);
    }

    fn visit_assignment_target_property(&mut self, property: &mut AssignmentTargetProperty<'a>) {
        walk_assignment_target_property_mut(self, property);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        ident: &mut AssignmentTargetPropertyIdentifier<'a>,
    ) {
        walk_assignment_target_property_identifier_mut(self, ident);
    }

    fn visit_assignment_target_property_property(
        &mut self,
        property: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        walk_assignment_target_property_property_mut(self, property);
    }

    fn visit_assignment_target_rest(&mut self, rest: &mut AssignmentTargetRest<'a>) {
        walk_assignment_target_rest_mut(self, rest);
    }

    /* ----------  Expression ---------- */

    fn visit_jsx_element(&mut self, elem: &mut JSXElement<'a>) {
        walk_jsx_element_mut(self, elem);
    }

    fn visit_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        walk_jsx_opening_element_mut(self, elem);
    }

    fn visit_jsx_closing_element(&mut self, elem: &mut JSXClosingElement<'a>) {
        walk_jsx_closing_element_mut(self, elem);
    }

    fn visit_jsx_element_name(&mut self, name: &mut JSXElementName<'a>) {
        walk_jsx_element_name_mut(self, name);
    }

    fn visit_jsx_identifier(&mut self, ident: &mut JSXIdentifier<'a>) {
        walk_jsx_identifier_mut(self, ident);
    }

    fn visit_jsx_member_expression(&mut self, expr: &mut JSXMemberExpression<'a>) {
        walk_jsx_member_expression_mut(self, expr);
    }

    fn visit_jsx_member_expression_object(&mut self, expr: &mut JSXMemberExpressionObject<'a>) {
        walk_jsx_member_expression_object_mut(self, expr);
    }

    fn visit_jsx_namespaced_name(&mut self, name: &mut JSXNamespacedName<'a>) {
        walk_jsx_namespaced_name_mut(self, name);
    }

    fn visit_jsx_attribute_item(&mut self, item: &mut JSXAttributeItem<'a>) {
        walk_jsx_attribute_item_mut(self, item);
    }

    fn visit_jsx_attribute(&mut self, attribute: &mut JSXAttribute<'a>) {
        walk_jsx_attribute_mut(self, attribute);
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &mut JSXSpreadAttribute<'a>) {
        walk_jsx_spread_attribute_mut(self, attribute);
    }

    fn visit_jsx_attribute_value(&mut self, value: &mut JSXAttributeValue<'a>) {
        walk_jsx_attribute_value_mut(self, value);
    }

    fn visit_jsx_expression_container(&mut self, expr: &mut JSXExpressionContainer<'a>) {
        walk_jsx_expression_container_mut(self, expr);
    }

    fn visit_jsx_expression(&mut self, expr: &mut JSXExpression<'a>) {
        walk_jsx_expression_mut(self, expr);
    }

    fn visit_jsx_fragment(&mut self, elem: &mut JSXFragment<'a>) {
        walk_jsx_fragment_mut(self, elem);
    }

    fn visit_jsx_child(&mut self, child: &mut JSXChild<'a>) {
        walk_jsx_child_mut(self, child);
    }

    fn visit_jsx_spread_child(&mut self, child: &mut JSXSpreadChild<'a>) {
        walk_jsx_spread_child_mut(self, child);
    }

    fn visit_jsx_text(&mut self, child: &JSXText<'a>) {
        walk_jsx_text_mut(self, child);
    }

    /* ----------  Pattern ---------- */

    fn visit_binding_pattern(&mut self, pat: &mut BindingPattern<'a>) {
        walk_binding_pattern_mut(self, pat);
    }

    fn visit_binding_identifier(&mut self, ident: &mut BindingIdentifier<'a>) {
        walk_binding_identifier_mut(self, ident);
    }

    fn visit_object_pattern(&mut self, pat: &mut ObjectPattern<'a>) {
        walk_object_pattern_mut(self, pat);
    }

    fn visit_binding_property(&mut self, prop: &mut BindingProperty<'a>) {
        walk_binding_property_mut(self, prop);
    }

    fn visit_array_pattern(&mut self, pat: &mut ArrayPattern<'a>) {
        walk_array_pattern_mut(self, pat);
    }

    fn visit_rest_element(&mut self, pat: &mut BindingRestElement<'a>) {
        walk_rest_element_mut(self, pat);
    }

    fn visit_assignment_pattern(&mut self, pat: &mut AssignmentPattern<'a>) {
        walk_assignment_pattern_mut(self, pat);
    }

    /* ----------  Identifier ---------- */

    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        walk_identifier_reference_mut(self, ident);
    }

    fn visit_private_identifier(&mut self, ident: &mut PrivateIdentifier<'a>) {
        walk_private_identifier_mut(self, ident);
    }

    fn visit_label_identifier(&mut self, ident: &mut LabelIdentifier<'a>) {
        walk_label_identifier_mut(self, ident);
    }

    fn visit_identifier_name(&mut self, ident: &mut IdentifierName<'a>) {
        walk_identifier_name_mut(self, ident);
    }

    /* ----------  Literal ---------- */

    fn visit_number_literal(&mut self, lit: &mut NumericLiteral<'a>) {
        walk_number_literal_mut(self, lit);
    }

    fn visit_boolean_literal(&mut self, lit: &mut BooleanLiteral) {
        walk_boolean_literal_mut(self, lit);
    }

    fn visit_null_literal(&mut self, lit: &mut NullLiteral) {
        walk_null_literal_mut(self, lit);
    }

    fn visit_bigint_literal(&mut self, lit: &mut BigIntLiteral<'a>) {
        walk_bigint_literal_mut(self, lit);
    }

    fn visit_string_literal(&mut self, lit: &mut StringLiteral<'a>) {
        walk_string_literal_mut(self, lit);
    }

    fn visit_template_literal(&mut self, lit: &mut TemplateLiteral<'a>) {
        walk_template_literal_mut(self, lit);
    }

    fn visit_reg_expr_literal(&mut self, lit: &mut RegExpLiteral<'a>) {
        walk_reg_expr_literal_mut(self, lit);
    }

    fn visit_template_element(&mut self, _elem: &mut TemplateElement) {}

    /* ----------  Module ---------- */

    fn visit_module_declaration(&mut self, decl: &mut ModuleDeclaration<'a>) {
        walk_module_declaration_mut(self, decl);
    }

    fn visit_import_declaration(&mut self, decl: &mut ImportDeclaration<'a>) {
        walk_import_declaration_mut(self, decl);
    }

    fn visit_with_clause(&mut self, with_clause: &mut WithClause<'a>) {
        walk_with_clause_mut(self, with_clause);
    }

    fn visit_import_attribute(&mut self, attribute: &mut ImportAttribute<'a>) {
        walk_import_attribute_mut(self, attribute);
    }

    fn visit_import_attribute_key(&mut self, key: &mut ImportAttributeKey<'a>) {
        walk_import_attribute_key_mut(self, key);
    }

    fn visit_import_declaration_specifier(
        &mut self,
        specifier: &mut ImportDeclarationSpecifier<'a>,
    ) {
        walk_import_declaration_specifier_mut(self, specifier);
    }

    fn visit_import_specifier(&mut self, specifier: &mut ImportSpecifier<'a>) {
        walk_import_specifier_mut(self, specifier);
    }

    fn visit_import_default_specifier(&mut self, specifier: &mut ImportDefaultSpecifier<'a>) {
        walk_import_default_specifier_mut(self, specifier);
    }

    fn visit_import_name_specifier(&mut self, specifier: &mut ImportNamespaceSpecifier<'a>) {
        walk_import_name_specifier_mut(self, specifier);
    }

    fn visit_export_all_declaration(&mut self, decl: &mut ExportAllDeclaration<'a>) {
        walk_export_all_declaration_mut(self, decl);
    }

    fn visit_export_default_declaration(&mut self, decl: &mut ExportDefaultDeclaration<'a>) {
        walk_export_default_declaration_mut(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &mut ExportNamedDeclaration<'a>) {
        walk_export_named_declaration_mut(self, decl);
    }

    fn visit_enum_member(&mut self, member: &mut TSEnumMember<'a>) {
        walk_enum_member_mut(self, member);
    }

    fn visit_enum(&mut self, decl: &mut TSEnumDeclaration<'a>) {
        walk_enum_mut(self, decl);
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) {
        walk_declaration_mut(self, decl);
    }

    /* ----------  TypeScript ---------- */

    fn visit_ts_import_equals_declaration(&mut self, decl: &mut TSImportEqualsDeclaration<'a>) {
        walk_ts_import_equals_declaration_mut(self, decl);
    }

    fn visit_ts_module_reference(&mut self, reference: &mut TSModuleReference<'a>) {
        walk_ts_module_reference_mut(self, reference);
    }

    fn visit_ts_type_name(&mut self, name: &mut TSTypeName<'a>) {
        walk_ts_type_name_mut(self, name);
    }

    fn visit_ts_external_module_reference(
        &mut self,
        reference: &mut TSExternalModuleReference<'a>,
    ) {
        walk_ts_external_module_reference_mut(self, reference);
    }

    fn visit_ts_qualified_name(&mut self, name: &mut TSQualifiedName<'a>) {
        walk_ts_qualified_name_mut(self, name);
    }

    fn visit_ts_module_declaration(&mut self, decl: &mut TSModuleDeclaration<'a>) {
        walk_ts_module_declaration_mut(self, decl);
    }

    fn visit_ts_module_block(&mut self, block: &mut TSModuleBlock<'a>) {
        walk_ts_module_block_mut(self, block);
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &mut TSTypeAliasDeclaration<'a>) {
        walk_ts_type_alias_declaration_mut(self, decl);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &mut TSInterfaceDeclaration<'a>) {
        walk_ts_interface_declaration_mut(self, decl);
    }

    fn visit_ts_as_expression(&mut self, expr: &mut TSAsExpression<'a>) {
        walk_ts_as_expression_mut(self, expr);
    }

    fn visit_ts_satisfies_expression(&mut self, expr: &mut TSSatisfiesExpression<'a>) {
        walk_ts_satisfies_expression_mut(self, expr);
    }

    fn visit_ts_non_null_expression(&mut self, expr: &mut TSNonNullExpression<'a>) {
        walk_ts_non_null_expression_mut(self, expr);
    }

    fn visit_ts_type_assertion(&mut self, expr: &mut TSTypeAssertion<'a>) {
        walk_ts_type_assertion_mut(self, expr);
    }

    fn visit_ts_instantiation_expression(&mut self, expr: &mut TSInstantiationExpression<'a>) {
        walk_ts_instantiation_expression_mut(self, expr);
    }

    fn visit_ts_type_annotation(&mut self, annotation: &mut TSTypeAnnotation<'a>) {
        walk_ts_type_annotation_mut(self, annotation);
    }

    fn visit_ts_type(&mut self, ty: &mut TSType<'a>) {
        walk_ts_type_mut(self, ty);
    }

    fn visit_ts_type_literal(&mut self, ty: &mut TSTypeLiteral<'a>) {
        walk_ts_type_literal_mut(self, ty);
    }

    fn visit_ts_indexed_access_type(&mut self, ty: &mut TSIndexedAccessType<'a>) {
        walk_ts_indexed_access_type_mut(self, ty);
    }

    fn visit_ts_type_predicate(&mut self, ty: &mut TSTypePredicate<'a>) {
        walk_ts_type_predicate_mut(self, ty);
    }

    fn visit_ts_type_operator_type(&mut self, ty: &mut TSTypeOperator<'a>) {
        walk_ts_type_operator_type_mut(self, ty);
    }

    fn visit_ts_tuple_type(&mut self, ty: &mut TSTupleType<'a>) {
        walk_ts_tuple_type_mut(self, ty);
    }

    fn visit_ts_tuple_element(&mut self, ty: &mut TSTupleElement<'a>) {
        walk_ts_tuple_element_mut(self, ty);
    }

    fn visit_ts_mapped_type(&mut self, ty: &mut TSMappedType<'a>) {
        walk_ts_mapped_type_mut(self, ty);
    }

    fn visit_ts_function_type(&mut self, ty: &mut TSFunctionType<'a>) {
        walk_ts_function_type_mut(self, ty);
    }

    fn visit_ts_type_parameter(&mut self, ty: &mut TSTypeParameter<'a>) {
        walk_ts_type_parameter_mut(self, ty);
    }

    fn visit_ts_type_parameter_instantiation(&mut self, ty: &mut TSTypeParameterInstantiation<'a>) {
        walk_ts_type_parameter_instantiation_mut(self, ty);
    }

    fn visit_ts_type_parameter_declaration(&mut self, ty: &mut TSTypeParameterDeclaration<'a>) {
        walk_ts_type_parameter_declaration_mut(self, ty);
    }

    fn visit_ts_constructor_type(&mut self, ty: &mut TSConstructorType<'a>) {
        walk_ts_constructor_type_mut(self, ty);
    }

    fn visit_ts_conditional_type(&mut self, ty: &mut TSConditionalType<'a>) {
        walk_ts_conditional_type_mut(self, ty);
    }

    fn visit_ts_array_type(&mut self, ty: &mut TSArrayType<'a>) {
        walk_ts_array_type_mut(self, ty);
    }

    fn visit_ts_null_keyword(&mut self, ty: &mut TSNullKeyword) {
        walk_ts_null_keyword_mut(self, ty);
    }

    fn visit_ts_any_keyword(&mut self, ty: &mut TSAnyKeyword) {
        walk_ts_any_keyword_mut(self, ty);
    }

    fn visit_ts_void_keyword(&mut self, ty: &mut TSVoidKeyword) {
        walk_ts_void_keyword_mut(self, ty);
    }

    fn visit_ts_intersection_type(&mut self, ty: &mut TSIntersectionType<'a>) {
        walk_ts_intersection_type_mut(self, ty);
    }

    fn visit_ts_type_reference(&mut self, ty: &mut TSTypeReference<'a>) {
        walk_ts_type_reference_mut(self, ty);
    }

    fn visit_ts_union_type(&mut self, ty: &mut TSUnionType<'a>) {
        walk_ts_union_type_mut(self, ty);
    }

    fn visit_ts_literal_type(&mut self, ty: &mut TSLiteralType<'a>) {
        walk_ts_literal_type_mut(self, ty);
    }

    fn visit_ts_signature(&mut self, signature: &mut TSSignature<'a>) {
        walk_ts_signature_mut(self, signature);
    }

    fn visit_ts_construct_signature_declaration(
        &mut self,
        signature: &mut TSConstructSignatureDeclaration<'a>,
    ) {
        walk_ts_construct_signature_declaration_mut(self, signature);
    }

    fn visit_ts_method_signature(&mut self, signature: &mut TSMethodSignature<'a>) {
        walk_ts_method_signature_mut(self, signature);
    }

    fn visit_ts_index_signature_name(&mut self, name: &mut TSIndexSignatureName<'a>) {
        walk_ts_index_signature_name_mut(self, name);
    }

    fn visit_ts_index_signature(&mut self, signature: &mut TSIndexSignature<'a>) {
        walk_ts_index_signature_mut(self, signature);
    }

    fn visit_ts_property_signature(&mut self, signature: &mut TSPropertySignature<'a>) {
        walk_ts_property_signature_mut(self, signature);
    }

    fn visit_ts_call_signature_declaration(
        &mut self,
        signature: &mut TSCallSignatureDeclaration<'a>,
    ) {
        walk_ts_call_signature_declaration_mut(self, signature);
    }

    fn visit_ts_type_query(&mut self, ty: &mut TSTypeQuery<'a>) {
        walk_ts_type_query_mut(self, ty);
    }

    fn visit_ts_import_type(&mut self, ty: &mut TSImportType<'a>) {
        walk_ts_import_type_mut(self, ty);
    }

    fn visit_ts_import_attributes(&mut self, attributes: &mut TSImportAttributes<'a>) {
        walk_ts_import_attributes_mut(self, attributes);
    }

    fn visit_ts_import_attribute(&mut self, attribute: &mut TSImportAttribute<'a>) {
        walk_ts_import_attribute_mut(self, attribute);
    }

    fn visit_ts_import_attribute_name(&mut self, name: &mut TSImportAttributeName<'a>) {
        walk_ts_import_attribute_name_mut(self, name);
    }
}

pub mod walk_mut {
    use super::*;

    pub fn walk_program_mut<'a, V: VisitMut<'a>>(visitor: &mut V, program: &mut Program<'a>) {
        let kind = AstType::Program;
        visitor.enter_scope({
            let mut flags = ScopeFlags::Top;
            if program.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        visitor.enter_node(kind);
        for directive in program.directives.iter_mut() {
            visitor.visit_directive(directive);
        }
        visitor.visit_statements(&mut program.body);

        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    /* ----------  Statement ---------- */

    pub fn walk_statements_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmts: &mut Vec<'a, Statement<'a>>,
    ) {
        for stmt in stmts.iter_mut() {
            visitor.visit_statement(stmt);
        }
    }

    pub fn walk_statement_mut<'a, V: VisitMut<'a>>(visitor: &mut V, stmt: &mut Statement<'a>) {
        match stmt {
            Statement::BlockStatement(stmt) => visitor.visit_block_statement(stmt),
            Statement::BreakStatement(stmt) => visitor.visit_break_statement(stmt),
            Statement::ContinueStatement(stmt) => visitor.visit_continue_statement(stmt),
            Statement::DebuggerStatement(stmt) => visitor.visit_debugger_statement(stmt),
            Statement::DoWhileStatement(stmt) => visitor.visit_do_while_statement(stmt),
            Statement::EmptyStatement(stmt) => visitor.visit_empty_statement(stmt),
            Statement::ExpressionStatement(stmt) => visitor.visit_expression_statement(stmt),
            Statement::ForInStatement(stmt) => visitor.visit_for_in_statement(stmt),
            Statement::ForOfStatement(stmt) => visitor.visit_for_of_statement(stmt),
            Statement::ForStatement(stmt) => visitor.visit_for_statement(stmt),
            Statement::IfStatement(stmt) => visitor.visit_if_statement(stmt),
            Statement::LabeledStatement(stmt) => visitor.visit_labeled_statement(stmt),
            Statement::ReturnStatement(stmt) => visitor.visit_return_statement(stmt),
            Statement::SwitchStatement(stmt) => visitor.visit_switch_statement(stmt),
            Statement::ThrowStatement(stmt) => visitor.visit_throw_statement(stmt),
            Statement::TryStatement(stmt) => visitor.visit_try_statement(stmt),
            Statement::WhileStatement(stmt) => visitor.visit_while_statement(stmt),
            Statement::WithStatement(stmt) => visitor.visit_with_statement(stmt),

            Statement::ModuleDeclaration(decl) => visitor.visit_module_declaration(decl),
            Statement::Declaration(decl) => visitor.visit_declaration(decl),
        }
    }

    pub fn walk_block_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut BlockStatement<'a>,
    ) {
        let kind = AstType::BlockStatement;
        visitor.enter_scope(ScopeFlags::empty());
        visitor.enter_node(kind);
        visitor.visit_statements(&mut stmt.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_break_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut BreakStatement<'a>,
    ) {
        let kind = AstType::BreakStatement;
        visitor.enter_node(kind);
        if let Some(break_target) = &mut stmt.label {
            visitor.visit_label_identifier(break_target);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_continue_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut ContinueStatement<'a>,
    ) {
        let kind = AstType::ContinueStatement;
        visitor.enter_node(kind);
        if let Some(continue_target) = &mut stmt.label {
            visitor.visit_label_identifier(continue_target);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_debugger_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _stmt: &mut DebuggerStatement,
    ) {
        let kind = AstType::DebuggerStatement;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_do_while_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut DoWhileStatement<'a>,
    ) {
        let kind = AstType::DoWhileStatement;
        visitor.enter_node(kind);
        visitor.visit_statement(&mut stmt.body);
        visitor.visit_expression(&mut stmt.test);
        visitor.leave_node(kind);
    }

    pub fn walk_empty_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _stmt: &mut EmptyStatement,
    ) {
        let kind = AstType::EmptyStatement;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_expression_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut ExpressionStatement<'a>,
    ) {
        let kind = AstType::ExpressionStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut stmt.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_for_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut ForStatement<'a>,
    ) {
        let kind = AstType::ForStatement;
        let is_lexical_declaration =
            stmt.init.as_ref().is_some_and(ForStatementInit::is_lexical_declaration);
        if is_lexical_declaration {
            visitor.enter_scope(ScopeFlags::empty());
        }
        visitor.enter_node(kind);
        if let Some(init) = &mut stmt.init {
            visitor.visit_for_statement_init(init);
        }
        if let Some(test) = &mut stmt.test {
            visitor.visit_expression(test);
        }
        if let Some(update) = &mut stmt.update {
            visitor.visit_expression(update);
        }
        visitor.visit_statement(&mut stmt.body);
        visitor.leave_node(kind);
        if is_lexical_declaration {
            visitor.leave_scope();
        }
    }

    pub fn walk_for_statement_init_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        init: &mut ForStatementInit<'a>,
    ) {
        let kind = AstType::ForStatementInit;
        visitor.enter_node(kind);
        match init {
            ForStatementInit::VariableDeclaration(decl) => {
                visitor.visit_variable_declaration(decl);
            }
            ForStatementInit::Expression(expr) => visitor.visit_expression(expr),
            ForStatementInit::UsingDeclaration(decl) => {
                visitor.visit_using_declaration(decl);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_for_in_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut ForInStatement<'a>,
    ) {
        let kind = AstType::ForInStatement;
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            visitor.enter_scope(ScopeFlags::empty());
        }
        visitor.enter_node(kind);
        visitor.visit_for_statement_left(&mut stmt.left);
        visitor.visit_expression(&mut stmt.right);
        visitor.visit_statement(&mut stmt.body);
        visitor.leave_node(kind);
        if is_lexical_declaration {
            visitor.leave_scope();
        }
    }

    pub fn walk_for_of_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut ForOfStatement<'a>,
    ) {
        let kind = AstType::ForOfStatement;
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            visitor.enter_scope(ScopeFlags::empty());
        }
        visitor.enter_node(kind);
        visitor.visit_for_statement_left(&mut stmt.left);
        visitor.visit_expression(&mut stmt.right);
        visitor.visit_statement(&mut stmt.body);
        visitor.leave_node(kind);
        if is_lexical_declaration {
            visitor.leave_scope();
        }
    }

    pub fn walk_for_statement_left_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        left: &mut ForStatementLeft<'a>,
    ) {
        match left {
            ForStatementLeft::VariableDeclaration(decl) => {
                visitor.visit_variable_declaration(decl);
            }
            ForStatementLeft::AssignmentTarget(target) => visitor.visit_assignment_target(target),
            ForStatementLeft::UsingDeclaration(decl) => {
                visitor.visit_using_declaration(decl);
            }
        }
    }

    pub fn walk_if_statement_mut<'a, V: VisitMut<'a>>(visitor: &mut V, stmt: &mut IfStatement<'a>) {
        let kind = AstType::IfStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut stmt.test);
        visitor.visit_statement(&mut stmt.consequent);
        if let Some(alternate) = &mut stmt.alternate {
            visitor.visit_statement(alternate);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_labeled_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut LabeledStatement<'a>,
    ) {
        let kind = AstType::LabeledStatement;
        visitor.enter_node(kind);
        visitor.visit_label_identifier(&mut stmt.label);
        visitor.visit_statement(&mut stmt.body);
        visitor.leave_node(kind);
    }

    pub fn walk_return_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut ReturnStatement<'a>,
    ) {
        let kind = AstType::ReturnStatement;
        visitor.enter_node(kind);
        if let Some(arg) = &mut stmt.argument {
            visitor.visit_expression(arg);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_switch_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut SwitchStatement<'a>,
    ) {
        let kind = AstType::SwitchStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut stmt.discriminant);
        visitor.enter_scope(ScopeFlags::empty());
        for case in stmt.cases.iter_mut() {
            visitor.visit_switch_case(case);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    pub fn walk_switch_case_mut<'a, V: VisitMut<'a>>(visitor: &mut V, case: &mut SwitchCase<'a>) {
        let kind = AstType::SwitchCase;
        visitor.enter_node(kind);
        if let Some(expr) = &mut case.test {
            visitor.visit_expression(expr);
        }
        visitor.visit_statements(&mut case.consequent);
        visitor.leave_node(kind);
    }

    pub fn walk_throw_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut ThrowStatement<'a>,
    ) {
        let kind = AstType::ThrowStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut stmt.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_try_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut TryStatement<'a>,
    ) {
        let kind = AstType::TryStatement;
        visitor.enter_node(kind);
        visitor.visit_block_statement(&mut stmt.block);
        if let Some(handler) = &mut stmt.handler {
            visitor.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &mut stmt.finalizer {
            visitor.visit_finally_clause(finalizer);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_catch_clause_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        clause: &mut CatchClause<'a>,
    ) {
        let kind = AstType::CatchClause;
        visitor.enter_scope(ScopeFlags::empty());
        visitor.enter_node(kind);
        if let Some(param) = &mut clause.param {
            visitor.visit_binding_pattern(param);
        }
        visitor.visit_statements(&mut clause.body.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_finally_clause_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        clause: &mut BlockStatement<'a>,
    ) {
        let kind = AstType::FinallyClause;
        visitor.enter_scope(ScopeFlags::empty());
        visitor.enter_node(kind);
        visitor.visit_statements(&mut clause.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_while_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut WhileStatement<'a>,
    ) {
        let kind = AstType::WhileStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut stmt.test);
        visitor.visit_statement(&mut stmt.body);
        visitor.leave_node(kind);
    }

    pub fn walk_with_statement_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        stmt: &mut WithStatement<'a>,
    ) {
        let kind = AstType::WithStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut stmt.object);
        visitor.visit_statement(&mut stmt.body);
        visitor.leave_node(kind);
    }

    pub fn walk_directive_mut<'a, V: VisitMut<'a>>(visitor: &mut V, directive: &mut Directive<'a>) {
        let kind = AstType::Directive;
        visitor.enter_node(kind);
        visitor.visit_string_literal(&mut directive.expression);
        visitor.leave_node(kind);
    }

    /* ----------  Declaration ---------- */

    pub fn walk_variable_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut VariableDeclaration<'a>,
    ) {
        let kind = AstType::VariableDeclaration;
        visitor.enter_node(kind);
        for declarator in decl.declarations.iter_mut() {
            visitor.visit_variable_declarator(declarator);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_variable_declarator_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        declarator: &mut VariableDeclarator<'a>,
    ) {
        let kind = AstType::VariableDeclarator;
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&mut declarator.id);
        if let Some(init) = &mut declarator.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_using_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        declaration: &mut UsingDeclaration<'a>,
    ) {
        let kind = AstType::UsingDeclaration;
        visitor.enter_node(kind);
        for decl in declaration.declarations.iter_mut() {
            visitor.visit_variable_declarator(decl);
        }
        visitor.leave_node(kind);
    }

    /* ----------  Function ---------- */

    pub fn walk_function_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        func: &mut Function<'a>,
        flags: Option<ScopeFlags>,
    ) {
        let kind = AstType::Function;
        visitor.enter_scope({
            let mut flags = flags.unwrap_or(ScopeFlags::empty()) | ScopeFlags::Function;
            if func.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        visitor.enter_node(kind);
        if let Some(ident) = &mut func.id {
            visitor.visit_binding_identifier(ident);
        }
        visitor.visit_formal_parameters(&mut func.params);
        if let Some(body) = &mut func.body {
            visitor.visit_function_body(body);
        }
        if let Some(parameters) = &mut func.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &mut func.return_type {
            visitor.visit_ts_type_annotation(annotation);
        }
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_function_body_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        body: &mut FunctionBody<'a>,
    ) {
        let kind = AstType::FunctionBody;
        visitor.enter_node(kind);
        for directive in body.directives.iter_mut() {
            visitor.visit_directive(directive);
        }
        visitor.visit_statements(&mut body.statements);
        visitor.leave_node(kind);
    }

    pub fn walk_formal_parameters_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        params: &mut FormalParameters<'a>,
    ) {
        let kind = AstType::FormalParameters;
        visitor.enter_node(kind);
        for param in params.items.iter_mut() {
            visitor.visit_formal_parameter(param);
        }
        if let Some(rest) = &mut params.rest {
            visitor.visit_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_formal_parameter_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        param: &mut FormalParameter<'a>,
    ) {
        let kind = AstType::FormalParameter;
        visitor.enter_node(kind);
        for decorator in param.decorators.iter_mut() {
            visitor.visit_decorator(decorator);
        }
        visitor.visit_binding_pattern(&mut param.pattern);
        visitor.leave_node(kind);
    }

    /* ----------  Class ---------- */

    pub fn walk_decorator_mut<'a, V: VisitMut<'a>>(visitor: &mut V, decorator: &mut Decorator<'a>) {
        let kind = AstType::Decorator;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut decorator.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_class_mut<'a, V: VisitMut<'a>>(visitor: &mut V, class: &mut Class<'a>) {
        for decorator in class.decorators.iter_mut() {
            visitor.visit_decorator(decorator);
        }

        let kind = AstType::Class;

        // FIXME(don): Should we enter a scope when visiting class declarations?
        let is_class_expr = class.r#type == ClassType::ClassExpression;
        if is_class_expr {
            // Class expressions create a temporary scope with the class name as its only variable
            // E.g., `let c = class A { foo() { console.log(A) } }`
            visitor.enter_scope(ScopeFlags::empty());
        }

        visitor.enter_node(kind);
        if let Some(id) = &mut class.id {
            visitor.visit_binding_identifier(id);
        }
        if let Some(parameters) = &mut class.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(super_class) = &mut class.super_class {
            visitor.visit_class_heritage(super_class);
        }
        if let Some(super_parameters) = &mut class.super_type_parameters {
            visitor.visit_ts_type_parameter_instantiation(super_parameters);
        }
        visitor.visit_class_body(&mut class.body);
        visitor.leave_node(kind);
        if is_class_expr {
            visitor.leave_scope();
        }
    }

    pub fn walk_class_heritage_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut Expression<'a>,
    ) {
        let kind = AstType::ClassHeritage;
        visitor.enter_node(kind);
        visitor.visit_expression(expr);
        visitor.leave_node(kind);
    }

    pub fn walk_class_body_mut<'a, V: VisitMut<'a>>(visitor: &mut V, body: &mut ClassBody<'a>) {
        for elem in body.body.iter_mut() {
            visitor.visit_class_element(elem);
        }
    }

    pub fn walk_class_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        elem: &mut ClassElement<'a>,
    ) {
        match elem {
            ClassElement::StaticBlock(block) => visitor.visit_static_block(block),
            ClassElement::MethodDefinition(def) => visitor.visit_method_definition(def),
            ClassElement::PropertyDefinition(def) => visitor.visit_property_definition(def),
            ClassElement::AccessorProperty(_def) => { /* TODO */ }
            ClassElement::TSIndexSignature(sig) => visitor.visit_ts_index_signature(sig),
        }
    }

    pub fn walk_static_block_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        block: &mut StaticBlock<'a>,
    ) {
        let kind = AstType::StaticBlock;
        visitor.enter_scope(ScopeFlags::ClassStaticBlock);
        visitor.enter_node(kind);
        visitor.visit_statements(&mut block.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_method_definition_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        def: &mut MethodDefinition<'a>,
    ) {
        let kind = AstType::MethodDefinition;
        visitor.enter_node(kind);
        for decorator in def.decorators.iter_mut() {
            visitor.visit_decorator(decorator);
        }

        let flags = match def.kind {
            MethodDefinitionKind::Get => ScopeFlags::GetAccessor,
            MethodDefinitionKind::Set => ScopeFlags::SetAccessor,
            MethodDefinitionKind::Constructor => ScopeFlags::Constructor,
            MethodDefinitionKind::Method => ScopeFlags::empty(),
        };
        visitor.visit_property_key(&mut def.key);
        visitor.visit_function(&mut def.value, Some(flags));
        visitor.leave_node(kind);
    }

    pub fn walk_property_definition_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        def: &mut PropertyDefinition<'a>,
    ) {
        let kind = AstType::PropertyDefinition;
        visitor.enter_node(kind);
        for decorator in def.decorators.iter_mut() {
            visitor.visit_decorator(decorator);
        }
        visitor.visit_property_key(&mut def.key);
        if let Some(value) = &mut def.value {
            visitor.visit_expression(value);
        }
        if let Some(annotation) = &mut def.type_annotation {
            visitor.visit_ts_type_annotation(annotation);
        }
        visitor.leave_node(kind);
    }

    /* ----------  Expression ---------- */

    pub fn walk_expression_mut<'a, V: VisitMut<'a>>(visitor: &mut V, expr: &mut Expression<'a>) {
        match expr {
            Expression::BigintLiteral(lit) => visitor.visit_bigint_literal(lit),
            Expression::BooleanLiteral(lit) => visitor.visit_boolean_literal(lit),
            Expression::NullLiteral(lit) => visitor.visit_null_literal(lit),
            Expression::NumericLiteral(lit) => visitor.visit_number_literal(lit),
            Expression::RegExpLiteral(lit) => visitor.visit_reg_expr_literal(lit),
            Expression::StringLiteral(lit) => visitor.visit_string_literal(lit),
            Expression::TemplateLiteral(lit) => visitor.visit_template_literal(lit),

            Expression::Identifier(ident) => visitor.visit_identifier_reference(ident),
            Expression::MetaProperty(meta) => visitor.visit_meta_property(meta),

            Expression::ArrayExpression(expr) => visitor.visit_array_expression(expr),
            Expression::ArrowFunctionExpression(expr) => visitor.visit_arrow_expression(expr),
            Expression::AssignmentExpression(expr) => visitor.visit_assignment_expression(expr),
            Expression::AwaitExpression(expr) => visitor.visit_await_expression(expr),
            Expression::BinaryExpression(expr) => visitor.visit_binary_expression(expr),
            Expression::CallExpression(expr) => visitor.visit_call_expression(expr),
            Expression::ChainExpression(expr) => visitor.visit_chain_expression(expr),
            Expression::ClassExpression(expr) => visitor.visit_class(expr),
            Expression::ConditionalExpression(expr) => visitor.visit_conditional_expression(expr),
            Expression::FunctionExpression(expr) => visitor.visit_function(expr, None),
            Expression::ImportExpression(expr) => visitor.visit_import_expression(expr),
            Expression::LogicalExpression(expr) => visitor.visit_logical_expression(expr),
            Expression::MemberExpression(expr) => visitor.visit_member_expression(expr),
            Expression::NewExpression(expr) => visitor.visit_new_expression(expr),
            Expression::ObjectExpression(expr) => visitor.visit_object_expression(expr),
            Expression::ParenthesizedExpression(expr) => {
                visitor.visit_parenthesized_expression(expr);
            }
            Expression::PrivateInExpression(expr) => visitor.visit_private_in_expression(expr),
            Expression::SequenceExpression(expr) => visitor.visit_sequence_expression(expr),
            Expression::TaggedTemplateExpression(expr) => {
                visitor.visit_tagged_template_expression(expr);
            }
            Expression::ThisExpression(expr) => visitor.visit_this_expression(expr),
            Expression::UnaryExpression(expr) => visitor.visit_unary_expression(expr),
            Expression::UpdateExpression(expr) => visitor.visit_update_expression(expr),
            Expression::YieldExpression(expr) => visitor.visit_yield_expression(expr),
            Expression::Super(expr) => visitor.visit_super(expr),
            Expression::JSXElement(elem) => visitor.visit_jsx_element(elem),
            Expression::JSXFragment(elem) => visitor.visit_jsx_fragment(elem),

            Expression::TSAsExpression(expr) => visitor.visit_ts_as_expression(expr),
            Expression::TSSatisfiesExpression(expr) => visitor.visit_ts_satisfies_expression(expr),
            Expression::TSNonNullExpression(expr) => visitor.visit_ts_non_null_expression(expr),
            Expression::TSTypeAssertion(expr) => visitor.visit_ts_type_assertion(expr),
            Expression::TSInstantiationExpression(expr) => {
                visitor.visit_ts_instantiation_expression(expr);
            }
        }
    }

    pub fn walk_meta_property_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _meta: &mut MetaProperty<'a>,
    ) {
        let kind = AstType::MetaProperty;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_array_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut ArrayExpression<'a>,
    ) {
        let kind = AstType::ArrayExpression;
        visitor.enter_node(kind);
        for elem in expr.elements.iter_mut() {
            visitor.visit_array_expression_element(elem);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_array_expression_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        arg: &mut ArrayExpressionElement<'a>,
    ) {
        let kind = AstType::ArrayExpressionElement;
        visitor.enter_node(kind);
        match arg {
            ArrayExpressionElement::SpreadElement(spread) => visitor.visit_spread_element(spread),
            ArrayExpressionElement::Expression(expr) => {
                visitor.visit_expression_array_element(expr);
            }
            ArrayExpressionElement::Elision(span) => visitor.visit_elision(*span),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_argument_mut<'a, V: VisitMut<'a>>(visitor: &mut V, arg: &mut Argument<'a>) {
        let kind = AstType::Argument;
        visitor.enter_node(kind);
        match arg {
            Argument::SpreadElement(spread) => visitor.visit_spread_element(spread),
            Argument::Expression(expr) => visitor.visit_expression(expr),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_spread_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        elem: &mut SpreadElement<'a>,
    ) {
        let kind = AstType::SpreadElement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut elem.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_expression_array_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut Expression<'a>,
    ) {
        let kind = AstType::ExpressionArrayElement;
        visitor.enter_node(kind);
        visitor.visit_expression(expr);
        visitor.leave_node(kind);
    }

    pub fn walk_elision_mut<'a, V: VisitMut<'a>>(visitor: &mut V, _span: Span) {
        let kind = AstType::Elision;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_assignment_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut AssignmentExpression<'a>,
    ) {
        let kind = AstType::AssignmentExpression;
        visitor.enter_node(kind);
        visitor.visit_assignment_target(&mut expr.left);
        visitor.visit_expression(&mut expr.right);
        visitor.leave_node(kind);
    }

    pub fn walk_arrow_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut ArrowFunctionExpression<'a>,
    ) {
        let kind = AstType::ArrowFunctionExpression;
        visitor.enter_scope(ScopeFlags::Function | ScopeFlags::Arrow);
        visitor.enter_node(kind);
        visitor.visit_formal_parameters(&mut expr.params);
        visitor.visit_function_body(&mut expr.body);
        if let Some(parameters) = &mut expr.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_await_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut AwaitExpression<'a>,
    ) {
        let kind = AstType::AwaitExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_binary_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut BinaryExpression<'a>,
    ) {
        let kind = AstType::BinaryExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.left);
        visitor.visit_expression(&mut expr.right);
        visitor.leave_node(kind);
    }

    pub fn walk_call_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut CallExpression<'a>,
    ) {
        let kind = AstType::CallExpression;
        visitor.enter_node(kind);
        for arg in expr.arguments.iter_mut() {
            visitor.visit_argument(arg);
        }
        visitor.visit_expression(&mut expr.callee);
        if let Some(parameters) = &mut expr.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(parameters);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_chain_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut ChainExpression<'a>,
    ) {
        let kind = AstType::ChainExpression;
        visitor.enter_node(kind);
        visitor.visit_chain_element(&mut expr.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_chain_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        elem: &mut ChainElement<'a>,
    ) {
        match elem {
            ChainElement::CallExpression(expr) => visitor.visit_call_expression(expr),
            ChainElement::MemberExpression(expr) => visitor.visit_member_expression(expr),
        }
    }

    pub fn walk_conditional_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut ConditionalExpression<'a>,
    ) {
        let kind = AstType::ConditionalExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.test);
        visitor.visit_expression(&mut expr.consequent);
        visitor.visit_expression(&mut expr.alternate);
        visitor.leave_node(kind);
    }

    pub fn walk_import_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut ImportExpression<'a>,
    ) {
        visitor.visit_expression(&mut expr.source);
        for arg in expr.arguments.iter_mut() {
            visitor.visit_expression(arg);
        }
    }

    pub fn walk_logical_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut LogicalExpression<'a>,
    ) {
        let kind = AstType::LogicalExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.left);
        visitor.visit_expression(&mut expr.right);
        visitor.leave_node(kind);
    }

    pub fn walk_member_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut MemberExpression<'a>,
    ) {
        let kind = AstType::MemberExpression;
        visitor.enter_node(kind);
        match expr {
            MemberExpression::ComputedMemberExpression(expr) => {
                visitor.visit_computed_member_expression(expr);
            }
            MemberExpression::StaticMemberExpression(expr) => {
                visitor.visit_static_member_expression(expr);
            }
            MemberExpression::PrivateFieldExpression(expr) => {
                visitor.visit_private_field_expression(expr);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_computed_member_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut ComputedMemberExpression<'a>,
    ) {
        visitor.visit_expression(&mut expr.object);
        visitor.visit_expression(&mut expr.expression);
    }

    pub fn walk_static_member_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut StaticMemberExpression<'a>,
    ) {
        visitor.visit_expression(&mut expr.object);
        visitor.visit_identifier_name(&mut expr.property);
    }

    pub fn walk_private_field_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut PrivateFieldExpression<'a>,
    ) {
        visitor.visit_expression(&mut expr.object);
        visitor.visit_private_identifier(&mut expr.field);
    }

    pub fn walk_new_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut NewExpression<'a>,
    ) {
        let kind = AstType::NewExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.callee);
        if let Some(parameters) = &mut expr.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(parameters);
        }
        for arg in expr.arguments.iter_mut() {
            visitor.visit_argument(arg);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_object_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut ObjectExpression<'a>,
    ) {
        let kind = AstType::ObjectExpression;
        visitor.enter_node(kind);
        for prop in expr.properties.iter_mut() {
            visitor.visit_object_property_kind(prop);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_object_property_kind_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        prop: &mut ObjectPropertyKind<'a>,
    ) {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => visitor.visit_object_property(prop),
            ObjectPropertyKind::SpreadProperty(elem) => visitor.visit_spread_element(elem),
        }
    }

    pub fn walk_object_property_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        prop: &mut ObjectProperty<'a>,
    ) {
        let kind = AstType::ObjectProperty;
        visitor.enter_node(kind);
        visitor.visit_property_key(&mut prop.key);
        visitor.visit_expression(&mut prop.value);
        if let Some(init) = &mut prop.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_property_key_mut<'a, V: VisitMut<'a>>(visitor: &mut V, key: &mut PropertyKey<'a>) {
        let kind = AstType::PropertyKey;
        visitor.enter_node(kind);
        match key {
            PropertyKey::Identifier(ident) => visitor.visit_identifier_name(ident),
            PropertyKey::PrivateIdentifier(ident) => visitor.visit_private_identifier(ident),
            PropertyKey::Expression(expr) => visitor.visit_expression(expr),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_parenthesized_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut ParenthesizedExpression<'a>,
    ) {
        let kind = AstType::ParenthesizedExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_private_in_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut PrivateInExpression<'a>,
    ) {
        visitor.visit_private_identifier(&mut expr.left);
        visitor.visit_expression(&mut expr.right);
    }

    pub fn walk_sequence_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut SequenceExpression<'a>,
    ) {
        let kind = AstType::SequenceExpression;
        visitor.enter_node(kind);
        for expr in expr.expressions.iter_mut() {
            visitor.visit_expression(expr);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_tagged_template_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut TaggedTemplateExpression<'a>,
    ) {
        let kind = AstType::TaggedTemplateExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.tag);
        visitor.visit_template_literal(&mut expr.quasi);
        visitor.leave_node(kind);
    }

    pub fn walk_this_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _expr: &mut ThisExpression,
    ) {
        let kind = AstType::ThisExpression;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_unary_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut UnaryExpression<'a>,
    ) {
        let kind = AstType::UnaryExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_update_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut UpdateExpression<'a>,
    ) {
        let kind = AstType::UpdateExpression;
        visitor.enter_node(kind);
        visitor.visit_simple_assignment_target(&mut expr.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_yield_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut YieldExpression<'a>,
    ) {
        let kind = AstType::YieldExpression;
        visitor.enter_node(kind);
        if let Some(argument) = &mut expr.argument {
            visitor.visit_expression(argument);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_super_mut<'a, V: VisitMut<'a>>(visitor: &mut V, _expr: &mut Super) {
        let kind = AstType::Super;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_assignment_target_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        target: &mut AssignmentTarget<'a>,
    ) {
        let kind = AstType::AssignmentTarget;
        visitor.enter_node(kind);
        match target {
            AssignmentTarget::SimpleAssignmentTarget(target) => {
                visitor.visit_simple_assignment_target(target);
            }
            AssignmentTarget::AssignmentTargetPattern(pat) => {
                visitor.visit_assignment_target_pattern(pat);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_simple_assignment_target_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        target: &mut SimpleAssignmentTarget<'a>,
    ) {
        let kind = AstType::SimpleAssignmentTarget;
        visitor.enter_node(kind);
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                visitor.visit_identifier_reference(ident);
            }
            SimpleAssignmentTarget::MemberAssignmentTarget(expr) => {
                visitor.visit_member_expression(expr);
            }
            SimpleAssignmentTarget::TSAsExpression(expr) => {
                visitor.visit_expression(&mut expr.expression);
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(expr) => {
                visitor.visit_expression(&mut expr.expression);
            }
            SimpleAssignmentTarget::TSNonNullExpression(expr) => {
                visitor.visit_expression(&mut expr.expression);
            }
            SimpleAssignmentTarget::TSTypeAssertion(expr) => {
                visitor.visit_expression(&mut expr.expression);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_assignment_target_pattern_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        pat: &mut AssignmentTargetPattern<'a>,
    ) {
        match pat {
            AssignmentTargetPattern::ArrayAssignmentTarget(target) => {
                visitor.visit_array_assignment_target(target);
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(target) => {
                visitor.visit_object_assignment_target(target);
            }
        }
    }

    pub fn walk_array_assignment_target_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        target: &mut ArrayAssignmentTarget<'a>,
    ) {
        for element in target.elements.iter_mut().flatten() {
            visitor.visit_assignment_target_maybe_default(element);
        }
        if let Some(target) = &mut target.rest {
            visitor.visit_assignment_target_rest(target);
        }
    }

    pub fn walk_assignment_target_maybe_default_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        target: &mut AssignmentTargetMaybeDefault<'a>,
    ) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTarget(target) => {
                visitor.visit_assignment_target(target);
            }
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                visitor.visit_assignment_target_with_default(target);
            }
        }
    }

    pub fn walk_assignment_target_with_default_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        target: &mut AssignmentTargetWithDefault<'a>,
    ) {
        let kind = AstType::AssignmentTargetWithDefault;
        visitor.enter_node(kind);
        visitor.visit_assignment_target(&mut target.binding);
        visitor.visit_expression(&mut target.init);
        visitor.leave_node(kind);
    }

    pub fn walk_object_assignment_target_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        target: &mut ObjectAssignmentTarget<'a>,
    ) {
        for property in target.properties.iter_mut() {
            visitor.visit_assignment_target_property(property);
        }
        if let Some(target) = &mut target.rest {
            visitor.visit_assignment_target_rest(target);
        }
    }

    pub fn walk_assignment_target_property_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        property: &mut AssignmentTargetProperty<'a>,
    ) {
        match property {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                visitor.visit_assignment_target_property_identifier(ident);
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                visitor.visit_assignment_target_property_property(prop);
            }
        }
    }

    pub fn walk_assignment_target_property_identifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ident: &mut AssignmentTargetPropertyIdentifier<'a>,
    ) {
        visitor.visit_identifier_reference(&mut ident.binding);
        if let Some(expr) = &mut ident.init {
            visitor.visit_expression(expr);
        }
    }

    pub fn walk_assignment_target_property_property_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        property: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        visitor.visit_property_key(&mut property.name);
        visitor.visit_assignment_target_maybe_default(&mut property.binding);
    }

    pub fn walk_assignment_target_rest_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        rest: &mut AssignmentTargetRest<'a>,
    ) {
        visitor.visit_assignment_target(&mut rest.target);
    }

    /* ----------  Expression ---------- */

    pub fn walk_jsx_element_mut<'a, V: VisitMut<'a>>(visitor: &mut V, elem: &mut JSXElement<'a>) {
        let kind = AstType::JSXElement;
        visitor.enter_node(kind);
        visitor.visit_jsx_opening_element(&mut elem.opening_element);
        for child in elem.children.iter_mut() {
            visitor.visit_jsx_child(child);
        }
        if let Some(closing_elem) = &mut elem.closing_element {
            visitor.visit_jsx_closing_element(closing_elem);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_opening_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        elem: &mut JSXOpeningElement<'a>,
    ) {
        let kind = AstType::JSXOpeningElement;
        visitor.enter_node(kind);

        visitor.visit_jsx_element_name(&mut elem.name);
        for attribute in elem.attributes.iter_mut() {
            visitor.visit_jsx_attribute_item(attribute);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_closing_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        elem: &mut JSXClosingElement<'a>,
    ) {
        let kind = AstType::JSXClosingElement;
        visitor.enter_node(kind);
        visitor.visit_jsx_element_name(&mut elem.name);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_element_name_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        name: &mut JSXElementName<'a>,
    ) {
        let kind = AstType::JSXElementName;
        visitor.enter_node(kind);
        match name {
            JSXElementName::Identifier(ident) => visitor.visit_jsx_identifier(ident),
            JSXElementName::MemberExpression(expr) => visitor.visit_jsx_member_expression(expr),
            JSXElementName::NamespacedName(name) => visitor.visit_jsx_namespaced_name(name),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_identifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _ident: &mut JSXIdentifier<'a>,
    ) {
        let kind = AstType::JSXIdentifier;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_member_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut JSXMemberExpression<'a>,
    ) {
        let kind = AstType::JSXMemberExpression;
        visitor.enter_node(kind);
        visitor.visit_jsx_member_expression_object(&mut expr.object);
        visitor.visit_jsx_identifier(&mut expr.property);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_member_expression_object_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut JSXMemberExpressionObject<'a>,
    ) {
        let kind = AstType::JSXMemberExpressionObject;
        visitor.enter_node(kind);
        match expr {
            JSXMemberExpressionObject::Identifier(ident) => visitor.visit_jsx_identifier(ident),
            JSXMemberExpressionObject::MemberExpression(expr) => {
                visitor.visit_jsx_member_expression(expr);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_namespaced_name_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        name: &mut JSXNamespacedName<'a>,
    ) {
        let kind = AstType::JSXNamespacedName;
        visitor.enter_node(kind);
        visitor.visit_jsx_identifier(&mut name.namespace);
        visitor.visit_jsx_identifier(&mut name.property);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_attribute_item_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        item: &mut JSXAttributeItem<'a>,
    ) {
        let kind = AstType::JSXAttributeItem;
        visitor.enter_node(kind);
        match item {
            JSXAttributeItem::Attribute(attribute) => visitor.visit_jsx_attribute(attribute),
            JSXAttributeItem::SpreadAttribute(attribute) => {
                visitor.visit_jsx_spread_attribute(attribute);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_attribute_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        attribute: &mut JSXAttribute<'a>,
    ) {
        if let Some(value) = &mut attribute.value {
            visitor.visit_jsx_attribute_value(value);
        }
    }

    pub fn walk_jsx_spread_attribute_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        attribute: &mut JSXSpreadAttribute<'a>,
    ) {
        visitor.visit_expression(&mut attribute.argument);
    }

    pub fn walk_jsx_attribute_value_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        value: &mut JSXAttributeValue<'a>,
    ) {
        match value {
            JSXAttributeValue::ExpressionContainer(expr) => {
                visitor.visit_jsx_expression_container(expr);
            }
            JSXAttributeValue::Element(elem) => visitor.visit_jsx_element(elem),
            JSXAttributeValue::Fragment(elem) => visitor.visit_jsx_fragment(elem),
            JSXAttributeValue::StringLiteral(_) => {}
        }
    }

    pub fn walk_jsx_expression_container_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut JSXExpressionContainer<'a>,
    ) {
        let kind = AstType::JSXExpressionContainer;
        visitor.enter_node(kind);
        visitor.visit_jsx_expression(&mut expr.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut JSXExpression<'a>,
    ) {
        match expr {
            JSXExpression::Expression(expr) => visitor.visit_expression(expr),
            JSXExpression::EmptyExpression(_) => {}
        }
    }

    pub fn walk_jsx_fragment_mut<'a, V: VisitMut<'a>>(visitor: &mut V, elem: &mut JSXFragment<'a>) {
        let kind = AstType::JSXFragment;
        visitor.enter_node(kind);
        for child in elem.children.iter_mut() {
            visitor.visit_jsx_child(child);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_child_mut<'a, V: VisitMut<'a>>(visitor: &mut V, child: &mut JSXChild<'a>) {
        match child {
            JSXChild::Element(elem) => visitor.visit_jsx_element(elem),
            JSXChild::Fragment(elem) => visitor.visit_jsx_fragment(elem),
            JSXChild::ExpressionContainer(expr) => visitor.visit_jsx_expression_container(expr),
            JSXChild::Spread(expr) => visitor.visit_jsx_spread_child(expr),
            JSXChild::Text(expr) => visitor.visit_jsx_text(expr),
        }
    }

    pub fn walk_jsx_spread_child_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        child: &mut JSXSpreadChild<'a>,
    ) {
        visitor.visit_expression(&mut child.expression);
    }

    pub fn walk_jsx_text_mut<'a, V: VisitMut<'a>>(visitor: &mut V, _child: &JSXText<'a>) {
        let kind = AstType::JSXText;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    /* ----------  Pattern ---------- */

    pub fn walk_binding_pattern_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        pat: &mut BindingPattern<'a>,
    ) {
        match &mut pat.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                visitor.visit_binding_identifier(ident);
            }
            BindingPatternKind::ObjectPattern(pat) => visitor.visit_object_pattern(pat),
            BindingPatternKind::ArrayPattern(pat) => visitor.visit_array_pattern(pat),
            BindingPatternKind::AssignmentPattern(pat) => visitor.visit_assignment_pattern(pat),
        }
        if let Some(type_annotation) = &mut pat.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
    }

    pub fn walk_binding_identifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _ident: &mut BindingIdentifier<'a>,
    ) {
        let kind = AstType::BindingIdentifier;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_object_pattern_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        pat: &mut ObjectPattern<'a>,
    ) {
        let kind = AstType::ObjectPattern;
        visitor.enter_node(kind);
        for prop in pat.properties.iter_mut() {
            visitor.visit_binding_property(prop);
        }
        if let Some(rest) = &mut pat.rest {
            visitor.visit_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_binding_property_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        prop: &mut BindingProperty<'a>,
    ) {
        visitor.visit_property_key(&mut prop.key);
        visitor.visit_binding_pattern(&mut prop.value);
    }

    pub fn walk_array_pattern_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        pat: &mut ArrayPattern<'a>,
    ) {
        let kind = AstType::ArrayPattern;
        visitor.enter_node(kind);
        for pat in pat.elements.iter_mut().flatten() {
            visitor.visit_binding_pattern(pat);
        }
        if let Some(rest) = &mut pat.rest {
            visitor.visit_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_rest_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        pat: &mut BindingRestElement<'a>,
    ) {
        let kind = AstType::BindingRestElement;
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&mut pat.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_assignment_pattern_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        pat: &mut AssignmentPattern<'a>,
    ) {
        let kind = AstType::AssignmentPattern;
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&mut pat.left);
        visitor.visit_expression(&mut pat.right);
        visitor.leave_node(kind);
    }

    /* ----------  Identifier ---------- */

    pub fn walk_identifier_reference_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _ident: &mut IdentifierReference<'a>,
    ) {
        let kind = AstType::IdentifierReference;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_private_identifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _ident: &mut PrivateIdentifier<'a>,
    ) {
        let kind = AstType::PrivateIdentifier;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_label_identifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _ident: &mut LabelIdentifier<'a>,
    ) {
        let kind = AstType::LabelIdentifier;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_identifier_name_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _ident: &mut IdentifierName<'a>,
    ) {
        let kind = AstType::IdentifierName;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    /* ----------  Literal ---------- */

    pub fn walk_number_literal_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _lit: &mut NumericLiteral<'a>,
    ) {
        let kind = AstType::NumericLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_boolean_literal_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _lit: &mut BooleanLiteral,
    ) {
        let kind = AstType::BooleanLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_null_literal_mut<'a, V: VisitMut<'a>>(visitor: &mut V, _lit: &mut NullLiteral) {
        let kind = AstType::NullLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_bigint_literal_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _lit: &mut BigIntLiteral<'a>,
    ) {
        let kind = AstType::BigintLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_string_literal_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _lit: &mut StringLiteral<'a>,
    ) {
        let kind = AstType::StringLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_template_literal_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        lit: &mut TemplateLiteral<'a>,
    ) {
        let kind = AstType::TemplateLiteral;
        visitor.enter_node(kind);
        for elem in lit.quasis.iter_mut() {
            visitor.visit_template_element(elem);
        }
        for expr in lit.expressions.iter_mut() {
            visitor.visit_expression(expr);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_reg_expr_literal_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        _lit: &mut RegExpLiteral<'a>,
    ) {
        let kind = AstType::RegExpLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_template_element_mut<'a, V: VisitMut<'a>>(
        _visitor: &mut V,
        _elem: &mut TemplateElement,
    ) {
        // noop!
    }

    /* ----------  Module ---------- */

    pub fn walk_module_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut ModuleDeclaration<'a>,
    ) {
        let kind = AstType::ModuleDeclaration;
        visitor.enter_node(kind);
        match decl {
            ModuleDeclaration::ImportDeclaration(decl) => {
                visitor.visit_import_declaration(decl);
            }
            ModuleDeclaration::ExportAllDeclaration(decl) => {
                visitor.visit_export_all_declaration(decl);
            }
            ModuleDeclaration::ExportDefaultDeclaration(decl) => {
                visitor.visit_export_default_declaration(decl);
            }
            ModuleDeclaration::ExportNamedDeclaration(decl) => {
                visitor.visit_export_named_declaration(decl);
            }
            ModuleDeclaration::TSExportAssignment(decl) => {
                visitor.visit_expression(&mut decl.expression);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(_) => {}
        }
        visitor.leave_node(kind);
    }

    pub fn walk_import_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut ImportDeclaration<'a>,
    ) {
        let kind = AstType::ImportDeclaration;
        visitor.enter_node(kind);
        if let Some(specifiers) = &mut decl.specifiers {
            for specifier in specifiers.iter_mut() {
                visitor.visit_import_declaration_specifier(specifier);
            }
        }
        visitor.visit_string_literal(&mut decl.source);
        if let Some(with_clause) = &mut decl.with_clause {
            visitor.visit_with_clause(with_clause);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_with_clause_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        with_clause: &mut WithClause<'a>,
    ) {
        for attribute in with_clause.with_entries.iter_mut() {
            visitor.visit_import_attribute(attribute);
        }
    }

    pub fn walk_import_attribute_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        attribute: &mut ImportAttribute<'a>,
    ) {
        visitor.visit_import_attribute_key(&mut attribute.key);
        visitor.visit_string_literal(&mut attribute.value);
    }

    pub fn walk_import_attribute_key_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        key: &mut ImportAttributeKey<'a>,
    ) {
        match key {
            ImportAttributeKey::Identifier(ident) => visitor.visit_identifier_name(ident),
            ImportAttributeKey::StringLiteral(ident) => visitor.visit_string_literal(ident),
        }
    }

    pub fn walk_import_declaration_specifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        specifier: &mut ImportDeclarationSpecifier<'a>,
    ) {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                visitor.visit_import_specifier(specifier);
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                visitor.visit_import_default_specifier(specifier);
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                visitor.visit_import_name_specifier(specifier);
            }
        }
    }

    pub fn walk_import_specifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        specifier: &mut ImportSpecifier<'a>,
    ) {
        let kind = AstType::ImportSpecifier;
        visitor.enter_node(kind);
        // TODO: imported
        visitor.visit_binding_identifier(&mut specifier.local);
        visitor.leave_node(kind);
    }

    pub fn walk_import_default_specifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        specifier: &mut ImportDefaultSpecifier<'a>,
    ) {
        let kind = AstType::ImportDefaultSpecifier;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut specifier.local);
        visitor.leave_node(kind);
    }

    pub fn walk_import_name_specifier_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        specifier: &mut ImportNamespaceSpecifier<'a>,
    ) {
        let kind = AstType::ImportNamespaceSpecifier;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut specifier.local);
        visitor.leave_node(kind);
    }

    pub fn walk_export_all_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut ExportAllDeclaration<'a>,
    ) {
        let kind = AstType::ExportAllDeclaration;
        visitor.enter_node(kind);
        visitor.visit_string_literal(&mut decl.source);
        visitor.leave_node(kind);
    }

    pub fn walk_export_default_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut ExportDefaultDeclaration<'a>,
    ) {
        let kind = AstType::ExportDefaultDeclaration;
        visitor.enter_node(kind);
        match &mut decl.declaration {
            ExportDefaultDeclarationKind::Expression(expr) => visitor.visit_expression(expr),
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                visitor.visit_function(func, None);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => visitor.visit_class(class),
            _ => {}
        }
        visitor.leave_node(kind);
    }

    pub fn walk_export_named_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut ExportNamedDeclaration<'a>,
    ) {
        let kind = AstType::ExportNamedDeclaration;
        visitor.enter_node(kind);
        if let Some(decl) = &mut decl.declaration {
            visitor.visit_declaration(decl);
        }
        if let Some(source) = &mut decl.source {
            visitor.visit_string_literal(source);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_enum_member_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        member: &mut TSEnumMember<'a>,
    ) {
        let kind = AstType::TSEnumMember;
        visitor.enter_node(kind);
        if let Some(initializer) = &mut member.initializer {
            visitor.visit_expression(initializer);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_enum_mut<'a, V: VisitMut<'a>>(visitor: &mut V, decl: &mut TSEnumDeclaration<'a>) {
        let kind = AstType::TSEnumDeclaration;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut decl.id);
        visitor.enter_scope(ScopeFlags::empty());
        for member in decl.members.iter_mut() {
            visitor.visit_enum_member(member);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    pub fn walk_declaration_mut<'a, V: VisitMut<'a>>(visitor: &mut V, decl: &mut Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(decl) => visitor.visit_variable_declaration(decl),
            Declaration::FunctionDeclaration(func) => visitor.visit_function(func, None),
            Declaration::ClassDeclaration(class) => visitor.visit_class(class),
            Declaration::UsingDeclaration(decl) => visitor.visit_using_declaration(decl),
            Declaration::TSModuleDeclaration(module) => {
                visitor.visit_ts_module_declaration(module);
            }
            Declaration::TSTypeAliasDeclaration(decl) => {
                visitor.visit_ts_type_alias_declaration(decl);
            }
            Declaration::TSEnumDeclaration(decl) => visitor.visit_enum(decl),
            Declaration::TSImportEqualsDeclaration(decl) => {
                visitor.visit_ts_import_equals_declaration(decl);
            }
            Declaration::TSInterfaceDeclaration(decl) => {
                visitor.visit_ts_interface_declaration(decl);
            }
        }
    }

    /* ----------  TypeScript ---------- */

    pub fn walk_ts_import_equals_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut TSImportEqualsDeclaration<'a>,
    ) {
        let kind = AstType::TSImportEqualsDeclaration;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut decl.id);
        visitor.visit_ts_module_reference(&mut decl.module_reference);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_module_reference_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        reference: &mut TSModuleReference<'a>,
    ) {
        match reference {
            TSModuleReference::TypeName(name) => visitor.visit_ts_type_name(name),
            TSModuleReference::ExternalModuleReference(reference) => {
                visitor.visit_ts_external_module_reference(reference);
            }
        }
    }

    pub fn walk_ts_type_name_mut<'a, V: VisitMut<'a>>(visitor: &mut V, name: &mut TSTypeName<'a>) {
        let kind = AstType::TSTypeName;
        visitor.enter_node(kind);
        match name {
            TSTypeName::IdentifierReference(ident) => visitor.visit_identifier_reference(ident),
            TSTypeName::QualifiedName(name) => visitor.visit_ts_qualified_name(name),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_external_module_reference_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        reference: &mut TSExternalModuleReference<'a>,
    ) {
        let kind = AstType::TSExternalModuleReference;
        visitor.enter_node(kind);
        visitor.visit_string_literal(&mut reference.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_qualified_name_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        name: &mut TSQualifiedName<'a>,
    ) {
        let kind = AstType::TSQualifiedName;
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&mut name.left);
        visitor.visit_identifier_name(&mut name.right);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_module_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut TSModuleDeclaration<'a>,
    ) {
        let kind = AstType::TSModuleDeclaration;
        visitor.enter_node(kind);
        match &mut decl.id {
            TSModuleDeclarationName::Identifier(ident) => visitor.visit_identifier_name(ident),
            TSModuleDeclarationName::StringLiteral(lit) => visitor.visit_string_literal(lit),
        }
        match &mut decl.body {
            Some(TSModuleDeclarationBody::TSModuleDeclaration(decl)) => {
                visitor.visit_ts_module_declaration(decl);
            }
            Some(TSModuleDeclarationBody::TSModuleBlock(block)) => {
                visitor.visit_ts_module_block(block);
            }
            None => {}
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_module_block_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        block: &mut TSModuleBlock<'a>,
    ) {
        let kind = AstType::TSModuleBlock;
        visitor.enter_scope(ScopeFlags::TsModuleBlock);
        visitor.enter_node(kind);
        visitor.visit_statements(&mut block.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_ts_type_alias_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut TSTypeAliasDeclaration<'a>,
    ) {
        let kind = AstType::TSTypeAliasDeclaration;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut decl.id);
        if let Some(parameters) = &mut decl.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        visitor.visit_ts_type(&mut decl.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_interface_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        decl: &mut TSInterfaceDeclaration<'a>,
    ) {
        let kind = AstType::TSInterfaceDeclaration;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut decl.id);
        if let Some(parameters) = &mut decl.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        for signature in decl.body.body.iter_mut() {
            visitor.visit_ts_signature(signature);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_as_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut TSAsExpression<'a>,
    ) {
        let kind = AstType::TSAsExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.expression);
        visitor.visit_ts_type(&mut expr.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_satisfies_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut TSSatisfiesExpression<'a>,
    ) {
        let kind = AstType::TSSatisfiesExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.expression);
        visitor.visit_ts_type(&mut expr.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_non_null_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut TSNonNullExpression<'a>,
    ) {
        let kind = AstType::TSNonNullExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_assertion_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut TSTypeAssertion<'a>,
    ) {
        let kind = AstType::TSTypeAssertion;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut expr.expression);
        visitor.visit_ts_type(&mut expr.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_instantiation_expression_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        expr: &mut TSInstantiationExpression<'a>,
    ) {
        visitor.visit_expression(&mut expr.expression);
        visitor.visit_ts_type_parameter_instantiation(&mut expr.type_parameters);
    }

    pub fn walk_ts_type_annotation_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        annotation: &mut TSTypeAnnotation<'a>,
    ) {
        let kind = AstType::TSTypeAnnotation;
        visitor.enter_node(kind);
        visitor.visit_ts_type(&mut annotation.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_mut<'a, V: VisitMut<'a>>(visitor: &mut V, ty: &mut TSType<'a>) {
        match ty {
            TSType::TSAnyKeyword(ty) => visitor.visit_ts_any_keyword(ty),
            TSType::TSNullKeyword(ty) => visitor.visit_ts_null_keyword(ty),
            TSType::TSVoidKeyword(ty) => visitor.visit_ts_void_keyword(ty),
            TSType::TSIntersectionType(ty) => visitor.visit_ts_intersection_type(ty),
            TSType::TSTypeReference(ty) => visitor.visit_ts_type_reference(ty),
            TSType::TSUnionType(ty) => visitor.visit_ts_union_type(ty),
            TSType::TSLiteralType(ty) => visitor.visit_ts_literal_type(ty),
            TSType::TSArrayType(ty) => visitor.visit_ts_array_type(ty),
            TSType::TSConditionalType(ty) => visitor.visit_ts_conditional_type(ty),
            TSType::TSConstructorType(ty) => visitor.visit_ts_constructor_type(ty),
            TSType::TSFunctionType(ty) => visitor.visit_ts_function_type(ty),
            TSType::TSMappedType(ty) => visitor.visit_ts_mapped_type(ty),
            TSType::TSTupleType(ty) => visitor.visit_ts_tuple_type(ty),
            TSType::TSTypeOperatorType(ty) => visitor.visit_ts_type_operator_type(ty),
            TSType::TSTypePredicate(ty) => visitor.visit_ts_type_predicate(ty),
            TSType::TSTypeLiteral(ty) => visitor.visit_ts_type_literal(ty),
            TSType::TSIndexedAccessType(ty) => visitor.visit_ts_indexed_access_type(ty),
            TSType::TSTypeQuery(ty) => visitor.visit_ts_type_query(ty),
            _ => {}
        }
    }

    pub fn walk_ts_type_literal_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSTypeLiteral<'a>,
    ) {
        let kind = AstType::TSTypeLiteral;
        visitor.enter_node(kind);
        for signature in ty.members.iter_mut() {
            visitor.visit_ts_signature(signature);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_indexed_access_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSIndexedAccessType<'a>,
    ) {
        let kind = AstType::TSIndexedAccessType;
        visitor.enter_node(kind);
        visitor.visit_ts_type(&mut ty.object_type);
        visitor.visit_ts_type(&mut ty.index_type);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_predicate_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSTypePredicate<'a>,
    ) {
        if let Some(annotation) = &mut ty.type_annotation {
            visitor.visit_ts_type_annotation(annotation);
        }
    }

    pub fn walk_ts_type_operator_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSTypeOperator<'a>,
    ) {
        visitor.visit_ts_type(&mut ty.type_annotation);
    }

    pub fn walk_ts_tuple_type_mut<'a, V: VisitMut<'a>>(visitor: &mut V, ty: &mut TSTupleType<'a>) {
        for element in ty.element_types.iter_mut() {
            visitor.visit_ts_tuple_element(element);
        }
    }

    pub fn walk_ts_tuple_element_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSTupleElement<'a>,
    ) {
        match ty {
            TSTupleElement::TSType(ty) => visitor.visit_ts_type(ty),
            TSTupleElement::TSOptionalType(ty) => visitor.visit_ts_type(&mut ty.type_annotation),
            TSTupleElement::TSRestType(ty) => visitor.visit_ts_type(&mut ty.type_annotation),
            TSTupleElement::TSNamedTupleMember(ty) => visitor.visit_ts_type(&mut ty.element_type),
        };
    }

    pub fn walk_ts_mapped_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSMappedType<'a>,
    ) {
        visitor.visit_ts_type_parameter(&mut ty.type_parameter);
        if let Some(name) = &mut ty.name_type {
            visitor.visit_ts_type(name);
        }
        if let Some(type_annotation) = &mut ty.type_annotation {
            visitor.visit_ts_type(type_annotation);
        }
    }

    pub fn walk_ts_function_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSFunctionType<'a>,
    ) {
        visitor.visit_formal_parameters(&mut ty.params);
        if let Some(parameters) = &mut ty.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        visitor.visit_ts_type_annotation(&mut ty.return_type);
    }

    pub fn walk_ts_type_parameter_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSTypeParameter<'a>,
    ) {
        let kind = AstType::TSTypeParameter;
        visitor.enter_scope(ScopeFlags::empty());
        visitor.enter_node(kind);
        if let Some(constraint) = &mut ty.constraint {
            visitor.visit_ts_type(constraint);
        }

        if let Some(default) = &mut ty.default {
            visitor.visit_ts_type(default);
        }
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_ts_type_parameter_instantiation_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSTypeParameterInstantiation<'a>,
    ) {
        let kind = AstType::TSTypeParameterInstantiation;
        visitor.enter_node(kind);
        for ts_parameter in ty.params.iter_mut() {
            visitor.visit_ts_type(ts_parameter);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_parameter_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSTypeParameterDeclaration<'a>,
    ) {
        let kind = AstType::TSTypeParameterDeclaration;
        visitor.enter_node(kind);
        for ts_parameter in ty.params.iter_mut() {
            visitor.visit_ts_type_parameter(ts_parameter);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_constructor_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSConstructorType<'a>,
    ) {
        visitor.visit_formal_parameters(&mut ty.params);
        if let Some(parameters) = &mut ty.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        visitor.visit_ts_type_annotation(&mut ty.return_type);
    }

    pub fn walk_ts_conditional_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSConditionalType<'a>,
    ) {
        visitor.visit_ts_type(&mut ty.check_type);
        visitor.visit_ts_type(&mut ty.extends_type);
        visitor.visit_ts_type(&mut ty.true_type);
        visitor.visit_ts_type(&mut ty.false_type);
    }

    pub fn walk_ts_array_type_mut<'a, V: VisitMut<'a>>(visitor: &mut V, ty: &mut TSArrayType<'a>) {
        visitor.visit_ts_type(&mut ty.element_type);
    }

    pub fn walk_ts_null_keyword_mut<'a, V: VisitMut<'a>>(visitor: &mut V, _ty: &mut TSNullKeyword) {
        let kind = AstType::TSNullKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_any_keyword_mut<'a, V: VisitMut<'a>>(visitor: &mut V, _ty: &mut TSAnyKeyword) {
        let kind = AstType::TSAnyKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_void_keyword_mut<'a, V: VisitMut<'a>>(visitor: &mut V, _ty: &mut TSVoidKeyword) {
        let kind = AstType::TSVoidKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_intersection_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSIntersectionType<'a>,
    ) {
        let kind = AstType::TSIntersectionType;
        visitor.enter_node(kind);
        for ty in ty.types.iter_mut() {
            visitor.visit_ts_type(ty);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_reference_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSTypeReference<'a>,
    ) {
        let kind = AstType::TSTypeReference;
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&mut ty.type_name);
        if let Some(parameters) = &mut ty.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(parameters);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_union_type_mut<'a, V: VisitMut<'a>>(visitor: &mut V, ty: &mut TSUnionType<'a>) {
        let kind = AstType::TSUnionType;
        visitor.enter_node(kind);
        for ty in ty.types.iter_mut() {
            visitor.visit_ts_type(ty);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_literal_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSLiteralType<'a>,
    ) {
        let kind = AstType::TSLiteralType;
        visitor.enter_node(kind);
        match &mut ty.literal {
            TSLiteral::BigintLiteral(lit) => visitor.visit_bigint_literal(lit),
            TSLiteral::BooleanLiteral(lit) => visitor.visit_boolean_literal(lit),
            TSLiteral::NullLiteral(lit) => visitor.visit_null_literal(lit),
            TSLiteral::NumericLiteral(lit) => visitor.visit_number_literal(lit),
            TSLiteral::RegExpLiteral(lit) => visitor.visit_reg_expr_literal(lit),
            TSLiteral::StringLiteral(lit) => visitor.visit_string_literal(lit),
            TSLiteral::TemplateLiteral(lit) => visitor.visit_template_literal(lit),
            TSLiteral::UnaryExpression(expr) => visitor.visit_unary_expression(expr),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_signature_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        signature: &mut TSSignature<'a>,
    ) {
        match signature {
            TSSignature::TSPropertySignature(sig) => visitor.visit_ts_property_signature(sig),
            TSSignature::TSCallSignatureDeclaration(sig) => {
                visitor.visit_ts_call_signature_declaration(sig);
            }
            TSSignature::TSIndexSignature(sig) => visitor.visit_ts_index_signature(sig),
            TSSignature::TSMethodSignature(sig) => visitor.visit_ts_method_signature(sig),
            TSSignature::TSConstructSignatureDeclaration(sig) => {
                visitor.visit_ts_construct_signature_declaration(sig);
            }
        }
    }

    pub fn walk_ts_construct_signature_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        signature: &mut TSConstructSignatureDeclaration<'a>,
    ) {
        visitor.visit_formal_parameters(&mut signature.params);
        if let Some(parameters) = &mut signature.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &mut signature.return_type {
            visitor.visit_ts_type_annotation(annotation);
        }
    }

    pub fn walk_ts_method_signature_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        signature: &mut TSMethodSignature<'a>,
    ) {
        let kind = AstType::TSMethodSignature;
        visitor.enter_node(kind);
        visitor.visit_formal_parameters(&mut signature.params);
        if let Some(parameters) = &mut signature.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &mut signature.return_type {
            visitor.visit_ts_type_annotation(annotation);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_index_signature_name_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        name: &mut TSIndexSignatureName<'a>,
    ) {
        visitor.visit_ts_type_annotation(&mut name.type_annotation);
    }

    pub fn walk_ts_index_signature_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        signature: &mut TSIndexSignature<'a>,
    ) {
        for name in signature.parameters.iter_mut() {
            visitor.visit_ts_index_signature_name(name);
        }

        visitor.visit_ts_type_annotation(&mut signature.type_annotation);
    }

    pub fn walk_ts_property_signature_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        signature: &mut TSPropertySignature<'a>,
    ) {
        let kind = AstType::TSPropertySignature;
        visitor.enter_node(kind);
        visitor.visit_property_key(&mut signature.key);
        if let Some(annotation) = &mut signature.type_annotation {
            visitor.visit_ts_type_annotation(annotation);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_call_signature_declaration_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        signature: &mut TSCallSignatureDeclaration<'a>,
    ) {
        visitor.visit_formal_parameters(&mut signature.params);
        if let Some(parameters) = &mut signature.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(annotation) = &mut signature.return_type {
            visitor.visit_ts_type_annotation(annotation);
        }
    }

    pub fn walk_ts_type_query_mut<'a, V: VisitMut<'a>>(visitor: &mut V, ty: &mut TSTypeQuery<'a>) {
        let kind = AstType::TSTypeQuery;
        visitor.enter_node(kind);
        match &mut ty.expr_name {
            TSTypeQueryExprName::TSTypeName(name) => visitor.visit_ts_type_name(name),
            TSTypeQueryExprName::TSImportType(import) => visitor.visit_ts_import_type(import),
        }
        if let Some(type_parameters) = &mut ty.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_import_type_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        ty: &mut TSImportType<'a>,
    ) {
        let kind = AstType::TSImportType;
        visitor.enter_node(kind);
        visitor.visit_ts_type(&mut ty.argument);
        if let Some(name) = &mut ty.qualifier {
            visitor.visit_ts_type_name(name);
        }
        if let Some(attrs) = &mut ty.attributes {
            visitor.visit_ts_import_attributes(attrs);
        }
        if let Some(type_parameter) = &mut ty.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameter);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_import_attributes_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        attributes: &mut TSImportAttributes<'a>,
    ) {
        for element in attributes.elements.iter_mut() {
            visitor.visit_ts_import_attribute(element);
        }
    }

    pub fn walk_ts_import_attribute_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        attribute: &mut TSImportAttribute<'a>,
    ) {
        visitor.visit_ts_import_attribute_name(&mut attribute.name);
        visitor.visit_expression(&mut attribute.value);
    }

    pub fn walk_ts_import_attribute_name_mut<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        name: &mut TSImportAttributeName<'a>,
    ) {
        match name {
            TSImportAttributeName::Identifier(ident) => visitor.visit_identifier_name(ident),
            TSImportAttributeName::StringLiteral(ident) => visitor.visit_string_literal(ident),
        }
    }
}
