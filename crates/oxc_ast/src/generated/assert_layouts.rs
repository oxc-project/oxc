// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`

use std::mem::{align_of, offset_of, size_of};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

#[allow(clippy::wildcard_imports)]
use oxc_regular_expression::ast::*;
use oxc_syntax::node::NodeId;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<BooleanLiteral>() == 16usize);
    assert!(align_of::<BooleanLiteral>() == 4usize);
    assert!(offset_of!(BooleanLiteral, node_id) == 0usize);
    assert!(offset_of!(BooleanLiteral, span) == 4usize);
    assert!(offset_of!(BooleanLiteral, value) == 12usize);

    assert!(size_of::<NullLiteral>() == 12usize);
    assert!(align_of::<NullLiteral>() == 4usize);
    assert!(offset_of!(NullLiteral, node_id) == 0usize);
    assert!(offset_of!(NullLiteral, span) == 4usize);

    assert!(size_of::<NumericLiteral>() == 48usize);
    assert!(align_of::<NumericLiteral>() == 8usize);
    assert!(offset_of!(NumericLiteral, node_id) == 0usize);
    assert!(offset_of!(NumericLiteral, span) == 4usize);
    assert!(offset_of!(NumericLiteral, value) == 16usize);
    assert!(offset_of!(NumericLiteral, raw) == 24usize);
    assert!(offset_of!(NumericLiteral, base) == 40usize);

    assert!(size_of::<BigIntLiteral>() == 40usize);
    assert!(align_of::<BigIntLiteral>() == 8usize);
    assert!(offset_of!(BigIntLiteral, node_id) == 0usize);
    assert!(offset_of!(BigIntLiteral, span) == 4usize);
    assert!(offset_of!(BigIntLiteral, raw) == 16usize);
    assert!(offset_of!(BigIntLiteral, base) == 32usize);

    assert!(size_of::<RegExpLiteral>() == 48usize);
    assert!(align_of::<RegExpLiteral>() == 8usize);
    assert!(offset_of!(RegExpLiteral, node_id) == 0usize);
    assert!(offset_of!(RegExpLiteral, span) == 4usize);
    assert!(offset_of!(RegExpLiteral, value) == 12usize);
    assert!(offset_of!(RegExpLiteral, regex) == 16usize);

    assert!(size_of::<RegExp>() == 32usize);
    assert!(align_of::<RegExp>() == 8usize);
    assert!(offset_of!(RegExp, pattern) == 0usize);
    assert!(offset_of!(RegExp, flags) == 24usize);

    assert!(size_of::<RegExpPattern>() == 24usize);
    assert!(align_of::<RegExpPattern>() == 8usize);

    assert!(size_of::<EmptyObject>() == 0usize);
    assert!(align_of::<EmptyObject>() == 1usize);

    assert!(size_of::<StringLiteral>() == 32usize);
    assert!(align_of::<StringLiteral>() == 8usize);
    assert!(offset_of!(StringLiteral, node_id) == 0usize);
    assert!(offset_of!(StringLiteral, span) == 4usize);
    assert!(offset_of!(StringLiteral, value) == 16usize);

    assert!(size_of::<Program>() == 120usize);
    assert!(align_of::<Program>() == 8usize);
    assert!(offset_of!(Program, node_id) == 0usize);
    assert!(offset_of!(Program, span) == 4usize);
    assert!(offset_of!(Program, source_type) == 12usize);
    assert!(offset_of!(Program, hashbang) == 16usize);
    assert!(offset_of!(Program, directives) == 48usize);
    assert!(offset_of!(Program, body) == 80usize);
    assert!(offset_of!(Program, scope_id) == 112usize);

    assert!(size_of::<Expression>() == 16usize);
    assert!(align_of::<Expression>() == 8usize);

    assert!(size_of::<IdentifierName>() == 32usize);
    assert!(align_of::<IdentifierName>() == 8usize);
    assert!(offset_of!(IdentifierName, node_id) == 0usize);
    assert!(offset_of!(IdentifierName, span) == 4usize);
    assert!(offset_of!(IdentifierName, name) == 16usize);

    assert!(size_of::<IdentifierReference>() == 40usize);
    assert!(align_of::<IdentifierReference>() == 8usize);
    assert!(offset_of!(IdentifierReference, node_id) == 0usize);
    assert!(offset_of!(IdentifierReference, span) == 4usize);
    assert!(offset_of!(IdentifierReference, name) == 16usize);
    assert!(offset_of!(IdentifierReference, reference_id) == 32usize);

    assert!(size_of::<BindingIdentifier>() == 40usize);
    assert!(align_of::<BindingIdentifier>() == 8usize);
    assert!(offset_of!(BindingIdentifier, node_id) == 0usize);
    assert!(offset_of!(BindingIdentifier, span) == 4usize);
    assert!(offset_of!(BindingIdentifier, name) == 16usize);
    assert!(offset_of!(BindingIdentifier, symbol_id) == 32usize);

    assert!(size_of::<LabelIdentifier>() == 32usize);
    assert!(align_of::<LabelIdentifier>() == 8usize);
    assert!(offset_of!(LabelIdentifier, node_id) == 0usize);
    assert!(offset_of!(LabelIdentifier, span) == 4usize);
    assert!(offset_of!(LabelIdentifier, name) == 16usize);

    assert!(size_of::<ThisExpression>() == 12usize);
    assert!(align_of::<ThisExpression>() == 4usize);
    assert!(offset_of!(ThisExpression, node_id) == 0usize);
    assert!(offset_of!(ThisExpression, span) == 4usize);

    assert!(size_of::<ArrayExpression>() == 64usize);
    assert!(align_of::<ArrayExpression>() == 8usize);
    assert!(offset_of!(ArrayExpression, node_id) == 0usize);
    assert!(offset_of!(ArrayExpression, span) == 4usize);
    assert!(offset_of!(ArrayExpression, elements) == 16usize);
    assert!(offset_of!(ArrayExpression, trailing_comma) == 48usize);

    assert!(size_of::<ArrayExpressionElement>() == 24usize);
    assert!(align_of::<ArrayExpressionElement>() == 8usize);

    assert!(size_of::<Elision>() == 12usize);
    assert!(align_of::<Elision>() == 4usize);
    assert!(offset_of!(Elision, node_id) == 0usize);
    assert!(offset_of!(Elision, span) == 4usize);

    assert!(size_of::<ObjectExpression>() == 64usize);
    assert!(align_of::<ObjectExpression>() == 8usize);
    assert!(offset_of!(ObjectExpression, node_id) == 0usize);
    assert!(offset_of!(ObjectExpression, span) == 4usize);
    assert!(offset_of!(ObjectExpression, properties) == 16usize);
    assert!(offset_of!(ObjectExpression, trailing_comma) == 48usize);

    assert!(size_of::<ObjectPropertyKind>() == 16usize);
    assert!(align_of::<ObjectPropertyKind>() == 8usize);

    assert!(size_of::<ObjectProperty>() == 72usize);
    assert!(align_of::<ObjectProperty>() == 8usize);
    assert!(offset_of!(ObjectProperty, node_id) == 0usize);
    assert!(offset_of!(ObjectProperty, span) == 4usize);
    assert!(offset_of!(ObjectProperty, kind) == 12usize);
    assert!(offset_of!(ObjectProperty, key) == 16usize);
    assert!(offset_of!(ObjectProperty, value) == 32usize);
    assert!(offset_of!(ObjectProperty, init) == 48usize);
    assert!(offset_of!(ObjectProperty, method) == 64usize);
    assert!(offset_of!(ObjectProperty, shorthand) == 65usize);
    assert!(offset_of!(ObjectProperty, computed) == 66usize);

    assert!(size_of::<PropertyKey>() == 16usize);
    assert!(align_of::<PropertyKey>() == 8usize);

    assert!(size_of::<PropertyKind>() == 1usize);
    assert!(align_of::<PropertyKind>() == 1usize);

    assert!(size_of::<TemplateLiteral>() == 80usize);
    assert!(align_of::<TemplateLiteral>() == 8usize);
    assert!(offset_of!(TemplateLiteral, node_id) == 0usize);
    assert!(offset_of!(TemplateLiteral, span) == 4usize);
    assert!(offset_of!(TemplateLiteral, quasis) == 16usize);
    assert!(offset_of!(TemplateLiteral, expressions) == 48usize);

    assert!(size_of::<TaggedTemplateExpression>() == 120usize);
    assert!(align_of::<TaggedTemplateExpression>() == 8usize);
    assert!(offset_of!(TaggedTemplateExpression, node_id) == 0usize);
    assert!(offset_of!(TaggedTemplateExpression, span) == 4usize);
    assert!(offset_of!(TaggedTemplateExpression, tag) == 16usize);
    assert!(offset_of!(TaggedTemplateExpression, quasi) == 32usize);
    assert!(offset_of!(TaggedTemplateExpression, type_parameters) == 112usize);

    assert!(size_of::<TemplateElement>() == 48usize);
    assert!(align_of::<TemplateElement>() == 8usize);
    assert!(offset_of!(TemplateElement, node_id) == 0usize);
    assert!(offset_of!(TemplateElement, span) == 4usize);
    assert!(offset_of!(TemplateElement, tail) == 12usize);
    assert!(offset_of!(TemplateElement, value) == 16usize);

    assert!(size_of::<TemplateElementValue>() == 32usize);
    assert!(align_of::<TemplateElementValue>() == 8usize);
    assert!(offset_of!(TemplateElementValue, raw) == 0usize);
    assert!(offset_of!(TemplateElementValue, cooked) == 16usize);

    assert!(size_of::<MemberExpression>() == 16usize);
    assert!(align_of::<MemberExpression>() == 8usize);

    assert!(size_of::<ComputedMemberExpression>() == 56usize);
    assert!(align_of::<ComputedMemberExpression>() == 8usize);
    assert!(offset_of!(ComputedMemberExpression, node_id) == 0usize);
    assert!(offset_of!(ComputedMemberExpression, span) == 4usize);
    assert!(offset_of!(ComputedMemberExpression, object) == 16usize);
    assert!(offset_of!(ComputedMemberExpression, expression) == 32usize);
    assert!(offset_of!(ComputedMemberExpression, optional) == 48usize);

    assert!(size_of::<StaticMemberExpression>() == 72usize);
    assert!(align_of::<StaticMemberExpression>() == 8usize);
    assert!(offset_of!(StaticMemberExpression, node_id) == 0usize);
    assert!(offset_of!(StaticMemberExpression, span) == 4usize);
    assert!(offset_of!(StaticMemberExpression, object) == 16usize);
    assert!(offset_of!(StaticMemberExpression, property) == 32usize);
    assert!(offset_of!(StaticMemberExpression, optional) == 64usize);

    assert!(size_of::<PrivateFieldExpression>() == 72usize);
    assert!(align_of::<PrivateFieldExpression>() == 8usize);
    assert!(offset_of!(PrivateFieldExpression, node_id) == 0usize);
    assert!(offset_of!(PrivateFieldExpression, span) == 4usize);
    assert!(offset_of!(PrivateFieldExpression, object) == 16usize);
    assert!(offset_of!(PrivateFieldExpression, field) == 32usize);
    assert!(offset_of!(PrivateFieldExpression, optional) == 64usize);

    assert!(size_of::<CallExpression>() == 80usize);
    assert!(align_of::<CallExpression>() == 8usize);
    assert!(offset_of!(CallExpression, node_id) == 0usize);
    assert!(offset_of!(CallExpression, span) == 4usize);
    assert!(offset_of!(CallExpression, callee) == 16usize);
    assert!(offset_of!(CallExpression, type_parameters) == 32usize);
    assert!(offset_of!(CallExpression, arguments) == 40usize);
    assert!(offset_of!(CallExpression, optional) == 72usize);

    assert!(size_of::<NewExpression>() == 72usize);
    assert!(align_of::<NewExpression>() == 8usize);
    assert!(offset_of!(NewExpression, node_id) == 0usize);
    assert!(offset_of!(NewExpression, span) == 4usize);
    assert!(offset_of!(NewExpression, callee) == 16usize);
    assert!(offset_of!(NewExpression, arguments) == 32usize);
    assert!(offset_of!(NewExpression, type_parameters) == 64usize);

    assert!(size_of::<MetaProperty>() == 80usize);
    assert!(align_of::<MetaProperty>() == 8usize);
    assert!(offset_of!(MetaProperty, node_id) == 0usize);
    assert!(offset_of!(MetaProperty, span) == 4usize);
    assert!(offset_of!(MetaProperty, meta) == 16usize);
    assert!(offset_of!(MetaProperty, property) == 48usize);

    assert!(size_of::<SpreadElement>() == 32usize);
    assert!(align_of::<SpreadElement>() == 8usize);
    assert!(offset_of!(SpreadElement, node_id) == 0usize);
    assert!(offset_of!(SpreadElement, span) == 4usize);
    assert!(offset_of!(SpreadElement, argument) == 16usize);

    assert!(size_of::<Argument>() == 16usize);
    assert!(align_of::<Argument>() == 8usize);

    assert!(size_of::<UpdateExpression>() == 32usize);
    assert!(align_of::<UpdateExpression>() == 8usize);
    assert!(offset_of!(UpdateExpression, node_id) == 0usize);
    assert!(offset_of!(UpdateExpression, span) == 4usize);
    assert!(offset_of!(UpdateExpression, operator) == 12usize);
    assert!(offset_of!(UpdateExpression, prefix) == 13usize);
    assert!(offset_of!(UpdateExpression, argument) == 16usize);

    assert!(size_of::<UnaryExpression>() == 32usize);
    assert!(align_of::<UnaryExpression>() == 8usize);
    assert!(offset_of!(UnaryExpression, node_id) == 0usize);
    assert!(offset_of!(UnaryExpression, span) == 4usize);
    assert!(offset_of!(UnaryExpression, operator) == 12usize);
    assert!(offset_of!(UnaryExpression, argument) == 16usize);

    assert!(size_of::<BinaryExpression>() == 56usize);
    assert!(align_of::<BinaryExpression>() == 8usize);
    assert!(offset_of!(BinaryExpression, node_id) == 0usize);
    assert!(offset_of!(BinaryExpression, span) == 4usize);
    assert!(offset_of!(BinaryExpression, left) == 16usize);
    assert!(offset_of!(BinaryExpression, operator) == 32usize);
    assert!(offset_of!(BinaryExpression, right) == 40usize);

    assert!(size_of::<PrivateInExpression>() == 72usize);
    assert!(align_of::<PrivateInExpression>() == 8usize);
    assert!(offset_of!(PrivateInExpression, node_id) == 0usize);
    assert!(offset_of!(PrivateInExpression, span) == 4usize);
    assert!(offset_of!(PrivateInExpression, left) == 16usize);
    assert!(offset_of!(PrivateInExpression, operator) == 48usize);
    assert!(offset_of!(PrivateInExpression, right) == 56usize);

    assert!(size_of::<LogicalExpression>() == 56usize);
    assert!(align_of::<LogicalExpression>() == 8usize);
    assert!(offset_of!(LogicalExpression, node_id) == 0usize);
    assert!(offset_of!(LogicalExpression, span) == 4usize);
    assert!(offset_of!(LogicalExpression, left) == 16usize);
    assert!(offset_of!(LogicalExpression, operator) == 32usize);
    assert!(offset_of!(LogicalExpression, right) == 40usize);

    assert!(size_of::<ConditionalExpression>() == 64usize);
    assert!(align_of::<ConditionalExpression>() == 8usize);
    assert!(offset_of!(ConditionalExpression, node_id) == 0usize);
    assert!(offset_of!(ConditionalExpression, span) == 4usize);
    assert!(offset_of!(ConditionalExpression, test) == 16usize);
    assert!(offset_of!(ConditionalExpression, consequent) == 32usize);
    assert!(offset_of!(ConditionalExpression, alternate) == 48usize);

    assert!(size_of::<AssignmentExpression>() == 48usize);
    assert!(align_of::<AssignmentExpression>() == 8usize);
    assert!(offset_of!(AssignmentExpression, node_id) == 0usize);
    assert!(offset_of!(AssignmentExpression, span) == 4usize);
    assert!(offset_of!(AssignmentExpression, operator) == 12usize);
    assert!(offset_of!(AssignmentExpression, left) == 16usize);
    assert!(offset_of!(AssignmentExpression, right) == 32usize);

    assert!(size_of::<AssignmentTarget>() == 16usize);
    assert!(align_of::<AssignmentTarget>() == 8usize);

    assert!(size_of::<SimpleAssignmentTarget>() == 16usize);
    assert!(align_of::<SimpleAssignmentTarget>() == 8usize);

    assert!(size_of::<AssignmentTargetPattern>() == 16usize);
    assert!(align_of::<AssignmentTargetPattern>() == 8usize);

    assert!(size_of::<ArrayAssignmentTarget>() == 96usize);
    assert!(align_of::<ArrayAssignmentTarget>() == 8usize);
    assert!(offset_of!(ArrayAssignmentTarget, node_id) == 0usize);
    assert!(offset_of!(ArrayAssignmentTarget, span) == 4usize);
    assert!(offset_of!(ArrayAssignmentTarget, elements) == 16usize);
    assert!(offset_of!(ArrayAssignmentTarget, rest) == 48usize);
    assert!(offset_of!(ArrayAssignmentTarget, trailing_comma) == 80usize);

    assert!(size_of::<ObjectAssignmentTarget>() == 80usize);
    assert!(align_of::<ObjectAssignmentTarget>() == 8usize);
    assert!(offset_of!(ObjectAssignmentTarget, node_id) == 0usize);
    assert!(offset_of!(ObjectAssignmentTarget, span) == 4usize);
    assert!(offset_of!(ObjectAssignmentTarget, properties) == 16usize);
    assert!(offset_of!(ObjectAssignmentTarget, rest) == 48usize);

    assert!(size_of::<AssignmentTargetRest>() == 32usize);
    assert!(align_of::<AssignmentTargetRest>() == 8usize);
    assert!(offset_of!(AssignmentTargetRest, node_id) == 0usize);
    assert!(offset_of!(AssignmentTargetRest, span) == 4usize);
    assert!(offset_of!(AssignmentTargetRest, target) == 16usize);

    assert!(size_of::<AssignmentTargetMaybeDefault>() == 16usize);
    assert!(align_of::<AssignmentTargetMaybeDefault>() == 8usize);

    assert!(size_of::<AssignmentTargetWithDefault>() == 48usize);
    assert!(align_of::<AssignmentTargetWithDefault>() == 8usize);
    assert!(offset_of!(AssignmentTargetWithDefault, node_id) == 0usize);
    assert!(offset_of!(AssignmentTargetWithDefault, span) == 4usize);
    assert!(offset_of!(AssignmentTargetWithDefault, binding) == 16usize);
    assert!(offset_of!(AssignmentTargetWithDefault, init) == 32usize);

    assert!(size_of::<AssignmentTargetProperty>() == 16usize);
    assert!(align_of::<AssignmentTargetProperty>() == 8usize);

    assert!(size_of::<AssignmentTargetPropertyIdentifier>() == 72usize);
    assert!(align_of::<AssignmentTargetPropertyIdentifier>() == 8usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, node_id) == 0usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, span) == 4usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, binding) == 16usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, init) == 56usize);

    assert!(size_of::<AssignmentTargetPropertyProperty>() == 48usize);
    assert!(align_of::<AssignmentTargetPropertyProperty>() == 8usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, node_id) == 0usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, span) == 4usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, name) == 16usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, binding) == 32usize);

    assert!(size_of::<SequenceExpression>() == 48usize);
    assert!(align_of::<SequenceExpression>() == 8usize);
    assert!(offset_of!(SequenceExpression, node_id) == 0usize);
    assert!(offset_of!(SequenceExpression, span) == 4usize);
    assert!(offset_of!(SequenceExpression, expressions) == 16usize);

    assert!(size_of::<Super>() == 12usize);
    assert!(align_of::<Super>() == 4usize);
    assert!(offset_of!(Super, node_id) == 0usize);
    assert!(offset_of!(Super, span) == 4usize);

    assert!(size_of::<AwaitExpression>() == 32usize);
    assert!(align_of::<AwaitExpression>() == 8usize);
    assert!(offset_of!(AwaitExpression, node_id) == 0usize);
    assert!(offset_of!(AwaitExpression, span) == 4usize);
    assert!(offset_of!(AwaitExpression, argument) == 16usize);

    assert!(size_of::<ChainExpression>() == 32usize);
    assert!(align_of::<ChainExpression>() == 8usize);
    assert!(offset_of!(ChainExpression, node_id) == 0usize);
    assert!(offset_of!(ChainExpression, span) == 4usize);
    assert!(offset_of!(ChainExpression, expression) == 16usize);

    assert!(size_of::<ChainElement>() == 16usize);
    assert!(align_of::<ChainElement>() == 8usize);

    assert!(size_of::<ParenthesizedExpression>() == 32usize);
    assert!(align_of::<ParenthesizedExpression>() == 8usize);
    assert!(offset_of!(ParenthesizedExpression, node_id) == 0usize);
    assert!(offset_of!(ParenthesizedExpression, span) == 4usize);
    assert!(offset_of!(ParenthesizedExpression, expression) == 16usize);

    assert!(size_of::<Statement>() == 16usize);
    assert!(align_of::<Statement>() == 8usize);

    assert!(size_of::<Directive>() == 64usize);
    assert!(align_of::<Directive>() == 8usize);
    assert!(offset_of!(Directive, node_id) == 0usize);
    assert!(offset_of!(Directive, span) == 4usize);
    assert!(offset_of!(Directive, expression) == 16usize);
    assert!(offset_of!(Directive, directive) == 48usize);

    assert!(size_of::<Hashbang>() == 32usize);
    assert!(align_of::<Hashbang>() == 8usize);
    assert!(offset_of!(Hashbang, node_id) == 0usize);
    assert!(offset_of!(Hashbang, span) == 4usize);
    assert!(offset_of!(Hashbang, value) == 16usize);

    assert!(size_of::<BlockStatement>() == 56usize);
    assert!(align_of::<BlockStatement>() == 8usize);
    assert!(offset_of!(BlockStatement, node_id) == 0usize);
    assert!(offset_of!(BlockStatement, span) == 4usize);
    assert!(offset_of!(BlockStatement, body) == 16usize);
    assert!(offset_of!(BlockStatement, scope_id) == 48usize);

    assert!(size_of::<Declaration>() == 16usize);
    assert!(align_of::<Declaration>() == 8usize);

    assert!(size_of::<VariableDeclaration>() == 56usize);
    assert!(align_of::<VariableDeclaration>() == 8usize);
    assert!(offset_of!(VariableDeclaration, node_id) == 0usize);
    assert!(offset_of!(VariableDeclaration, span) == 4usize);
    assert!(offset_of!(VariableDeclaration, kind) == 12usize);
    assert!(offset_of!(VariableDeclaration, declarations) == 16usize);
    assert!(offset_of!(VariableDeclaration, declare) == 48usize);

    assert!(size_of::<VariableDeclarationKind>() == 1usize);
    assert!(align_of::<VariableDeclarationKind>() == 1usize);

    assert!(size_of::<VariableDeclarator>() == 80usize);
    assert!(align_of::<VariableDeclarator>() == 8usize);
    assert!(offset_of!(VariableDeclarator, node_id) == 0usize);
    assert!(offset_of!(VariableDeclarator, span) == 4usize);
    assert!(offset_of!(VariableDeclarator, kind) == 12usize);
    assert!(offset_of!(VariableDeclarator, id) == 16usize);
    assert!(offset_of!(VariableDeclarator, init) == 56usize);
    assert!(offset_of!(VariableDeclarator, definite) == 72usize);

    assert!(size_of::<EmptyStatement>() == 12usize);
    assert!(align_of::<EmptyStatement>() == 4usize);
    assert!(offset_of!(EmptyStatement, node_id) == 0usize);
    assert!(offset_of!(EmptyStatement, span) == 4usize);

    assert!(size_of::<ExpressionStatement>() == 32usize);
    assert!(align_of::<ExpressionStatement>() == 8usize);
    assert!(offset_of!(ExpressionStatement, node_id) == 0usize);
    assert!(offset_of!(ExpressionStatement, span) == 4usize);
    assert!(offset_of!(ExpressionStatement, expression) == 16usize);

    assert!(size_of::<IfStatement>() == 64usize);
    assert!(align_of::<IfStatement>() == 8usize);
    assert!(offset_of!(IfStatement, node_id) == 0usize);
    assert!(offset_of!(IfStatement, span) == 4usize);
    assert!(offset_of!(IfStatement, test) == 16usize);
    assert!(offset_of!(IfStatement, consequent) == 32usize);
    assert!(offset_of!(IfStatement, alternate) == 48usize);

    assert!(size_of::<DoWhileStatement>() == 48usize);
    assert!(align_of::<DoWhileStatement>() == 8usize);
    assert!(offset_of!(DoWhileStatement, node_id) == 0usize);
    assert!(offset_of!(DoWhileStatement, span) == 4usize);
    assert!(offset_of!(DoWhileStatement, body) == 16usize);
    assert!(offset_of!(DoWhileStatement, test) == 32usize);

    assert!(size_of::<WhileStatement>() == 48usize);
    assert!(align_of::<WhileStatement>() == 8usize);
    assert!(offset_of!(WhileStatement, node_id) == 0usize);
    assert!(offset_of!(WhileStatement, span) == 4usize);
    assert!(offset_of!(WhileStatement, test) == 16usize);
    assert!(offset_of!(WhileStatement, body) == 32usize);

    assert!(size_of::<ForStatement>() == 88usize);
    assert!(align_of::<ForStatement>() == 8usize);
    assert!(offset_of!(ForStatement, node_id) == 0usize);
    assert!(offset_of!(ForStatement, span) == 4usize);
    assert!(offset_of!(ForStatement, init) == 16usize);
    assert!(offset_of!(ForStatement, test) == 32usize);
    assert!(offset_of!(ForStatement, update) == 48usize);
    assert!(offset_of!(ForStatement, body) == 64usize);
    assert!(offset_of!(ForStatement, scope_id) == 80usize);

    assert!(size_of::<ForStatementInit>() == 16usize);
    assert!(align_of::<ForStatementInit>() == 8usize);

    assert!(size_of::<ForInStatement>() == 72usize);
    assert!(align_of::<ForInStatement>() == 8usize);
    assert!(offset_of!(ForInStatement, node_id) == 0usize);
    assert!(offset_of!(ForInStatement, span) == 4usize);
    assert!(offset_of!(ForInStatement, left) == 16usize);
    assert!(offset_of!(ForInStatement, right) == 32usize);
    assert!(offset_of!(ForInStatement, body) == 48usize);
    assert!(offset_of!(ForInStatement, scope_id) == 64usize);

    assert!(size_of::<ForStatementLeft>() == 16usize);
    assert!(align_of::<ForStatementLeft>() == 8usize);

    assert!(size_of::<ForOfStatement>() == 72usize);
    assert!(align_of::<ForOfStatement>() == 8usize);
    assert!(offset_of!(ForOfStatement, node_id) == 0usize);
    assert!(offset_of!(ForOfStatement, span) == 4usize);
    assert!(offset_of!(ForOfStatement, r#await) == 12usize);
    assert!(offset_of!(ForOfStatement, left) == 16usize);
    assert!(offset_of!(ForOfStatement, right) == 32usize);
    assert!(offset_of!(ForOfStatement, body) == 48usize);
    assert!(offset_of!(ForOfStatement, scope_id) == 64usize);

    assert!(size_of::<ContinueStatement>() == 48usize);
    assert!(align_of::<ContinueStatement>() == 8usize);
    assert!(offset_of!(ContinueStatement, node_id) == 0usize);
    assert!(offset_of!(ContinueStatement, span) == 4usize);
    assert!(offset_of!(ContinueStatement, label) == 16usize);

    assert!(size_of::<BreakStatement>() == 48usize);
    assert!(align_of::<BreakStatement>() == 8usize);
    assert!(offset_of!(BreakStatement, node_id) == 0usize);
    assert!(offset_of!(BreakStatement, span) == 4usize);
    assert!(offset_of!(BreakStatement, label) == 16usize);

    assert!(size_of::<ReturnStatement>() == 32usize);
    assert!(align_of::<ReturnStatement>() == 8usize);
    assert!(offset_of!(ReturnStatement, node_id) == 0usize);
    assert!(offset_of!(ReturnStatement, span) == 4usize);
    assert!(offset_of!(ReturnStatement, argument) == 16usize);

    assert!(size_of::<WithStatement>() == 48usize);
    assert!(align_of::<WithStatement>() == 8usize);
    assert!(offset_of!(WithStatement, node_id) == 0usize);
    assert!(offset_of!(WithStatement, span) == 4usize);
    assert!(offset_of!(WithStatement, object) == 16usize);
    assert!(offset_of!(WithStatement, body) == 32usize);

    assert!(size_of::<SwitchStatement>() == 72usize);
    assert!(align_of::<SwitchStatement>() == 8usize);
    assert!(offset_of!(SwitchStatement, node_id) == 0usize);
    assert!(offset_of!(SwitchStatement, span) == 4usize);
    assert!(offset_of!(SwitchStatement, discriminant) == 16usize);
    assert!(offset_of!(SwitchStatement, cases) == 32usize);
    assert!(offset_of!(SwitchStatement, scope_id) == 64usize);

    assert!(size_of::<SwitchCase>() == 64usize);
    assert!(align_of::<SwitchCase>() == 8usize);
    assert!(offset_of!(SwitchCase, node_id) == 0usize);
    assert!(offset_of!(SwitchCase, span) == 4usize);
    assert!(offset_of!(SwitchCase, test) == 16usize);
    assert!(offset_of!(SwitchCase, consequent) == 32usize);

    assert!(size_of::<LabeledStatement>() == 64usize);
    assert!(align_of::<LabeledStatement>() == 8usize);
    assert!(offset_of!(LabeledStatement, node_id) == 0usize);
    assert!(offset_of!(LabeledStatement, span) == 4usize);
    assert!(offset_of!(LabeledStatement, label) == 16usize);
    assert!(offset_of!(LabeledStatement, body) == 48usize);

    assert!(size_of::<ThrowStatement>() == 32usize);
    assert!(align_of::<ThrowStatement>() == 8usize);
    assert!(offset_of!(ThrowStatement, node_id) == 0usize);
    assert!(offset_of!(ThrowStatement, span) == 4usize);
    assert!(offset_of!(ThrowStatement, argument) == 16usize);

    assert!(size_of::<TryStatement>() == 40usize);
    assert!(align_of::<TryStatement>() == 8usize);
    assert!(offset_of!(TryStatement, node_id) == 0usize);
    assert!(offset_of!(TryStatement, span) == 4usize);
    assert!(offset_of!(TryStatement, block) == 16usize);
    assert!(offset_of!(TryStatement, handler) == 24usize);
    assert!(offset_of!(TryStatement, finalizer) == 32usize);

    assert!(size_of::<CatchClause>() == 88usize);
    assert!(align_of::<CatchClause>() == 8usize);
    assert!(offset_of!(CatchClause, node_id) == 0usize);
    assert!(offset_of!(CatchClause, span) == 4usize);
    assert!(offset_of!(CatchClause, param) == 16usize);
    assert!(offset_of!(CatchClause, body) == 72usize);
    assert!(offset_of!(CatchClause, scope_id) == 80usize);

    assert!(size_of::<CatchParameter>() == 56usize);
    assert!(align_of::<CatchParameter>() == 8usize);
    assert!(offset_of!(CatchParameter, node_id) == 0usize);
    assert!(offset_of!(CatchParameter, span) == 4usize);
    assert!(offset_of!(CatchParameter, pattern) == 16usize);

    assert!(size_of::<DebuggerStatement>() == 12usize);
    assert!(align_of::<DebuggerStatement>() == 4usize);
    assert!(offset_of!(DebuggerStatement, node_id) == 0usize);
    assert!(offset_of!(DebuggerStatement, span) == 4usize);

    assert!(size_of::<BindingPattern>() == 40usize);
    assert!(align_of::<BindingPattern>() == 8usize);
    assert!(offset_of!(BindingPattern, node_id) == 0usize);
    assert!(offset_of!(BindingPattern, kind) == 8usize);
    assert!(offset_of!(BindingPattern, type_annotation) == 24usize);
    assert!(offset_of!(BindingPattern, optional) == 32usize);

    assert!(size_of::<BindingPatternKind>() == 16usize);
    assert!(align_of::<BindingPatternKind>() == 8usize);

    assert!(size_of::<AssignmentPattern>() == 72usize);
    assert!(align_of::<AssignmentPattern>() == 8usize);
    assert!(offset_of!(AssignmentPattern, node_id) == 0usize);
    assert!(offset_of!(AssignmentPattern, span) == 4usize);
    assert!(offset_of!(AssignmentPattern, left) == 16usize);
    assert!(offset_of!(AssignmentPattern, right) == 56usize);

    assert!(size_of::<ObjectPattern>() == 56usize);
    assert!(align_of::<ObjectPattern>() == 8usize);
    assert!(offset_of!(ObjectPattern, node_id) == 0usize);
    assert!(offset_of!(ObjectPattern, span) == 4usize);
    assert!(offset_of!(ObjectPattern, properties) == 16usize);
    assert!(offset_of!(ObjectPattern, rest) == 48usize);

    assert!(size_of::<BindingProperty>() == 80usize);
    assert!(align_of::<BindingProperty>() == 8usize);
    assert!(offset_of!(BindingProperty, node_id) == 0usize);
    assert!(offset_of!(BindingProperty, span) == 4usize);
    assert!(offset_of!(BindingProperty, key) == 16usize);
    assert!(offset_of!(BindingProperty, value) == 32usize);
    assert!(offset_of!(BindingProperty, shorthand) == 72usize);
    assert!(offset_of!(BindingProperty, computed) == 73usize);

    assert!(size_of::<ArrayPattern>() == 56usize);
    assert!(align_of::<ArrayPattern>() == 8usize);
    assert!(offset_of!(ArrayPattern, node_id) == 0usize);
    assert!(offset_of!(ArrayPattern, span) == 4usize);
    assert!(offset_of!(ArrayPattern, elements) == 16usize);
    assert!(offset_of!(ArrayPattern, rest) == 48usize);

    assert!(size_of::<BindingRestElement>() == 56usize);
    assert!(align_of::<BindingRestElement>() == 8usize);
    assert!(offset_of!(BindingRestElement, node_id) == 0usize);
    assert!(offset_of!(BindingRestElement, span) == 4usize);
    assert!(offset_of!(BindingRestElement, argument) == 16usize);

    assert!(size_of::<Function>() == 112usize);
    assert!(align_of::<Function>() == 8usize);
    assert!(offset_of!(Function, node_id) == 0usize);
    assert!(offset_of!(Function, r#type) == 4usize);
    assert!(offset_of!(Function, span) == 8usize);
    assert!(offset_of!(Function, id) == 16usize);
    assert!(offset_of!(Function, generator) == 56usize);
    assert!(offset_of!(Function, r#async) == 57usize);
    assert!(offset_of!(Function, declare) == 58usize);
    assert!(offset_of!(Function, type_parameters) == 64usize);
    assert!(offset_of!(Function, this_param) == 72usize);
    assert!(offset_of!(Function, params) == 80usize);
    assert!(offset_of!(Function, return_type) == 88usize);
    assert!(offset_of!(Function, body) == 96usize);
    assert!(offset_of!(Function, scope_id) == 104usize);

    assert!(size_of::<FunctionType>() == 1usize);
    assert!(align_of::<FunctionType>() == 1usize);

    assert!(size_of::<FormalParameters>() == 56usize);
    assert!(align_of::<FormalParameters>() == 8usize);
    assert!(offset_of!(FormalParameters, node_id) == 0usize);
    assert!(offset_of!(FormalParameters, span) == 4usize);
    assert!(offset_of!(FormalParameters, kind) == 12usize);
    assert!(offset_of!(FormalParameters, items) == 16usize);
    assert!(offset_of!(FormalParameters, rest) == 48usize);

    assert!(size_of::<FormalParameter>() == 96usize);
    assert!(align_of::<FormalParameter>() == 8usize);
    assert!(offset_of!(FormalParameter, node_id) == 0usize);
    assert!(offset_of!(FormalParameter, span) == 4usize);
    assert!(offset_of!(FormalParameter, decorators) == 16usize);
    assert!(offset_of!(FormalParameter, pattern) == 48usize);
    assert!(offset_of!(FormalParameter, accessibility) == 88usize);
    assert!(offset_of!(FormalParameter, readonly) == 89usize);
    assert!(offset_of!(FormalParameter, r#override) == 90usize);

    assert!(size_of::<FormalParameterKind>() == 1usize);
    assert!(align_of::<FormalParameterKind>() == 1usize);

    assert!(size_of::<FunctionBody>() == 80usize);
    assert!(align_of::<FunctionBody>() == 8usize);
    assert!(offset_of!(FunctionBody, node_id) == 0usize);
    assert!(offset_of!(FunctionBody, span) == 4usize);
    assert!(offset_of!(FunctionBody, directives) == 16usize);
    assert!(offset_of!(FunctionBody, statements) == 48usize);

    assert!(size_of::<ArrowFunctionExpression>() == 56usize);
    assert!(align_of::<ArrowFunctionExpression>() == 8usize);
    assert!(offset_of!(ArrowFunctionExpression, node_id) == 0usize);
    assert!(offset_of!(ArrowFunctionExpression, span) == 4usize);
    assert!(offset_of!(ArrowFunctionExpression, expression) == 12usize);
    assert!(offset_of!(ArrowFunctionExpression, r#async) == 13usize);
    assert!(offset_of!(ArrowFunctionExpression, type_parameters) == 16usize);
    assert!(offset_of!(ArrowFunctionExpression, params) == 24usize);
    assert!(offset_of!(ArrowFunctionExpression, return_type) == 32usize);
    assert!(offset_of!(ArrowFunctionExpression, body) == 40usize);
    assert!(offset_of!(ArrowFunctionExpression, scope_id) == 48usize);

    assert!(size_of::<YieldExpression>() == 32usize);
    assert!(align_of::<YieldExpression>() == 8usize);
    assert!(offset_of!(YieldExpression, node_id) == 0usize);
    assert!(offset_of!(YieldExpression, span) == 4usize);
    assert!(offset_of!(YieldExpression, delegate) == 12usize);
    assert!(offset_of!(YieldExpression, argument) == 16usize);

    assert!(size_of::<Class>() == 168usize);
    assert!(align_of::<Class>() == 8usize);
    assert!(offset_of!(Class, node_id) == 0usize);
    assert!(offset_of!(Class, r#type) == 4usize);
    assert!(offset_of!(Class, span) == 8usize);
    assert!(offset_of!(Class, decorators) == 16usize);
    assert!(offset_of!(Class, id) == 48usize);
    assert!(offset_of!(Class, type_parameters) == 88usize);
    assert!(offset_of!(Class, super_class) == 96usize);
    assert!(offset_of!(Class, super_type_parameters) == 112usize);
    assert!(offset_of!(Class, implements) == 120usize);
    assert!(offset_of!(Class, body) == 152usize);
    assert!(offset_of!(Class, r#abstract) == 160usize);
    assert!(offset_of!(Class, declare) == 161usize);
    assert!(offset_of!(Class, scope_id) == 164usize);

    assert!(size_of::<ClassType>() == 1usize);
    assert!(align_of::<ClassType>() == 1usize);

    assert!(size_of::<ClassBody>() == 48usize);
    assert!(align_of::<ClassBody>() == 8usize);
    assert!(offset_of!(ClassBody, node_id) == 0usize);
    assert!(offset_of!(ClassBody, span) == 4usize);
    assert!(offset_of!(ClassBody, body) == 16usize);

    assert!(size_of::<ClassElement>() == 16usize);
    assert!(align_of::<ClassElement>() == 8usize);

    assert!(size_of::<MethodDefinition>() == 80usize);
    assert!(align_of::<MethodDefinition>() == 8usize);
    assert!(offset_of!(MethodDefinition, node_id) == 0usize);
    assert!(offset_of!(MethodDefinition, r#type) == 4usize);
    assert!(offset_of!(MethodDefinition, span) == 8usize);
    assert!(offset_of!(MethodDefinition, decorators) == 16usize);
    assert!(offset_of!(MethodDefinition, key) == 48usize);
    assert!(offset_of!(MethodDefinition, value) == 64usize);
    assert!(offset_of!(MethodDefinition, kind) == 72usize);
    assert!(offset_of!(MethodDefinition, computed) == 73usize);
    assert!(offset_of!(MethodDefinition, r#static) == 74usize);
    assert!(offset_of!(MethodDefinition, r#override) == 75usize);
    assert!(offset_of!(MethodDefinition, optional) == 76usize);
    assert!(offset_of!(MethodDefinition, accessibility) == 77usize);

    assert!(size_of::<MethodDefinitionType>() == 1usize);
    assert!(align_of::<MethodDefinitionType>() == 1usize);

    assert!(size_of::<PropertyDefinition>() == 104usize);
    assert!(align_of::<PropertyDefinition>() == 8usize);
    assert!(offset_of!(PropertyDefinition, node_id) == 0usize);
    assert!(offset_of!(PropertyDefinition, r#type) == 4usize);
    assert!(offset_of!(PropertyDefinition, span) == 8usize);
    assert!(offset_of!(PropertyDefinition, decorators) == 16usize);
    assert!(offset_of!(PropertyDefinition, key) == 48usize);
    assert!(offset_of!(PropertyDefinition, value) == 64usize);
    assert!(offset_of!(PropertyDefinition, computed) == 80usize);
    assert!(offset_of!(PropertyDefinition, r#static) == 81usize);
    assert!(offset_of!(PropertyDefinition, declare) == 82usize);
    assert!(offset_of!(PropertyDefinition, r#override) == 83usize);
    assert!(offset_of!(PropertyDefinition, optional) == 84usize);
    assert!(offset_of!(PropertyDefinition, definite) == 85usize);
    assert!(offset_of!(PropertyDefinition, readonly) == 86usize);
    assert!(offset_of!(PropertyDefinition, type_annotation) == 88usize);
    assert!(offset_of!(PropertyDefinition, accessibility) == 96usize);

    assert!(size_of::<PropertyDefinitionType>() == 1usize);
    assert!(align_of::<PropertyDefinitionType>() == 1usize);

    assert!(size_of::<MethodDefinitionKind>() == 1usize);
    assert!(align_of::<MethodDefinitionKind>() == 1usize);

    assert!(size_of::<PrivateIdentifier>() == 32usize);
    assert!(align_of::<PrivateIdentifier>() == 8usize);
    assert!(offset_of!(PrivateIdentifier, node_id) == 0usize);
    assert!(offset_of!(PrivateIdentifier, span) == 4usize);
    assert!(offset_of!(PrivateIdentifier, name) == 16usize);

    assert!(size_of::<StaticBlock>() == 56usize);
    assert!(align_of::<StaticBlock>() == 8usize);
    assert!(offset_of!(StaticBlock, node_id) == 0usize);
    assert!(offset_of!(StaticBlock, span) == 4usize);
    assert!(offset_of!(StaticBlock, body) == 16usize);
    assert!(offset_of!(StaticBlock, scope_id) == 48usize);

    assert!(size_of::<ModuleDeclaration>() == 16usize);
    assert!(align_of::<ModuleDeclaration>() == 8usize);

    assert!(size_of::<AccessorPropertyType>() == 1usize);
    assert!(align_of::<AccessorPropertyType>() == 1usize);

    assert!(size_of::<AccessorProperty>() == 104usize);
    assert!(align_of::<AccessorProperty>() == 8usize);
    assert!(offset_of!(AccessorProperty, node_id) == 0usize);
    assert!(offset_of!(AccessorProperty, r#type) == 4usize);
    assert!(offset_of!(AccessorProperty, span) == 8usize);
    assert!(offset_of!(AccessorProperty, decorators) == 16usize);
    assert!(offset_of!(AccessorProperty, key) == 48usize);
    assert!(offset_of!(AccessorProperty, value) == 64usize);
    assert!(offset_of!(AccessorProperty, computed) == 80usize);
    assert!(offset_of!(AccessorProperty, r#static) == 81usize);
    assert!(offset_of!(AccessorProperty, definite) == 82usize);
    assert!(offset_of!(AccessorProperty, type_annotation) == 88usize);
    assert!(offset_of!(AccessorProperty, accessibility) == 96usize);

    assert!(size_of::<ImportExpression>() == 64usize);
    assert!(align_of::<ImportExpression>() == 8usize);
    assert!(offset_of!(ImportExpression, node_id) == 0usize);
    assert!(offset_of!(ImportExpression, span) == 4usize);
    assert!(offset_of!(ImportExpression, source) == 16usize);
    assert!(offset_of!(ImportExpression, arguments) == 32usize);

    assert!(size_of::<ImportDeclaration>() == 96usize);
    assert!(align_of::<ImportDeclaration>() == 8usize);
    assert!(offset_of!(ImportDeclaration, node_id) == 0usize);
    assert!(offset_of!(ImportDeclaration, span) == 4usize);
    assert!(offset_of!(ImportDeclaration, specifiers) == 16usize);
    assert!(offset_of!(ImportDeclaration, source) == 48usize);
    assert!(offset_of!(ImportDeclaration, with_clause) == 80usize);
    assert!(offset_of!(ImportDeclaration, import_kind) == 88usize);

    assert!(size_of::<ImportDeclarationSpecifier>() == 16usize);
    assert!(align_of::<ImportDeclarationSpecifier>() == 8usize);

    assert!(size_of::<ImportSpecifier>() == 112usize);
    assert!(align_of::<ImportSpecifier>() == 8usize);
    assert!(offset_of!(ImportSpecifier, node_id) == 0usize);
    assert!(offset_of!(ImportSpecifier, span) == 4usize);
    assert!(offset_of!(ImportSpecifier, imported) == 16usize);
    assert!(offset_of!(ImportSpecifier, local) == 64usize);
    assert!(offset_of!(ImportSpecifier, import_kind) == 104usize);

    assert!(size_of::<ImportDefaultSpecifier>() == 56usize);
    assert!(align_of::<ImportDefaultSpecifier>() == 8usize);
    assert!(offset_of!(ImportDefaultSpecifier, node_id) == 0usize);
    assert!(offset_of!(ImportDefaultSpecifier, span) == 4usize);
    assert!(offset_of!(ImportDefaultSpecifier, local) == 16usize);

    assert!(size_of::<ImportNamespaceSpecifier>() == 56usize);
    assert!(align_of::<ImportNamespaceSpecifier>() == 8usize);
    assert!(offset_of!(ImportNamespaceSpecifier, node_id) == 0usize);
    assert!(offset_of!(ImportNamespaceSpecifier, span) == 4usize);
    assert!(offset_of!(ImportNamespaceSpecifier, local) == 16usize);

    assert!(size_of::<WithClause>() == 80usize);
    assert!(align_of::<WithClause>() == 8usize);
    assert!(offset_of!(WithClause, node_id) == 0usize);
    assert!(offset_of!(WithClause, span) == 4usize);
    assert!(offset_of!(WithClause, attributes_keyword) == 16usize);
    assert!(offset_of!(WithClause, with_entries) == 48usize);

    assert!(size_of::<ImportAttribute>() == 88usize);
    assert!(align_of::<ImportAttribute>() == 8usize);
    assert!(offset_of!(ImportAttribute, node_id) == 0usize);
    assert!(offset_of!(ImportAttribute, span) == 4usize);
    assert!(offset_of!(ImportAttribute, key) == 16usize);
    assert!(offset_of!(ImportAttribute, value) == 56usize);

    assert!(size_of::<ImportAttributeKey>() == 40usize);
    assert!(align_of::<ImportAttributeKey>() == 8usize);

    assert!(size_of::<ExportNamedDeclaration>() == 112usize);
    assert!(align_of::<ExportNamedDeclaration>() == 8usize);
    assert!(offset_of!(ExportNamedDeclaration, node_id) == 0usize);
    assert!(offset_of!(ExportNamedDeclaration, span) == 4usize);
    assert!(offset_of!(ExportNamedDeclaration, declaration) == 16usize);
    assert!(offset_of!(ExportNamedDeclaration, specifiers) == 32usize);
    assert!(offset_of!(ExportNamedDeclaration, source) == 64usize);
    assert!(offset_of!(ExportNamedDeclaration, export_kind) == 96usize);
    assert!(offset_of!(ExportNamedDeclaration, with_clause) == 104usize);

    assert!(size_of::<ExportDefaultDeclaration>() == 80usize);
    assert!(align_of::<ExportDefaultDeclaration>() == 8usize);
    assert!(offset_of!(ExportDefaultDeclaration, node_id) == 0usize);
    assert!(offset_of!(ExportDefaultDeclaration, span) == 4usize);
    assert!(offset_of!(ExportDefaultDeclaration, declaration) == 16usize);
    assert!(offset_of!(ExportDefaultDeclaration, exported) == 32usize);

    assert!(size_of::<ExportAllDeclaration>() == 112usize);
    assert!(align_of::<ExportAllDeclaration>() == 8usize);
    assert!(offset_of!(ExportAllDeclaration, node_id) == 0usize);
    assert!(offset_of!(ExportAllDeclaration, span) == 4usize);
    assert!(offset_of!(ExportAllDeclaration, exported) == 16usize);
    assert!(offset_of!(ExportAllDeclaration, source) == 64usize);
    assert!(offset_of!(ExportAllDeclaration, with_clause) == 96usize);
    assert!(offset_of!(ExportAllDeclaration, export_kind) == 104usize);

    assert!(size_of::<ExportSpecifier>() == 120usize);
    assert!(align_of::<ExportSpecifier>() == 8usize);
    assert!(offset_of!(ExportSpecifier, node_id) == 0usize);
    assert!(offset_of!(ExportSpecifier, span) == 4usize);
    assert!(offset_of!(ExportSpecifier, local) == 16usize);
    assert!(offset_of!(ExportSpecifier, exported) == 64usize);
    assert!(offset_of!(ExportSpecifier, export_kind) == 112usize);

    assert!(size_of::<ExportDefaultDeclarationKind>() == 16usize);
    assert!(align_of::<ExportDefaultDeclarationKind>() == 8usize);

    assert!(size_of::<ModuleExportName>() == 48usize);
    assert!(align_of::<ModuleExportName>() == 8usize);

    assert!(size_of::<TSThisParameter>() == 32usize);
    assert!(align_of::<TSThisParameter>() == 8usize);
    assert!(offset_of!(TSThisParameter, node_id) == 0usize);
    assert!(offset_of!(TSThisParameter, span) == 4usize);
    assert!(offset_of!(TSThisParameter, this_span) == 12usize);
    assert!(offset_of!(TSThisParameter, type_annotation) == 24usize);

    assert!(size_of::<TSEnumDeclaration>() == 96usize);
    assert!(align_of::<TSEnumDeclaration>() == 8usize);
    assert!(offset_of!(TSEnumDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSEnumDeclaration, span) == 4usize);
    assert!(offset_of!(TSEnumDeclaration, id) == 16usize);
    assert!(offset_of!(TSEnumDeclaration, members) == 56usize);
    assert!(offset_of!(TSEnumDeclaration, r#const) == 88usize);
    assert!(offset_of!(TSEnumDeclaration, declare) == 89usize);
    assert!(offset_of!(TSEnumDeclaration, scope_id) == 92usize);

    assert!(size_of::<TSEnumMember>() == 48usize);
    assert!(align_of::<TSEnumMember>() == 8usize);
    assert!(offset_of!(TSEnumMember, node_id) == 0usize);
    assert!(offset_of!(TSEnumMember, span) == 4usize);
    assert!(offset_of!(TSEnumMember, id) == 16usize);
    assert!(offset_of!(TSEnumMember, initializer) == 32usize);

    assert!(size_of::<TSEnumMemberName>() == 16usize);
    assert!(align_of::<TSEnumMemberName>() == 8usize);

    assert!(size_of::<TSTypeAnnotation>() == 32usize);
    assert!(align_of::<TSTypeAnnotation>() == 8usize);
    assert!(offset_of!(TSTypeAnnotation, node_id) == 0usize);
    assert!(offset_of!(TSTypeAnnotation, span) == 4usize);
    assert!(offset_of!(TSTypeAnnotation, type_annotation) == 16usize);

    assert!(size_of::<TSLiteralType>() == 32usize);
    assert!(align_of::<TSLiteralType>() == 8usize);
    assert!(offset_of!(TSLiteralType, node_id) == 0usize);
    assert!(offset_of!(TSLiteralType, span) == 4usize);
    assert!(offset_of!(TSLiteralType, literal) == 16usize);

    assert!(size_of::<TSLiteral>() == 16usize);
    assert!(align_of::<TSLiteral>() == 8usize);

    assert!(size_of::<TSType>() == 16usize);
    assert!(align_of::<TSType>() == 8usize);

    assert!(size_of::<TSConditionalType>() == 88usize);
    assert!(align_of::<TSConditionalType>() == 8usize);
    assert!(offset_of!(TSConditionalType, node_id) == 0usize);
    assert!(offset_of!(TSConditionalType, span) == 4usize);
    assert!(offset_of!(TSConditionalType, check_type) == 16usize);
    assert!(offset_of!(TSConditionalType, extends_type) == 32usize);
    assert!(offset_of!(TSConditionalType, true_type) == 48usize);
    assert!(offset_of!(TSConditionalType, false_type) == 64usize);
    assert!(offset_of!(TSConditionalType, scope_id) == 80usize);

    assert!(size_of::<TSUnionType>() == 48usize);
    assert!(align_of::<TSUnionType>() == 8usize);
    assert!(offset_of!(TSUnionType, node_id) == 0usize);
    assert!(offset_of!(TSUnionType, span) == 4usize);
    assert!(offset_of!(TSUnionType, types) == 16usize);

    assert!(size_of::<TSIntersectionType>() == 48usize);
    assert!(align_of::<TSIntersectionType>() == 8usize);
    assert!(offset_of!(TSIntersectionType, node_id) == 0usize);
    assert!(offset_of!(TSIntersectionType, span) == 4usize);
    assert!(offset_of!(TSIntersectionType, types) == 16usize);

    assert!(size_of::<TSParenthesizedType>() == 32usize);
    assert!(align_of::<TSParenthesizedType>() == 8usize);
    assert!(offset_of!(TSParenthesizedType, node_id) == 0usize);
    assert!(offset_of!(TSParenthesizedType, span) == 4usize);
    assert!(offset_of!(TSParenthesizedType, type_annotation) == 16usize);

    assert!(size_of::<TSTypeOperator>() == 32usize);
    assert!(align_of::<TSTypeOperator>() == 8usize);
    assert!(offset_of!(TSTypeOperator, node_id) == 0usize);
    assert!(offset_of!(TSTypeOperator, span) == 4usize);
    assert!(offset_of!(TSTypeOperator, operator) == 12usize);
    assert!(offset_of!(TSTypeOperator, type_annotation) == 16usize);

    assert!(size_of::<TSTypeOperatorOperator>() == 1usize);
    assert!(align_of::<TSTypeOperatorOperator>() == 1usize);

    assert!(size_of::<TSArrayType>() == 32usize);
    assert!(align_of::<TSArrayType>() == 8usize);
    assert!(offset_of!(TSArrayType, node_id) == 0usize);
    assert!(offset_of!(TSArrayType, span) == 4usize);
    assert!(offset_of!(TSArrayType, element_type) == 16usize);

    assert!(size_of::<TSIndexedAccessType>() == 48usize);
    assert!(align_of::<TSIndexedAccessType>() == 8usize);
    assert!(offset_of!(TSIndexedAccessType, node_id) == 0usize);
    assert!(offset_of!(TSIndexedAccessType, span) == 4usize);
    assert!(offset_of!(TSIndexedAccessType, object_type) == 16usize);
    assert!(offset_of!(TSIndexedAccessType, index_type) == 32usize);

    assert!(size_of::<TSTupleType>() == 48usize);
    assert!(align_of::<TSTupleType>() == 8usize);
    assert!(offset_of!(TSTupleType, node_id) == 0usize);
    assert!(offset_of!(TSTupleType, span) == 4usize);
    assert!(offset_of!(TSTupleType, element_types) == 16usize);

    assert!(size_of::<TSNamedTupleMember>() == 72usize);
    assert!(align_of::<TSNamedTupleMember>() == 8usize);
    assert!(offset_of!(TSNamedTupleMember, node_id) == 0usize);
    assert!(offset_of!(TSNamedTupleMember, span) == 4usize);
    assert!(offset_of!(TSNamedTupleMember, element_type) == 16usize);
    assert!(offset_of!(TSNamedTupleMember, label) == 32usize);
    assert!(offset_of!(TSNamedTupleMember, optional) == 64usize);

    assert!(size_of::<TSOptionalType>() == 32usize);
    assert!(align_of::<TSOptionalType>() == 8usize);
    assert!(offset_of!(TSOptionalType, node_id) == 0usize);
    assert!(offset_of!(TSOptionalType, span) == 4usize);
    assert!(offset_of!(TSOptionalType, type_annotation) == 16usize);

    assert!(size_of::<TSRestType>() == 32usize);
    assert!(align_of::<TSRestType>() == 8usize);
    assert!(offset_of!(TSRestType, node_id) == 0usize);
    assert!(offset_of!(TSRestType, span) == 4usize);
    assert!(offset_of!(TSRestType, type_annotation) == 16usize);

    assert!(size_of::<TSTupleElement>() == 16usize);
    assert!(align_of::<TSTupleElement>() == 8usize);

    assert!(size_of::<TSAnyKeyword>() == 12usize);
    assert!(align_of::<TSAnyKeyword>() == 4usize);
    assert!(offset_of!(TSAnyKeyword, node_id) == 0usize);
    assert!(offset_of!(TSAnyKeyword, span) == 4usize);

    assert!(size_of::<TSStringKeyword>() == 12usize);
    assert!(align_of::<TSStringKeyword>() == 4usize);
    assert!(offset_of!(TSStringKeyword, node_id) == 0usize);
    assert!(offset_of!(TSStringKeyword, span) == 4usize);

    assert!(size_of::<TSBooleanKeyword>() == 12usize);
    assert!(align_of::<TSBooleanKeyword>() == 4usize);
    assert!(offset_of!(TSBooleanKeyword, node_id) == 0usize);
    assert!(offset_of!(TSBooleanKeyword, span) == 4usize);

    assert!(size_of::<TSNumberKeyword>() == 12usize);
    assert!(align_of::<TSNumberKeyword>() == 4usize);
    assert!(offset_of!(TSNumberKeyword, node_id) == 0usize);
    assert!(offset_of!(TSNumberKeyword, span) == 4usize);

    assert!(size_of::<TSNeverKeyword>() == 12usize);
    assert!(align_of::<TSNeverKeyword>() == 4usize);
    assert!(offset_of!(TSNeverKeyword, node_id) == 0usize);
    assert!(offset_of!(TSNeverKeyword, span) == 4usize);

    assert!(size_of::<TSIntrinsicKeyword>() == 12usize);
    assert!(align_of::<TSIntrinsicKeyword>() == 4usize);
    assert!(offset_of!(TSIntrinsicKeyword, node_id) == 0usize);
    assert!(offset_of!(TSIntrinsicKeyword, span) == 4usize);

    assert!(size_of::<TSUnknownKeyword>() == 12usize);
    assert!(align_of::<TSUnknownKeyword>() == 4usize);
    assert!(offset_of!(TSUnknownKeyword, node_id) == 0usize);
    assert!(offset_of!(TSUnknownKeyword, span) == 4usize);

    assert!(size_of::<TSNullKeyword>() == 12usize);
    assert!(align_of::<TSNullKeyword>() == 4usize);
    assert!(offset_of!(TSNullKeyword, node_id) == 0usize);
    assert!(offset_of!(TSNullKeyword, span) == 4usize);

    assert!(size_of::<TSUndefinedKeyword>() == 12usize);
    assert!(align_of::<TSUndefinedKeyword>() == 4usize);
    assert!(offset_of!(TSUndefinedKeyword, node_id) == 0usize);
    assert!(offset_of!(TSUndefinedKeyword, span) == 4usize);

    assert!(size_of::<TSVoidKeyword>() == 12usize);
    assert!(align_of::<TSVoidKeyword>() == 4usize);
    assert!(offset_of!(TSVoidKeyword, node_id) == 0usize);
    assert!(offset_of!(TSVoidKeyword, span) == 4usize);

    assert!(size_of::<TSSymbolKeyword>() == 12usize);
    assert!(align_of::<TSSymbolKeyword>() == 4usize);
    assert!(offset_of!(TSSymbolKeyword, node_id) == 0usize);
    assert!(offset_of!(TSSymbolKeyword, span) == 4usize);

    assert!(size_of::<TSThisType>() == 12usize);
    assert!(align_of::<TSThisType>() == 4usize);
    assert!(offset_of!(TSThisType, node_id) == 0usize);
    assert!(offset_of!(TSThisType, span) == 4usize);

    assert!(size_of::<TSObjectKeyword>() == 12usize);
    assert!(align_of::<TSObjectKeyword>() == 4usize);
    assert!(offset_of!(TSObjectKeyword, node_id) == 0usize);
    assert!(offset_of!(TSObjectKeyword, span) == 4usize);

    assert!(size_of::<TSBigIntKeyword>() == 12usize);
    assert!(align_of::<TSBigIntKeyword>() == 4usize);
    assert!(offset_of!(TSBigIntKeyword, node_id) == 0usize);
    assert!(offset_of!(TSBigIntKeyword, span) == 4usize);

    assert!(size_of::<TSTypeReference>() == 40usize);
    assert!(align_of::<TSTypeReference>() == 8usize);
    assert!(offset_of!(TSTypeReference, node_id) == 0usize);
    assert!(offset_of!(TSTypeReference, span) == 4usize);
    assert!(offset_of!(TSTypeReference, type_name) == 16usize);
    assert!(offset_of!(TSTypeReference, type_parameters) == 32usize);

    assert!(size_of::<TSTypeName>() == 16usize);
    assert!(align_of::<TSTypeName>() == 8usize);

    assert!(size_of::<TSQualifiedName>() == 64usize);
    assert!(align_of::<TSQualifiedName>() == 8usize);
    assert!(offset_of!(TSQualifiedName, node_id) == 0usize);
    assert!(offset_of!(TSQualifiedName, span) == 4usize);
    assert!(offset_of!(TSQualifiedName, left) == 16usize);
    assert!(offset_of!(TSQualifiedName, right) == 32usize);

    assert!(size_of::<TSTypeParameterInstantiation>() == 48usize);
    assert!(align_of::<TSTypeParameterInstantiation>() == 8usize);
    assert!(offset_of!(TSTypeParameterInstantiation, node_id) == 0usize);
    assert!(offset_of!(TSTypeParameterInstantiation, span) == 4usize);
    assert!(offset_of!(TSTypeParameterInstantiation, params) == 16usize);

    assert!(size_of::<TSTypeParameter>() == 96usize);
    assert!(align_of::<TSTypeParameter>() == 8usize);
    assert!(offset_of!(TSTypeParameter, node_id) == 0usize);
    assert!(offset_of!(TSTypeParameter, span) == 4usize);
    assert!(offset_of!(TSTypeParameter, name) == 16usize);
    assert!(offset_of!(TSTypeParameter, constraint) == 56usize);
    assert!(offset_of!(TSTypeParameter, default) == 72usize);
    assert!(offset_of!(TSTypeParameter, r#in) == 88usize);
    assert!(offset_of!(TSTypeParameter, out) == 89usize);
    assert!(offset_of!(TSTypeParameter, r#const) == 90usize);

    assert!(size_of::<TSTypeParameterDeclaration>() == 48usize);
    assert!(align_of::<TSTypeParameterDeclaration>() == 8usize);
    assert!(offset_of!(TSTypeParameterDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSTypeParameterDeclaration, span) == 4usize);
    assert!(offset_of!(TSTypeParameterDeclaration, params) == 16usize);

    assert!(size_of::<TSTypeAliasDeclaration>() == 88usize);
    assert!(align_of::<TSTypeAliasDeclaration>() == 8usize);
    assert!(offset_of!(TSTypeAliasDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSTypeAliasDeclaration, span) == 4usize);
    assert!(offset_of!(TSTypeAliasDeclaration, id) == 16usize);
    assert!(offset_of!(TSTypeAliasDeclaration, type_parameters) == 56usize);
    assert!(offset_of!(TSTypeAliasDeclaration, type_annotation) == 64usize);
    assert!(offset_of!(TSTypeAliasDeclaration, declare) == 80usize);
    assert!(offset_of!(TSTypeAliasDeclaration, scope_id) == 84usize);

    assert!(size_of::<TSAccessibility>() == 1usize);
    assert!(align_of::<TSAccessibility>() == 1usize);

    assert!(size_of::<TSClassImplements>() == 40usize);
    assert!(align_of::<TSClassImplements>() == 8usize);
    assert!(offset_of!(TSClassImplements, node_id) == 0usize);
    assert!(offset_of!(TSClassImplements, span) == 4usize);
    assert!(offset_of!(TSClassImplements, expression) == 16usize);
    assert!(offset_of!(TSClassImplements, type_parameters) == 32usize);

    assert!(size_of::<TSInterfaceDeclaration>() == 112usize);
    assert!(align_of::<TSInterfaceDeclaration>() == 8usize);
    assert!(offset_of!(TSInterfaceDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSInterfaceDeclaration, span) == 4usize);
    assert!(offset_of!(TSInterfaceDeclaration, id) == 16usize);
    assert!(offset_of!(TSInterfaceDeclaration, extends) == 56usize);
    assert!(offset_of!(TSInterfaceDeclaration, type_parameters) == 88usize);
    assert!(offset_of!(TSInterfaceDeclaration, body) == 96usize);
    assert!(offset_of!(TSInterfaceDeclaration, declare) == 104usize);
    assert!(offset_of!(TSInterfaceDeclaration, scope_id) == 108usize);

    assert!(size_of::<TSInterfaceBody>() == 48usize);
    assert!(align_of::<TSInterfaceBody>() == 8usize);
    assert!(offset_of!(TSInterfaceBody, node_id) == 0usize);
    assert!(offset_of!(TSInterfaceBody, span) == 4usize);
    assert!(offset_of!(TSInterfaceBody, body) == 16usize);

    assert!(size_of::<TSPropertySignature>() == 40usize);
    assert!(align_of::<TSPropertySignature>() == 8usize);
    assert!(offset_of!(TSPropertySignature, node_id) == 0usize);
    assert!(offset_of!(TSPropertySignature, span) == 4usize);
    assert!(offset_of!(TSPropertySignature, computed) == 12usize);
    assert!(offset_of!(TSPropertySignature, optional) == 13usize);
    assert!(offset_of!(TSPropertySignature, readonly) == 14usize);
    assert!(offset_of!(TSPropertySignature, key) == 16usize);
    assert!(offset_of!(TSPropertySignature, type_annotation) == 32usize);

    assert!(size_of::<TSSignature>() == 16usize);
    assert!(align_of::<TSSignature>() == 8usize);

    assert!(size_of::<TSIndexSignature>() == 64usize);
    assert!(align_of::<TSIndexSignature>() == 8usize);
    assert!(offset_of!(TSIndexSignature, node_id) == 0usize);
    assert!(offset_of!(TSIndexSignature, span) == 4usize);
    assert!(offset_of!(TSIndexSignature, parameters) == 16usize);
    assert!(offset_of!(TSIndexSignature, type_annotation) == 48usize);
    assert!(offset_of!(TSIndexSignature, readonly) == 56usize);

    assert!(size_of::<TSCallSignatureDeclaration>() == 72usize);
    assert!(align_of::<TSCallSignatureDeclaration>() == 8usize);
    assert!(offset_of!(TSCallSignatureDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSCallSignatureDeclaration, span) == 4usize);
    assert!(offset_of!(TSCallSignatureDeclaration, this_param) == 16usize);
    assert!(offset_of!(TSCallSignatureDeclaration, params) == 48usize);
    assert!(offset_of!(TSCallSignatureDeclaration, return_type) == 56usize);
    assert!(offset_of!(TSCallSignatureDeclaration, type_parameters) == 64usize);

    assert!(size_of::<TSMethodSignatureKind>() == 1usize);
    assert!(align_of::<TSMethodSignatureKind>() == 1usize);

    assert!(size_of::<TSMethodSignature>() == 80usize);
    assert!(align_of::<TSMethodSignature>() == 8usize);
    assert!(offset_of!(TSMethodSignature, node_id) == 0usize);
    assert!(offset_of!(TSMethodSignature, span) == 4usize);
    assert!(offset_of!(TSMethodSignature, key) == 16usize);
    assert!(offset_of!(TSMethodSignature, computed) == 32usize);
    assert!(offset_of!(TSMethodSignature, optional) == 33usize);
    assert!(offset_of!(TSMethodSignature, kind) == 34usize);
    assert!(offset_of!(TSMethodSignature, this_param) == 40usize);
    assert!(offset_of!(TSMethodSignature, params) == 48usize);
    assert!(offset_of!(TSMethodSignature, return_type) == 56usize);
    assert!(offset_of!(TSMethodSignature, type_parameters) == 64usize);
    assert!(offset_of!(TSMethodSignature, scope_id) == 72usize);

    assert!(size_of::<TSConstructSignatureDeclaration>() == 48usize);
    assert!(align_of::<TSConstructSignatureDeclaration>() == 8usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, span) == 4usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, params) == 16usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, return_type) == 24usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, type_parameters) == 32usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, scope_id) == 40usize);

    assert!(size_of::<TSIndexSignatureName>() == 40usize);
    assert!(align_of::<TSIndexSignatureName>() == 8usize);
    assert!(offset_of!(TSIndexSignatureName, node_id) == 0usize);
    assert!(offset_of!(TSIndexSignatureName, span) == 4usize);
    assert!(offset_of!(TSIndexSignatureName, name) == 16usize);
    assert!(offset_of!(TSIndexSignatureName, type_annotation) == 32usize);

    assert!(size_of::<TSInterfaceHeritage>() == 40usize);
    assert!(align_of::<TSInterfaceHeritage>() == 8usize);
    assert!(offset_of!(TSInterfaceHeritage, node_id) == 0usize);
    assert!(offset_of!(TSInterfaceHeritage, span) == 4usize);
    assert!(offset_of!(TSInterfaceHeritage, expression) == 16usize);
    assert!(offset_of!(TSInterfaceHeritage, type_parameters) == 32usize);

    assert!(size_of::<TSTypePredicate>() == 56usize);
    assert!(align_of::<TSTypePredicate>() == 8usize);
    assert!(offset_of!(TSTypePredicate, node_id) == 0usize);
    assert!(offset_of!(TSTypePredicate, span) == 4usize);
    assert!(offset_of!(TSTypePredicate, parameter_name) == 16usize);
    assert!(offset_of!(TSTypePredicate, asserts) == 40usize);
    assert!(offset_of!(TSTypePredicate, type_annotation) == 48usize);

    assert!(size_of::<TSTypePredicateName>() == 24usize);
    assert!(align_of::<TSTypePredicateName>() == 8usize);

    assert!(size_of::<TSModuleDeclaration>() == 80usize);
    assert!(align_of::<TSModuleDeclaration>() == 8usize);
    assert!(offset_of!(TSModuleDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSModuleDeclaration, span) == 4usize);
    assert!(offset_of!(TSModuleDeclaration, id) == 16usize);
    assert!(offset_of!(TSModuleDeclaration, body) == 56usize);
    assert!(offset_of!(TSModuleDeclaration, kind) == 72usize);
    assert!(offset_of!(TSModuleDeclaration, declare) == 73usize);
    assert!(offset_of!(TSModuleDeclaration, scope_id) == 76usize);

    assert!(size_of::<TSModuleDeclarationKind>() == 1usize);
    assert!(align_of::<TSModuleDeclarationKind>() == 1usize);

    assert!(size_of::<TSModuleDeclarationName>() == 40usize);
    assert!(align_of::<TSModuleDeclarationName>() == 8usize);

    assert!(size_of::<TSModuleDeclarationBody>() == 16usize);
    assert!(align_of::<TSModuleDeclarationBody>() == 8usize);

    assert!(size_of::<TSModuleBlock>() == 80usize);
    assert!(align_of::<TSModuleBlock>() == 8usize);
    assert!(offset_of!(TSModuleBlock, node_id) == 0usize);
    assert!(offset_of!(TSModuleBlock, span) == 4usize);
    assert!(offset_of!(TSModuleBlock, directives) == 16usize);
    assert!(offset_of!(TSModuleBlock, body) == 48usize);

    assert!(size_of::<TSTypeLiteral>() == 48usize);
    assert!(align_of::<TSTypeLiteral>() == 8usize);
    assert!(offset_of!(TSTypeLiteral, node_id) == 0usize);
    assert!(offset_of!(TSTypeLiteral, span) == 4usize);
    assert!(offset_of!(TSTypeLiteral, members) == 16usize);

    assert!(size_of::<TSInferType>() == 24usize);
    assert!(align_of::<TSInferType>() == 8usize);
    assert!(offset_of!(TSInferType, node_id) == 0usize);
    assert!(offset_of!(TSInferType, span) == 4usize);
    assert!(offset_of!(TSInferType, type_parameter) == 16usize);

    assert!(size_of::<TSTypeQuery>() == 40usize);
    assert!(align_of::<TSTypeQuery>() == 8usize);
    assert!(offset_of!(TSTypeQuery, node_id) == 0usize);
    assert!(offset_of!(TSTypeQuery, span) == 4usize);
    assert!(offset_of!(TSTypeQuery, expr_name) == 16usize);
    assert!(offset_of!(TSTypeQuery, type_parameters) == 32usize);

    assert!(size_of::<TSTypeQueryExprName>() == 16usize);
    assert!(align_of::<TSTypeQueryExprName>() == 8usize);

    assert!(size_of::<TSImportType>() == 64usize);
    assert!(align_of::<TSImportType>() == 8usize);
    assert!(offset_of!(TSImportType, node_id) == 0usize);
    assert!(offset_of!(TSImportType, span) == 4usize);
    assert!(offset_of!(TSImportType, is_type_of) == 12usize);
    assert!(offset_of!(TSImportType, parameter) == 16usize);
    assert!(offset_of!(TSImportType, qualifier) == 32usize);
    assert!(offset_of!(TSImportType, attributes) == 48usize);
    assert!(offset_of!(TSImportType, type_parameters) == 56usize);

    assert!(size_of::<TSImportAttributes>() == 80usize);
    assert!(align_of::<TSImportAttributes>() == 8usize);
    assert!(offset_of!(TSImportAttributes, node_id) == 0usize);
    assert!(offset_of!(TSImportAttributes, span) == 4usize);
    assert!(offset_of!(TSImportAttributes, attributes_keyword) == 16usize);
    assert!(offset_of!(TSImportAttributes, elements) == 48usize);

    assert!(size_of::<TSImportAttribute>() == 72usize);
    assert!(align_of::<TSImportAttribute>() == 8usize);
    assert!(offset_of!(TSImportAttribute, node_id) == 0usize);
    assert!(offset_of!(TSImportAttribute, span) == 4usize);
    assert!(offset_of!(TSImportAttribute, name) == 16usize);
    assert!(offset_of!(TSImportAttribute, value) == 56usize);

    assert!(size_of::<TSImportAttributeName>() == 40usize);
    assert!(align_of::<TSImportAttributeName>() == 8usize);

    assert!(size_of::<TSFunctionType>() == 48usize);
    assert!(align_of::<TSFunctionType>() == 8usize);
    assert!(offset_of!(TSFunctionType, node_id) == 0usize);
    assert!(offset_of!(TSFunctionType, span) == 4usize);
    assert!(offset_of!(TSFunctionType, this_param) == 16usize);
    assert!(offset_of!(TSFunctionType, params) == 24usize);
    assert!(offset_of!(TSFunctionType, return_type) == 32usize);
    assert!(offset_of!(TSFunctionType, type_parameters) == 40usize);

    assert!(size_of::<TSConstructorType>() == 40usize);
    assert!(align_of::<TSConstructorType>() == 8usize);
    assert!(offset_of!(TSConstructorType, node_id) == 0usize);
    assert!(offset_of!(TSConstructorType, span) == 4usize);
    assert!(offset_of!(TSConstructorType, r#abstract) == 12usize);
    assert!(offset_of!(TSConstructorType, params) == 16usize);
    assert!(offset_of!(TSConstructorType, return_type) == 24usize);
    assert!(offset_of!(TSConstructorType, type_parameters) == 32usize);

    assert!(size_of::<TSMappedType>() == 64usize);
    assert!(align_of::<TSMappedType>() == 8usize);
    assert!(offset_of!(TSMappedType, node_id) == 0usize);
    assert!(offset_of!(TSMappedType, span) == 4usize);
    assert!(offset_of!(TSMappedType, type_parameter) == 16usize);
    assert!(offset_of!(TSMappedType, name_type) == 24usize);
    assert!(offset_of!(TSMappedType, type_annotation) == 40usize);
    assert!(offset_of!(TSMappedType, optional) == 56usize);
    assert!(offset_of!(TSMappedType, readonly) == 57usize);
    assert!(offset_of!(TSMappedType, scope_id) == 60usize);

    assert!(size_of::<TSMappedTypeModifierOperator>() == 1usize);
    assert!(align_of::<TSMappedTypeModifierOperator>() == 1usize);

    assert!(size_of::<TSTemplateLiteralType>() == 80usize);
    assert!(align_of::<TSTemplateLiteralType>() == 8usize);
    assert!(offset_of!(TSTemplateLiteralType, node_id) == 0usize);
    assert!(offset_of!(TSTemplateLiteralType, span) == 4usize);
    assert!(offset_of!(TSTemplateLiteralType, quasis) == 16usize);
    assert!(offset_of!(TSTemplateLiteralType, types) == 48usize);

    assert!(size_of::<TSAsExpression>() == 48usize);
    assert!(align_of::<TSAsExpression>() == 8usize);
    assert!(offset_of!(TSAsExpression, node_id) == 0usize);
    assert!(offset_of!(TSAsExpression, span) == 4usize);
    assert!(offset_of!(TSAsExpression, expression) == 16usize);
    assert!(offset_of!(TSAsExpression, type_annotation) == 32usize);

    assert!(size_of::<TSSatisfiesExpression>() == 48usize);
    assert!(align_of::<TSSatisfiesExpression>() == 8usize);
    assert!(offset_of!(TSSatisfiesExpression, node_id) == 0usize);
    assert!(offset_of!(TSSatisfiesExpression, span) == 4usize);
    assert!(offset_of!(TSSatisfiesExpression, expression) == 16usize);
    assert!(offset_of!(TSSatisfiesExpression, type_annotation) == 32usize);

    assert!(size_of::<TSTypeAssertion>() == 48usize);
    assert!(align_of::<TSTypeAssertion>() == 8usize);
    assert!(offset_of!(TSTypeAssertion, node_id) == 0usize);
    assert!(offset_of!(TSTypeAssertion, span) == 4usize);
    assert!(offset_of!(TSTypeAssertion, expression) == 16usize);
    assert!(offset_of!(TSTypeAssertion, type_annotation) == 32usize);

    assert!(size_of::<TSImportEqualsDeclaration>() == 80usize);
    assert!(align_of::<TSImportEqualsDeclaration>() == 8usize);
    assert!(offset_of!(TSImportEqualsDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSImportEqualsDeclaration, span) == 4usize);
    assert!(offset_of!(TSImportEqualsDeclaration, id) == 16usize);
    assert!(offset_of!(TSImportEqualsDeclaration, module_reference) == 56usize);
    assert!(offset_of!(TSImportEqualsDeclaration, import_kind) == 72usize);

    assert!(size_of::<TSModuleReference>() == 16usize);
    assert!(align_of::<TSModuleReference>() == 8usize);

    assert!(size_of::<TSExternalModuleReference>() == 48usize);
    assert!(align_of::<TSExternalModuleReference>() == 8usize);
    assert!(offset_of!(TSExternalModuleReference, node_id) == 0usize);
    assert!(offset_of!(TSExternalModuleReference, span) == 4usize);
    assert!(offset_of!(TSExternalModuleReference, expression) == 16usize);

    assert!(size_of::<TSNonNullExpression>() == 32usize);
    assert!(align_of::<TSNonNullExpression>() == 8usize);
    assert!(offset_of!(TSNonNullExpression, node_id) == 0usize);
    assert!(offset_of!(TSNonNullExpression, span) == 4usize);
    assert!(offset_of!(TSNonNullExpression, expression) == 16usize);

    assert!(size_of::<Decorator>() == 32usize);
    assert!(align_of::<Decorator>() == 8usize);
    assert!(offset_of!(Decorator, node_id) == 0usize);
    assert!(offset_of!(Decorator, span) == 4usize);
    assert!(offset_of!(Decorator, expression) == 16usize);

    assert!(size_of::<TSExportAssignment>() == 32usize);
    assert!(align_of::<TSExportAssignment>() == 8usize);
    assert!(offset_of!(TSExportAssignment, node_id) == 0usize);
    assert!(offset_of!(TSExportAssignment, span) == 4usize);
    assert!(offset_of!(TSExportAssignment, expression) == 16usize);

    assert!(size_of::<TSNamespaceExportDeclaration>() == 48usize);
    assert!(align_of::<TSNamespaceExportDeclaration>() == 8usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, span) == 4usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, id) == 16usize);

    assert!(size_of::<TSInstantiationExpression>() == 40usize);
    assert!(align_of::<TSInstantiationExpression>() == 8usize);
    assert!(offset_of!(TSInstantiationExpression, node_id) == 0usize);
    assert!(offset_of!(TSInstantiationExpression, span) == 4usize);
    assert!(offset_of!(TSInstantiationExpression, expression) == 16usize);
    assert!(offset_of!(TSInstantiationExpression, type_parameters) == 32usize);

    assert!(size_of::<ImportOrExportKind>() == 1usize);
    assert!(align_of::<ImportOrExportKind>() == 1usize);

    assert!(size_of::<JSDocNullableType>() == 40usize);
    assert!(align_of::<JSDocNullableType>() == 8usize);
    assert!(offset_of!(JSDocNullableType, node_id) == 0usize);
    assert!(offset_of!(JSDocNullableType, span) == 4usize);
    assert!(offset_of!(JSDocNullableType, type_annotation) == 16usize);
    assert!(offset_of!(JSDocNullableType, postfix) == 32usize);

    assert!(size_of::<JSDocNonNullableType>() == 40usize);
    assert!(align_of::<JSDocNonNullableType>() == 8usize);
    assert!(offset_of!(JSDocNonNullableType, node_id) == 0usize);
    assert!(offset_of!(JSDocNonNullableType, span) == 4usize);
    assert!(offset_of!(JSDocNonNullableType, type_annotation) == 16usize);
    assert!(offset_of!(JSDocNonNullableType, postfix) == 32usize);

    assert!(size_of::<JSDocUnknownType>() == 12usize);
    assert!(align_of::<JSDocUnknownType>() == 4usize);
    assert!(offset_of!(JSDocUnknownType, node_id) == 0usize);
    assert!(offset_of!(JSDocUnknownType, span) == 4usize);

    assert!(size_of::<JSXElement>() == 64usize);
    assert!(align_of::<JSXElement>() == 8usize);
    assert!(offset_of!(JSXElement, node_id) == 0usize);
    assert!(offset_of!(JSXElement, span) == 4usize);
    assert!(offset_of!(JSXElement, opening_element) == 16usize);
    assert!(offset_of!(JSXElement, closing_element) == 24usize);
    assert!(offset_of!(JSXElement, children) == 32usize);

    assert!(size_of::<JSXOpeningElement>() == 72usize);
    assert!(align_of::<JSXOpeningElement>() == 8usize);
    assert!(offset_of!(JSXOpeningElement, node_id) == 0usize);
    assert!(offset_of!(JSXOpeningElement, span) == 4usize);
    assert!(offset_of!(JSXOpeningElement, self_closing) == 12usize);
    assert!(offset_of!(JSXOpeningElement, name) == 16usize);
    assert!(offset_of!(JSXOpeningElement, attributes) == 32usize);
    assert!(offset_of!(JSXOpeningElement, type_parameters) == 64usize);

    assert!(size_of::<JSXClosingElement>() == 32usize);
    assert!(align_of::<JSXClosingElement>() == 8usize);
    assert!(offset_of!(JSXClosingElement, node_id) == 0usize);
    assert!(offset_of!(JSXClosingElement, span) == 4usize);
    assert!(offset_of!(JSXClosingElement, name) == 16usize);

    assert!(size_of::<JSXFragment>() == 64usize);
    assert!(align_of::<JSXFragment>() == 8usize);
    assert!(offset_of!(JSXFragment, node_id) == 0usize);
    assert!(offset_of!(JSXFragment, span) == 4usize);
    assert!(offset_of!(JSXFragment, opening_fragment) == 12usize);
    assert!(offset_of!(JSXFragment, closing_fragment) == 20usize);
    assert!(offset_of!(JSXFragment, children) == 32usize);

    assert!(size_of::<JSXOpeningFragment>() == 8usize);
    assert!(align_of::<JSXOpeningFragment>() == 4usize);
    assert!(offset_of!(JSXOpeningFragment, span) == 0usize);

    assert!(size_of::<JSXClosingFragment>() == 8usize);
    assert!(align_of::<JSXClosingFragment>() == 4usize);
    assert!(offset_of!(JSXClosingFragment, span) == 0usize);

    assert!(size_of::<JSXElementName>() == 16usize);
    assert!(align_of::<JSXElementName>() == 8usize);

    assert!(size_of::<JSXNamespacedName>() == 80usize);
    assert!(align_of::<JSXNamespacedName>() == 8usize);
    assert!(offset_of!(JSXNamespacedName, node_id) == 0usize);
    assert!(offset_of!(JSXNamespacedName, span) == 4usize);
    assert!(offset_of!(JSXNamespacedName, namespace) == 16usize);
    assert!(offset_of!(JSXNamespacedName, property) == 48usize);

    assert!(size_of::<JSXMemberExpression>() == 64usize);
    assert!(align_of::<JSXMemberExpression>() == 8usize);
    assert!(offset_of!(JSXMemberExpression, node_id) == 0usize);
    assert!(offset_of!(JSXMemberExpression, span) == 4usize);
    assert!(offset_of!(JSXMemberExpression, object) == 16usize);
    assert!(offset_of!(JSXMemberExpression, property) == 32usize);

    assert!(size_of::<JSXMemberExpressionObject>() == 16usize);
    assert!(align_of::<JSXMemberExpressionObject>() == 8usize);

    assert!(size_of::<JSXExpressionContainer>() == 40usize);
    assert!(align_of::<JSXExpressionContainer>() == 8usize);
    assert!(offset_of!(JSXExpressionContainer, node_id) == 0usize);
    assert!(offset_of!(JSXExpressionContainer, span) == 4usize);
    assert!(offset_of!(JSXExpressionContainer, expression) == 16usize);

    assert!(size_of::<JSXExpression>() == 24usize);
    assert!(align_of::<JSXExpression>() == 8usize);

    assert!(size_of::<JSXEmptyExpression>() == 12usize);
    assert!(align_of::<JSXEmptyExpression>() == 4usize);
    assert!(offset_of!(JSXEmptyExpression, node_id) == 0usize);
    assert!(offset_of!(JSXEmptyExpression, span) == 4usize);

    assert!(size_of::<JSXAttributeItem>() == 16usize);
    assert!(align_of::<JSXAttributeItem>() == 8usize);

    assert!(size_of::<JSXAttribute>() == 48usize);
    assert!(align_of::<JSXAttribute>() == 8usize);
    assert!(offset_of!(JSXAttribute, node_id) == 0usize);
    assert!(offset_of!(JSXAttribute, span) == 4usize);
    assert!(offset_of!(JSXAttribute, name) == 16usize);
    assert!(offset_of!(JSXAttribute, value) == 32usize);

    assert!(size_of::<JSXSpreadAttribute>() == 32usize);
    assert!(align_of::<JSXSpreadAttribute>() == 8usize);
    assert!(offset_of!(JSXSpreadAttribute, node_id) == 0usize);
    assert!(offset_of!(JSXSpreadAttribute, span) == 4usize);
    assert!(offset_of!(JSXSpreadAttribute, argument) == 16usize);

    assert!(size_of::<JSXAttributeName>() == 16usize);
    assert!(align_of::<JSXAttributeName>() == 8usize);

    assert!(size_of::<JSXAttributeValue>() == 16usize);
    assert!(align_of::<JSXAttributeValue>() == 8usize);

    assert!(size_of::<JSXIdentifier>() == 32usize);
    assert!(align_of::<JSXIdentifier>() == 8usize);
    assert!(offset_of!(JSXIdentifier, node_id) == 0usize);
    assert!(offset_of!(JSXIdentifier, span) == 4usize);
    assert!(offset_of!(JSXIdentifier, name) == 16usize);

    assert!(size_of::<JSXChild>() == 16usize);
    assert!(align_of::<JSXChild>() == 8usize);

    assert!(size_of::<JSXSpreadChild>() == 32usize);
    assert!(align_of::<JSXSpreadChild>() == 8usize);
    assert!(offset_of!(JSXSpreadChild, node_id) == 0usize);
    assert!(offset_of!(JSXSpreadChild, span) == 4usize);
    assert!(offset_of!(JSXSpreadChild, expression) == 16usize);

    assert!(size_of::<JSXText>() == 32usize);
    assert!(align_of::<JSXText>() == 8usize);
    assert!(offset_of!(JSXText, node_id) == 0usize);
    assert!(offset_of!(JSXText, span) == 4usize);
    assert!(offset_of!(JSXText, value) == 16usize);

    assert!(size_of::<NodeId>() == 4usize);
    assert!(align_of::<NodeId>() == 4usize);

    assert!(size_of::<NumberBase>() == 1usize);
    assert!(align_of::<NumberBase>() == 1usize);

    assert!(size_of::<BigintBase>() == 1usize);
    assert!(align_of::<BigintBase>() == 1usize);

    assert!(size_of::<AssignmentOperator>() == 1usize);
    assert!(align_of::<AssignmentOperator>() == 1usize);

    assert!(size_of::<BinaryOperator>() == 1usize);
    assert!(align_of::<BinaryOperator>() == 1usize);

    assert!(size_of::<LogicalOperator>() == 1usize);
    assert!(align_of::<LogicalOperator>() == 1usize);

    assert!(size_of::<UnaryOperator>() == 1usize);
    assert!(align_of::<UnaryOperator>() == 1usize);

    assert!(size_of::<UpdateOperator>() == 1usize);
    assert!(align_of::<UpdateOperator>() == 1usize);

    assert!(size_of::<Span>() == 8usize);
    assert!(align_of::<Span>() == 4usize);
    assert!(offset_of!(Span, start) == 0usize);
    assert!(offset_of!(Span, end) == 4usize);

    assert!(size_of::<SourceType>() == 3usize);
    assert!(align_of::<SourceType>() == 1usize);

    assert!(size_of::<Language>() == 1usize);
    assert!(align_of::<Language>() == 1usize);

    assert!(size_of::<ModuleKind>() == 1usize);
    assert!(align_of::<ModuleKind>() == 1usize);

    assert!(size_of::<LanguageVariant>() == 1usize);
    assert!(align_of::<LanguageVariant>() == 1usize);

    assert!(size_of::<RegularExpression>() == 72usize);
    assert!(align_of::<RegularExpression>() == 8usize);
    assert!(offset_of!(RegularExpression, span) == 0usize);
    assert!(offset_of!(RegularExpression, pattern) == 8usize);
    assert!(offset_of!(RegularExpression, flags) == 56usize);

    assert!(size_of::<Flags>() == 16usize);
    assert!(align_of::<Flags>() == 4usize);
    assert!(offset_of!(Flags, span) == 0usize);
    assert!(offset_of!(Flags, global) == 8usize);
    assert!(offset_of!(Flags, ignore_case) == 9usize);
    assert!(offset_of!(Flags, multiline) == 10usize);
    assert!(offset_of!(Flags, unicode) == 11usize);
    assert!(offset_of!(Flags, sticky) == 12usize);
    assert!(offset_of!(Flags, dot_all) == 13usize);
    assert!(offset_of!(Flags, has_indices) == 14usize);
    assert!(offset_of!(Flags, unicode_sets) == 15usize);

    assert!(size_of::<Pattern>() == 48usize);
    assert!(align_of::<Pattern>() == 8usize);
    assert!(offset_of!(Pattern, span) == 0usize);
    assert!(offset_of!(Pattern, body) == 8usize);

    assert!(size_of::<Disjunction>() == 40usize);
    assert!(align_of::<Disjunction>() == 8usize);
    assert!(offset_of!(Disjunction, span) == 0usize);
    assert!(offset_of!(Disjunction, body) == 8usize);

    assert!(size_of::<Alternative>() == 40usize);
    assert!(align_of::<Alternative>() == 8usize);
    assert!(offset_of!(Alternative, span) == 0usize);
    assert!(offset_of!(Alternative, body) == 8usize);

    assert!(size_of::<Term>() == 24usize);
    assert!(align_of::<Term>() == 8usize);

    assert!(size_of::<BoundaryAssertion>() == 12usize);
    assert!(align_of::<BoundaryAssertion>() == 4usize);
    assert!(offset_of!(BoundaryAssertion, span) == 0usize);
    assert!(offset_of!(BoundaryAssertion, kind) == 8usize);

    assert!(size_of::<BoundaryAssertionKind>() == 1usize);
    assert!(align_of::<BoundaryAssertionKind>() == 1usize);

    assert!(size_of::<LookAroundAssertion>() == 56usize);
    assert!(align_of::<LookAroundAssertion>() == 8usize);
    assert!(offset_of!(LookAroundAssertion, span) == 0usize);
    assert!(offset_of!(LookAroundAssertion, kind) == 8usize);
    assert!(offset_of!(LookAroundAssertion, body) == 16usize);

    assert!(size_of::<LookAroundAssertionKind>() == 1usize);
    assert!(align_of::<LookAroundAssertionKind>() == 1usize);

    assert!(size_of::<Quantifier>() == 64usize);
    assert!(align_of::<Quantifier>() == 8usize);
    assert!(offset_of!(Quantifier, span) == 0usize);
    assert!(offset_of!(Quantifier, min) == 8usize);
    assert!(offset_of!(Quantifier, max) == 16usize);
    assert!(offset_of!(Quantifier, greedy) == 32usize);
    assert!(offset_of!(Quantifier, body) == 40usize);

    assert!(size_of::<Character>() == 16usize);
    assert!(align_of::<Character>() == 4usize);
    assert!(offset_of!(Character, span) == 0usize);
    assert!(offset_of!(Character, kind) == 8usize);
    assert!(offset_of!(Character, value) == 12usize);

    assert!(size_of::<CharacterKind>() == 1usize);
    assert!(align_of::<CharacterKind>() == 1usize);

    assert!(size_of::<CharacterClassEscape>() == 12usize);
    assert!(align_of::<CharacterClassEscape>() == 4usize);
    assert!(offset_of!(CharacterClassEscape, span) == 0usize);
    assert!(offset_of!(CharacterClassEscape, kind) == 8usize);

    assert!(size_of::<CharacterClassEscapeKind>() == 1usize);
    assert!(align_of::<CharacterClassEscapeKind>() == 1usize);

    assert!(size_of::<UnicodePropertyEscape>() == 48usize);
    assert!(align_of::<UnicodePropertyEscape>() == 8usize);
    assert!(offset_of!(UnicodePropertyEscape, span) == 0usize);
    assert!(offset_of!(UnicodePropertyEscape, negative) == 8usize);
    assert!(offset_of!(UnicodePropertyEscape, strings) == 9usize);
    assert!(offset_of!(UnicodePropertyEscape, name) == 16usize);
    assert!(offset_of!(UnicodePropertyEscape, value) == 32usize);

    assert!(size_of::<Dot>() == 8usize);
    assert!(align_of::<Dot>() == 4usize);
    assert!(offset_of!(Dot, span) == 0usize);

    assert!(size_of::<CharacterClass>() == 48usize);
    assert!(align_of::<CharacterClass>() == 8usize);
    assert!(offset_of!(CharacterClass, span) == 0usize);
    assert!(offset_of!(CharacterClass, negative) == 8usize);
    assert!(offset_of!(CharacterClass, strings) == 9usize);
    assert!(offset_of!(CharacterClass, kind) == 10usize);
    assert!(offset_of!(CharacterClass, body) == 16usize);

    assert!(size_of::<CharacterClassContentsKind>() == 1usize);
    assert!(align_of::<CharacterClassContentsKind>() == 1usize);

    assert!(size_of::<CharacterClassContents>() == 24usize);
    assert!(align_of::<CharacterClassContents>() == 8usize);

    assert!(size_of::<CharacterClassRange>() == 40usize);
    assert!(align_of::<CharacterClassRange>() == 4usize);
    assert!(offset_of!(CharacterClassRange, span) == 0usize);
    assert!(offset_of!(CharacterClassRange, min) == 8usize);
    assert!(offset_of!(CharacterClassRange, max) == 24usize);

    assert!(size_of::<ClassStringDisjunction>() == 48usize);
    assert!(align_of::<ClassStringDisjunction>() == 8usize);
    assert!(offset_of!(ClassStringDisjunction, span) == 0usize);
    assert!(offset_of!(ClassStringDisjunction, strings) == 8usize);
    assert!(offset_of!(ClassStringDisjunction, body) == 16usize);

    assert!(size_of::<ClassString>() == 48usize);
    assert!(align_of::<ClassString>() == 8usize);
    assert!(offset_of!(ClassString, span) == 0usize);
    assert!(offset_of!(ClassString, strings) == 8usize);
    assert!(offset_of!(ClassString, body) == 16usize);

    assert!(size_of::<CapturingGroup>() == 64usize);
    assert!(align_of::<CapturingGroup>() == 8usize);
    assert!(offset_of!(CapturingGroup, span) == 0usize);
    assert!(offset_of!(CapturingGroup, name) == 8usize);
    assert!(offset_of!(CapturingGroup, body) == 24usize);

    assert!(size_of::<IgnoreGroup>() == 56usize);
    assert!(align_of::<IgnoreGroup>() == 8usize);
    assert!(offset_of!(IgnoreGroup, span) == 0usize);
    assert!(offset_of!(IgnoreGroup, enabling_modifiers) == 8usize);
    assert!(offset_of!(IgnoreGroup, disabling_modifiers) == 11usize);
    assert!(offset_of!(IgnoreGroup, body) == 16usize);

    assert!(size_of::<ModifierFlags>() == 3usize);
    assert!(align_of::<ModifierFlags>() == 1usize);
    assert!(offset_of!(ModifierFlags, ignore_case) == 0usize);
    assert!(offset_of!(ModifierFlags, sticky) == 1usize);
    assert!(offset_of!(ModifierFlags, multiline) == 2usize);

    assert!(size_of::<IndexedReference>() == 12usize);
    assert!(align_of::<IndexedReference>() == 4usize);
    assert!(offset_of!(IndexedReference, span) == 0usize);
    assert!(offset_of!(IndexedReference, index) == 8usize);

    assert!(size_of::<NamedReference>() == 24usize);
    assert!(align_of::<NamedReference>() == 8usize);
    assert!(offset_of!(NamedReference, span) == 0usize);
    assert!(offset_of!(NamedReference, name) == 8usize);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(size_of::<BooleanLiteral>() == 16usize);
    assert!(align_of::<BooleanLiteral>() == 4usize);
    assert!(offset_of!(BooleanLiteral, node_id) == 0usize);
    assert!(offset_of!(BooleanLiteral, span) == 4usize);
    assert!(offset_of!(BooleanLiteral, value) == 12usize);

    assert!(size_of::<NullLiteral>() == 12usize);
    assert!(align_of::<NullLiteral>() == 4usize);
    assert!(offset_of!(NullLiteral, node_id) == 0usize);
    assert!(offset_of!(NullLiteral, span) == 4usize);

    assert!(size_of::<NumericLiteral>() == 40usize);
    assert!(align_of::<NumericLiteral>() == 8usize);
    assert!(offset_of!(NumericLiteral, node_id) == 0usize);
    assert!(offset_of!(NumericLiteral, span) == 4usize);
    assert!(offset_of!(NumericLiteral, value) == 16usize);
    assert!(offset_of!(NumericLiteral, raw) == 24usize);
    assert!(offset_of!(NumericLiteral, base) == 32usize);

    assert!(size_of::<BigIntLiteral>() == 24usize);
    assert!(align_of::<BigIntLiteral>() == 4usize);
    assert!(offset_of!(BigIntLiteral, node_id) == 0usize);
    assert!(offset_of!(BigIntLiteral, span) == 4usize);
    assert!(offset_of!(BigIntLiteral, raw) == 12usize);
    assert!(offset_of!(BigIntLiteral, base) == 20usize);

    assert!(size_of::<RegExpLiteral>() == 28usize);
    assert!(align_of::<RegExpLiteral>() == 4usize);
    assert!(offset_of!(RegExpLiteral, node_id) == 0usize);
    assert!(offset_of!(RegExpLiteral, span) == 4usize);
    assert!(offset_of!(RegExpLiteral, value) == 12usize);
    assert!(offset_of!(RegExpLiteral, regex) == 12usize);

    assert!(size_of::<RegExp>() == 16usize);
    assert!(align_of::<RegExp>() == 4usize);
    assert!(offset_of!(RegExp, pattern) == 0usize);
    assert!(offset_of!(RegExp, flags) == 12usize);

    assert!(size_of::<RegExpPattern>() == 12usize);
    assert!(align_of::<RegExpPattern>() == 4usize);

    assert!(size_of::<EmptyObject>() == 0usize);
    assert!(align_of::<EmptyObject>() == 1usize);

    assert!(size_of::<StringLiteral>() == 20usize);
    assert!(align_of::<StringLiteral>() == 4usize);
    assert!(offset_of!(StringLiteral, node_id) == 0usize);
    assert!(offset_of!(StringLiteral, span) == 4usize);
    assert!(offset_of!(StringLiteral, value) == 12usize);

    assert!(size_of::<Program>() == 72usize);
    assert!(align_of::<Program>() == 4usize);
    assert!(offset_of!(Program, node_id) == 0usize);
    assert!(offset_of!(Program, span) == 4usize);
    assert!(offset_of!(Program, source_type) == 12usize);
    assert!(offset_of!(Program, hashbang) == 16usize);
    assert!(offset_of!(Program, directives) == 36usize);
    assert!(offset_of!(Program, body) == 52usize);
    assert!(offset_of!(Program, scope_id) == 68usize);

    assert!(size_of::<Expression>() == 8usize);
    assert!(align_of::<Expression>() == 4usize);

    assert!(size_of::<IdentifierName>() == 20usize);
    assert!(align_of::<IdentifierName>() == 4usize);
    assert!(offset_of!(IdentifierName, node_id) == 0usize);
    assert!(offset_of!(IdentifierName, span) == 4usize);
    assert!(offset_of!(IdentifierName, name) == 12usize);

    assert!(size_of::<IdentifierReference>() == 24usize);
    assert!(align_of::<IdentifierReference>() == 4usize);
    assert!(offset_of!(IdentifierReference, node_id) == 0usize);
    assert!(offset_of!(IdentifierReference, span) == 4usize);
    assert!(offset_of!(IdentifierReference, name) == 12usize);
    assert!(offset_of!(IdentifierReference, reference_id) == 20usize);

    assert!(size_of::<BindingIdentifier>() == 24usize);
    assert!(align_of::<BindingIdentifier>() == 4usize);
    assert!(offset_of!(BindingIdentifier, node_id) == 0usize);
    assert!(offset_of!(BindingIdentifier, span) == 4usize);
    assert!(offset_of!(BindingIdentifier, name) == 12usize);
    assert!(offset_of!(BindingIdentifier, symbol_id) == 20usize);

    assert!(size_of::<LabelIdentifier>() == 20usize);
    assert!(align_of::<LabelIdentifier>() == 4usize);
    assert!(offset_of!(LabelIdentifier, node_id) == 0usize);
    assert!(offset_of!(LabelIdentifier, span) == 4usize);
    assert!(offset_of!(LabelIdentifier, name) == 12usize);

    assert!(size_of::<ThisExpression>() == 12usize);
    assert!(align_of::<ThisExpression>() == 4usize);
    assert!(offset_of!(ThisExpression, node_id) == 0usize);
    assert!(offset_of!(ThisExpression, span) == 4usize);

    assert!(size_of::<ArrayExpression>() == 40usize);
    assert!(align_of::<ArrayExpression>() == 4usize);
    assert!(offset_of!(ArrayExpression, node_id) == 0usize);
    assert!(offset_of!(ArrayExpression, span) == 4usize);
    assert!(offset_of!(ArrayExpression, elements) == 12usize);
    assert!(offset_of!(ArrayExpression, trailing_comma) == 28usize);

    assert!(size_of::<ArrayExpressionElement>() == 16usize);
    assert!(align_of::<ArrayExpressionElement>() == 4usize);

    assert!(size_of::<Elision>() == 12usize);
    assert!(align_of::<Elision>() == 4usize);
    assert!(offset_of!(Elision, node_id) == 0usize);
    assert!(offset_of!(Elision, span) == 4usize);

    assert!(size_of::<ObjectExpression>() == 40usize);
    assert!(align_of::<ObjectExpression>() == 4usize);
    assert!(offset_of!(ObjectExpression, node_id) == 0usize);
    assert!(offset_of!(ObjectExpression, span) == 4usize);
    assert!(offset_of!(ObjectExpression, properties) == 12usize);
    assert!(offset_of!(ObjectExpression, trailing_comma) == 28usize);

    assert!(size_of::<ObjectPropertyKind>() == 8usize);
    assert!(align_of::<ObjectPropertyKind>() == 4usize);

    assert!(size_of::<ObjectProperty>() == 44usize);
    assert!(align_of::<ObjectProperty>() == 4usize);
    assert!(offset_of!(ObjectProperty, node_id) == 0usize);
    assert!(offset_of!(ObjectProperty, span) == 4usize);
    assert!(offset_of!(ObjectProperty, kind) == 12usize);
    assert!(offset_of!(ObjectProperty, key) == 16usize);
    assert!(offset_of!(ObjectProperty, value) == 24usize);
    assert!(offset_of!(ObjectProperty, init) == 32usize);
    assert!(offset_of!(ObjectProperty, method) == 40usize);
    assert!(offset_of!(ObjectProperty, shorthand) == 41usize);
    assert!(offset_of!(ObjectProperty, computed) == 42usize);

    assert!(size_of::<PropertyKey>() == 8usize);
    assert!(align_of::<PropertyKey>() == 4usize);

    assert!(size_of::<PropertyKind>() == 1usize);
    assert!(align_of::<PropertyKind>() == 1usize);

    assert!(size_of::<TemplateLiteral>() == 44usize);
    assert!(align_of::<TemplateLiteral>() == 4usize);
    assert!(offset_of!(TemplateLiteral, node_id) == 0usize);
    assert!(offset_of!(TemplateLiteral, span) == 4usize);
    assert!(offset_of!(TemplateLiteral, quasis) == 12usize);
    assert!(offset_of!(TemplateLiteral, expressions) == 28usize);

    assert!(size_of::<TaggedTemplateExpression>() == 68usize);
    assert!(align_of::<TaggedTemplateExpression>() == 4usize);
    assert!(offset_of!(TaggedTemplateExpression, node_id) == 0usize);
    assert!(offset_of!(TaggedTemplateExpression, span) == 4usize);
    assert!(offset_of!(TaggedTemplateExpression, tag) == 12usize);
    assert!(offset_of!(TaggedTemplateExpression, quasi) == 20usize);
    assert!(offset_of!(TaggedTemplateExpression, type_parameters) == 64usize);

    assert!(size_of::<TemplateElement>() == 32usize);
    assert!(align_of::<TemplateElement>() == 4usize);
    assert!(offset_of!(TemplateElement, node_id) == 0usize);
    assert!(offset_of!(TemplateElement, span) == 4usize);
    assert!(offset_of!(TemplateElement, tail) == 12usize);
    assert!(offset_of!(TemplateElement, value) == 16usize);

    assert!(size_of::<TemplateElementValue>() == 16usize);
    assert!(align_of::<TemplateElementValue>() == 4usize);
    assert!(offset_of!(TemplateElementValue, raw) == 0usize);
    assert!(offset_of!(TemplateElementValue, cooked) == 8usize);

    assert!(size_of::<MemberExpression>() == 8usize);
    assert!(align_of::<MemberExpression>() == 4usize);

    assert!(size_of::<ComputedMemberExpression>() == 32usize);
    assert!(align_of::<ComputedMemberExpression>() == 4usize);
    assert!(offset_of!(ComputedMemberExpression, node_id) == 0usize);
    assert!(offset_of!(ComputedMemberExpression, span) == 4usize);
    assert!(offset_of!(ComputedMemberExpression, object) == 12usize);
    assert!(offset_of!(ComputedMemberExpression, expression) == 20usize);
    assert!(offset_of!(ComputedMemberExpression, optional) == 28usize);

    assert!(size_of::<StaticMemberExpression>() == 44usize);
    assert!(align_of::<StaticMemberExpression>() == 4usize);
    assert!(offset_of!(StaticMemberExpression, node_id) == 0usize);
    assert!(offset_of!(StaticMemberExpression, span) == 4usize);
    assert!(offset_of!(StaticMemberExpression, object) == 12usize);
    assert!(offset_of!(StaticMemberExpression, property) == 20usize);
    assert!(offset_of!(StaticMemberExpression, optional) == 40usize);

    assert!(size_of::<PrivateFieldExpression>() == 44usize);
    assert!(align_of::<PrivateFieldExpression>() == 4usize);
    assert!(offset_of!(PrivateFieldExpression, node_id) == 0usize);
    assert!(offset_of!(PrivateFieldExpression, span) == 4usize);
    assert!(offset_of!(PrivateFieldExpression, object) == 12usize);
    assert!(offset_of!(PrivateFieldExpression, field) == 20usize);
    assert!(offset_of!(PrivateFieldExpression, optional) == 40usize);

    assert!(size_of::<CallExpression>() == 44usize);
    assert!(align_of::<CallExpression>() == 4usize);
    assert!(offset_of!(CallExpression, node_id) == 0usize);
    assert!(offset_of!(CallExpression, span) == 4usize);
    assert!(offset_of!(CallExpression, callee) == 12usize);
    assert!(offset_of!(CallExpression, type_parameters) == 20usize);
    assert!(offset_of!(CallExpression, arguments) == 24usize);
    assert!(offset_of!(CallExpression, optional) == 40usize);

    assert!(size_of::<NewExpression>() == 40usize);
    assert!(align_of::<NewExpression>() == 4usize);
    assert!(offset_of!(NewExpression, node_id) == 0usize);
    assert!(offset_of!(NewExpression, span) == 4usize);
    assert!(offset_of!(NewExpression, callee) == 12usize);
    assert!(offset_of!(NewExpression, arguments) == 20usize);
    assert!(offset_of!(NewExpression, type_parameters) == 36usize);

    assert!(size_of::<MetaProperty>() == 52usize);
    assert!(align_of::<MetaProperty>() == 4usize);
    assert!(offset_of!(MetaProperty, node_id) == 0usize);
    assert!(offset_of!(MetaProperty, span) == 4usize);
    assert!(offset_of!(MetaProperty, meta) == 12usize);
    assert!(offset_of!(MetaProperty, property) == 32usize);

    assert!(size_of::<SpreadElement>() == 20usize);
    assert!(align_of::<SpreadElement>() == 4usize);
    assert!(offset_of!(SpreadElement, node_id) == 0usize);
    assert!(offset_of!(SpreadElement, span) == 4usize);
    assert!(offset_of!(SpreadElement, argument) == 12usize);

    assert!(size_of::<Argument>() == 8usize);
    assert!(align_of::<Argument>() == 4usize);

    assert!(size_of::<UpdateExpression>() == 24usize);
    assert!(align_of::<UpdateExpression>() == 4usize);
    assert!(offset_of!(UpdateExpression, node_id) == 0usize);
    assert!(offset_of!(UpdateExpression, span) == 4usize);
    assert!(offset_of!(UpdateExpression, operator) == 12usize);
    assert!(offset_of!(UpdateExpression, prefix) == 13usize);
    assert!(offset_of!(UpdateExpression, argument) == 16usize);

    assert!(size_of::<UnaryExpression>() == 24usize);
    assert!(align_of::<UnaryExpression>() == 4usize);
    assert!(offset_of!(UnaryExpression, node_id) == 0usize);
    assert!(offset_of!(UnaryExpression, span) == 4usize);
    assert!(offset_of!(UnaryExpression, operator) == 12usize);
    assert!(offset_of!(UnaryExpression, argument) == 16usize);

    assert!(size_of::<BinaryExpression>() == 32usize);
    assert!(align_of::<BinaryExpression>() == 4usize);
    assert!(offset_of!(BinaryExpression, node_id) == 0usize);
    assert!(offset_of!(BinaryExpression, span) == 4usize);
    assert!(offset_of!(BinaryExpression, left) == 12usize);
    assert!(offset_of!(BinaryExpression, operator) == 20usize);
    assert!(offset_of!(BinaryExpression, right) == 24usize);

    assert!(size_of::<PrivateInExpression>() == 44usize);
    assert!(align_of::<PrivateInExpression>() == 4usize);
    assert!(offset_of!(PrivateInExpression, node_id) == 0usize);
    assert!(offset_of!(PrivateInExpression, span) == 4usize);
    assert!(offset_of!(PrivateInExpression, left) == 12usize);
    assert!(offset_of!(PrivateInExpression, operator) == 32usize);
    assert!(offset_of!(PrivateInExpression, right) == 36usize);

    assert!(size_of::<LogicalExpression>() == 32usize);
    assert!(align_of::<LogicalExpression>() == 4usize);
    assert!(offset_of!(LogicalExpression, node_id) == 0usize);
    assert!(offset_of!(LogicalExpression, span) == 4usize);
    assert!(offset_of!(LogicalExpression, left) == 12usize);
    assert!(offset_of!(LogicalExpression, operator) == 20usize);
    assert!(offset_of!(LogicalExpression, right) == 24usize);

    assert!(size_of::<ConditionalExpression>() == 36usize);
    assert!(align_of::<ConditionalExpression>() == 4usize);
    assert!(offset_of!(ConditionalExpression, node_id) == 0usize);
    assert!(offset_of!(ConditionalExpression, span) == 4usize);
    assert!(offset_of!(ConditionalExpression, test) == 12usize);
    assert!(offset_of!(ConditionalExpression, consequent) == 20usize);
    assert!(offset_of!(ConditionalExpression, alternate) == 28usize);

    assert!(size_of::<AssignmentExpression>() == 32usize);
    assert!(align_of::<AssignmentExpression>() == 4usize);
    assert!(offset_of!(AssignmentExpression, node_id) == 0usize);
    assert!(offset_of!(AssignmentExpression, span) == 4usize);
    assert!(offset_of!(AssignmentExpression, operator) == 12usize);
    assert!(offset_of!(AssignmentExpression, left) == 16usize);
    assert!(offset_of!(AssignmentExpression, right) == 24usize);

    assert!(size_of::<AssignmentTarget>() == 8usize);
    assert!(align_of::<AssignmentTarget>() == 4usize);

    assert!(size_of::<SimpleAssignmentTarget>() == 8usize);
    assert!(align_of::<SimpleAssignmentTarget>() == 4usize);

    assert!(size_of::<AssignmentTargetPattern>() == 8usize);
    assert!(align_of::<AssignmentTargetPattern>() == 4usize);

    assert!(size_of::<ArrayAssignmentTarget>() == 60usize);
    assert!(align_of::<ArrayAssignmentTarget>() == 4usize);
    assert!(offset_of!(ArrayAssignmentTarget, node_id) == 0usize);
    assert!(offset_of!(ArrayAssignmentTarget, span) == 4usize);
    assert!(offset_of!(ArrayAssignmentTarget, elements) == 12usize);
    assert!(offset_of!(ArrayAssignmentTarget, rest) == 28usize);
    assert!(offset_of!(ArrayAssignmentTarget, trailing_comma) == 48usize);

    assert!(size_of::<ObjectAssignmentTarget>() == 48usize);
    assert!(align_of::<ObjectAssignmentTarget>() == 4usize);
    assert!(offset_of!(ObjectAssignmentTarget, node_id) == 0usize);
    assert!(offset_of!(ObjectAssignmentTarget, span) == 4usize);
    assert!(offset_of!(ObjectAssignmentTarget, properties) == 12usize);
    assert!(offset_of!(ObjectAssignmentTarget, rest) == 28usize);

    assert!(size_of::<AssignmentTargetRest>() == 20usize);
    assert!(align_of::<AssignmentTargetRest>() == 4usize);
    assert!(offset_of!(AssignmentTargetRest, node_id) == 0usize);
    assert!(offset_of!(AssignmentTargetRest, span) == 4usize);
    assert!(offset_of!(AssignmentTargetRest, target) == 12usize);

    assert!(size_of::<AssignmentTargetMaybeDefault>() == 8usize);
    assert!(align_of::<AssignmentTargetMaybeDefault>() == 4usize);

    assert!(size_of::<AssignmentTargetWithDefault>() == 28usize);
    assert!(align_of::<AssignmentTargetWithDefault>() == 4usize);
    assert!(offset_of!(AssignmentTargetWithDefault, node_id) == 0usize);
    assert!(offset_of!(AssignmentTargetWithDefault, span) == 4usize);
    assert!(offset_of!(AssignmentTargetWithDefault, binding) == 12usize);
    assert!(offset_of!(AssignmentTargetWithDefault, init) == 20usize);

    assert!(size_of::<AssignmentTargetProperty>() == 8usize);
    assert!(align_of::<AssignmentTargetProperty>() == 4usize);

    assert!(size_of::<AssignmentTargetPropertyIdentifier>() == 44usize);
    assert!(align_of::<AssignmentTargetPropertyIdentifier>() == 4usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, node_id) == 0usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, span) == 4usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, binding) == 12usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, init) == 36usize);

    assert!(size_of::<AssignmentTargetPropertyProperty>() == 28usize);
    assert!(align_of::<AssignmentTargetPropertyProperty>() == 4usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, node_id) == 0usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, span) == 4usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, name) == 12usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, binding) == 20usize);

    assert!(size_of::<SequenceExpression>() == 28usize);
    assert!(align_of::<SequenceExpression>() == 4usize);
    assert!(offset_of!(SequenceExpression, node_id) == 0usize);
    assert!(offset_of!(SequenceExpression, span) == 4usize);
    assert!(offset_of!(SequenceExpression, expressions) == 12usize);

    assert!(size_of::<Super>() == 12usize);
    assert!(align_of::<Super>() == 4usize);
    assert!(offset_of!(Super, node_id) == 0usize);
    assert!(offset_of!(Super, span) == 4usize);

    assert!(size_of::<AwaitExpression>() == 20usize);
    assert!(align_of::<AwaitExpression>() == 4usize);
    assert!(offset_of!(AwaitExpression, node_id) == 0usize);
    assert!(offset_of!(AwaitExpression, span) == 4usize);
    assert!(offset_of!(AwaitExpression, argument) == 12usize);

    assert!(size_of::<ChainExpression>() == 20usize);
    assert!(align_of::<ChainExpression>() == 4usize);
    assert!(offset_of!(ChainExpression, node_id) == 0usize);
    assert!(offset_of!(ChainExpression, span) == 4usize);
    assert!(offset_of!(ChainExpression, expression) == 12usize);

    assert!(size_of::<ChainElement>() == 8usize);
    assert!(align_of::<ChainElement>() == 4usize);

    assert!(size_of::<ParenthesizedExpression>() == 20usize);
    assert!(align_of::<ParenthesizedExpression>() == 4usize);
    assert!(offset_of!(ParenthesizedExpression, node_id) == 0usize);
    assert!(offset_of!(ParenthesizedExpression, span) == 4usize);
    assert!(offset_of!(ParenthesizedExpression, expression) == 12usize);

    assert!(size_of::<Statement>() == 8usize);
    assert!(align_of::<Statement>() == 4usize);

    assert!(size_of::<Directive>() == 40usize);
    assert!(align_of::<Directive>() == 4usize);
    assert!(offset_of!(Directive, node_id) == 0usize);
    assert!(offset_of!(Directive, span) == 4usize);
    assert!(offset_of!(Directive, expression) == 12usize);
    assert!(offset_of!(Directive, directive) == 32usize);

    assert!(size_of::<Hashbang>() == 20usize);
    assert!(align_of::<Hashbang>() == 4usize);
    assert!(offset_of!(Hashbang, node_id) == 0usize);
    assert!(offset_of!(Hashbang, span) == 4usize);
    assert!(offset_of!(Hashbang, value) == 12usize);

    assert!(size_of::<BlockStatement>() == 32usize);
    assert!(align_of::<BlockStatement>() == 4usize);
    assert!(offset_of!(BlockStatement, node_id) == 0usize);
    assert!(offset_of!(BlockStatement, span) == 4usize);
    assert!(offset_of!(BlockStatement, body) == 12usize);
    assert!(offset_of!(BlockStatement, scope_id) == 28usize);

    assert!(size_of::<Declaration>() == 8usize);
    assert!(align_of::<Declaration>() == 4usize);

    assert!(size_of::<VariableDeclaration>() == 36usize);
    assert!(align_of::<VariableDeclaration>() == 4usize);
    assert!(offset_of!(VariableDeclaration, node_id) == 0usize);
    assert!(offset_of!(VariableDeclaration, span) == 4usize);
    assert!(offset_of!(VariableDeclaration, kind) == 12usize);
    assert!(offset_of!(VariableDeclaration, declarations) == 16usize);
    assert!(offset_of!(VariableDeclaration, declare) == 32usize);

    assert!(size_of::<VariableDeclarationKind>() == 1usize);
    assert!(align_of::<VariableDeclarationKind>() == 1usize);

    assert!(size_of::<VariableDeclarator>() == 48usize);
    assert!(align_of::<VariableDeclarator>() == 4usize);
    assert!(offset_of!(VariableDeclarator, node_id) == 0usize);
    assert!(offset_of!(VariableDeclarator, span) == 4usize);
    assert!(offset_of!(VariableDeclarator, kind) == 12usize);
    assert!(offset_of!(VariableDeclarator, id) == 16usize);
    assert!(offset_of!(VariableDeclarator, init) == 36usize);
    assert!(offset_of!(VariableDeclarator, definite) == 44usize);

    assert!(size_of::<EmptyStatement>() == 12usize);
    assert!(align_of::<EmptyStatement>() == 4usize);
    assert!(offset_of!(EmptyStatement, node_id) == 0usize);
    assert!(offset_of!(EmptyStatement, span) == 4usize);

    assert!(size_of::<ExpressionStatement>() == 20usize);
    assert!(align_of::<ExpressionStatement>() == 4usize);
    assert!(offset_of!(ExpressionStatement, node_id) == 0usize);
    assert!(offset_of!(ExpressionStatement, span) == 4usize);
    assert!(offset_of!(ExpressionStatement, expression) == 12usize);

    assert!(size_of::<IfStatement>() == 36usize);
    assert!(align_of::<IfStatement>() == 4usize);
    assert!(offset_of!(IfStatement, node_id) == 0usize);
    assert!(offset_of!(IfStatement, span) == 4usize);
    assert!(offset_of!(IfStatement, test) == 12usize);
    assert!(offset_of!(IfStatement, consequent) == 20usize);
    assert!(offset_of!(IfStatement, alternate) == 28usize);

    assert!(size_of::<DoWhileStatement>() == 28usize);
    assert!(align_of::<DoWhileStatement>() == 4usize);
    assert!(offset_of!(DoWhileStatement, node_id) == 0usize);
    assert!(offset_of!(DoWhileStatement, span) == 4usize);
    assert!(offset_of!(DoWhileStatement, body) == 12usize);
    assert!(offset_of!(DoWhileStatement, test) == 20usize);

    assert!(size_of::<WhileStatement>() == 28usize);
    assert!(align_of::<WhileStatement>() == 4usize);
    assert!(offset_of!(WhileStatement, node_id) == 0usize);
    assert!(offset_of!(WhileStatement, span) == 4usize);
    assert!(offset_of!(WhileStatement, test) == 12usize);
    assert!(offset_of!(WhileStatement, body) == 20usize);

    assert!(size_of::<ForStatement>() == 48usize);
    assert!(align_of::<ForStatement>() == 4usize);
    assert!(offset_of!(ForStatement, node_id) == 0usize);
    assert!(offset_of!(ForStatement, span) == 4usize);
    assert!(offset_of!(ForStatement, init) == 12usize);
    assert!(offset_of!(ForStatement, test) == 20usize);
    assert!(offset_of!(ForStatement, update) == 28usize);
    assert!(offset_of!(ForStatement, body) == 36usize);
    assert!(offset_of!(ForStatement, scope_id) == 44usize);

    assert!(size_of::<ForStatementInit>() == 8usize);
    assert!(align_of::<ForStatementInit>() == 4usize);

    assert!(size_of::<ForInStatement>() == 40usize);
    assert!(align_of::<ForInStatement>() == 4usize);
    assert!(offset_of!(ForInStatement, node_id) == 0usize);
    assert!(offset_of!(ForInStatement, span) == 4usize);
    assert!(offset_of!(ForInStatement, left) == 12usize);
    assert!(offset_of!(ForInStatement, right) == 20usize);
    assert!(offset_of!(ForInStatement, body) == 28usize);
    assert!(offset_of!(ForInStatement, scope_id) == 36usize);

    assert!(size_of::<ForStatementLeft>() == 8usize);
    assert!(align_of::<ForStatementLeft>() == 4usize);

    assert!(size_of::<ForOfStatement>() == 44usize);
    assert!(align_of::<ForOfStatement>() == 4usize);
    assert!(offset_of!(ForOfStatement, node_id) == 0usize);
    assert!(offset_of!(ForOfStatement, span) == 4usize);
    assert!(offset_of!(ForOfStatement, r#await) == 12usize);
    assert!(offset_of!(ForOfStatement, left) == 16usize);
    assert!(offset_of!(ForOfStatement, right) == 24usize);
    assert!(offset_of!(ForOfStatement, body) == 32usize);
    assert!(offset_of!(ForOfStatement, scope_id) == 40usize);

    assert!(size_of::<ContinueStatement>() == 32usize);
    assert!(align_of::<ContinueStatement>() == 4usize);
    assert!(offset_of!(ContinueStatement, node_id) == 0usize);
    assert!(offset_of!(ContinueStatement, span) == 4usize);
    assert!(offset_of!(ContinueStatement, label) == 12usize);

    assert!(size_of::<BreakStatement>() == 32usize);
    assert!(align_of::<BreakStatement>() == 4usize);
    assert!(offset_of!(BreakStatement, node_id) == 0usize);
    assert!(offset_of!(BreakStatement, span) == 4usize);
    assert!(offset_of!(BreakStatement, label) == 12usize);

    assert!(size_of::<ReturnStatement>() == 20usize);
    assert!(align_of::<ReturnStatement>() == 4usize);
    assert!(offset_of!(ReturnStatement, node_id) == 0usize);
    assert!(offset_of!(ReturnStatement, span) == 4usize);
    assert!(offset_of!(ReturnStatement, argument) == 12usize);

    assert!(size_of::<WithStatement>() == 28usize);
    assert!(align_of::<WithStatement>() == 4usize);
    assert!(offset_of!(WithStatement, node_id) == 0usize);
    assert!(offset_of!(WithStatement, span) == 4usize);
    assert!(offset_of!(WithStatement, object) == 12usize);
    assert!(offset_of!(WithStatement, body) == 20usize);

    assert!(size_of::<SwitchStatement>() == 40usize);
    assert!(align_of::<SwitchStatement>() == 4usize);
    assert!(offset_of!(SwitchStatement, node_id) == 0usize);
    assert!(offset_of!(SwitchStatement, span) == 4usize);
    assert!(offset_of!(SwitchStatement, discriminant) == 12usize);
    assert!(offset_of!(SwitchStatement, cases) == 20usize);
    assert!(offset_of!(SwitchStatement, scope_id) == 36usize);

    assert!(size_of::<SwitchCase>() == 36usize);
    assert!(align_of::<SwitchCase>() == 4usize);
    assert!(offset_of!(SwitchCase, node_id) == 0usize);
    assert!(offset_of!(SwitchCase, span) == 4usize);
    assert!(offset_of!(SwitchCase, test) == 12usize);
    assert!(offset_of!(SwitchCase, consequent) == 20usize);

    assert!(size_of::<LabeledStatement>() == 40usize);
    assert!(align_of::<LabeledStatement>() == 4usize);
    assert!(offset_of!(LabeledStatement, node_id) == 0usize);
    assert!(offset_of!(LabeledStatement, span) == 4usize);
    assert!(offset_of!(LabeledStatement, label) == 12usize);
    assert!(offset_of!(LabeledStatement, body) == 32usize);

    assert!(size_of::<ThrowStatement>() == 20usize);
    assert!(align_of::<ThrowStatement>() == 4usize);
    assert!(offset_of!(ThrowStatement, node_id) == 0usize);
    assert!(offset_of!(ThrowStatement, span) == 4usize);
    assert!(offset_of!(ThrowStatement, argument) == 12usize);

    assert!(size_of::<TryStatement>() == 24usize);
    assert!(align_of::<TryStatement>() == 4usize);
    assert!(offset_of!(TryStatement, node_id) == 0usize);
    assert!(offset_of!(TryStatement, span) == 4usize);
    assert!(offset_of!(TryStatement, block) == 12usize);
    assert!(offset_of!(TryStatement, handler) == 16usize);
    assert!(offset_of!(TryStatement, finalizer) == 20usize);

    assert!(size_of::<CatchClause>() == 52usize);
    assert!(align_of::<CatchClause>() == 4usize);
    assert!(offset_of!(CatchClause, node_id) == 0usize);
    assert!(offset_of!(CatchClause, span) == 4usize);
    assert!(offset_of!(CatchClause, param) == 12usize);
    assert!(offset_of!(CatchClause, body) == 44usize);
    assert!(offset_of!(CatchClause, scope_id) == 48usize);

    assert!(size_of::<CatchParameter>() == 32usize);
    assert!(align_of::<CatchParameter>() == 4usize);
    assert!(offset_of!(CatchParameter, node_id) == 0usize);
    assert!(offset_of!(CatchParameter, span) == 4usize);
    assert!(offset_of!(CatchParameter, pattern) == 12usize);

    assert!(size_of::<DebuggerStatement>() == 12usize);
    assert!(align_of::<DebuggerStatement>() == 4usize);
    assert!(offset_of!(DebuggerStatement, node_id) == 0usize);
    assert!(offset_of!(DebuggerStatement, span) == 4usize);

    assert!(size_of::<BindingPattern>() == 20usize);
    assert!(align_of::<BindingPattern>() == 4usize);
    assert!(offset_of!(BindingPattern, node_id) == 0usize);
    assert!(offset_of!(BindingPattern, kind) == 4usize);
    assert!(offset_of!(BindingPattern, type_annotation) == 12usize);
    assert!(offset_of!(BindingPattern, optional) == 16usize);

    assert!(size_of::<BindingPatternKind>() == 8usize);
    assert!(align_of::<BindingPatternKind>() == 4usize);

    assert!(size_of::<AssignmentPattern>() == 40usize);
    assert!(align_of::<AssignmentPattern>() == 4usize);
    assert!(offset_of!(AssignmentPattern, node_id) == 0usize);
    assert!(offset_of!(AssignmentPattern, span) == 4usize);
    assert!(offset_of!(AssignmentPattern, left) == 12usize);
    assert!(offset_of!(AssignmentPattern, right) == 32usize);

    assert!(size_of::<ObjectPattern>() == 32usize);
    assert!(align_of::<ObjectPattern>() == 4usize);
    assert!(offset_of!(ObjectPattern, node_id) == 0usize);
    assert!(offset_of!(ObjectPattern, span) == 4usize);
    assert!(offset_of!(ObjectPattern, properties) == 12usize);
    assert!(offset_of!(ObjectPattern, rest) == 28usize);

    assert!(size_of::<BindingProperty>() == 44usize);
    assert!(align_of::<BindingProperty>() == 4usize);
    assert!(offset_of!(BindingProperty, node_id) == 0usize);
    assert!(offset_of!(BindingProperty, span) == 4usize);
    assert!(offset_of!(BindingProperty, key) == 12usize);
    assert!(offset_of!(BindingProperty, value) == 20usize);
    assert!(offset_of!(BindingProperty, shorthand) == 40usize);
    assert!(offset_of!(BindingProperty, computed) == 41usize);

    assert!(size_of::<ArrayPattern>() == 32usize);
    assert!(align_of::<ArrayPattern>() == 4usize);
    assert!(offset_of!(ArrayPattern, node_id) == 0usize);
    assert!(offset_of!(ArrayPattern, span) == 4usize);
    assert!(offset_of!(ArrayPattern, elements) == 12usize);
    assert!(offset_of!(ArrayPattern, rest) == 28usize);

    assert!(size_of::<BindingRestElement>() == 32usize);
    assert!(align_of::<BindingRestElement>() == 4usize);
    assert!(offset_of!(BindingRestElement, node_id) == 0usize);
    assert!(offset_of!(BindingRestElement, span) == 4usize);
    assert!(offset_of!(BindingRestElement, argument) == 12usize);

    assert!(size_of::<Function>() == 68usize);
    assert!(align_of::<Function>() == 4usize);
    assert!(offset_of!(Function, node_id) == 0usize);
    assert!(offset_of!(Function, r#type) == 4usize);
    assert!(offset_of!(Function, span) == 8usize);
    assert!(offset_of!(Function, id) == 16usize);
    assert!(offset_of!(Function, generator) == 40usize);
    assert!(offset_of!(Function, r#async) == 41usize);
    assert!(offset_of!(Function, declare) == 42usize);
    assert!(offset_of!(Function, type_parameters) == 44usize);
    assert!(offset_of!(Function, this_param) == 48usize);
    assert!(offset_of!(Function, params) == 52usize);
    assert!(offset_of!(Function, return_type) == 56usize);
    assert!(offset_of!(Function, body) == 60usize);
    assert!(offset_of!(Function, scope_id) == 64usize);

    assert!(size_of::<FunctionType>() == 1usize);
    assert!(align_of::<FunctionType>() == 1usize);

    assert!(size_of::<FormalParameters>() == 36usize);
    assert!(align_of::<FormalParameters>() == 4usize);
    assert!(offset_of!(FormalParameters, node_id) == 0usize);
    assert!(offset_of!(FormalParameters, span) == 4usize);
    assert!(offset_of!(FormalParameters, kind) == 12usize);
    assert!(offset_of!(FormalParameters, items) == 16usize);
    assert!(offset_of!(FormalParameters, rest) == 32usize);

    assert!(size_of::<FormalParameter>() == 52usize);
    assert!(align_of::<FormalParameter>() == 4usize);
    assert!(offset_of!(FormalParameter, node_id) == 0usize);
    assert!(offset_of!(FormalParameter, span) == 4usize);
    assert!(offset_of!(FormalParameter, decorators) == 12usize);
    assert!(offset_of!(FormalParameter, pattern) == 28usize);
    assert!(offset_of!(FormalParameter, accessibility) == 48usize);
    assert!(offset_of!(FormalParameter, readonly) == 49usize);
    assert!(offset_of!(FormalParameter, r#override) == 50usize);

    assert!(size_of::<FormalParameterKind>() == 1usize);
    assert!(align_of::<FormalParameterKind>() == 1usize);

    assert!(size_of::<FunctionBody>() == 44usize);
    assert!(align_of::<FunctionBody>() == 4usize);
    assert!(offset_of!(FunctionBody, node_id) == 0usize);
    assert!(offset_of!(FunctionBody, span) == 4usize);
    assert!(offset_of!(FunctionBody, directives) == 12usize);
    assert!(offset_of!(FunctionBody, statements) == 28usize);

    assert!(size_of::<ArrowFunctionExpression>() == 36usize);
    assert!(align_of::<ArrowFunctionExpression>() == 4usize);
    assert!(offset_of!(ArrowFunctionExpression, node_id) == 0usize);
    assert!(offset_of!(ArrowFunctionExpression, span) == 4usize);
    assert!(offset_of!(ArrowFunctionExpression, expression) == 12usize);
    assert!(offset_of!(ArrowFunctionExpression, r#async) == 13usize);
    assert!(offset_of!(ArrowFunctionExpression, type_parameters) == 16usize);
    assert!(offset_of!(ArrowFunctionExpression, params) == 20usize);
    assert!(offset_of!(ArrowFunctionExpression, return_type) == 24usize);
    assert!(offset_of!(ArrowFunctionExpression, body) == 28usize);
    assert!(offset_of!(ArrowFunctionExpression, scope_id) == 32usize);

    assert!(size_of::<YieldExpression>() == 24usize);
    assert!(align_of::<YieldExpression>() == 4usize);
    assert!(offset_of!(YieldExpression, node_id) == 0usize);
    assert!(offset_of!(YieldExpression, span) == 4usize);
    assert!(offset_of!(YieldExpression, delegate) == 12usize);
    assert!(offset_of!(YieldExpression, argument) == 16usize);

    assert!(size_of::<Class>() == 100usize);
    assert!(align_of::<Class>() == 4usize);
    assert!(offset_of!(Class, node_id) == 0usize);
    assert!(offset_of!(Class, r#type) == 4usize);
    assert!(offset_of!(Class, span) == 8usize);
    assert!(offset_of!(Class, decorators) == 16usize);
    assert!(offset_of!(Class, id) == 32usize);
    assert!(offset_of!(Class, type_parameters) == 56usize);
    assert!(offset_of!(Class, super_class) == 60usize);
    assert!(offset_of!(Class, super_type_parameters) == 68usize);
    assert!(offset_of!(Class, implements) == 72usize);
    assert!(offset_of!(Class, body) == 88usize);
    assert!(offset_of!(Class, r#abstract) == 92usize);
    assert!(offset_of!(Class, declare) == 93usize);
    assert!(offset_of!(Class, scope_id) == 96usize);

    assert!(size_of::<ClassType>() == 1usize);
    assert!(align_of::<ClassType>() == 1usize);

    assert!(size_of::<ClassBody>() == 28usize);
    assert!(align_of::<ClassBody>() == 4usize);
    assert!(offset_of!(ClassBody, node_id) == 0usize);
    assert!(offset_of!(ClassBody, span) == 4usize);
    assert!(offset_of!(ClassBody, body) == 12usize);

    assert!(size_of::<ClassElement>() == 8usize);
    assert!(align_of::<ClassElement>() == 4usize);

    assert!(size_of::<MethodDefinition>() == 52usize);
    assert!(align_of::<MethodDefinition>() == 4usize);
    assert!(offset_of!(MethodDefinition, node_id) == 0usize);
    assert!(offset_of!(MethodDefinition, r#type) == 4usize);
    assert!(offset_of!(MethodDefinition, span) == 8usize);
    assert!(offset_of!(MethodDefinition, decorators) == 16usize);
    assert!(offset_of!(MethodDefinition, key) == 32usize);
    assert!(offset_of!(MethodDefinition, value) == 40usize);
    assert!(offset_of!(MethodDefinition, kind) == 44usize);
    assert!(offset_of!(MethodDefinition, computed) == 45usize);
    assert!(offset_of!(MethodDefinition, r#static) == 46usize);
    assert!(offset_of!(MethodDefinition, r#override) == 47usize);
    assert!(offset_of!(MethodDefinition, optional) == 48usize);
    assert!(offset_of!(MethodDefinition, accessibility) == 49usize);

    assert!(size_of::<MethodDefinitionType>() == 1usize);
    assert!(align_of::<MethodDefinitionType>() == 1usize);

    assert!(size_of::<PropertyDefinition>() == 64usize);
    assert!(align_of::<PropertyDefinition>() == 4usize);
    assert!(offset_of!(PropertyDefinition, node_id) == 0usize);
    assert!(offset_of!(PropertyDefinition, r#type) == 4usize);
    assert!(offset_of!(PropertyDefinition, span) == 8usize);
    assert!(offset_of!(PropertyDefinition, decorators) == 16usize);
    assert!(offset_of!(PropertyDefinition, key) == 32usize);
    assert!(offset_of!(PropertyDefinition, value) == 40usize);
    assert!(offset_of!(PropertyDefinition, computed) == 48usize);
    assert!(offset_of!(PropertyDefinition, r#static) == 49usize);
    assert!(offset_of!(PropertyDefinition, declare) == 50usize);
    assert!(offset_of!(PropertyDefinition, r#override) == 51usize);
    assert!(offset_of!(PropertyDefinition, optional) == 52usize);
    assert!(offset_of!(PropertyDefinition, definite) == 53usize);
    assert!(offset_of!(PropertyDefinition, readonly) == 54usize);
    assert!(offset_of!(PropertyDefinition, type_annotation) == 56usize);
    assert!(offset_of!(PropertyDefinition, accessibility) == 60usize);

    assert!(size_of::<PropertyDefinitionType>() == 1usize);
    assert!(align_of::<PropertyDefinitionType>() == 1usize);

    assert!(size_of::<MethodDefinitionKind>() == 1usize);
    assert!(align_of::<MethodDefinitionKind>() == 1usize);

    assert!(size_of::<PrivateIdentifier>() == 20usize);
    assert!(align_of::<PrivateIdentifier>() == 4usize);
    assert!(offset_of!(PrivateIdentifier, node_id) == 0usize);
    assert!(offset_of!(PrivateIdentifier, span) == 4usize);
    assert!(offset_of!(PrivateIdentifier, name) == 12usize);

    assert!(size_of::<StaticBlock>() == 32usize);
    assert!(align_of::<StaticBlock>() == 4usize);
    assert!(offset_of!(StaticBlock, node_id) == 0usize);
    assert!(offset_of!(StaticBlock, span) == 4usize);
    assert!(offset_of!(StaticBlock, body) == 12usize);
    assert!(offset_of!(StaticBlock, scope_id) == 28usize);

    assert!(size_of::<ModuleDeclaration>() == 8usize);
    assert!(align_of::<ModuleDeclaration>() == 4usize);

    assert!(size_of::<AccessorPropertyType>() == 1usize);
    assert!(align_of::<AccessorPropertyType>() == 1usize);

    assert!(size_of::<AccessorProperty>() == 60usize);
    assert!(align_of::<AccessorProperty>() == 4usize);
    assert!(offset_of!(AccessorProperty, node_id) == 0usize);
    assert!(offset_of!(AccessorProperty, r#type) == 4usize);
    assert!(offset_of!(AccessorProperty, span) == 8usize);
    assert!(offset_of!(AccessorProperty, decorators) == 16usize);
    assert!(offset_of!(AccessorProperty, key) == 32usize);
    assert!(offset_of!(AccessorProperty, value) == 40usize);
    assert!(offset_of!(AccessorProperty, computed) == 48usize);
    assert!(offset_of!(AccessorProperty, r#static) == 49usize);
    assert!(offset_of!(AccessorProperty, definite) == 50usize);
    assert!(offset_of!(AccessorProperty, type_annotation) == 52usize);
    assert!(offset_of!(AccessorProperty, accessibility) == 56usize);

    assert!(size_of::<ImportExpression>() == 36usize);
    assert!(align_of::<ImportExpression>() == 4usize);
    assert!(offset_of!(ImportExpression, node_id) == 0usize);
    assert!(offset_of!(ImportExpression, span) == 4usize);
    assert!(offset_of!(ImportExpression, source) == 12usize);
    assert!(offset_of!(ImportExpression, arguments) == 20usize);

    assert!(size_of::<ImportDeclaration>() == 56usize);
    assert!(align_of::<ImportDeclaration>() == 4usize);
    assert!(offset_of!(ImportDeclaration, node_id) == 0usize);
    assert!(offset_of!(ImportDeclaration, span) == 4usize);
    assert!(offset_of!(ImportDeclaration, specifiers) == 12usize);
    assert!(offset_of!(ImportDeclaration, source) == 28usize);
    assert!(offset_of!(ImportDeclaration, with_clause) == 48usize);
    assert!(offset_of!(ImportDeclaration, import_kind) == 52usize);

    assert!(size_of::<ImportDeclarationSpecifier>() == 8usize);
    assert!(align_of::<ImportDeclarationSpecifier>() == 4usize);

    assert!(size_of::<ImportSpecifier>() == 68usize);
    assert!(align_of::<ImportSpecifier>() == 4usize);
    assert!(offset_of!(ImportSpecifier, node_id) == 0usize);
    assert!(offset_of!(ImportSpecifier, span) == 4usize);
    assert!(offset_of!(ImportSpecifier, imported) == 12usize);
    assert!(offset_of!(ImportSpecifier, local) == 40usize);
    assert!(offset_of!(ImportSpecifier, import_kind) == 64usize);

    assert!(size_of::<ImportDefaultSpecifier>() == 36usize);
    assert!(align_of::<ImportDefaultSpecifier>() == 4usize);
    assert!(offset_of!(ImportDefaultSpecifier, node_id) == 0usize);
    assert!(offset_of!(ImportDefaultSpecifier, span) == 4usize);
    assert!(offset_of!(ImportDefaultSpecifier, local) == 12usize);

    assert!(size_of::<ImportNamespaceSpecifier>() == 36usize);
    assert!(align_of::<ImportNamespaceSpecifier>() == 4usize);
    assert!(offset_of!(ImportNamespaceSpecifier, node_id) == 0usize);
    assert!(offset_of!(ImportNamespaceSpecifier, span) == 4usize);
    assert!(offset_of!(ImportNamespaceSpecifier, local) == 12usize);

    assert!(size_of::<WithClause>() == 48usize);
    assert!(align_of::<WithClause>() == 4usize);
    assert!(offset_of!(WithClause, node_id) == 0usize);
    assert!(offset_of!(WithClause, span) == 4usize);
    assert!(offset_of!(WithClause, attributes_keyword) == 12usize);
    assert!(offset_of!(WithClause, with_entries) == 32usize);

    assert!(size_of::<ImportAttribute>() == 56usize);
    assert!(align_of::<ImportAttribute>() == 4usize);
    assert!(offset_of!(ImportAttribute, node_id) == 0usize);
    assert!(offset_of!(ImportAttribute, span) == 4usize);
    assert!(offset_of!(ImportAttribute, key) == 12usize);
    assert!(offset_of!(ImportAttribute, value) == 36usize);

    assert!(size_of::<ImportAttributeKey>() == 24usize);
    assert!(align_of::<ImportAttributeKey>() == 4usize);

    assert!(size_of::<ExportNamedDeclaration>() == 64usize);
    assert!(align_of::<ExportNamedDeclaration>() == 4usize);
    assert!(offset_of!(ExportNamedDeclaration, node_id) == 0usize);
    assert!(offset_of!(ExportNamedDeclaration, span) == 4usize);
    assert!(offset_of!(ExportNamedDeclaration, declaration) == 12usize);
    assert!(offset_of!(ExportNamedDeclaration, specifiers) == 20usize);
    assert!(offset_of!(ExportNamedDeclaration, source) == 36usize);
    assert!(offset_of!(ExportNamedDeclaration, export_kind) == 56usize);
    assert!(offset_of!(ExportNamedDeclaration, with_clause) == 60usize);

    assert!(size_of::<ExportDefaultDeclaration>() == 48usize);
    assert!(align_of::<ExportDefaultDeclaration>() == 4usize);
    assert!(offset_of!(ExportDefaultDeclaration, node_id) == 0usize);
    assert!(offset_of!(ExportDefaultDeclaration, span) == 4usize);
    assert!(offset_of!(ExportDefaultDeclaration, declaration) == 12usize);
    assert!(offset_of!(ExportDefaultDeclaration, exported) == 20usize);

    assert!(size_of::<ExportAllDeclaration>() == 68usize);
    assert!(align_of::<ExportAllDeclaration>() == 4usize);
    assert!(offset_of!(ExportAllDeclaration, node_id) == 0usize);
    assert!(offset_of!(ExportAllDeclaration, span) == 4usize);
    assert!(offset_of!(ExportAllDeclaration, exported) == 12usize);
    assert!(offset_of!(ExportAllDeclaration, source) == 40usize);
    assert!(offset_of!(ExportAllDeclaration, with_clause) == 60usize);
    assert!(offset_of!(ExportAllDeclaration, export_kind) == 64usize);

    assert!(size_of::<ExportSpecifier>() == 72usize);
    assert!(align_of::<ExportSpecifier>() == 4usize);
    assert!(offset_of!(ExportSpecifier, node_id) == 0usize);
    assert!(offset_of!(ExportSpecifier, span) == 4usize);
    assert!(offset_of!(ExportSpecifier, local) == 12usize);
    assert!(offset_of!(ExportSpecifier, exported) == 40usize);
    assert!(offset_of!(ExportSpecifier, export_kind) == 68usize);

    assert!(size_of::<ExportDefaultDeclarationKind>() == 8usize);
    assert!(align_of::<ExportDefaultDeclarationKind>() == 4usize);

    assert!(size_of::<ModuleExportName>() == 28usize);
    assert!(align_of::<ModuleExportName>() == 4usize);

    assert!(size_of::<TSThisParameter>() == 24usize);
    assert!(align_of::<TSThisParameter>() == 4usize);
    assert!(offset_of!(TSThisParameter, node_id) == 0usize);
    assert!(offset_of!(TSThisParameter, span) == 4usize);
    assert!(offset_of!(TSThisParameter, this_span) == 12usize);
    assert!(offset_of!(TSThisParameter, type_annotation) == 20usize);

    assert!(size_of::<TSEnumDeclaration>() == 60usize);
    assert!(align_of::<TSEnumDeclaration>() == 4usize);
    assert!(offset_of!(TSEnumDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSEnumDeclaration, span) == 4usize);
    assert!(offset_of!(TSEnumDeclaration, id) == 12usize);
    assert!(offset_of!(TSEnumDeclaration, members) == 36usize);
    assert!(offset_of!(TSEnumDeclaration, r#const) == 52usize);
    assert!(offset_of!(TSEnumDeclaration, declare) == 53usize);
    assert!(offset_of!(TSEnumDeclaration, scope_id) == 56usize);

    assert!(size_of::<TSEnumMember>() == 28usize);
    assert!(align_of::<TSEnumMember>() == 4usize);
    assert!(offset_of!(TSEnumMember, node_id) == 0usize);
    assert!(offset_of!(TSEnumMember, span) == 4usize);
    assert!(offset_of!(TSEnumMember, id) == 12usize);
    assert!(offset_of!(TSEnumMember, initializer) == 20usize);

    assert!(size_of::<TSEnumMemberName>() == 8usize);
    assert!(align_of::<TSEnumMemberName>() == 4usize);

    assert!(size_of::<TSTypeAnnotation>() == 20usize);
    assert!(align_of::<TSTypeAnnotation>() == 4usize);
    assert!(offset_of!(TSTypeAnnotation, node_id) == 0usize);
    assert!(offset_of!(TSTypeAnnotation, span) == 4usize);
    assert!(offset_of!(TSTypeAnnotation, type_annotation) == 12usize);

    assert!(size_of::<TSLiteralType>() == 20usize);
    assert!(align_of::<TSLiteralType>() == 4usize);
    assert!(offset_of!(TSLiteralType, node_id) == 0usize);
    assert!(offset_of!(TSLiteralType, span) == 4usize);
    assert!(offset_of!(TSLiteralType, literal) == 12usize);

    assert!(size_of::<TSLiteral>() == 8usize);
    assert!(align_of::<TSLiteral>() == 4usize);

    assert!(size_of::<TSType>() == 8usize);
    assert!(align_of::<TSType>() == 4usize);

    assert!(size_of::<TSConditionalType>() == 48usize);
    assert!(align_of::<TSConditionalType>() == 4usize);
    assert!(offset_of!(TSConditionalType, node_id) == 0usize);
    assert!(offset_of!(TSConditionalType, span) == 4usize);
    assert!(offset_of!(TSConditionalType, check_type) == 12usize);
    assert!(offset_of!(TSConditionalType, extends_type) == 20usize);
    assert!(offset_of!(TSConditionalType, true_type) == 28usize);
    assert!(offset_of!(TSConditionalType, false_type) == 36usize);
    assert!(offset_of!(TSConditionalType, scope_id) == 44usize);

    assert!(size_of::<TSUnionType>() == 28usize);
    assert!(align_of::<TSUnionType>() == 4usize);
    assert!(offset_of!(TSUnionType, node_id) == 0usize);
    assert!(offset_of!(TSUnionType, span) == 4usize);
    assert!(offset_of!(TSUnionType, types) == 12usize);

    assert!(size_of::<TSIntersectionType>() == 28usize);
    assert!(align_of::<TSIntersectionType>() == 4usize);
    assert!(offset_of!(TSIntersectionType, node_id) == 0usize);
    assert!(offset_of!(TSIntersectionType, span) == 4usize);
    assert!(offset_of!(TSIntersectionType, types) == 12usize);

    assert!(size_of::<TSParenthesizedType>() == 20usize);
    assert!(align_of::<TSParenthesizedType>() == 4usize);
    assert!(offset_of!(TSParenthesizedType, node_id) == 0usize);
    assert!(offset_of!(TSParenthesizedType, span) == 4usize);
    assert!(offset_of!(TSParenthesizedType, type_annotation) == 12usize);

    assert!(size_of::<TSTypeOperator>() == 24usize);
    assert!(align_of::<TSTypeOperator>() == 4usize);
    assert!(offset_of!(TSTypeOperator, node_id) == 0usize);
    assert!(offset_of!(TSTypeOperator, span) == 4usize);
    assert!(offset_of!(TSTypeOperator, operator) == 12usize);
    assert!(offset_of!(TSTypeOperator, type_annotation) == 16usize);

    assert!(size_of::<TSTypeOperatorOperator>() == 1usize);
    assert!(align_of::<TSTypeOperatorOperator>() == 1usize);

    assert!(size_of::<TSArrayType>() == 20usize);
    assert!(align_of::<TSArrayType>() == 4usize);
    assert!(offset_of!(TSArrayType, node_id) == 0usize);
    assert!(offset_of!(TSArrayType, span) == 4usize);
    assert!(offset_of!(TSArrayType, element_type) == 12usize);

    assert!(size_of::<TSIndexedAccessType>() == 28usize);
    assert!(align_of::<TSIndexedAccessType>() == 4usize);
    assert!(offset_of!(TSIndexedAccessType, node_id) == 0usize);
    assert!(offset_of!(TSIndexedAccessType, span) == 4usize);
    assert!(offset_of!(TSIndexedAccessType, object_type) == 12usize);
    assert!(offset_of!(TSIndexedAccessType, index_type) == 20usize);

    assert!(size_of::<TSTupleType>() == 28usize);
    assert!(align_of::<TSTupleType>() == 4usize);
    assert!(offset_of!(TSTupleType, node_id) == 0usize);
    assert!(offset_of!(TSTupleType, span) == 4usize);
    assert!(offset_of!(TSTupleType, element_types) == 12usize);

    assert!(size_of::<TSNamedTupleMember>() == 44usize);
    assert!(align_of::<TSNamedTupleMember>() == 4usize);
    assert!(offset_of!(TSNamedTupleMember, node_id) == 0usize);
    assert!(offset_of!(TSNamedTupleMember, span) == 4usize);
    assert!(offset_of!(TSNamedTupleMember, element_type) == 12usize);
    assert!(offset_of!(TSNamedTupleMember, label) == 20usize);
    assert!(offset_of!(TSNamedTupleMember, optional) == 40usize);

    assert!(size_of::<TSOptionalType>() == 20usize);
    assert!(align_of::<TSOptionalType>() == 4usize);
    assert!(offset_of!(TSOptionalType, node_id) == 0usize);
    assert!(offset_of!(TSOptionalType, span) == 4usize);
    assert!(offset_of!(TSOptionalType, type_annotation) == 12usize);

    assert!(size_of::<TSRestType>() == 20usize);
    assert!(align_of::<TSRestType>() == 4usize);
    assert!(offset_of!(TSRestType, node_id) == 0usize);
    assert!(offset_of!(TSRestType, span) == 4usize);
    assert!(offset_of!(TSRestType, type_annotation) == 12usize);

    assert!(size_of::<TSTupleElement>() == 8usize);
    assert!(align_of::<TSTupleElement>() == 4usize);

    assert!(size_of::<TSAnyKeyword>() == 12usize);
    assert!(align_of::<TSAnyKeyword>() == 4usize);
    assert!(offset_of!(TSAnyKeyword, node_id) == 0usize);
    assert!(offset_of!(TSAnyKeyword, span) == 4usize);

    assert!(size_of::<TSStringKeyword>() == 12usize);
    assert!(align_of::<TSStringKeyword>() == 4usize);
    assert!(offset_of!(TSStringKeyword, node_id) == 0usize);
    assert!(offset_of!(TSStringKeyword, span) == 4usize);

    assert!(size_of::<TSBooleanKeyword>() == 12usize);
    assert!(align_of::<TSBooleanKeyword>() == 4usize);
    assert!(offset_of!(TSBooleanKeyword, node_id) == 0usize);
    assert!(offset_of!(TSBooleanKeyword, span) == 4usize);

    assert!(size_of::<TSNumberKeyword>() == 12usize);
    assert!(align_of::<TSNumberKeyword>() == 4usize);
    assert!(offset_of!(TSNumberKeyword, node_id) == 0usize);
    assert!(offset_of!(TSNumberKeyword, span) == 4usize);

    assert!(size_of::<TSNeverKeyword>() == 12usize);
    assert!(align_of::<TSNeverKeyword>() == 4usize);
    assert!(offset_of!(TSNeverKeyword, node_id) == 0usize);
    assert!(offset_of!(TSNeverKeyword, span) == 4usize);

    assert!(size_of::<TSIntrinsicKeyword>() == 12usize);
    assert!(align_of::<TSIntrinsicKeyword>() == 4usize);
    assert!(offset_of!(TSIntrinsicKeyword, node_id) == 0usize);
    assert!(offset_of!(TSIntrinsicKeyword, span) == 4usize);

    assert!(size_of::<TSUnknownKeyword>() == 12usize);
    assert!(align_of::<TSUnknownKeyword>() == 4usize);
    assert!(offset_of!(TSUnknownKeyword, node_id) == 0usize);
    assert!(offset_of!(TSUnknownKeyword, span) == 4usize);

    assert!(size_of::<TSNullKeyword>() == 12usize);
    assert!(align_of::<TSNullKeyword>() == 4usize);
    assert!(offset_of!(TSNullKeyword, node_id) == 0usize);
    assert!(offset_of!(TSNullKeyword, span) == 4usize);

    assert!(size_of::<TSUndefinedKeyword>() == 12usize);
    assert!(align_of::<TSUndefinedKeyword>() == 4usize);
    assert!(offset_of!(TSUndefinedKeyword, node_id) == 0usize);
    assert!(offset_of!(TSUndefinedKeyword, span) == 4usize);

    assert!(size_of::<TSVoidKeyword>() == 12usize);
    assert!(align_of::<TSVoidKeyword>() == 4usize);
    assert!(offset_of!(TSVoidKeyword, node_id) == 0usize);
    assert!(offset_of!(TSVoidKeyword, span) == 4usize);

    assert!(size_of::<TSSymbolKeyword>() == 12usize);
    assert!(align_of::<TSSymbolKeyword>() == 4usize);
    assert!(offset_of!(TSSymbolKeyword, node_id) == 0usize);
    assert!(offset_of!(TSSymbolKeyword, span) == 4usize);

    assert!(size_of::<TSThisType>() == 12usize);
    assert!(align_of::<TSThisType>() == 4usize);
    assert!(offset_of!(TSThisType, node_id) == 0usize);
    assert!(offset_of!(TSThisType, span) == 4usize);

    assert!(size_of::<TSObjectKeyword>() == 12usize);
    assert!(align_of::<TSObjectKeyword>() == 4usize);
    assert!(offset_of!(TSObjectKeyword, node_id) == 0usize);
    assert!(offset_of!(TSObjectKeyword, span) == 4usize);

    assert!(size_of::<TSBigIntKeyword>() == 12usize);
    assert!(align_of::<TSBigIntKeyword>() == 4usize);
    assert!(offset_of!(TSBigIntKeyword, node_id) == 0usize);
    assert!(offset_of!(TSBigIntKeyword, span) == 4usize);

    assert!(size_of::<TSTypeReference>() == 24usize);
    assert!(align_of::<TSTypeReference>() == 4usize);
    assert!(offset_of!(TSTypeReference, node_id) == 0usize);
    assert!(offset_of!(TSTypeReference, span) == 4usize);
    assert!(offset_of!(TSTypeReference, type_name) == 12usize);
    assert!(offset_of!(TSTypeReference, type_parameters) == 20usize);

    assert!(size_of::<TSTypeName>() == 8usize);
    assert!(align_of::<TSTypeName>() == 4usize);

    assert!(size_of::<TSQualifiedName>() == 40usize);
    assert!(align_of::<TSQualifiedName>() == 4usize);
    assert!(offset_of!(TSQualifiedName, node_id) == 0usize);
    assert!(offset_of!(TSQualifiedName, span) == 4usize);
    assert!(offset_of!(TSQualifiedName, left) == 12usize);
    assert!(offset_of!(TSQualifiedName, right) == 20usize);

    assert!(size_of::<TSTypeParameterInstantiation>() == 28usize);
    assert!(align_of::<TSTypeParameterInstantiation>() == 4usize);
    assert!(offset_of!(TSTypeParameterInstantiation, node_id) == 0usize);
    assert!(offset_of!(TSTypeParameterInstantiation, span) == 4usize);
    assert!(offset_of!(TSTypeParameterInstantiation, params) == 12usize);

    assert!(size_of::<TSTypeParameter>() == 56usize);
    assert!(align_of::<TSTypeParameter>() == 4usize);
    assert!(offset_of!(TSTypeParameter, node_id) == 0usize);
    assert!(offset_of!(TSTypeParameter, span) == 4usize);
    assert!(offset_of!(TSTypeParameter, name) == 12usize);
    assert!(offset_of!(TSTypeParameter, constraint) == 36usize);
    assert!(offset_of!(TSTypeParameter, default) == 44usize);
    assert!(offset_of!(TSTypeParameter, r#in) == 52usize);
    assert!(offset_of!(TSTypeParameter, out) == 53usize);
    assert!(offset_of!(TSTypeParameter, r#const) == 54usize);

    assert!(size_of::<TSTypeParameterDeclaration>() == 28usize);
    assert!(align_of::<TSTypeParameterDeclaration>() == 4usize);
    assert!(offset_of!(TSTypeParameterDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSTypeParameterDeclaration, span) == 4usize);
    assert!(offset_of!(TSTypeParameterDeclaration, params) == 12usize);

    assert!(size_of::<TSTypeAliasDeclaration>() == 56usize);
    assert!(align_of::<TSTypeAliasDeclaration>() == 4usize);
    assert!(offset_of!(TSTypeAliasDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSTypeAliasDeclaration, span) == 4usize);
    assert!(offset_of!(TSTypeAliasDeclaration, id) == 12usize);
    assert!(offset_of!(TSTypeAliasDeclaration, type_parameters) == 36usize);
    assert!(offset_of!(TSTypeAliasDeclaration, type_annotation) == 40usize);
    assert!(offset_of!(TSTypeAliasDeclaration, declare) == 48usize);
    assert!(offset_of!(TSTypeAliasDeclaration, scope_id) == 52usize);

    assert!(size_of::<TSAccessibility>() == 1usize);
    assert!(align_of::<TSAccessibility>() == 1usize);

    assert!(size_of::<TSClassImplements>() == 24usize);
    assert!(align_of::<TSClassImplements>() == 4usize);
    assert!(offset_of!(TSClassImplements, node_id) == 0usize);
    assert!(offset_of!(TSClassImplements, span) == 4usize);
    assert!(offset_of!(TSClassImplements, expression) == 12usize);
    assert!(offset_of!(TSClassImplements, type_parameters) == 20usize);

    assert!(size_of::<TSInterfaceDeclaration>() == 68usize);
    assert!(align_of::<TSInterfaceDeclaration>() == 4usize);
    assert!(offset_of!(TSInterfaceDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSInterfaceDeclaration, span) == 4usize);
    assert!(offset_of!(TSInterfaceDeclaration, id) == 12usize);
    assert!(offset_of!(TSInterfaceDeclaration, extends) == 36usize);
    assert!(offset_of!(TSInterfaceDeclaration, type_parameters) == 52usize);
    assert!(offset_of!(TSInterfaceDeclaration, body) == 56usize);
    assert!(offset_of!(TSInterfaceDeclaration, declare) == 60usize);
    assert!(offset_of!(TSInterfaceDeclaration, scope_id) == 64usize);

    assert!(size_of::<TSInterfaceBody>() == 28usize);
    assert!(align_of::<TSInterfaceBody>() == 4usize);
    assert!(offset_of!(TSInterfaceBody, node_id) == 0usize);
    assert!(offset_of!(TSInterfaceBody, span) == 4usize);
    assert!(offset_of!(TSInterfaceBody, body) == 12usize);

    assert!(size_of::<TSPropertySignature>() == 28usize);
    assert!(align_of::<TSPropertySignature>() == 4usize);
    assert!(offset_of!(TSPropertySignature, node_id) == 0usize);
    assert!(offset_of!(TSPropertySignature, span) == 4usize);
    assert!(offset_of!(TSPropertySignature, computed) == 12usize);
    assert!(offset_of!(TSPropertySignature, optional) == 13usize);
    assert!(offset_of!(TSPropertySignature, readonly) == 14usize);
    assert!(offset_of!(TSPropertySignature, key) == 16usize);
    assert!(offset_of!(TSPropertySignature, type_annotation) == 24usize);

    assert!(size_of::<TSSignature>() == 8usize);
    assert!(align_of::<TSSignature>() == 4usize);

    assert!(size_of::<TSIndexSignature>() == 36usize);
    assert!(align_of::<TSIndexSignature>() == 4usize);
    assert!(offset_of!(TSIndexSignature, node_id) == 0usize);
    assert!(offset_of!(TSIndexSignature, span) == 4usize);
    assert!(offset_of!(TSIndexSignature, parameters) == 12usize);
    assert!(offset_of!(TSIndexSignature, type_annotation) == 28usize);
    assert!(offset_of!(TSIndexSignature, readonly) == 32usize);

    assert!(size_of::<TSCallSignatureDeclaration>() == 48usize);
    assert!(align_of::<TSCallSignatureDeclaration>() == 4usize);
    assert!(offset_of!(TSCallSignatureDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSCallSignatureDeclaration, span) == 4usize);
    assert!(offset_of!(TSCallSignatureDeclaration, this_param) == 12usize);
    assert!(offset_of!(TSCallSignatureDeclaration, params) == 36usize);
    assert!(offset_of!(TSCallSignatureDeclaration, return_type) == 40usize);
    assert!(offset_of!(TSCallSignatureDeclaration, type_parameters) == 44usize);

    assert!(size_of::<TSMethodSignatureKind>() == 1usize);
    assert!(align_of::<TSMethodSignatureKind>() == 1usize);

    assert!(size_of::<TSMethodSignature>() == 44usize);
    assert!(align_of::<TSMethodSignature>() == 4usize);
    assert!(offset_of!(TSMethodSignature, node_id) == 0usize);
    assert!(offset_of!(TSMethodSignature, span) == 4usize);
    assert!(offset_of!(TSMethodSignature, key) == 12usize);
    assert!(offset_of!(TSMethodSignature, computed) == 20usize);
    assert!(offset_of!(TSMethodSignature, optional) == 21usize);
    assert!(offset_of!(TSMethodSignature, kind) == 22usize);
    assert!(offset_of!(TSMethodSignature, this_param) == 24usize);
    assert!(offset_of!(TSMethodSignature, params) == 28usize);
    assert!(offset_of!(TSMethodSignature, return_type) == 32usize);
    assert!(offset_of!(TSMethodSignature, type_parameters) == 36usize);
    assert!(offset_of!(TSMethodSignature, scope_id) == 40usize);

    assert!(size_of::<TSConstructSignatureDeclaration>() == 28usize);
    assert!(align_of::<TSConstructSignatureDeclaration>() == 4usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, span) == 4usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, params) == 12usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, return_type) == 16usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, type_parameters) == 20usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, scope_id) == 24usize);

    assert!(size_of::<TSIndexSignatureName>() == 24usize);
    assert!(align_of::<TSIndexSignatureName>() == 4usize);
    assert!(offset_of!(TSIndexSignatureName, node_id) == 0usize);
    assert!(offset_of!(TSIndexSignatureName, span) == 4usize);
    assert!(offset_of!(TSIndexSignatureName, name) == 12usize);
    assert!(offset_of!(TSIndexSignatureName, type_annotation) == 20usize);

    assert!(size_of::<TSInterfaceHeritage>() == 24usize);
    assert!(align_of::<TSInterfaceHeritage>() == 4usize);
    assert!(offset_of!(TSInterfaceHeritage, node_id) == 0usize);
    assert!(offset_of!(TSInterfaceHeritage, span) == 4usize);
    assert!(offset_of!(TSInterfaceHeritage, expression) == 12usize);
    assert!(offset_of!(TSInterfaceHeritage, type_parameters) == 20usize);

    assert!(size_of::<TSTypePredicate>() == 36usize);
    assert!(align_of::<TSTypePredicate>() == 4usize);
    assert!(offset_of!(TSTypePredicate, node_id) == 0usize);
    assert!(offset_of!(TSTypePredicate, span) == 4usize);
    assert!(offset_of!(TSTypePredicate, parameter_name) == 12usize);
    assert!(offset_of!(TSTypePredicate, asserts) == 28usize);
    assert!(offset_of!(TSTypePredicate, type_annotation) == 32usize);

    assert!(size_of::<TSTypePredicateName>() == 16usize);
    assert!(align_of::<TSTypePredicateName>() == 4usize);

    assert!(size_of::<TSModuleDeclaration>() == 52usize);
    assert!(align_of::<TSModuleDeclaration>() == 4usize);
    assert!(offset_of!(TSModuleDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSModuleDeclaration, span) == 4usize);
    assert!(offset_of!(TSModuleDeclaration, id) == 12usize);
    assert!(offset_of!(TSModuleDeclaration, body) == 36usize);
    assert!(offset_of!(TSModuleDeclaration, kind) == 44usize);
    assert!(offset_of!(TSModuleDeclaration, declare) == 45usize);
    assert!(offset_of!(TSModuleDeclaration, scope_id) == 48usize);

    assert!(size_of::<TSModuleDeclarationKind>() == 1usize);
    assert!(align_of::<TSModuleDeclarationKind>() == 1usize);

    assert!(size_of::<TSModuleDeclarationName>() == 24usize);
    assert!(align_of::<TSModuleDeclarationName>() == 4usize);

    assert!(size_of::<TSModuleDeclarationBody>() == 8usize);
    assert!(align_of::<TSModuleDeclarationBody>() == 4usize);

    assert!(size_of::<TSModuleBlock>() == 44usize);
    assert!(align_of::<TSModuleBlock>() == 4usize);
    assert!(offset_of!(TSModuleBlock, node_id) == 0usize);
    assert!(offset_of!(TSModuleBlock, span) == 4usize);
    assert!(offset_of!(TSModuleBlock, directives) == 12usize);
    assert!(offset_of!(TSModuleBlock, body) == 28usize);

    assert!(size_of::<TSTypeLiteral>() == 28usize);
    assert!(align_of::<TSTypeLiteral>() == 4usize);
    assert!(offset_of!(TSTypeLiteral, node_id) == 0usize);
    assert!(offset_of!(TSTypeLiteral, span) == 4usize);
    assert!(offset_of!(TSTypeLiteral, members) == 12usize);

    assert!(size_of::<TSInferType>() == 16usize);
    assert!(align_of::<TSInferType>() == 4usize);
    assert!(offset_of!(TSInferType, node_id) == 0usize);
    assert!(offset_of!(TSInferType, span) == 4usize);
    assert!(offset_of!(TSInferType, type_parameter) == 12usize);

    assert!(size_of::<TSTypeQuery>() == 24usize);
    assert!(align_of::<TSTypeQuery>() == 4usize);
    assert!(offset_of!(TSTypeQuery, node_id) == 0usize);
    assert!(offset_of!(TSTypeQuery, span) == 4usize);
    assert!(offset_of!(TSTypeQuery, expr_name) == 12usize);
    assert!(offset_of!(TSTypeQuery, type_parameters) == 20usize);

    assert!(size_of::<TSTypeQueryExprName>() == 8usize);
    assert!(align_of::<TSTypeQueryExprName>() == 4usize);

    assert!(size_of::<TSImportType>() == 40usize);
    assert!(align_of::<TSImportType>() == 4usize);
    assert!(offset_of!(TSImportType, node_id) == 0usize);
    assert!(offset_of!(TSImportType, span) == 4usize);
    assert!(offset_of!(TSImportType, is_type_of) == 12usize);
    assert!(offset_of!(TSImportType, parameter) == 16usize);
    assert!(offset_of!(TSImportType, qualifier) == 24usize);
    assert!(offset_of!(TSImportType, attributes) == 32usize);
    assert!(offset_of!(TSImportType, type_parameters) == 36usize);

    assert!(size_of::<TSImportAttributes>() == 48usize);
    assert!(align_of::<TSImportAttributes>() == 4usize);
    assert!(offset_of!(TSImportAttributes, node_id) == 0usize);
    assert!(offset_of!(TSImportAttributes, span) == 4usize);
    assert!(offset_of!(TSImportAttributes, attributes_keyword) == 12usize);
    assert!(offset_of!(TSImportAttributes, elements) == 32usize);

    assert!(size_of::<TSImportAttribute>() == 44usize);
    assert!(align_of::<TSImportAttribute>() == 4usize);
    assert!(offset_of!(TSImportAttribute, node_id) == 0usize);
    assert!(offset_of!(TSImportAttribute, span) == 4usize);
    assert!(offset_of!(TSImportAttribute, name) == 12usize);
    assert!(offset_of!(TSImportAttribute, value) == 36usize);

    assert!(size_of::<TSImportAttributeName>() == 24usize);
    assert!(align_of::<TSImportAttributeName>() == 4usize);

    assert!(size_of::<TSFunctionType>() == 28usize);
    assert!(align_of::<TSFunctionType>() == 4usize);
    assert!(offset_of!(TSFunctionType, node_id) == 0usize);
    assert!(offset_of!(TSFunctionType, span) == 4usize);
    assert!(offset_of!(TSFunctionType, this_param) == 12usize);
    assert!(offset_of!(TSFunctionType, params) == 16usize);
    assert!(offset_of!(TSFunctionType, return_type) == 20usize);
    assert!(offset_of!(TSFunctionType, type_parameters) == 24usize);

    assert!(size_of::<TSConstructorType>() == 28usize);
    assert!(align_of::<TSConstructorType>() == 4usize);
    assert!(offset_of!(TSConstructorType, node_id) == 0usize);
    assert!(offset_of!(TSConstructorType, span) == 4usize);
    assert!(offset_of!(TSConstructorType, r#abstract) == 12usize);
    assert!(offset_of!(TSConstructorType, params) == 16usize);
    assert!(offset_of!(TSConstructorType, return_type) == 20usize);
    assert!(offset_of!(TSConstructorType, type_parameters) == 24usize);

    assert!(size_of::<TSMappedType>() == 40usize);
    assert!(align_of::<TSMappedType>() == 4usize);
    assert!(offset_of!(TSMappedType, node_id) == 0usize);
    assert!(offset_of!(TSMappedType, span) == 4usize);
    assert!(offset_of!(TSMappedType, type_parameter) == 12usize);
    assert!(offset_of!(TSMappedType, name_type) == 16usize);
    assert!(offset_of!(TSMappedType, type_annotation) == 24usize);
    assert!(offset_of!(TSMappedType, optional) == 32usize);
    assert!(offset_of!(TSMappedType, readonly) == 33usize);
    assert!(offset_of!(TSMappedType, scope_id) == 36usize);

    assert!(size_of::<TSMappedTypeModifierOperator>() == 1usize);
    assert!(align_of::<TSMappedTypeModifierOperator>() == 1usize);

    assert!(size_of::<TSTemplateLiteralType>() == 44usize);
    assert!(align_of::<TSTemplateLiteralType>() == 4usize);
    assert!(offset_of!(TSTemplateLiteralType, node_id) == 0usize);
    assert!(offset_of!(TSTemplateLiteralType, span) == 4usize);
    assert!(offset_of!(TSTemplateLiteralType, quasis) == 12usize);
    assert!(offset_of!(TSTemplateLiteralType, types) == 28usize);

    assert!(size_of::<TSAsExpression>() == 28usize);
    assert!(align_of::<TSAsExpression>() == 4usize);
    assert!(offset_of!(TSAsExpression, node_id) == 0usize);
    assert!(offset_of!(TSAsExpression, span) == 4usize);
    assert!(offset_of!(TSAsExpression, expression) == 12usize);
    assert!(offset_of!(TSAsExpression, type_annotation) == 20usize);

    assert!(size_of::<TSSatisfiesExpression>() == 28usize);
    assert!(align_of::<TSSatisfiesExpression>() == 4usize);
    assert!(offset_of!(TSSatisfiesExpression, node_id) == 0usize);
    assert!(offset_of!(TSSatisfiesExpression, span) == 4usize);
    assert!(offset_of!(TSSatisfiesExpression, expression) == 12usize);
    assert!(offset_of!(TSSatisfiesExpression, type_annotation) == 20usize);

    assert!(size_of::<TSTypeAssertion>() == 28usize);
    assert!(align_of::<TSTypeAssertion>() == 4usize);
    assert!(offset_of!(TSTypeAssertion, node_id) == 0usize);
    assert!(offset_of!(TSTypeAssertion, span) == 4usize);
    assert!(offset_of!(TSTypeAssertion, expression) == 12usize);
    assert!(offset_of!(TSTypeAssertion, type_annotation) == 20usize);

    assert!(size_of::<TSImportEqualsDeclaration>() == 48usize);
    assert!(align_of::<TSImportEqualsDeclaration>() == 4usize);
    assert!(offset_of!(TSImportEqualsDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSImportEqualsDeclaration, span) == 4usize);
    assert!(offset_of!(TSImportEqualsDeclaration, id) == 12usize);
    assert!(offset_of!(TSImportEqualsDeclaration, module_reference) == 36usize);
    assert!(offset_of!(TSImportEqualsDeclaration, import_kind) == 44usize);

    assert!(size_of::<TSModuleReference>() == 8usize);
    assert!(align_of::<TSModuleReference>() == 4usize);

    assert!(size_of::<TSExternalModuleReference>() == 32usize);
    assert!(align_of::<TSExternalModuleReference>() == 4usize);
    assert!(offset_of!(TSExternalModuleReference, node_id) == 0usize);
    assert!(offset_of!(TSExternalModuleReference, span) == 4usize);
    assert!(offset_of!(TSExternalModuleReference, expression) == 12usize);

    assert!(size_of::<TSNonNullExpression>() == 20usize);
    assert!(align_of::<TSNonNullExpression>() == 4usize);
    assert!(offset_of!(TSNonNullExpression, node_id) == 0usize);
    assert!(offset_of!(TSNonNullExpression, span) == 4usize);
    assert!(offset_of!(TSNonNullExpression, expression) == 12usize);

    assert!(size_of::<Decorator>() == 20usize);
    assert!(align_of::<Decorator>() == 4usize);
    assert!(offset_of!(Decorator, node_id) == 0usize);
    assert!(offset_of!(Decorator, span) == 4usize);
    assert!(offset_of!(Decorator, expression) == 12usize);

    assert!(size_of::<TSExportAssignment>() == 20usize);
    assert!(align_of::<TSExportAssignment>() == 4usize);
    assert!(offset_of!(TSExportAssignment, node_id) == 0usize);
    assert!(offset_of!(TSExportAssignment, span) == 4usize);
    assert!(offset_of!(TSExportAssignment, expression) == 12usize);

    assert!(size_of::<TSNamespaceExportDeclaration>() == 32usize);
    assert!(align_of::<TSNamespaceExportDeclaration>() == 4usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, node_id) == 0usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, span) == 4usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, id) == 12usize);

    assert!(size_of::<TSInstantiationExpression>() == 24usize);
    assert!(align_of::<TSInstantiationExpression>() == 4usize);
    assert!(offset_of!(TSInstantiationExpression, node_id) == 0usize);
    assert!(offset_of!(TSInstantiationExpression, span) == 4usize);
    assert!(offset_of!(TSInstantiationExpression, expression) == 12usize);
    assert!(offset_of!(TSInstantiationExpression, type_parameters) == 20usize);

    assert!(size_of::<ImportOrExportKind>() == 1usize);
    assert!(align_of::<ImportOrExportKind>() == 1usize);

    assert!(size_of::<JSDocNullableType>() == 24usize);
    assert!(align_of::<JSDocNullableType>() == 4usize);
    assert!(offset_of!(JSDocNullableType, node_id) == 0usize);
    assert!(offset_of!(JSDocNullableType, span) == 4usize);
    assert!(offset_of!(JSDocNullableType, type_annotation) == 12usize);
    assert!(offset_of!(JSDocNullableType, postfix) == 20usize);

    assert!(size_of::<JSDocNonNullableType>() == 24usize);
    assert!(align_of::<JSDocNonNullableType>() == 4usize);
    assert!(offset_of!(JSDocNonNullableType, node_id) == 0usize);
    assert!(offset_of!(JSDocNonNullableType, span) == 4usize);
    assert!(offset_of!(JSDocNonNullableType, type_annotation) == 12usize);
    assert!(offset_of!(JSDocNonNullableType, postfix) == 20usize);

    assert!(size_of::<JSDocUnknownType>() == 12usize);
    assert!(align_of::<JSDocUnknownType>() == 4usize);
    assert!(offset_of!(JSDocUnknownType, node_id) == 0usize);
    assert!(offset_of!(JSDocUnknownType, span) == 4usize);

    assert!(size_of::<JSXElement>() == 36usize);
    assert!(align_of::<JSXElement>() == 4usize);
    assert!(offset_of!(JSXElement, node_id) == 0usize);
    assert!(offset_of!(JSXElement, span) == 4usize);
    assert!(offset_of!(JSXElement, opening_element) == 12usize);
    assert!(offset_of!(JSXElement, closing_element) == 16usize);
    assert!(offset_of!(JSXElement, children) == 20usize);

    assert!(size_of::<JSXOpeningElement>() == 44usize);
    assert!(align_of::<JSXOpeningElement>() == 4usize);
    assert!(offset_of!(JSXOpeningElement, node_id) == 0usize);
    assert!(offset_of!(JSXOpeningElement, span) == 4usize);
    assert!(offset_of!(JSXOpeningElement, self_closing) == 12usize);
    assert!(offset_of!(JSXOpeningElement, name) == 16usize);
    assert!(offset_of!(JSXOpeningElement, attributes) == 24usize);
    assert!(offset_of!(JSXOpeningElement, type_parameters) == 40usize);

    assert!(size_of::<JSXClosingElement>() == 20usize);
    assert!(align_of::<JSXClosingElement>() == 4usize);
    assert!(offset_of!(JSXClosingElement, node_id) == 0usize);
    assert!(offset_of!(JSXClosingElement, span) == 4usize);
    assert!(offset_of!(JSXClosingElement, name) == 12usize);

    assert!(size_of::<JSXFragment>() == 44usize);
    assert!(align_of::<JSXFragment>() == 4usize);
    assert!(offset_of!(JSXFragment, node_id) == 0usize);
    assert!(offset_of!(JSXFragment, span) == 4usize);
    assert!(offset_of!(JSXFragment, opening_fragment) == 12usize);
    assert!(offset_of!(JSXFragment, closing_fragment) == 20usize);
    assert!(offset_of!(JSXFragment, children) == 28usize);

    assert!(size_of::<JSXOpeningFragment>() == 8usize);
    assert!(align_of::<JSXOpeningFragment>() == 4usize);
    assert!(offset_of!(JSXOpeningFragment, span) == 0usize);

    assert!(size_of::<JSXClosingFragment>() == 8usize);
    assert!(align_of::<JSXClosingFragment>() == 4usize);
    assert!(offset_of!(JSXClosingFragment, span) == 0usize);

    assert!(size_of::<JSXElementName>() == 8usize);
    assert!(align_of::<JSXElementName>() == 4usize);

    assert!(size_of::<JSXNamespacedName>() == 52usize);
    assert!(align_of::<JSXNamespacedName>() == 4usize);
    assert!(offset_of!(JSXNamespacedName, node_id) == 0usize);
    assert!(offset_of!(JSXNamespacedName, span) == 4usize);
    assert!(offset_of!(JSXNamespacedName, namespace) == 12usize);
    assert!(offset_of!(JSXNamespacedName, property) == 32usize);

    assert!(size_of::<JSXMemberExpression>() == 40usize);
    assert!(align_of::<JSXMemberExpression>() == 4usize);
    assert!(offset_of!(JSXMemberExpression, node_id) == 0usize);
    assert!(offset_of!(JSXMemberExpression, span) == 4usize);
    assert!(offset_of!(JSXMemberExpression, object) == 12usize);
    assert!(offset_of!(JSXMemberExpression, property) == 20usize);

    assert!(size_of::<JSXMemberExpressionObject>() == 8usize);
    assert!(align_of::<JSXMemberExpressionObject>() == 4usize);

    assert!(size_of::<JSXExpressionContainer>() == 28usize);
    assert!(align_of::<JSXExpressionContainer>() == 4usize);
    assert!(offset_of!(JSXExpressionContainer, node_id) == 0usize);
    assert!(offset_of!(JSXExpressionContainer, span) == 4usize);
    assert!(offset_of!(JSXExpressionContainer, expression) == 12usize);

    assert!(size_of::<JSXExpression>() == 16usize);
    assert!(align_of::<JSXExpression>() == 4usize);

    assert!(size_of::<JSXEmptyExpression>() == 12usize);
    assert!(align_of::<JSXEmptyExpression>() == 4usize);
    assert!(offset_of!(JSXEmptyExpression, node_id) == 0usize);
    assert!(offset_of!(JSXEmptyExpression, span) == 4usize);

    assert!(size_of::<JSXAttributeItem>() == 8usize);
    assert!(align_of::<JSXAttributeItem>() == 4usize);

    assert!(size_of::<JSXAttribute>() == 28usize);
    assert!(align_of::<JSXAttribute>() == 4usize);
    assert!(offset_of!(JSXAttribute, node_id) == 0usize);
    assert!(offset_of!(JSXAttribute, span) == 4usize);
    assert!(offset_of!(JSXAttribute, name) == 12usize);
    assert!(offset_of!(JSXAttribute, value) == 20usize);

    assert!(size_of::<JSXSpreadAttribute>() == 20usize);
    assert!(align_of::<JSXSpreadAttribute>() == 4usize);
    assert!(offset_of!(JSXSpreadAttribute, node_id) == 0usize);
    assert!(offset_of!(JSXSpreadAttribute, span) == 4usize);
    assert!(offset_of!(JSXSpreadAttribute, argument) == 12usize);

    assert!(size_of::<JSXAttributeName>() == 8usize);
    assert!(align_of::<JSXAttributeName>() == 4usize);

    assert!(size_of::<JSXAttributeValue>() == 8usize);
    assert!(align_of::<JSXAttributeValue>() == 4usize);

    assert!(size_of::<JSXIdentifier>() == 20usize);
    assert!(align_of::<JSXIdentifier>() == 4usize);
    assert!(offset_of!(JSXIdentifier, node_id) == 0usize);
    assert!(offset_of!(JSXIdentifier, span) == 4usize);
    assert!(offset_of!(JSXIdentifier, name) == 12usize);

    assert!(size_of::<JSXChild>() == 8usize);
    assert!(align_of::<JSXChild>() == 4usize);

    assert!(size_of::<JSXSpreadChild>() == 20usize);
    assert!(align_of::<JSXSpreadChild>() == 4usize);
    assert!(offset_of!(JSXSpreadChild, node_id) == 0usize);
    assert!(offset_of!(JSXSpreadChild, span) == 4usize);
    assert!(offset_of!(JSXSpreadChild, expression) == 12usize);

    assert!(size_of::<JSXText>() == 20usize);
    assert!(align_of::<JSXText>() == 4usize);
    assert!(offset_of!(JSXText, node_id) == 0usize);
    assert!(offset_of!(JSXText, span) == 4usize);
    assert!(offset_of!(JSXText, value) == 12usize);

    assert!(size_of::<NodeId>() == 4usize);
    assert!(align_of::<NodeId>() == 4usize);

    assert!(size_of::<NumberBase>() == 1usize);
    assert!(align_of::<NumberBase>() == 1usize);

    assert!(size_of::<BigintBase>() == 1usize);
    assert!(align_of::<BigintBase>() == 1usize);

    assert!(size_of::<AssignmentOperator>() == 1usize);
    assert!(align_of::<AssignmentOperator>() == 1usize);

    assert!(size_of::<BinaryOperator>() == 1usize);
    assert!(align_of::<BinaryOperator>() == 1usize);

    assert!(size_of::<LogicalOperator>() == 1usize);
    assert!(align_of::<LogicalOperator>() == 1usize);

    assert!(size_of::<UnaryOperator>() == 1usize);
    assert!(align_of::<UnaryOperator>() == 1usize);

    assert!(size_of::<UpdateOperator>() == 1usize);
    assert!(align_of::<UpdateOperator>() == 1usize);

    assert!(size_of::<Span>() == 8usize);
    assert!(align_of::<Span>() == 4usize);
    assert!(offset_of!(Span, start) == 0usize);
    assert!(offset_of!(Span, end) == 4usize);

    assert!(size_of::<SourceType>() == 3usize);
    assert!(align_of::<SourceType>() == 1usize);

    assert!(size_of::<Language>() == 1usize);
    assert!(align_of::<Language>() == 1usize);

    assert!(size_of::<ModuleKind>() == 1usize);
    assert!(align_of::<ModuleKind>() == 1usize);

    assert!(size_of::<LanguageVariant>() == 1usize);
    assert!(align_of::<LanguageVariant>() == 1usize);

    assert!(size_of::<RegularExpression>() == 56usize);
    assert!(align_of::<RegularExpression>() == 4usize);
    assert!(offset_of!(RegularExpression, span) == 0usize);
    assert!(offset_of!(RegularExpression, pattern) == 8usize);
    assert!(offset_of!(RegularExpression, flags) == 40usize);

    assert!(size_of::<Flags>() == 16usize);
    assert!(align_of::<Flags>() == 4usize);
    assert!(offset_of!(Flags, span) == 0usize);
    assert!(offset_of!(Flags, global) == 8usize);
    assert!(offset_of!(Flags, ignore_case) == 9usize);
    assert!(offset_of!(Flags, multiline) == 10usize);
    assert!(offset_of!(Flags, unicode) == 11usize);
    assert!(offset_of!(Flags, sticky) == 12usize);
    assert!(offset_of!(Flags, dot_all) == 13usize);
    assert!(offset_of!(Flags, has_indices) == 14usize);
    assert!(offset_of!(Flags, unicode_sets) == 15usize);

    assert!(size_of::<Pattern>() == 32usize);
    assert!(align_of::<Pattern>() == 4usize);
    assert!(offset_of!(Pattern, span) == 0usize);
    assert!(offset_of!(Pattern, body) == 8usize);

    assert!(size_of::<Disjunction>() == 24usize);
    assert!(align_of::<Disjunction>() == 4usize);
    assert!(offset_of!(Disjunction, span) == 0usize);
    assert!(offset_of!(Disjunction, body) == 8usize);

    assert!(size_of::<Alternative>() == 24usize);
    assert!(align_of::<Alternative>() == 4usize);
    assert!(offset_of!(Alternative, span) == 0usize);
    assert!(offset_of!(Alternative, body) == 8usize);

    assert!(size_of::<Term>() == 20usize);
    assert!(align_of::<Term>() == 4usize);

    assert!(size_of::<BoundaryAssertion>() == 12usize);
    assert!(align_of::<BoundaryAssertion>() == 4usize);
    assert!(offset_of!(BoundaryAssertion, span) == 0usize);
    assert!(offset_of!(BoundaryAssertion, kind) == 8usize);

    assert!(size_of::<BoundaryAssertionKind>() == 1usize);
    assert!(align_of::<BoundaryAssertionKind>() == 1usize);

    assert!(size_of::<LookAroundAssertion>() == 36usize);
    assert!(align_of::<LookAroundAssertion>() == 4usize);
    assert!(offset_of!(LookAroundAssertion, span) == 0usize);
    assert!(offset_of!(LookAroundAssertion, kind) == 8usize);
    assert!(offset_of!(LookAroundAssertion, body) == 12usize);

    assert!(size_of::<LookAroundAssertionKind>() == 1usize);
    assert!(align_of::<LookAroundAssertionKind>() == 1usize);

    assert!(size_of::<Quantifier>() == 56usize);
    assert!(align_of::<Quantifier>() == 8usize);
    assert!(offset_of!(Quantifier, span) == 0usize);
    assert!(offset_of!(Quantifier, min) == 8usize);
    assert!(offset_of!(Quantifier, max) == 16usize);
    assert!(offset_of!(Quantifier, greedy) == 32usize);
    assert!(offset_of!(Quantifier, body) == 36usize);

    assert!(size_of::<Character>() == 16usize);
    assert!(align_of::<Character>() == 4usize);
    assert!(offset_of!(Character, span) == 0usize);
    assert!(offset_of!(Character, kind) == 8usize);
    assert!(offset_of!(Character, value) == 12usize);

    assert!(size_of::<CharacterKind>() == 1usize);
    assert!(align_of::<CharacterKind>() == 1usize);

    assert!(size_of::<CharacterClassEscape>() == 12usize);
    assert!(align_of::<CharacterClassEscape>() == 4usize);
    assert!(offset_of!(CharacterClassEscape, span) == 0usize);
    assert!(offset_of!(CharacterClassEscape, kind) == 8usize);

    assert!(size_of::<CharacterClassEscapeKind>() == 1usize);
    assert!(align_of::<CharacterClassEscapeKind>() == 1usize);

    assert!(size_of::<UnicodePropertyEscape>() == 28usize);
    assert!(align_of::<UnicodePropertyEscape>() == 4usize);
    assert!(offset_of!(UnicodePropertyEscape, span) == 0usize);
    assert!(offset_of!(UnicodePropertyEscape, negative) == 8usize);
    assert!(offset_of!(UnicodePropertyEscape, strings) == 9usize);
    assert!(offset_of!(UnicodePropertyEscape, name) == 12usize);
    assert!(offset_of!(UnicodePropertyEscape, value) == 20usize);

    assert!(size_of::<Dot>() == 8usize);
    assert!(align_of::<Dot>() == 4usize);
    assert!(offset_of!(Dot, span) == 0usize);

    assert!(size_of::<CharacterClass>() == 28usize);
    assert!(align_of::<CharacterClass>() == 4usize);
    assert!(offset_of!(CharacterClass, span) == 0usize);
    assert!(offset_of!(CharacterClass, negative) == 8usize);
    assert!(offset_of!(CharacterClass, strings) == 9usize);
    assert!(offset_of!(CharacterClass, kind) == 10usize);
    assert!(offset_of!(CharacterClass, body) == 12usize);

    assert!(size_of::<CharacterClassContentsKind>() == 1usize);
    assert!(align_of::<CharacterClassContentsKind>() == 1usize);

    assert!(size_of::<CharacterClassContents>() == 20usize);
    assert!(align_of::<CharacterClassContents>() == 4usize);

    assert!(size_of::<CharacterClassRange>() == 40usize);
    assert!(align_of::<CharacterClassRange>() == 4usize);
    assert!(offset_of!(CharacterClassRange, span) == 0usize);
    assert!(offset_of!(CharacterClassRange, min) == 8usize);
    assert!(offset_of!(CharacterClassRange, max) == 24usize);

    assert!(size_of::<ClassStringDisjunction>() == 28usize);
    assert!(align_of::<ClassStringDisjunction>() == 4usize);
    assert!(offset_of!(ClassStringDisjunction, span) == 0usize);
    assert!(offset_of!(ClassStringDisjunction, strings) == 8usize);
    assert!(offset_of!(ClassStringDisjunction, body) == 12usize);

    assert!(size_of::<ClassString>() == 28usize);
    assert!(align_of::<ClassString>() == 4usize);
    assert!(offset_of!(ClassString, span) == 0usize);
    assert!(offset_of!(ClassString, strings) == 8usize);
    assert!(offset_of!(ClassString, body) == 12usize);

    assert!(size_of::<CapturingGroup>() == 40usize);
    assert!(align_of::<CapturingGroup>() == 4usize);
    assert!(offset_of!(CapturingGroup, span) == 0usize);
    assert!(offset_of!(CapturingGroup, name) == 8usize);
    assert!(offset_of!(CapturingGroup, body) == 16usize);

    assert!(size_of::<IgnoreGroup>() == 40usize);
    assert!(align_of::<IgnoreGroup>() == 4usize);
    assert!(offset_of!(IgnoreGroup, span) == 0usize);
    assert!(offset_of!(IgnoreGroup, enabling_modifiers) == 8usize);
    assert!(offset_of!(IgnoreGroup, disabling_modifiers) == 11usize);
    assert!(offset_of!(IgnoreGroup, body) == 16usize);

    assert!(size_of::<ModifierFlags>() == 3usize);
    assert!(align_of::<ModifierFlags>() == 1usize);
    assert!(offset_of!(ModifierFlags, ignore_case) == 0usize);
    assert!(offset_of!(ModifierFlags, sticky) == 1usize);
    assert!(offset_of!(ModifierFlags, multiline) == 2usize);

    assert!(size_of::<IndexedReference>() == 12usize);
    assert!(align_of::<IndexedReference>() == 4usize);
    assert!(offset_of!(IndexedReference, span) == 0usize);
    assert!(offset_of!(IndexedReference, index) == 8usize);

    assert!(size_of::<NamedReference>() == 16usize);
    assert!(align_of::<NamedReference>() == 4usize);
    assert!(offset_of!(NamedReference, span) == 0usize);
    assert!(offset_of!(NamedReference, name) == 8usize);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
