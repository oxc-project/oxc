// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/visit.rs`

//! Visitor Pattern
//!
//! See:
//! * [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)
//! * [rustc visitor](https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs)

#![allow(
    unused_variables,
    clippy::extra_unused_type_parameters,
    clippy::explicit_iter_loop,
    clippy::self_named_module_files,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants
)]

use std::cell::Cell;

use oxc_allocator::Vec;
use oxc_syntax::scope::{ScopeFlags, ScopeId};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use crate::ast_kind::AstKind;

use walk::*;

/// Syntax tree traversal
pub trait Visit<'a>: Sized {
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
    fn visit_hashbang(&mut self, it: &Hashbang<'a>) {
        walk_hashbang(self, it);
    }

    #[inline]
    fn visit_directives(&mut self, it: &Vec<'a, Directive<'a>>) {
        walk_directives(self, it);
    }

    #[inline]
    fn visit_directive(&mut self, it: &Directive<'a>) {
        walk_directive(self, it);
    }

    #[inline]
    fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
        walk_string_literal(self, it);
    }

    #[inline]
    fn visit_statements(&mut self, it: &Vec<'a, Statement<'a>>) {
        walk_statements(self, it);
    }

    #[inline]
    fn visit_statement(&mut self, it: &Statement<'a>) {
        walk_statement(self, it);
    }

    #[inline]
    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        walk_block_statement(self, it);
    }

    #[inline]
    fn visit_break_statement(&mut self, it: &BreakStatement<'a>) {
        walk_break_statement(self, it);
    }

    #[inline]
    fn visit_label_identifier(&mut self, it: &LabelIdentifier<'a>) {
        walk_label_identifier(self, it);
    }

    #[inline]
    fn visit_continue_statement(&mut self, it: &ContinueStatement<'a>) {
        walk_continue_statement(self, it);
    }

    #[inline]
    fn visit_debugger_statement(&mut self, it: &DebuggerStatement) {
        walk_debugger_statement(self, it);
    }

    #[inline]
    fn visit_do_while_statement(&mut self, it: &DoWhileStatement<'a>) {
        walk_do_while_statement(self, it);
    }

    #[inline]
    fn visit_expression(&mut self, it: &Expression<'a>) {
        walk_expression(self, it);
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
    fn visit_big_int_literal(&mut self, it: &BigIntLiteral<'a>) {
        walk_big_int_literal(self, it);
    }

    #[inline]
    fn visit_reg_exp_literal(&mut self, it: &RegExpLiteral<'a>) {
        walk_reg_exp_literal(self, it);
    }

    #[inline]
    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        walk_template_literal(self, it);
    }

    #[inline]
    fn visit_template_elements(&mut self, it: &Vec<'a, TemplateElement<'a>>) {
        walk_template_elements(self, it);
    }

    #[inline]
    fn visit_template_element(&mut self, it: &TemplateElement<'a>) {
        walk_template_element(self, it);
    }

    #[inline]
    fn visit_expressions(&mut self, it: &Vec<'a, Expression<'a>>) {
        walk_expressions(self, it);
    }

    #[inline]
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        walk_identifier_reference(self, it);
    }

    #[inline]
    fn visit_meta_property(&mut self, it: &MetaProperty<'a>) {
        walk_meta_property(self, it);
    }

    #[inline]
    fn visit_identifier_name(&mut self, it: &IdentifierName<'a>) {
        walk_identifier_name(self, it);
    }

    #[inline]
    fn visit_super(&mut self, it: &Super) {
        walk_super(self, it);
    }

    #[inline]
    fn visit_array_expression(&mut self, it: &ArrayExpression<'a>) {
        walk_array_expression(self, it);
    }

    #[inline]
    fn visit_array_expression_elements(&mut self, it: &Vec<'a, ArrayExpressionElement<'a>>) {
        walk_array_expression_elements(self, it);
    }

    #[inline]
    fn visit_array_expression_element(&mut self, it: &ArrayExpressionElement<'a>) {
        walk_array_expression_element(self, it);
    }

    #[inline]
    fn visit_spread_element(&mut self, it: &SpreadElement<'a>) {
        walk_spread_element(self, it);
    }

    #[inline]
    fn visit_elision(&mut self, it: &Elision) {
        walk_elision(self, it);
    }

    #[inline]
    fn visit_expression_array_element(&mut self, it: &Expression<'a>) {
        walk_expression_array_element(self, it);
    }

    #[inline]
    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        walk_arrow_function_expression(self, it);
    }

    #[inline]
    fn visit_ts_type_parameter_declaration(&mut self, it: &TSTypeParameterDeclaration<'a>) {
        walk_ts_type_parameter_declaration(self, it);
    }

    #[inline]
    fn visit_ts_type_parameters(&mut self, it: &Vec<'a, TSTypeParameter<'a>>) {
        walk_ts_type_parameters(self, it);
    }

    #[inline]
    fn visit_ts_type_parameter(&mut self, it: &TSTypeParameter<'a>) {
        walk_ts_type_parameter(self, it);
    }

    #[inline]
    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        walk_binding_identifier(self, it);
    }

    #[inline]
    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        walk_ts_type(self, it);
    }

    #[inline]
    fn visit_ts_any_keyword(&mut self, it: &TSAnyKeyword) {
        walk_ts_any_keyword(self, it);
    }

    #[inline]
    fn visit_ts_big_int_keyword(&mut self, it: &TSBigIntKeyword) {
        walk_ts_big_int_keyword(self, it);
    }

    #[inline]
    fn visit_ts_boolean_keyword(&mut self, it: &TSBooleanKeyword) {
        walk_ts_boolean_keyword(self, it);
    }

    #[inline]
    fn visit_ts_intrinsic_keyword(&mut self, it: &TSIntrinsicKeyword) {
        walk_ts_intrinsic_keyword(self, it);
    }

    #[inline]
    fn visit_ts_never_keyword(&mut self, it: &TSNeverKeyword) {
        walk_ts_never_keyword(self, it);
    }

    #[inline]
    fn visit_ts_null_keyword(&mut self, it: &TSNullKeyword) {
        walk_ts_null_keyword(self, it);
    }

    #[inline]
    fn visit_ts_number_keyword(&mut self, it: &TSNumberKeyword) {
        walk_ts_number_keyword(self, it);
    }

    #[inline]
    fn visit_ts_object_keyword(&mut self, it: &TSObjectKeyword) {
        walk_ts_object_keyword(self, it);
    }

    #[inline]
    fn visit_ts_string_keyword(&mut self, it: &TSStringKeyword) {
        walk_ts_string_keyword(self, it);
    }

    #[inline]
    fn visit_ts_symbol_keyword(&mut self, it: &TSSymbolKeyword) {
        walk_ts_symbol_keyword(self, it);
    }

    #[inline]
    fn visit_ts_undefined_keyword(&mut self, it: &TSUndefinedKeyword) {
        walk_ts_undefined_keyword(self, it);
    }

    #[inline]
    fn visit_ts_unknown_keyword(&mut self, it: &TSUnknownKeyword) {
        walk_ts_unknown_keyword(self, it);
    }

    #[inline]
    fn visit_ts_void_keyword(&mut self, it: &TSVoidKeyword) {
        walk_ts_void_keyword(self, it);
    }

    #[inline]
    fn visit_ts_array_type(&mut self, it: &TSArrayType<'a>) {
        walk_ts_array_type(self, it);
    }

    #[inline]
    fn visit_ts_conditional_type(&mut self, it: &TSConditionalType<'a>) {
        walk_ts_conditional_type(self, it);
    }

    #[inline]
    fn visit_ts_constructor_type(&mut self, it: &TSConstructorType<'a>) {
        walk_ts_constructor_type(self, it);
    }

    #[inline]
    fn visit_formal_parameters(&mut self, it: &FormalParameters<'a>) {
        walk_formal_parameters(self, it);
    }

    #[inline]
    fn visit_formal_parameter_list(&mut self, it: &Vec<'a, FormalParameter<'a>>) {
        walk_formal_parameter_list(self, it);
    }

    #[inline]
    fn visit_formal_parameter(&mut self, it: &FormalParameter<'a>) {
        walk_formal_parameter(self, it);
    }

    #[inline]
    fn visit_decorators(&mut self, it: &Vec<'a, Decorator<'a>>) {
        walk_decorators(self, it);
    }

    #[inline]
    fn visit_decorator(&mut self, it: &Decorator<'a>) {
        walk_decorator(self, it);
    }

    #[inline]
    fn visit_binding_pattern(&mut self, it: &BindingPattern<'a>) {
        walk_binding_pattern(self, it);
    }

    #[inline]
    fn visit_binding_pattern_kind(&mut self, it: &BindingPatternKind<'a>) {
        walk_binding_pattern_kind(self, it);
    }

    #[inline]
    fn visit_object_pattern(&mut self, it: &ObjectPattern<'a>) {
        walk_object_pattern(self, it);
    }

    #[inline]
    fn visit_binding_properties(&mut self, it: &Vec<'a, BindingProperty<'a>>) {
        walk_binding_properties(self, it);
    }

    #[inline]
    fn visit_binding_property(&mut self, it: &BindingProperty<'a>) {
        walk_binding_property(self, it);
    }

    #[inline]
    fn visit_property_key(&mut self, it: &PropertyKey<'a>) {
        walk_property_key(self, it);
    }

    #[inline]
    fn visit_private_identifier(&mut self, it: &PrivateIdentifier<'a>) {
        walk_private_identifier(self, it);
    }

    #[inline]
    fn visit_binding_rest_element(&mut self, it: &BindingRestElement<'a>) {
        walk_binding_rest_element(self, it);
    }

    #[inline]
    fn visit_array_pattern(&mut self, it: &ArrayPattern<'a>) {
        walk_array_pattern(self, it);
    }

    #[inline]
    fn visit_assignment_pattern(&mut self, it: &AssignmentPattern<'a>) {
        walk_assignment_pattern(self, it);
    }

    #[inline]
    fn visit_ts_type_annotation(&mut self, it: &TSTypeAnnotation<'a>) {
        walk_ts_type_annotation(self, it);
    }

    #[inline]
    fn visit_ts_function_type(&mut self, it: &TSFunctionType<'a>) {
        walk_ts_function_type(self, it);
    }

    #[inline]
    fn visit_ts_this_parameter(&mut self, it: &TSThisParameter<'a>) {
        walk_ts_this_parameter(self, it);
    }

    #[inline]
    fn visit_ts_import_type(&mut self, it: &TSImportType<'a>) {
        walk_ts_import_type(self, it);
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
    fn visit_ts_import_attributes(&mut self, it: &TSImportAttributes<'a>) {
        walk_ts_import_attributes(self, it);
    }

    #[inline]
    fn visit_ts_import_attribute_list(&mut self, it: &Vec<'a, TSImportAttribute<'a>>) {
        walk_ts_import_attribute_list(self, it);
    }

    #[inline]
    fn visit_ts_import_attribute(&mut self, it: &TSImportAttribute<'a>) {
        walk_ts_import_attribute(self, it);
    }

    #[inline]
    fn visit_ts_import_attribute_name(&mut self, it: &TSImportAttributeName<'a>) {
        walk_ts_import_attribute_name(self, it);
    }

    #[inline]
    fn visit_ts_type_parameter_instantiation(&mut self, it: &TSTypeParameterInstantiation<'a>) {
        walk_ts_type_parameter_instantiation(self, it);
    }

    #[inline]
    fn visit_ts_types(&mut self, it: &Vec<'a, TSType<'a>>) {
        walk_ts_types(self, it);
    }

    #[inline]
    fn visit_ts_indexed_access_type(&mut self, it: &TSIndexedAccessType<'a>) {
        walk_ts_indexed_access_type(self, it);
    }

    #[inline]
    fn visit_ts_infer_type(&mut self, it: &TSInferType<'a>) {
        walk_ts_infer_type(self, it);
    }

    #[inline]
    fn visit_ts_intersection_type(&mut self, it: &TSIntersectionType<'a>) {
        walk_ts_intersection_type(self, it);
    }

    #[inline]
    fn visit_ts_literal_type(&mut self, it: &TSLiteralType<'a>) {
        walk_ts_literal_type(self, it);
    }

    #[inline]
    fn visit_ts_literal(&mut self, it: &TSLiteral<'a>) {
        walk_ts_literal(self, it);
    }

    #[inline]
    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        walk_unary_expression(self, it);
    }

    #[inline]
    fn visit_ts_mapped_type(&mut self, it: &TSMappedType<'a>) {
        walk_ts_mapped_type(self, it);
    }

    #[inline]
    fn visit_ts_named_tuple_member(&mut self, it: &TSNamedTupleMember<'a>) {
        walk_ts_named_tuple_member(self, it);
    }

    #[inline]
    fn visit_ts_tuple_element(&mut self, it: &TSTupleElement<'a>) {
        walk_ts_tuple_element(self, it);
    }

    #[inline]
    fn visit_ts_optional_type(&mut self, it: &TSOptionalType<'a>) {
        walk_ts_optional_type(self, it);
    }

    #[inline]
    fn visit_ts_rest_type(&mut self, it: &TSRestType<'a>) {
        walk_ts_rest_type(self, it);
    }

    #[inline]
    fn visit_ts_template_literal_type(&mut self, it: &TSTemplateLiteralType<'a>) {
        walk_ts_template_literal_type(self, it);
    }

    #[inline]
    fn visit_ts_this_type(&mut self, it: &TSThisType) {
        walk_ts_this_type(self, it);
    }

    #[inline]
    fn visit_ts_tuple_type(&mut self, it: &TSTupleType<'a>) {
        walk_ts_tuple_type(self, it);
    }

    #[inline]
    fn visit_ts_tuple_elements(&mut self, it: &Vec<'a, TSTupleElement<'a>>) {
        walk_ts_tuple_elements(self, it);
    }

    #[inline]
    fn visit_ts_type_literal(&mut self, it: &TSTypeLiteral<'a>) {
        walk_ts_type_literal(self, it);
    }

    #[inline]
    fn visit_ts_signatures(&mut self, it: &Vec<'a, TSSignature<'a>>) {
        walk_ts_signatures(self, it);
    }

    #[inline]
    fn visit_ts_signature(&mut self, it: &TSSignature<'a>) {
        walk_ts_signature(self, it);
    }

    #[inline]
    fn visit_ts_index_signature(&mut self, it: &TSIndexSignature<'a>) {
        walk_ts_index_signature(self, it);
    }

    #[inline]
    fn visit_ts_index_signature_names(&mut self, it: &Vec<'a, TSIndexSignatureName<'a>>) {
        walk_ts_index_signature_names(self, it);
    }

    #[inline]
    fn visit_ts_index_signature_name(&mut self, it: &TSIndexSignatureName<'a>) {
        walk_ts_index_signature_name(self, it);
    }

    #[inline]
    fn visit_ts_property_signature(&mut self, it: &TSPropertySignature<'a>) {
        walk_ts_property_signature(self, it);
    }

    #[inline]
    fn visit_ts_call_signature_declaration(&mut self, it: &TSCallSignatureDeclaration<'a>) {
        walk_ts_call_signature_declaration(self, it);
    }

    #[inline]
    fn visit_ts_construct_signature_declaration(
        &mut self,
        it: &TSConstructSignatureDeclaration<'a>,
    ) {
        walk_ts_construct_signature_declaration(self, it);
    }

    #[inline]
    fn visit_ts_method_signature(&mut self, it: &TSMethodSignature<'a>) {
        walk_ts_method_signature(self, it);
    }

    #[inline]
    fn visit_ts_type_operator(&mut self, it: &TSTypeOperator<'a>) {
        walk_ts_type_operator(self, it);
    }

    #[inline]
    fn visit_ts_type_predicate(&mut self, it: &TSTypePredicate<'a>) {
        walk_ts_type_predicate(self, it);
    }

    #[inline]
    fn visit_ts_type_predicate_name(&mut self, it: &TSTypePredicateName<'a>) {
        walk_ts_type_predicate_name(self, it);
    }

    #[inline]
    fn visit_ts_type_query(&mut self, it: &TSTypeQuery<'a>) {
        walk_ts_type_query(self, it);
    }

    #[inline]
    fn visit_ts_type_query_expr_name(&mut self, it: &TSTypeQueryExprName<'a>) {
        walk_ts_type_query_expr_name(self, it);
    }

    #[inline]
    fn visit_ts_type_reference(&mut self, it: &TSTypeReference<'a>) {
        walk_ts_type_reference(self, it);
    }

    #[inline]
    fn visit_ts_union_type(&mut self, it: &TSUnionType<'a>) {
        walk_ts_union_type(self, it);
    }

    #[inline]
    fn visit_ts_parenthesized_type(&mut self, it: &TSParenthesizedType<'a>) {
        walk_ts_parenthesized_type(self, it);
    }

    #[inline]
    fn visit_js_doc_nullable_type(&mut self, it: &JSDocNullableType<'a>) {
        walk_js_doc_nullable_type(self, it);
    }

    #[inline]
    fn visit_js_doc_non_nullable_type(&mut self, it: &JSDocNonNullableType<'a>) {
        walk_js_doc_non_nullable_type(self, it);
    }

    #[inline]
    fn visit_js_doc_unknown_type(&mut self, it: &JSDocUnknownType) {
        walk_js_doc_unknown_type(self, it);
    }

    #[inline]
    fn visit_function_body(&mut self, it: &FunctionBody<'a>) {
        walk_function_body(self, it);
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
    fn visit_ts_as_expression(&mut self, it: &TSAsExpression<'a>) {
        walk_ts_as_expression(self, it);
    }

    #[inline]
    fn visit_ts_satisfies_expression(&mut self, it: &TSSatisfiesExpression<'a>) {
        walk_ts_satisfies_expression(self, it);
    }

    #[inline]
    fn visit_ts_non_null_expression(&mut self, it: &TSNonNullExpression<'a>) {
        walk_ts_non_null_expression(self, it);
    }

    #[inline]
    fn visit_ts_type_assertion(&mut self, it: &TSTypeAssertion<'a>) {
        walk_ts_type_assertion(self, it);
    }

    #[inline]
    fn visit_ts_instantiation_expression(&mut self, it: &TSInstantiationExpression<'a>) {
        walk_ts_instantiation_expression(self, it);
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
    fn visit_assignment_target_pattern(&mut self, it: &AssignmentTargetPattern<'a>) {
        walk_assignment_target_pattern(self, it);
    }

    #[inline]
    fn visit_array_assignment_target(&mut self, it: &ArrayAssignmentTarget<'a>) {
        walk_array_assignment_target(self, it);
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
    fn visit_assignment_target_rest(&mut self, it: &AssignmentTargetRest<'a>) {
        walk_assignment_target_rest(self, it);
    }

    #[inline]
    fn visit_object_assignment_target(&mut self, it: &ObjectAssignmentTarget<'a>) {
        walk_object_assignment_target(self, it);
    }

    #[inline]
    fn visit_assignment_target_properties(&mut self, it: &Vec<'a, AssignmentTargetProperty<'a>>) {
        walk_assignment_target_properties(self, it);
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
    fn visit_await_expression(&mut self, it: &AwaitExpression<'a>) {
        walk_await_expression(self, it);
    }

    #[inline]
    fn visit_binary_expression(&mut self, it: &BinaryExpression<'a>) {
        walk_binary_expression(self, it);
    }

    #[inline]
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        walk_call_expression(self, it);
    }

    #[inline]
    fn visit_arguments(&mut self, it: &Vec<'a, Argument<'a>>) {
        walk_arguments(self, it);
    }

    #[inline]
    fn visit_argument(&mut self, it: &Argument<'a>) {
        walk_argument(self, it);
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
    fn visit_class(&mut self, it: &Class<'a>) {
        walk_class(self, it);
    }

    #[inline]
    fn visit_class_heritage(&mut self, it: &Expression<'a>) {
        walk_class_heritage(self, it);
    }

    #[inline]
    fn visit_ts_class_implementses(&mut self, it: &Vec<'a, TSClassImplements<'a>>) {
        walk_ts_class_implementses(self, it);
    }

    #[inline]
    fn visit_ts_class_implements(&mut self, it: &TSClassImplements<'a>) {
        walk_ts_class_implements(self, it);
    }

    #[inline]
    fn visit_class_body(&mut self, it: &ClassBody<'a>) {
        walk_class_body(self, it);
    }

    #[inline]
    fn visit_class_elements(&mut self, it: &Vec<'a, ClassElement<'a>>) {
        walk_class_elements(self, it);
    }

    #[inline]
    fn visit_class_element(&mut self, it: &ClassElement<'a>) {
        walk_class_element(self, it);
    }

    #[inline]
    fn visit_static_block(&mut self, it: &StaticBlock<'a>) {
        walk_static_block(self, it);
    }

    #[inline]
    fn visit_method_definition(&mut self, it: &MethodDefinition<'a>) {
        walk_method_definition(self, it);
    }

    #[inline]
    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        walk_function(self, it, flags);
    }

    #[inline]
    fn visit_property_definition(&mut self, it: &PropertyDefinition<'a>) {
        walk_property_definition(self, it);
    }

    #[inline]
    fn visit_accessor_property(&mut self, it: &AccessorProperty<'a>) {
        walk_accessor_property(self, it);
    }

    #[inline]
    fn visit_conditional_expression(&mut self, it: &ConditionalExpression<'a>) {
        walk_conditional_expression(self, it);
    }

    #[inline]
    fn visit_import_expression(&mut self, it: &ImportExpression<'a>) {
        walk_import_expression(self, it);
    }

    #[inline]
    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        walk_logical_expression(self, it);
    }

    #[inline]
    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        walk_new_expression(self, it);
    }

    #[inline]
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        walk_object_expression(self, it);
    }

    #[inline]
    fn visit_object_property_kinds(&mut self, it: &Vec<'a, ObjectPropertyKind<'a>>) {
        walk_object_property_kinds(self, it);
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
    fn visit_parenthesized_expression(&mut self, it: &ParenthesizedExpression<'a>) {
        walk_parenthesized_expression(self, it);
    }

    #[inline]
    fn visit_sequence_expression(&mut self, it: &SequenceExpression<'a>) {
        walk_sequence_expression(self, it);
    }

    #[inline]
    fn visit_tagged_template_expression(&mut self, it: &TaggedTemplateExpression<'a>) {
        walk_tagged_template_expression(self, it);
    }

    #[inline]
    fn visit_this_expression(&mut self, it: &ThisExpression) {
        walk_this_expression(self, it);
    }

    #[inline]
    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        walk_update_expression(self, it);
    }

    #[inline]
    fn visit_yield_expression(&mut self, it: &YieldExpression<'a>) {
        walk_yield_expression(self, it);
    }

    #[inline]
    fn visit_private_in_expression(&mut self, it: &PrivateInExpression<'a>) {
        walk_private_in_expression(self, it);
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
    fn visit_jsx_element_name(&mut self, it: &JSXElementName<'a>) {
        walk_jsx_element_name(self, it);
    }

    #[inline]
    fn visit_jsx_identifier(&mut self, it: &JSXIdentifier<'a>) {
        walk_jsx_identifier(self, it);
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
    fn visit_jsx_attribute_items(&mut self, it: &Vec<'a, JSXAttributeItem<'a>>) {
        walk_jsx_attribute_items(self, it);
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
    fn visit_jsx_attribute_name(&mut self, it: &JSXAttributeName<'a>) {
        walk_jsx_attribute_name(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_value(&mut self, it: &JSXAttributeValue<'a>) {
        walk_jsx_attribute_value(self, it);
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
    fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
        walk_jsx_fragment(self, it);
    }

    #[inline]
    fn visit_jsx_children(&mut self, it: &Vec<'a, JSXChild<'a>>) {
        walk_jsx_children(self, it);
    }

    #[inline]
    fn visit_jsx_child(&mut self, it: &JSXChild<'a>) {
        walk_jsx_child(self, it);
    }

    #[inline]
    fn visit_jsx_text(&mut self, it: &JSXText<'a>) {
        walk_jsx_text(self, it);
    }

    #[inline]
    fn visit_jsx_spread_child(&mut self, it: &JSXSpreadChild<'a>) {
        walk_jsx_spread_child(self, it);
    }

    #[inline]
    fn visit_jsx_spread_attribute(&mut self, it: &JSXSpreadAttribute<'a>) {
        walk_jsx_spread_attribute(self, it);
    }

    #[inline]
    fn visit_jsx_closing_element(&mut self, it: &JSXClosingElement<'a>) {
        walk_jsx_closing_element(self, it);
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
    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        walk_for_in_statement(self, it);
    }

    #[inline]
    fn visit_for_statement_left(&mut self, it: &ForStatementLeft<'a>) {
        walk_for_statement_left(self, it);
    }

    #[inline]
    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        walk_variable_declaration(self, it);
    }

    #[inline]
    fn visit_variable_declarators(&mut self, it: &Vec<'a, VariableDeclarator<'a>>) {
        walk_variable_declarators(self, it);
    }

    #[inline]
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        walk_variable_declarator(self, it);
    }

    #[inline]
    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        walk_for_of_statement(self, it);
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
    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        walk_if_statement(self, it);
    }

    #[inline]
    fn visit_labeled_statement(&mut self, it: &LabeledStatement<'a>) {
        walk_labeled_statement(self, it);
    }

    #[inline]
    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        walk_return_statement(self, it);
    }

    #[inline]
    fn visit_switch_statement(&mut self, it: &SwitchStatement<'a>) {
        walk_switch_statement(self, it);
    }

    #[inline]
    fn visit_switch_cases(&mut self, it: &Vec<'a, SwitchCase<'a>>) {
        walk_switch_cases(self, it);
    }

    #[inline]
    fn visit_switch_case(&mut self, it: &SwitchCase<'a>) {
        walk_switch_case(self, it);
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
    fn visit_finally_clause(&mut self, it: &BlockStatement<'a>) {
        walk_finally_clause(self, it);
    }

    #[inline]
    fn visit_while_statement(&mut self, it: &WhileStatement<'a>) {
        walk_while_statement(self, it);
    }

    #[inline]
    fn visit_with_statement(&mut self, it: &WithStatement<'a>) {
        walk_with_statement(self, it);
    }

    #[inline]
    fn visit_declaration(&mut self, it: &Declaration<'a>) {
        walk_declaration(self, it);
    }

    #[inline]
    fn visit_ts_type_alias_declaration(&mut self, it: &TSTypeAliasDeclaration<'a>) {
        walk_ts_type_alias_declaration(self, it);
    }

    #[inline]
    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        walk_ts_interface_declaration(self, it);
    }

    #[inline]
    fn visit_ts_interface_heritages(&mut self, it: &Vec<'a, TSInterfaceHeritage<'a>>) {
        walk_ts_interface_heritages(self, it);
    }

    #[inline]
    fn visit_ts_interface_heritage(&mut self, it: &TSInterfaceHeritage<'a>) {
        walk_ts_interface_heritage(self, it);
    }

    #[inline]
    fn visit_ts_interface_body(&mut self, it: &TSInterfaceBody<'a>) {
        walk_ts_interface_body(self, it);
    }

    #[inline]
    fn visit_ts_enum_declaration(&mut self, it: &TSEnumDeclaration<'a>) {
        walk_ts_enum_declaration(self, it);
    }

    #[inline]
    fn visit_ts_enum_members(&mut self, it: &Vec<'a, TSEnumMember<'a>>) {
        walk_ts_enum_members(self, it);
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
    fn visit_ts_module_block(&mut self, it: &TSModuleBlock<'a>) {
        walk_ts_module_block(self, it);
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
    fn visit_module_declaration(&mut self, it: &ModuleDeclaration<'a>) {
        walk_module_declaration(self, it);
    }

    #[inline]
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        walk_import_declaration(self, it);
    }

    #[inline]
    fn visit_import_declaration_specifiers(
        &mut self,
        it: &Vec<'a, ImportDeclarationSpecifier<'a>>,
    ) {
        walk_import_declaration_specifiers(self, it);
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
    fn visit_module_export_name(&mut self, it: &ModuleExportName<'a>) {
        walk_module_export_name(self, it);
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
    fn visit_import_attributes(&mut self, it: &Vec<'a, ImportAttribute<'a>>) {
        walk_import_attributes(self, it);
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
    fn visit_export_all_declaration(&mut self, it: &ExportAllDeclaration<'a>) {
        walk_export_all_declaration(self, it);
    }

    #[inline]
    fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
        walk_export_default_declaration(self, it);
    }

    #[inline]
    fn visit_export_default_declaration_kind(&mut self, it: &ExportDefaultDeclarationKind<'a>) {
        walk_export_default_declaration_kind(self, it);
    }

    #[inline]
    fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
        walk_export_named_declaration(self, it);
    }

    #[inline]
    fn visit_export_specifiers(&mut self, it: &Vec<'a, ExportSpecifier<'a>>) {
        walk_export_specifiers(self, it);
    }

    #[inline]
    fn visit_export_specifier(&mut self, it: &ExportSpecifier<'a>) {
        walk_export_specifier(self, it);
    }

    #[inline]
    fn visit_ts_export_assignment(&mut self, it: &TSExportAssignment<'a>) {
        walk_ts_export_assignment(self, it);
    }

    #[inline]
    fn visit_ts_namespace_export_declaration(&mut self, it: &TSNamespaceExportDeclaration<'a>) {
        walk_ts_namespace_export_declaration(self, it);
    }
}

pub mod walk {
    use super::*;

    #[inline]
    pub fn walk_program<'a, V: Visit<'a>>(visitor: &mut V, it: &Program<'a>) {
        let kind = AstKind::Program(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(
            {
                let mut flags = ScopeFlags::Top;
                if it.source_type.is_strict() || it.directives.iter().any(Directive::is_use_strict)
                {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &it.scope_id,
        );
        if let Some(hashbang) = &it.hashbang {
            visitor.visit_hashbang(hashbang);
        }
        visitor.visit_directives(&it.directives);
        visitor.visit_statements(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_hashbang<'a, V: Visit<'a>>(visitor: &mut V, it: &Hashbang<'a>) {
        let kind = AstKind::Hashbang(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_directives<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, Directive<'a>>) {
        for el in it {
            visitor.visit_directive(el);
        }
    }

    #[inline]
    pub fn walk_directive<'a, V: Visit<'a>>(visitor: &mut V, it: &Directive<'a>) {
        let kind = AstKind::Directive(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_string_literal(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_string_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &StringLiteral<'a>) {
        let kind = AstKind::StringLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_statements<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, Statement<'a>>) {
        for el in it {
            visitor.visit_statement(el);
        }
    }

    pub fn walk_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &Statement<'a>) {
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
    pub fn walk_block_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &BlockStatement<'a>) {
        let kind = AstKind::BlockStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_statements(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_break_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &BreakStatement<'a>) {
        let kind = AstKind::BreakStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        if let Some(label) = &it.label {
            visitor.visit_label_identifier(label);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_label_identifier<'a, V: Visit<'a>>(visitor: &mut V, it: &LabelIdentifier<'a>) {
        let kind = AstKind::LabelIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_continue_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &ContinueStatement<'a>) {
        let kind = AstKind::ContinueStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        if let Some(label) = &it.label {
            visitor.visit_label_identifier(label);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_debugger_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &DebuggerStatement) {
        let kind = AstKind::DebuggerStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_do_while_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_statement(&it.body);
        visitor.visit_expression(&it.test);
        visitor.leave_node(kind);
    }

    pub fn walk_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &Expression<'a>) {
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
            match_member_expression!(Expression) => {
                visitor.visit_member_expression(it.to_member_expression())
            }
        }
    }

    #[inline]
    pub fn walk_boolean_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &BooleanLiteral) {
        let kind = AstKind::BooleanLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_null_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &NullLiteral) {
        let kind = AstKind::NullLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_numeric_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &NumericLiteral<'a>) {
        let kind = AstKind::NumericLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_big_int_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &BigIntLiteral<'a>) {
        let kind = AstKind::BigIntLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_reg_exp_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &RegExpLiteral<'a>) {
        let kind = AstKind::RegExpLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_template_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &TemplateLiteral<'a>) {
        let kind = AstKind::TemplateLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_template_elements(&it.quasis);
        visitor.visit_expressions(&it.expressions);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_template_elements<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, TemplateElement<'a>>,
    ) {
        for el in it {
            visitor.visit_template_element(el);
        }
    }

    #[inline]
    pub fn walk_template_element<'a, V: Visit<'a>>(visitor: &mut V, it: &TemplateElement<'a>) {
        // NOTE: AstKind doesn't exists!
    }

    #[inline]
    pub fn walk_expressions<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, Expression<'a>>) {
        for el in it {
            visitor.visit_expression(el);
        }
    }

    #[inline]
    pub fn walk_identifier_reference<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &IdentifierReference<'a>,
    ) {
        let kind = AstKind::IdentifierReference(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_meta_property<'a, V: Visit<'a>>(visitor: &mut V, it: &MetaProperty<'a>) {
        let kind = AstKind::MetaProperty(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_identifier_name(&it.meta);
        visitor.visit_identifier_name(&it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_identifier_name<'a, V: Visit<'a>>(visitor: &mut V, it: &IdentifierName<'a>) {
        let kind = AstKind::IdentifierName(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_super<'a, V: Visit<'a>>(visitor: &mut V, it: &Super) {
        let kind = AstKind::Super(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &ArrayExpression<'a>) {
        let kind = AstKind::ArrayExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_array_expression_elements(&it.elements);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_expression_elements<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, ArrayExpressionElement<'a>>,
    ) {
        for el in it {
            visitor.visit_array_expression_element(el);
        }
    }

    #[inline]
    pub fn walk_array_expression_element<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ArrayExpressionElement<'a>,
    ) {
        let kind = AstKind::ArrayExpressionElement(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            ArrayExpressionElement::SpreadElement(it) => visitor.visit_spread_element(it),
            ArrayExpressionElement::Elision(it) => visitor.visit_elision(it),
            match_expression!(ArrayExpressionElement) => {
                visitor.visit_expression_array_element(it.to_expression())
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_spread_element<'a, V: Visit<'a>>(visitor: &mut V, it: &SpreadElement<'a>) {
        let kind = AstKind::SpreadElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_elision<'a, V: Visit<'a>>(visitor: &mut V, it: &Elision) {
        let kind = AstKind::Elision(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_expression_array_element<'a, V: Visit<'a>>(visitor: &mut V, it: &Expression<'a>) {
        let kind = AstKind::ExpressionArrayElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(it);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_arrow_function_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ArrowFunctionExpression<'a>,
    ) {
        let kind = AstKind::ArrowFunctionExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(
            {
                let mut flags = ScopeFlags::Function | ScopeFlags::Arrow;
                if it.body.has_use_strict_directive() {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &it.scope_id,
        );
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.visit_formal_parameters(&it.params);
        if let Some(return_type) = &it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        visitor.visit_function_body(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_parameter_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSTypeParameterDeclaration<'a>,
    ) {
        let kind = AstKind::TSTypeParameterDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type_parameters(&it.params);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_parameters<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, TSTypeParameter<'a>>,
    ) {
        for el in it {
            visitor.visit_ts_type_parameter(el);
        }
    }

    #[inline]
    pub fn walk_ts_type_parameter<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypeParameter<'a>) {
        let kind = AstKind::TSTypeParameter(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&it.name);
        if let Some(constraint) = &it.constraint {
            visitor.visit_ts_type(constraint);
        }
        if let Some(default) = &it.default {
            visitor.visit_ts_type(default);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_identifier<'a, V: Visit<'a>>(visitor: &mut V, it: &BindingIdentifier<'a>) {
        let kind = AstKind::BindingIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSType<'a>) {
        match it {
            TSType::TSAnyKeyword(it) => visitor.visit_ts_any_keyword(it),
            TSType::TSBigIntKeyword(it) => visitor.visit_ts_big_int_keyword(it),
            TSType::TSBooleanKeyword(it) => visitor.visit_ts_boolean_keyword(it),
            TSType::TSIntrinsicKeyword(it) => visitor.visit_ts_intrinsic_keyword(it),
            TSType::TSNeverKeyword(it) => visitor.visit_ts_never_keyword(it),
            TSType::TSNullKeyword(it) => visitor.visit_ts_null_keyword(it),
            TSType::TSNumberKeyword(it) => visitor.visit_ts_number_keyword(it),
            TSType::TSObjectKeyword(it) => visitor.visit_ts_object_keyword(it),
            TSType::TSStringKeyword(it) => visitor.visit_ts_string_keyword(it),
            TSType::TSSymbolKeyword(it) => visitor.visit_ts_symbol_keyword(it),
            TSType::TSUndefinedKeyword(it) => visitor.visit_ts_undefined_keyword(it),
            TSType::TSUnknownKeyword(it) => visitor.visit_ts_unknown_keyword(it),
            TSType::TSVoidKeyword(it) => visitor.visit_ts_void_keyword(it),
            TSType::TSArrayType(it) => visitor.visit_ts_array_type(it),
            TSType::TSConditionalType(it) => visitor.visit_ts_conditional_type(it),
            TSType::TSConstructorType(it) => visitor.visit_ts_constructor_type(it),
            TSType::TSFunctionType(it) => visitor.visit_ts_function_type(it),
            TSType::TSImportType(it) => visitor.visit_ts_import_type(it),
            TSType::TSIndexedAccessType(it) => visitor.visit_ts_indexed_access_type(it),
            TSType::TSInferType(it) => visitor.visit_ts_infer_type(it),
            TSType::TSIntersectionType(it) => visitor.visit_ts_intersection_type(it),
            TSType::TSLiteralType(it) => visitor.visit_ts_literal_type(it),
            TSType::TSMappedType(it) => visitor.visit_ts_mapped_type(it),
            TSType::TSNamedTupleMember(it) => visitor.visit_ts_named_tuple_member(it),
            TSType::TSQualifiedName(it) => visitor.visit_ts_qualified_name(it),
            TSType::TSTemplateLiteralType(it) => visitor.visit_ts_template_literal_type(it),
            TSType::TSThisType(it) => visitor.visit_ts_this_type(it),
            TSType::TSTupleType(it) => visitor.visit_ts_tuple_type(it),
            TSType::TSTypeLiteral(it) => visitor.visit_ts_type_literal(it),
            TSType::TSTypeOperatorType(it) => visitor.visit_ts_type_operator(it),
            TSType::TSTypePredicate(it) => visitor.visit_ts_type_predicate(it),
            TSType::TSTypeQuery(it) => visitor.visit_ts_type_query(it),
            TSType::TSTypeReference(it) => visitor.visit_ts_type_reference(it),
            TSType::TSUnionType(it) => visitor.visit_ts_union_type(it),
            TSType::TSParenthesizedType(it) => visitor.visit_ts_parenthesized_type(it),
            TSType::JSDocNullableType(it) => visitor.visit_js_doc_nullable_type(it),
            TSType::JSDocNonNullableType(it) => visitor.visit_js_doc_non_nullable_type(it),
            TSType::JSDocUnknownType(it) => visitor.visit_js_doc_unknown_type(it),
        }
    }

    #[inline]
    pub fn walk_ts_any_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSAnyKeyword) {
        let kind = AstKind::TSAnyKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_big_int_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSBigIntKeyword) {
        let kind = AstKind::TSBigIntKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_boolean_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSBooleanKeyword) {
        let kind = AstKind::TSBooleanKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_intrinsic_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSIntrinsicKeyword) {
        let kind = AstKind::TSIntrinsicKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_never_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSNeverKeyword) {
        let kind = AstKind::TSNeverKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_null_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSNullKeyword) {
        let kind = AstKind::TSNullKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_number_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSNumberKeyword) {
        let kind = AstKind::TSNumberKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_object_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSObjectKeyword) {
        let kind = AstKind::TSObjectKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_string_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSStringKeyword) {
        let kind = AstKind::TSStringKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_symbol_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSSymbolKeyword) {
        let kind = AstKind::TSSymbolKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_undefined_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSUndefinedKeyword) {
        let kind = AstKind::TSUndefinedKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_unknown_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSUnknownKeyword) {
        let kind = AstKind::TSUnknownKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_void_keyword<'a, V: Visit<'a>>(visitor: &mut V, it: &TSVoidKeyword) {
        let kind = AstKind::TSVoidKeyword(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_array_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSArrayType<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_type(&it.element_type);
    }

    #[inline]
    pub fn walk_ts_conditional_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSConditionalType<'a>) {
        let kind = AstKind::TSConditionalType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type(&it.check_type);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_ts_type(&it.extends_type);
        visitor.visit_ts_type(&it.true_type);
        visitor.leave_scope();
        visitor.visit_ts_type(&it.false_type);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_constructor_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSConstructorType<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_formal_parameters(&it.params);
        visitor.visit_ts_type_annotation(&it.return_type);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
    }

    #[inline]
    pub fn walk_formal_parameters<'a, V: Visit<'a>>(visitor: &mut V, it: &FormalParameters<'a>) {
        let kind = AstKind::FormalParameters(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_formal_parameter_list(&it.items);
        if let Some(rest) = &it.rest {
            visitor.visit_binding_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_formal_parameter_list<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, FormalParameter<'a>>,
    ) {
        for el in it {
            visitor.visit_formal_parameter(el);
        }
    }

    #[inline]
    pub fn walk_formal_parameter<'a, V: Visit<'a>>(visitor: &mut V, it: &FormalParameter<'a>) {
        let kind = AstKind::FormalParameter(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_decorators(&it.decorators);
        visitor.visit_binding_pattern(&it.pattern);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_decorators<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, Decorator<'a>>) {
        for el in it {
            visitor.visit_decorator(el);
        }
    }

    #[inline]
    pub fn walk_decorator<'a, V: Visit<'a>>(visitor: &mut V, it: &Decorator<'a>) {
        let kind = AstKind::Decorator(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_pattern<'a, V: Visit<'a>>(visitor: &mut V, it: &BindingPattern<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_binding_pattern_kind(&it.kind);
        if let Some(type_annotation) = &it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    pub fn walk_binding_pattern_kind<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &BindingPatternKind<'a>,
    ) {
        match it {
            BindingPatternKind::BindingIdentifier(it) => visitor.visit_binding_identifier(it),
            BindingPatternKind::ObjectPattern(it) => visitor.visit_object_pattern(it),
            BindingPatternKind::ArrayPattern(it) => visitor.visit_array_pattern(it),
            BindingPatternKind::AssignmentPattern(it) => visitor.visit_assignment_pattern(it),
        }
    }

    #[inline]
    pub fn walk_object_pattern<'a, V: Visit<'a>>(visitor: &mut V, it: &ObjectPattern<'a>) {
        let kind = AstKind::ObjectPattern(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_properties(&it.properties);
        if let Some(rest) = &it.rest {
            visitor.visit_binding_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_properties<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, BindingProperty<'a>>,
    ) {
        for el in it {
            visitor.visit_binding_property(el);
        }
    }

    #[inline]
    pub fn walk_binding_property<'a, V: Visit<'a>>(visitor: &mut V, it: &BindingProperty<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_property_key(&it.key);
        visitor.visit_binding_pattern(&it.value);
    }

    #[inline]
    pub fn walk_property_key<'a, V: Visit<'a>>(visitor: &mut V, it: &PropertyKey<'a>) {
        let kind = AstKind::PropertyKey(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            PropertyKey::StaticIdentifier(it) => visitor.visit_identifier_name(it),
            PropertyKey::PrivateIdentifier(it) => visitor.visit_private_identifier(it),
            match_expression!(PropertyKey) => visitor.visit_expression(it.to_expression()),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_private_identifier<'a, V: Visit<'a>>(visitor: &mut V, it: &PrivateIdentifier<'a>) {
        let kind = AstKind::PrivateIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_rest_element<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &BindingRestElement<'a>,
    ) {
        let kind = AstKind::BindingRestElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_pattern<'a, V: Visit<'a>>(visitor: &mut V, it: &ArrayPattern<'a>) {
        let kind = AstKind::ArrayPattern(visitor.alloc(it));
        visitor.enter_node(kind);
        for elements in it.elements.iter().flatten() {
            visitor.visit_binding_pattern(elements);
        }
        if let Some(rest) = &it.rest {
            visitor.visit_binding_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_pattern<'a, V: Visit<'a>>(visitor: &mut V, it: &AssignmentPattern<'a>) {
        let kind = AstKind::AssignmentPattern(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_annotation<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypeAnnotation<'a>) {
        let kind = AstKind::TSTypeAnnotation(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type(&it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_function_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSFunctionType<'a>) {
        // NOTE: AstKind doesn't exists!
        if let Some(this_param) = &it.this_param {
            visitor.visit_ts_this_parameter(this_param);
        }
        visitor.visit_formal_parameters(&it.params);
        visitor.visit_ts_type_annotation(&it.return_type);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
    }

    #[inline]
    pub fn walk_ts_this_parameter<'a, V: Visit<'a>>(visitor: &mut V, it: &TSThisParameter<'a>) {
        let kind = AstKind::TSThisParameter(visitor.alloc(it));
        visitor.enter_node(kind);
        if let Some(type_annotation) = &it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_import_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSImportType<'a>) {
        let kind = AstKind::TSImportType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type(&it.parameter);
        if let Some(qualifier) = &it.qualifier {
            visitor.visit_ts_type_name(qualifier);
        }
        if let Some(attributes) = &it.attributes {
            visitor.visit_ts_import_attributes(attributes);
        }
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_name<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypeName<'a>) {
        let kind = AstKind::TSTypeName(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            TSTypeName::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            TSTypeName::QualifiedName(it) => visitor.visit_ts_qualified_name(it),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_qualified_name<'a, V: Visit<'a>>(visitor: &mut V, it: &TSQualifiedName<'a>) {
        let kind = AstKind::TSQualifiedName(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&it.left);
        visitor.visit_identifier_name(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_import_attributes<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSImportAttributes<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_identifier_name(&it.attributes_keyword);
        visitor.visit_ts_import_attribute_list(&it.elements);
    }

    #[inline]
    pub fn walk_ts_import_attribute_list<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, TSImportAttribute<'a>>,
    ) {
        for el in it {
            visitor.visit_ts_import_attribute(el);
        }
    }

    #[inline]
    pub fn walk_ts_import_attribute<'a, V: Visit<'a>>(visitor: &mut V, it: &TSImportAttribute<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_import_attribute_name(&it.name);
        visitor.visit_expression(&it.value);
    }

    #[inline]
    pub fn walk_ts_import_attribute_name<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSImportAttributeName<'a>,
    ) {
        match it {
            TSImportAttributeName::Identifier(it) => visitor.visit_identifier_name(it),
            TSImportAttributeName::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_ts_type_parameter_instantiation<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSTypeParameterInstantiation<'a>,
    ) {
        let kind = AstKind::TSTypeParameterInstantiation(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_types(&it.params);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_types<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, TSType<'a>>) {
        for el in it {
            visitor.visit_ts_type(el);
        }
    }

    #[inline]
    pub fn walk_ts_indexed_access_type<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSIndexedAccessType<'a>,
    ) {
        let kind = AstKind::TSIndexedAccessType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type(&it.object_type);
        visitor.visit_ts_type(&it.index_type);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_infer_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSInferType<'a>) {
        let kind = AstKind::TSInferType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type_parameter(&it.type_parameter);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_intersection_type<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSIntersectionType<'a>,
    ) {
        let kind = AstKind::TSIntersectionType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_types(&it.types);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_literal_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSLiteralType<'a>) {
        let kind = AstKind::TSLiteralType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_literal(&it.literal);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &TSLiteral<'a>) {
        match it {
            TSLiteral::BooleanLiteral(it) => visitor.visit_boolean_literal(it),
            TSLiteral::NullLiteral(it) => visitor.visit_null_literal(it),
            TSLiteral::NumericLiteral(it) => visitor.visit_numeric_literal(it),
            TSLiteral::BigIntLiteral(it) => visitor.visit_big_int_literal(it),
            TSLiteral::RegExpLiteral(it) => visitor.visit_reg_exp_literal(it),
            TSLiteral::StringLiteral(it) => visitor.visit_string_literal(it),
            TSLiteral::TemplateLiteral(it) => visitor.visit_template_literal(it),
            TSLiteral::UnaryExpression(it) => visitor.visit_unary_expression(it),
        }
    }

    #[inline]
    pub fn walk_unary_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &UnaryExpression<'a>) {
        let kind = AstKind::UnaryExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_mapped_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSMappedType<'a>) {
        let kind = AstKind::TSMappedType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_ts_type_parameter(&it.type_parameter);
        if let Some(name_type) = &it.name_type {
            visitor.visit_ts_type(name_type);
        }
        if let Some(type_annotation) = &it.type_annotation {
            visitor.visit_ts_type(type_annotation);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_named_tuple_member<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSNamedTupleMember<'a>,
    ) {
        let kind = AstKind::TSNamedTupleMember(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_tuple_element(&it.element_type);
        visitor.visit_identifier_name(&it.label);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_tuple_element<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTupleElement<'a>) {
        match it {
            TSTupleElement::TSOptionalType(it) => visitor.visit_ts_optional_type(it),
            TSTupleElement::TSRestType(it) => visitor.visit_ts_rest_type(it),
            match_ts_type!(TSTupleElement) => visitor.visit_ts_type(it.to_ts_type()),
        }
    }

    #[inline]
    pub fn walk_ts_optional_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSOptionalType<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_rest_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSRestType<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_template_literal_type<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSTemplateLiteralType<'a>,
    ) {
        let kind = AstKind::TSTemplateLiteralType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_template_elements(&it.quasis);
        visitor.visit_ts_types(&it.types);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_this_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSThisType) {
        let kind = AstKind::TSThisType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_tuple_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTupleType<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_tuple_elements(&it.element_types);
    }

    #[inline]
    pub fn walk_ts_tuple_elements<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, TSTupleElement<'a>>,
    ) {
        for el in it {
            visitor.visit_ts_tuple_element(el);
        }
    }

    #[inline]
    pub fn walk_ts_type_literal<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypeLiteral<'a>) {
        let kind = AstKind::TSTypeLiteral(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_signatures(&it.members);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_signatures<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, TSSignature<'a>>) {
        for el in it {
            visitor.visit_ts_signature(el);
        }
    }

    #[inline]
    pub fn walk_ts_signature<'a, V: Visit<'a>>(visitor: &mut V, it: &TSSignature<'a>) {
        match it {
            TSSignature::TSIndexSignature(it) => visitor.visit_ts_index_signature(it),
            TSSignature::TSPropertySignature(it) => visitor.visit_ts_property_signature(it),
            TSSignature::TSCallSignatureDeclaration(it) => {
                visitor.visit_ts_call_signature_declaration(it)
            }
            TSSignature::TSConstructSignatureDeclaration(it) => {
                visitor.visit_ts_construct_signature_declaration(it)
            }
            TSSignature::TSMethodSignature(it) => visitor.visit_ts_method_signature(it),
        }
    }

    #[inline]
    pub fn walk_ts_index_signature<'a, V: Visit<'a>>(visitor: &mut V, it: &TSIndexSignature<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_index_signature_names(&it.parameters);
        visitor.visit_ts_type_annotation(&it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_index_signature_names<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, TSIndexSignatureName<'a>>,
    ) {
        for el in it {
            visitor.visit_ts_index_signature_name(el);
        }
    }

    #[inline]
    pub fn walk_ts_index_signature_name<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSIndexSignatureName<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_type_annotation(&it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_property_signature<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSPropertySignature<'a>,
    ) {
        let kind = AstKind::TSPropertySignature(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_property_key(&it.key);
        if let Some(type_annotation) = &it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_call_signature_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSCallSignatureDeclaration<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        if let Some(this_param) = &it.this_param {
            visitor.visit_ts_this_parameter(this_param);
        }
        visitor.visit_formal_parameters(&it.params);
        if let Some(return_type) = &it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
    }

    #[inline]
    pub fn walk_ts_construct_signature_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSConstructSignatureDeclaration<'a>,
    ) {
        let kind = AstKind::TSConstructSignatureDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_formal_parameters(&it.params);
        if let Some(return_type) = &it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_method_signature<'a, V: Visit<'a>>(visitor: &mut V, it: &TSMethodSignature<'a>) {
        let kind = AstKind::TSMethodSignature(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_property_key(&it.key);
        if let Some(this_param) = &it.this_param {
            visitor.visit_ts_this_parameter(this_param);
        }
        visitor.visit_formal_parameters(&it.params);
        if let Some(return_type) = &it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_operator<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypeOperator<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_type_predicate<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypePredicate<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_type_predicate_name(&it.parameter_name);
        if let Some(type_annotation) = &it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    pub fn walk_ts_type_predicate_name<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSTypePredicateName<'a>,
    ) {
        match it {
            TSTypePredicateName::Identifier(it) => visitor.visit_identifier_name(it),
            TSTypePredicateName::This(it) => visitor.visit_ts_this_type(it),
        }
    }

    #[inline]
    pub fn walk_ts_type_query<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypeQuery<'a>) {
        let kind = AstKind::TSTypeQuery(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type_query_expr_name(&it.expr_name);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_query_expr_name<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSTypeQueryExprName<'a>,
    ) {
        match it {
            TSTypeQueryExprName::TSImportType(it) => visitor.visit_ts_import_type(it),
            match_ts_type_name!(TSTypeQueryExprName) => {
                visitor.visit_ts_type_name(it.to_ts_type_name())
            }
        }
    }

    #[inline]
    pub fn walk_ts_type_reference<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypeReference<'a>) {
        let kind = AstKind::TSTypeReference(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&it.type_name);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_union_type<'a, V: Visit<'a>>(visitor: &mut V, it: &TSUnionType<'a>) {
        let kind = AstKind::TSUnionType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_types(&it.types);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_parenthesized_type<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSParenthesizedType<'a>,
    ) {
        let kind = AstKind::TSParenthesizedType(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type(&it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_js_doc_nullable_type<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &JSDocNullableType<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    pub fn walk_js_doc_non_nullable_type<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &JSDocNonNullableType<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    pub fn walk_js_doc_unknown_type<'a, V: Visit<'a>>(visitor: &mut V, it: &JSDocUnknownType) {
        // NOTE: AstKind doesn't exists!
    }

    #[inline]
    pub fn walk_function_body<'a, V: Visit<'a>>(visitor: &mut V, it: &FunctionBody<'a>) {
        let kind = AstKind::FunctionBody(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_directives(&it.directives);
        visitor.visit_statements(&it.statements);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &AssignmentExpression<'a>,
    ) {
        let kind = AstKind::AssignmentExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_assignment_target(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target<'a, V: Visit<'a>>(visitor: &mut V, it: &AssignmentTarget<'a>) {
        let kind = AstKind::AssignmentTarget(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            match_simple_assignment_target!(AssignmentTarget) => {
                visitor.visit_simple_assignment_target(it.to_simple_assignment_target())
            }
            match_assignment_target_pattern!(AssignmentTarget) => {
                visitor.visit_assignment_target_pattern(it.to_assignment_target_pattern())
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_simple_assignment_target<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &SimpleAssignmentTarget<'a>,
    ) {
        let kind = AstKind::SimpleAssignmentTarget(visitor.alloc(it));
        visitor.enter_node(kind);
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
            SimpleAssignmentTarget::TSInstantiationExpression(it) => {
                visitor.visit_ts_instantiation_expression(it)
            }
            match_member_expression!(SimpleAssignmentTarget) => {
                visitor.visit_member_expression(it.to_member_expression())
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_as_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &TSAsExpression<'a>) {
        let kind = AstKind::TSAsExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.visit_ts_type(&it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_satisfies_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSSatisfiesExpression<'a>,
    ) {
        let kind = AstKind::TSSatisfiesExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.visit_ts_type(&it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_non_null_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSNonNullExpression<'a>,
    ) {
        let kind = AstKind::TSNonNullExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_assertion<'a, V: Visit<'a>>(visitor: &mut V, it: &TSTypeAssertion<'a>) {
        let kind = AstKind::TSTypeAssertion(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.visit_ts_type(&it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_instantiation_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSInstantiationExpression<'a>,
    ) {
        let kind = AstKind::TSInstantiationExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.visit_ts_type_parameter_instantiation(&it.type_parameters);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_member_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &MemberExpression<'a>) {
        let kind = AstKind::MemberExpression(visitor.alloc(it));
        visitor.enter_node(kind);
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
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_computed_member_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ComputedMemberExpression<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_expression(&it.object);
        visitor.visit_expression(&it.expression);
    }

    #[inline]
    pub fn walk_static_member_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &StaticMemberExpression<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_expression(&it.object);
        visitor.visit_identifier_name(&it.property);
    }

    #[inline]
    pub fn walk_private_field_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &PrivateFieldExpression<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_expression(&it.object);
        visitor.visit_private_identifier(&it.field);
    }

    #[inline]
    pub fn walk_assignment_target_pattern<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetPattern<'a>,
    ) {
        let kind = AstKind::AssignmentTargetPattern(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            AssignmentTargetPattern::ArrayAssignmentTarget(it) => {
                visitor.visit_array_assignment_target(it)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(it) => {
                visitor.visit_object_assignment_target(it)
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_assignment_target<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ArrayAssignmentTarget<'a>,
    ) {
        let kind = AstKind::ArrayAssignmentTarget(visitor.alloc(it));
        visitor.enter_node(kind);
        for elements in it.elements.iter().flatten() {
            visitor.visit_assignment_target_maybe_default(elements);
        }
        if let Some(rest) = &it.rest {
            visitor.visit_assignment_target_rest(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_maybe_default<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetMaybeDefault<'a>,
    ) {
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
    pub fn walk_assignment_target_with_default<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetWithDefault<'a>,
    ) {
        let kind = AstKind::AssignmentTargetWithDefault(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_assignment_target(&it.binding);
        visitor.visit_expression(&it.init);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_rest<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetRest<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_assignment_target(&it.target);
    }

    #[inline]
    pub fn walk_object_assignment_target<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ObjectAssignmentTarget<'a>,
    ) {
        let kind = AstKind::ObjectAssignmentTarget(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_assignment_target_properties(&it.properties);
        if let Some(rest) = &it.rest {
            visitor.visit_assignment_target_rest(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_properties<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, AssignmentTargetProperty<'a>>,
    ) {
        for el in it {
            visitor.visit_assignment_target_property(el);
        }
    }

    #[inline]
    pub fn walk_assignment_target_property<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetProperty<'a>,
    ) {
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
    pub fn walk_assignment_target_property_identifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_identifier_reference(&it.binding);
        if let Some(init) = &it.init {
            visitor.visit_expression(init);
        }
    }

    #[inline]
    pub fn walk_assignment_target_property_property<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &AssignmentTargetPropertyProperty<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_property_key(&it.name);
        visitor.visit_assignment_target_maybe_default(&it.binding);
    }

    #[inline]
    pub fn walk_await_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &AwaitExpression<'a>) {
        let kind = AstKind::AwaitExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binary_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &BinaryExpression<'a>) {
        let kind = AstKind::BinaryExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_call_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &CallExpression<'a>) {
        let kind = AstKind::CallExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.callee);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.visit_arguments(&it.arguments);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_arguments<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, Argument<'a>>) {
        for el in it {
            visitor.visit_argument(el);
        }
    }

    #[inline]
    pub fn walk_argument<'a, V: Visit<'a>>(visitor: &mut V, it: &Argument<'a>) {
        let kind = AstKind::Argument(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            Argument::SpreadElement(it) => visitor.visit_spread_element(it),
            match_expression!(Argument) => visitor.visit_expression(it.to_expression()),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_chain_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &ChainExpression<'a>) {
        let kind = AstKind::ChainExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_chain_element(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_chain_element<'a, V: Visit<'a>>(visitor: &mut V, it: &ChainElement<'a>) {
        match it {
            ChainElement::CallExpression(it) => visitor.visit_call_expression(it),
            match_member_expression!(ChainElement) => {
                visitor.visit_member_expression(it.to_member_expression())
            }
        }
    }

    pub fn walk_class<'a, V: Visit<'a>>(visitor: &mut V, it: &Class<'a>) {
        let kind = AstKind::Class(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_decorators(&it.decorators);
        if let Some(id) = &it.id {
            visitor.visit_binding_identifier(id);
        }
        visitor.enter_scope(ScopeFlags::StrictMode, &it.scope_id);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(super_class) = &it.super_class {
            visitor.visit_class_heritage(super_class);
        }
        if let Some(super_type_parameters) = &it.super_type_parameters {
            visitor.visit_ts_type_parameter_instantiation(super_type_parameters);
        }
        if let Some(implements) = &it.implements {
            visitor.visit_ts_class_implementses(implements);
        }
        visitor.visit_class_body(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    pub fn walk_class_heritage<'a, V: Visit<'a>>(visitor: &mut V, it: &Expression<'a>) {
        let kind = AstKind::ClassHeritage(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(it);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_class_implementses<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, TSClassImplements<'a>>,
    ) {
        for el in it {
            visitor.visit_ts_class_implements(el);
        }
    }

    #[inline]
    pub fn walk_ts_class_implements<'a, V: Visit<'a>>(visitor: &mut V, it: &TSClassImplements<'a>) {
        let kind = AstKind::TSClassImplements(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&it.expression);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_class_body<'a, V: Visit<'a>>(visitor: &mut V, it: &ClassBody<'a>) {
        let kind = AstKind::ClassBody(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_class_elements(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_class_elements<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, ClassElement<'a>>) {
        for el in it {
            visitor.visit_class_element(el);
        }
    }

    #[inline]
    pub fn walk_class_element<'a, V: Visit<'a>>(visitor: &mut V, it: &ClassElement<'a>) {
        match it {
            ClassElement::StaticBlock(it) => visitor.visit_static_block(it),
            ClassElement::MethodDefinition(it) => visitor.visit_method_definition(it),
            ClassElement::PropertyDefinition(it) => visitor.visit_property_definition(it),
            ClassElement::AccessorProperty(it) => visitor.visit_accessor_property(it),
            ClassElement::TSIndexSignature(it) => visitor.visit_ts_index_signature(it),
        }
    }

    #[inline]
    pub fn walk_static_block<'a, V: Visit<'a>>(visitor: &mut V, it: &StaticBlock<'a>) {
        let kind = AstKind::StaticBlock(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::ClassStaticBlock, &it.scope_id);
        visitor.visit_statements(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_method_definition<'a, V: Visit<'a>>(visitor: &mut V, it: &MethodDefinition<'a>) {
        let kind = AstKind::MethodDefinition(visitor.alloc(it));
        visitor.enter_node(kind);
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

    pub fn walk_function<'a, V: Visit<'a>>(visitor: &mut V, it: &Function<'a>, flags: ScopeFlags) {
        let kind = AstKind::Function(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(
            {
                let mut flags = flags;
                if it.is_strict() {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &it.scope_id,
        );
        if let Some(id) = &it.id {
            visitor.visit_binding_identifier(id);
        }
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(this_param) = &it.this_param {
            visitor.visit_ts_this_parameter(this_param);
        }
        visitor.visit_formal_parameters(&it.params);
        if let Some(return_type) = &it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        if let Some(body) = &it.body {
            visitor.visit_function_body(body);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_property_definition<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &PropertyDefinition<'a>,
    ) {
        let kind = AstKind::PropertyDefinition(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_decorators(&it.decorators);
        visitor.visit_property_key(&it.key);
        if let Some(value) = &it.value {
            visitor.visit_expression(value);
        }
        if let Some(type_annotation) = &it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_accessor_property<'a, V: Visit<'a>>(visitor: &mut V, it: &AccessorProperty<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_decorators(&it.decorators);
        visitor.visit_property_key(&it.key);
        if let Some(value) = &it.value {
            visitor.visit_expression(value);
        }
        if let Some(type_annotation) = &it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    pub fn walk_conditional_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ConditionalExpression<'a>,
    ) {
        let kind = AstKind::ConditionalExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.test);
        visitor.visit_expression(&it.consequent);
        visitor.visit_expression(&it.alternate);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &ImportExpression<'a>) {
        let kind = AstKind::ImportExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.source);
        visitor.visit_expressions(&it.arguments);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_logical_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &LogicalExpression<'a>) {
        let kind = AstKind::LogicalExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_new_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &NewExpression<'a>) {
        let kind = AstKind::NewExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.callee);
        visitor.visit_arguments(&it.arguments);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_object_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &ObjectExpression<'a>) {
        let kind = AstKind::ObjectExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_object_property_kinds(&it.properties);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_object_property_kinds<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, ObjectPropertyKind<'a>>,
    ) {
        for el in it {
            visitor.visit_object_property_kind(el);
        }
    }

    #[inline]
    pub fn walk_object_property_kind<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ObjectPropertyKind<'a>,
    ) {
        match it {
            ObjectPropertyKind::ObjectProperty(it) => visitor.visit_object_property(it),
            ObjectPropertyKind::SpreadProperty(it) => visitor.visit_spread_element(it),
        }
    }

    #[inline]
    pub fn walk_object_property<'a, V: Visit<'a>>(visitor: &mut V, it: &ObjectProperty<'a>) {
        let kind = AstKind::ObjectProperty(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_property_key(&it.key);
        visitor.visit_expression(&it.value);
        if let Some(init) = &it.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_parenthesized_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ParenthesizedExpression<'a>,
    ) {
        let kind = AstKind::ParenthesizedExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_sequence_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &SequenceExpression<'a>,
    ) {
        let kind = AstKind::SequenceExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expressions(&it.expressions);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_tagged_template_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TaggedTemplateExpression<'a>,
    ) {
        let kind = AstKind::TaggedTemplateExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.tag);
        visitor.visit_template_literal(&it.quasi);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_this_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &ThisExpression) {
        let kind = AstKind::ThisExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_update_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &UpdateExpression<'a>) {
        let kind = AstKind::UpdateExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_simple_assignment_target(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_yield_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &YieldExpression<'a>) {
        let kind = AstKind::YieldExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        if let Some(argument) = &it.argument {
            visitor.visit_expression(argument);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_private_in_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &PrivateInExpression<'a>,
    ) {
        let kind = AstKind::PrivateInExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_private_identifier(&it.left);
        visitor.visit_expression(&it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_element<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXElement<'a>) {
        let kind = AstKind::JSXElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_jsx_opening_element(&it.opening_element);
        if let Some(closing_element) = &it.closing_element {
            visitor.visit_jsx_closing_element(closing_element);
        }
        visitor.visit_jsx_children(&it.children);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_opening_element<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXOpeningElement<'a>) {
        let kind = AstKind::JSXOpeningElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_jsx_element_name(&it.name);
        visitor.visit_jsx_attribute_items(&it.attributes);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_element_name<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXElementName<'a>) {
        let kind = AstKind::JSXElementName(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            JSXElementName::Identifier(it) => visitor.visit_jsx_identifier(it),
            JSXElementName::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            JSXElementName::NamespacedName(it) => visitor.visit_jsx_namespaced_name(it),
            JSXElementName::MemberExpression(it) => visitor.visit_jsx_member_expression(it),
            JSXElementName::ThisExpression(it) => visitor.visit_this_expression(it),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_identifier<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXIdentifier<'a>) {
        let kind = AstKind::JSXIdentifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_namespaced_name<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXNamespacedName<'a>) {
        let kind = AstKind::JSXNamespacedName(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_jsx_identifier(&it.namespace);
        visitor.visit_jsx_identifier(&it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_member_expression<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &JSXMemberExpression<'a>,
    ) {
        let kind = AstKind::JSXMemberExpression(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_jsx_member_expression_object(&it.object);
        visitor.visit_jsx_identifier(&it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_member_expression_object<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &JSXMemberExpressionObject<'a>,
    ) {
        let kind = AstKind::JSXMemberExpressionObject(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            JSXMemberExpressionObject::IdentifierReference(it) => {
                visitor.visit_identifier_reference(it)
            }
            JSXMemberExpressionObject::MemberExpression(it) => {
                visitor.visit_jsx_member_expression(it)
            }
            JSXMemberExpressionObject::ThisExpression(it) => visitor.visit_this_expression(it),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_attribute_items<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, JSXAttributeItem<'a>>,
    ) {
        for el in it {
            visitor.visit_jsx_attribute_item(el);
        }
    }

    #[inline]
    pub fn walk_jsx_attribute_item<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXAttributeItem<'a>) {
        let kind = AstKind::JSXAttributeItem(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            JSXAttributeItem::Attribute(it) => visitor.visit_jsx_attribute(it),
            JSXAttributeItem::SpreadAttribute(it) => visitor.visit_jsx_spread_attribute(it),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_attribute<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXAttribute<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_jsx_attribute_name(&it.name);
        if let Some(value) = &it.value {
            visitor.visit_jsx_attribute_value(value);
        }
    }

    #[inline]
    pub fn walk_jsx_attribute_name<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXAttributeName<'a>) {
        match it {
            JSXAttributeName::Identifier(it) => visitor.visit_jsx_identifier(it),
            JSXAttributeName::NamespacedName(it) => visitor.visit_jsx_namespaced_name(it),
        }
    }

    #[inline]
    pub fn walk_jsx_attribute_value<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXAttributeValue<'a>) {
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
    pub fn walk_jsx_expression_container<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &JSXExpressionContainer<'a>,
    ) {
        let kind = AstKind::JSXExpressionContainer(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_jsx_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXExpression<'a>) {
        match it {
            JSXExpression::EmptyExpression(it) => visitor.visit_jsx_empty_expression(it),
            match_expression!(JSXExpression) => visitor.visit_expression(it.to_expression()),
        }
    }

    #[inline]
    pub fn walk_jsx_empty_expression<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXEmptyExpression) {
        // NOTE: AstKind doesn't exists!
    }

    #[inline]
    pub fn walk_jsx_fragment<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXFragment<'a>) {
        let kind = AstKind::JSXFragment(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_jsx_children(&it.children);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_children<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, JSXChild<'a>>) {
        for el in it {
            visitor.visit_jsx_child(el);
        }
    }

    #[inline]
    pub fn walk_jsx_child<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXChild<'a>) {
        match it {
            JSXChild::Text(it) => visitor.visit_jsx_text(it),
            JSXChild::Element(it) => visitor.visit_jsx_element(it),
            JSXChild::Fragment(it) => visitor.visit_jsx_fragment(it),
            JSXChild::ExpressionContainer(it) => visitor.visit_jsx_expression_container(it),
            JSXChild::Spread(it) => visitor.visit_jsx_spread_child(it),
        }
    }

    #[inline]
    pub fn walk_jsx_text<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXText<'a>) {
        let kind = AstKind::JSXText(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_spread_child<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXSpreadChild<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_expression(&it.expression);
    }

    #[inline]
    pub fn walk_jsx_spread_attribute<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &JSXSpreadAttribute<'a>,
    ) {
        let kind = AstKind::JSXSpreadAttribute(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_closing_element<'a, V: Visit<'a>>(visitor: &mut V, it: &JSXClosingElement<'a>) {
        let kind = AstKind::JSXClosingElement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_jsx_element_name(&it.name);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_empty_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &EmptyStatement) {
        let kind = AstKind::EmptyStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_expression_statement<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ExpressionStatement<'a>,
    ) {
        let kind = AstKind::ExpressionStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_in_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_for_statement_left(&it.left);
        visitor.visit_expression(&it.right);
        visitor.visit_statement(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_statement_left<'a, V: Visit<'a>>(visitor: &mut V, it: &ForStatementLeft<'a>) {
        match it {
            ForStatementLeft::VariableDeclaration(it) => visitor.visit_variable_declaration(it),
            match_assignment_target!(ForStatementLeft) => {
                visitor.visit_assignment_target(it.to_assignment_target())
            }
        }
    }

    #[inline]
    pub fn walk_variable_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &VariableDeclaration<'a>,
    ) {
        let kind = AstKind::VariableDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_variable_declarators(&it.declarations);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_variable_declarators<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, VariableDeclarator<'a>>,
    ) {
        for el in it {
            visitor.visit_variable_declarator(el);
        }
    }

    #[inline]
    pub fn walk_variable_declarator<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &VariableDeclarator<'a>,
    ) {
        let kind = AstKind::VariableDeclarator(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&it.id);
        if let Some(init) = &it.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_of_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_for_statement_left(&it.left);
        visitor.visit_expression(&it.right);
        visitor.visit_statement(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &ForStatement<'a>) {
        let kind = AstKind::ForStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
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
    pub fn walk_for_statement_init<'a, V: Visit<'a>>(visitor: &mut V, it: &ForStatementInit<'a>) {
        let kind = AstKind::ForStatementInit(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            ForStatementInit::VariableDeclaration(it) => visitor.visit_variable_declaration(it),
            match_expression!(ForStatementInit) => visitor.visit_expression(it.to_expression()),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_if_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &IfStatement<'a>) {
        let kind = AstKind::IfStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.test);
        visitor.visit_statement(&it.consequent);
        if let Some(alternate) = &it.alternate {
            visitor.visit_statement(alternate);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_labeled_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &LabeledStatement<'a>) {
        let kind = AstKind::LabeledStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_label_identifier(&it.label);
        visitor.visit_statement(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_return_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &ReturnStatement<'a>) {
        let kind = AstKind::ReturnStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        if let Some(argument) = &it.argument {
            visitor.visit_expression(argument);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_switch_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.discriminant);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_switch_cases(&it.cases);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_switch_cases<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, SwitchCase<'a>>) {
        for el in it {
            visitor.visit_switch_case(el);
        }
    }

    #[inline]
    pub fn walk_switch_case<'a, V: Visit<'a>>(visitor: &mut V, it: &SwitchCase<'a>) {
        let kind = AstKind::SwitchCase(visitor.alloc(it));
        visitor.enter_node(kind);
        if let Some(test) = &it.test {
            visitor.visit_expression(test);
        }
        visitor.visit_statements(&it.consequent);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_throw_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &ThrowStatement<'a>) {
        let kind = AstKind::ThrowStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_try_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &TryStatement<'a>) {
        let kind = AstKind::TryStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_block_statement(&it.block);
        if let Some(handler) = &it.handler {
            visitor.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &it.finalizer {
            visitor.visit_finally_clause(finalizer);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_catch_clause<'a, V: Visit<'a>>(visitor: &mut V, it: &CatchClause<'a>) {
        let kind = AstKind::CatchClause(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::CatchClause, &it.scope_id);
        if let Some(param) = &it.param {
            visitor.visit_catch_parameter(param);
        }
        visitor.visit_block_statement(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_catch_parameter<'a, V: Visit<'a>>(visitor: &mut V, it: &CatchParameter<'a>) {
        let kind = AstKind::CatchParameter(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&it.pattern);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_finally_clause<'a, V: Visit<'a>>(visitor: &mut V, it: &BlockStatement<'a>) {
        let kind = AstKind::FinallyClause(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_statements(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_while_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.test);
        visitor.visit_statement(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_with_statement<'a, V: Visit<'a>>(visitor: &mut V, it: &WithStatement<'a>) {
        let kind = AstKind::WithStatement(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.object);
        visitor.visit_statement(&it.body);
        visitor.leave_node(kind);
    }

    pub fn walk_declaration<'a, V: Visit<'a>>(visitor: &mut V, it: &Declaration<'a>) {
        match it {
            Declaration::VariableDeclaration(it) => visitor.visit_variable_declaration(it),
            Declaration::FunctionDeclaration(it) => {
                let flags = ScopeFlags::Function;
                visitor.visit_function(it, flags)
            }
            Declaration::ClassDeclaration(it) => visitor.visit_class(it),
            Declaration::TSTypeAliasDeclaration(it) => visitor.visit_ts_type_alias_declaration(it),
            Declaration::TSInterfaceDeclaration(it) => visitor.visit_ts_interface_declaration(it),
            Declaration::TSEnumDeclaration(it) => visitor.visit_ts_enum_declaration(it),
            Declaration::TSModuleDeclaration(it) => visitor.visit_ts_module_declaration(it),
            Declaration::TSImportEqualsDeclaration(it) => {
                visitor.visit_ts_import_equals_declaration(it)
            }
        }
    }

    #[inline]
    pub fn walk_ts_type_alias_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSTypeAliasDeclaration<'a>,
    ) {
        let kind = AstKind::TSTypeAliasDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&it.id);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.visit_ts_type(&it.type_annotation);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_interface_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSInterfaceDeclaration<'a>,
    ) {
        let kind = AstKind::TSInterfaceDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&it.id);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        if let Some(extends) = &it.extends {
            visitor.visit_ts_interface_heritages(extends);
        }
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.visit_ts_interface_body(&it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_interface_heritages<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, TSInterfaceHeritage<'a>>,
    ) {
        for el in it {
            visitor.visit_ts_interface_heritage(el);
        }
    }

    #[inline]
    pub fn walk_ts_interface_heritage<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSInterfaceHeritage<'a>,
    ) {
        let kind = AstKind::TSInterfaceHeritage(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        if let Some(type_parameters) = &it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_interface_body<'a, V: Visit<'a>>(visitor: &mut V, it: &TSInterfaceBody<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_ts_signatures(&it.body);
    }

    #[inline]
    pub fn walk_ts_enum_declaration<'a, V: Visit<'a>>(visitor: &mut V, it: &TSEnumDeclaration<'a>) {
        let kind = AstKind::TSEnumDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&it.id);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_ts_enum_members(&it.members);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_enum_members<'a, V: Visit<'a>>(visitor: &mut V, it: &Vec<'a, TSEnumMember<'a>>) {
        for el in it {
            visitor.visit_ts_enum_member(el);
        }
    }

    #[inline]
    pub fn walk_ts_enum_member<'a, V: Visit<'a>>(visitor: &mut V, it: &TSEnumMember<'a>) {
        let kind = AstKind::TSEnumMember(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_enum_member_name(&it.id);
        if let Some(initializer) = &it.initializer {
            visitor.visit_expression(initializer);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_enum_member_name<'a, V: Visit<'a>>(visitor: &mut V, it: &TSEnumMemberName<'a>) {
        match it {
            TSEnumMemberName::StaticIdentifier(it) => visitor.visit_identifier_name(it),
            TSEnumMemberName::StaticStringLiteral(it) => visitor.visit_string_literal(it),
            TSEnumMemberName::StaticTemplateLiteral(it) => visitor.visit_template_literal(it),
            TSEnumMemberName::StaticNumericLiteral(it) => visitor.visit_numeric_literal(it),
            match_expression!(TSEnumMemberName) => visitor.visit_expression(it.to_expression()),
        }
    }

    #[inline]
    pub fn walk_ts_module_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSModuleDeclaration<'a>,
    ) {
        let kind = AstKind::TSModuleDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_ts_module_declaration_name(&it.id);
        visitor.enter_scope(
            {
                let mut flags = ScopeFlags::TsModuleBlock;
                if it.body.as_ref().is_some_and(TSModuleDeclarationBody::is_strict) {
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
    pub fn walk_ts_module_declaration_name<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSModuleDeclarationName<'a>,
    ) {
        match it {
            TSModuleDeclarationName::Identifier(it) => visitor.visit_binding_identifier(it),
            TSModuleDeclarationName::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_ts_module_declaration_body<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSModuleDeclarationBody<'a>,
    ) {
        match it {
            TSModuleDeclarationBody::TSModuleDeclaration(it) => {
                visitor.visit_ts_module_declaration(it)
            }
            TSModuleDeclarationBody::TSModuleBlock(it) => visitor.visit_ts_module_block(it),
        }
    }

    #[inline]
    pub fn walk_ts_module_block<'a, V: Visit<'a>>(visitor: &mut V, it: &TSModuleBlock<'a>) {
        let kind = AstKind::TSModuleBlock(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_directives(&it.directives);
        visitor.visit_statements(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_import_equals_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSImportEqualsDeclaration<'a>,
    ) {
        let kind = AstKind::TSImportEqualsDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&it.id);
        visitor.visit_ts_module_reference(&it.module_reference);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_module_reference<'a, V: Visit<'a>>(visitor: &mut V, it: &TSModuleReference<'a>) {
        let kind = AstKind::TSModuleReference(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            TSModuleReference::ExternalModuleReference(it) => {
                visitor.visit_ts_external_module_reference(it)
            }
            match_ts_type_name!(TSModuleReference) => {
                visitor.visit_ts_type_name(it.to_ts_type_name())
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_external_module_reference<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSExternalModuleReference<'a>,
    ) {
        let kind = AstKind::TSExternalModuleReference(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_string_literal(&it.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_module_declaration<'a, V: Visit<'a>>(visitor: &mut V, it: &ModuleDeclaration<'a>) {
        let kind = AstKind::ModuleDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
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
            ModuleDeclaration::TSNamespaceExportDeclaration(it) => {
                visitor.visit_ts_namespace_export_declaration(it)
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_declaration<'a, V: Visit<'a>>(visitor: &mut V, it: &ImportDeclaration<'a>) {
        let kind = AstKind::ImportDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
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
    pub fn walk_import_declaration_specifiers<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, ImportDeclarationSpecifier<'a>>,
    ) {
        for el in it {
            visitor.visit_import_declaration_specifier(el);
        }
    }

    #[inline]
    pub fn walk_import_declaration_specifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ImportDeclarationSpecifier<'a>,
    ) {
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
    pub fn walk_import_specifier<'a, V: Visit<'a>>(visitor: &mut V, it: &ImportSpecifier<'a>) {
        let kind = AstKind::ImportSpecifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_module_export_name(&it.imported);
        visitor.visit_binding_identifier(&it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_module_export_name<'a, V: Visit<'a>>(visitor: &mut V, it: &ModuleExportName<'a>) {
        match it {
            ModuleExportName::IdentifierName(it) => visitor.visit_identifier_name(it),
            ModuleExportName::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            ModuleExportName::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_import_default_specifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ImportDefaultSpecifier<'a>,
    ) {
        let kind = AstKind::ImportDefaultSpecifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_namespace_specifier<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ImportNamespaceSpecifier<'a>,
    ) {
        let kind = AstKind::ImportNamespaceSpecifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_with_clause<'a, V: Visit<'a>>(visitor: &mut V, it: &WithClause<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_identifier_name(&it.attributes_keyword);
        visitor.visit_import_attributes(&it.with_entries);
    }

    #[inline]
    pub fn walk_import_attributes<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, ImportAttribute<'a>>,
    ) {
        for el in it {
            visitor.visit_import_attribute(el);
        }
    }

    #[inline]
    pub fn walk_import_attribute<'a, V: Visit<'a>>(visitor: &mut V, it: &ImportAttribute<'a>) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_import_attribute_key(&it.key);
        visitor.visit_string_literal(&it.value);
    }

    #[inline]
    pub fn walk_import_attribute_key<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ImportAttributeKey<'a>,
    ) {
        match it {
            ImportAttributeKey::Identifier(it) => visitor.visit_identifier_name(it),
            ImportAttributeKey::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_export_all_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ExportAllDeclaration<'a>,
    ) {
        let kind = AstKind::ExportAllDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
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
    pub fn walk_export_default_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ExportDefaultDeclaration<'a>,
    ) {
        let kind = AstKind::ExportDefaultDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_export_default_declaration_kind(&it.declaration);
        visitor.visit_module_export_name(&it.exported);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_export_default_declaration_kind<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ExportDefaultDeclarationKind<'a>,
    ) {
        match it {
            ExportDefaultDeclarationKind::FunctionDeclaration(it) => {
                let flags = ScopeFlags::Function;
                visitor.visit_function(it, flags)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(it) => visitor.visit_class(it),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(it) => {
                visitor.visit_ts_interface_declaration(it)
            }
            match_expression!(ExportDefaultDeclarationKind) => {
                visitor.visit_expression(it.to_expression())
            }
        }
    }

    #[inline]
    pub fn walk_export_named_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ExportNamedDeclaration<'a>,
    ) {
        let kind = AstKind::ExportNamedDeclaration(visitor.alloc(it));
        visitor.enter_node(kind);
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
    pub fn walk_export_specifiers<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &Vec<'a, ExportSpecifier<'a>>,
    ) {
        for el in it {
            visitor.visit_export_specifier(el);
        }
    }

    #[inline]
    pub fn walk_export_specifier<'a, V: Visit<'a>>(visitor: &mut V, it: &ExportSpecifier<'a>) {
        let kind = AstKind::ExportSpecifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_module_export_name(&it.local);
        visitor.visit_module_export_name(&it.exported);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_export_assignment<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSExportAssignment<'a>,
    ) {
        let kind = AstKind::TSExportAssignment(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_expression(&it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_namespace_export_declaration<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &TSNamespaceExportDeclaration<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        visitor.visit_identifier_name(&it.id);
    }
}
