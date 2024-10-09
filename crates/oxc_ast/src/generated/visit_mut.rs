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
use crate::ast_kind::AstType;

use walk_mut::*;

/// Syntax tree traversal
pub trait VisitMut<'a>: Sized {
    #[inline]
    fn enter_node(&mut self, kind: AstType) {}
    #[inline]
    fn leave_node(&mut self, kind: AstType) {}

    #[inline]
    fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {}
    #[inline]
    fn leave_scope(&mut self) {}

    #[inline]
    fn visit_program(&mut self, it: &mut Program<'a>) {
        walk_program(self, it);
    }

    #[inline]
    fn visit_hashbang(&mut self, it: &mut Hashbang<'a>) {
        walk_hashbang(self, it);
    }

    #[inline]
    fn visit_directives(&mut self, it: &mut Vec<'a, Directive<'a>>) {
        walk_directives(self, it);
    }

    #[inline]
    fn visit_directive(&mut self, it: &mut Directive<'a>) {
        walk_directive(self, it);
    }

    #[inline]
    fn visit_string_literal(&mut self, it: &mut StringLiteral<'a>) {
        walk_string_literal(self, it);
    }

    #[inline]
    fn visit_statements(&mut self, it: &mut Vec<'a, Statement<'a>>) {
        walk_statements(self, it);
    }

    #[inline]
    fn visit_statement(&mut self, it: &mut Statement<'a>) {
        walk_statement(self, it);
    }

    #[inline]
    fn visit_block_statement(&mut self, it: &mut BlockStatement<'a>) {
        walk_block_statement(self, it);
    }

    #[inline]
    fn visit_break_statement(&mut self, it: &mut BreakStatement<'a>) {
        walk_break_statement(self, it);
    }

    #[inline]
    fn visit_label_identifier(&mut self, it: &mut LabelIdentifier<'a>) {
        walk_label_identifier(self, it);
    }

    #[inline]
    fn visit_continue_statement(&mut self, it: &mut ContinueStatement<'a>) {
        walk_continue_statement(self, it);
    }

    #[inline]
    fn visit_debugger_statement(&mut self, it: &mut DebuggerStatement) {
        walk_debugger_statement(self, it);
    }

    #[inline]
    fn visit_do_while_statement(&mut self, it: &mut DoWhileStatement<'a>) {
        walk_do_while_statement(self, it);
    }

    #[inline]
    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        walk_expression(self, it);
    }

    #[inline]
    fn visit_boolean_literal(&mut self, it: &mut BooleanLiteral) {
        walk_boolean_literal(self, it);
    }

    #[inline]
    fn visit_null_literal(&mut self, it: &mut NullLiteral) {
        walk_null_literal(self, it);
    }

    #[inline]
    fn visit_numeric_literal(&mut self, it: &mut NumericLiteral<'a>) {
        walk_numeric_literal(self, it);
    }

    #[inline]
    fn visit_big_int_literal(&mut self, it: &mut BigIntLiteral<'a>) {
        walk_big_int_literal(self, it);
    }

    #[inline]
    fn visit_reg_exp_literal(&mut self, it: &mut RegExpLiteral<'a>) {
        walk_reg_exp_literal(self, it);
    }

    #[inline]
    fn visit_template_literal(&mut self, it: &mut TemplateLiteral<'a>) {
        walk_template_literal(self, it);
    }

    #[inline]
    fn visit_template_elements(&mut self, it: &mut Vec<'a, TemplateElement<'a>>) {
        walk_template_elements(self, it);
    }

    #[inline]
    fn visit_template_element(&mut self, it: &mut TemplateElement<'a>) {
        walk_template_element(self, it);
    }

    #[inline]
    fn visit_expressions(&mut self, it: &mut Vec<'a, Expression<'a>>) {
        walk_expressions(self, it);
    }

    #[inline]
    fn visit_identifier_reference(&mut self, it: &mut IdentifierReference<'a>) {
        walk_identifier_reference(self, it);
    }

    #[inline]
    fn visit_meta_property(&mut self, it: &mut MetaProperty<'a>) {
        walk_meta_property(self, it);
    }

    #[inline]
    fn visit_identifier_name(&mut self, it: &mut IdentifierName<'a>) {
        walk_identifier_name(self, it);
    }

    #[inline]
    fn visit_super(&mut self, it: &mut Super) {
        walk_super(self, it);
    }

    #[inline]
    fn visit_array_expression(&mut self, it: &mut ArrayExpression<'a>) {
        walk_array_expression(self, it);
    }

    #[inline]
    fn visit_array_expression_elements(&mut self, it: &mut Vec<'a, ArrayExpressionElement<'a>>) {
        walk_array_expression_elements(self, it);
    }

    #[inline]
    fn visit_array_expression_element(&mut self, it: &mut ArrayExpressionElement<'a>) {
        walk_array_expression_element(self, it);
    }

    #[inline]
    fn visit_spread_element(&mut self, it: &mut SpreadElement<'a>) {
        walk_spread_element(self, it);
    }

    #[inline]
    fn visit_elision(&mut self, it: &mut Elision) {
        walk_elision(self, it);
    }

    #[inline]
    fn visit_expression_array_element(&mut self, it: &mut Expression<'a>) {
        walk_expression_array_element(self, it);
    }

    #[inline]
    fn visit_arrow_function_expression(&mut self, it: &mut ArrowFunctionExpression<'a>) {
        walk_arrow_function_expression(self, it);
    }

    #[inline]
    fn visit_ts_type_parameter_declaration(&mut self, it: &mut TSTypeParameterDeclaration<'a>) {
        walk_ts_type_parameter_declaration(self, it);
    }

    #[inline]
    fn visit_ts_type_parameters(&mut self, it: &mut Vec<'a, TSTypeParameter<'a>>) {
        walk_ts_type_parameters(self, it);
    }

    #[inline]
    fn visit_ts_type_parameter(&mut self, it: &mut TSTypeParameter<'a>) {
        walk_ts_type_parameter(self, it);
    }

    #[inline]
    fn visit_binding_identifier(&mut self, it: &mut BindingIdentifier<'a>) {
        walk_binding_identifier(self, it);
    }

    #[inline]
    fn visit_ts_type(&mut self, it: &mut TSType<'a>) {
        walk_ts_type(self, it);
    }

    #[inline]
    fn visit_ts_any_keyword(&mut self, it: &mut TSAnyKeyword) {
        walk_ts_any_keyword(self, it);
    }

    #[inline]
    fn visit_ts_big_int_keyword(&mut self, it: &mut TSBigIntKeyword) {
        walk_ts_big_int_keyword(self, it);
    }

    #[inline]
    fn visit_ts_boolean_keyword(&mut self, it: &mut TSBooleanKeyword) {
        walk_ts_boolean_keyword(self, it);
    }

    #[inline]
    fn visit_ts_intrinsic_keyword(&mut self, it: &mut TSIntrinsicKeyword) {
        walk_ts_intrinsic_keyword(self, it);
    }

    #[inline]
    fn visit_ts_never_keyword(&mut self, it: &mut TSNeverKeyword) {
        walk_ts_never_keyword(self, it);
    }

    #[inline]
    fn visit_ts_null_keyword(&mut self, it: &mut TSNullKeyword) {
        walk_ts_null_keyword(self, it);
    }

    #[inline]
    fn visit_ts_number_keyword(&mut self, it: &mut TSNumberKeyword) {
        walk_ts_number_keyword(self, it);
    }

    #[inline]
    fn visit_ts_object_keyword(&mut self, it: &mut TSObjectKeyword) {
        walk_ts_object_keyword(self, it);
    }

    #[inline]
    fn visit_ts_string_keyword(&mut self, it: &mut TSStringKeyword) {
        walk_ts_string_keyword(self, it);
    }

    #[inline]
    fn visit_ts_symbol_keyword(&mut self, it: &mut TSSymbolKeyword) {
        walk_ts_symbol_keyword(self, it);
    }

    #[inline]
    fn visit_ts_undefined_keyword(&mut self, it: &mut TSUndefinedKeyword) {
        walk_ts_undefined_keyword(self, it);
    }

    #[inline]
    fn visit_ts_unknown_keyword(&mut self, it: &mut TSUnknownKeyword) {
        walk_ts_unknown_keyword(self, it);
    }

    #[inline]
    fn visit_ts_void_keyword(&mut self, it: &mut TSVoidKeyword) {
        walk_ts_void_keyword(self, it);
    }

    #[inline]
    fn visit_ts_array_type(&mut self, it: &mut TSArrayType<'a>) {
        walk_ts_array_type(self, it);
    }

    #[inline]
    fn visit_ts_conditional_type(&mut self, it: &mut TSConditionalType<'a>) {
        walk_ts_conditional_type(self, it);
    }

    #[inline]
    fn visit_ts_constructor_type(&mut self, it: &mut TSConstructorType<'a>) {
        walk_ts_constructor_type(self, it);
    }

    #[inline]
    fn visit_formal_parameters(&mut self, it: &mut FormalParameters<'a>) {
        walk_formal_parameters(self, it);
    }

    #[inline]
    fn visit_formal_parameter_list(&mut self, it: &mut Vec<'a, FormalParameter<'a>>) {
        walk_formal_parameter_list(self, it);
    }

    #[inline]
    fn visit_formal_parameter(&mut self, it: &mut FormalParameter<'a>) {
        walk_formal_parameter(self, it);
    }

    #[inline]
    fn visit_decorators(&mut self, it: &mut Vec<'a, Decorator<'a>>) {
        walk_decorators(self, it);
    }

    #[inline]
    fn visit_decorator(&mut self, it: &mut Decorator<'a>) {
        walk_decorator(self, it);
    }

    #[inline]
    fn visit_binding_pattern(&mut self, it: &mut BindingPattern<'a>) {
        walk_binding_pattern(self, it);
    }

    #[inline]
    fn visit_binding_pattern_kind(&mut self, it: &mut BindingPatternKind<'a>) {
        walk_binding_pattern_kind(self, it);
    }

    #[inline]
    fn visit_object_pattern(&mut self, it: &mut ObjectPattern<'a>) {
        walk_object_pattern(self, it);
    }

    #[inline]
    fn visit_binding_properties(&mut self, it: &mut Vec<'a, BindingProperty<'a>>) {
        walk_binding_properties(self, it);
    }

    #[inline]
    fn visit_binding_property(&mut self, it: &mut BindingProperty<'a>) {
        walk_binding_property(self, it);
    }

    #[inline]
    fn visit_property_key(&mut self, it: &mut PropertyKey<'a>) {
        walk_property_key(self, it);
    }

    #[inline]
    fn visit_private_identifier(&mut self, it: &mut PrivateIdentifier<'a>) {
        walk_private_identifier(self, it);
    }

    #[inline]
    fn visit_binding_rest_element(&mut self, it: &mut BindingRestElement<'a>) {
        walk_binding_rest_element(self, it);
    }

    #[inline]
    fn visit_array_pattern(&mut self, it: &mut ArrayPattern<'a>) {
        walk_array_pattern(self, it);
    }

    #[inline]
    fn visit_assignment_pattern(&mut self, it: &mut AssignmentPattern<'a>) {
        walk_assignment_pattern(self, it);
    }

    #[inline]
    fn visit_ts_type_annotation(&mut self, it: &mut TSTypeAnnotation<'a>) {
        walk_ts_type_annotation(self, it);
    }

    #[inline]
    fn visit_ts_function_type(&mut self, it: &mut TSFunctionType<'a>) {
        walk_ts_function_type(self, it);
    }

    #[inline]
    fn visit_ts_this_parameter(&mut self, it: &mut TSThisParameter<'a>) {
        walk_ts_this_parameter(self, it);
    }

    #[inline]
    fn visit_ts_import_type(&mut self, it: &mut TSImportType<'a>) {
        walk_ts_import_type(self, it);
    }

    #[inline]
    fn visit_ts_type_name(&mut self, it: &mut TSTypeName<'a>) {
        walk_ts_type_name(self, it);
    }

    #[inline]
    fn visit_ts_qualified_name(&mut self, it: &mut TSQualifiedName<'a>) {
        walk_ts_qualified_name(self, it);
    }

    #[inline]
    fn visit_ts_import_attributes(&mut self, it: &mut TSImportAttributes<'a>) {
        walk_ts_import_attributes(self, it);
    }

    #[inline]
    fn visit_ts_import_attribute_list(&mut self, it: &mut Vec<'a, TSImportAttribute<'a>>) {
        walk_ts_import_attribute_list(self, it);
    }

    #[inline]
    fn visit_ts_import_attribute(&mut self, it: &mut TSImportAttribute<'a>) {
        walk_ts_import_attribute(self, it);
    }

    #[inline]
    fn visit_ts_import_attribute_name(&mut self, it: &mut TSImportAttributeName<'a>) {
        walk_ts_import_attribute_name(self, it);
    }

    #[inline]
    fn visit_ts_type_parameter_instantiation(&mut self, it: &mut TSTypeParameterInstantiation<'a>) {
        walk_ts_type_parameter_instantiation(self, it);
    }

    #[inline]
    fn visit_ts_types(&mut self, it: &mut Vec<'a, TSType<'a>>) {
        walk_ts_types(self, it);
    }

    #[inline]
    fn visit_ts_indexed_access_type(&mut self, it: &mut TSIndexedAccessType<'a>) {
        walk_ts_indexed_access_type(self, it);
    }

    #[inline]
    fn visit_ts_infer_type(&mut self, it: &mut TSInferType<'a>) {
        walk_ts_infer_type(self, it);
    }

    #[inline]
    fn visit_ts_intersection_type(&mut self, it: &mut TSIntersectionType<'a>) {
        walk_ts_intersection_type(self, it);
    }

    #[inline]
    fn visit_ts_literal_type(&mut self, it: &mut TSLiteralType<'a>) {
        walk_ts_literal_type(self, it);
    }

    #[inline]
    fn visit_ts_literal(&mut self, it: &mut TSLiteral<'a>) {
        walk_ts_literal(self, it);
    }

    #[inline]
    fn visit_unary_expression(&mut self, it: &mut UnaryExpression<'a>) {
        walk_unary_expression(self, it);
    }

    #[inline]
    fn visit_ts_mapped_type(&mut self, it: &mut TSMappedType<'a>) {
        walk_ts_mapped_type(self, it);
    }

    #[inline]
    fn visit_ts_named_tuple_member(&mut self, it: &mut TSNamedTupleMember<'a>) {
        walk_ts_named_tuple_member(self, it);
    }

    #[inline]
    fn visit_ts_tuple_element(&mut self, it: &mut TSTupleElement<'a>) {
        walk_ts_tuple_element(self, it);
    }

    #[inline]
    fn visit_ts_optional_type(&mut self, it: &mut TSOptionalType<'a>) {
        walk_ts_optional_type(self, it);
    }

    #[inline]
    fn visit_ts_rest_type(&mut self, it: &mut TSRestType<'a>) {
        walk_ts_rest_type(self, it);
    }

    #[inline]
    fn visit_ts_template_literal_type(&mut self, it: &mut TSTemplateLiteralType<'a>) {
        walk_ts_template_literal_type(self, it);
    }

    #[inline]
    fn visit_ts_this_type(&mut self, it: &mut TSThisType) {
        walk_ts_this_type(self, it);
    }

    #[inline]
    fn visit_ts_tuple_type(&mut self, it: &mut TSTupleType<'a>) {
        walk_ts_tuple_type(self, it);
    }

    #[inline]
    fn visit_ts_tuple_elements(&mut self, it: &mut Vec<'a, TSTupleElement<'a>>) {
        walk_ts_tuple_elements(self, it);
    }

    #[inline]
    fn visit_ts_type_literal(&mut self, it: &mut TSTypeLiteral<'a>) {
        walk_ts_type_literal(self, it);
    }

    #[inline]
    fn visit_ts_signatures(&mut self, it: &mut Vec<'a, TSSignature<'a>>) {
        walk_ts_signatures(self, it);
    }

    #[inline]
    fn visit_ts_signature(&mut self, it: &mut TSSignature<'a>) {
        walk_ts_signature(self, it);
    }

    #[inline]
    fn visit_ts_index_signature(&mut self, it: &mut TSIndexSignature<'a>) {
        walk_ts_index_signature(self, it);
    }

    #[inline]
    fn visit_ts_index_signature_names(&mut self, it: &mut Vec<'a, TSIndexSignatureName<'a>>) {
        walk_ts_index_signature_names(self, it);
    }

    #[inline]
    fn visit_ts_index_signature_name(&mut self, it: &mut TSIndexSignatureName<'a>) {
        walk_ts_index_signature_name(self, it);
    }

    #[inline]
    fn visit_ts_property_signature(&mut self, it: &mut TSPropertySignature<'a>) {
        walk_ts_property_signature(self, it);
    }

    #[inline]
    fn visit_ts_call_signature_declaration(&mut self, it: &mut TSCallSignatureDeclaration<'a>) {
        walk_ts_call_signature_declaration(self, it);
    }

    #[inline]
    fn visit_ts_construct_signature_declaration(
        &mut self,
        it: &mut TSConstructSignatureDeclaration<'a>,
    ) {
        walk_ts_construct_signature_declaration(self, it);
    }

    #[inline]
    fn visit_ts_method_signature(&mut self, it: &mut TSMethodSignature<'a>) {
        walk_ts_method_signature(self, it);
    }

    #[inline]
    fn visit_ts_type_operator(&mut self, it: &mut TSTypeOperator<'a>) {
        walk_ts_type_operator(self, it);
    }

    #[inline]
    fn visit_ts_type_predicate(&mut self, it: &mut TSTypePredicate<'a>) {
        walk_ts_type_predicate(self, it);
    }

    #[inline]
    fn visit_ts_type_predicate_name(&mut self, it: &mut TSTypePredicateName<'a>) {
        walk_ts_type_predicate_name(self, it);
    }

    #[inline]
    fn visit_ts_type_query(&mut self, it: &mut TSTypeQuery<'a>) {
        walk_ts_type_query(self, it);
    }

    #[inline]
    fn visit_ts_type_query_expr_name(&mut self, it: &mut TSTypeQueryExprName<'a>) {
        walk_ts_type_query_expr_name(self, it);
    }

    #[inline]
    fn visit_ts_type_reference(&mut self, it: &mut TSTypeReference<'a>) {
        walk_ts_type_reference(self, it);
    }

    #[inline]
    fn visit_ts_union_type(&mut self, it: &mut TSUnionType<'a>) {
        walk_ts_union_type(self, it);
    }

    #[inline]
    fn visit_ts_parenthesized_type(&mut self, it: &mut TSParenthesizedType<'a>) {
        walk_ts_parenthesized_type(self, it);
    }

    #[inline]
    fn visit_js_doc_nullable_type(&mut self, it: &mut JSDocNullableType<'a>) {
        walk_js_doc_nullable_type(self, it);
    }

    #[inline]
    fn visit_js_doc_non_nullable_type(&mut self, it: &mut JSDocNonNullableType<'a>) {
        walk_js_doc_non_nullable_type(self, it);
    }

    #[inline]
    fn visit_js_doc_unknown_type(&mut self, it: &mut JSDocUnknownType) {
        walk_js_doc_unknown_type(self, it);
    }

    #[inline]
    fn visit_function_body(&mut self, it: &mut FunctionBody<'a>) {
        walk_function_body(self, it);
    }

    #[inline]
    fn visit_assignment_expression(&mut self, it: &mut AssignmentExpression<'a>) {
        walk_assignment_expression(self, it);
    }

    #[inline]
    fn visit_assignment_target(&mut self, it: &mut AssignmentTarget<'a>) {
        walk_assignment_target(self, it);
    }

    #[inline]
    fn visit_simple_assignment_target(&mut self, it: &mut SimpleAssignmentTarget<'a>) {
        walk_simple_assignment_target(self, it);
    }

    #[inline]
    fn visit_ts_as_expression(&mut self, it: &mut TSAsExpression<'a>) {
        walk_ts_as_expression(self, it);
    }

    #[inline]
    fn visit_ts_satisfies_expression(&mut self, it: &mut TSSatisfiesExpression<'a>) {
        walk_ts_satisfies_expression(self, it);
    }

    #[inline]
    fn visit_ts_non_null_expression(&mut self, it: &mut TSNonNullExpression<'a>) {
        walk_ts_non_null_expression(self, it);
    }

    #[inline]
    fn visit_ts_type_assertion(&mut self, it: &mut TSTypeAssertion<'a>) {
        walk_ts_type_assertion(self, it);
    }

    #[inline]
    fn visit_ts_instantiation_expression(&mut self, it: &mut TSInstantiationExpression<'a>) {
        walk_ts_instantiation_expression(self, it);
    }

    #[inline]
    fn visit_member_expression(&mut self, it: &mut MemberExpression<'a>) {
        walk_member_expression(self, it);
    }

    #[inline]
    fn visit_computed_member_expression(&mut self, it: &mut ComputedMemberExpression<'a>) {
        walk_computed_member_expression(self, it);
    }

    #[inline]
    fn visit_static_member_expression(&mut self, it: &mut StaticMemberExpression<'a>) {
        walk_static_member_expression(self, it);
    }

    #[inline]
    fn visit_private_field_expression(&mut self, it: &mut PrivateFieldExpression<'a>) {
        walk_private_field_expression(self, it);
    }

    #[inline]
    fn visit_assignment_target_pattern(&mut self, it: &mut AssignmentTargetPattern<'a>) {
        walk_assignment_target_pattern(self, it);
    }

    #[inline]
    fn visit_array_assignment_target(&mut self, it: &mut ArrayAssignmentTarget<'a>) {
        walk_array_assignment_target(self, it);
    }

    #[inline]
    fn visit_assignment_target_maybe_default(&mut self, it: &mut AssignmentTargetMaybeDefault<'a>) {
        walk_assignment_target_maybe_default(self, it);
    }

    #[inline]
    fn visit_assignment_target_with_default(&mut self, it: &mut AssignmentTargetWithDefault<'a>) {
        walk_assignment_target_with_default(self, it);
    }

    #[inline]
    fn visit_assignment_target_rest(&mut self, it: &mut AssignmentTargetRest<'a>) {
        walk_assignment_target_rest(self, it);
    }

    #[inline]
    fn visit_object_assignment_target(&mut self, it: &mut ObjectAssignmentTarget<'a>) {
        walk_object_assignment_target(self, it);
    }

    #[inline]
    fn visit_assignment_target_properties(
        &mut self,
        it: &mut Vec<'a, AssignmentTargetProperty<'a>>,
    ) {
        walk_assignment_target_properties(self, it);
    }

    #[inline]
    fn visit_assignment_target_property(&mut self, it: &mut AssignmentTargetProperty<'a>) {
        walk_assignment_target_property(self, it);
    }

    #[inline]
    fn visit_assignment_target_property_identifier(
        &mut self,
        it: &mut AssignmentTargetPropertyIdentifier<'a>,
    ) {
        walk_assignment_target_property_identifier(self, it);
    }

    #[inline]
    fn visit_assignment_target_property_property(
        &mut self,
        it: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        walk_assignment_target_property_property(self, it);
    }

    #[inline]
    fn visit_await_expression(&mut self, it: &mut AwaitExpression<'a>) {
        walk_await_expression(self, it);
    }

    #[inline]
    fn visit_binary_expression(&mut self, it: &mut BinaryExpression<'a>) {
        walk_binary_expression(self, it);
    }

    #[inline]
    fn visit_call_expression(&mut self, it: &mut CallExpression<'a>) {
        walk_call_expression(self, it);
    }

    #[inline]
    fn visit_arguments(&mut self, it: &mut Vec<'a, Argument<'a>>) {
        walk_arguments(self, it);
    }

    #[inline]
    fn visit_argument(&mut self, it: &mut Argument<'a>) {
        walk_argument(self, it);
    }

    #[inline]
    fn visit_chain_expression(&mut self, it: &mut ChainExpression<'a>) {
        walk_chain_expression(self, it);
    }

    #[inline]
    fn visit_chain_element(&mut self, it: &mut ChainElement<'a>) {
        walk_chain_element(self, it);
    }

    #[inline]
    fn visit_class(&mut self, it: &mut Class<'a>) {
        walk_class(self, it);
    }

    #[inline]
    fn visit_class_heritage(&mut self, it: &mut Expression<'a>) {
        walk_class_heritage(self, it);
    }

    #[inline]
    fn visit_ts_class_implementses(&mut self, it: &mut Vec<'a, TSClassImplements<'a>>) {
        walk_ts_class_implementses(self, it);
    }

    #[inline]
    fn visit_ts_class_implements(&mut self, it: &mut TSClassImplements<'a>) {
        walk_ts_class_implements(self, it);
    }

    #[inline]
    fn visit_class_body(&mut self, it: &mut ClassBody<'a>) {
        walk_class_body(self, it);
    }

    #[inline]
    fn visit_class_elements(&mut self, it: &mut Vec<'a, ClassElement<'a>>) {
        walk_class_elements(self, it);
    }

    #[inline]
    fn visit_class_element(&mut self, it: &mut ClassElement<'a>) {
        walk_class_element(self, it);
    }

    #[inline]
    fn visit_static_block(&mut self, it: &mut StaticBlock<'a>) {
        walk_static_block(self, it);
    }

    #[inline]
    fn visit_method_definition(&mut self, it: &mut MethodDefinition<'a>) {
        walk_method_definition(self, it);
    }

    #[inline]
    fn visit_function(&mut self, it: &mut Function<'a>, flags: ScopeFlags) {
        walk_function(self, it, flags);
    }

    #[inline]
    fn visit_property_definition(&mut self, it: &mut PropertyDefinition<'a>) {
        walk_property_definition(self, it);
    }

    #[inline]
    fn visit_accessor_property(&mut self, it: &mut AccessorProperty<'a>) {
        walk_accessor_property(self, it);
    }

    #[inline]
    fn visit_conditional_expression(&mut self, it: &mut ConditionalExpression<'a>) {
        walk_conditional_expression(self, it);
    }

    #[inline]
    fn visit_import_expression(&mut self, it: &mut ImportExpression<'a>) {
        walk_import_expression(self, it);
    }

    #[inline]
    fn visit_logical_expression(&mut self, it: &mut LogicalExpression<'a>) {
        walk_logical_expression(self, it);
    }

    #[inline]
    fn visit_new_expression(&mut self, it: &mut NewExpression<'a>) {
        walk_new_expression(self, it);
    }

    #[inline]
    fn visit_object_expression(&mut self, it: &mut ObjectExpression<'a>) {
        walk_object_expression(self, it);
    }

    #[inline]
    fn visit_object_property_kinds(&mut self, it: &mut Vec<'a, ObjectPropertyKind<'a>>) {
        walk_object_property_kinds(self, it);
    }

    #[inline]
    fn visit_object_property_kind(&mut self, it: &mut ObjectPropertyKind<'a>) {
        walk_object_property_kind(self, it);
    }

    #[inline]
    fn visit_object_property(&mut self, it: &mut ObjectProperty<'a>) {
        walk_object_property(self, it);
    }

    #[inline]
    fn visit_parenthesized_expression(&mut self, it: &mut ParenthesizedExpression<'a>) {
        walk_parenthesized_expression(self, it);
    }

    #[inline]
    fn visit_sequence_expression(&mut self, it: &mut SequenceExpression<'a>) {
        walk_sequence_expression(self, it);
    }

    #[inline]
    fn visit_tagged_template_expression(&mut self, it: &mut TaggedTemplateExpression<'a>) {
        walk_tagged_template_expression(self, it);
    }

    #[inline]
    fn visit_this_expression(&mut self, it: &mut ThisExpression) {
        walk_this_expression(self, it);
    }

    #[inline]
    fn visit_update_expression(&mut self, it: &mut UpdateExpression<'a>) {
        walk_update_expression(self, it);
    }

    #[inline]
    fn visit_yield_expression(&mut self, it: &mut YieldExpression<'a>) {
        walk_yield_expression(self, it);
    }

    #[inline]
    fn visit_private_in_expression(&mut self, it: &mut PrivateInExpression<'a>) {
        walk_private_in_expression(self, it);
    }

    #[inline]
    fn visit_jsx_element(&mut self, it: &mut JSXElement<'a>) {
        walk_jsx_element(self, it);
    }

    #[inline]
    fn visit_jsx_opening_element(&mut self, it: &mut JSXOpeningElement<'a>) {
        walk_jsx_opening_element(self, it);
    }

    #[inline]
    fn visit_jsx_element_name(&mut self, it: &mut JSXElementName<'a>) {
        walk_jsx_element_name(self, it);
    }

    #[inline]
    fn visit_jsx_identifier(&mut self, it: &mut JSXIdentifier<'a>) {
        walk_jsx_identifier(self, it);
    }

    #[inline]
    fn visit_jsx_namespaced_name(&mut self, it: &mut JSXNamespacedName<'a>) {
        walk_jsx_namespaced_name(self, it);
    }

    #[inline]
    fn visit_jsx_member_expression(&mut self, it: &mut JSXMemberExpression<'a>) {
        walk_jsx_member_expression(self, it);
    }

    #[inline]
    fn visit_jsx_member_expression_object(&mut self, it: &mut JSXMemberExpressionObject<'a>) {
        walk_jsx_member_expression_object(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_items(&mut self, it: &mut Vec<'a, JSXAttributeItem<'a>>) {
        walk_jsx_attribute_items(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_item(&mut self, it: &mut JSXAttributeItem<'a>) {
        walk_jsx_attribute_item(self, it);
    }

    #[inline]
    fn visit_jsx_attribute(&mut self, it: &mut JSXAttribute<'a>) {
        walk_jsx_attribute(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_name(&mut self, it: &mut JSXAttributeName<'a>) {
        walk_jsx_attribute_name(self, it);
    }

    #[inline]
    fn visit_jsx_attribute_value(&mut self, it: &mut JSXAttributeValue<'a>) {
        walk_jsx_attribute_value(self, it);
    }

    #[inline]
    fn visit_jsx_expression_container(&mut self, it: &mut JSXExpressionContainer<'a>) {
        walk_jsx_expression_container(self, it);
    }

    #[inline]
    fn visit_jsx_expression(&mut self, it: &mut JSXExpression<'a>) {
        walk_jsx_expression(self, it);
    }

    #[inline]
    fn visit_jsx_empty_expression(&mut self, it: &mut JSXEmptyExpression) {
        walk_jsx_empty_expression(self, it);
    }

    #[inline]
    fn visit_jsx_fragment(&mut self, it: &mut JSXFragment<'a>) {
        walk_jsx_fragment(self, it);
    }

    #[inline]
    fn visit_jsx_children(&mut self, it: &mut Vec<'a, JSXChild<'a>>) {
        walk_jsx_children(self, it);
    }

    #[inline]
    fn visit_jsx_child(&mut self, it: &mut JSXChild<'a>) {
        walk_jsx_child(self, it);
    }

    #[inline]
    fn visit_jsx_text(&mut self, it: &mut JSXText<'a>) {
        walk_jsx_text(self, it);
    }

    #[inline]
    fn visit_jsx_spread_child(&mut self, it: &mut JSXSpreadChild<'a>) {
        walk_jsx_spread_child(self, it);
    }

    #[inline]
    fn visit_jsx_spread_attribute(&mut self, it: &mut JSXSpreadAttribute<'a>) {
        walk_jsx_spread_attribute(self, it);
    }

    #[inline]
    fn visit_jsx_closing_element(&mut self, it: &mut JSXClosingElement<'a>) {
        walk_jsx_closing_element(self, it);
    }

    #[inline]
    fn visit_empty_statement(&mut self, it: &mut EmptyStatement) {
        walk_empty_statement(self, it);
    }

    #[inline]
    fn visit_expression_statement(&mut self, it: &mut ExpressionStatement<'a>) {
        walk_expression_statement(self, it);
    }

    #[inline]
    fn visit_for_in_statement(&mut self, it: &mut ForInStatement<'a>) {
        walk_for_in_statement(self, it);
    }

    #[inline]
    fn visit_for_statement_left(&mut self, it: &mut ForStatementLeft<'a>) {
        walk_for_statement_left(self, it);
    }

    #[inline]
    fn visit_variable_declaration(&mut self, it: &mut VariableDeclaration<'a>) {
        walk_variable_declaration(self, it);
    }

    #[inline]
    fn visit_variable_declarators(&mut self, it: &mut Vec<'a, VariableDeclarator<'a>>) {
        walk_variable_declarators(self, it);
    }

    #[inline]
    fn visit_variable_declarator(&mut self, it: &mut VariableDeclarator<'a>) {
        walk_variable_declarator(self, it);
    }

    #[inline]
    fn visit_for_of_statement(&mut self, it: &mut ForOfStatement<'a>) {
        walk_for_of_statement(self, it);
    }

    #[inline]
    fn visit_for_statement(&mut self, it: &mut ForStatement<'a>) {
        walk_for_statement(self, it);
    }

    #[inline]
    fn visit_for_statement_init(&mut self, it: &mut ForStatementInit<'a>) {
        walk_for_statement_init(self, it);
    }

    #[inline]
    fn visit_if_statement(&mut self, it: &mut IfStatement<'a>) {
        walk_if_statement(self, it);
    }

    #[inline]
    fn visit_labeled_statement(&mut self, it: &mut LabeledStatement<'a>) {
        walk_labeled_statement(self, it);
    }

    #[inline]
    fn visit_return_statement(&mut self, it: &mut ReturnStatement<'a>) {
        walk_return_statement(self, it);
    }

    #[inline]
    fn visit_switch_statement(&mut self, it: &mut SwitchStatement<'a>) {
        walk_switch_statement(self, it);
    }

    #[inline]
    fn visit_switch_cases(&mut self, it: &mut Vec<'a, SwitchCase<'a>>) {
        walk_switch_cases(self, it);
    }

    #[inline]
    fn visit_switch_case(&mut self, it: &mut SwitchCase<'a>) {
        walk_switch_case(self, it);
    }

    #[inline]
    fn visit_throw_statement(&mut self, it: &mut ThrowStatement<'a>) {
        walk_throw_statement(self, it);
    }

    #[inline]
    fn visit_try_statement(&mut self, it: &mut TryStatement<'a>) {
        walk_try_statement(self, it);
    }

    #[inline]
    fn visit_catch_clause(&mut self, it: &mut CatchClause<'a>) {
        walk_catch_clause(self, it);
    }

    #[inline]
    fn visit_catch_parameter(&mut self, it: &mut CatchParameter<'a>) {
        walk_catch_parameter(self, it);
    }

    #[inline]
    fn visit_finally_clause(&mut self, it: &mut BlockStatement<'a>) {
        walk_finally_clause(self, it);
    }

    #[inline]
    fn visit_while_statement(&mut self, it: &mut WhileStatement<'a>) {
        walk_while_statement(self, it);
    }

    #[inline]
    fn visit_with_statement(&mut self, it: &mut WithStatement<'a>) {
        walk_with_statement(self, it);
    }

    #[inline]
    fn visit_declaration(&mut self, it: &mut Declaration<'a>) {
        walk_declaration(self, it);
    }

    #[inline]
    fn visit_ts_type_alias_declaration(&mut self, it: &mut TSTypeAliasDeclaration<'a>) {
        walk_ts_type_alias_declaration(self, it);
    }

    #[inline]
    fn visit_ts_interface_declaration(&mut self, it: &mut TSInterfaceDeclaration<'a>) {
        walk_ts_interface_declaration(self, it);
    }

    #[inline]
    fn visit_ts_interface_heritages(&mut self, it: &mut Vec<'a, TSInterfaceHeritage<'a>>) {
        walk_ts_interface_heritages(self, it);
    }

    #[inline]
    fn visit_ts_interface_heritage(&mut self, it: &mut TSInterfaceHeritage<'a>) {
        walk_ts_interface_heritage(self, it);
    }

    #[inline]
    fn visit_ts_interface_body(&mut self, it: &mut TSInterfaceBody<'a>) {
        walk_ts_interface_body(self, it);
    }

    #[inline]
    fn visit_ts_enum_declaration(&mut self, it: &mut TSEnumDeclaration<'a>) {
        walk_ts_enum_declaration(self, it);
    }

    #[inline]
    fn visit_ts_enum_members(&mut self, it: &mut Vec<'a, TSEnumMember<'a>>) {
        walk_ts_enum_members(self, it);
    }

    #[inline]
    fn visit_ts_enum_member(&mut self, it: &mut TSEnumMember<'a>) {
        walk_ts_enum_member(self, it);
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &mut TSEnumMemberName<'a>) {
        walk_ts_enum_member_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration(&mut self, it: &mut TSModuleDeclaration<'a>) {
        walk_ts_module_declaration(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_name(&mut self, it: &mut TSModuleDeclarationName<'a>) {
        walk_ts_module_declaration_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_body(&mut self, it: &mut TSModuleDeclarationBody<'a>) {
        walk_ts_module_declaration_body(self, it);
    }

    #[inline]
    fn visit_ts_module_block(&mut self, it: &mut TSModuleBlock<'a>) {
        walk_ts_module_block(self, it);
    }

    #[inline]
    fn visit_ts_import_equals_declaration(&mut self, it: &mut TSImportEqualsDeclaration<'a>) {
        walk_ts_import_equals_declaration(self, it);
    }

    #[inline]
    fn visit_ts_module_reference(&mut self, it: &mut TSModuleReference<'a>) {
        walk_ts_module_reference(self, it);
    }

    #[inline]
    fn visit_ts_external_module_reference(&mut self, it: &mut TSExternalModuleReference<'a>) {
        walk_ts_external_module_reference(self, it);
    }

    #[inline]
    fn visit_module_declaration(&mut self, it: &mut ModuleDeclaration<'a>) {
        walk_module_declaration(self, it);
    }

    #[inline]
    fn visit_import_declaration(&mut self, it: &mut ImportDeclaration<'a>) {
        walk_import_declaration(self, it);
    }

    #[inline]
    fn visit_import_declaration_specifiers(
        &mut self,
        it: &mut Vec<'a, ImportDeclarationSpecifier<'a>>,
    ) {
        walk_import_declaration_specifiers(self, it);
    }

    #[inline]
    fn visit_import_declaration_specifier(&mut self, it: &mut ImportDeclarationSpecifier<'a>) {
        walk_import_declaration_specifier(self, it);
    }

    #[inline]
    fn visit_import_specifier(&mut self, it: &mut ImportSpecifier<'a>) {
        walk_import_specifier(self, it);
    }

    #[inline]
    fn visit_module_export_name(&mut self, it: &mut ModuleExportName<'a>) {
        walk_module_export_name(self, it);
    }

    #[inline]
    fn visit_import_default_specifier(&mut self, it: &mut ImportDefaultSpecifier<'a>) {
        walk_import_default_specifier(self, it);
    }

    #[inline]
    fn visit_import_namespace_specifier(&mut self, it: &mut ImportNamespaceSpecifier<'a>) {
        walk_import_namespace_specifier(self, it);
    }

    #[inline]
    fn visit_with_clause(&mut self, it: &mut WithClause<'a>) {
        walk_with_clause(self, it);
    }

    #[inline]
    fn visit_import_attributes(&mut self, it: &mut Vec<'a, ImportAttribute<'a>>) {
        walk_import_attributes(self, it);
    }

    #[inline]
    fn visit_import_attribute(&mut self, it: &mut ImportAttribute<'a>) {
        walk_import_attribute(self, it);
    }

    #[inline]
    fn visit_import_attribute_key(&mut self, it: &mut ImportAttributeKey<'a>) {
        walk_import_attribute_key(self, it);
    }

    #[inline]
    fn visit_export_all_declaration(&mut self, it: &mut ExportAllDeclaration<'a>) {
        walk_export_all_declaration(self, it);
    }

    #[inline]
    fn visit_export_default_declaration(&mut self, it: &mut ExportDefaultDeclaration<'a>) {
        walk_export_default_declaration(self, it);
    }

    #[inline]
    fn visit_export_default_declaration_kind(&mut self, it: &mut ExportDefaultDeclarationKind<'a>) {
        walk_export_default_declaration_kind(self, it);
    }

    #[inline]
    fn visit_export_named_declaration(&mut self, it: &mut ExportNamedDeclaration<'a>) {
        walk_export_named_declaration(self, it);
    }

    #[inline]
    fn visit_export_specifiers(&mut self, it: &mut Vec<'a, ExportSpecifier<'a>>) {
        walk_export_specifiers(self, it);
    }

    #[inline]
    fn visit_export_specifier(&mut self, it: &mut ExportSpecifier<'a>) {
        walk_export_specifier(self, it);
    }

    #[inline]
    fn visit_ts_export_assignment(&mut self, it: &mut TSExportAssignment<'a>) {
        walk_ts_export_assignment(self, it);
    }

    #[inline]
    fn visit_ts_namespace_export_declaration(&mut self, it: &mut TSNamespaceExportDeclaration<'a>) {
        walk_ts_namespace_export_declaration(self, it);
    }
}

pub mod walk_mut {
    use super::*;

    #[inline]
    pub fn walk_program<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Program<'a>) {
        let kind = AstType::Program;
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
        if let Some(hashbang) = &mut it.hashbang {
            visitor.visit_hashbang(hashbang);
        }
        visitor.visit_directives(&mut it.directives);
        visitor.visit_statements(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_hashbang<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Hashbang<'a>) {
        let kind = AstType::Hashbang;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_directives<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Vec<'a, Directive<'a>>) {
        for el in it.iter_mut() {
            visitor.visit_directive(el);
        }
    }

    #[inline]
    pub fn walk_directive<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Directive<'a>) {
        let kind = AstType::Directive;
        visitor.enter_node(kind);
        visitor.visit_string_literal(&mut it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_string_literal<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut StringLiteral<'a>) {
        let kind = AstType::StringLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_statements<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Vec<'a, Statement<'a>>) {
        for el in it.iter_mut() {
            visitor.visit_statement(el);
        }
    }

    pub fn walk_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Statement<'a>) {
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
            match_declaration!(Statement) => visitor.visit_declaration(it.to_declaration_mut()),
            match_module_declaration!(Statement) => {
                visitor.visit_module_declaration(it.to_module_declaration_mut())
            }
        }
    }

    #[inline]
    pub fn walk_block_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut BlockStatement<'a>) {
        let kind = AstType::BlockStatement;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_statements(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_break_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut BreakStatement<'a>) {
        let kind = AstType::BreakStatement;
        visitor.enter_node(kind);
        if let Some(label) = &mut it.label {
            visitor.visit_label_identifier(label);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_label_identifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut LabelIdentifier<'a>,
    ) {
        let kind = AstType::LabelIdentifier;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_continue_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ContinueStatement<'a>,
    ) {
        let kind = AstType::ContinueStatement;
        visitor.enter_node(kind);
        if let Some(label) = &mut it.label {
            visitor.visit_label_identifier(label);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_debugger_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut DebuggerStatement,
    ) {
        let kind = AstType::DebuggerStatement;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_do_while_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut DoWhileStatement<'a>,
    ) {
        let kind = AstType::DoWhileStatement;
        visitor.enter_node(kind);
        visitor.visit_statement(&mut it.body);
        visitor.visit_expression(&mut it.test);
        visitor.leave_node(kind);
    }

    pub fn walk_expression<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Expression<'a>) {
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
                visitor.visit_member_expression(it.to_member_expression_mut())
            }
        }
    }

    #[inline]
    pub fn walk_boolean_literal<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut BooleanLiteral) {
        let kind = AstType::BooleanLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_null_literal<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut NullLiteral) {
        let kind = AstType::NullLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_numeric_literal<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut NumericLiteral<'a>) {
        let kind = AstType::NumericLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_big_int_literal<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut BigIntLiteral<'a>) {
        let kind = AstType::BigIntLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_reg_exp_literal<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut RegExpLiteral<'a>) {
        let kind = AstType::RegExpLiteral;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_template_literal<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TemplateLiteral<'a>,
    ) {
        let kind = AstType::TemplateLiteral;
        visitor.enter_node(kind);
        visitor.visit_template_elements(&mut it.quasis);
        visitor.visit_expressions(&mut it.expressions);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_template_elements<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TemplateElement<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_template_element(el);
        }
    }

    #[inline]
    pub fn walk_template_element<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TemplateElement<'a>,
    ) {
        // NOTE: AstType doesn't exists!
    }

    #[inline]
    pub fn walk_expressions<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, Expression<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_expression(el);
        }
    }

    #[inline]
    pub fn walk_identifier_reference<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut IdentifierReference<'a>,
    ) {
        let kind = AstType::IdentifierReference;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_meta_property<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut MetaProperty<'a>) {
        let kind = AstType::MetaProperty;
        visitor.enter_node(kind);
        visitor.visit_identifier_name(&mut it.meta);
        visitor.visit_identifier_name(&mut it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_identifier_name<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut IdentifierName<'a>) {
        let kind = AstType::IdentifierName;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_super<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Super) {
        let kind = AstType::Super;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ArrayExpression<'a>,
    ) {
        let kind = AstType::ArrayExpression;
        visitor.enter_node(kind);
        visitor.visit_array_expression_elements(&mut it.elements);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_expression_elements<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, ArrayExpressionElement<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_array_expression_element(el);
        }
    }

    #[inline]
    pub fn walk_array_expression_element<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ArrayExpressionElement<'a>,
    ) {
        let kind = AstType::ArrayExpressionElement;
        visitor.enter_node(kind);
        match it {
            ArrayExpressionElement::SpreadElement(it) => visitor.visit_spread_element(it),
            ArrayExpressionElement::Elision(it) => visitor.visit_elision(it),
            match_expression!(ArrayExpressionElement) => {
                visitor.visit_expression_array_element(it.to_expression_mut())
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_spread_element<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut SpreadElement<'a>) {
        let kind = AstType::SpreadElement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_elision<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Elision) {
        let kind = AstType::Elision;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_expression_array_element<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Expression<'a>,
    ) {
        let kind = AstType::ExpressionArrayElement;
        visitor.enter_node(kind);
        visitor.visit_expression(it);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_arrow_function_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ArrowFunctionExpression<'a>,
    ) {
        let kind = AstType::ArrowFunctionExpression;
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
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.visit_formal_parameters(&mut it.params);
        if let Some(return_type) = &mut it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        visitor.visit_function_body(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_parameter_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeParameterDeclaration<'a>,
    ) {
        let kind = AstType::TSTypeParameterDeclaration;
        visitor.enter_node(kind);
        visitor.visit_ts_type_parameters(&mut it.params);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_parameters<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TSTypeParameter<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_ts_type_parameter(el);
        }
    }

    #[inline]
    pub fn walk_ts_type_parameter<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeParameter<'a>,
    ) {
        let kind = AstType::TSTypeParameter;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut it.name);
        if let Some(constraint) = &mut it.constraint {
            visitor.visit_ts_type(constraint);
        }
        if let Some(default) = &mut it.default {
            visitor.visit_ts_type(default);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_identifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut BindingIdentifier<'a>,
    ) {
        let kind = AstType::BindingIdentifier;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSType<'a>) {
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
    pub fn walk_ts_any_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSAnyKeyword) {
        let kind = AstType::TSAnyKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_big_int_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSBigIntKeyword) {
        let kind = AstType::TSBigIntKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_boolean_keyword<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSBooleanKeyword,
    ) {
        let kind = AstType::TSBooleanKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_intrinsic_keyword<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSIntrinsicKeyword,
    ) {
        let kind = AstType::TSIntrinsicKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_never_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSNeverKeyword) {
        let kind = AstType::TSNeverKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_null_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSNullKeyword) {
        let kind = AstType::TSNullKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_number_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSNumberKeyword) {
        let kind = AstType::TSNumberKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_object_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSObjectKeyword) {
        let kind = AstType::TSObjectKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_string_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSStringKeyword) {
        let kind = AstType::TSStringKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_symbol_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSSymbolKeyword) {
        let kind = AstType::TSSymbolKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_undefined_keyword<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSUndefinedKeyword,
    ) {
        let kind = AstType::TSUndefinedKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_unknown_keyword<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSUnknownKeyword,
    ) {
        let kind = AstType::TSUnknownKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_void_keyword<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSVoidKeyword) {
        let kind = AstType::TSVoidKeyword;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_array_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSArrayType<'a>) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_type(&mut it.element_type);
    }

    #[inline]
    pub fn walk_ts_conditional_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSConditionalType<'a>,
    ) {
        let kind = AstType::TSConditionalType;
        visitor.enter_node(kind);
        visitor.visit_ts_type(&mut it.check_type);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_ts_type(&mut it.extends_type);
        visitor.visit_ts_type(&mut it.true_type);
        visitor.leave_scope();
        visitor.visit_ts_type(&mut it.false_type);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_constructor_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSConstructorType<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_formal_parameters(&mut it.params);
        visitor.visit_ts_type_annotation(&mut it.return_type);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
    }

    #[inline]
    pub fn walk_formal_parameters<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut FormalParameters<'a>,
    ) {
        let kind = AstType::FormalParameters;
        visitor.enter_node(kind);
        visitor.visit_formal_parameter_list(&mut it.items);
        if let Some(rest) = &mut it.rest {
            visitor.visit_binding_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_formal_parameter_list<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, FormalParameter<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_formal_parameter(el);
        }
    }

    #[inline]
    pub fn walk_formal_parameter<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut FormalParameter<'a>,
    ) {
        let kind = AstType::FormalParameter;
        visitor.enter_node(kind);
        visitor.visit_decorators(&mut it.decorators);
        visitor.visit_binding_pattern(&mut it.pattern);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_decorators<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Vec<'a, Decorator<'a>>) {
        for el in it.iter_mut() {
            visitor.visit_decorator(el);
        }
    }

    #[inline]
    pub fn walk_decorator<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Decorator<'a>) {
        let kind = AstType::Decorator;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_pattern<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut BindingPattern<'a>) {
        // NOTE: AstType doesn't exists!
        visitor.visit_binding_pattern_kind(&mut it.kind);
        if let Some(type_annotation) = &mut it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    pub fn walk_binding_pattern_kind<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut BindingPatternKind<'a>,
    ) {
        match it {
            BindingPatternKind::BindingIdentifier(it) => visitor.visit_binding_identifier(it),
            BindingPatternKind::ObjectPattern(it) => visitor.visit_object_pattern(it),
            BindingPatternKind::ArrayPattern(it) => visitor.visit_array_pattern(it),
            BindingPatternKind::AssignmentPattern(it) => visitor.visit_assignment_pattern(it),
        }
    }

    #[inline]
    pub fn walk_object_pattern<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ObjectPattern<'a>) {
        let kind = AstType::ObjectPattern;
        visitor.enter_node(kind);
        visitor.visit_binding_properties(&mut it.properties);
        if let Some(rest) = &mut it.rest {
            visitor.visit_binding_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_properties<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, BindingProperty<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_binding_property(el);
        }
    }

    #[inline]
    pub fn walk_binding_property<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut BindingProperty<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_property_key(&mut it.key);
        visitor.visit_binding_pattern(&mut it.value);
    }

    #[inline]
    pub fn walk_property_key<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut PropertyKey<'a>) {
        let kind = AstType::PropertyKey;
        visitor.enter_node(kind);
        match it {
            PropertyKey::StaticIdentifier(it) => visitor.visit_identifier_name(it),
            PropertyKey::PrivateIdentifier(it) => visitor.visit_private_identifier(it),
            match_expression!(PropertyKey) => visitor.visit_expression(it.to_expression_mut()),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_private_identifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut PrivateIdentifier<'a>,
    ) {
        let kind = AstType::PrivateIdentifier;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binding_rest_element<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut BindingRestElement<'a>,
    ) {
        let kind = AstType::BindingRestElement;
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&mut it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_array_pattern<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ArrayPattern<'a>) {
        let kind = AstType::ArrayPattern;
        visitor.enter_node(kind);
        for elements in it.elements.iter_mut().flatten() {
            visitor.visit_binding_pattern(elements);
        }
        if let Some(rest) = &mut it.rest {
            visitor.visit_binding_rest_element(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_pattern<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentPattern<'a>,
    ) {
        let kind = AstType::AssignmentPattern;
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&mut it.left);
        visitor.visit_expression(&mut it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_annotation<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeAnnotation<'a>,
    ) {
        let kind = AstType::TSTypeAnnotation;
        visitor.enter_node(kind);
        visitor.visit_ts_type(&mut it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_function_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSFunctionType<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        if let Some(this_param) = &mut it.this_param {
            visitor.visit_ts_this_parameter(this_param);
        }
        visitor.visit_formal_parameters(&mut it.params);
        visitor.visit_ts_type_annotation(&mut it.return_type);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
    }

    #[inline]
    pub fn walk_ts_this_parameter<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSThisParameter<'a>,
    ) {
        let kind = AstType::TSThisParameter;
        visitor.enter_node(kind);
        if let Some(type_annotation) = &mut it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_import_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSImportType<'a>) {
        let kind = AstType::TSImportType;
        visitor.enter_node(kind);
        visitor.visit_ts_type(&mut it.parameter);
        if let Some(qualifier) = &mut it.qualifier {
            visitor.visit_ts_type_name(qualifier);
        }
        if let Some(attributes) = &mut it.attributes {
            visitor.visit_ts_import_attributes(attributes);
        }
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_name<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSTypeName<'a>) {
        let kind = AstType::TSTypeName;
        visitor.enter_node(kind);
        match it {
            TSTypeName::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            TSTypeName::QualifiedName(it) => visitor.visit_ts_qualified_name(it),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_qualified_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSQualifiedName<'a>,
    ) {
        let kind = AstType::TSQualifiedName;
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&mut it.left);
        visitor.visit_identifier_name(&mut it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_import_attributes<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSImportAttributes<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_identifier_name(&mut it.attributes_keyword);
        visitor.visit_ts_import_attribute_list(&mut it.elements);
    }

    #[inline]
    pub fn walk_ts_import_attribute_list<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TSImportAttribute<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_ts_import_attribute(el);
        }
    }

    #[inline]
    pub fn walk_ts_import_attribute<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSImportAttribute<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_import_attribute_name(&mut it.name);
        visitor.visit_expression(&mut it.value);
    }

    #[inline]
    pub fn walk_ts_import_attribute_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSImportAttributeName<'a>,
    ) {
        match it {
            TSImportAttributeName::Identifier(it) => visitor.visit_identifier_name(it),
            TSImportAttributeName::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_ts_type_parameter_instantiation<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeParameterInstantiation<'a>,
    ) {
        let kind = AstType::TSTypeParameterInstantiation;
        visitor.enter_node(kind);
        visitor.visit_ts_types(&mut it.params);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_types<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Vec<'a, TSType<'a>>) {
        for el in it.iter_mut() {
            visitor.visit_ts_type(el);
        }
    }

    #[inline]
    pub fn walk_ts_indexed_access_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSIndexedAccessType<'a>,
    ) {
        let kind = AstType::TSIndexedAccessType;
        visitor.enter_node(kind);
        visitor.visit_ts_type(&mut it.object_type);
        visitor.visit_ts_type(&mut it.index_type);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_infer_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSInferType<'a>) {
        let kind = AstType::TSInferType;
        visitor.enter_node(kind);
        visitor.visit_ts_type_parameter(&mut it.type_parameter);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_intersection_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSIntersectionType<'a>,
    ) {
        let kind = AstType::TSIntersectionType;
        visitor.enter_node(kind);
        visitor.visit_ts_types(&mut it.types);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_literal_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSLiteralType<'a>) {
        let kind = AstType::TSLiteralType;
        visitor.enter_node(kind);
        visitor.visit_ts_literal(&mut it.literal);
        visitor.leave_node(kind);
    }

    pub fn walk_ts_literal<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSLiteral<'a>) {
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
    pub fn walk_unary_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut UnaryExpression<'a>,
    ) {
        let kind = AstType::UnaryExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_mapped_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSMappedType<'a>) {
        let kind = AstType::TSMappedType;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_ts_type_parameter(&mut it.type_parameter);
        if let Some(name_type) = &mut it.name_type {
            visitor.visit_ts_type(name_type);
        }
        if let Some(type_annotation) = &mut it.type_annotation {
            visitor.visit_ts_type(type_annotation);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_named_tuple_member<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSNamedTupleMember<'a>,
    ) {
        let kind = AstType::TSNamedTupleMember;
        visitor.enter_node(kind);
        visitor.visit_ts_tuple_element(&mut it.element_type);
        visitor.visit_identifier_name(&mut it.label);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_tuple_element<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTupleElement<'a>,
    ) {
        match it {
            TSTupleElement::TSOptionalType(it) => visitor.visit_ts_optional_type(it),
            TSTupleElement::TSRestType(it) => visitor.visit_ts_rest_type(it),
            match_ts_type!(TSTupleElement) => visitor.visit_ts_type(it.to_ts_type_mut()),
        }
    }

    #[inline]
    pub fn walk_ts_optional_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSOptionalType<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_type(&mut it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_rest_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSRestType<'a>) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_type(&mut it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_template_literal_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTemplateLiteralType<'a>,
    ) {
        let kind = AstType::TSTemplateLiteralType;
        visitor.enter_node(kind);
        visitor.visit_template_elements(&mut it.quasis);
        visitor.visit_ts_types(&mut it.types);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_this_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSThisType) {
        let kind = AstType::TSThisType;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_tuple_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSTupleType<'a>) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_tuple_elements(&mut it.element_types);
    }

    #[inline]
    pub fn walk_ts_tuple_elements<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TSTupleElement<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_ts_tuple_element(el);
        }
    }

    #[inline]
    pub fn walk_ts_type_literal<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSTypeLiteral<'a>) {
        let kind = AstType::TSTypeLiteral;
        visitor.enter_node(kind);
        visitor.visit_ts_signatures(&mut it.members);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_signatures<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TSSignature<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_ts_signature(el);
        }
    }

    #[inline]
    pub fn walk_ts_signature<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSSignature<'a>) {
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
    pub fn walk_ts_index_signature<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSIndexSignature<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_index_signature_names(&mut it.parameters);
        visitor.visit_ts_type_annotation(&mut it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_index_signature_names<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TSIndexSignatureName<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_ts_index_signature_name(el);
        }
    }

    #[inline]
    pub fn walk_ts_index_signature_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSIndexSignatureName<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_type_annotation(&mut it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_property_signature<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSPropertySignature<'a>,
    ) {
        let kind = AstType::TSPropertySignature;
        visitor.enter_node(kind);
        visitor.visit_property_key(&mut it.key);
        if let Some(type_annotation) = &mut it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_call_signature_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSCallSignatureDeclaration<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        if let Some(this_param) = &mut it.this_param {
            visitor.visit_ts_this_parameter(this_param);
        }
        visitor.visit_formal_parameters(&mut it.params);
        if let Some(return_type) = &mut it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
    }

    #[inline]
    pub fn walk_ts_construct_signature_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSConstructSignatureDeclaration<'a>,
    ) {
        let kind = AstType::TSConstructSignatureDeclaration;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_formal_parameters(&mut it.params);
        if let Some(return_type) = &mut it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_method_signature<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSMethodSignature<'a>,
    ) {
        let kind = AstType::TSMethodSignature;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_property_key(&mut it.key);
        if let Some(this_param) = &mut it.this_param {
            visitor.visit_ts_this_parameter(this_param);
        }
        visitor.visit_formal_parameters(&mut it.params);
        if let Some(return_type) = &mut it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_operator<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeOperator<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_type(&mut it.type_annotation);
    }

    #[inline]
    pub fn walk_ts_type_predicate<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypePredicate<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_type_predicate_name(&mut it.parameter_name);
        if let Some(type_annotation) = &mut it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    pub fn walk_ts_type_predicate_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypePredicateName<'a>,
    ) {
        match it {
            TSTypePredicateName::Identifier(it) => visitor.visit_identifier_name(it),
            TSTypePredicateName::This(it) => visitor.visit_ts_this_type(it),
        }
    }

    #[inline]
    pub fn walk_ts_type_query<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSTypeQuery<'a>) {
        let kind = AstType::TSTypeQuery;
        visitor.enter_node(kind);
        visitor.visit_ts_type_query_expr_name(&mut it.expr_name);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_query_expr_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeQueryExprName<'a>,
    ) {
        match it {
            TSTypeQueryExprName::TSImportType(it) => visitor.visit_ts_import_type(it),
            match_ts_type_name!(TSTypeQueryExprName) => {
                visitor.visit_ts_type_name(it.to_ts_type_name_mut())
            }
        }
    }

    #[inline]
    pub fn walk_ts_type_reference<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeReference<'a>,
    ) {
        let kind = AstType::TSTypeReference;
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&mut it.type_name);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_union_type<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSUnionType<'a>) {
        let kind = AstType::TSUnionType;
        visitor.enter_node(kind);
        visitor.visit_ts_types(&mut it.types);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_parenthesized_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSParenthesizedType<'a>,
    ) {
        let kind = AstType::TSParenthesizedType;
        visitor.enter_node(kind);
        visitor.visit_ts_type(&mut it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_js_doc_nullable_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSDocNullableType<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_type(&mut it.type_annotation);
    }

    #[inline]
    pub fn walk_js_doc_non_nullable_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSDocNonNullableType<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_type(&mut it.type_annotation);
    }

    #[inline]
    pub fn walk_js_doc_unknown_type<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSDocUnknownType,
    ) {
        // NOTE: AstType doesn't exists!
    }

    #[inline]
    pub fn walk_function_body<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut FunctionBody<'a>) {
        let kind = AstType::FunctionBody;
        visitor.enter_node(kind);
        visitor.visit_directives(&mut it.directives);
        visitor.visit_statements(&mut it.statements);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentExpression<'a>,
    ) {
        let kind = AstType::AssignmentExpression;
        visitor.enter_node(kind);
        visitor.visit_assignment_target(&mut it.left);
        visitor.visit_expression(&mut it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentTarget<'a>,
    ) {
        let kind = AstType::AssignmentTarget;
        visitor.enter_node(kind);
        match it {
            match_simple_assignment_target!(AssignmentTarget) => {
                visitor.visit_simple_assignment_target(it.to_simple_assignment_target_mut())
            }
            match_assignment_target_pattern!(AssignmentTarget) => {
                visitor.visit_assignment_target_pattern(it.to_assignment_target_pattern_mut())
            }
        }
        visitor.leave_node(kind);
    }

    pub fn walk_simple_assignment_target<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut SimpleAssignmentTarget<'a>,
    ) {
        let kind = AstType::SimpleAssignmentTarget;
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
                visitor.visit_member_expression(it.to_member_expression_mut())
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_as_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSAsExpression<'a>,
    ) {
        let kind = AstType::TSAsExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.visit_ts_type(&mut it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_satisfies_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSSatisfiesExpression<'a>,
    ) {
        let kind = AstType::TSSatisfiesExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.visit_ts_type(&mut it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_non_null_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSNonNullExpression<'a>,
    ) {
        let kind = AstType::TSNonNullExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_type_assertion<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeAssertion<'a>,
    ) {
        let kind = AstType::TSTypeAssertion;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.visit_ts_type(&mut it.type_annotation);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_instantiation_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSInstantiationExpression<'a>,
    ) {
        let kind = AstType::TSInstantiationExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.visit_ts_type_parameter_instantiation(&mut it.type_parameters);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_member_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut MemberExpression<'a>,
    ) {
        let kind = AstType::MemberExpression;
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
    pub fn walk_computed_member_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ComputedMemberExpression<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_expression(&mut it.object);
        visitor.visit_expression(&mut it.expression);
    }

    #[inline]
    pub fn walk_static_member_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut StaticMemberExpression<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_expression(&mut it.object);
        visitor.visit_identifier_name(&mut it.property);
    }

    #[inline]
    pub fn walk_private_field_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut PrivateFieldExpression<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_expression(&mut it.object);
        visitor.visit_private_identifier(&mut it.field);
    }

    #[inline]
    pub fn walk_assignment_target_pattern<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentTargetPattern<'a>,
    ) {
        let kind = AstType::AssignmentTargetPattern;
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
    pub fn walk_array_assignment_target<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ArrayAssignmentTarget<'a>,
    ) {
        let kind = AstType::ArrayAssignmentTarget;
        visitor.enter_node(kind);
        for elements in it.elements.iter_mut().flatten() {
            visitor.visit_assignment_target_maybe_default(elements);
        }
        if let Some(rest) = &mut it.rest {
            visitor.visit_assignment_target_rest(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_maybe_default<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentTargetMaybeDefault<'a>,
    ) {
        match it {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(it) => {
                visitor.visit_assignment_target_with_default(it)
            }
            match_assignment_target!(AssignmentTargetMaybeDefault) => {
                visitor.visit_assignment_target(it.to_assignment_target_mut())
            }
        }
    }

    #[inline]
    pub fn walk_assignment_target_with_default<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentTargetWithDefault<'a>,
    ) {
        let kind = AstType::AssignmentTargetWithDefault;
        visitor.enter_node(kind);
        visitor.visit_assignment_target(&mut it.binding);
        visitor.visit_expression(&mut it.init);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_rest<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentTargetRest<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_assignment_target(&mut it.target);
    }

    #[inline]
    pub fn walk_object_assignment_target<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ObjectAssignmentTarget<'a>,
    ) {
        let kind = AstType::ObjectAssignmentTarget;
        visitor.enter_node(kind);
        visitor.visit_assignment_target_properties(&mut it.properties);
        if let Some(rest) = &mut it.rest {
            visitor.visit_assignment_target_rest(rest);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_assignment_target_properties<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, AssignmentTargetProperty<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_assignment_target_property(el);
        }
    }

    #[inline]
    pub fn walk_assignment_target_property<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentTargetProperty<'a>,
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
    pub fn walk_assignment_target_property_identifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentTargetPropertyIdentifier<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_identifier_reference(&mut it.binding);
        if let Some(init) = &mut it.init {
            visitor.visit_expression(init);
        }
    }

    #[inline]
    pub fn walk_assignment_target_property_property<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_property_key(&mut it.name);
        visitor.visit_assignment_target_maybe_default(&mut it.binding);
    }

    #[inline]
    pub fn walk_await_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AwaitExpression<'a>,
    ) {
        let kind = AstType::AwaitExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_binary_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut BinaryExpression<'a>,
    ) {
        let kind = AstType::BinaryExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.left);
        visitor.visit_expression(&mut it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_call_expression<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut CallExpression<'a>) {
        let kind = AstType::CallExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.callee);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.visit_arguments(&mut it.arguments);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_arguments<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Vec<'a, Argument<'a>>) {
        for el in it.iter_mut() {
            visitor.visit_argument(el);
        }
    }

    #[inline]
    pub fn walk_argument<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Argument<'a>) {
        let kind = AstType::Argument;
        visitor.enter_node(kind);
        match it {
            Argument::SpreadElement(it) => visitor.visit_spread_element(it),
            match_expression!(Argument) => visitor.visit_expression(it.to_expression_mut()),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_chain_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ChainExpression<'a>,
    ) {
        let kind = AstType::ChainExpression;
        visitor.enter_node(kind);
        visitor.visit_chain_element(&mut it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_chain_element<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ChainElement<'a>) {
        match it {
            ChainElement::CallExpression(it) => visitor.visit_call_expression(it),
            match_member_expression!(ChainElement) => {
                visitor.visit_member_expression(it.to_member_expression_mut())
            }
        }
    }

    pub fn walk_class<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Class<'a>) {
        let kind = AstType::Class;
        visitor.enter_node(kind);
        visitor.visit_decorators(&mut it.decorators);
        if let Some(id) = &mut it.id {
            visitor.visit_binding_identifier(id);
        }
        visitor.enter_scope(ScopeFlags::StrictMode, &it.scope_id);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(super_class) = &mut it.super_class {
            visitor.visit_class_heritage(super_class);
        }
        if let Some(super_type_parameters) = &mut it.super_type_parameters {
            visitor.visit_ts_type_parameter_instantiation(super_type_parameters);
        }
        if let Some(implements) = &mut it.implements {
            visitor.visit_ts_class_implementses(implements);
        }
        visitor.visit_class_body(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    pub fn walk_class_heritage<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Expression<'a>) {
        let kind = AstType::ClassHeritage;
        visitor.enter_node(kind);
        visitor.visit_expression(it);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_class_implementses<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TSClassImplements<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_ts_class_implements(el);
        }
    }

    #[inline]
    pub fn walk_ts_class_implements<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSClassImplements<'a>,
    ) {
        let kind = AstType::TSClassImplements;
        visitor.enter_node(kind);
        visitor.visit_ts_type_name(&mut it.expression);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_class_body<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ClassBody<'a>) {
        let kind = AstType::ClassBody;
        visitor.enter_node(kind);
        visitor.visit_class_elements(&mut it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_class_elements<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, ClassElement<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_class_element(el);
        }
    }

    #[inline]
    pub fn walk_class_element<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ClassElement<'a>) {
        match it {
            ClassElement::StaticBlock(it) => visitor.visit_static_block(it),
            ClassElement::MethodDefinition(it) => visitor.visit_method_definition(it),
            ClassElement::PropertyDefinition(it) => visitor.visit_property_definition(it),
            ClassElement::AccessorProperty(it) => visitor.visit_accessor_property(it),
            ClassElement::TSIndexSignature(it) => visitor.visit_ts_index_signature(it),
        }
    }

    #[inline]
    pub fn walk_static_block<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut StaticBlock<'a>) {
        let kind = AstType::StaticBlock;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::ClassStaticBlock, &it.scope_id);
        visitor.visit_statements(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_method_definition<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut MethodDefinition<'a>,
    ) {
        let kind = AstType::MethodDefinition;
        visitor.enter_node(kind);
        visitor.visit_decorators(&mut it.decorators);
        visitor.visit_property_key(&mut it.key);
        {
            let flags = match it.kind {
                MethodDefinitionKind::Get => ScopeFlags::Function | ScopeFlags::GetAccessor,
                MethodDefinitionKind::Set => ScopeFlags::Function | ScopeFlags::SetAccessor,
                MethodDefinitionKind::Constructor => ScopeFlags::Function | ScopeFlags::Constructor,
                MethodDefinitionKind::Method => ScopeFlags::Function,
            };
            visitor.visit_function(&mut it.value, flags);
        }
        visitor.leave_node(kind);
    }

    pub fn walk_function<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Function<'a>,
        flags: ScopeFlags,
    ) {
        let kind = AstType::Function;
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
        if let Some(id) = &mut it.id {
            visitor.visit_binding_identifier(id);
        }
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(this_param) = &mut it.this_param {
            visitor.visit_ts_this_parameter(this_param);
        }
        visitor.visit_formal_parameters(&mut it.params);
        if let Some(return_type) = &mut it.return_type {
            visitor.visit_ts_type_annotation(return_type);
        }
        if let Some(body) = &mut it.body {
            visitor.visit_function_body(body);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_property_definition<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut PropertyDefinition<'a>,
    ) {
        let kind = AstType::PropertyDefinition;
        visitor.enter_node(kind);
        visitor.visit_decorators(&mut it.decorators);
        visitor.visit_property_key(&mut it.key);
        if let Some(value) = &mut it.value {
            visitor.visit_expression(value);
        }
        if let Some(type_annotation) = &mut it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_accessor_property<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut AccessorProperty<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_decorators(&mut it.decorators);
        visitor.visit_property_key(&mut it.key);
        if let Some(value) = &mut it.value {
            visitor.visit_expression(value);
        }
        if let Some(type_annotation) = &mut it.type_annotation {
            visitor.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    pub fn walk_conditional_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ConditionalExpression<'a>,
    ) {
        let kind = AstType::ConditionalExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.test);
        visitor.visit_expression(&mut it.consequent);
        visitor.visit_expression(&mut it.alternate);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ImportExpression<'a>,
    ) {
        let kind = AstType::ImportExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.source);
        visitor.visit_expressions(&mut it.arguments);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_logical_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut LogicalExpression<'a>,
    ) {
        let kind = AstType::LogicalExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.left);
        visitor.visit_expression(&mut it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_new_expression<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut NewExpression<'a>) {
        let kind = AstType::NewExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.callee);
        visitor.visit_arguments(&mut it.arguments);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_object_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ObjectExpression<'a>,
    ) {
        let kind = AstType::ObjectExpression;
        visitor.enter_node(kind);
        visitor.visit_object_property_kinds(&mut it.properties);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_object_property_kinds<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, ObjectPropertyKind<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_object_property_kind(el);
        }
    }

    #[inline]
    pub fn walk_object_property_kind<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ObjectPropertyKind<'a>,
    ) {
        match it {
            ObjectPropertyKind::ObjectProperty(it) => visitor.visit_object_property(it),
            ObjectPropertyKind::SpreadProperty(it) => visitor.visit_spread_element(it),
        }
    }

    #[inline]
    pub fn walk_object_property<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ObjectProperty<'a>) {
        let kind = AstType::ObjectProperty;
        visitor.enter_node(kind);
        visitor.visit_property_key(&mut it.key);
        visitor.visit_expression(&mut it.value);
        if let Some(init) = &mut it.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_parenthesized_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ParenthesizedExpression<'a>,
    ) {
        let kind = AstType::ParenthesizedExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_sequence_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut SequenceExpression<'a>,
    ) {
        let kind = AstType::SequenceExpression;
        visitor.enter_node(kind);
        visitor.visit_expressions(&mut it.expressions);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_tagged_template_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TaggedTemplateExpression<'a>,
    ) {
        let kind = AstType::TaggedTemplateExpression;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.tag);
        visitor.visit_template_literal(&mut it.quasi);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_this_expression<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ThisExpression) {
        let kind = AstType::ThisExpression;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_update_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut UpdateExpression<'a>,
    ) {
        let kind = AstType::UpdateExpression;
        visitor.enter_node(kind);
        visitor.visit_simple_assignment_target(&mut it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_yield_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut YieldExpression<'a>,
    ) {
        let kind = AstType::YieldExpression;
        visitor.enter_node(kind);
        if let Some(argument) = &mut it.argument {
            visitor.visit_expression(argument);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_private_in_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut PrivateInExpression<'a>,
    ) {
        let kind = AstType::PrivateInExpression;
        visitor.enter_node(kind);
        visitor.visit_private_identifier(&mut it.left);
        visitor.visit_expression(&mut it.right);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_element<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut JSXElement<'a>) {
        let kind = AstType::JSXElement;
        visitor.enter_node(kind);
        visitor.visit_jsx_opening_element(&mut it.opening_element);
        if let Some(closing_element) = &mut it.closing_element {
            visitor.visit_jsx_closing_element(closing_element);
        }
        visitor.visit_jsx_children(&mut it.children);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_opening_element<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXOpeningElement<'a>,
    ) {
        let kind = AstType::JSXOpeningElement;
        visitor.enter_node(kind);
        visitor.visit_jsx_element_name(&mut it.name);
        visitor.visit_jsx_attribute_items(&mut it.attributes);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_element_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXElementName<'a>,
    ) {
        let kind = AstType::JSXElementName;
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
    pub fn walk_jsx_identifier<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut JSXIdentifier<'a>) {
        let kind = AstType::JSXIdentifier;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_namespaced_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXNamespacedName<'a>,
    ) {
        let kind = AstType::JSXNamespacedName;
        visitor.enter_node(kind);
        visitor.visit_jsx_identifier(&mut it.namespace);
        visitor.visit_jsx_identifier(&mut it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_member_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXMemberExpression<'a>,
    ) {
        let kind = AstType::JSXMemberExpression;
        visitor.enter_node(kind);
        visitor.visit_jsx_member_expression_object(&mut it.object);
        visitor.visit_jsx_identifier(&mut it.property);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_member_expression_object<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXMemberExpressionObject<'a>,
    ) {
        let kind = AstType::JSXMemberExpressionObject;
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
    pub fn walk_jsx_attribute_items<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, JSXAttributeItem<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_jsx_attribute_item(el);
        }
    }

    #[inline]
    pub fn walk_jsx_attribute_item<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXAttributeItem<'a>,
    ) {
        let kind = AstType::JSXAttributeItem;
        visitor.enter_node(kind);
        match it {
            JSXAttributeItem::Attribute(it) => visitor.visit_jsx_attribute(it),
            JSXAttributeItem::SpreadAttribute(it) => visitor.visit_jsx_spread_attribute(it),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_attribute<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut JSXAttribute<'a>) {
        // NOTE: AstType doesn't exists!
        visitor.visit_jsx_attribute_name(&mut it.name);
        if let Some(value) = &mut it.value {
            visitor.visit_jsx_attribute_value(value);
        }
    }

    #[inline]
    pub fn walk_jsx_attribute_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXAttributeName<'a>,
    ) {
        match it {
            JSXAttributeName::Identifier(it) => visitor.visit_jsx_identifier(it),
            JSXAttributeName::NamespacedName(it) => visitor.visit_jsx_namespaced_name(it),
        }
    }

    #[inline]
    pub fn walk_jsx_attribute_value<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXAttributeValue<'a>,
    ) {
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
    pub fn walk_jsx_expression_container<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXExpressionContainer<'a>,
    ) {
        let kind = AstType::JSXExpressionContainer;
        visitor.enter_node(kind);
        visitor.visit_jsx_expression(&mut it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_expression<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut JSXExpression<'a>) {
        match it {
            JSXExpression::EmptyExpression(it) => visitor.visit_jsx_empty_expression(it),
            match_expression!(JSXExpression) => visitor.visit_expression(it.to_expression_mut()),
        }
    }

    #[inline]
    pub fn walk_jsx_empty_expression<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXEmptyExpression,
    ) {
        // NOTE: AstType doesn't exists!
    }

    #[inline]
    pub fn walk_jsx_fragment<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut JSXFragment<'a>) {
        let kind = AstType::JSXFragment;
        visitor.enter_node(kind);
        visitor.visit_jsx_children(&mut it.children);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_children<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Vec<'a, JSXChild<'a>>) {
        for el in it.iter_mut() {
            visitor.visit_jsx_child(el);
        }
    }

    #[inline]
    pub fn walk_jsx_child<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut JSXChild<'a>) {
        match it {
            JSXChild::Text(it) => visitor.visit_jsx_text(it),
            JSXChild::Element(it) => visitor.visit_jsx_element(it),
            JSXChild::Fragment(it) => visitor.visit_jsx_fragment(it),
            JSXChild::ExpressionContainer(it) => visitor.visit_jsx_expression_container(it),
            JSXChild::Spread(it) => visitor.visit_jsx_spread_child(it),
        }
    }

    #[inline]
    pub fn walk_jsx_text<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut JSXText<'a>) {
        let kind = AstType::JSXText;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_spread_child<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXSpreadChild<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_expression(&mut it.expression);
    }

    #[inline]
    pub fn walk_jsx_spread_attribute<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXSpreadAttribute<'a>,
    ) {
        let kind = AstType::JSXSpreadAttribute;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_jsx_closing_element<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut JSXClosingElement<'a>,
    ) {
        let kind = AstType::JSXClosingElement;
        visitor.enter_node(kind);
        visitor.visit_jsx_element_name(&mut it.name);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_empty_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut EmptyStatement) {
        let kind = AstType::EmptyStatement;
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_expression_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ExpressionStatement<'a>,
    ) {
        let kind = AstType::ExpressionStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_in_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ForInStatement<'a>,
    ) {
        let kind = AstType::ForInStatement;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_for_statement_left(&mut it.left);
        visitor.visit_expression(&mut it.right);
        visitor.visit_statement(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_statement_left<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ForStatementLeft<'a>,
    ) {
        match it {
            ForStatementLeft::VariableDeclaration(it) => visitor.visit_variable_declaration(it),
            match_assignment_target!(ForStatementLeft) => {
                visitor.visit_assignment_target(it.to_assignment_target_mut())
            }
        }
    }

    #[inline]
    pub fn walk_variable_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut VariableDeclaration<'a>,
    ) {
        let kind = AstType::VariableDeclaration;
        visitor.enter_node(kind);
        visitor.visit_variable_declarators(&mut it.declarations);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_variable_declarators<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, VariableDeclarator<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_variable_declarator(el);
        }
    }

    #[inline]
    pub fn walk_variable_declarator<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut VariableDeclarator<'a>,
    ) {
        let kind = AstType::VariableDeclarator;
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&mut it.id);
        if let Some(init) = &mut it.init {
            visitor.visit_expression(init);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_of_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ForOfStatement<'a>,
    ) {
        let kind = AstType::ForOfStatement;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_for_statement_left(&mut it.left);
        visitor.visit_expression(&mut it.right);
        visitor.visit_statement(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ForStatement<'a>) {
        let kind = AstType::ForStatement;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        if let Some(init) = &mut it.init {
            visitor.visit_for_statement_init(init);
        }
        if let Some(test) = &mut it.test {
            visitor.visit_expression(test);
        }
        if let Some(update) = &mut it.update {
            visitor.visit_expression(update);
        }
        visitor.visit_statement(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_for_statement_init<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ForStatementInit<'a>,
    ) {
        let kind = AstType::ForStatementInit;
        visitor.enter_node(kind);
        match it {
            ForStatementInit::VariableDeclaration(it) => visitor.visit_variable_declaration(it),
            match_expression!(ForStatementInit) => visitor.visit_expression(it.to_expression_mut()),
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_if_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut IfStatement<'a>) {
        let kind = AstType::IfStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.test);
        visitor.visit_statement(&mut it.consequent);
        if let Some(alternate) = &mut it.alternate {
            visitor.visit_statement(alternate);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_labeled_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut LabeledStatement<'a>,
    ) {
        let kind = AstType::LabeledStatement;
        visitor.enter_node(kind);
        visitor.visit_label_identifier(&mut it.label);
        visitor.visit_statement(&mut it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_return_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ReturnStatement<'a>,
    ) {
        let kind = AstType::ReturnStatement;
        visitor.enter_node(kind);
        if let Some(argument) = &mut it.argument {
            visitor.visit_expression(argument);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_switch_statement<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut SwitchStatement<'a>,
    ) {
        let kind = AstType::SwitchStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.discriminant);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_switch_cases(&mut it.cases);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_switch_cases<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, SwitchCase<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_switch_case(el);
        }
    }

    #[inline]
    pub fn walk_switch_case<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut SwitchCase<'a>) {
        let kind = AstType::SwitchCase;
        visitor.enter_node(kind);
        if let Some(test) = &mut it.test {
            visitor.visit_expression(test);
        }
        visitor.visit_statements(&mut it.consequent);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_throw_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut ThrowStatement<'a>) {
        let kind = AstType::ThrowStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.argument);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_try_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TryStatement<'a>) {
        let kind = AstType::TryStatement;
        visitor.enter_node(kind);
        visitor.visit_block_statement(&mut it.block);
        if let Some(handler) = &mut it.handler {
            visitor.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &mut it.finalizer {
            visitor.visit_finally_clause(finalizer);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_catch_clause<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut CatchClause<'a>) {
        let kind = AstType::CatchClause;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::CatchClause, &it.scope_id);
        if let Some(param) = &mut it.param {
            visitor.visit_catch_parameter(param);
        }
        visitor.visit_block_statement(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_catch_parameter<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut CatchParameter<'a>) {
        let kind = AstType::CatchParameter;
        visitor.enter_node(kind);
        visitor.visit_binding_pattern(&mut it.pattern);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_finally_clause<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut BlockStatement<'a>) {
        let kind = AstType::FinallyClause;
        visitor.enter_node(kind);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_statements(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_while_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut WhileStatement<'a>) {
        let kind = AstType::WhileStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.test);
        visitor.visit_statement(&mut it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_with_statement<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut WithStatement<'a>) {
        let kind = AstType::WithStatement;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.object);
        visitor.visit_statement(&mut it.body);
        visitor.leave_node(kind);
    }

    pub fn walk_declaration<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut Declaration<'a>) {
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
    pub fn walk_ts_type_alias_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSTypeAliasDeclaration<'a>,
    ) {
        let kind = AstType::TSTypeAliasDeclaration;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut it.id);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.visit_ts_type(&mut it.type_annotation);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_interface_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSInterfaceDeclaration<'a>,
    ) {
        let kind = AstType::TSInterfaceDeclaration;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut it.id);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        if let Some(extends) = &mut it.extends {
            visitor.visit_ts_interface_heritages(extends);
        }
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_declaration(type_parameters);
        }
        visitor.visit_ts_interface_body(&mut it.body);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_interface_heritages<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TSInterfaceHeritage<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_ts_interface_heritage(el);
        }
    }

    #[inline]
    pub fn walk_ts_interface_heritage<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSInterfaceHeritage<'a>,
    ) {
        let kind = AstType::TSInterfaceHeritage;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        if let Some(type_parameters) = &mut it.type_parameters {
            visitor.visit_ts_type_parameter_instantiation(type_parameters);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_interface_body<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSInterfaceBody<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_ts_signatures(&mut it.body);
    }

    #[inline]
    pub fn walk_ts_enum_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSEnumDeclaration<'a>,
    ) {
        let kind = AstType::TSEnumDeclaration;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut it.id);
        visitor.enter_scope(ScopeFlags::empty(), &it.scope_id);
        visitor.visit_ts_enum_members(&mut it.members);
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_enum_members<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, TSEnumMember<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_ts_enum_member(el);
        }
    }

    #[inline]
    pub fn walk_ts_enum_member<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSEnumMember<'a>) {
        let kind = AstType::TSEnumMember;
        visitor.enter_node(kind);
        visitor.visit_ts_enum_member_name(&mut it.id);
        if let Some(initializer) = &mut it.initializer {
            visitor.visit_expression(initializer);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_enum_member_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSEnumMemberName<'a>,
    ) {
        match it {
            TSEnumMemberName::StaticIdentifier(it) => visitor.visit_identifier_name(it),
            TSEnumMemberName::StaticStringLiteral(it) => visitor.visit_string_literal(it),
            TSEnumMemberName::StaticTemplateLiteral(it) => visitor.visit_template_literal(it),
            TSEnumMemberName::StaticNumericLiteral(it) => visitor.visit_numeric_literal(it),
            match_expression!(TSEnumMemberName) => visitor.visit_expression(it.to_expression_mut()),
        }
    }

    #[inline]
    pub fn walk_ts_module_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSModuleDeclaration<'a>,
    ) {
        let kind = AstType::TSModuleDeclaration;
        visitor.enter_node(kind);
        visitor.visit_ts_module_declaration_name(&mut it.id);
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
        if let Some(body) = &mut it.body {
            visitor.visit_ts_module_declaration_body(body);
        }
        visitor.leave_scope();
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_module_declaration_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSModuleDeclarationName<'a>,
    ) {
        match it {
            TSModuleDeclarationName::Identifier(it) => visitor.visit_binding_identifier(it),
            TSModuleDeclarationName::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_ts_module_declaration_body<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSModuleDeclarationBody<'a>,
    ) {
        match it {
            TSModuleDeclarationBody::TSModuleDeclaration(it) => {
                visitor.visit_ts_module_declaration(it)
            }
            TSModuleDeclarationBody::TSModuleBlock(it) => visitor.visit_ts_module_block(it),
        }
    }

    #[inline]
    pub fn walk_ts_module_block<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut TSModuleBlock<'a>) {
        let kind = AstType::TSModuleBlock;
        visitor.enter_node(kind);
        visitor.visit_directives(&mut it.directives);
        visitor.visit_statements(&mut it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_import_equals_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSImportEqualsDeclaration<'a>,
    ) {
        let kind = AstType::TSImportEqualsDeclaration;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut it.id);
        visitor.visit_ts_module_reference(&mut it.module_reference);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_module_reference<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSModuleReference<'a>,
    ) {
        let kind = AstType::TSModuleReference;
        visitor.enter_node(kind);
        match it {
            TSModuleReference::ExternalModuleReference(it) => {
                visitor.visit_ts_external_module_reference(it)
            }
            match_ts_type_name!(TSModuleReference) => {
                visitor.visit_ts_type_name(it.to_ts_type_name_mut())
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_external_module_reference<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSExternalModuleReference<'a>,
    ) {
        let kind = AstType::TSExternalModuleReference;
        visitor.enter_node(kind);
        visitor.visit_string_literal(&mut it.expression);
        visitor.leave_node(kind);
    }

    pub fn walk_module_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ModuleDeclaration<'a>,
    ) {
        let kind = AstType::ModuleDeclaration;
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
    pub fn walk_import_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ImportDeclaration<'a>,
    ) {
        let kind = AstType::ImportDeclaration;
        visitor.enter_node(kind);
        if let Some(specifiers) = &mut it.specifiers {
            visitor.visit_import_declaration_specifiers(specifiers);
        }
        visitor.visit_string_literal(&mut it.source);
        if let Some(with_clause) = &mut it.with_clause {
            visitor.visit_with_clause(with_clause);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_declaration_specifiers<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, ImportDeclarationSpecifier<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_import_declaration_specifier(el);
        }
    }

    #[inline]
    pub fn walk_import_declaration_specifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ImportDeclarationSpecifier<'a>,
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
    pub fn walk_import_specifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ImportSpecifier<'a>,
    ) {
        let kind = AstType::ImportSpecifier;
        visitor.enter_node(kind);
        visitor.visit_module_export_name(&mut it.imported);
        visitor.visit_binding_identifier(&mut it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_module_export_name<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ModuleExportName<'a>,
    ) {
        match it {
            ModuleExportName::IdentifierName(it) => visitor.visit_identifier_name(it),
            ModuleExportName::IdentifierReference(it) => visitor.visit_identifier_reference(it),
            ModuleExportName::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_import_default_specifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ImportDefaultSpecifier<'a>,
    ) {
        let kind = AstType::ImportDefaultSpecifier;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_import_namespace_specifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ImportNamespaceSpecifier<'a>,
    ) {
        let kind = AstType::ImportNamespaceSpecifier;
        visitor.enter_node(kind);
        visitor.visit_binding_identifier(&mut it.local);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_with_clause<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut WithClause<'a>) {
        // NOTE: AstType doesn't exists!
        visitor.visit_identifier_name(&mut it.attributes_keyword);
        visitor.visit_import_attributes(&mut it.with_entries);
    }

    #[inline]
    pub fn walk_import_attributes<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, ImportAttribute<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_import_attribute(el);
        }
    }

    #[inline]
    pub fn walk_import_attribute<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ImportAttribute<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_import_attribute_key(&mut it.key);
        visitor.visit_string_literal(&mut it.value);
    }

    #[inline]
    pub fn walk_import_attribute_key<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ImportAttributeKey<'a>,
    ) {
        match it {
            ImportAttributeKey::Identifier(it) => visitor.visit_identifier_name(it),
            ImportAttributeKey::StringLiteral(it) => visitor.visit_string_literal(it),
        }
    }

    #[inline]
    pub fn walk_export_all_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ExportAllDeclaration<'a>,
    ) {
        let kind = AstType::ExportAllDeclaration;
        visitor.enter_node(kind);
        if let Some(exported) = &mut it.exported {
            visitor.visit_module_export_name(exported);
        }
        visitor.visit_string_literal(&mut it.source);
        if let Some(with_clause) = &mut it.with_clause {
            visitor.visit_with_clause(with_clause);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_export_default_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ExportDefaultDeclaration<'a>,
    ) {
        let kind = AstType::ExportDefaultDeclaration;
        visitor.enter_node(kind);
        visitor.visit_export_default_declaration_kind(&mut it.declaration);
        visitor.visit_module_export_name(&mut it.exported);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_export_default_declaration_kind<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ExportDefaultDeclarationKind<'a>,
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
                visitor.visit_expression(it.to_expression_mut())
            }
        }
    }

    #[inline]
    pub fn walk_export_named_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ExportNamedDeclaration<'a>,
    ) {
        let kind = AstType::ExportNamedDeclaration;
        visitor.enter_node(kind);
        if let Some(declaration) = &mut it.declaration {
            visitor.visit_declaration(declaration);
        }
        visitor.visit_export_specifiers(&mut it.specifiers);
        if let Some(source) = &mut it.source {
            visitor.visit_string_literal(source);
        }
        if let Some(with_clause) = &mut it.with_clause {
            visitor.visit_with_clause(with_clause);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_export_specifiers<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut Vec<'a, ExportSpecifier<'a>>,
    ) {
        for el in it.iter_mut() {
            visitor.visit_export_specifier(el);
        }
    }

    #[inline]
    pub fn walk_export_specifier<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut ExportSpecifier<'a>,
    ) {
        let kind = AstType::ExportSpecifier;
        visitor.enter_node(kind);
        visitor.visit_module_export_name(&mut it.local);
        visitor.visit_module_export_name(&mut it.exported);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_export_assignment<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSExportAssignment<'a>,
    ) {
        let kind = AstType::TSExportAssignment;
        visitor.enter_node(kind);
        visitor.visit_expression(&mut it.expression);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ts_namespace_export_declaration<'a, V: VisitMut<'a>>(
        visitor: &mut V,
        it: &mut TSNamespaceExportDeclaration<'a>,
    ) {
        // NOTE: AstType doesn't exists!
        visitor.visit_identifier_name(&mut it.id);
    }
}
