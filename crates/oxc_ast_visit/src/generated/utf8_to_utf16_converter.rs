// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/utf8_to_utf16.rs`.

use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;

use crate::{VisitMut, utf8_to_utf16::Utf8ToUtf16Converter, walk_mut};

impl<'a> VisitMut<'a> for Utf8ToUtf16Converter<'_> {
    fn visit_program(&mut self, it: &mut Program<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_program(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_identifier_name(&mut self, it: &mut IdentifierName<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_identifier_name(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_identifier_reference(&mut self, it: &mut IdentifierReference<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_identifier_reference(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_binding_identifier(&mut self, it: &mut BindingIdentifier<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_binding_identifier(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_label_identifier(&mut self, it: &mut LabelIdentifier<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_label_identifier(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_this_expression(&mut self, it: &mut ThisExpression) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_this_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_array_expression(&mut self, it: &mut ArrayExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_array_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_elision(&mut self, it: &mut Elision) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_elision(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_object_expression(&mut self, it: &mut ObjectExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_object_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_tagged_template_expression(&mut self, it: &mut TaggedTemplateExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_tagged_template_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_template_element(&mut self, it: &mut TemplateElement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_template_element(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_computed_member_expression(&mut self, it: &mut ComputedMemberExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_computed_member_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_static_member_expression(&mut self, it: &mut StaticMemberExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_static_member_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_private_field_expression(&mut self, it: &mut PrivateFieldExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_private_field_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_call_expression(&mut self, it: &mut CallExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_call_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_new_expression(&mut self, it: &mut NewExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_new_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_meta_property(&mut self, it: &mut MetaProperty<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_meta_property(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_spread_element(&mut self, it: &mut SpreadElement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_spread_element(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_update_expression(&mut self, it: &mut UpdateExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_update_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_unary_expression(&mut self, it: &mut UnaryExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_unary_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_binary_expression(&mut self, it: &mut BinaryExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_binary_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_private_in_expression(&mut self, it: &mut PrivateInExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_private_in_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_logical_expression(&mut self, it: &mut LogicalExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_logical_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_conditional_expression(&mut self, it: &mut ConditionalExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_conditional_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_assignment_expression(&mut self, it: &mut AssignmentExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_assignment_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_array_assignment_target(&mut self, it: &mut ArrayAssignmentTarget<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_array_assignment_target(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_object_assignment_target(&mut self, it: &mut ObjectAssignmentTarget<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_object_assignment_target(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_assignment_target_rest(&mut self, it: &mut AssignmentTargetRest<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_assignment_target_rest(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_assignment_target_with_default(&mut self, it: &mut AssignmentTargetWithDefault<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_assignment_target_with_default(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        it: &mut AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_assignment_target_property_identifier(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_assignment_target_property_property(
        &mut self,
        it: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_assignment_target_property_property(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_sequence_expression(&mut self, it: &mut SequenceExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_sequence_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_super(&mut self, it: &mut Super) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_super(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_await_expression(&mut self, it: &mut AwaitExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_await_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_chain_expression(&mut self, it: &mut ChainExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_chain_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_parenthesized_expression(&mut self, it: &mut ParenthesizedExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_parenthesized_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_directive(&mut self, it: &mut Directive<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_directive(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_hashbang(&mut self, it: &mut Hashbang<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_hashbang(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_block_statement(&mut self, it: &mut BlockStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_block_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_variable_declaration(&mut self, it: &mut VariableDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_variable_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_variable_declarator(&mut self, it: &mut VariableDeclarator<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_variable_declarator(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_empty_statement(&mut self, it: &mut EmptyStatement) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_empty_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_expression_statement(&mut self, it: &mut ExpressionStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_expression_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_if_statement(&mut self, it: &mut IfStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_if_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_do_while_statement(&mut self, it: &mut DoWhileStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_do_while_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_while_statement(&mut self, it: &mut WhileStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_while_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_for_statement(&mut self, it: &mut ForStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_for_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_for_in_statement(&mut self, it: &mut ForInStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_for_in_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_for_of_statement(&mut self, it: &mut ForOfStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_for_of_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_continue_statement(&mut self, it: &mut ContinueStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_continue_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_break_statement(&mut self, it: &mut BreakStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_break_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_return_statement(&mut self, it: &mut ReturnStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_return_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_with_statement(&mut self, it: &mut WithStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_with_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_switch_statement(&mut self, it: &mut SwitchStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_switch_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_switch_case(&mut self, it: &mut SwitchCase<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_switch_case(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_labeled_statement(&mut self, it: &mut LabeledStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_labeled_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_throw_statement(&mut self, it: &mut ThrowStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_throw_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_try_statement(&mut self, it: &mut TryStatement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_try_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_catch_clause(&mut self, it: &mut CatchClause<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_catch_clause(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_catch_parameter(&mut self, it: &mut CatchParameter<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_catch_parameter(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_debugger_statement(&mut self, it: &mut DebuggerStatement) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_debugger_statement(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_assignment_pattern(&mut self, it: &mut AssignmentPattern<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_assignment_pattern(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_object_pattern(&mut self, it: &mut ObjectPattern<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_object_pattern(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_array_pattern(&mut self, it: &mut ArrayPattern<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_array_pattern(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_function(&mut self, it: &mut Function<'a>, flags: ScopeFlags) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_function(self, it, flags);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_formal_parameter(&mut self, it: &mut FormalParameter<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_formal_parameter(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_function_body(&mut self, it: &mut FunctionBody<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_function_body(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_arrow_function_expression(&mut self, it: &mut ArrowFunctionExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_arrow_function_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_yield_expression(&mut self, it: &mut YieldExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_yield_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_class(&mut self, it: &mut Class<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_class(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_class_body(&mut self, it: &mut ClassBody<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_class_body(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_method_definition(&mut self, it: &mut MethodDefinition<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_method_definition(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_property_definition(&mut self, it: &mut PropertyDefinition<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_property_definition(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_private_identifier(&mut self, it: &mut PrivateIdentifier<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_private_identifier(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_static_block(&mut self, it: &mut StaticBlock<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_static_block(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_accessor_property(&mut self, it: &mut AccessorProperty<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_accessor_property(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_import_expression(&mut self, it: &mut ImportExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_import_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_import_declaration(&mut self, it: &mut ImportDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_import_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_import_default_specifier(&mut self, it: &mut ImportDefaultSpecifier<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_import_default_specifier(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_import_namespace_specifier(&mut self, it: &mut ImportNamespaceSpecifier<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_import_namespace_specifier(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_import_attribute(&mut self, it: &mut ImportAttribute<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_import_attribute(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_export_all_declaration(&mut self, it: &mut ExportAllDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_export_all_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_v_8_intrinsic_expression(&mut self, it: &mut V8IntrinsicExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_v_8_intrinsic_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_boolean_literal(&mut self, it: &mut BooleanLiteral) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_boolean_literal(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_null_literal(&mut self, it: &mut NullLiteral) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_null_literal(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_numeric_literal(&mut self, it: &mut NumericLiteral<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_numeric_literal(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_string_literal(&mut self, it: &mut StringLiteral<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_string_literal(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_big_int_literal(&mut self, it: &mut BigIntLiteral<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_big_int_literal(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_reg_exp_literal(&mut self, it: &mut RegExpLiteral<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_reg_exp_literal(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_element(&mut self, it: &mut JSXElement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_element(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_opening_element(&mut self, it: &mut JSXOpeningElement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_opening_element(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_closing_element(&mut self, it: &mut JSXClosingElement<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_closing_element(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_fragment(&mut self, it: &mut JSXFragment<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_fragment(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_opening_fragment(&mut self, it: &mut JSXOpeningFragment) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_opening_fragment(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_closing_fragment(&mut self, it: &mut JSXClosingFragment) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_closing_fragment(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_namespaced_name(&mut self, it: &mut JSXNamespacedName<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_namespaced_name(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_member_expression(&mut self, it: &mut JSXMemberExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_member_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_expression_container(&mut self, it: &mut JSXExpressionContainer<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_expression_container(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_empty_expression(&mut self, it: &mut JSXEmptyExpression) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_empty_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_attribute(&mut self, it: &mut JSXAttribute<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_attribute(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_spread_attribute(&mut self, it: &mut JSXSpreadAttribute<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_spread_attribute(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_identifier(&mut self, it: &mut JSXIdentifier<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_identifier(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_spread_child(&mut self, it: &mut JSXSpreadChild<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_spread_child(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_jsx_text(&mut self, it: &mut JSXText<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_jsx_text(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_this_parameter(&mut self, it: &mut TSThisParameter<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_this_parameter(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_enum_declaration(&mut self, it: &mut TSEnumDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_enum_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_enum_body(&mut self, it: &mut TSEnumBody<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_enum_body(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_enum_member(&mut self, it: &mut TSEnumMember<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_enum_member(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_annotation(&mut self, it: &mut TSTypeAnnotation<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_annotation(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_literal_type(&mut self, it: &mut TSLiteralType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_literal_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_conditional_type(&mut self, it: &mut TSConditionalType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_conditional_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_union_type(&mut self, it: &mut TSUnionType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_union_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_intersection_type(&mut self, it: &mut TSIntersectionType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_intersection_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_parenthesized_type(&mut self, it: &mut TSParenthesizedType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_parenthesized_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_operator(&mut self, it: &mut TSTypeOperator<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_operator(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_array_type(&mut self, it: &mut TSArrayType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_array_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_indexed_access_type(&mut self, it: &mut TSIndexedAccessType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_indexed_access_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_tuple_type(&mut self, it: &mut TSTupleType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_tuple_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_named_tuple_member(&mut self, it: &mut TSNamedTupleMember<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_named_tuple_member(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_optional_type(&mut self, it: &mut TSOptionalType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_optional_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_rest_type(&mut self, it: &mut TSRestType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_rest_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_any_keyword(&mut self, it: &mut TSAnyKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_any_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_string_keyword(&mut self, it: &mut TSStringKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_string_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_boolean_keyword(&mut self, it: &mut TSBooleanKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_boolean_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_number_keyword(&mut self, it: &mut TSNumberKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_number_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_never_keyword(&mut self, it: &mut TSNeverKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_never_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_intrinsic_keyword(&mut self, it: &mut TSIntrinsicKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_intrinsic_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_unknown_keyword(&mut self, it: &mut TSUnknownKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_unknown_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_null_keyword(&mut self, it: &mut TSNullKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_null_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_undefined_keyword(&mut self, it: &mut TSUndefinedKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_undefined_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_void_keyword(&mut self, it: &mut TSVoidKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_void_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_symbol_keyword(&mut self, it: &mut TSSymbolKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_symbol_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_this_type(&mut self, it: &mut TSThisType) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_this_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_object_keyword(&mut self, it: &mut TSObjectKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_object_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_big_int_keyword(&mut self, it: &mut TSBigIntKeyword) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_big_int_keyword(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_reference(&mut self, it: &mut TSTypeReference<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_reference(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_qualified_name(&mut self, it: &mut TSQualifiedName<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_qualified_name(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_parameter_instantiation(&mut self, it: &mut TSTypeParameterInstantiation<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_parameter_instantiation(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_parameter(&mut self, it: &mut TSTypeParameter<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_parameter(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_parameter_declaration(&mut self, it: &mut TSTypeParameterDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_parameter_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_alias_declaration(&mut self, it: &mut TSTypeAliasDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_alias_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_class_implements(&mut self, it: &mut TSClassImplements<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_class_implements(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_interface_declaration(&mut self, it: &mut TSInterfaceDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_interface_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_interface_body(&mut self, it: &mut TSInterfaceBody<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_interface_body(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_property_signature(&mut self, it: &mut TSPropertySignature<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_property_signature(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_index_signature(&mut self, it: &mut TSIndexSignature<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_index_signature(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_call_signature_declaration(&mut self, it: &mut TSCallSignatureDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_call_signature_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_method_signature(&mut self, it: &mut TSMethodSignature<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_method_signature(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_construct_signature_declaration(
        &mut self,
        it: &mut TSConstructSignatureDeclaration<'a>,
    ) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_construct_signature_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_index_signature_name(&mut self, it: &mut TSIndexSignatureName<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_index_signature_name(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_interface_heritage(&mut self, it: &mut TSInterfaceHeritage<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_interface_heritage(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_predicate(&mut self, it: &mut TSTypePredicate<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_predicate(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_module_declaration(&mut self, it: &mut TSModuleDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_module_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_module_block(&mut self, it: &mut TSModuleBlock<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_module_block(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_literal(&mut self, it: &mut TSTypeLiteral<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_literal(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_infer_type(&mut self, it: &mut TSInferType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_infer_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_query(&mut self, it: &mut TSTypeQuery<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_query(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_import_type(&mut self, it: &mut TSImportType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_import_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_import_type_qualified_name(&mut self, it: &mut TSImportTypeQualifiedName<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_import_type_qualified_name(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_function_type(&mut self, it: &mut TSFunctionType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_function_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_constructor_type(&mut self, it: &mut TSConstructorType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_constructor_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_mapped_type(&mut self, it: &mut TSMappedType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_mapped_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_as_expression(&mut self, it: &mut TSAsExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_as_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_satisfies_expression(&mut self, it: &mut TSSatisfiesExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_satisfies_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_type_assertion(&mut self, it: &mut TSTypeAssertion<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_type_assertion(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_import_equals_declaration(&mut self, it: &mut TSImportEqualsDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_import_equals_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_external_module_reference(&mut self, it: &mut TSExternalModuleReference<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_external_module_reference(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_non_null_expression(&mut self, it: &mut TSNonNullExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_non_null_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_decorator(&mut self, it: &mut Decorator<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_decorator(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_export_assignment(&mut self, it: &mut TSExportAssignment<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_export_assignment(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_namespace_export_declaration(&mut self, it: &mut TSNamespaceExportDeclaration<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_namespace_export_declaration(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_ts_instantiation_expression(&mut self, it: &mut TSInstantiationExpression<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_ts_instantiation_expression(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_js_doc_nullable_type(&mut self, it: &mut JSDocNullableType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_js_doc_nullable_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_js_doc_non_nullable_type(&mut self, it: &mut JSDocNonNullableType<'a>) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_js_doc_non_nullable_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_js_doc_unknown_type(&mut self, it: &mut JSDocUnknownType) {
        self.convert_offset(&mut it.span.start);
        walk_mut::walk_js_doc_unknown_type(self, it);
        self.convert_offset(&mut it.span.end);
    }

    fn visit_formal_parameters(&mut self, params: &mut FormalParameters<'a>) {
        walk_mut::walk_formal_parameters(self, params);
    }

    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        self.convert_offset(&mut prop.span.start);
        match (prop.shorthand, &mut prop.key, &mut prop.value) {
            (true, PropertyKey::StaticIdentifier(key), Expression::Identifier(value)) => {
                self.visit_identifier_name(key);
                value.span = key.span;
            }
            (_, key, value) => {
                self.visit_property_key(key);
                self.visit_expression(value);
            }
        }
        self.convert_offset(&mut prop.span.end);
    }

    fn visit_binding_pattern(&mut self, pattern: &mut BindingPattern<'a>) {
        let span_end = match &mut pattern.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                self.convert_offset(&mut ident.span.start);
                walk_mut::walk_binding_identifier(self, ident);
                &mut ident.span.end
            }
            BindingPatternKind::ObjectPattern(obj_pattern) => {
                self.convert_offset(&mut obj_pattern.span.start);
                walk_mut::walk_object_pattern(self, obj_pattern);
                &mut obj_pattern.span.end
            }
            BindingPatternKind::ArrayPattern(arr_pattern) => {
                self.convert_offset(&mut arr_pattern.span.start);
                walk_mut::walk_array_pattern(self, arr_pattern);
                &mut arr_pattern.span.end
            }
            BindingPatternKind::AssignmentPattern(assign_pattern) => {
                self.convert_offset(&mut assign_pattern.span.start);
                walk_mut::walk_assignment_pattern(self, assign_pattern);
                &mut assign_pattern.span.end
            }
        };
        if let Some(type_annotation) = &mut pattern.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
        self.convert_offset(span_end);
    }

    fn visit_binding_rest_element(&mut self, rest_element: &mut BindingRestElement<'a>) {
        self.convert_offset(&mut rest_element.span.start);
        self.visit_binding_pattern_kind(&mut rest_element.argument.kind);
        if let Some(type_annotation) = &mut rest_element.argument.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
        self.convert_offset(&mut rest_element.span.end);
    }

    fn visit_binding_property(&mut self, prop: &mut BindingProperty<'a>) {
        self.convert_offset(&mut prop.span.start);
        match (prop.shorthand, &mut prop.key, &mut prop.value) {
            (
                true,
                PropertyKey::StaticIdentifier(key),
                BindingPattern { kind: BindingPatternKind::BindingIdentifier(value), .. },
            ) => {
                self.visit_identifier_name(key);
                value.span = key.span;
            }
            (
                true,
                PropertyKey::StaticIdentifier(key),
                BindingPattern { kind: BindingPatternKind::AssignmentPattern(pattern), .. },
            ) => {
                self.visit_assignment_pattern(pattern);
                key.span = pattern.left.span();
            }
            (_, key, value) => {
                self.visit_property_key(key);
                self.visit_binding_pattern(value);
            }
        }
        self.convert_offset(&mut prop.span.end);
    }

    fn visit_export_named_declaration(&mut self, decl: &mut ExportNamedDeclaration<'a>) {
        // Special case logic for `@dec export class C {}`
        if let Some(Declaration::ClassDeclaration(class)) = &mut decl.declaration {
            self.visit_export_class(class, &mut decl.span);
        } else {
            self.convert_offset(&mut decl.span.start);
            walk_mut::walk_export_named_declaration(self, decl);
            self.convert_offset(&mut decl.span.end);
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &mut ExportDefaultDeclaration<'a>) {
        // Special case logic for `@dec export default class {}`
        if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &mut decl.declaration {
            self.visit_export_class(class, &mut decl.span);
        } else {
            self.convert_offset(&mut decl.span.start);
            walk_mut::walk_export_default_declaration(self, decl);
            self.convert_offset(&mut decl.span.end);
        }
    }

    fn visit_export_specifier(&mut self, specifier: &mut ExportSpecifier<'a>) {
        self.convert_offset(&mut specifier.span.start);
        match (&mut specifier.local, &mut specifier.exported) {
            (
                ModuleExportName::IdentifierReference(local),
                ModuleExportName::IdentifierName(exported),
            ) if local.span == exported.span => {
                self.visit_identifier_reference(local);
                exported.span = local.span;
            }
            (
                ModuleExportName::IdentifierName(local),
                ModuleExportName::IdentifierName(exported),
            ) if local.span == exported.span => {
                self.visit_identifier_name(local);
                exported.span = local.span;
            }
            (ModuleExportName::StringLiteral(local), ModuleExportName::StringLiteral(exported))
                if local.span == exported.span =>
            {
                self.visit_string_literal(local);
                exported.span = local.span;
            }
            (local, exported) => {
                self.visit_module_export_name(local);
                self.visit_module_export_name(exported);
            }
        }
        self.convert_offset(&mut specifier.span.end);
    }

    fn visit_import_specifier(&mut self, specifier: &mut ImportSpecifier<'a>) {
        self.convert_offset(&mut specifier.span.start);
        match &mut specifier.imported {
            ModuleExportName::IdentifierName(imported) if imported.span == specifier.local.span => {
                self.visit_identifier_name(imported);
                specifier.local.span = imported.span;
            }
            imported => {
                self.visit_module_export_name(imported);
                self.visit_binding_identifier(&mut specifier.local);
            }
        }
        self.convert_offset(&mut specifier.span.end);
    }

    fn visit_with_clause(&mut self, with_clause: &mut WithClause<'a>) {
        self.visit_import_attributes(&mut with_clause.with_entries);
    }

    fn visit_template_literal(&mut self, lit: &mut TemplateLiteral<'a>) {
        self.convert_offset(&mut lit.span.start);
        for (quasi, expression) in lit.quasis.iter_mut().zip(&mut lit.expressions) {
            self.visit_template_element(quasi);
            self.visit_expression(expression);
        }
        self.visit_template_element(lit.quasis.last_mut().unwrap());
        self.convert_offset(&mut lit.span.end);
    }

    fn visit_ts_template_literal_type(&mut self, lit: &mut TSTemplateLiteralType<'a>) {
        self.convert_offset(&mut lit.span.start);
        for (quasi, ts_type) in lit.quasis.iter_mut().zip(&mut lit.types) {
            self.visit_template_element(quasi);
            self.visit_ts_type(ts_type);
        }
        self.visit_template_element(lit.quasis.last_mut().unwrap());
        self.convert_offset(&mut lit.span.end);
    }
}

impl Utf8ToUtf16Converter<'_> {
    /// Visit `ExportNamedDeclaration` or `ExportDefaultDeclaration` containing a `Class`.
    /// e.g. `export class C {}`, `export default class {}`
    ///
    /// These need special handing because decorators before the `export` keyword
    /// have `Span`s which are before the start of the export statement.
    /// e.g. `@dec export class C {}`, `@dec export default class {}`.
    /// So they need to be processed first.
    fn visit_export_class(&mut self, class: &mut Class<'_>, export_decl_span: &mut Span) {
        // Process decorators.
        // Process decorators before the `export` keyword first.
        // These have spans which are before the export statement span start.
        // Then process export statement and `Class` start, then remaining decorators,
        // which have spans within the span of `Class`.
        let mut decl_start = export_decl_span.start;
        for decorator in &mut class.decorators {
            if decorator.span.start > decl_start {
                // Process span start of export statement and `Class`
                self.convert_offset(&mut export_decl_span.start);
                self.convert_offset(&mut class.span.start);
                // Prevent this branch being taken again
                decl_start = u32::MAX;
            }
            self.visit_decorator(decorator);
        }
        // If didn't already, process span start of export statement and `Class`
        if decl_start < u32::MAX {
            self.convert_offset(&mut export_decl_span.start);
            self.convert_offset(&mut class.span.start);
        }
        // Process rest of the class
        if let Some(id) = &mut class.id {
            self.visit_binding_identifier(id);
        }
        if let Some(type_parameters) = &mut class.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(super_class) = &mut class.super_class {
            self.visit_expression(super_class);
        }
        if let Some(super_type_arguments) = &mut class.super_type_arguments {
            self.visit_ts_type_parameter_instantiation(super_type_arguments);
        }
        self.visit_ts_class_implements_list(&mut class.implements);
        self.visit_class_body(&mut class.body);
        // Process span end of `Class` and export statement
        self.convert_offset(&mut class.span.end);
        self.convert_offset(&mut export_decl_span.end);
    }
}
