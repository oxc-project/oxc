//! Visitor Pattern
//!
//! See:
//! * [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)
//! * [rustc visitor](https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs)

use oxc_allocator::Vec;
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use crate::{ast::*, ast_kind::AstKind};

use walk::*;

/// Syntax tree traversal
pub trait Visit<'a>: Sized {
    fn enter_node(&mut self, _kind: AstKind<'a>) {}
    fn leave_node(&mut self, _kind: AstKind<'a>) {}

    fn enter_scope(&mut self, _flags: ScopeFlags) {}
    fn leave_scope(&mut self) {}

    fn alloc<T>(&self, t: &T) -> &'a T {
        // SAFETY:
        // This should be safe as long as `src` is an reference from the allocator.
        // But honestly, I'm not really sure if this is safe.
        unsafe { std::mem::transmute(t) }
    }

    fn visit_program(&mut self, program: &Program<'a>) {
        walk_program(self, program);
    }

    /* ----------  Statement ---------- */

    fn visit_statements(&mut self, stmts: &Vec<'a, Statement<'a>>) {
        walk_statements(self, stmts);
    }

    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        walk_statement(self, stmt);
    }

    fn visit_block_statement(&mut self, stmt: &BlockStatement<'a>) {
        walk_block_statement(self, stmt);
    }

    fn visit_break_statement(&mut self, stmt: &BreakStatement<'a>) {
        walk_break_statement(self, stmt);
    }

    fn visit_continue_statement(&mut self, stmt: &ContinueStatement<'a>) {
        walk_continue_statement(self, stmt);
    }

    fn visit_debugger_statement(&mut self, stmt: &DebuggerStatement) {
        walk_debugger_statement(self, stmt);
    }

    fn visit_do_while_statement(&mut self, stmt: &DoWhileStatement<'a>) {
        walk_do_while_statement(self, stmt);
    }

    fn visit_empty_statement(&mut self, stmt: &EmptyStatement) {
        walk_empty_statement(self, stmt);
    }

    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        walk_expression_statement(self, stmt);
    }

    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        walk_for_statement(self, stmt);
    }

    fn visit_for_statement_init(&mut self, init: &ForStatementInit<'a>) {
        walk_for_statement_init(self, init);
    }

    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        walk_for_in_statement(self, stmt);
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        walk_for_of_statement(self, stmt);
    }

    fn visit_for_statement_left(&mut self, left: &ForStatementLeft<'a>) {
        walk_for_statement_left(self, left);
    }

    fn visit_if_statement(&mut self, stmt: &IfStatement<'a>) {
        walk_if_statement(self, stmt);
    }

    fn visit_labeled_statement(&mut self, stmt: &LabeledStatement<'a>) {
        walk_labeled_statement(self, stmt);
    }

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        walk_return_statement(self, stmt);
    }

    fn visit_switch_statement(&mut self, stmt: &SwitchStatement<'a>) {
        walk_switch_statement(self, stmt);
    }

    fn visit_switch_case(&mut self, case: &SwitchCase<'a>) {
        walk_switch_case(self, case);
    }

    fn visit_throw_statement(&mut self, stmt: &ThrowStatement<'a>) {
        walk_throw_statement(self, stmt);
    }

    fn visit_try_statement(&mut self, stmt: &TryStatement<'a>) {
        walk_try_statement(self, stmt);
    }

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        walk_catch_clause(self, clause);
    }

    fn visit_finally_clause(&mut self, clause: &BlockStatement<'a>) {
        walk_finally_clause(self, clause);
    }

    fn visit_while_statement(&mut self, stmt: &WhileStatement<'a>) {
        walk_while_statement(self, stmt);
    }

    fn visit_with_statement(&mut self, stmt: &WithStatement<'a>) {
        walk_with_statement(self, stmt);
    }

    fn visit_directive(&mut self, directive: &Directive<'a>) {
        walk_directive(self, directive);
    }

    /* ----------  Declaration ---------- */

    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        walk_variable_declaration(self, decl);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        walk_variable_declarator(self, declarator);
    }

    /* ----------  Function ---------- */

    fn visit_function(&mut self, func: &Function<'a>, flags: Option<ScopeFlags>) {
        walk_function(self, func, flags);
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        walk_function_body(self, body);
    }

    fn visit_formal_parameters(&mut self, params: &FormalParameters<'a>) {
        walk_formal_parameters(self, params);
    }

    fn visit_formal_parameter(&mut self, param: &FormalParameter<'a>) {
        walk_formal_parameter(self, param);
    }

    /* ----------  Class ---------- */

    fn visit_decorator(&mut self, decorator: &Decorator<'a>) {
        walk_decorator(self, decorator);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        walk_class(self, class);
    }

    fn visit_class_heritage(&mut self, expr: &Expression<'a>) {
        walk_class_heritage(self, expr);
    }

    fn visit_class_body(&mut self, body: &ClassBody<'a>) {
        walk_class_body(self, body);
    }

    fn visit_class_element(&mut self, elem: &ClassElement<'a>) {
        walk_class_element(self, elem);
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        walk_static_block(self, block);
    }

    fn visit_method_definition(&mut self, def: &MethodDefinition<'a>) {
        walk_method_definition(self, def);
    }

    fn visit_property_definition(&mut self, def: &PropertyDefinition<'a>) {
        walk_property_definition(self, def);
    }

    fn visit_using_declaration(&mut self, decl: &UsingDeclaration<'a>) {
        walk_using_declaration(self, decl);
    }

    /* ----------  Expression ---------- */

    fn visit_expression(&mut self, expr: &Expression<'a>) {
        walk_expression(self, expr);
    }

    fn visit_meta_property(&mut self, meta: &MetaProperty<'a>) {
        walk_meta_property(self, meta);
    }

    fn visit_array_expression(&mut self, expr: &ArrayExpression<'a>) {
        walk_array_expression(self, expr);
    }

    fn visit_array_expression_element(&mut self, arg: &ArrayExpressionElement<'a>) {
        walk_array_expression_element(self, arg);
    }

    fn visit_argument(&mut self, arg: &Argument<'a>) {
        walk_argument(self, arg);
    }

    fn visit_spread_element(&mut self, elem: &SpreadElement<'a>) {
        walk_spread_element(self, elem);
    }

    fn visit_expression_array_element(&mut self, expr: &Expression<'a>) {
        walk_expression_array_element(self, expr);
    }

    fn visit_elision(&mut self, span: Span) {
        walk_elision(self, span);
    }

    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        walk_assignment_expression(self, expr);
    }

    fn visit_arrow_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        walk_arrow_expression(self, expr);
    }

    fn visit_await_expression(&mut self, expr: &AwaitExpression<'a>) {
        walk_await_expression(self, expr);
    }

    fn visit_binary_expression(&mut self, expr: &BinaryExpression<'a>) {
        walk_binary_expression(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        walk_call_expression(self, expr);
    }

    fn visit_chain_expression(&mut self, expr: &ChainExpression<'a>) {
        walk_chain_expression(self, expr);
    }

    fn visit_chain_element(&mut self, elem: &ChainElement<'a>) {
        walk_chain_element(self, elem);
    }

    fn visit_conditional_expression(&mut self, expr: &ConditionalExpression<'a>) {
        walk_conditional_expression(self, expr);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        walk_import_expression(self, expr);
    }

    fn visit_logical_expression(&mut self, expr: &LogicalExpression<'a>) {
        walk_logical_expression(self, expr);
    }

    fn visit_member_expression(&mut self, expr: &MemberExpression<'a>) {
        walk_member_expression(self, expr);
    }

    fn visit_computed_member_expression(&mut self, expr: &ComputedMemberExpression<'a>) {
        walk_computed_member_expression(self, expr);
    }

    fn visit_static_member_expression(&mut self, expr: &StaticMemberExpression<'a>) {
        walk_static_member_expression(self, expr);
    }

    fn visit_private_field_expression(&mut self, expr: &PrivateFieldExpression<'a>) {
        walk_private_field_expression(self, expr);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        walk_new_expression(self, expr);
    }

    fn visit_object_expression(&mut self, expr: &ObjectExpression<'a>) {
        walk_object_expression(self, expr);
    }

    fn visit_object_property_kind(&mut self, prop: &ObjectPropertyKind<'a>) {
        walk_object_property_kind(self, prop);
    }

    fn visit_object_property(&mut self, prop: &ObjectProperty<'a>) {
        walk_object_property(self, prop);
    }

    fn visit_property_key(&mut self, key: &PropertyKey<'a>) {
        walk_property_key(self, key);
    }

    fn visit_parenthesized_expression(&mut self, expr: &ParenthesizedExpression<'a>) {
        walk_parenthesized_expression(self, expr);
    }

    fn visit_private_in_expression(&mut self, expr: &PrivateInExpression<'a>) {
        walk_private_in_expression(self, expr);
    }

    fn visit_sequence_expression(&mut self, expr: &SequenceExpression<'a>) {
        walk_sequence_expression(self, expr);
    }

    fn visit_tagged_template_expression(&mut self, expr: &TaggedTemplateExpression<'a>) {
        walk_tagged_template_expression(self, expr);
    }

    fn visit_this_expression(&mut self, expr: &ThisExpression) {
        walk_this_expression(self, expr);
    }

    fn visit_unary_expression(&mut self, expr: &UnaryExpression<'a>) {
        walk_unary_expression(self, expr);
    }

    fn visit_update_expression(&mut self, expr: &UpdateExpression<'a>) {
        walk_update_expression(self, expr);
    }

    fn visit_yield_expression(&mut self, expr: &YieldExpression<'a>) {
        walk_yield_expression(self, expr);
    }

    fn visit_super(&mut self, expr: &Super) {
        walk_super(self, expr);
    }

    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        walk_assignment_target(self, target);
    }

    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        walk_simple_assignment_target(self, target);
    }

    fn visit_assignment_target_pattern(&mut self, pat: &AssignmentTargetPattern<'a>) {
        walk_assignment_target_pattern(self, pat);
    }

    fn visit_array_assignment_target(&mut self, target: &ArrayAssignmentTarget<'a>) {
        walk_array_assignment_target(self, target);
    }

    fn visit_assignment_target_maybe_default(&mut self, target: &AssignmentTargetMaybeDefault<'a>) {
        walk_assignment_target_maybe_default(self, target);
    }

    fn visit_assignment_target_with_default(&mut self, target: &AssignmentTargetWithDefault<'a>) {
        walk_assignment_target_with_default(self, target);
    }

    fn visit_object_assignment_target(&mut self, target: &ObjectAssignmentTarget<'a>) {
        walk_object_assignment_target(self, target);
    }

    fn visit_assignment_target_property(&mut self, property: &AssignmentTargetProperty<'a>) {
        walk_assignment_target_property(self, property);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        ident: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        walk_assignment_target_property_identifier(self, ident);
    }

    fn visit_assignment_target_property_property(
        &mut self,
        property: &AssignmentTargetPropertyProperty<'a>,
    ) {
        walk_assignment_target_property_property(self, property);
    }

    fn visit_assignment_target_rest(&mut self, rest: &AssignmentTargetRest<'a>) {
        walk_assignment_target_rest(self, rest);
    }

    /* ----------  Expression ---------- */

    fn visit_jsx_element(&mut self, elem: &JSXElement<'a>) {
        walk_jsx_element(self, elem);
    }

    fn visit_jsx_opening_element(&mut self, elem: &JSXOpeningElement<'a>) {
        walk_jsx_opening_element(self, elem);
    }

    fn visit_jsx_closing_element(&mut self, elem: &JSXClosingElement<'a>) {
        walk_jsx_closing_element(self, elem);
    }

    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        walk_jsx_element_name(self, name);
    }

    fn visit_jsx_identifier(&mut self, ident: &JSXIdentifier<'a>) {
        walk_jsx_identifier(self, ident);
    }

    fn visit_jsx_member_expression(&mut self, expr: &JSXMemberExpression<'a>) {
        walk_jsx_member_expression(self, expr);
    }

    fn visit_jsx_member_expression_object(&mut self, expr: &JSXMemberExpressionObject<'a>) {
        walk_jsx_member_expression_object(self, expr);
    }

    fn visit_jsx_namespaced_name(&mut self, name: &JSXNamespacedName<'a>) {
        walk_jsx_namespaced_name(self, name);
    }

    fn visit_jsx_attribute_item(&mut self, item: &JSXAttributeItem<'a>) {
        walk_jsx_attribute_item(self, item);
    }

    fn visit_jsx_attribute(&mut self, attribute: &JSXAttribute<'a>) {
        walk_jsx_attribute(self, attribute);
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &JSXSpreadAttribute<'a>) {
        walk_jsx_spread_attribute(self, attribute);
    }

    fn visit_jsx_attribute_value(&mut self, value: &JSXAttributeValue<'a>) {
        walk_jsx_attribute_value(self, value);
    }

    fn visit_jsx_expression_container(&mut self, expr: &JSXExpressionContainer<'a>) {
        walk_jsx_expression_container(self, expr);
    }

    fn visit_jsx_expression(&mut self, expr: &JSXExpression<'a>) {
        walk_jsx_expression(self, expr);
    }

    fn visit_jsx_fragment(&mut self, elem: &JSXFragment<'a>) {
        walk_jsx_fragment(self, elem);
    }

    fn visit_jsx_child(&mut self, child: &JSXChild<'a>) {
        walk_jsx_child(self, child);
    }

    fn visit_jsx_spread_child(&mut self, child: &JSXSpreadChild<'a>) {
        walk_jsx_spread_child(self, child);
    }

    fn visit_jsx_text(&mut self, child: &JSXText<'a>) {
        walk_jsx_text(self, child);
    }

    /* ----------  Pattern ---------- */

    fn visit_binding_pattern(&mut self, pat: &BindingPattern<'a>) {
        walk_binding_pattern(self, pat);
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        walk_binding_identifier(self, ident);
    }

    fn visit_object_pattern(&mut self, pat: &ObjectPattern<'a>) {
        walk_object_pattern(self, pat);
    }

    fn visit_binding_property(&mut self, prop: &BindingProperty<'a>) {
        walk_binding_property(self, prop);
    }

    fn visit_array_pattern(&mut self, pat: &ArrayPattern<'a>) {
        walk_array_pattern(self, pat);
    }

    fn visit_rest_element(&mut self, pat: &BindingRestElement<'a>) {
        walk_rest_element(self, pat);
    }

    fn visit_assignment_pattern(&mut self, pat: &AssignmentPattern<'a>) {
        walk_assignment_pattern(self, pat);
    }

    /* ----------  Identifier ---------- */

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        walk_identifier_reference(self, ident);
    }

    fn visit_private_identifier(&mut self, ident: &PrivateIdentifier<'a>) {
        walk_private_identifier(self, ident);
    }

    fn visit_label_identifier(&mut self, ident: &LabelIdentifier<'a>) {
        walk_label_identifier(self, ident);
    }

    fn visit_identifier_name(&mut self, ident: &IdentifierName<'a>) {
        walk_identifier_name(self, ident);
    }

    /* ----------  Literal ---------- */

    fn visit_number_literal(&mut self, lit: &NumericLiteral<'a>) {
        walk_number_literal(self, lit);
    }

    fn visit_boolean_literal(&mut self, lit: &BooleanLiteral) {
        walk_boolean_literal(self, lit);
    }

    fn visit_null_literal(&mut self, lit: &NullLiteral) {
        walk_null_literal(self, lit);
    }

    fn visit_bigint_literal(&mut self, lit: &BigIntLiteral<'a>) {
        walk_bigint_literal(self, lit);
    }

    fn visit_string_literal(&mut self, lit: &StringLiteral<'a>) {
        walk_string_literal(self, lit);
    }

    fn visit_template_literal(&mut self, lit: &TemplateLiteral<'a>) {
        walk_template_literal(self, lit);
    }

    fn visit_reg_expr_literal(&mut self, lit: &RegExpLiteral<'a>) {
        walk_reg_expr_literal(self, lit);
    }

    fn visit_template_element(&mut self, elem: &TemplateElement) {
        walk_template_element(self, elem);
    }

    /* ----------  Module ---------- */

    fn visit_module_declaration(&mut self, decl: &ModuleDeclaration<'a>) {
        walk_module_declaration(self, decl);
    }

    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        walk_import_declaration(self, decl);
    }

    fn visit_with_clause(&mut self, with_clause: &WithClause<'a>) {
        walk_with_clause(self, with_clause);
    }

    fn visit_import_attribute(&mut self, attribute: &ImportAttribute<'a>) {
        walk_import_attribute(self, attribute);
    }

    fn visit_import_attribute_key(&mut self, key: &ImportAttributeKey<'a>) {
        walk_import_attribute_key(self, key);
    }

    fn visit_import_declaration_specifier(&mut self, specifier: &ImportDeclarationSpecifier<'a>) {
        walk_import_declaration_specifier(self, specifier);
    }

    fn visit_import_specifier(&mut self, specifier: &ImportSpecifier<'a>) {
        walk_import_specifier(self, specifier);
    }

    fn visit_import_default_specifier(&mut self, specifier: &ImportDefaultSpecifier<'a>) {
        walk_import_default_specifier(self, specifier);
    }

    fn visit_import_name_specifier(&mut self, specifier: &ImportNamespaceSpecifier<'a>) {
        walk_import_name_specifier(self, specifier);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        walk_export_all_declaration(self, decl);
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        walk_export_default_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        walk_export_named_declaration(self, decl);
    }

    fn visit_enum_member(&mut self, member: &TSEnumMember<'a>) {
        walk_enum_member(self, member);
    }

    fn visit_enum(&mut self, decl: &TSEnumDeclaration<'a>) {
        walk_enum(self, decl);
    }

    fn visit_declaration(&mut self, decl: &Declaration<'a>) {
        walk_declaration(self, decl);
    }

    fn visit_ts_import_equals_declaration(&mut self, decl: &TSImportEqualsDeclaration<'a>) {
        walk_ts_import_equals_declaration(self, decl);
    }

    fn visit_ts_module_reference(&mut self, reference: &TSModuleReference<'a>) {
        walk_ts_module_reference(self, reference);
    }

    fn visit_ts_type_name(&mut self, name: &TSTypeName<'a>) {
        walk_ts_type_name(self, name);
    }

    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        walk_ts_external_module_reference(self, reference);
    }

    fn visit_ts_qualified_name(&mut self, name: &TSQualifiedName<'a>) {
        walk_ts_qualified_name(self, name);
    }

    fn visit_ts_module_declaration(&mut self, decl: &TSModuleDeclaration<'a>) {
        walk_ts_module_declaration(self, decl);
    }

    fn visit_ts_module_block(&mut self, block: &TSModuleBlock<'a>) {
        walk_ts_module_block(self, block);
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        walk_ts_type_alias_declaration(self, decl);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
        walk_ts_interface_declaration(self, decl);
    }

    fn visit_ts_as_expression(&mut self, expr: &TSAsExpression<'a>) {
        walk_ts_as_expression(self, expr);
    }

    fn visit_ts_satisfies_expression(&mut self, expr: &TSSatisfiesExpression<'a>) {
        walk_ts_satisfies_expression(self, expr);
    }

    fn visit_ts_non_null_expression(&mut self, expr: &TSNonNullExpression<'a>) {
        walk_ts_non_null_expression(self, expr);
    }

    fn visit_ts_type_assertion(&mut self, expr: &TSTypeAssertion<'a>) {
        walk_ts_type_assertion(self, expr);
    }

    fn visit_ts_instantiation_expression(&mut self, expr: &TSInstantiationExpression<'a>) {
        walk_ts_instantiation_expression(self, expr);
    }

    fn visit_ts_type_annotation(&mut self, annotation: &TSTypeAnnotation<'a>) {
        walk_ts_type_annotation(self, annotation);
    }

    fn visit_ts_type(&mut self, ty: &TSType<'a>) {
        walk_ts_type(self, ty);
    }

    fn visit_ts_type_literal(&mut self, ty: &TSTypeLiteral<'a>) {
        walk_ts_type_literal(self, ty);
    }

    fn visit_ts_indexed_access_type(&mut self, ty: &TSIndexedAccessType<'a>) {
        walk_ts_indexed_access_type(self, ty);
    }

    fn visit_ts_type_predicate(&mut self, ty: &TSTypePredicate<'a>) {
        walk_ts_type_predicate(self, ty);
    }

    fn visit_ts_type_operator_type(&mut self, ty: &TSTypeOperator<'a>) {
        walk_ts_type_operator_type(self, ty);
    }

    fn visit_ts_tuple_type(&mut self, ty: &TSTupleType<'a>) {
        walk_ts_tuple_type(self, ty);
    }

    fn visit_ts_tuple_element(&mut self, ty: &TSTupleElement<'a>) {
        walk_ts_tuple_element(self, ty);
    }

    fn visit_ts_mapped_type(&mut self, ty: &TSMappedType<'a>) {
        walk_ts_mapped_type(self, ty);
    }

    fn visit_ts_function_type(&mut self, ty: &TSFunctionType<'a>) {
        walk_ts_function_type(self, ty);
    }

    fn visit_ts_type_parameter(&mut self, ty: &TSTypeParameter<'a>) {
        walk_ts_type_parameter(self, ty);
    }

    fn visit_ts_type_parameter_instantiation(&mut self, ty: &TSTypeParameterInstantiation<'a>) {
        walk_ts_type_parameter_instantiation(self, ty);
    }

    fn visit_ts_type_parameter_declaration(&mut self, ty: &TSTypeParameterDeclaration<'a>) {
        walk_ts_type_parameter_declaration(self, ty);
    }

    fn visit_ts_constructor_type(&mut self, ty: &TSConstructorType<'a>) {
        walk_ts_constructor_type(self, ty);
    }

    fn visit_ts_conditional_type(&mut self, ty: &TSConditionalType<'a>) {
        walk_ts_conditional_type(self, ty);
    }

    fn visit_ts_array_type(&mut self, ty: &TSArrayType<'a>) {
        walk_ts_array_type(self, ty);
    }

    fn visit_ts_null_keyword(&mut self, ty: &TSNullKeyword) {
        walk_ts_null_keyword(self, ty);
    }

    fn visit_ts_any_keyword(&mut self, ty: &TSAnyKeyword) {
        walk_ts_any_keyword(self, ty);
    }

    fn visit_ts_void_keyword(&mut self, ty: &TSVoidKeyword) {
        walk_ts_void_keyword(self, ty);
    }

    fn visit_ts_intersection_type(&mut self, ty: &TSIntersectionType<'a>) {
        walk_ts_intersection_type(self, ty);
    }

    fn visit_ts_type_reference(&mut self, ty: &TSTypeReference<'a>) {
        walk_ts_type_reference(self, ty);
    }

    fn visit_ts_union_type(&mut self, ty: &TSUnionType<'a>) {
        walk_ts_union_type(self, ty);
    }

    fn visit_ts_literal_type(&mut self, ty: &TSLiteralType<'a>) {
        walk_ts_literal_type(self, ty);
    }

    fn visit_ts_signature(&mut self, signature: &TSSignature<'a>) {
        walk_ts_signature(self, signature);
    }

    fn visit_ts_construct_signature_declaration(
        &mut self,
        signature: &TSConstructSignatureDeclaration<'a>,
    ) {
        walk_ts_construct_signature_declaration(self, signature);
    }

    fn visit_ts_method_signature(&mut self, signature: &TSMethodSignature<'a>) {
        walk_ts_method_signature(self, signature);
    }

    fn visit_ts_index_signature_name(&mut self, name: &TSIndexSignatureName<'a>) {
        walk_ts_index_signature_name(self, name);
    }

    fn visit_ts_index_signature(&mut self, signature: &TSIndexSignature<'a>) {
        walk_ts_index_signature(self, signature);
    }

    fn visit_ts_property_signature(&mut self, signature: &TSPropertySignature<'a>) {
        walk_ts_property_signature(self, signature);
    }

    fn visit_ts_call_signature_declaration(&mut self, signature: &TSCallSignatureDeclaration<'a>) {
        walk_ts_call_signature_declaration(self, signature);
    }

    fn visit_ts_type_query(&mut self, ty: &TSTypeQuery<'a>) {
        walk_ts_type_query(self, ty);
    }

    fn visit_ts_import_type(&mut self, ty: &TSImportType<'a>) {
        walk_ts_import_type(self, ty);
    }

    fn visit_ts_import_attributes(&mut self, attributes: &TSImportAttributes<'a>) {
        walk_ts_import_attributes(self, attributes);
    }

    fn visit_ts_import_attribute(&mut self, attribute: &TSImportAttribute<'a>) {
        walk_ts_import_attribute(self, attribute);
    }

    fn visit_ts_import_attribute_name(&mut self, name: &TSImportAttributeName<'a>) {
        walk_ts_import_attribute_name(self, name);
    }
}

pub mod walk {
    use super::*;

    pub fn walk_program<'a, V: Visit<'a>>(visitor: &mut V, program: &Program<'a>) {
        let kind = AstKind::Program(visitor.alloc(program));
        visitor.enter_scope({
            let mut flags = ScopeFlags::Top;
            if program.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        visitor.enter_node(kind);
        for directive in &program.directives {
            visitor.visit_directive(directive);
        }
        visitor.visit_statements(&program.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    /* ----------  Statement ---------- */

    pub fn walk_statements<'a, V: Visit<'a>>(visitor: &mut V, stmts: &Vec<'a, Statement<'a>>) {
        for stmt in stmts {
            visitor.visit_statement(stmt);
        }
    }

    pub fn walk_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &Statement<'a>) {
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

    pub fn walk_block_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &BlockStatement<'a>) {
        let kind = AstKind::BlockStatement(visitor.alloc(stmt));
        visitor.enter_scope(ScopeFlags::empty());
        visitor.enter_node(kind);
        visitor.visit_statements(&stmt.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_break_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &BreakStatement<'a>) {
        let kind = AstKind::BreakStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        if let Some(break_target) = &stmt.label {
            visitor.visit_label_identifier(break_target);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_continue_statement<'a, V: Visit<'a>>(
        visitor: &mut V,
        stmt: &ContinueStatement<'a>,
    ) {
        let kind = AstKind::ContinueStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        if let Some(continue_target) = &stmt.label {
            visitor.visit_label_identifier(continue_target);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_debugger_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &DebuggerStatement) {
        let kind = AstKind::DebuggerStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_do_while_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_statement(&stmt.body);
        visitor.visit_expression(&stmt.test);
        visitor.leave_node(kind);
    }

    pub fn walk_empty_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &EmptyStatement) {
        let kind = AstKind::EmptyStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_expression_statement<'a, V: Visit<'a>>(
        visitor: &mut V,
        stmt: &ExpressionStatement<'a>,
    ) {
        let kind = AstKind::ExpressionStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_expression(&stmt.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_for_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &ForStatement<'a>) {
        let kind = AstKind::ForStatement(visitor.alloc(stmt));
        let is_lexical_declaration =
            stmt.init.as_ref().is_some_and(ForStatementInit::is_lexical_declaration);
        if is_lexical_declaration {
            visitor.enter_scope(ScopeFlags::empty());
        }
        visitor.enter_node(kind);
        if let Some(init) = &stmt.init {
            visitor.visit_for_statement_init(init);
        }
        if let Some(test) = &stmt.test {
            visitor.visit_expression(test);
        }
        if let Some(update) = &stmt.update {
            visitor.visit_expression(update);
        }
        visitor.visit_statement(&stmt.body);
        visitor.leave_node(kind);
        if is_lexical_declaration {
            visitor.leave_scope();
        }
    }

    pub fn walk_for_statement_init<'a, V: Visit<'a>>(visitor: &mut V, init: &ForStatementInit<'a>) {
        let kind = AstKind::ForStatementInit(visitor.alloc(init));
        visitor.enter_node(kind);
        match init {
            ForStatementInit::UsingDeclaration(decl) => {
                visitor.visit_using_declaration(decl);
            }
            ForStatementInit::VariableDeclaration(decl) => {
                visitor.visit_variable_declaration(decl);
            }
            ForStatementInit::Expression(expr) => visitor.visit_expression(expr),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_for_in_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(visitor.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            visitor.enter_scope(ScopeFlags::empty());
        }
        visitor.enter_node(kind);
        visitor.visit_for_statement_left(&stmt.left);
        visitor.visit_expression(&stmt.right);
        visitor.visit_statement(&stmt.body);
        visitor.leave_node(kind);
        if is_lexical_declaration {
            visitor.leave_scope();
        }
    }

    pub fn walk_for_of_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(visitor.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            visitor.enter_scope(ScopeFlags::empty());
        }
        visitor.enter_node(kind);
        visitor.visit_for_statement_left(&stmt.left);
        visitor.visit_expression(&stmt.right);
        visitor.visit_statement(&stmt.body);
        visitor.leave_node(kind);
        if is_lexical_declaration {
            visitor.leave_scope();
        }
    }

    pub fn walk_for_statement_left<'a, V: Visit<'a>>(visitor: &mut V, left: &ForStatementLeft<'a>) {
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

    pub fn walk_if_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &IfStatement<'a>) {
        let kind = AstKind::IfStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_expression(&stmt.test);
        visitor.visit_statement(&stmt.consequent);
        if let Some(alternate) = &stmt.alternate {
            visitor.visit_statement(alternate);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_labeled_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &LabeledStatement<'a>) {
        let kind = AstKind::LabeledStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_label_identifier(&stmt.label);
        visitor.visit_statement(&stmt.body);
        visitor.leave_node(kind);
    }

    pub fn walk_return_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &ReturnStatement<'a>) {
        let kind = AstKind::ReturnStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        if let Some(arg) = &stmt.argument {
            visitor.visit_expression(arg);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_switch_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_expression(&stmt.discriminant);
        visitor.enter_scope(ScopeFlags::empty());
        for case in &stmt.cases {
            visitor.visit_switch_case(case);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    pub fn walk_switch_case<'a, V: Visit<'a>>(visitor: &mut V, case: &SwitchCase<'a>) {
        let kind = AstKind::SwitchCase(visitor.alloc(case));
        visitor.enter_node(kind);
        if let Some(expr) = &case.test {
            visitor.visit_expression(expr);
        }
        visitor.visit_statements(&case.consequent);
        visitor.leave_node(kind);
    }

    pub fn walk_throw_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &ThrowStatement<'a>) {
        let kind = AstKind::ThrowStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_expression(&stmt.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_try_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &TryStatement<'a>) {
        let kind = AstKind::TryStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_block_statement(&stmt.block);
        if let Some(handler) = &stmt.handler {
            visitor.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &stmt.finalizer {
            visitor.visit_finally_clause(finalizer);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_catch_clause<'a, V: Visit<'a>>(visitor: &mut V, clause: &CatchClause<'a>) {
        let kind = AstKind::CatchClause(visitor.alloc(clause));
        visitor.enter_scope(ScopeFlags::empty());
        visitor.enter_node(kind);
        if let Some(param) = &clause.param {
            visitor.visit_binding_pattern(param);
        }
        visitor.visit_statements(&clause.body.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_finally_clause<'a, V: Visit<'a>>(visitor: &mut V, clause: &BlockStatement<'a>) {
        let kind = AstKind::FinallyClause(visitor.alloc(clause));
        visitor.enter_node(kind);
        visitor.visit_block_statement(clause);
        visitor.leave_node(kind);
    }

    pub fn walk_while_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_expression(&stmt.test);
        visitor.visit_statement(&stmt.body);
        visitor.leave_node(kind);
    }

    pub fn walk_with_statement<'a, V: Visit<'a>>(visitor: &mut V, stmt: &WithStatement<'a>) {
        let kind = AstKind::WithStatement(visitor.alloc(stmt));
        visitor.enter_node(kind);
        visitor.visit_expression(&stmt.object);
        visitor.visit_statement(&stmt.body);
        visitor.leave_node(kind);
    }

    pub fn walk_directive<'a, V: Visit<'a>>(visitor: &mut V, directive: &Directive<'a>) {
        let kind = AstKind::Directive(visitor.alloc(directive));
        visitor.enter_node(kind);
        visitor.visit_string_literal(&directive.expression);
        visitor.leave_node(kind);
    }

    /* ----------  Declaration ---------- */

    pub fn walk_variable_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &VariableDeclaration<'a>,
    ) {
        let kind = AstKind::VariableDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        for declarator in &decl.declarations {
            visitor.visit_variable_declarator(declarator);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_variable_declarator<'a, V: Visit<'a>>(
        visitor: &mut V,
        declarator: &VariableDeclarator<'a>,
    ) {
        let kind = AstKind::VariableDeclarator(visitor.alloc(declarator));
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&declarator.id);
        if let Some(init) = &declarator.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    /* ----------  Function ---------- */

    pub fn walk_function<'a, V: Visit<'a>>(
        visitor: &mut V,
        func: &Function<'a>,
        flags: Option<ScopeFlags>,
    ) {
        let kind = AstKind::Function(visitor.alloc(func));
        visitor.enter_scope({
            let mut flags = flags.unwrap_or(ScopeFlags::empty()) | ScopeFlags::Function;
            if func.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        visitor.enter_node(kind);
        if let Some(ident) = &func.id {
            visitor.visit_binding_identifier(ident);
        }
        visitor.visit_formal_parameters(&func.params);
        if let Some(body) = &func.body {
            visitor.visit_function_body(body);
        }
        if let Some(parameters) = &func.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &func.return_type {
            visitor.visit_ts_type_annotation(annotation);
        }
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_function_body<'a, V: Visit<'a>>(visitor: &mut V, body: &FunctionBody<'a>) {
        let kind = AstKind::FunctionBody(visitor.alloc(body));
        visitor.enter_node(kind);
        for directive in &body.directives {
            visitor.visit_directive(directive);
        }
        visitor.visit_statements(&body.statements);
        visitor.leave_node(kind);
    }

    pub fn walk_formal_parameters<'a, V: Visit<'a>>(
        visitor: &mut V,
        params: &FormalParameters<'a>,
    ) {
        let kind = AstKind::FormalParameters(visitor.alloc(params));
        visitor.enter_node(kind);
        for param in &params.items {
            visitor.visit_formal_parameter(param);
        }
        if let Some(rest) = &params.rest {
            visitor.visit_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_formal_parameter<'a, V: Visit<'a>>(visitor: &mut V, param: &FormalParameter<'a>) {
        let kind = AstKind::FormalParameter(visitor.alloc(param));
        visitor.enter_node(kind);
        for decorator in &param.decorators {
            visitor.visit_decorator(decorator);
        }
        visitor.visit_binding_pattern(&param.pattern);
        visitor.leave_node(kind);
    }

    /* ----------  Class ---------- */

    pub fn walk_decorator<'a, V: Visit<'a>>(visitor: &mut V, decorator: &Decorator<'a>) {
        let kind = AstKind::Decorator(visitor.alloc(decorator));
        visitor.enter_node(kind);
        visitor.visit_expression(&decorator.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_class<'a, V: Visit<'a>>(visitor: &mut V, class: &Class<'a>) {
        // Class level decorators are transpiled as functions outside of the class taking the class
        // itvisitor as argument. They should be visited before class is entered. E.g., they inherit
        // strict mode from the enclosing scope rather than from class.
        for decorator in &class.decorators {
            visitor.visit_decorator(decorator);
        }
        let kind = AstKind::Class(visitor.alloc(class));

        // FIXME(don): Should we enter a scope when visiting class declarations?
        let is_class_expr = class.r#type == ClassType::ClassExpression;
        if is_class_expr {
            // Class expressions create a temporary scope with the class name as its only variable
            // E.g., `let c = class A { foo() { console.log(A) } }`
            visitor.enter_scope(ScopeFlags::empty());
        }

        visitor.enter_node(kind);

        if let Some(id) = &class.id {
            visitor.visit_binding_identifier(id);
        }
        if let Some(parameters) = &class.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(super_class) = &class.super_class {
            visitor.visit_class_heritage(super_class);
        }
        if let Some(super_parameters) = &class.super_type_parameters {
            visitor.visit_ts_type_parameter_instantiation(super_parameters);
        }
        visitor.visit_class_body(&class.body);
        visitor.leave_node(kind);
        if is_class_expr {
            visitor.leave_scope();
        }
    }

    pub fn walk_class_heritage<'a, V: Visit<'a>>(visitor: &mut V, expr: &Expression<'a>) {
        let kind = AstKind::ClassHeritage(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(expr);
        visitor.leave_node(kind);
    }

    pub fn walk_class_body<'a, V: Visit<'a>>(visitor: &mut V, body: &ClassBody<'a>) {
        let kind = AstKind::ClassBody(visitor.alloc(body));
        visitor.enter_node(kind);
        for elem in &body.body {
            visitor.visit_class_element(elem);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_class_element<'a, V: Visit<'a>>(visitor: &mut V, elem: &ClassElement<'a>) {
        match elem {
            ClassElement::StaticBlock(block) => visitor.visit_static_block(block),
            ClassElement::MethodDefinition(def) => visitor.visit_method_definition(def),
            ClassElement::PropertyDefinition(def) => visitor.visit_property_definition(def),
            ClassElement::AccessorProperty(_def) => { /* TODO */ }
            ClassElement::TSIndexSignature(sig) => visitor.visit_ts_index_signature(sig),
        }
    }

    pub fn walk_static_block<'a, V: Visit<'a>>(visitor: &mut V, block: &StaticBlock<'a>) {
        let kind = AstKind::StaticBlock(visitor.alloc(block));
        visitor.enter_scope(ScopeFlags::ClassStaticBlock);
        visitor.enter_node(kind);
        visitor.visit_statements(&block.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_method_definition<'a, V: Visit<'a>>(visitor: &mut V, def: &MethodDefinition<'a>) {
        let kind = AstKind::MethodDefinition(visitor.alloc(def));
        visitor.enter_node(kind);
        for decorator in &def.decorators {
            visitor.visit_decorator(decorator);
        }
        let flags = match def.kind {
            MethodDefinitionKind::Get => ScopeFlags::GetAccessor,
            MethodDefinitionKind::Set => ScopeFlags::SetAccessor,
            MethodDefinitionKind::Constructor => ScopeFlags::Constructor,
            MethodDefinitionKind::Method => ScopeFlags::empty(),
        };
        visitor.visit_property_key(&def.key);
        visitor.visit_function(&def.value, Some(flags));
        visitor.leave_node(kind);
    }

    pub fn walk_property_definition<'a, V: Visit<'a>>(
        visitor: &mut V,
        def: &PropertyDefinition<'a>,
    ) {
        let kind = AstKind::PropertyDefinition(visitor.alloc(def));
        visitor.enter_node(kind);
        for decorator in &def.decorators {
            visitor.visit_decorator(decorator);
        }
        visitor.visit_property_key(&def.key);
        if let Some(value) = &def.value {
            visitor.visit_expression(value);
        }
        if let Some(annotation) = &def.type_annotation {
            visitor.visit_ts_type_annotation(annotation);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_using_declaration<'a, V: Visit<'a>>(visitor: &mut V, decl: &UsingDeclaration<'a>) {
        let kind = AstKind::UsingDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        for decl in &decl.declarations {
            visitor.visit_variable_declarator(decl);
        }
        visitor.leave_node(kind);
    }

    /* ----------  Expression ---------- */

    pub fn walk_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &Expression<'a>) {
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
            Expression::ClassExpression(expr) => {
                debug_assert_eq!(expr.r#type, ClassType::ClassExpression);
                visitor.visit_class(expr);
            }
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

    pub fn walk_meta_property<'a, V: Visit<'a>>(visitor: &mut V, meta: &MetaProperty<'a>) {
        let kind = AstKind::MetaProperty(visitor.alloc(meta));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_array_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &ArrayExpression<'a>) {
        let kind = AstKind::ArrayExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        for elem in &expr.elements {
            visitor.visit_array_expression_element(elem);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_array_expression_element<'a, V: Visit<'a>>(
        visitor: &mut V,
        arg: &ArrayExpressionElement<'a>,
    ) {
        let kind = AstKind::ArrayExpressionElement(visitor.alloc(arg));
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

    pub fn walk_argument<'a, V: Visit<'a>>(visitor: &mut V, arg: &Argument<'a>) {
        let kind = AstKind::Argument(visitor.alloc(arg));
        visitor.enter_node(kind);
        match arg {
            Argument::SpreadElement(spread) => visitor.visit_spread_element(spread),
            Argument::Expression(expr) => visitor.visit_expression(expr),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_spread_element<'a, V: Visit<'a>>(visitor: &mut V, elem: &SpreadElement<'a>) {
        let kind = AstKind::SpreadElement(visitor.alloc(elem));
        visitor.enter_node(kind);
        visitor.visit_expression(&elem.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_expression_array_element<'a, V: Visit<'a>>(visitor: &mut V, expr: &Expression<'a>) {
        let kind = AstKind::ExpressionArrayElement(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(expr);
        visitor.leave_node(kind);
    }

    pub fn walk_elision<'a, V: Visit<'a>>(visitor: &mut V, span: Span) {
        let kind = AstKind::Elision(span);
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_assignment_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &AssignmentExpression<'a>,
    ) {
        let kind = AstKind::AssignmentExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_assignment_target(&expr.left);
        visitor.visit_expression(&expr.right);
        visitor.leave_node(kind);
    }

    pub fn walk_arrow_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &ArrowFunctionExpression<'a>,
    ) {
        let kind = AstKind::ArrowFunctionExpression(visitor.alloc(expr));
        visitor.enter_scope(ScopeFlags::Function | ScopeFlags::Arrow);
        visitor.enter_node(kind);
        visitor.visit_formal_parameters(&expr.params);
        visitor.visit_function_body(&expr.body);
        if let Some(parameters) = &expr.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_await_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &AwaitExpression<'a>) {
        let kind = AstKind::AwaitExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_binary_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &BinaryExpression<'a>) {
        let kind = AstKind::BinaryExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.left);
        visitor.visit_expression(&expr.right);
        visitor.leave_node(kind);
    }

    pub fn walk_call_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &CallExpression<'a>) {
        let kind = AstKind::CallExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        for arg in &expr.arguments {
            visitor.visit_argument(arg);
        }
        visitor.visit_expression(&expr.callee);
        if let Some(parameters) = &expr.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(parameters);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_chain_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &ChainExpression<'a>) {
        let kind = AstKind::ChainExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_chain_element(&expr.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_chain_element<'a, V: Visit<'a>>(visitor: &mut V, elem: &ChainElement<'a>) {
        match elem {
            ChainElement::CallExpression(expr) => visitor.visit_call_expression(expr),
            ChainElement::MemberExpression(expr) => visitor.visit_member_expression(expr),
        }
    }

    pub fn walk_conditional_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &ConditionalExpression<'a>,
    ) {
        let kind = AstKind::ConditionalExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.test);
        visitor.visit_expression(&expr.consequent);
        visitor.visit_expression(&expr.alternate);
        visitor.leave_node(kind);
    }

    pub fn walk_import_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &ImportExpression<'a>) {
        let kind = AstKind::ImportExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.source);
        for arg in &expr.arguments {
            visitor.visit_expression(arg);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_logical_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &LogicalExpression<'a>,
    ) {
        let kind = AstKind::LogicalExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.left);
        visitor.visit_expression(&expr.right);
        visitor.leave_node(kind);
    }

    pub fn walk_member_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &MemberExpression<'a>) {
        let kind = AstKind::MemberExpression(visitor.alloc(expr));
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

    pub fn walk_computed_member_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &ComputedMemberExpression<'a>,
    ) {
        visitor.visit_expression(&expr.object);
        visitor.visit_expression(&expr.expression);
    }

    pub fn walk_static_member_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &StaticMemberExpression<'a>,
    ) {
        visitor.visit_expression(&expr.object);
        visitor.visit_identifier_name(&expr.property);
    }

    pub fn walk_private_field_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &PrivateFieldExpression<'a>,
    ) {
        visitor.visit_expression(&expr.object);
        visitor.visit_private_identifier(&expr.field);
    }

    pub fn walk_new_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &NewExpression<'a>) {
        let kind = AstKind::NewExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.callee);
        if let Some(parameters) = &expr.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(parameters);
        }
        for arg in &expr.arguments {
            visitor.visit_argument(arg);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_object_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &ObjectExpression<'a>) {
        let kind = AstKind::ObjectExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        for prop in &expr.properties {
            visitor.visit_object_property_kind(prop);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_object_property_kind<'a, V: Visit<'a>>(
        visitor: &mut V,
        prop: &ObjectPropertyKind<'a>,
    ) {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => visitor.visit_object_property(prop),
            ObjectPropertyKind::SpreadProperty(elem) => visitor.visit_spread_element(elem),
        }
    }

    pub fn walk_object_property<'a, V: Visit<'a>>(visitor: &mut V, prop: &ObjectProperty<'a>) {
        let kind = AstKind::ObjectProperty(visitor.alloc(prop));
        visitor.enter_node(kind);
        visitor.visit_property_key(&prop.key);
        visitor.visit_expression(&prop.value);
        visitor.leave_node(kind);
    }

    pub fn walk_property_key<'a, V: Visit<'a>>(visitor: &mut V, key: &PropertyKey<'a>) {
        let kind = AstKind::PropertyKey(visitor.alloc(key));
        visitor.enter_node(kind);
        match key {
            PropertyKey::Identifier(ident) => visitor.visit_identifier_name(ident),
            PropertyKey::PrivateIdentifier(ident) => visitor.visit_private_identifier(ident),
            PropertyKey::Expression(expr) => visitor.visit_expression(expr),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_parenthesized_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &ParenthesizedExpression<'a>,
    ) {
        let kind = AstKind::ParenthesizedExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_private_in_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &PrivateInExpression<'a>,
    ) {
        let kind = AstKind::PrivateInExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_private_identifier(&expr.left);
        visitor.visit_expression(&expr.right);
        visitor.leave_node(kind);
    }

    pub fn walk_sequence_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &SequenceExpression<'a>,
    ) {
        let kind = AstKind::SequenceExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        for expr in &expr.expressions {
            visitor.visit_expression(expr);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_tagged_template_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &TaggedTemplateExpression<'a>,
    ) {
        let kind = AstKind::TaggedTemplateExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.tag);
        visitor.visit_template_literal(&expr.quasi);
        visitor.leave_node(kind);
    }

    pub fn walk_this_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &ThisExpression) {
        let kind = AstKind::ThisExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_unary_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &UnaryExpression<'a>) {
        let kind = AstKind::UnaryExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_update_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &UpdateExpression<'a>) {
        let kind = AstKind::UpdateExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_simple_assignment_target(&expr.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_yield_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &YieldExpression<'a>) {
        let kind = AstKind::YieldExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        if let Some(argument) = &expr.argument {
            visitor.visit_expression(argument);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_super<'a, V: Visit<'a>>(visitor: &mut V, expr: &Super) {
        let kind = AstKind::Super(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_assignment_target<'a, V: Visit<'a>>(
        visitor: &mut V,
        target: &AssignmentTarget<'a>,
    ) {
        let kind = AstKind::AssignmentTarget(visitor.alloc(target));
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

    pub fn walk_simple_assignment_target<'a, V: Visit<'a>>(
        visitor: &mut V,
        target: &SimpleAssignmentTarget<'a>,
    ) {
        let kind = AstKind::SimpleAssignmentTarget(visitor.alloc(target));
        visitor.enter_node(kind);
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                visitor.visit_identifier_reference(ident);
            }
            SimpleAssignmentTarget::MemberAssignmentTarget(expr) => {
                visitor.visit_member_expression(expr);
            }
            SimpleAssignmentTarget::TSAsExpression(expr) => {
                visitor.visit_expression(&expr.expression);
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(expr) => {
                visitor.visit_expression(&expr.expression);
            }
            SimpleAssignmentTarget::TSNonNullExpression(expr) => {
                visitor.visit_expression(&expr.expression);
            }
            SimpleAssignmentTarget::TSTypeAssertion(expr) => {
                visitor.visit_expression(&expr.expression);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_assignment_target_pattern<'a, V: Visit<'a>>(
        visitor: &mut V,
        pat: &AssignmentTargetPattern<'a>,
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

    pub fn walk_array_assignment_target<'a, V: Visit<'a>>(
        visitor: &mut V,
        target: &ArrayAssignmentTarget<'a>,
    ) {
        for element in target.elements.iter().flatten() {
            visitor.visit_assignment_target_maybe_default(element);
        }
        if let Some(target) = &target.rest {
            visitor.visit_assignment_target_rest(target);
        }
    }

    pub fn walk_assignment_target_maybe_default<'a, V: Visit<'a>>(
        visitor: &mut V,
        target: &AssignmentTargetMaybeDefault<'a>,
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

    pub fn walk_assignment_target_with_default<'a, V: Visit<'a>>(
        visitor: &mut V,
        target: &AssignmentTargetWithDefault<'a>,
    ) {
        let kind = AstKind::AssignmentTargetWithDefault(visitor.alloc(target));
        visitor.enter_node(kind);
        visitor.visit_assignment_target(&target.binding);
        visitor.visit_expression(&target.init);
        visitor.leave_node(kind);
    }

    pub fn walk_object_assignment_target<'a, V: Visit<'a>>(
        visitor: &mut V,
        target: &ObjectAssignmentTarget<'a>,
    ) {
        for property in &target.properties {
            visitor.visit_assignment_target_property(property);
        }
        if let Some(target) = &target.rest {
            visitor.visit_assignment_target_rest(target);
        }
    }

    pub fn walk_assignment_target_property<'a, V: Visit<'a>>(
        visitor: &mut V,
        property: &AssignmentTargetProperty<'a>,
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

    pub fn walk_assignment_target_property_identifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        ident: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        visitor.visit_identifier_reference(&ident.binding);
        if let Some(expr) = &ident.init {
            visitor.visit_expression(expr);
        }
    }

    pub fn walk_assignment_target_property_property<'a, V: Visit<'a>>(
        visitor: &mut V,
        property: &AssignmentTargetPropertyProperty<'a>,
    ) {
        visitor.visit_property_key(&property.name);
        visitor.visit_assignment_target_maybe_default(&property.binding);
    }

    pub fn walk_assignment_target_rest<'a, V: Visit<'a>>(
        visitor: &mut V,
        rest: &AssignmentTargetRest<'a>,
    ) {
        visitor.visit_assignment_target(&rest.target);
    }

    /* ----------  Expression ---------- */

    pub fn walk_jsx_element<'a, V: Visit<'a>>(visitor: &mut V, elem: &JSXElement<'a>) {
        let kind = AstKind::JSXElement(visitor.alloc(elem));
        visitor.enter_node(kind);
        visitor.visit_jsx_opening_element(&elem.opening_element);
        for child in &elem.children {
            visitor.visit_jsx_child(child);
        }
        if let Some(closing_elem) = &elem.closing_element {
            visitor.visit_jsx_closing_element(closing_elem);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_opening_element<'a, V: Visit<'a>>(
        visitor: &mut V,
        elem: &JSXOpeningElement<'a>,
    ) {
        let kind = AstKind::JSXOpeningElement(visitor.alloc(elem));
        visitor.enter_node(kind);
        visitor.visit_jsx_element_name(&elem.name);
        for attribute in &elem.attributes {
            visitor.visit_jsx_attribute_item(attribute);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_closing_element<'a, V: Visit<'a>>(
        visitor: &mut V,
        elem: &JSXClosingElement<'a>,
    ) {
        let kind = AstKind::JSXClosingElement(visitor.alloc(elem));
        visitor.enter_node(kind);
        visitor.visit_jsx_element_name(&elem.name);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_element_name<'a, V: Visit<'a>>(visitor: &mut V, name: &JSXElementName<'a>) {
        let kind = AstKind::JSXElementName(visitor.alloc(name));
        visitor.enter_node(kind);
        match name {
            JSXElementName::Identifier(ident) => visitor.visit_jsx_identifier(ident),
            JSXElementName::NamespacedName(expr) => visitor.visit_jsx_namespaced_name(expr),
            JSXElementName::MemberExpression(expr) => visitor.visit_jsx_member_expression(expr),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_identifier<'a, V: Visit<'a>>(visitor: &mut V, ident: &JSXIdentifier<'a>) {
        let kind = AstKind::JSXIdentifier(visitor.alloc(ident));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_member_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &JSXMemberExpression<'a>,
    ) {
        let kind = AstKind::JSXMemberExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_jsx_member_expression_object(&expr.object);
        visitor.visit_jsx_identifier(&expr.property);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_member_expression_object<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &JSXMemberExpressionObject<'a>,
    ) {
        let kind = AstKind::JSXMemberExpressionObject(visitor.alloc(expr));
        visitor.enter_node(kind);
        match expr {
            JSXMemberExpressionObject::Identifier(ident) => visitor.visit_jsx_identifier(ident),
            JSXMemberExpressionObject::MemberExpression(expr) => {
                visitor.visit_jsx_member_expression(expr);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_namespaced_name<'a, V: Visit<'a>>(
        visitor: &mut V,
        name: &JSXNamespacedName<'a>,
    ) {
        let kind = AstKind::JSXNamespacedName(visitor.alloc(name));
        visitor.enter_node(kind);
        visitor.visit_jsx_identifier(&name.namespace);
        visitor.visit_jsx_identifier(&name.property);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_attribute_item<'a, V: Visit<'a>>(visitor: &mut V, item: &JSXAttributeItem<'a>) {
        let kind = AstKind::JSXAttributeItem(visitor.alloc(item));
        visitor.enter_node(kind);
        match &item {
            JSXAttributeItem::Attribute(attribute) => visitor.visit_jsx_attribute(attribute),
            JSXAttributeItem::SpreadAttribute(attribute) => {
                visitor.visit_jsx_spread_attribute(attribute);
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_attribute<'a, V: Visit<'a>>(visitor: &mut V, attribute: &JSXAttribute<'a>) {
        if let Some(value) = &attribute.value {
            visitor.visit_jsx_attribute_value(value);
        }
    }

    pub fn walk_jsx_spread_attribute<'a, V: Visit<'a>>(
        visitor: &mut V,
        attribute: &JSXSpreadAttribute<'a>,
    ) {
        visitor.visit_expression(&attribute.argument);
    }

    pub fn walk_jsx_attribute_value<'a, V: Visit<'a>>(
        visitor: &mut V,
        value: &JSXAttributeValue<'a>,
    ) {
        match value {
            JSXAttributeValue::ExpressionContainer(expr) => {
                visitor.visit_jsx_expression_container(expr);
            }
            JSXAttributeValue::Element(elem) => visitor.visit_jsx_element(elem),
            JSXAttributeValue::Fragment(elem) => visitor.visit_jsx_fragment(elem),
            JSXAttributeValue::StringLiteral(lit) => visitor.visit_string_literal(lit),
        }
    }

    pub fn walk_jsx_expression_container<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &JSXExpressionContainer<'a>,
    ) {
        let kind = AstKind::JSXExpressionContainer(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_jsx_expression(&expr.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &JSXExpression<'a>) {
        match expr {
            JSXExpression::Expression(expr) => visitor.visit_expression(expr),
            JSXExpression::EmptyExpression(_) => {}
        }
    }

    pub fn walk_jsx_fragment<'a, V: Visit<'a>>(visitor: &mut V, elem: &JSXFragment<'a>) {
        let kind = AstKind::JSXFragment(visitor.alloc(elem));
        visitor.enter_node(kind);
        for child in &elem.children {
            visitor.visit_jsx_child(child);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_jsx_child<'a, V: Visit<'a>>(visitor: &mut V, child: &JSXChild<'a>) {
        match child {
            JSXChild::Element(elem) => visitor.visit_jsx_element(elem),
            JSXChild::Fragment(elem) => visitor.visit_jsx_fragment(elem),
            JSXChild::ExpressionContainer(expr) => visitor.visit_jsx_expression_container(expr),
            JSXChild::Spread(expr) => visitor.visit_jsx_spread_child(expr),
            JSXChild::Text(expr) => visitor.visit_jsx_text(expr),
        }
    }

    pub fn walk_jsx_spread_child<'a, V: Visit<'a>>(visitor: &mut V, child: &JSXSpreadChild<'a>) {
        visitor.visit_expression(&child.expression);
    }

    pub fn walk_jsx_text<'a, V: Visit<'a>>(visitor: &mut V, child: &JSXText<'a>) {
        let kind = AstKind::JSXText(visitor.alloc(child));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    /* ----------  Pattern ---------- */

    pub fn walk_binding_pattern<'a, V: Visit<'a>>(visitor: &mut V, pat: &BindingPattern<'a>) {
        match &pat.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                visitor.visit_binding_identifier(ident);
            }
            BindingPatternKind::ObjectPattern(pat) => visitor.visit_object_pattern(pat),
            BindingPatternKind::ArrayPattern(pat) => visitor.visit_array_pattern(pat),
            BindingPatternKind::AssignmentPattern(pat) => visitor.visit_assignment_pattern(pat),
        }
        if let Some(type_annotation) = &pat.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
    }

    pub fn walk_binding_identifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        ident: &BindingIdentifier<'a>,
    ) {
        let kind = AstKind::BindingIdentifier(visitor.alloc(ident));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_object_pattern<'a, V: Visit<'a>>(visitor: &mut V, pat: &ObjectPattern<'a>) {
        let kind = AstKind::ObjectPattern(visitor.alloc(pat));
        visitor.enter_node(kind);
        for prop in &pat.properties {
            visitor.visit_binding_property(prop);
        }
        if let Some(rest) = &pat.rest {
            visitor.visit_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_binding_property<'a, V: Visit<'a>>(visitor: &mut V, prop: &BindingProperty<'a>) {
        visitor.visit_property_key(&prop.key);
        visitor.visit_binding_pattern(&prop.value);
    }

    pub fn walk_array_pattern<'a, V: Visit<'a>>(visitor: &mut V, pat: &ArrayPattern<'a>) {
        let kind = AstKind::ArrayPattern(visitor.alloc(pat));
        visitor.enter_node(kind);
        for pat in pat.elements.iter().flatten() {
            visitor.visit_binding_pattern(pat);
        }
        if let Some(rest) = &pat.rest {
            visitor.visit_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_rest_element<'a, V: Visit<'a>>(visitor: &mut V, pat: &BindingRestElement<'a>) {
        let kind = AstKind::BindingRestElement(visitor.alloc(pat));
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&pat.argument);
        visitor.leave_node(kind);
    }

    pub fn walk_assignment_pattern<'a, V: Visit<'a>>(visitor: &mut V, pat: &AssignmentPattern<'a>) {
        let kind = AstKind::AssignmentPattern(visitor.alloc(pat));
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&pat.left);
        visitor.visit_expression(&pat.right);
        visitor.leave_node(kind);
    }

    /* ----------  Identifier ---------- */

    pub fn walk_identifier_reference<'a, V: Visit<'a>>(
        visitor: &mut V,
        ident: &IdentifierReference<'a>,
    ) {
        let kind = AstKind::IdentifierReference(visitor.alloc(ident));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_private_identifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        ident: &PrivateIdentifier<'a>,
    ) {
        let kind = AstKind::PrivateIdentifier(visitor.alloc(ident));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_label_identifier<'a, V: Visit<'a>>(visitor: &mut V, ident: &LabelIdentifier<'a>) {
        let kind = AstKind::LabelIdentifier(visitor.alloc(ident));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_identifier_name<'a, V: Visit<'a>>(visitor: &mut V, ident: &IdentifierName<'a>) {
        let kind = AstKind::IdentifierName(visitor.alloc(ident));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    /* ----------  Literal ---------- */

    pub fn walk_number_literal<'a, V: Visit<'a>>(visitor: &mut V, lit: &NumericLiteral<'a>) {
        let kind = AstKind::NumericLiteral(visitor.alloc(lit));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_boolean_literal<'a, V: Visit<'a>>(visitor: &mut V, lit: &BooleanLiteral) {
        let kind = AstKind::BooleanLiteral(visitor.alloc(lit));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_null_literal<'a, V: Visit<'a>>(visitor: &mut V, lit: &NullLiteral) {
        let kind = AstKind::NullLiteral(visitor.alloc(lit));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_bigint_literal<'a, V: Visit<'a>>(visitor: &mut V, lit: &BigIntLiteral<'a>) {
        let kind = AstKind::BigintLiteral(visitor.alloc(lit));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_string_literal<'a, V: Visit<'a>>(visitor: &mut V, lit: &StringLiteral<'a>) {
        let kind = AstKind::StringLiteral(visitor.alloc(lit));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_template_literal<'a, V: Visit<'a>>(visitor: &mut V, lit: &TemplateLiteral<'a>) {
        let kind = AstKind::TemplateLiteral(visitor.alloc(lit));
        visitor.enter_node(kind);
        for elem in &lit.quasis {
            visitor.visit_template_element(elem);
        }
        for expr in &lit.expressions {
            visitor.visit_expression(expr);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_reg_expr_literal<'a, V: Visit<'a>>(visitor: &mut V, lit: &RegExpLiteral<'a>) {
        let kind = AstKind::RegExpLiteral(visitor.alloc(lit));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_template_element<'a, V: Visit<'a>>(_visitor: &mut V, _elem: &TemplateElement) {}

    /* ----------  Module ---------- */

    pub fn walk_module_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &ModuleDeclaration<'a>,
    ) {
        let kind = AstKind::ModuleDeclaration(visitor.alloc(decl));
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
                visitor.visit_expression(&decl.expression);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(_) => {}
        }
        visitor.leave_node(kind);
    }

    pub fn walk_import_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &ImportDeclaration<'a>,
    ) {
        let kind = AstKind::ImportDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        if let Some(specifiers) = &decl.specifiers {
            for specifier in specifiers {
                visitor.visit_import_declaration_specifier(specifier);
            }
        }
        visitor.visit_string_literal(&decl.source);
        if let Some(with_clause) = &decl.with_clause {
            visitor.visit_with_clause(with_clause);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_with_clause<'a, V: Visit<'a>>(visitor: &mut V, with_clause: &WithClause<'a>) {
        for attribute in &with_clause.with_entries {
            visitor.visit_import_attribute(attribute);
        }
    }

    pub fn walk_import_attribute<'a, V: Visit<'a>>(
        visitor: &mut V,
        attribute: &ImportAttribute<'a>,
    ) {
        visitor.visit_import_attribute_key(&attribute.key);
        visitor.visit_string_literal(&attribute.value);
    }

    pub fn walk_import_attribute_key<'a, V: Visit<'a>>(
        visitor: &mut V,
        key: &ImportAttributeKey<'a>,
    ) {
        match key {
            ImportAttributeKey::Identifier(ident) => visitor.visit_identifier_name(ident),
            ImportAttributeKey::StringLiteral(ident) => visitor.visit_string_literal(ident),
        }
    }

    pub fn walk_import_declaration_specifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        specifier: &ImportDeclarationSpecifier<'a>,
    ) {
        match &specifier {
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

    pub fn walk_import_specifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        specifier: &ImportSpecifier<'a>,
    ) {
        let kind = AstKind::ImportSpecifier(visitor.alloc(specifier));
        visitor.enter_node(kind);
        // TODO: imported
        visitor.visit_binding_identifier(&specifier.local);
        visitor.leave_node(kind);
    }

    pub fn walk_import_default_specifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        specifier: &ImportDefaultSpecifier<'a>,
    ) {
        let kind = AstKind::ImportDefaultSpecifier(visitor.alloc(specifier));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&specifier.local);
        visitor.leave_node(kind);
    }

    pub fn walk_import_name_specifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        specifier: &ImportNamespaceSpecifier<'a>,
    ) {
        let kind = AstKind::ImportNamespaceSpecifier(visitor.alloc(specifier));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&specifier.local);
        visitor.leave_node(kind);
    }

    pub fn walk_export_all_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &ExportAllDeclaration<'a>,
    ) {
        let kind = AstKind::ExportAllDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        visitor.visit_string_literal(&decl.source);
        visitor.leave_node(kind);
    }

    pub fn walk_export_default_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &ExportDefaultDeclaration<'a>,
    ) {
        let kind = AstKind::ExportDefaultDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        match &decl.declaration {
            ExportDefaultDeclarationKind::Expression(expr) => visitor.visit_expression(expr),
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                visitor.visit_function(func, None);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => visitor.visit_class(class),
            _ => {}
        }
        visitor.leave_node(kind);
    }

    pub fn walk_export_named_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &ExportNamedDeclaration<'a>,
    ) {
        let kind = AstKind::ExportNamedDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        if let Some(decl) = &decl.declaration {
            visitor.visit_declaration(decl);
        }
        if let Some(ref source) = decl.source {
            visitor.visit_string_literal(source);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_enum_member<'a, V: Visit<'a>>(visitor: &mut V, member: &TSEnumMember<'a>) {
        let kind = AstKind::TSEnumMember(visitor.alloc(member));
        visitor.enter_node(kind);

        if let Some(initializer) = &member.initializer {
            visitor.visit_expression(initializer);
        }

        visitor.leave_node(kind);
    }

    pub fn walk_enum<'a, V: Visit<'a>>(visitor: &mut V, decl: &TSEnumDeclaration<'a>) {
        let kind = AstKind::TSEnumDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&decl.id);
        visitor.enter_scope(ScopeFlags::empty());
        for member in &decl.members {
            visitor.visit_enum_member(member);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    pub fn walk_declaration<'a, V: Visit<'a>>(visitor: &mut V, decl: &Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(decl) => visitor.visit_variable_declaration(decl),
            Declaration::FunctionDeclaration(func) => visitor.visit_function(func, None),
            Declaration::ClassDeclaration(class) => {
                debug_assert_eq!(class.r#type, ClassType::ClassDeclaration);
                visitor.visit_class(class);
            }
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

    pub fn walk_ts_import_equals_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &TSImportEqualsDeclaration<'a>,
    ) {
        let kind = AstKind::TSImportEqualsDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&decl.id);
        visitor.visit_ts_module_reference(&decl.module_reference);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_module_reference<'a, V: Visit<'a>>(
        visitor: &mut V,
        reference: &TSModuleReference<'a>,
    ) {
        match reference {
            TSModuleReference::TypeName(name) => visitor.visit_ts_type_name(name),
            TSModuleReference::ExternalModuleReference(reference) => {
                visitor.visit_ts_external_module_reference(reference);
            }
        }
    }

    pub fn walk_ts_type_name<'a, V: Visit<'a>>(visitor: &mut V, name: &TSTypeName<'a>) {
        let kind = AstKind::TSTypeName(visitor.alloc(name));
        visitor.enter_node(kind);
        match &name {
            TSTypeName::IdentifierReference(ident) => visitor.visit_identifier_reference(ident),
            TSTypeName::QualifiedName(name) => visitor.visit_ts_qualified_name(name),
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_external_module_reference<'a, V: Visit<'a>>(
        visitor: &mut V,
        reference: &TSExternalModuleReference<'a>,
    ) {
        let kind = AstKind::TSExternalModuleReference(visitor.alloc(reference));
        visitor.enter_node(kind);
        visitor.visit_string_literal(&reference.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_qualified_name<'a, V: Visit<'a>>(visitor: &mut V, name: &TSQualifiedName<'a>) {
        let kind = AstKind::TSQualifiedName(visitor.alloc(name));
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&name.left);
        visitor.visit_identifier_name(&name.right);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_module_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &TSModuleDeclaration<'a>,
    ) {
        let kind = AstKind::TSModuleDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        match &decl.id {
            TSModuleDeclarationName::Identifier(ident) => visitor.visit_identifier_name(ident),
            TSModuleDeclarationName::StringLiteral(lit) => visitor.visit_string_literal(lit),
        }
        match &decl.body {
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

    pub fn walk_ts_module_block<'a, V: Visit<'a>>(visitor: &mut V, block: &TSModuleBlock<'a>) {
        let kind = AstKind::TSModuleBlock(visitor.alloc(block));
        visitor.enter_scope(ScopeFlags::TsModuleBlock);
        visitor.enter_node(kind);
        visitor.visit_statements(&block.body);
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_ts_type_alias_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &TSTypeAliasDeclaration<'a>,
    ) {
        let kind = AstKind::TSTypeAliasDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&decl.id);
        if let Some(parameters) = &decl.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        visitor.visit_ts_type(&decl.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_interface_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        decl: &TSInterfaceDeclaration<'a>,
    ) {
        let kind = AstKind::TSInterfaceDeclaration(visitor.alloc(decl));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&decl.id);
        if let Some(parameters) = &decl.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        for signature in &decl.body.body {
            visitor.visit_ts_signature(signature);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_as_expression<'a, V: Visit<'a>>(visitor: &mut V, expr: &TSAsExpression<'a>) {
        let kind = AstKind::TSAsExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.expression);
        visitor.visit_ts_type(&expr.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_satisfies_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &TSSatisfiesExpression<'a>,
    ) {
        let kind = AstKind::TSSatisfiesExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.expression);
        visitor.visit_ts_type(&expr.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_non_null_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &TSNonNullExpression<'a>,
    ) {
        let kind = AstKind::TSNonNullExpression(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_assertion<'a, V: Visit<'a>>(visitor: &mut V, expr: &TSTypeAssertion<'a>) {
        let kind = AstKind::TSTypeAssertion(visitor.alloc(expr));
        visitor.enter_node(kind);
        visitor.visit_expression(&expr.expression);
        visitor.visit_ts_type(&expr.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_instantiation_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        expr: &TSInstantiationExpression<'a>,
    ) {
        visitor.visit_expression(&expr.expression);
        visitor.visit_ts_type_parameter_instantiation(&expr.type_parameters);
    }

    pub fn walk_ts_type_annotation<'a, V: Visit<'a>>(
        visitor: &mut V,
        annotation: &TSTypeAnnotation<'a>,
    ) {
        let kind = AstKind::TSTypeAnnotation(visitor.alloc(annotation));
        visitor.enter_node(kind);
        visitor.visit_ts_type(&annotation.type_annotation);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSType<'a>) {
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

    pub fn walk_ts_type_literal<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSTypeLiteral<'a>) {
        let kind = AstKind::TSTypeLiteral(visitor.alloc(ty));
        visitor.enter_node(kind);
        for signature in &ty.members {
            visitor.visit_ts_signature(signature);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_indexed_access_type<'a, V: Visit<'a>>(
        visitor: &mut V,
        ty: &TSIndexedAccessType<'a>,
    ) {
        let kind = AstKind::TSIndexedAccessType(visitor.alloc(ty));
        visitor.enter_node(kind);
        visitor.visit_ts_type(&ty.object_type);
        visitor.visit_ts_type(&ty.index_type);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_predicate<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSTypePredicate<'a>) {
        if let Some(annotation) = &ty.type_annotation {
            visitor.visit_ts_type_annotation(annotation);
        }
    }

    pub fn walk_ts_type_operator_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSTypeOperator<'a>) {
        visitor.visit_ts_type(&ty.type_annotation);
    }

    pub fn walk_ts_tuple_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSTupleType<'a>) {
        for element in &ty.element_types {
            visitor.visit_ts_tuple_element(element);
        }
    }

    pub fn walk_ts_tuple_element<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSTupleElement<'a>) {
        match ty {
            TSTupleElement::TSType(ty) => visitor.visit_ts_type(ty),
            TSTupleElement::TSOptionalType(ty) => visitor.visit_ts_type(&ty.type_annotation),
            TSTupleElement::TSRestType(ty) => visitor.visit_ts_type(&ty.type_annotation),
            TSTupleElement::TSNamedTupleMember(ty) => visitor.visit_ts_type(&ty.element_type),
        };
    }

    pub fn walk_ts_mapped_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSMappedType<'a>) {
        visitor.visit_ts_type_parameter(&ty.type_parameter);
        if let Some(name) = &ty.name_type {
            visitor.visit_ts_type(name);
        }
        if let Some(type_annotation) = &ty.type_annotation {
            visitor.visit_ts_type(type_annotation);
        }
    }

    pub fn walk_ts_function_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSFunctionType<'a>) {
        visitor.visit_formal_parameters(&ty.params);
        if let Some(parameters) = &ty.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        visitor.visit_ts_type_annotation(&ty.return_type);
    }

    pub fn walk_ts_type_parameter<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSTypeParameter<'a>) {
        let kind = AstKind::TSTypeParameter(visitor.alloc(ty));
        visitor.enter_scope(ScopeFlags::empty());
        visitor.enter_node(kind);
        if let Some(constraint) = &ty.constraint {
            visitor.visit_ts_type(constraint);
        }

        if let Some(default) = &ty.default {
            visitor.visit_ts_type(default);
        }
        visitor.leave_node(kind);
        visitor.leave_scope();
    }

    pub fn walk_ts_type_parameter_instantiation<'a, V: Visit<'a>>(
        visitor: &mut V,
        ty: &TSTypeParameterInstantiation<'a>,
    ) {
        let kind = AstKind::TSTypeParameterInstantiation(visitor.alloc(ty));
        visitor.enter_node(kind);
        for ts_parameter in &ty.params {
            visitor.visit_ts_type(ts_parameter);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_parameter_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        ty: &TSTypeParameterDeclaration<'a>,
    ) {
        let kind = AstKind::TSTypeParameterDeclaration(visitor.alloc(ty));
        visitor.enter_node(kind);
        for ts_parameter in &ty.params {
            visitor.visit_ts_type_parameter(ts_parameter);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_constructor_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSConstructorType<'a>) {
        visitor.visit_formal_parameters(&ty.params);
        if let Some(parameters) = &ty.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        visitor.visit_ts_type_annotation(&ty.return_type);
    }

    pub fn walk_ts_conditional_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSConditionalType<'a>) {
        visitor.visit_ts_type(&ty.check_type);
        visitor.visit_ts_type(&ty.extends_type);
        visitor.visit_ts_type(&ty.true_type);
        visitor.visit_ts_type(&ty.false_type);
    }

    pub fn walk_ts_array_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSArrayType<'a>) {
        visitor.visit_ts_type(&ty.element_type);
    }

    pub fn walk_ts_null_keyword<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSNullKeyword) {
        let kind = AstKind::TSNullKeyword(visitor.alloc(ty));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_any_keyword<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSAnyKeyword) {
        let kind = AstKind::TSAnyKeyword(visitor.alloc(ty));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_void_keyword<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSVoidKeyword) {
        let kind = AstKind::TSVoidKeyword(visitor.alloc(ty));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_intersection_type<'a, V: Visit<'a>>(
        visitor: &mut V,
        ty: &TSIntersectionType<'a>,
    ) {
        let kind = AstKind::TSIntersectionType(visitor.alloc(ty));
        visitor.enter_node(kind);
        for ty in &ty.types {
            visitor.visit_ts_type(ty);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type_reference<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSTypeReference<'a>) {
        let kind = AstKind::TSTypeReference(visitor.alloc(ty));
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&ty.type_name);
        if let Some(parameters) = &ty.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(parameters);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_union_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSUnionType<'a>) {
        let kind = AstKind::TSUnionType(visitor.alloc(ty));
        visitor.enter_node(kind);
        for ty in &ty.types {
            visitor.visit_ts_type(ty);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_literal_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSLiteralType<'a>) {
        let kind = AstKind::TSLiteralType(visitor.alloc(ty));
        visitor.enter_node(kind);
        match &ty.literal {
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

    pub fn walk_ts_signature<'a, V: Visit<'a>>(visitor: &mut V, signature: &TSSignature<'a>) {
        match &signature {
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

    pub fn walk_ts_construct_signature_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        signature: &TSConstructSignatureDeclaration<'a>,
    ) {
        visitor.visit_formal_parameters(&signature.params);
        if let Some(parameters) = &signature.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &signature.return_type {
            visitor.visit_ts_type_annotation(annotation);
        }
    }

    pub fn walk_ts_method_signature<'a, V: Visit<'a>>(
        visitor: &mut V,
        signature: &TSMethodSignature<'a>,
    ) {
        let kind = AstKind::TSMethodSignature(visitor.alloc(signature));
        visitor.enter_node(kind);
        visitor.visit_formal_parameters(&signature.params);
        if let Some(parameters) = &signature.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &signature.return_type {
            visitor.visit_ts_type_annotation(annotation);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_index_signature_name<'a, V: Visit<'a>>(
        visitor: &mut V,
        name: &TSIndexSignatureName<'a>,
    ) {
        visitor.visit_ts_type_annotation(&name.type_annotation);
    }

    pub fn walk_ts_index_signature<'a, V: Visit<'a>>(
        visitor: &mut V,
        signature: &TSIndexSignature<'a>,
    ) {
        for name in &signature.parameters {
            visitor.visit_ts_index_signature_name(name);
        }

        visitor.visit_ts_type_annotation(&signature.type_annotation);
    }

    pub fn walk_ts_property_signature<'a, V: Visit<'a>>(
        visitor: &mut V,
        signature: &TSPropertySignature<'a>,
    ) {
        let kind = AstKind::TSPropertySignature(visitor.alloc(signature));
        visitor.enter_node(kind);
        visitor.visit_property_key(&signature.key);
        if let Some(annotation) = &signature.type_annotation {
            visitor.visit_ts_type_annotation(annotation);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_call_signature_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        signature: &TSCallSignatureDeclaration<'a>,
    ) {
        visitor.visit_formal_parameters(&signature.params);
        if let Some(parameters) = &signature.type_parameters {
            visitor.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(annotation) = &signature.return_type {
            visitor.visit_ts_type_annotation(annotation);
        }
    }

    pub fn walk_ts_type_query<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSTypeQuery<'a>) {
        let kind = AstKind::TSTypeQuery(visitor.alloc(ty));
        visitor.enter_node(kind);
        match &ty.expr_name {
            TSTypeQueryExprName::TSTypeName(name) => visitor.visit_ts_type_name(name),
            TSTypeQueryExprName::TSImportType(import) => visitor.visit_ts_import_type(import),
        }
        if let Some(type_parameters) = &ty.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_import_type<'a, V: Visit<'a>>(visitor: &mut V, ty: &TSImportType<'a>) {
        let kind = AstKind::TSImportType(visitor.alloc(ty));
        visitor.enter_node(kind);
        visitor.visit_ts_type(&ty.argument);
        if let Some(name) = &ty.qualifier {
            visitor.visit_ts_type_name(name);
        }
        if let Some(attrs) = &ty.attributes {
            visitor.visit_ts_import_attributes(attrs);
        }
        if let Some(type_parameter) = &ty.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameter);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_ts_import_attributes<'a, V: Visit<'a>>(
        visitor: &mut V,
        attributes: &TSImportAttributes<'a>,
    ) {
        for element in &attributes.elements {
            visitor.visit_ts_import_attribute(element);
        }
    }

    pub fn walk_ts_import_attribute<'a, V: Visit<'a>>(
        visitor: &mut V,
        attribute: &TSImportAttribute<'a>,
    ) {
        visitor.visit_ts_import_attribute_name(&attribute.name);
        visitor.visit_expression(&attribute.value);
    }

    pub fn walk_ts_import_attribute_name<'a, V: Visit<'a>>(
        visitor: &mut V,
        name: &TSImportAttributeName<'a>,
    ) {
        match name {
            TSImportAttributeName::Identifier(ident) => visitor.visit_identifier_name(ident),
            TSImportAttributeName::StringLiteral(ident) => visitor.visit_string_literal(ident),
        }
    }
}
