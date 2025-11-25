// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/scopes_collector.rs`.

#![expect(
    unused_variables,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::single_match_else
)]

use std::cell::Cell;

use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_syntax::scope::{ScopeFlags, ScopeId};

/// Visitor that locates all child scopes.
///
/// Note: Direct child scopes only, not grandchild scopes.
/// Does not do full traversal - stops each time it hits a node with a scope.
pub struct ChildScopeCollector {
    pub(crate) scope_ids: Vec<ScopeId>,
}

impl ChildScopeCollector {
    pub(crate) fn new() -> Self {
        Self { scope_ids: vec![] }
    }

    pub(crate) fn add_scope(&mut self, scope_id: &Cell<Option<ScopeId>>) {
        self.scope_ids.push(scope_id.get().unwrap());
    }
}

impl<'a> Visit<'a> for ChildScopeCollector {
    #[inline]
    fn visit_program(&mut self, it: &Program<'a>) {
        self.add_scope(&it.scope_id);
    }

    fn visit_expression(&mut self, it: &Expression<'a>) {
        match it {
            Expression::TemplateLiteral(it) => self.visit_template_literal(it),
            Expression::ArrayExpression(it) => self.visit_array_expression(it),
            Expression::ArrowFunctionExpression(it) => self.visit_arrow_function_expression(it),
            Expression::AssignmentExpression(it) => self.visit_assignment_expression(it),
            Expression::AwaitExpression(it) => self.visit_await_expression(it),
            Expression::BinaryExpression(it) => self.visit_binary_expression(it),
            Expression::CallExpression(it) => self.visit_call_expression(it),
            Expression::ChainExpression(it) => self.visit_chain_expression(it),
            Expression::ClassExpression(it) => self.visit_class(it),
            Expression::ConditionalExpression(it) => self.visit_conditional_expression(it),
            Expression::FunctionExpression(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            Expression::ImportExpression(it) => self.visit_import_expression(it),
            Expression::LogicalExpression(it) => self.visit_logical_expression(it),
            Expression::NewExpression(it) => self.visit_new_expression(it),
            Expression::ObjectExpression(it) => self.visit_object_expression(it),
            Expression::ParenthesizedExpression(it) => self.visit_parenthesized_expression(it),
            Expression::SequenceExpression(it) => self.visit_sequence_expression(it),
            Expression::TaggedTemplateExpression(it) => self.visit_tagged_template_expression(it),
            Expression::UnaryExpression(it) => self.visit_unary_expression(it),
            Expression::UpdateExpression(it) => self.visit_update_expression(it),
            Expression::YieldExpression(it) => self.visit_yield_expression(it),
            Expression::PrivateInExpression(it) => self.visit_private_in_expression(it),
            Expression::JSXElement(it) => self.visit_jsx_element(it),
            Expression::JSXFragment(it) => self.visit_jsx_fragment(it),
            Expression::TSAsExpression(it) => self.visit_ts_as_expression(it),
            Expression::TSSatisfiesExpression(it) => self.visit_ts_satisfies_expression(it),
            Expression::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            Expression::TSNonNullExpression(it) => self.visit_ts_non_null_expression(it),
            Expression::TSInstantiationExpression(it) => self.visit_ts_instantiation_expression(it),
            Expression::V8IntrinsicExpression(it) => self.visit_v_8_intrinsic_expression(it),
            Expression::ComputedMemberExpression(it) => self.visit_computed_member_expression(it),
            Expression::StaticMemberExpression(it) => self.visit_static_member_expression(it),
            Expression::PrivateFieldExpression(it) => self.visit_private_field_expression(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `BooleanLiteral`
                // `NullLiteral`
                // `NumericLiteral`
                // `BigIntLiteral`
                // `RegExpLiteral`
                // `StringLiteral`
                // `Identifier`
                // `MetaProperty`
                // `Super`
                // `ThisExpression`
            }
        }
    }

    #[inline(always)]
    fn visit_identifier_name(&mut self, it: &IdentifierName<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_label_identifier(&mut self, it: &LabelIdentifier<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_this_expression(&mut self, it: &ThisExpression) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_array_expression(&mut self, it: &ArrayExpression<'a>) {
        self.visit_array_expression_elements(&it.elements);
    }

    fn visit_array_expression_element(&mut self, it: &ArrayExpressionElement<'a>) {
        match it {
            ArrayExpressionElement::SpreadElement(it) => self.visit_spread_element(it),
            ArrayExpressionElement::TemplateLiteral(it) => self.visit_template_literal(it),
            ArrayExpressionElement::ArrayExpression(it) => self.visit_array_expression(it),
            ArrayExpressionElement::ArrowFunctionExpression(it) => {
                self.visit_arrow_function_expression(it)
            }
            ArrayExpressionElement::AssignmentExpression(it) => {
                self.visit_assignment_expression(it)
            }
            ArrayExpressionElement::AwaitExpression(it) => self.visit_await_expression(it),
            ArrayExpressionElement::BinaryExpression(it) => self.visit_binary_expression(it),
            ArrayExpressionElement::CallExpression(it) => self.visit_call_expression(it),
            ArrayExpressionElement::ChainExpression(it) => self.visit_chain_expression(it),
            ArrayExpressionElement::ClassExpression(it) => self.visit_class(it),
            ArrayExpressionElement::ConditionalExpression(it) => {
                self.visit_conditional_expression(it)
            }
            ArrayExpressionElement::FunctionExpression(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            ArrayExpressionElement::ImportExpression(it) => self.visit_import_expression(it),
            ArrayExpressionElement::LogicalExpression(it) => self.visit_logical_expression(it),
            ArrayExpressionElement::NewExpression(it) => self.visit_new_expression(it),
            ArrayExpressionElement::ObjectExpression(it) => self.visit_object_expression(it),
            ArrayExpressionElement::ParenthesizedExpression(it) => {
                self.visit_parenthesized_expression(it)
            }
            ArrayExpressionElement::SequenceExpression(it) => self.visit_sequence_expression(it),
            ArrayExpressionElement::TaggedTemplateExpression(it) => {
                self.visit_tagged_template_expression(it)
            }
            ArrayExpressionElement::UnaryExpression(it) => self.visit_unary_expression(it),
            ArrayExpressionElement::UpdateExpression(it) => self.visit_update_expression(it),
            ArrayExpressionElement::YieldExpression(it) => self.visit_yield_expression(it),
            ArrayExpressionElement::PrivateInExpression(it) => self.visit_private_in_expression(it),
            ArrayExpressionElement::JSXElement(it) => self.visit_jsx_element(it),
            ArrayExpressionElement::JSXFragment(it) => self.visit_jsx_fragment(it),
            ArrayExpressionElement::TSAsExpression(it) => self.visit_ts_as_expression(it),
            ArrayExpressionElement::TSSatisfiesExpression(it) => {
                self.visit_ts_satisfies_expression(it)
            }
            ArrayExpressionElement::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            ArrayExpressionElement::TSNonNullExpression(it) => {
                self.visit_ts_non_null_expression(it)
            }
            ArrayExpressionElement::TSInstantiationExpression(it) => {
                self.visit_ts_instantiation_expression(it)
            }
            ArrayExpressionElement::V8IntrinsicExpression(it) => {
                self.visit_v_8_intrinsic_expression(it)
            }
            ArrayExpressionElement::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it)
            }
            ArrayExpressionElement::StaticMemberExpression(it) => {
                self.visit_static_member_expression(it)
            }
            ArrayExpressionElement::PrivateFieldExpression(it) => {
                self.visit_private_field_expression(it)
            }
            _ => {
                // Remaining variants do not contain scopes:
                // `Elision`
                // `BooleanLiteral`
                // `NullLiteral`
                // `NumericLiteral`
                // `BigIntLiteral`
                // `RegExpLiteral`
                // `StringLiteral`
                // `Identifier`
                // `MetaProperty`
                // `Super`
                // `ThisExpression`
            }
        }
    }

    #[inline(always)]
    fn visit_elision(&mut self, it: &Elision) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        self.visit_object_property_kinds(&it.properties);
    }

    #[inline]
    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        self.visit_property_key(&it.key);
        self.visit_expression(&it.value);
    }

    fn visit_property_key(&mut self, it: &PropertyKey<'a>) {
        match it {
            PropertyKey::TemplateLiteral(it) => self.visit_template_literal(it),
            PropertyKey::ArrayExpression(it) => self.visit_array_expression(it),
            PropertyKey::ArrowFunctionExpression(it) => self.visit_arrow_function_expression(it),
            PropertyKey::AssignmentExpression(it) => self.visit_assignment_expression(it),
            PropertyKey::AwaitExpression(it) => self.visit_await_expression(it),
            PropertyKey::BinaryExpression(it) => self.visit_binary_expression(it),
            PropertyKey::CallExpression(it) => self.visit_call_expression(it),
            PropertyKey::ChainExpression(it) => self.visit_chain_expression(it),
            PropertyKey::ClassExpression(it) => self.visit_class(it),
            PropertyKey::ConditionalExpression(it) => self.visit_conditional_expression(it),
            PropertyKey::FunctionExpression(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            PropertyKey::ImportExpression(it) => self.visit_import_expression(it),
            PropertyKey::LogicalExpression(it) => self.visit_logical_expression(it),
            PropertyKey::NewExpression(it) => self.visit_new_expression(it),
            PropertyKey::ObjectExpression(it) => self.visit_object_expression(it),
            PropertyKey::ParenthesizedExpression(it) => self.visit_parenthesized_expression(it),
            PropertyKey::SequenceExpression(it) => self.visit_sequence_expression(it),
            PropertyKey::TaggedTemplateExpression(it) => self.visit_tagged_template_expression(it),
            PropertyKey::UnaryExpression(it) => self.visit_unary_expression(it),
            PropertyKey::UpdateExpression(it) => self.visit_update_expression(it),
            PropertyKey::YieldExpression(it) => self.visit_yield_expression(it),
            PropertyKey::PrivateInExpression(it) => self.visit_private_in_expression(it),
            PropertyKey::JSXElement(it) => self.visit_jsx_element(it),
            PropertyKey::JSXFragment(it) => self.visit_jsx_fragment(it),
            PropertyKey::TSAsExpression(it) => self.visit_ts_as_expression(it),
            PropertyKey::TSSatisfiesExpression(it) => self.visit_ts_satisfies_expression(it),
            PropertyKey::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            PropertyKey::TSNonNullExpression(it) => self.visit_ts_non_null_expression(it),
            PropertyKey::TSInstantiationExpression(it) => {
                self.visit_ts_instantiation_expression(it)
            }
            PropertyKey::V8IntrinsicExpression(it) => self.visit_v_8_intrinsic_expression(it),
            PropertyKey::ComputedMemberExpression(it) => self.visit_computed_member_expression(it),
            PropertyKey::StaticMemberExpression(it) => self.visit_static_member_expression(it),
            PropertyKey::PrivateFieldExpression(it) => self.visit_private_field_expression(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `StaticIdentifier`
                // `PrivateIdentifier`
                // `BooleanLiteral`
                // `NullLiteral`
                // `NumericLiteral`
                // `BigIntLiteral`
                // `RegExpLiteral`
                // `StringLiteral`
                // `Identifier`
                // `MetaProperty`
                // `Super`
                // `ThisExpression`
            }
        }
    }

    #[inline]
    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        self.visit_expressions(&it.expressions);
    }

    #[inline]
    fn visit_tagged_template_expression(&mut self, it: &TaggedTemplateExpression<'a>) {
        self.visit_expression(&it.tag);
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
        self.visit_template_literal(&it.quasi);
    }

    #[inline(always)]
    fn visit_template_element(&mut self, it: &TemplateElement<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
        self.visit_expression(&it.object);
        self.visit_expression(&it.expression);
    }

    #[inline]
    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        self.visit_expression(&it.object);
    }

    #[inline]
    fn visit_private_field_expression(&mut self, it: &PrivateFieldExpression<'a>) {
        self.visit_expression(&it.object);
    }

    #[inline]
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.visit_expression(&it.callee);
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
        self.visit_arguments(&it.arguments);
    }

    #[inline]
    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        self.visit_expression(&it.callee);
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
        self.visit_arguments(&it.arguments);
    }

    #[inline(always)]
    fn visit_meta_property(&mut self, it: &MetaProperty<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_spread_element(&mut self, it: &SpreadElement<'a>) {
        self.visit_expression(&it.argument);
    }

    fn visit_argument(&mut self, it: &Argument<'a>) {
        match it {
            Argument::SpreadElement(it) => self.visit_spread_element(it),
            Argument::TemplateLiteral(it) => self.visit_template_literal(it),
            Argument::ArrayExpression(it) => self.visit_array_expression(it),
            Argument::ArrowFunctionExpression(it) => self.visit_arrow_function_expression(it),
            Argument::AssignmentExpression(it) => self.visit_assignment_expression(it),
            Argument::AwaitExpression(it) => self.visit_await_expression(it),
            Argument::BinaryExpression(it) => self.visit_binary_expression(it),
            Argument::CallExpression(it) => self.visit_call_expression(it),
            Argument::ChainExpression(it) => self.visit_chain_expression(it),
            Argument::ClassExpression(it) => self.visit_class(it),
            Argument::ConditionalExpression(it) => self.visit_conditional_expression(it),
            Argument::FunctionExpression(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            Argument::ImportExpression(it) => self.visit_import_expression(it),
            Argument::LogicalExpression(it) => self.visit_logical_expression(it),
            Argument::NewExpression(it) => self.visit_new_expression(it),
            Argument::ObjectExpression(it) => self.visit_object_expression(it),
            Argument::ParenthesizedExpression(it) => self.visit_parenthesized_expression(it),
            Argument::SequenceExpression(it) => self.visit_sequence_expression(it),
            Argument::TaggedTemplateExpression(it) => self.visit_tagged_template_expression(it),
            Argument::UnaryExpression(it) => self.visit_unary_expression(it),
            Argument::UpdateExpression(it) => self.visit_update_expression(it),
            Argument::YieldExpression(it) => self.visit_yield_expression(it),
            Argument::PrivateInExpression(it) => self.visit_private_in_expression(it),
            Argument::JSXElement(it) => self.visit_jsx_element(it),
            Argument::JSXFragment(it) => self.visit_jsx_fragment(it),
            Argument::TSAsExpression(it) => self.visit_ts_as_expression(it),
            Argument::TSSatisfiesExpression(it) => self.visit_ts_satisfies_expression(it),
            Argument::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            Argument::TSNonNullExpression(it) => self.visit_ts_non_null_expression(it),
            Argument::TSInstantiationExpression(it) => self.visit_ts_instantiation_expression(it),
            Argument::V8IntrinsicExpression(it) => self.visit_v_8_intrinsic_expression(it),
            Argument::ComputedMemberExpression(it) => self.visit_computed_member_expression(it),
            Argument::StaticMemberExpression(it) => self.visit_static_member_expression(it),
            Argument::PrivateFieldExpression(it) => self.visit_private_field_expression(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `BooleanLiteral`
                // `NullLiteral`
                // `NumericLiteral`
                // `BigIntLiteral`
                // `RegExpLiteral`
                // `StringLiteral`
                // `Identifier`
                // `MetaProperty`
                // `Super`
                // `ThisExpression`
            }
        }
    }

    #[inline]
    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        self.visit_simple_assignment_target(&it.argument);
    }

    #[inline]
    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        self.visit_expression(&it.argument);
    }

    #[inline]
    fn visit_binary_expression(&mut self, it: &BinaryExpression<'a>) {
        self.visit_expression(&it.left);
        self.visit_expression(&it.right);
    }

    #[inline]
    fn visit_private_in_expression(&mut self, it: &PrivateInExpression<'a>) {
        self.visit_expression(&it.right);
    }

    #[inline]
    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        self.visit_expression(&it.left);
        self.visit_expression(&it.right);
    }

    #[inline]
    fn visit_conditional_expression(&mut self, it: &ConditionalExpression<'a>) {
        self.visit_expression(&it.test);
        self.visit_expression(&it.consequent);
        self.visit_expression(&it.alternate);
    }

    #[inline]
    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        self.visit_assignment_target(&it.left);
        self.visit_expression(&it.right);
    }

    fn visit_assignment_target(&mut self, it: &AssignmentTarget<'a>) {
        match it {
            AssignmentTarget::TSAsExpression(it) => self.visit_ts_as_expression(it),
            AssignmentTarget::TSSatisfiesExpression(it) => self.visit_ts_satisfies_expression(it),
            AssignmentTarget::TSNonNullExpression(it) => self.visit_ts_non_null_expression(it),
            AssignmentTarget::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            AssignmentTarget::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it)
            }
            AssignmentTarget::StaticMemberExpression(it) => self.visit_static_member_expression(it),
            AssignmentTarget::PrivateFieldExpression(it) => self.visit_private_field_expression(it),
            AssignmentTarget::ArrayAssignmentTarget(it) => self.visit_array_assignment_target(it),
            AssignmentTarget::ObjectAssignmentTarget(it) => self.visit_object_assignment_target(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `AssignmentTargetIdentifier`
            }
        }
    }

    fn visit_simple_assignment_target(&mut self, it: &SimpleAssignmentTarget<'a>) {
        match it {
            SimpleAssignmentTarget::TSAsExpression(it) => self.visit_ts_as_expression(it),
            SimpleAssignmentTarget::TSSatisfiesExpression(it) => {
                self.visit_ts_satisfies_expression(it)
            }
            SimpleAssignmentTarget::TSNonNullExpression(it) => {
                self.visit_ts_non_null_expression(it)
            }
            SimpleAssignmentTarget::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            SimpleAssignmentTarget::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it)
            }
            SimpleAssignmentTarget::StaticMemberExpression(it) => {
                self.visit_static_member_expression(it)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(it) => {
                self.visit_private_field_expression(it)
            }
            _ => {
                // Remaining variants do not contain scopes:
                // `AssignmentTargetIdentifier`
            }
        }
    }

    #[inline]
    fn visit_array_assignment_target(&mut self, it: &ArrayAssignmentTarget<'a>) {
        for el in it.elements.iter().flatten() {
            self.visit_assignment_target_maybe_default(el);
        }
        if let Some(rest) = &it.rest {
            self.visit_assignment_target_rest(rest);
        }
    }

    #[inline]
    fn visit_object_assignment_target(&mut self, it: &ObjectAssignmentTarget<'a>) {
        self.visit_assignment_target_properties(&it.properties);
        if let Some(rest) = &it.rest {
            self.visit_assignment_target_rest(rest);
        }
    }

    #[inline]
    fn visit_assignment_target_rest(&mut self, it: &AssignmentTargetRest<'a>) {
        self.visit_assignment_target(&it.target);
    }

    fn visit_assignment_target_maybe_default(&mut self, it: &AssignmentTargetMaybeDefault<'a>) {
        match it {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(it) => {
                self.visit_assignment_target_with_default(it)
            }
            AssignmentTargetMaybeDefault::TSAsExpression(it) => self.visit_ts_as_expression(it),
            AssignmentTargetMaybeDefault::TSSatisfiesExpression(it) => {
                self.visit_ts_satisfies_expression(it)
            }
            AssignmentTargetMaybeDefault::TSNonNullExpression(it) => {
                self.visit_ts_non_null_expression(it)
            }
            AssignmentTargetMaybeDefault::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            AssignmentTargetMaybeDefault::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it)
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(it) => {
                self.visit_static_member_expression(it)
            }
            AssignmentTargetMaybeDefault::PrivateFieldExpression(it) => {
                self.visit_private_field_expression(it)
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(it) => {
                self.visit_array_assignment_target(it)
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(it) => {
                self.visit_object_assignment_target(it)
            }
            _ => {
                // Remaining variants do not contain scopes:
                // `AssignmentTargetIdentifier`
            }
        }
    }

    #[inline]
    fn visit_assignment_target_with_default(&mut self, it: &AssignmentTargetWithDefault<'a>) {
        self.visit_assignment_target(&it.binding);
        self.visit_expression(&it.init);
    }

    #[inline]
    fn visit_assignment_target_property_identifier(
        &mut self,
        it: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        if let Some(init) = &it.init {
            self.visit_expression(init);
        }
    }

    #[inline]
    fn visit_assignment_target_property_property(
        &mut self,
        it: &AssignmentTargetPropertyProperty<'a>,
    ) {
        self.visit_property_key(&it.name);
        self.visit_assignment_target_maybe_default(&it.binding);
    }

    #[inline]
    fn visit_sequence_expression(&mut self, it: &SequenceExpression<'a>) {
        self.visit_expressions(&it.expressions);
    }

    #[inline(always)]
    fn visit_super(&mut self, it: &Super) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_await_expression(&mut self, it: &AwaitExpression<'a>) {
        self.visit_expression(&it.argument);
    }

    #[inline]
    fn visit_chain_expression(&mut self, it: &ChainExpression<'a>) {
        self.visit_chain_element(&it.expression);
    }

    #[inline]
    fn visit_parenthesized_expression(&mut self, it: &ParenthesizedExpression<'a>) {
        self.visit_expression(&it.expression);
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        match it {
            Statement::BlockStatement(it) => self.visit_block_statement(it),
            Statement::DoWhileStatement(it) => self.visit_do_while_statement(it),
            Statement::ExpressionStatement(it) => self.visit_expression_statement(it),
            Statement::ForInStatement(it) => self.visit_for_in_statement(it),
            Statement::ForOfStatement(it) => self.visit_for_of_statement(it),
            Statement::ForStatement(it) => self.visit_for_statement(it),
            Statement::IfStatement(it) => self.visit_if_statement(it),
            Statement::LabeledStatement(it) => self.visit_labeled_statement(it),
            Statement::ReturnStatement(it) => self.visit_return_statement(it),
            Statement::SwitchStatement(it) => self.visit_switch_statement(it),
            Statement::ThrowStatement(it) => self.visit_throw_statement(it),
            Statement::TryStatement(it) => self.visit_try_statement(it),
            Statement::WhileStatement(it) => self.visit_while_statement(it),
            Statement::WithStatement(it) => self.visit_with_statement(it),
            Statement::VariableDeclaration(it) => self.visit_variable_declaration(it),
            Statement::FunctionDeclaration(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            Statement::ClassDeclaration(it) => self.visit_class(it),
            Statement::TSTypeAliasDeclaration(it) => self.visit_ts_type_alias_declaration(it),
            Statement::TSInterfaceDeclaration(it) => self.visit_ts_interface_declaration(it),
            Statement::TSEnumDeclaration(it) => self.visit_ts_enum_declaration(it),
            Statement::TSModuleDeclaration(it) => self.visit_ts_module_declaration(it),
            Statement::TSGlobalDeclaration(it) => self.visit_ts_global_declaration(it),
            Statement::ExportDefaultDeclaration(it) => self.visit_export_default_declaration(it),
            Statement::ExportNamedDeclaration(it) => self.visit_export_named_declaration(it),
            Statement::TSExportAssignment(it) => self.visit_ts_export_assignment(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `BreakStatement`
                // `ContinueStatement`
                // `DebuggerStatement`
                // `EmptyStatement`
                // `TSImportEqualsDeclaration`
                // `ImportDeclaration`
                // `ExportAllDeclaration`
                // `TSNamespaceExportDeclaration`
            }
        }
    }

    #[inline(always)]
    fn visit_directive(&mut self, it: &Directive<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_hashbang(&mut self, it: &Hashbang<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        self.add_scope(&it.scope_id);
    }

    fn visit_declaration(&mut self, it: &Declaration<'a>) {
        match it {
            Declaration::VariableDeclaration(it) => self.visit_variable_declaration(it),
            Declaration::FunctionDeclaration(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            Declaration::ClassDeclaration(it) => self.visit_class(it),
            Declaration::TSTypeAliasDeclaration(it) => self.visit_ts_type_alias_declaration(it),
            Declaration::TSInterfaceDeclaration(it) => self.visit_ts_interface_declaration(it),
            Declaration::TSEnumDeclaration(it) => self.visit_ts_enum_declaration(it),
            Declaration::TSModuleDeclaration(it) => self.visit_ts_module_declaration(it),
            Declaration::TSGlobalDeclaration(it) => self.visit_ts_global_declaration(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `TSImportEqualsDeclaration`
            }
        }
    }

    #[inline]
    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        self.visit_variable_declarators(&it.declarations);
    }

    #[inline]
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        self.visit_binding_pattern(&it.id);
        if let Some(init) = &it.init {
            self.visit_expression(init);
        }
    }

    #[inline(always)]
    fn visit_empty_statement(&mut self, it: &EmptyStatement) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_expression_statement(&mut self, it: &ExpressionStatement<'a>) {
        self.visit_expression(&it.expression);
    }

    #[inline]
    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        self.visit_expression(&it.test);
        self.visit_statement(&it.consequent);
        if let Some(alternate) = &it.alternate {
            self.visit_statement(alternate);
        }
    }

    #[inline]
    fn visit_do_while_statement(&mut self, it: &DoWhileStatement<'a>) {
        self.visit_statement(&it.body);
        self.visit_expression(&it.test);
    }

    #[inline]
    fn visit_while_statement(&mut self, it: &WhileStatement<'a>) {
        self.visit_expression(&it.test);
        self.visit_statement(&it.body);
    }

    #[inline]
    fn visit_for_statement(&mut self, it: &ForStatement<'a>) {
        self.add_scope(&it.scope_id);
    }

    fn visit_for_statement_init(&mut self, it: &ForStatementInit<'a>) {
        match it {
            ForStatementInit::VariableDeclaration(it) => self.visit_variable_declaration(it),
            ForStatementInit::TemplateLiteral(it) => self.visit_template_literal(it),
            ForStatementInit::ArrayExpression(it) => self.visit_array_expression(it),
            ForStatementInit::ArrowFunctionExpression(it) => {
                self.visit_arrow_function_expression(it)
            }
            ForStatementInit::AssignmentExpression(it) => self.visit_assignment_expression(it),
            ForStatementInit::AwaitExpression(it) => self.visit_await_expression(it),
            ForStatementInit::BinaryExpression(it) => self.visit_binary_expression(it),
            ForStatementInit::CallExpression(it) => self.visit_call_expression(it),
            ForStatementInit::ChainExpression(it) => self.visit_chain_expression(it),
            ForStatementInit::ClassExpression(it) => self.visit_class(it),
            ForStatementInit::ConditionalExpression(it) => self.visit_conditional_expression(it),
            ForStatementInit::FunctionExpression(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            ForStatementInit::ImportExpression(it) => self.visit_import_expression(it),
            ForStatementInit::LogicalExpression(it) => self.visit_logical_expression(it),
            ForStatementInit::NewExpression(it) => self.visit_new_expression(it),
            ForStatementInit::ObjectExpression(it) => self.visit_object_expression(it),
            ForStatementInit::ParenthesizedExpression(it) => {
                self.visit_parenthesized_expression(it)
            }
            ForStatementInit::SequenceExpression(it) => self.visit_sequence_expression(it),
            ForStatementInit::TaggedTemplateExpression(it) => {
                self.visit_tagged_template_expression(it)
            }
            ForStatementInit::UnaryExpression(it) => self.visit_unary_expression(it),
            ForStatementInit::UpdateExpression(it) => self.visit_update_expression(it),
            ForStatementInit::YieldExpression(it) => self.visit_yield_expression(it),
            ForStatementInit::PrivateInExpression(it) => self.visit_private_in_expression(it),
            ForStatementInit::JSXElement(it) => self.visit_jsx_element(it),
            ForStatementInit::JSXFragment(it) => self.visit_jsx_fragment(it),
            ForStatementInit::TSAsExpression(it) => self.visit_ts_as_expression(it),
            ForStatementInit::TSSatisfiesExpression(it) => self.visit_ts_satisfies_expression(it),
            ForStatementInit::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            ForStatementInit::TSNonNullExpression(it) => self.visit_ts_non_null_expression(it),
            ForStatementInit::TSInstantiationExpression(it) => {
                self.visit_ts_instantiation_expression(it)
            }
            ForStatementInit::V8IntrinsicExpression(it) => self.visit_v_8_intrinsic_expression(it),
            ForStatementInit::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it)
            }
            ForStatementInit::StaticMemberExpression(it) => self.visit_static_member_expression(it),
            ForStatementInit::PrivateFieldExpression(it) => self.visit_private_field_expression(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `BooleanLiteral`
                // `NullLiteral`
                // `NumericLiteral`
                // `BigIntLiteral`
                // `RegExpLiteral`
                // `StringLiteral`
                // `Identifier`
                // `MetaProperty`
                // `Super`
                // `ThisExpression`
            }
        }
    }

    #[inline]
    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        self.add_scope(&it.scope_id);
    }

    fn visit_for_statement_left(&mut self, it: &ForStatementLeft<'a>) {
        match it {
            ForStatementLeft::VariableDeclaration(it) => self.visit_variable_declaration(it),
            ForStatementLeft::TSAsExpression(it) => self.visit_ts_as_expression(it),
            ForStatementLeft::TSSatisfiesExpression(it) => self.visit_ts_satisfies_expression(it),
            ForStatementLeft::TSNonNullExpression(it) => self.visit_ts_non_null_expression(it),
            ForStatementLeft::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            ForStatementLeft::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it)
            }
            ForStatementLeft::StaticMemberExpression(it) => self.visit_static_member_expression(it),
            ForStatementLeft::PrivateFieldExpression(it) => self.visit_private_field_expression(it),
            ForStatementLeft::ArrayAssignmentTarget(it) => self.visit_array_assignment_target(it),
            ForStatementLeft::ObjectAssignmentTarget(it) => self.visit_object_assignment_target(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `AssignmentTargetIdentifier`
            }
        }
    }

    #[inline]
    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline(always)]
    fn visit_continue_statement(&mut self, it: &ContinueStatement<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_break_statement(&mut self, it: &BreakStatement<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        if let Some(argument) = &it.argument {
            self.visit_expression(argument);
        }
    }

    #[inline]
    fn visit_with_statement(&mut self, it: &WithStatement<'a>) {
        self.visit_expression(&it.object);
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_switch_statement(&mut self, it: &SwitchStatement<'a>) {
        self.visit_expression(&it.discriminant);
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_switch_case(&mut self, it: &SwitchCase<'a>) {
        if let Some(test) = &it.test {
            self.visit_expression(test);
        }
        self.visit_statements(&it.consequent);
    }

    #[inline]
    fn visit_labeled_statement(&mut self, it: &LabeledStatement<'a>) {
        self.visit_statement(&it.body);
    }

    #[inline]
    fn visit_throw_statement(&mut self, it: &ThrowStatement<'a>) {
        self.visit_expression(&it.argument);
    }

    #[inline]
    fn visit_try_statement(&mut self, it: &TryStatement<'a>) {
        self.visit_block_statement(&it.block);
        if let Some(handler) = &it.handler {
            self.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &it.finalizer {
            self.visit_block_statement(finalizer);
        }
    }

    #[inline]
    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_catch_parameter(&mut self, it: &CatchParameter<'a>) {
        self.visit_binding_pattern(&it.pattern);
    }

    #[inline(always)]
    fn visit_debugger_statement(&mut self, it: &DebuggerStatement) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_binding_pattern(&mut self, it: &BindingPattern<'a>) {
        self.visit_binding_pattern_kind(&it.kind);
        if let Some(type_annotation) = &it.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    fn visit_binding_pattern_kind(&mut self, it: &BindingPatternKind<'a>) {
        match it {
            BindingPatternKind::ObjectPattern(it) => self.visit_object_pattern(it),
            BindingPatternKind::ArrayPattern(it) => self.visit_array_pattern(it),
            BindingPatternKind::AssignmentPattern(it) => self.visit_assignment_pattern(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `BindingIdentifier`
            }
        }
    }

    #[inline]
    fn visit_assignment_pattern(&mut self, it: &AssignmentPattern<'a>) {
        self.visit_binding_pattern(&it.left);
        self.visit_expression(&it.right);
    }

    #[inline]
    fn visit_object_pattern(&mut self, it: &ObjectPattern<'a>) {
        self.visit_binding_properties(&it.properties);
        if let Some(rest) = &it.rest {
            self.visit_binding_rest_element(rest);
        }
    }

    #[inline]
    fn visit_binding_property(&mut self, it: &BindingProperty<'a>) {
        self.visit_property_key(&it.key);
        self.visit_binding_pattern(&it.value);
    }

    #[inline]
    fn visit_array_pattern(&mut self, it: &ArrayPattern<'a>) {
        for el in it.elements.iter().flatten() {
            self.visit_binding_pattern(el);
        }
        if let Some(rest) = &it.rest {
            self.visit_binding_rest_element(rest);
        }
    }

    #[inline]
    fn visit_binding_rest_element(&mut self, it: &BindingRestElement<'a>) {
        self.visit_binding_pattern(&it.argument);
    }

    #[inline]
    fn visit_function(&mut self, it: &Function<'a>, _: ScopeFlags) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_formal_parameters(&mut self, it: &FormalParameters<'a>) {
        self.visit_formal_parameter_list(&it.items);
        if let Some(rest) = &it.rest {
            self.visit_binding_rest_element(rest);
        }
    }

    #[inline]
    fn visit_formal_parameter(&mut self, it: &FormalParameter<'a>) {
        self.visit_decorators(&it.decorators);
        self.visit_binding_pattern(&it.pattern);
    }

    #[inline]
    fn visit_function_body(&mut self, it: &FunctionBody<'a>) {
        self.visit_statements(&it.statements);
    }

    #[inline]
    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_yield_expression(&mut self, it: &YieldExpression<'a>) {
        if let Some(argument) = &it.argument {
            self.visit_expression(argument);
        }
    }

    #[inline]
    fn visit_class(&mut self, it: &Class<'a>) {
        self.visit_decorators(&it.decorators);
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_class_body(&mut self, it: &ClassBody<'a>) {
        self.visit_class_elements(&it.body);
    }

    #[inline]
    fn visit_method_definition(&mut self, it: &MethodDefinition<'a>) {
        self.visit_decorators(&it.decorators);
        self.visit_property_key(&it.key);
        {
            let flags = match it.kind {
                MethodDefinitionKind::Get => ScopeFlags::Function | ScopeFlags::GetAccessor,
                MethodDefinitionKind::Set => ScopeFlags::Function | ScopeFlags::SetAccessor,
                MethodDefinitionKind::Constructor => ScopeFlags::Function | ScopeFlags::Constructor,
                MethodDefinitionKind::Method => ScopeFlags::Function,
            };
            self.visit_function(&it.value, flags);
        }
    }

    #[inline]
    fn visit_property_definition(&mut self, it: &PropertyDefinition<'a>) {
        self.visit_decorators(&it.decorators);
        self.visit_property_key(&it.key);
        if let Some(type_annotation) = &it.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
        if let Some(value) = &it.value {
            self.visit_expression(value);
        }
    }

    #[inline(always)]
    fn visit_private_identifier(&mut self, it: &PrivateIdentifier<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_static_block(&mut self, it: &StaticBlock<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_module_declaration(&mut self, it: &ModuleDeclaration<'a>) {
        match it {
            ModuleDeclaration::ExportDefaultDeclaration(it) => {
                self.visit_export_default_declaration(it)
            }
            ModuleDeclaration::ExportNamedDeclaration(it) => {
                self.visit_export_named_declaration(it)
            }
            ModuleDeclaration::TSExportAssignment(it) => self.visit_ts_export_assignment(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `ImportDeclaration`
                // `ExportAllDeclaration`
                // `TSNamespaceExportDeclaration`
            }
        }
    }

    #[inline]
    fn visit_accessor_property(&mut self, it: &AccessorProperty<'a>) {
        self.visit_decorators(&it.decorators);
        self.visit_property_key(&it.key);
        if let Some(type_annotation) = &it.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
        if let Some(value) = &it.value {
            self.visit_expression(value);
        }
    }

    #[inline]
    fn visit_import_expression(&mut self, it: &ImportExpression<'a>) {
        self.visit_expression(&it.source);
        if let Some(options) = &it.options {
            self.visit_expression(options);
        }
    }

    #[inline(always)]
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_import_declaration_specifier(&mut self, it: &ImportDeclarationSpecifier<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_import_specifier(&mut self, it: &ImportSpecifier<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_import_default_specifier(&mut self, it: &ImportDefaultSpecifier<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_import_namespace_specifier(&mut self, it: &ImportNamespaceSpecifier<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_with_clause(&mut self, it: &WithClause<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_import_attribute(&mut self, it: &ImportAttribute<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_import_attribute_key(&mut self, it: &ImportAttributeKey<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
        if let Some(declaration) = &it.declaration {
            self.visit_declaration(declaration);
        }
    }

    #[inline]
    fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
        self.visit_export_default_declaration_kind(&it.declaration);
    }

    #[inline(always)]
    fn visit_export_all_declaration(&mut self, it: &ExportAllDeclaration<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_export_specifier(&mut self, it: &ExportSpecifier<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    fn visit_export_default_declaration_kind(&mut self, it: &ExportDefaultDeclarationKind<'a>) {
        match it {
            ExportDefaultDeclarationKind::FunctionDeclaration(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(it) => self.visit_class(it),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(it) => {
                self.visit_ts_interface_declaration(it)
            }
            ExportDefaultDeclarationKind::TemplateLiteral(it) => self.visit_template_literal(it),
            ExportDefaultDeclarationKind::ArrayExpression(it) => self.visit_array_expression(it),
            ExportDefaultDeclarationKind::ArrowFunctionExpression(it) => {
                self.visit_arrow_function_expression(it)
            }
            ExportDefaultDeclarationKind::AssignmentExpression(it) => {
                self.visit_assignment_expression(it)
            }
            ExportDefaultDeclarationKind::AwaitExpression(it) => self.visit_await_expression(it),
            ExportDefaultDeclarationKind::BinaryExpression(it) => self.visit_binary_expression(it),
            ExportDefaultDeclarationKind::CallExpression(it) => self.visit_call_expression(it),
            ExportDefaultDeclarationKind::ChainExpression(it) => self.visit_chain_expression(it),
            ExportDefaultDeclarationKind::ClassExpression(it) => self.visit_class(it),
            ExportDefaultDeclarationKind::ConditionalExpression(it) => {
                self.visit_conditional_expression(it)
            }
            ExportDefaultDeclarationKind::FunctionExpression(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            ExportDefaultDeclarationKind::ImportExpression(it) => self.visit_import_expression(it),
            ExportDefaultDeclarationKind::LogicalExpression(it) => {
                self.visit_logical_expression(it)
            }
            ExportDefaultDeclarationKind::NewExpression(it) => self.visit_new_expression(it),
            ExportDefaultDeclarationKind::ObjectExpression(it) => self.visit_object_expression(it),
            ExportDefaultDeclarationKind::ParenthesizedExpression(it) => {
                self.visit_parenthesized_expression(it)
            }
            ExportDefaultDeclarationKind::SequenceExpression(it) => {
                self.visit_sequence_expression(it)
            }
            ExportDefaultDeclarationKind::TaggedTemplateExpression(it) => {
                self.visit_tagged_template_expression(it)
            }
            ExportDefaultDeclarationKind::UnaryExpression(it) => self.visit_unary_expression(it),
            ExportDefaultDeclarationKind::UpdateExpression(it) => self.visit_update_expression(it),
            ExportDefaultDeclarationKind::YieldExpression(it) => self.visit_yield_expression(it),
            ExportDefaultDeclarationKind::PrivateInExpression(it) => {
                self.visit_private_in_expression(it)
            }
            ExportDefaultDeclarationKind::JSXElement(it) => self.visit_jsx_element(it),
            ExportDefaultDeclarationKind::JSXFragment(it) => self.visit_jsx_fragment(it),
            ExportDefaultDeclarationKind::TSAsExpression(it) => self.visit_ts_as_expression(it),
            ExportDefaultDeclarationKind::TSSatisfiesExpression(it) => {
                self.visit_ts_satisfies_expression(it)
            }
            ExportDefaultDeclarationKind::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            ExportDefaultDeclarationKind::TSNonNullExpression(it) => {
                self.visit_ts_non_null_expression(it)
            }
            ExportDefaultDeclarationKind::TSInstantiationExpression(it) => {
                self.visit_ts_instantiation_expression(it)
            }
            ExportDefaultDeclarationKind::V8IntrinsicExpression(it) => {
                self.visit_v_8_intrinsic_expression(it)
            }
            ExportDefaultDeclarationKind::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it)
            }
            ExportDefaultDeclarationKind::StaticMemberExpression(it) => {
                self.visit_static_member_expression(it)
            }
            ExportDefaultDeclarationKind::PrivateFieldExpression(it) => {
                self.visit_private_field_expression(it)
            }
            _ => {
                // Remaining variants do not contain scopes:
                // `BooleanLiteral`
                // `NullLiteral`
                // `NumericLiteral`
                // `BigIntLiteral`
                // `RegExpLiteral`
                // `StringLiteral`
                // `Identifier`
                // `MetaProperty`
                // `Super`
                // `ThisExpression`
            }
        }
    }

    #[inline(always)]
    fn visit_module_export_name(&mut self, it: &ModuleExportName<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_v_8_intrinsic_expression(&mut self, it: &V8IntrinsicExpression<'a>) {
        self.visit_arguments(&it.arguments);
    }

    #[inline(always)]
    fn visit_boolean_literal(&mut self, it: &BooleanLiteral) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_null_literal(&mut self, it: &NullLiteral) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_numeric_literal(&mut self, it: &NumericLiteral<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_big_int_literal(&mut self, it: &BigIntLiteral<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_reg_exp_literal(&mut self, it: &RegExpLiteral<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
        self.visit_jsx_opening_element(&it.opening_element);
        self.visit_jsx_children(&it.children);
    }

    #[inline]
    fn visit_jsx_opening_element(&mut self, it: &JSXOpeningElement<'a>) {
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
        self.visit_jsx_attribute_items(&it.attributes);
    }

    #[inline(always)]
    fn visit_jsx_closing_element(&mut self, it: &JSXClosingElement<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
        self.visit_jsx_children(&it.children);
    }

    #[inline(always)]
    fn visit_jsx_opening_fragment(&mut self, it: &JSXOpeningFragment) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_jsx_closing_fragment(&mut self, it: &JSXClosingFragment) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_jsx_element_name(&mut self, it: &JSXElementName<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_jsx_namespaced_name(&mut self, it: &JSXNamespacedName<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_jsx_member_expression(&mut self, it: &JSXMemberExpression<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_jsx_member_expression_object(&mut self, it: &JSXMemberExpressionObject<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_jsx_expression_container(&mut self, it: &JSXExpressionContainer<'a>) {
        self.visit_jsx_expression(&it.expression);
    }

    fn visit_jsx_expression(&mut self, it: &JSXExpression<'a>) {
        match it {
            JSXExpression::TemplateLiteral(it) => self.visit_template_literal(it),
            JSXExpression::ArrayExpression(it) => self.visit_array_expression(it),
            JSXExpression::ArrowFunctionExpression(it) => self.visit_arrow_function_expression(it),
            JSXExpression::AssignmentExpression(it) => self.visit_assignment_expression(it),
            JSXExpression::AwaitExpression(it) => self.visit_await_expression(it),
            JSXExpression::BinaryExpression(it) => self.visit_binary_expression(it),
            JSXExpression::CallExpression(it) => self.visit_call_expression(it),
            JSXExpression::ChainExpression(it) => self.visit_chain_expression(it),
            JSXExpression::ClassExpression(it) => self.visit_class(it),
            JSXExpression::ConditionalExpression(it) => self.visit_conditional_expression(it),
            JSXExpression::FunctionExpression(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags)
            }
            JSXExpression::ImportExpression(it) => self.visit_import_expression(it),
            JSXExpression::LogicalExpression(it) => self.visit_logical_expression(it),
            JSXExpression::NewExpression(it) => self.visit_new_expression(it),
            JSXExpression::ObjectExpression(it) => self.visit_object_expression(it),
            JSXExpression::ParenthesizedExpression(it) => self.visit_parenthesized_expression(it),
            JSXExpression::SequenceExpression(it) => self.visit_sequence_expression(it),
            JSXExpression::TaggedTemplateExpression(it) => {
                self.visit_tagged_template_expression(it)
            }
            JSXExpression::UnaryExpression(it) => self.visit_unary_expression(it),
            JSXExpression::UpdateExpression(it) => self.visit_update_expression(it),
            JSXExpression::YieldExpression(it) => self.visit_yield_expression(it),
            JSXExpression::PrivateInExpression(it) => self.visit_private_in_expression(it),
            JSXExpression::JSXElement(it) => self.visit_jsx_element(it),
            JSXExpression::JSXFragment(it) => self.visit_jsx_fragment(it),
            JSXExpression::TSAsExpression(it) => self.visit_ts_as_expression(it),
            JSXExpression::TSSatisfiesExpression(it) => self.visit_ts_satisfies_expression(it),
            JSXExpression::TSTypeAssertion(it) => self.visit_ts_type_assertion(it),
            JSXExpression::TSNonNullExpression(it) => self.visit_ts_non_null_expression(it),
            JSXExpression::TSInstantiationExpression(it) => {
                self.visit_ts_instantiation_expression(it)
            }
            JSXExpression::V8IntrinsicExpression(it) => self.visit_v_8_intrinsic_expression(it),
            JSXExpression::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it)
            }
            JSXExpression::StaticMemberExpression(it) => self.visit_static_member_expression(it),
            JSXExpression::PrivateFieldExpression(it) => self.visit_private_field_expression(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `EmptyExpression`
                // `BooleanLiteral`
                // `NullLiteral`
                // `NumericLiteral`
                // `BigIntLiteral`
                // `RegExpLiteral`
                // `StringLiteral`
                // `Identifier`
                // `MetaProperty`
                // `Super`
                // `ThisExpression`
            }
        }
    }

    #[inline(always)]
    fn visit_jsx_empty_expression(&mut self, it: &JSXEmptyExpression) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_jsx_attribute(&mut self, it: &JSXAttribute<'a>) {
        if let Some(value) = &it.value {
            self.visit_jsx_attribute_value(value);
        }
    }

    #[inline]
    fn visit_jsx_spread_attribute(&mut self, it: &JSXSpreadAttribute<'a>) {
        self.visit_expression(&it.argument);
    }

    #[inline(always)]
    fn visit_jsx_attribute_name(&mut self, it: &JSXAttributeName<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_jsx_attribute_value(&mut self, it: &JSXAttributeValue<'a>) {
        match it {
            JSXAttributeValue::ExpressionContainer(it) => self.visit_jsx_expression_container(it),
            JSXAttributeValue::Element(it) => self.visit_jsx_element(it),
            JSXAttributeValue::Fragment(it) => self.visit_jsx_fragment(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `StringLiteral`
            }
        }
    }

    #[inline(always)]
    fn visit_jsx_identifier(&mut self, it: &JSXIdentifier<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_jsx_child(&mut self, it: &JSXChild<'a>) {
        match it {
            JSXChild::Element(it) => self.visit_jsx_element(it),
            JSXChild::Fragment(it) => self.visit_jsx_fragment(it),
            JSXChild::ExpressionContainer(it) => self.visit_jsx_expression_container(it),
            JSXChild::Spread(it) => self.visit_jsx_spread_child(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `Text`
            }
        }
    }

    #[inline]
    fn visit_jsx_spread_child(&mut self, it: &JSXSpreadChild<'a>) {
        self.visit_expression(&it.expression);
    }

    #[inline(always)]
    fn visit_jsx_text(&mut self, it: &JSXText<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_ts_this_parameter(&mut self, it: &TSThisParameter<'a>) {
        if let Some(type_annotation) = &it.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    fn visit_ts_enum_declaration(&mut self, it: &TSEnumDeclaration<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_enum_body(&mut self, it: &TSEnumBody<'a>) {
        self.visit_ts_enum_members(&it.members);
    }

    #[inline]
    fn visit_ts_enum_member(&mut self, it: &TSEnumMember<'a>) {
        self.visit_ts_enum_member_name(&it.id);
        if let Some(initializer) = &it.initializer {
            self.visit_expression(initializer);
        }
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        match it {
            TSEnumMemberName::ComputedTemplateString(it) => self.visit_template_literal(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `Identifier`
                // `String`
                // `ComputedString`
            }
        }
    }

    #[inline]
    fn visit_ts_type_annotation(&mut self, it: &TSTypeAnnotation<'a>) {
        self.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    fn visit_ts_literal_type(&mut self, it: &TSLiteralType<'a>) {
        self.visit_ts_literal(&it.literal);
    }

    #[inline]
    fn visit_ts_literal(&mut self, it: &TSLiteral<'a>) {
        match it {
            TSLiteral::TemplateLiteral(it) => self.visit_template_literal(it),
            TSLiteral::UnaryExpression(it) => self.visit_unary_expression(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `BooleanLiteral`
                // `NumericLiteral`
                // `BigIntLiteral`
                // `StringLiteral`
            }
        }
    }

    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        match it {
            TSType::TSArrayType(it) => self.visit_ts_array_type(it),
            TSType::TSConditionalType(it) => self.visit_ts_conditional_type(it),
            TSType::TSConstructorType(it) => self.visit_ts_constructor_type(it),
            TSType::TSFunctionType(it) => self.visit_ts_function_type(it),
            TSType::TSImportType(it) => self.visit_ts_import_type(it),
            TSType::TSIndexedAccessType(it) => self.visit_ts_indexed_access_type(it),
            TSType::TSInferType(it) => self.visit_ts_infer_type(it),
            TSType::TSIntersectionType(it) => self.visit_ts_intersection_type(it),
            TSType::TSLiteralType(it) => self.visit_ts_literal_type(it),
            TSType::TSMappedType(it) => self.visit_ts_mapped_type(it),
            TSType::TSNamedTupleMember(it) => self.visit_ts_named_tuple_member(it),
            TSType::TSTemplateLiteralType(it) => self.visit_ts_template_literal_type(it),
            TSType::TSTupleType(it) => self.visit_ts_tuple_type(it),
            TSType::TSTypeLiteral(it) => self.visit_ts_type_literal(it),
            TSType::TSTypeOperatorType(it) => self.visit_ts_type_operator(it),
            TSType::TSTypePredicate(it) => self.visit_ts_type_predicate(it),
            TSType::TSTypeQuery(it) => self.visit_ts_type_query(it),
            TSType::TSTypeReference(it) => self.visit_ts_type_reference(it),
            TSType::TSUnionType(it) => self.visit_ts_union_type(it),
            TSType::TSParenthesizedType(it) => self.visit_ts_parenthesized_type(it),
            TSType::JSDocNullableType(it) => self.visit_js_doc_nullable_type(it),
            TSType::JSDocNonNullableType(it) => self.visit_js_doc_non_nullable_type(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `TSAnyKeyword`
                // `TSBigIntKeyword`
                // `TSBooleanKeyword`
                // `TSIntrinsicKeyword`
                // `TSNeverKeyword`
                // `TSNullKeyword`
                // `TSNumberKeyword`
                // `TSObjectKeyword`
                // `TSStringKeyword`
                // `TSSymbolKeyword`
                // `TSUndefinedKeyword`
                // `TSUnknownKeyword`
                // `TSVoidKeyword`
                // `TSThisType`
                // `JSDocUnknownType`
            }
        }
    }

    #[inline]
    fn visit_ts_conditional_type(&mut self, it: &TSConditionalType<'a>) {
        self.visit_ts_type(&it.check_type);
        self.add_scope(&it.scope_id);
        self.visit_ts_type(&it.false_type);
    }

    #[inline]
    fn visit_ts_union_type(&mut self, it: &TSUnionType<'a>) {
        self.visit_ts_types(&it.types);
    }

    #[inline]
    fn visit_ts_intersection_type(&mut self, it: &TSIntersectionType<'a>) {
        self.visit_ts_types(&it.types);
    }

    #[inline]
    fn visit_ts_parenthesized_type(&mut self, it: &TSParenthesizedType<'a>) {
        self.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    fn visit_ts_type_operator(&mut self, it: &TSTypeOperator<'a>) {
        self.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    fn visit_ts_array_type(&mut self, it: &TSArrayType<'a>) {
        self.visit_ts_type(&it.element_type);
    }

    #[inline]
    fn visit_ts_indexed_access_type(&mut self, it: &TSIndexedAccessType<'a>) {
        self.visit_ts_type(&it.object_type);
        self.visit_ts_type(&it.index_type);
    }

    #[inline]
    fn visit_ts_tuple_type(&mut self, it: &TSTupleType<'a>) {
        self.visit_ts_tuple_elements(&it.element_types);
    }

    #[inline]
    fn visit_ts_named_tuple_member(&mut self, it: &TSNamedTupleMember<'a>) {
        self.visit_ts_tuple_element(&it.element_type);
    }

    #[inline]
    fn visit_ts_optional_type(&mut self, it: &TSOptionalType<'a>) {
        self.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    fn visit_ts_rest_type(&mut self, it: &TSRestType<'a>) {
        self.visit_ts_type(&it.type_annotation);
    }

    fn visit_ts_tuple_element(&mut self, it: &TSTupleElement<'a>) {
        match it {
            TSTupleElement::TSOptionalType(it) => self.visit_ts_optional_type(it),
            TSTupleElement::TSRestType(it) => self.visit_ts_rest_type(it),
            TSTupleElement::TSArrayType(it) => self.visit_ts_array_type(it),
            TSTupleElement::TSConditionalType(it) => self.visit_ts_conditional_type(it),
            TSTupleElement::TSConstructorType(it) => self.visit_ts_constructor_type(it),
            TSTupleElement::TSFunctionType(it) => self.visit_ts_function_type(it),
            TSTupleElement::TSImportType(it) => self.visit_ts_import_type(it),
            TSTupleElement::TSIndexedAccessType(it) => self.visit_ts_indexed_access_type(it),
            TSTupleElement::TSInferType(it) => self.visit_ts_infer_type(it),
            TSTupleElement::TSIntersectionType(it) => self.visit_ts_intersection_type(it),
            TSTupleElement::TSLiteralType(it) => self.visit_ts_literal_type(it),
            TSTupleElement::TSMappedType(it) => self.visit_ts_mapped_type(it),
            TSTupleElement::TSNamedTupleMember(it) => self.visit_ts_named_tuple_member(it),
            TSTupleElement::TSTemplateLiteralType(it) => self.visit_ts_template_literal_type(it),
            TSTupleElement::TSTupleType(it) => self.visit_ts_tuple_type(it),
            TSTupleElement::TSTypeLiteral(it) => self.visit_ts_type_literal(it),
            TSTupleElement::TSTypeOperatorType(it) => self.visit_ts_type_operator(it),
            TSTupleElement::TSTypePredicate(it) => self.visit_ts_type_predicate(it),
            TSTupleElement::TSTypeQuery(it) => self.visit_ts_type_query(it),
            TSTupleElement::TSTypeReference(it) => self.visit_ts_type_reference(it),
            TSTupleElement::TSUnionType(it) => self.visit_ts_union_type(it),
            TSTupleElement::TSParenthesizedType(it) => self.visit_ts_parenthesized_type(it),
            TSTupleElement::JSDocNullableType(it) => self.visit_js_doc_nullable_type(it),
            TSTupleElement::JSDocNonNullableType(it) => self.visit_js_doc_non_nullable_type(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `TSAnyKeyword`
                // `TSBigIntKeyword`
                // `TSBooleanKeyword`
                // `TSIntrinsicKeyword`
                // `TSNeverKeyword`
                // `TSNullKeyword`
                // `TSNumberKeyword`
                // `TSObjectKeyword`
                // `TSStringKeyword`
                // `TSSymbolKeyword`
                // `TSUndefinedKeyword`
                // `TSUnknownKeyword`
                // `TSVoidKeyword`
                // `TSThisType`
                // `JSDocUnknownType`
            }
        }
    }

    #[inline(always)]
    fn visit_ts_any_keyword(&mut self, it: &TSAnyKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_string_keyword(&mut self, it: &TSStringKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_boolean_keyword(&mut self, it: &TSBooleanKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_number_keyword(&mut self, it: &TSNumberKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_never_keyword(&mut self, it: &TSNeverKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_intrinsic_keyword(&mut self, it: &TSIntrinsicKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_unknown_keyword(&mut self, it: &TSUnknownKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_null_keyword(&mut self, it: &TSNullKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_undefined_keyword(&mut self, it: &TSUndefinedKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_void_keyword(&mut self, it: &TSVoidKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_symbol_keyword(&mut self, it: &TSSymbolKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_this_type(&mut self, it: &TSThisType) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_object_keyword(&mut self, it: &TSObjectKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_big_int_keyword(&mut self, it: &TSBigIntKeyword) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_ts_type_reference(&mut self, it: &TSTypeReference<'a>) {
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    #[inline(always)]
    fn visit_ts_type_name(&mut self, it: &TSTypeName<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_qualified_name(&mut self, it: &TSQualifiedName<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_ts_type_parameter_instantiation(&mut self, it: &TSTypeParameterInstantiation<'a>) {
        self.visit_ts_types(&it.params);
    }

    #[inline]
    fn visit_ts_type_parameter(&mut self, it: &TSTypeParameter<'a>) {
        if let Some(constraint) = &it.constraint {
            self.visit_ts_type(constraint);
        }
        if let Some(default) = &it.default {
            self.visit_ts_type(default);
        }
    }

    #[inline]
    fn visit_ts_type_parameter_declaration(&mut self, it: &TSTypeParameterDeclaration<'a>) {
        self.visit_ts_type_parameters(&it.params);
    }

    #[inline]
    fn visit_ts_type_alias_declaration(&mut self, it: &TSTypeAliasDeclaration<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_class_implements(&mut self, it: &TSClassImplements<'a>) {
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    #[inline]
    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_interface_body(&mut self, it: &TSInterfaceBody<'a>) {
        self.visit_ts_signatures(&it.body);
    }

    #[inline]
    fn visit_ts_property_signature(&mut self, it: &TSPropertySignature<'a>) {
        self.visit_property_key(&it.key);
        if let Some(type_annotation) = &it.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline]
    fn visit_ts_index_signature(&mut self, it: &TSIndexSignature<'a>) {
        self.visit_ts_index_signature_names(&it.parameters);
        self.visit_ts_type_annotation(&it.type_annotation);
    }

    #[inline]
    fn visit_ts_call_signature_declaration(&mut self, it: &TSCallSignatureDeclaration<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_method_signature(&mut self, it: &TSMethodSignature<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_construct_signature_declaration(
        &mut self,
        it: &TSConstructSignatureDeclaration<'a>,
    ) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_index_signature_name(&mut self, it: &TSIndexSignatureName<'a>) {
        self.visit_ts_type_annotation(&it.type_annotation);
    }

    #[inline]
    fn visit_ts_interface_heritage(&mut self, it: &TSInterfaceHeritage<'a>) {
        self.visit_expression(&it.expression);
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    #[inline]
    fn visit_ts_type_predicate(&mut self, it: &TSTypePredicate<'a>) {
        if let Some(type_annotation) = &it.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    #[inline(always)]
    fn visit_ts_type_predicate_name(&mut self, it: &TSTypePredicateName<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_ts_module_declaration(&mut self, it: &TSModuleDeclaration<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline(always)]
    fn visit_ts_module_declaration_name(&mut self, it: &TSModuleDeclarationName<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_ts_global_declaration(&mut self, it: &TSGlobalDeclaration<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_module_block(&mut self, it: &TSModuleBlock<'a>) {
        self.visit_statements(&it.body);
    }

    #[inline]
    fn visit_ts_type_literal(&mut self, it: &TSTypeLiteral<'a>) {
        self.visit_ts_signatures(&it.members);
    }

    #[inline]
    fn visit_ts_infer_type(&mut self, it: &TSInferType<'a>) {
        self.visit_ts_type_parameter(&it.type_parameter);
    }

    #[inline]
    fn visit_ts_type_query(&mut self, it: &TSTypeQuery<'a>) {
        self.visit_ts_type_query_expr_name(&it.expr_name);
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    #[inline]
    fn visit_ts_type_query_expr_name(&mut self, it: &TSTypeQueryExprName<'a>) {
        match it {
            TSTypeQueryExprName::TSImportType(it) => self.visit_ts_import_type(it),
            _ => {
                // Remaining variants do not contain scopes:
                // `IdentifierReference`
                // `QualifiedName`
                // `ThisExpression`
            }
        }
    }

    #[inline]
    fn visit_ts_import_type(&mut self, it: &TSImportType<'a>) {
        if let Some(options) = &it.options {
            self.visit_object_expression(options);
        }
        if let Some(type_arguments) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    #[inline(always)]
    fn visit_ts_import_type_qualifier(&mut self, it: &TSImportTypeQualifier<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_import_type_qualified_name(&mut self, it: &TSImportTypeQualifiedName<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_ts_function_type(&mut self, it: &TSFunctionType<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_constructor_type(&mut self, it: &TSConstructorType<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_mapped_type(&mut self, it: &TSMappedType<'a>) {
        self.add_scope(&it.scope_id);
    }

    #[inline]
    fn visit_ts_template_literal_type(&mut self, it: &TSTemplateLiteralType<'a>) {
        self.visit_ts_types(&it.types);
    }

    #[inline]
    fn visit_ts_as_expression(&mut self, it: &TSAsExpression<'a>) {
        self.visit_expression(&it.expression);
        self.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    fn visit_ts_satisfies_expression(&mut self, it: &TSSatisfiesExpression<'a>) {
        self.visit_expression(&it.expression);
        self.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    fn visit_ts_type_assertion(&mut self, it: &TSTypeAssertion<'a>) {
        self.visit_ts_type(&it.type_annotation);
        self.visit_expression(&it.expression);
    }

    #[inline(always)]
    fn visit_ts_import_equals_declaration(&mut self, it: &TSImportEqualsDeclaration<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_module_reference(&mut self, it: &TSModuleReference<'a>) {
        // Enum does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_ts_external_module_reference(&mut self, it: &TSExternalModuleReference<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_ts_non_null_expression(&mut self, it: &TSNonNullExpression<'a>) {
        self.visit_expression(&it.expression);
    }

    #[inline]
    fn visit_decorator(&mut self, it: &Decorator<'a>) {
        self.visit_expression(&it.expression);
    }

    #[inline]
    fn visit_ts_export_assignment(&mut self, it: &TSExportAssignment<'a>) {
        self.visit_expression(&it.expression);
    }

    #[inline(always)]
    fn visit_ts_namespace_export_declaration(&mut self, it: &TSNamespaceExportDeclaration<'a>) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline]
    fn visit_ts_instantiation_expression(&mut self, it: &TSInstantiationExpression<'a>) {
        self.visit_expression(&it.expression);
        self.visit_ts_type_parameter_instantiation(&it.type_arguments);
    }

    #[inline]
    fn visit_js_doc_nullable_type(&mut self, it: &JSDocNullableType<'a>) {
        self.visit_ts_type(&it.type_annotation);
    }

    #[inline]
    fn visit_js_doc_non_nullable_type(&mut self, it: &JSDocNonNullableType<'a>) {
        self.visit_ts_type(&it.type_annotation);
    }

    #[inline(always)]
    fn visit_js_doc_unknown_type(&mut self, it: &JSDocUnknownType) {
        // Struct does not contain a scope. Halt traversal.
    }

    #[inline(always)]
    fn visit_span(&mut self, it: &Span) {
        // Struct does not contain a scope. Halt traversal.
    }
}
