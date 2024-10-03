// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`

use std::mem::{align_of, offset_of, size_of};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

#[allow(clippy::wildcard_imports)]
use oxc_regular_expression::ast::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<BooleanLiteral>() == 12usize);
    assert!(align_of::<BooleanLiteral>() == 4usize);
    assert!(offset_of!(BooleanLiteral, span) == 0usize);
    assert!(offset_of!(BooleanLiteral, value) == 8usize);

    assert!(size_of::<NullLiteral>() == 8usize);
    assert!(align_of::<NullLiteral>() == 4usize);
    assert!(offset_of!(NullLiteral, span) == 0usize);

    assert!(size_of::<NumericLiteral>() == 40usize);
    assert!(align_of::<NumericLiteral>() == 8usize);
    assert!(offset_of!(NumericLiteral, span) == 0usize);
    assert!(offset_of!(NumericLiteral, value) == 8usize);
    assert!(offset_of!(NumericLiteral, raw) == 16usize);
    assert!(offset_of!(NumericLiteral, base) == 32usize);

    assert!(size_of::<BigIntLiteral>() == 32usize);
    assert!(align_of::<BigIntLiteral>() == 8usize);
    assert!(offset_of!(BigIntLiteral, span) == 0usize);
    assert!(offset_of!(BigIntLiteral, raw) == 8usize);
    assert!(offset_of!(BigIntLiteral, base) == 24usize);

    assert!(size_of::<RegExpLiteral>() == 40usize);
    assert!(align_of::<RegExpLiteral>() == 8usize);
    assert!(offset_of!(RegExpLiteral, span) == 0usize);
    assert!(offset_of!(RegExpLiteral, value) == 8usize);
    assert!(offset_of!(RegExpLiteral, regex) == 8usize);

    assert!(size_of::<RegExp>() == 32usize);
    assert!(align_of::<RegExp>() == 8usize);
    assert!(offset_of!(RegExp, pattern) == 0usize);
    assert!(offset_of!(RegExp, flags) == 24usize);

    assert!(size_of::<RegExpPattern>() == 24usize);
    assert!(align_of::<RegExpPattern>() == 8usize);

    assert!(size_of::<EmptyObject>() == 0usize);
    assert!(align_of::<EmptyObject>() == 1usize);

    assert!(size_of::<StringLiteral>() == 24usize);
    assert!(align_of::<StringLiteral>() == 8usize);
    assert!(offset_of!(StringLiteral, span) == 0usize);
    assert!(offset_of!(StringLiteral, value) == 8usize);

    assert!(size_of::<Program>() == 112usize);
    assert!(align_of::<Program>() == 8usize);
    assert!(offset_of!(Program, span) == 0usize);
    assert!(offset_of!(Program, source_type) == 8usize);
    assert!(offset_of!(Program, hashbang) == 16usize);
    assert!(offset_of!(Program, directives) == 40usize);
    assert!(offset_of!(Program, body) == 72usize);
    assert!(offset_of!(Program, scope_id) == 104usize);

    assert!(size_of::<Expression>() == 16usize);
    assert!(align_of::<Expression>() == 8usize);

    assert!(size_of::<IdentifierName>() == 24usize);
    assert!(align_of::<IdentifierName>() == 8usize);
    assert!(offset_of!(IdentifierName, span) == 0usize);
    assert!(offset_of!(IdentifierName, name) == 8usize);

    assert!(size_of::<IdentifierReference>() == 32usize);
    assert!(align_of::<IdentifierReference>() == 8usize);
    assert!(offset_of!(IdentifierReference, span) == 0usize);
    assert!(offset_of!(IdentifierReference, name) == 8usize);
    assert!(offset_of!(IdentifierReference, reference_id) == 24usize);

    assert!(size_of::<BindingIdentifier>() == 32usize);
    assert!(align_of::<BindingIdentifier>() == 8usize);
    assert!(offset_of!(BindingIdentifier, span) == 0usize);
    assert!(offset_of!(BindingIdentifier, name) == 8usize);
    assert!(offset_of!(BindingIdentifier, symbol_id) == 24usize);

    assert!(size_of::<LabelIdentifier>() == 24usize);
    assert!(align_of::<LabelIdentifier>() == 8usize);
    assert!(offset_of!(LabelIdentifier, span) == 0usize);
    assert!(offset_of!(LabelIdentifier, name) == 8usize);

    assert!(size_of::<ThisExpression>() == 8usize);
    assert!(align_of::<ThisExpression>() == 4usize);
    assert!(offset_of!(ThisExpression, span) == 0usize);

    assert!(size_of::<ArrayExpression>() == 56usize);
    assert!(align_of::<ArrayExpression>() == 8usize);
    assert!(offset_of!(ArrayExpression, span) == 0usize);
    assert!(offset_of!(ArrayExpression, elements) == 8usize);
    assert!(offset_of!(ArrayExpression, trailing_comma) == 40usize);

    assert!(size_of::<ArrayExpressionElement>() == 16usize);
    assert!(align_of::<ArrayExpressionElement>() == 8usize);

    assert!(size_of::<Elision>() == 8usize);
    assert!(align_of::<Elision>() == 4usize);
    assert!(offset_of!(Elision, span) == 0usize);

    assert!(size_of::<ObjectExpression>() == 56usize);
    assert!(align_of::<ObjectExpression>() == 8usize);
    assert!(offset_of!(ObjectExpression, span) == 0usize);
    assert!(offset_of!(ObjectExpression, properties) == 8usize);
    assert!(offset_of!(ObjectExpression, trailing_comma) == 40usize);

    assert!(size_of::<ObjectPropertyKind>() == 16usize);
    assert!(align_of::<ObjectPropertyKind>() == 8usize);

    assert!(size_of::<ObjectProperty>() == 72usize);
    assert!(align_of::<ObjectProperty>() == 8usize);
    assert!(offset_of!(ObjectProperty, span) == 0usize);
    assert!(offset_of!(ObjectProperty, kind) == 8usize);
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

    assert!(size_of::<TemplateLiteral>() == 72usize);
    assert!(align_of::<TemplateLiteral>() == 8usize);
    assert!(offset_of!(TemplateLiteral, span) == 0usize);
    assert!(offset_of!(TemplateLiteral, quasis) == 8usize);
    assert!(offset_of!(TemplateLiteral, expressions) == 40usize);

    assert!(size_of::<TaggedTemplateExpression>() == 104usize);
    assert!(align_of::<TaggedTemplateExpression>() == 8usize);
    assert!(offset_of!(TaggedTemplateExpression, span) == 0usize);
    assert!(offset_of!(TaggedTemplateExpression, tag) == 8usize);
    assert!(offset_of!(TaggedTemplateExpression, quasi) == 24usize);
    assert!(offset_of!(TaggedTemplateExpression, type_parameters) == 96usize);

    assert!(size_of::<TemplateElement>() == 48usize);
    assert!(align_of::<TemplateElement>() == 8usize);
    assert!(offset_of!(TemplateElement, span) == 0usize);
    assert!(offset_of!(TemplateElement, tail) == 8usize);
    assert!(offset_of!(TemplateElement, value) == 16usize);

    assert!(size_of::<TemplateElementValue>() == 32usize);
    assert!(align_of::<TemplateElementValue>() == 8usize);
    assert!(offset_of!(TemplateElementValue, raw) == 0usize);
    assert!(offset_of!(TemplateElementValue, cooked) == 16usize);

    assert!(size_of::<MemberExpression>() == 16usize);
    assert!(align_of::<MemberExpression>() == 8usize);

    assert!(size_of::<ComputedMemberExpression>() == 48usize);
    assert!(align_of::<ComputedMemberExpression>() == 8usize);
    assert!(offset_of!(ComputedMemberExpression, span) == 0usize);
    assert!(offset_of!(ComputedMemberExpression, object) == 8usize);
    assert!(offset_of!(ComputedMemberExpression, expression) == 24usize);
    assert!(offset_of!(ComputedMemberExpression, optional) == 40usize);

    assert!(size_of::<StaticMemberExpression>() == 56usize);
    assert!(align_of::<StaticMemberExpression>() == 8usize);
    assert!(offset_of!(StaticMemberExpression, span) == 0usize);
    assert!(offset_of!(StaticMemberExpression, object) == 8usize);
    assert!(offset_of!(StaticMemberExpression, property) == 24usize);
    assert!(offset_of!(StaticMemberExpression, optional) == 48usize);

    assert!(size_of::<PrivateFieldExpression>() == 56usize);
    assert!(align_of::<PrivateFieldExpression>() == 8usize);
    assert!(offset_of!(PrivateFieldExpression, span) == 0usize);
    assert!(offset_of!(PrivateFieldExpression, object) == 8usize);
    assert!(offset_of!(PrivateFieldExpression, field) == 24usize);
    assert!(offset_of!(PrivateFieldExpression, optional) == 48usize);

    assert!(size_of::<CallExpression>() == 72usize);
    assert!(align_of::<CallExpression>() == 8usize);
    assert!(offset_of!(CallExpression, span) == 0usize);
    assert!(offset_of!(CallExpression, callee) == 8usize);
    assert!(offset_of!(CallExpression, type_parameters) == 24usize);
    assert!(offset_of!(CallExpression, arguments) == 32usize);
    assert!(offset_of!(CallExpression, optional) == 64usize);

    assert!(size_of::<NewExpression>() == 64usize);
    assert!(align_of::<NewExpression>() == 8usize);
    assert!(offset_of!(NewExpression, span) == 0usize);
    assert!(offset_of!(NewExpression, callee) == 8usize);
    assert!(offset_of!(NewExpression, arguments) == 24usize);
    assert!(offset_of!(NewExpression, type_parameters) == 56usize);

    assert!(size_of::<MetaProperty>() == 56usize);
    assert!(align_of::<MetaProperty>() == 8usize);
    assert!(offset_of!(MetaProperty, span) == 0usize);
    assert!(offset_of!(MetaProperty, meta) == 8usize);
    assert!(offset_of!(MetaProperty, property) == 32usize);

    assert!(size_of::<SpreadElement>() == 24usize);
    assert!(align_of::<SpreadElement>() == 8usize);
    assert!(offset_of!(SpreadElement, span) == 0usize);
    assert!(offset_of!(SpreadElement, argument) == 8usize);

    assert!(size_of::<Argument>() == 16usize);
    assert!(align_of::<Argument>() == 8usize);

    assert!(size_of::<UpdateExpression>() == 32usize);
    assert!(align_of::<UpdateExpression>() == 8usize);
    assert!(offset_of!(UpdateExpression, span) == 0usize);
    assert!(offset_of!(UpdateExpression, operator) == 8usize);
    assert!(offset_of!(UpdateExpression, prefix) == 9usize);
    assert!(offset_of!(UpdateExpression, argument) == 16usize);

    assert!(size_of::<UnaryExpression>() == 32usize);
    assert!(align_of::<UnaryExpression>() == 8usize);
    assert!(offset_of!(UnaryExpression, span) == 0usize);
    assert!(offset_of!(UnaryExpression, operator) == 8usize);
    assert!(offset_of!(UnaryExpression, argument) == 16usize);

    assert!(size_of::<BinaryExpression>() == 48usize);
    assert!(align_of::<BinaryExpression>() == 8usize);
    assert!(offset_of!(BinaryExpression, span) == 0usize);
    assert!(offset_of!(BinaryExpression, left) == 8usize);
    assert!(offset_of!(BinaryExpression, operator) == 24usize);
    assert!(offset_of!(BinaryExpression, right) == 32usize);

    assert!(size_of::<PrivateInExpression>() == 56usize);
    assert!(align_of::<PrivateInExpression>() == 8usize);
    assert!(offset_of!(PrivateInExpression, span) == 0usize);
    assert!(offset_of!(PrivateInExpression, left) == 8usize);
    assert!(offset_of!(PrivateInExpression, operator) == 32usize);
    assert!(offset_of!(PrivateInExpression, right) == 40usize);

    assert!(size_of::<LogicalExpression>() == 48usize);
    assert!(align_of::<LogicalExpression>() == 8usize);
    assert!(offset_of!(LogicalExpression, span) == 0usize);
    assert!(offset_of!(LogicalExpression, left) == 8usize);
    assert!(offset_of!(LogicalExpression, operator) == 24usize);
    assert!(offset_of!(LogicalExpression, right) == 32usize);

    assert!(size_of::<ConditionalExpression>() == 56usize);
    assert!(align_of::<ConditionalExpression>() == 8usize);
    assert!(offset_of!(ConditionalExpression, span) == 0usize);
    assert!(offset_of!(ConditionalExpression, test) == 8usize);
    assert!(offset_of!(ConditionalExpression, consequent) == 24usize);
    assert!(offset_of!(ConditionalExpression, alternate) == 40usize);

    assert!(size_of::<AssignmentExpression>() == 48usize);
    assert!(align_of::<AssignmentExpression>() == 8usize);
    assert!(offset_of!(AssignmentExpression, span) == 0usize);
    assert!(offset_of!(AssignmentExpression, operator) == 8usize);
    assert!(offset_of!(AssignmentExpression, left) == 16usize);
    assert!(offset_of!(AssignmentExpression, right) == 32usize);

    assert!(size_of::<AssignmentTarget>() == 16usize);
    assert!(align_of::<AssignmentTarget>() == 8usize);

    assert!(size_of::<SimpleAssignmentTarget>() == 16usize);
    assert!(align_of::<SimpleAssignmentTarget>() == 8usize);

    assert!(size_of::<AssignmentTargetPattern>() == 16usize);
    assert!(align_of::<AssignmentTargetPattern>() == 8usize);

    assert!(size_of::<ArrayAssignmentTarget>() == 80usize);
    assert!(align_of::<ArrayAssignmentTarget>() == 8usize);
    assert!(offset_of!(ArrayAssignmentTarget, span) == 0usize);
    assert!(offset_of!(ArrayAssignmentTarget, elements) == 8usize);
    assert!(offset_of!(ArrayAssignmentTarget, rest) == 40usize);
    assert!(offset_of!(ArrayAssignmentTarget, trailing_comma) == 64usize);

    assert!(size_of::<ObjectAssignmentTarget>() == 64usize);
    assert!(align_of::<ObjectAssignmentTarget>() == 8usize);
    assert!(offset_of!(ObjectAssignmentTarget, span) == 0usize);
    assert!(offset_of!(ObjectAssignmentTarget, properties) == 8usize);
    assert!(offset_of!(ObjectAssignmentTarget, rest) == 40usize);

    assert!(size_of::<AssignmentTargetRest>() == 24usize);
    assert!(align_of::<AssignmentTargetRest>() == 8usize);
    assert!(offset_of!(AssignmentTargetRest, span) == 0usize);
    assert!(offset_of!(AssignmentTargetRest, target) == 8usize);

    assert!(size_of::<AssignmentTargetMaybeDefault>() == 16usize);
    assert!(align_of::<AssignmentTargetMaybeDefault>() == 8usize);

    assert!(size_of::<AssignmentTargetWithDefault>() == 40usize);
    assert!(align_of::<AssignmentTargetWithDefault>() == 8usize);
    assert!(offset_of!(AssignmentTargetWithDefault, span) == 0usize);
    assert!(offset_of!(AssignmentTargetWithDefault, binding) == 8usize);
    assert!(offset_of!(AssignmentTargetWithDefault, init) == 24usize);

    assert!(size_of::<AssignmentTargetProperty>() == 16usize);
    assert!(align_of::<AssignmentTargetProperty>() == 8usize);

    assert!(size_of::<AssignmentTargetPropertyIdentifier>() == 56usize);
    assert!(align_of::<AssignmentTargetPropertyIdentifier>() == 8usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, span) == 0usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, binding) == 8usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, init) == 40usize);

    assert!(size_of::<AssignmentTargetPropertyProperty>() == 40usize);
    assert!(align_of::<AssignmentTargetPropertyProperty>() == 8usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, span) == 0usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, name) == 8usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, binding) == 24usize);

    assert!(size_of::<SequenceExpression>() == 40usize);
    assert!(align_of::<SequenceExpression>() == 8usize);
    assert!(offset_of!(SequenceExpression, span) == 0usize);
    assert!(offset_of!(SequenceExpression, expressions) == 8usize);

    assert!(size_of::<Super>() == 8usize);
    assert!(align_of::<Super>() == 4usize);
    assert!(offset_of!(Super, span) == 0usize);

    assert!(size_of::<AwaitExpression>() == 24usize);
    assert!(align_of::<AwaitExpression>() == 8usize);
    assert!(offset_of!(AwaitExpression, span) == 0usize);
    assert!(offset_of!(AwaitExpression, argument) == 8usize);

    assert!(size_of::<ChainExpression>() == 24usize);
    assert!(align_of::<ChainExpression>() == 8usize);
    assert!(offset_of!(ChainExpression, span) == 0usize);
    assert!(offset_of!(ChainExpression, expression) == 8usize);

    assert!(size_of::<ChainElement>() == 16usize);
    assert!(align_of::<ChainElement>() == 8usize);

    assert!(size_of::<ParenthesizedExpression>() == 24usize);
    assert!(align_of::<ParenthesizedExpression>() == 8usize);
    assert!(offset_of!(ParenthesizedExpression, span) == 0usize);
    assert!(offset_of!(ParenthesizedExpression, expression) == 8usize);

    assert!(size_of::<Statement>() == 16usize);
    assert!(align_of::<Statement>() == 8usize);

    assert!(size_of::<Directive>() == 48usize);
    assert!(align_of::<Directive>() == 8usize);
    assert!(offset_of!(Directive, span) == 0usize);
    assert!(offset_of!(Directive, expression) == 8usize);
    assert!(offset_of!(Directive, directive) == 32usize);

    assert!(size_of::<Hashbang>() == 24usize);
    assert!(align_of::<Hashbang>() == 8usize);
    assert!(offset_of!(Hashbang, span) == 0usize);
    assert!(offset_of!(Hashbang, value) == 8usize);

    assert!(size_of::<BlockStatement>() == 48usize);
    assert!(align_of::<BlockStatement>() == 8usize);
    assert!(offset_of!(BlockStatement, span) == 0usize);
    assert!(offset_of!(BlockStatement, body) == 8usize);
    assert!(offset_of!(BlockStatement, scope_id) == 40usize);

    assert!(size_of::<Declaration>() == 16usize);
    assert!(align_of::<Declaration>() == 8usize);

    assert!(size_of::<VariableDeclaration>() == 56usize);
    assert!(align_of::<VariableDeclaration>() == 8usize);
    assert!(offset_of!(VariableDeclaration, span) == 0usize);
    assert!(offset_of!(VariableDeclaration, kind) == 8usize);
    assert!(offset_of!(VariableDeclaration, declarations) == 16usize);
    assert!(offset_of!(VariableDeclaration, declare) == 48usize);

    assert!(size_of::<VariableDeclarationKind>() == 1usize);
    assert!(align_of::<VariableDeclarationKind>() == 1usize);

    assert!(size_of::<VariableDeclarator>() == 72usize);
    assert!(align_of::<VariableDeclarator>() == 8usize);
    assert!(offset_of!(VariableDeclarator, span) == 0usize);
    assert!(offset_of!(VariableDeclarator, kind) == 8usize);
    assert!(offset_of!(VariableDeclarator, id) == 16usize);
    assert!(offset_of!(VariableDeclarator, init) == 48usize);
    assert!(offset_of!(VariableDeclarator, definite) == 64usize);

    assert!(size_of::<EmptyStatement>() == 8usize);
    assert!(align_of::<EmptyStatement>() == 4usize);
    assert!(offset_of!(EmptyStatement, span) == 0usize);

    assert!(size_of::<ExpressionStatement>() == 24usize);
    assert!(align_of::<ExpressionStatement>() == 8usize);
    assert!(offset_of!(ExpressionStatement, span) == 0usize);
    assert!(offset_of!(ExpressionStatement, expression) == 8usize);

    assert!(size_of::<IfStatement>() == 56usize);
    assert!(align_of::<IfStatement>() == 8usize);
    assert!(offset_of!(IfStatement, span) == 0usize);
    assert!(offset_of!(IfStatement, test) == 8usize);
    assert!(offset_of!(IfStatement, consequent) == 24usize);
    assert!(offset_of!(IfStatement, alternate) == 40usize);

    assert!(size_of::<DoWhileStatement>() == 40usize);
    assert!(align_of::<DoWhileStatement>() == 8usize);
    assert!(offset_of!(DoWhileStatement, span) == 0usize);
    assert!(offset_of!(DoWhileStatement, body) == 8usize);
    assert!(offset_of!(DoWhileStatement, test) == 24usize);

    assert!(size_of::<WhileStatement>() == 40usize);
    assert!(align_of::<WhileStatement>() == 8usize);
    assert!(offset_of!(WhileStatement, span) == 0usize);
    assert!(offset_of!(WhileStatement, test) == 8usize);
    assert!(offset_of!(WhileStatement, body) == 24usize);

    assert!(size_of::<ForStatement>() == 80usize);
    assert!(align_of::<ForStatement>() == 8usize);
    assert!(offset_of!(ForStatement, span) == 0usize);
    assert!(offset_of!(ForStatement, init) == 8usize);
    assert!(offset_of!(ForStatement, test) == 24usize);
    assert!(offset_of!(ForStatement, update) == 40usize);
    assert!(offset_of!(ForStatement, body) == 56usize);
    assert!(offset_of!(ForStatement, scope_id) == 72usize);

    assert!(size_of::<ForStatementInit>() == 16usize);
    assert!(align_of::<ForStatementInit>() == 8usize);

    assert!(size_of::<ForInStatement>() == 64usize);
    assert!(align_of::<ForInStatement>() == 8usize);
    assert!(offset_of!(ForInStatement, span) == 0usize);
    assert!(offset_of!(ForInStatement, left) == 8usize);
    assert!(offset_of!(ForInStatement, right) == 24usize);
    assert!(offset_of!(ForInStatement, body) == 40usize);
    assert!(offset_of!(ForInStatement, scope_id) == 56usize);

    assert!(size_of::<ForStatementLeft>() == 16usize);
    assert!(align_of::<ForStatementLeft>() == 8usize);

    assert!(size_of::<ForOfStatement>() == 72usize);
    assert!(align_of::<ForOfStatement>() == 8usize);
    assert!(offset_of!(ForOfStatement, span) == 0usize);
    assert!(offset_of!(ForOfStatement, r#await) == 8usize);
    assert!(offset_of!(ForOfStatement, left) == 16usize);
    assert!(offset_of!(ForOfStatement, right) == 32usize);
    assert!(offset_of!(ForOfStatement, body) == 48usize);
    assert!(offset_of!(ForOfStatement, scope_id) == 64usize);

    assert!(size_of::<ContinueStatement>() == 32usize);
    assert!(align_of::<ContinueStatement>() == 8usize);
    assert!(offset_of!(ContinueStatement, span) == 0usize);
    assert!(offset_of!(ContinueStatement, label) == 8usize);

    assert!(size_of::<BreakStatement>() == 32usize);
    assert!(align_of::<BreakStatement>() == 8usize);
    assert!(offset_of!(BreakStatement, span) == 0usize);
    assert!(offset_of!(BreakStatement, label) == 8usize);

    assert!(size_of::<ReturnStatement>() == 24usize);
    assert!(align_of::<ReturnStatement>() == 8usize);
    assert!(offset_of!(ReturnStatement, span) == 0usize);
    assert!(offset_of!(ReturnStatement, argument) == 8usize);

    assert!(size_of::<WithStatement>() == 40usize);
    assert!(align_of::<WithStatement>() == 8usize);
    assert!(offset_of!(WithStatement, span) == 0usize);
    assert!(offset_of!(WithStatement, object) == 8usize);
    assert!(offset_of!(WithStatement, body) == 24usize);

    assert!(size_of::<SwitchStatement>() == 64usize);
    assert!(align_of::<SwitchStatement>() == 8usize);
    assert!(offset_of!(SwitchStatement, span) == 0usize);
    assert!(offset_of!(SwitchStatement, discriminant) == 8usize);
    assert!(offset_of!(SwitchStatement, cases) == 24usize);
    assert!(offset_of!(SwitchStatement, scope_id) == 56usize);

    assert!(size_of::<SwitchCase>() == 56usize);
    assert!(align_of::<SwitchCase>() == 8usize);
    assert!(offset_of!(SwitchCase, span) == 0usize);
    assert!(offset_of!(SwitchCase, test) == 8usize);
    assert!(offset_of!(SwitchCase, consequent) == 24usize);

    assert!(size_of::<LabeledStatement>() == 48usize);
    assert!(align_of::<LabeledStatement>() == 8usize);
    assert!(offset_of!(LabeledStatement, span) == 0usize);
    assert!(offset_of!(LabeledStatement, label) == 8usize);
    assert!(offset_of!(LabeledStatement, body) == 32usize);

    assert!(size_of::<ThrowStatement>() == 24usize);
    assert!(align_of::<ThrowStatement>() == 8usize);
    assert!(offset_of!(ThrowStatement, span) == 0usize);
    assert!(offset_of!(ThrowStatement, argument) == 8usize);

    assert!(size_of::<TryStatement>() == 32usize);
    assert!(align_of::<TryStatement>() == 8usize);
    assert!(offset_of!(TryStatement, span) == 0usize);
    assert!(offset_of!(TryStatement, block) == 8usize);
    assert!(offset_of!(TryStatement, handler) == 16usize);
    assert!(offset_of!(TryStatement, finalizer) == 24usize);

    assert!(size_of::<CatchClause>() == 64usize);
    assert!(align_of::<CatchClause>() == 8usize);
    assert!(offset_of!(CatchClause, span) == 0usize);
    assert!(offset_of!(CatchClause, param) == 8usize);
    assert!(offset_of!(CatchClause, body) == 48usize);
    assert!(offset_of!(CatchClause, scope_id) == 56usize);

    assert!(size_of::<CatchParameter>() == 40usize);
    assert!(align_of::<CatchParameter>() == 8usize);
    assert!(offset_of!(CatchParameter, span) == 0usize);
    assert!(offset_of!(CatchParameter, pattern) == 8usize);

    assert!(size_of::<DebuggerStatement>() == 8usize);
    assert!(align_of::<DebuggerStatement>() == 4usize);
    assert!(offset_of!(DebuggerStatement, span) == 0usize);

    assert!(size_of::<BindingPattern>() == 32usize);
    assert!(align_of::<BindingPattern>() == 8usize);
    assert!(offset_of!(BindingPattern, kind) == 0usize);
    assert!(offset_of!(BindingPattern, type_annotation) == 16usize);
    assert!(offset_of!(BindingPattern, optional) == 24usize);

    assert!(size_of::<BindingPatternKind>() == 16usize);
    assert!(align_of::<BindingPatternKind>() == 8usize);

    assert!(size_of::<AssignmentPattern>() == 56usize);
    assert!(align_of::<AssignmentPattern>() == 8usize);
    assert!(offset_of!(AssignmentPattern, span) == 0usize);
    assert!(offset_of!(AssignmentPattern, left) == 8usize);
    assert!(offset_of!(AssignmentPattern, right) == 40usize);

    assert!(size_of::<ObjectPattern>() == 48usize);
    assert!(align_of::<ObjectPattern>() == 8usize);
    assert!(offset_of!(ObjectPattern, span) == 0usize);
    assert!(offset_of!(ObjectPattern, properties) == 8usize);
    assert!(offset_of!(ObjectPattern, rest) == 40usize);

    assert!(size_of::<BindingProperty>() == 64usize);
    assert!(align_of::<BindingProperty>() == 8usize);
    assert!(offset_of!(BindingProperty, span) == 0usize);
    assert!(offset_of!(BindingProperty, key) == 8usize);
    assert!(offset_of!(BindingProperty, value) == 24usize);
    assert!(offset_of!(BindingProperty, shorthand) == 56usize);
    assert!(offset_of!(BindingProperty, computed) == 57usize);

    assert!(size_of::<ArrayPattern>() == 48usize);
    assert!(align_of::<ArrayPattern>() == 8usize);
    assert!(offset_of!(ArrayPattern, span) == 0usize);
    assert!(offset_of!(ArrayPattern, elements) == 8usize);
    assert!(offset_of!(ArrayPattern, rest) == 40usize);

    assert!(size_of::<BindingRestElement>() == 40usize);
    assert!(align_of::<BindingRestElement>() == 8usize);
    assert!(offset_of!(BindingRestElement, span) == 0usize);
    assert!(offset_of!(BindingRestElement, argument) == 8usize);

    assert!(size_of::<Function>() == 104usize);
    assert!(align_of::<Function>() == 8usize);
    assert!(offset_of!(Function, r#type) == 0usize);
    assert!(offset_of!(Function, span) == 4usize);
    assert!(offset_of!(Function, id) == 16usize);
    assert!(offset_of!(Function, generator) == 48usize);
    assert!(offset_of!(Function, r#async) == 49usize);
    assert!(offset_of!(Function, declare) == 50usize);
    assert!(offset_of!(Function, type_parameters) == 56usize);
    assert!(offset_of!(Function, this_param) == 64usize);
    assert!(offset_of!(Function, params) == 72usize);
    assert!(offset_of!(Function, return_type) == 80usize);
    assert!(offset_of!(Function, body) == 88usize);
    assert!(offset_of!(Function, scope_id) == 96usize);

    assert!(size_of::<FunctionType>() == 1usize);
    assert!(align_of::<FunctionType>() == 1usize);

    assert!(size_of::<FormalParameters>() == 56usize);
    assert!(align_of::<FormalParameters>() == 8usize);
    assert!(offset_of!(FormalParameters, span) == 0usize);
    assert!(offset_of!(FormalParameters, kind) == 8usize);
    assert!(offset_of!(FormalParameters, items) == 16usize);
    assert!(offset_of!(FormalParameters, rest) == 48usize);

    assert!(size_of::<FormalParameter>() == 80usize);
    assert!(align_of::<FormalParameter>() == 8usize);
    assert!(offset_of!(FormalParameter, span) == 0usize);
    assert!(offset_of!(FormalParameter, decorators) == 8usize);
    assert!(offset_of!(FormalParameter, pattern) == 40usize);
    assert!(offset_of!(FormalParameter, accessibility) == 72usize);
    assert!(offset_of!(FormalParameter, readonly) == 73usize);
    assert!(offset_of!(FormalParameter, r#override) == 74usize);

    assert!(size_of::<FormalParameterKind>() == 1usize);
    assert!(align_of::<FormalParameterKind>() == 1usize);

    assert!(size_of::<FunctionBody>() == 72usize);
    assert!(align_of::<FunctionBody>() == 8usize);
    assert!(offset_of!(FunctionBody, span) == 0usize);
    assert!(offset_of!(FunctionBody, directives) == 8usize);
    assert!(offset_of!(FunctionBody, statements) == 40usize);

    assert!(size_of::<ArrowFunctionExpression>() == 56usize);
    assert!(align_of::<ArrowFunctionExpression>() == 8usize);
    assert!(offset_of!(ArrowFunctionExpression, span) == 0usize);
    assert!(offset_of!(ArrowFunctionExpression, expression) == 8usize);
    assert!(offset_of!(ArrowFunctionExpression, r#async) == 9usize);
    assert!(offset_of!(ArrowFunctionExpression, type_parameters) == 16usize);
    assert!(offset_of!(ArrowFunctionExpression, params) == 24usize);
    assert!(offset_of!(ArrowFunctionExpression, return_type) == 32usize);
    assert!(offset_of!(ArrowFunctionExpression, body) == 40usize);
    assert!(offset_of!(ArrowFunctionExpression, scope_id) == 48usize);

    assert!(size_of::<YieldExpression>() == 32usize);
    assert!(align_of::<YieldExpression>() == 8usize);
    assert!(offset_of!(YieldExpression, span) == 0usize);
    assert!(offset_of!(YieldExpression, delegate) == 8usize);
    assert!(offset_of!(YieldExpression, argument) == 16usize);

    assert!(size_of::<Class>() == 160usize);
    assert!(align_of::<Class>() == 8usize);
    assert!(offset_of!(Class, r#type) == 0usize);
    assert!(offset_of!(Class, span) == 4usize);
    assert!(offset_of!(Class, decorators) == 16usize);
    assert!(offset_of!(Class, id) == 48usize);
    assert!(offset_of!(Class, type_parameters) == 80usize);
    assert!(offset_of!(Class, super_class) == 88usize);
    assert!(offset_of!(Class, super_type_parameters) == 104usize);
    assert!(offset_of!(Class, implements) == 112usize);
    assert!(offset_of!(Class, body) == 144usize);
    assert!(offset_of!(Class, r#abstract) == 152usize);
    assert!(offset_of!(Class, declare) == 153usize);
    assert!(offset_of!(Class, scope_id) == 156usize);

    assert!(size_of::<ClassType>() == 1usize);
    assert!(align_of::<ClassType>() == 1usize);

    assert!(size_of::<ClassBody>() == 40usize);
    assert!(align_of::<ClassBody>() == 8usize);
    assert!(offset_of!(ClassBody, span) == 0usize);
    assert!(offset_of!(ClassBody, body) == 8usize);

    assert!(size_of::<ClassElement>() == 16usize);
    assert!(align_of::<ClassElement>() == 8usize);

    assert!(size_of::<MethodDefinition>() == 80usize);
    assert!(align_of::<MethodDefinition>() == 8usize);
    assert!(offset_of!(MethodDefinition, r#type) == 0usize);
    assert!(offset_of!(MethodDefinition, span) == 4usize);
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
    assert!(offset_of!(PropertyDefinition, r#type) == 0usize);
    assert!(offset_of!(PropertyDefinition, span) == 4usize);
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

    assert!(size_of::<PrivateIdentifier>() == 24usize);
    assert!(align_of::<PrivateIdentifier>() == 8usize);
    assert!(offset_of!(PrivateIdentifier, span) == 0usize);
    assert!(offset_of!(PrivateIdentifier, name) == 8usize);

    assert!(size_of::<StaticBlock>() == 48usize);
    assert!(align_of::<StaticBlock>() == 8usize);
    assert!(offset_of!(StaticBlock, span) == 0usize);
    assert!(offset_of!(StaticBlock, body) == 8usize);
    assert!(offset_of!(StaticBlock, scope_id) == 40usize);

    assert!(size_of::<ModuleDeclaration>() == 16usize);
    assert!(align_of::<ModuleDeclaration>() == 8usize);

    assert!(size_of::<AccessorPropertyType>() == 1usize);
    assert!(align_of::<AccessorPropertyType>() == 1usize);

    assert!(size_of::<AccessorProperty>() == 104usize);
    assert!(align_of::<AccessorProperty>() == 8usize);
    assert!(offset_of!(AccessorProperty, r#type) == 0usize);
    assert!(offset_of!(AccessorProperty, span) == 4usize);
    assert!(offset_of!(AccessorProperty, decorators) == 16usize);
    assert!(offset_of!(AccessorProperty, key) == 48usize);
    assert!(offset_of!(AccessorProperty, value) == 64usize);
    assert!(offset_of!(AccessorProperty, computed) == 80usize);
    assert!(offset_of!(AccessorProperty, r#static) == 81usize);
    assert!(offset_of!(AccessorProperty, definite) == 82usize);
    assert!(offset_of!(AccessorProperty, type_annotation) == 88usize);
    assert!(offset_of!(AccessorProperty, accessibility) == 96usize);

    assert!(size_of::<ImportExpression>() == 56usize);
    assert!(align_of::<ImportExpression>() == 8usize);
    assert!(offset_of!(ImportExpression, span) == 0usize);
    assert!(offset_of!(ImportExpression, source) == 8usize);
    assert!(offset_of!(ImportExpression, arguments) == 24usize);

    assert!(size_of::<ImportDeclaration>() == 80usize);
    assert!(align_of::<ImportDeclaration>() == 8usize);
    assert!(offset_of!(ImportDeclaration, span) == 0usize);
    assert!(offset_of!(ImportDeclaration, specifiers) == 8usize);
    assert!(offset_of!(ImportDeclaration, source) == 40usize);
    assert!(offset_of!(ImportDeclaration, with_clause) == 64usize);
    assert!(offset_of!(ImportDeclaration, import_kind) == 72usize);

    assert!(size_of::<ImportDeclarationSpecifier>() == 16usize);
    assert!(align_of::<ImportDeclarationSpecifier>() == 8usize);

    assert!(size_of::<ImportSpecifier>() == 88usize);
    assert!(align_of::<ImportSpecifier>() == 8usize);
    assert!(offset_of!(ImportSpecifier, span) == 0usize);
    assert!(offset_of!(ImportSpecifier, imported) == 8usize);
    assert!(offset_of!(ImportSpecifier, local) == 48usize);
    assert!(offset_of!(ImportSpecifier, import_kind) == 80usize);

    assert!(size_of::<ImportDefaultSpecifier>() == 40usize);
    assert!(align_of::<ImportDefaultSpecifier>() == 8usize);
    assert!(offset_of!(ImportDefaultSpecifier, span) == 0usize);
    assert!(offset_of!(ImportDefaultSpecifier, local) == 8usize);

    assert!(size_of::<ImportNamespaceSpecifier>() == 40usize);
    assert!(align_of::<ImportNamespaceSpecifier>() == 8usize);
    assert!(offset_of!(ImportNamespaceSpecifier, span) == 0usize);
    assert!(offset_of!(ImportNamespaceSpecifier, local) == 8usize);

    assert!(size_of::<WithClause>() == 64usize);
    assert!(align_of::<WithClause>() == 8usize);
    assert!(offset_of!(WithClause, span) == 0usize);
    assert!(offset_of!(WithClause, attributes_keyword) == 8usize);
    assert!(offset_of!(WithClause, with_entries) == 32usize);

    assert!(size_of::<ImportAttribute>() == 64usize);
    assert!(align_of::<ImportAttribute>() == 8usize);
    assert!(offset_of!(ImportAttribute, span) == 0usize);
    assert!(offset_of!(ImportAttribute, key) == 8usize);
    assert!(offset_of!(ImportAttribute, value) == 40usize);

    assert!(size_of::<ImportAttributeKey>() == 32usize);
    assert!(align_of::<ImportAttributeKey>() == 8usize);

    assert!(size_of::<ExportNamedDeclaration>() == 96usize);
    assert!(align_of::<ExportNamedDeclaration>() == 8usize);
    assert!(offset_of!(ExportNamedDeclaration, span) == 0usize);
    assert!(offset_of!(ExportNamedDeclaration, declaration) == 8usize);
    assert!(offset_of!(ExportNamedDeclaration, specifiers) == 24usize);
    assert!(offset_of!(ExportNamedDeclaration, source) == 56usize);
    assert!(offset_of!(ExportNamedDeclaration, export_kind) == 80usize);
    assert!(offset_of!(ExportNamedDeclaration, with_clause) == 88usize);

    assert!(size_of::<ExportDefaultDeclaration>() == 64usize);
    assert!(align_of::<ExportDefaultDeclaration>() == 8usize);
    assert!(offset_of!(ExportDefaultDeclaration, span) == 0usize);
    assert!(offset_of!(ExportDefaultDeclaration, declaration) == 8usize);
    assert!(offset_of!(ExportDefaultDeclaration, exported) == 24usize);

    assert!(size_of::<ExportAllDeclaration>() == 88usize);
    assert!(align_of::<ExportAllDeclaration>() == 8usize);
    assert!(offset_of!(ExportAllDeclaration, span) == 0usize);
    assert!(offset_of!(ExportAllDeclaration, exported) == 8usize);
    assert!(offset_of!(ExportAllDeclaration, source) == 48usize);
    assert!(offset_of!(ExportAllDeclaration, with_clause) == 72usize);
    assert!(offset_of!(ExportAllDeclaration, export_kind) == 80usize);

    assert!(size_of::<ExportSpecifier>() == 96usize);
    assert!(align_of::<ExportSpecifier>() == 8usize);
    assert!(offset_of!(ExportSpecifier, span) == 0usize);
    assert!(offset_of!(ExportSpecifier, local) == 8usize);
    assert!(offset_of!(ExportSpecifier, exported) == 48usize);
    assert!(offset_of!(ExportSpecifier, export_kind) == 88usize);

    assert!(size_of::<ExportDefaultDeclarationKind>() == 16usize);
    assert!(align_of::<ExportDefaultDeclarationKind>() == 8usize);

    assert!(size_of::<ModuleExportName>() == 40usize);
    assert!(align_of::<ModuleExportName>() == 8usize);

    assert!(size_of::<TSThisParameter>() == 24usize);
    assert!(align_of::<TSThisParameter>() == 8usize);
    assert!(offset_of!(TSThisParameter, span) == 0usize);
    assert!(offset_of!(TSThisParameter, this_span) == 8usize);
    assert!(offset_of!(TSThisParameter, type_annotation) == 16usize);

    assert!(size_of::<TSEnumDeclaration>() == 80usize);
    assert!(align_of::<TSEnumDeclaration>() == 8usize);
    assert!(offset_of!(TSEnumDeclaration, span) == 0usize);
    assert!(offset_of!(TSEnumDeclaration, id) == 8usize);
    assert!(offset_of!(TSEnumDeclaration, members) == 40usize);
    assert!(offset_of!(TSEnumDeclaration, r#const) == 72usize);
    assert!(offset_of!(TSEnumDeclaration, declare) == 73usize);
    assert!(offset_of!(TSEnumDeclaration, scope_id) == 76usize);

    assert!(size_of::<TSEnumMember>() == 40usize);
    assert!(align_of::<TSEnumMember>() == 8usize);
    assert!(offset_of!(TSEnumMember, span) == 0usize);
    assert!(offset_of!(TSEnumMember, id) == 8usize);
    assert!(offset_of!(TSEnumMember, initializer) == 24usize);

    assert!(size_of::<TSEnumMemberName>() == 16usize);
    assert!(align_of::<TSEnumMemberName>() == 8usize);

    assert!(size_of::<TSTypeAnnotation>() == 24usize);
    assert!(align_of::<TSTypeAnnotation>() == 8usize);
    assert!(offset_of!(TSTypeAnnotation, span) == 0usize);
    assert!(offset_of!(TSTypeAnnotation, type_annotation) == 8usize);

    assert!(size_of::<TSLiteralType>() == 24usize);
    assert!(align_of::<TSLiteralType>() == 8usize);
    assert!(offset_of!(TSLiteralType, span) == 0usize);
    assert!(offset_of!(TSLiteralType, literal) == 8usize);

    assert!(size_of::<TSLiteral>() == 16usize);
    assert!(align_of::<TSLiteral>() == 8usize);

    assert!(size_of::<TSType>() == 16usize);
    assert!(align_of::<TSType>() == 8usize);

    assert!(size_of::<TSConditionalType>() == 80usize);
    assert!(align_of::<TSConditionalType>() == 8usize);
    assert!(offset_of!(TSConditionalType, span) == 0usize);
    assert!(offset_of!(TSConditionalType, check_type) == 8usize);
    assert!(offset_of!(TSConditionalType, extends_type) == 24usize);
    assert!(offset_of!(TSConditionalType, true_type) == 40usize);
    assert!(offset_of!(TSConditionalType, false_type) == 56usize);
    assert!(offset_of!(TSConditionalType, scope_id) == 72usize);

    assert!(size_of::<TSUnionType>() == 40usize);
    assert!(align_of::<TSUnionType>() == 8usize);
    assert!(offset_of!(TSUnionType, span) == 0usize);
    assert!(offset_of!(TSUnionType, types) == 8usize);

    assert!(size_of::<TSIntersectionType>() == 40usize);
    assert!(align_of::<TSIntersectionType>() == 8usize);
    assert!(offset_of!(TSIntersectionType, span) == 0usize);
    assert!(offset_of!(TSIntersectionType, types) == 8usize);

    assert!(size_of::<TSParenthesizedType>() == 24usize);
    assert!(align_of::<TSParenthesizedType>() == 8usize);
    assert!(offset_of!(TSParenthesizedType, span) == 0usize);
    assert!(offset_of!(TSParenthesizedType, type_annotation) == 8usize);

    assert!(size_of::<TSTypeOperator>() == 32usize);
    assert!(align_of::<TSTypeOperator>() == 8usize);
    assert!(offset_of!(TSTypeOperator, span) == 0usize);
    assert!(offset_of!(TSTypeOperator, operator) == 8usize);
    assert!(offset_of!(TSTypeOperator, type_annotation) == 16usize);

    assert!(size_of::<TSTypeOperatorOperator>() == 1usize);
    assert!(align_of::<TSTypeOperatorOperator>() == 1usize);

    assert!(size_of::<TSArrayType>() == 24usize);
    assert!(align_of::<TSArrayType>() == 8usize);
    assert!(offset_of!(TSArrayType, span) == 0usize);
    assert!(offset_of!(TSArrayType, element_type) == 8usize);

    assert!(size_of::<TSIndexedAccessType>() == 40usize);
    assert!(align_of::<TSIndexedAccessType>() == 8usize);
    assert!(offset_of!(TSIndexedAccessType, span) == 0usize);
    assert!(offset_of!(TSIndexedAccessType, object_type) == 8usize);
    assert!(offset_of!(TSIndexedAccessType, index_type) == 24usize);

    assert!(size_of::<TSTupleType>() == 40usize);
    assert!(align_of::<TSTupleType>() == 8usize);
    assert!(offset_of!(TSTupleType, span) == 0usize);
    assert!(offset_of!(TSTupleType, element_types) == 8usize);

    assert!(size_of::<TSNamedTupleMember>() == 56usize);
    assert!(align_of::<TSNamedTupleMember>() == 8usize);
    assert!(offset_of!(TSNamedTupleMember, span) == 0usize);
    assert!(offset_of!(TSNamedTupleMember, element_type) == 8usize);
    assert!(offset_of!(TSNamedTupleMember, label) == 24usize);
    assert!(offset_of!(TSNamedTupleMember, optional) == 48usize);

    assert!(size_of::<TSOptionalType>() == 24usize);
    assert!(align_of::<TSOptionalType>() == 8usize);
    assert!(offset_of!(TSOptionalType, span) == 0usize);
    assert!(offset_of!(TSOptionalType, type_annotation) == 8usize);

    assert!(size_of::<TSRestType>() == 24usize);
    assert!(align_of::<TSRestType>() == 8usize);
    assert!(offset_of!(TSRestType, span) == 0usize);
    assert!(offset_of!(TSRestType, type_annotation) == 8usize);

    assert!(size_of::<TSTupleElement>() == 16usize);
    assert!(align_of::<TSTupleElement>() == 8usize);

    assert!(size_of::<TSAnyKeyword>() == 8usize);
    assert!(align_of::<TSAnyKeyword>() == 4usize);
    assert!(offset_of!(TSAnyKeyword, span) == 0usize);

    assert!(size_of::<TSStringKeyword>() == 8usize);
    assert!(align_of::<TSStringKeyword>() == 4usize);
    assert!(offset_of!(TSStringKeyword, span) == 0usize);

    assert!(size_of::<TSBooleanKeyword>() == 8usize);
    assert!(align_of::<TSBooleanKeyword>() == 4usize);
    assert!(offset_of!(TSBooleanKeyword, span) == 0usize);

    assert!(size_of::<TSNumberKeyword>() == 8usize);
    assert!(align_of::<TSNumberKeyword>() == 4usize);
    assert!(offset_of!(TSNumberKeyword, span) == 0usize);

    assert!(size_of::<TSNeverKeyword>() == 8usize);
    assert!(align_of::<TSNeverKeyword>() == 4usize);
    assert!(offset_of!(TSNeverKeyword, span) == 0usize);

    assert!(size_of::<TSIntrinsicKeyword>() == 8usize);
    assert!(align_of::<TSIntrinsicKeyword>() == 4usize);
    assert!(offset_of!(TSIntrinsicKeyword, span) == 0usize);

    assert!(size_of::<TSUnknownKeyword>() == 8usize);
    assert!(align_of::<TSUnknownKeyword>() == 4usize);
    assert!(offset_of!(TSUnknownKeyword, span) == 0usize);

    assert!(size_of::<TSNullKeyword>() == 8usize);
    assert!(align_of::<TSNullKeyword>() == 4usize);
    assert!(offset_of!(TSNullKeyword, span) == 0usize);

    assert!(size_of::<TSUndefinedKeyword>() == 8usize);
    assert!(align_of::<TSUndefinedKeyword>() == 4usize);
    assert!(offset_of!(TSUndefinedKeyword, span) == 0usize);

    assert!(size_of::<TSVoidKeyword>() == 8usize);
    assert!(align_of::<TSVoidKeyword>() == 4usize);
    assert!(offset_of!(TSVoidKeyword, span) == 0usize);

    assert!(size_of::<TSSymbolKeyword>() == 8usize);
    assert!(align_of::<TSSymbolKeyword>() == 4usize);
    assert!(offset_of!(TSSymbolKeyword, span) == 0usize);

    assert!(size_of::<TSThisType>() == 8usize);
    assert!(align_of::<TSThisType>() == 4usize);
    assert!(offset_of!(TSThisType, span) == 0usize);

    assert!(size_of::<TSObjectKeyword>() == 8usize);
    assert!(align_of::<TSObjectKeyword>() == 4usize);
    assert!(offset_of!(TSObjectKeyword, span) == 0usize);

    assert!(size_of::<TSBigIntKeyword>() == 8usize);
    assert!(align_of::<TSBigIntKeyword>() == 4usize);
    assert!(offset_of!(TSBigIntKeyword, span) == 0usize);

    assert!(size_of::<TSTypeReference>() == 32usize);
    assert!(align_of::<TSTypeReference>() == 8usize);
    assert!(offset_of!(TSTypeReference, span) == 0usize);
    assert!(offset_of!(TSTypeReference, type_name) == 8usize);
    assert!(offset_of!(TSTypeReference, type_parameters) == 24usize);

    assert!(size_of::<TSTypeName>() == 16usize);
    assert!(align_of::<TSTypeName>() == 8usize);

    assert!(size_of::<TSQualifiedName>() == 48usize);
    assert!(align_of::<TSQualifiedName>() == 8usize);
    assert!(offset_of!(TSQualifiedName, span) == 0usize);
    assert!(offset_of!(TSQualifiedName, left) == 8usize);
    assert!(offset_of!(TSQualifiedName, right) == 24usize);

    assert!(size_of::<TSTypeParameterInstantiation>() == 40usize);
    assert!(align_of::<TSTypeParameterInstantiation>() == 8usize);
    assert!(offset_of!(TSTypeParameterInstantiation, span) == 0usize);
    assert!(offset_of!(TSTypeParameterInstantiation, params) == 8usize);

    assert!(size_of::<TSTypeParameter>() == 80usize);
    assert!(align_of::<TSTypeParameter>() == 8usize);
    assert!(offset_of!(TSTypeParameter, span) == 0usize);
    assert!(offset_of!(TSTypeParameter, name) == 8usize);
    assert!(offset_of!(TSTypeParameter, constraint) == 40usize);
    assert!(offset_of!(TSTypeParameter, default) == 56usize);
    assert!(offset_of!(TSTypeParameter, r#in) == 72usize);
    assert!(offset_of!(TSTypeParameter, out) == 73usize);
    assert!(offset_of!(TSTypeParameter, r#const) == 74usize);

    assert!(size_of::<TSTypeParameterDeclaration>() == 40usize);
    assert!(align_of::<TSTypeParameterDeclaration>() == 8usize);
    assert!(offset_of!(TSTypeParameterDeclaration, span) == 0usize);
    assert!(offset_of!(TSTypeParameterDeclaration, params) == 8usize);

    assert!(size_of::<TSTypeAliasDeclaration>() == 72usize);
    assert!(align_of::<TSTypeAliasDeclaration>() == 8usize);
    assert!(offset_of!(TSTypeAliasDeclaration, span) == 0usize);
    assert!(offset_of!(TSTypeAliasDeclaration, id) == 8usize);
    assert!(offset_of!(TSTypeAliasDeclaration, type_parameters) == 40usize);
    assert!(offset_of!(TSTypeAliasDeclaration, type_annotation) == 48usize);
    assert!(offset_of!(TSTypeAliasDeclaration, declare) == 64usize);
    assert!(offset_of!(TSTypeAliasDeclaration, scope_id) == 68usize);

    assert!(size_of::<TSAccessibility>() == 1usize);
    assert!(align_of::<TSAccessibility>() == 1usize);

    assert!(size_of::<TSClassImplements>() == 32usize);
    assert!(align_of::<TSClassImplements>() == 8usize);
    assert!(offset_of!(TSClassImplements, span) == 0usize);
    assert!(offset_of!(TSClassImplements, expression) == 8usize);
    assert!(offset_of!(TSClassImplements, type_parameters) == 24usize);

    assert!(size_of::<TSInterfaceDeclaration>() == 96usize);
    assert!(align_of::<TSInterfaceDeclaration>() == 8usize);
    assert!(offset_of!(TSInterfaceDeclaration, span) == 0usize);
    assert!(offset_of!(TSInterfaceDeclaration, id) == 8usize);
    assert!(offset_of!(TSInterfaceDeclaration, extends) == 40usize);
    assert!(offset_of!(TSInterfaceDeclaration, type_parameters) == 72usize);
    assert!(offset_of!(TSInterfaceDeclaration, body) == 80usize);
    assert!(offset_of!(TSInterfaceDeclaration, declare) == 88usize);
    assert!(offset_of!(TSInterfaceDeclaration, scope_id) == 92usize);

    assert!(size_of::<TSInterfaceBody>() == 40usize);
    assert!(align_of::<TSInterfaceBody>() == 8usize);
    assert!(offset_of!(TSInterfaceBody, span) == 0usize);
    assert!(offset_of!(TSInterfaceBody, body) == 8usize);

    assert!(size_of::<TSPropertySignature>() == 40usize);
    assert!(align_of::<TSPropertySignature>() == 8usize);
    assert!(offset_of!(TSPropertySignature, span) == 0usize);
    assert!(offset_of!(TSPropertySignature, computed) == 8usize);
    assert!(offset_of!(TSPropertySignature, optional) == 9usize);
    assert!(offset_of!(TSPropertySignature, readonly) == 10usize);
    assert!(offset_of!(TSPropertySignature, key) == 16usize);
    assert!(offset_of!(TSPropertySignature, type_annotation) == 32usize);

    assert!(size_of::<TSSignature>() == 16usize);
    assert!(align_of::<TSSignature>() == 8usize);

    assert!(size_of::<TSIndexSignature>() == 56usize);
    assert!(align_of::<TSIndexSignature>() == 8usize);
    assert!(offset_of!(TSIndexSignature, span) == 0usize);
    assert!(offset_of!(TSIndexSignature, parameters) == 8usize);
    assert!(offset_of!(TSIndexSignature, type_annotation) == 40usize);
    assert!(offset_of!(TSIndexSignature, readonly) == 48usize);

    assert!(size_of::<TSCallSignatureDeclaration>() == 64usize);
    assert!(align_of::<TSCallSignatureDeclaration>() == 8usize);
    assert!(offset_of!(TSCallSignatureDeclaration, span) == 0usize);
    assert!(offset_of!(TSCallSignatureDeclaration, this_param) == 8usize);
    assert!(offset_of!(TSCallSignatureDeclaration, params) == 40usize);
    assert!(offset_of!(TSCallSignatureDeclaration, return_type) == 48usize);
    assert!(offset_of!(TSCallSignatureDeclaration, type_parameters) == 56usize);

    assert!(size_of::<TSMethodSignatureKind>() == 1usize);
    assert!(align_of::<TSMethodSignatureKind>() == 1usize);

    assert!(size_of::<TSMethodSignature>() == 72usize);
    assert!(align_of::<TSMethodSignature>() == 8usize);
    assert!(offset_of!(TSMethodSignature, span) == 0usize);
    assert!(offset_of!(TSMethodSignature, key) == 8usize);
    assert!(offset_of!(TSMethodSignature, computed) == 24usize);
    assert!(offset_of!(TSMethodSignature, optional) == 25usize);
    assert!(offset_of!(TSMethodSignature, kind) == 26usize);
    assert!(offset_of!(TSMethodSignature, this_param) == 32usize);
    assert!(offset_of!(TSMethodSignature, params) == 40usize);
    assert!(offset_of!(TSMethodSignature, return_type) == 48usize);
    assert!(offset_of!(TSMethodSignature, type_parameters) == 56usize);
    assert!(offset_of!(TSMethodSignature, scope_id) == 64usize);

    assert!(size_of::<TSConstructSignatureDeclaration>() == 40usize);
    assert!(align_of::<TSConstructSignatureDeclaration>() == 8usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, span) == 0usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, params) == 8usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, return_type) == 16usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, type_parameters) == 24usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, scope_id) == 32usize);

    assert!(size_of::<TSIndexSignatureName>() == 32usize);
    assert!(align_of::<TSIndexSignatureName>() == 8usize);
    assert!(offset_of!(TSIndexSignatureName, span) == 0usize);
    assert!(offset_of!(TSIndexSignatureName, name) == 8usize);
    assert!(offset_of!(TSIndexSignatureName, type_annotation) == 24usize);

    assert!(size_of::<TSInterfaceHeritage>() == 32usize);
    assert!(align_of::<TSInterfaceHeritage>() == 8usize);
    assert!(offset_of!(TSInterfaceHeritage, span) == 0usize);
    assert!(offset_of!(TSInterfaceHeritage, expression) == 8usize);
    assert!(offset_of!(TSInterfaceHeritage, type_parameters) == 24usize);

    assert!(size_of::<TSTypePredicate>() == 40usize);
    assert!(align_of::<TSTypePredicate>() == 8usize);
    assert!(offset_of!(TSTypePredicate, span) == 0usize);
    assert!(offset_of!(TSTypePredicate, parameter_name) == 8usize);
    assert!(offset_of!(TSTypePredicate, asserts) == 24usize);
    assert!(offset_of!(TSTypePredicate, type_annotation) == 32usize);

    assert!(size_of::<TSTypePredicateName>() == 16usize);
    assert!(align_of::<TSTypePredicateName>() == 8usize);

    assert!(size_of::<TSModuleDeclaration>() == 64usize);
    assert!(align_of::<TSModuleDeclaration>() == 8usize);
    assert!(offset_of!(TSModuleDeclaration, span) == 0usize);
    assert!(offset_of!(TSModuleDeclaration, id) == 8usize);
    assert!(offset_of!(TSModuleDeclaration, body) == 40usize);
    assert!(offset_of!(TSModuleDeclaration, kind) == 56usize);
    assert!(offset_of!(TSModuleDeclaration, declare) == 57usize);
    assert!(offset_of!(TSModuleDeclaration, scope_id) == 60usize);

    assert!(size_of::<TSModuleDeclarationKind>() == 1usize);
    assert!(align_of::<TSModuleDeclarationKind>() == 1usize);

    assert!(size_of::<TSModuleDeclarationName>() == 32usize);
    assert!(align_of::<TSModuleDeclarationName>() == 8usize);

    assert!(size_of::<TSModuleDeclarationBody>() == 16usize);
    assert!(align_of::<TSModuleDeclarationBody>() == 8usize);

    assert!(size_of::<TSModuleBlock>() == 72usize);
    assert!(align_of::<TSModuleBlock>() == 8usize);
    assert!(offset_of!(TSModuleBlock, span) == 0usize);
    assert!(offset_of!(TSModuleBlock, directives) == 8usize);
    assert!(offset_of!(TSModuleBlock, body) == 40usize);

    assert!(size_of::<TSTypeLiteral>() == 40usize);
    assert!(align_of::<TSTypeLiteral>() == 8usize);
    assert!(offset_of!(TSTypeLiteral, span) == 0usize);
    assert!(offset_of!(TSTypeLiteral, members) == 8usize);

    assert!(size_of::<TSInferType>() == 16usize);
    assert!(align_of::<TSInferType>() == 8usize);
    assert!(offset_of!(TSInferType, span) == 0usize);
    assert!(offset_of!(TSInferType, type_parameter) == 8usize);

    assert!(size_of::<TSTypeQuery>() == 32usize);
    assert!(align_of::<TSTypeQuery>() == 8usize);
    assert!(offset_of!(TSTypeQuery, span) == 0usize);
    assert!(offset_of!(TSTypeQuery, expr_name) == 8usize);
    assert!(offset_of!(TSTypeQuery, type_parameters) == 24usize);

    assert!(size_of::<TSTypeQueryExprName>() == 16usize);
    assert!(align_of::<TSTypeQueryExprName>() == 8usize);

    assert!(size_of::<TSImportType>() == 64usize);
    assert!(align_of::<TSImportType>() == 8usize);
    assert!(offset_of!(TSImportType, span) == 0usize);
    assert!(offset_of!(TSImportType, is_type_of) == 8usize);
    assert!(offset_of!(TSImportType, parameter) == 16usize);
    assert!(offset_of!(TSImportType, qualifier) == 32usize);
    assert!(offset_of!(TSImportType, attributes) == 48usize);
    assert!(offset_of!(TSImportType, type_parameters) == 56usize);

    assert!(size_of::<TSImportAttributes>() == 64usize);
    assert!(align_of::<TSImportAttributes>() == 8usize);
    assert!(offset_of!(TSImportAttributes, span) == 0usize);
    assert!(offset_of!(TSImportAttributes, attributes_keyword) == 8usize);
    assert!(offset_of!(TSImportAttributes, elements) == 32usize);

    assert!(size_of::<TSImportAttribute>() == 56usize);
    assert!(align_of::<TSImportAttribute>() == 8usize);
    assert!(offset_of!(TSImportAttribute, span) == 0usize);
    assert!(offset_of!(TSImportAttribute, name) == 8usize);
    assert!(offset_of!(TSImportAttribute, value) == 40usize);

    assert!(size_of::<TSImportAttributeName>() == 32usize);
    assert!(align_of::<TSImportAttributeName>() == 8usize);

    assert!(size_of::<TSFunctionType>() == 40usize);
    assert!(align_of::<TSFunctionType>() == 8usize);
    assert!(offset_of!(TSFunctionType, span) == 0usize);
    assert!(offset_of!(TSFunctionType, this_param) == 8usize);
    assert!(offset_of!(TSFunctionType, params) == 16usize);
    assert!(offset_of!(TSFunctionType, return_type) == 24usize);
    assert!(offset_of!(TSFunctionType, type_parameters) == 32usize);

    assert!(size_of::<TSConstructorType>() == 40usize);
    assert!(align_of::<TSConstructorType>() == 8usize);
    assert!(offset_of!(TSConstructorType, span) == 0usize);
    assert!(offset_of!(TSConstructorType, r#abstract) == 8usize);
    assert!(offset_of!(TSConstructorType, params) == 16usize);
    assert!(offset_of!(TSConstructorType, return_type) == 24usize);
    assert!(offset_of!(TSConstructorType, type_parameters) == 32usize);

    assert!(size_of::<TSMappedType>() == 56usize);
    assert!(align_of::<TSMappedType>() == 8usize);
    assert!(offset_of!(TSMappedType, span) == 0usize);
    assert!(offset_of!(TSMappedType, type_parameter) == 8usize);
    assert!(offset_of!(TSMappedType, name_type) == 16usize);
    assert!(offset_of!(TSMappedType, type_annotation) == 32usize);
    assert!(offset_of!(TSMappedType, optional) == 48usize);
    assert!(offset_of!(TSMappedType, readonly) == 49usize);
    assert!(offset_of!(TSMappedType, scope_id) == 52usize);

    assert!(size_of::<TSMappedTypeModifierOperator>() == 1usize);
    assert!(align_of::<TSMappedTypeModifierOperator>() == 1usize);

    assert!(size_of::<TSTemplateLiteralType>() == 72usize);
    assert!(align_of::<TSTemplateLiteralType>() == 8usize);
    assert!(offset_of!(TSTemplateLiteralType, span) == 0usize);
    assert!(offset_of!(TSTemplateLiteralType, quasis) == 8usize);
    assert!(offset_of!(TSTemplateLiteralType, types) == 40usize);

    assert!(size_of::<TSAsExpression>() == 40usize);
    assert!(align_of::<TSAsExpression>() == 8usize);
    assert!(offset_of!(TSAsExpression, span) == 0usize);
    assert!(offset_of!(TSAsExpression, expression) == 8usize);
    assert!(offset_of!(TSAsExpression, type_annotation) == 24usize);

    assert!(size_of::<TSSatisfiesExpression>() == 40usize);
    assert!(align_of::<TSSatisfiesExpression>() == 8usize);
    assert!(offset_of!(TSSatisfiesExpression, span) == 0usize);
    assert!(offset_of!(TSSatisfiesExpression, expression) == 8usize);
    assert!(offset_of!(TSSatisfiesExpression, type_annotation) == 24usize);

    assert!(size_of::<TSTypeAssertion>() == 40usize);
    assert!(align_of::<TSTypeAssertion>() == 8usize);
    assert!(offset_of!(TSTypeAssertion, span) == 0usize);
    assert!(offset_of!(TSTypeAssertion, expression) == 8usize);
    assert!(offset_of!(TSTypeAssertion, type_annotation) == 24usize);

    assert!(size_of::<TSImportEqualsDeclaration>() == 64usize);
    assert!(align_of::<TSImportEqualsDeclaration>() == 8usize);
    assert!(offset_of!(TSImportEqualsDeclaration, span) == 0usize);
    assert!(offset_of!(TSImportEqualsDeclaration, id) == 8usize);
    assert!(offset_of!(TSImportEqualsDeclaration, module_reference) == 40usize);
    assert!(offset_of!(TSImportEqualsDeclaration, import_kind) == 56usize);

    assert!(size_of::<TSModuleReference>() == 16usize);
    assert!(align_of::<TSModuleReference>() == 8usize);

    assert!(size_of::<TSExternalModuleReference>() == 32usize);
    assert!(align_of::<TSExternalModuleReference>() == 8usize);
    assert!(offset_of!(TSExternalModuleReference, span) == 0usize);
    assert!(offset_of!(TSExternalModuleReference, expression) == 8usize);

    assert!(size_of::<TSNonNullExpression>() == 24usize);
    assert!(align_of::<TSNonNullExpression>() == 8usize);
    assert!(offset_of!(TSNonNullExpression, span) == 0usize);
    assert!(offset_of!(TSNonNullExpression, expression) == 8usize);

    assert!(size_of::<Decorator>() == 24usize);
    assert!(align_of::<Decorator>() == 8usize);
    assert!(offset_of!(Decorator, span) == 0usize);
    assert!(offset_of!(Decorator, expression) == 8usize);

    assert!(size_of::<TSExportAssignment>() == 24usize);
    assert!(align_of::<TSExportAssignment>() == 8usize);
    assert!(offset_of!(TSExportAssignment, span) == 0usize);
    assert!(offset_of!(TSExportAssignment, expression) == 8usize);

    assert!(size_of::<TSNamespaceExportDeclaration>() == 32usize);
    assert!(align_of::<TSNamespaceExportDeclaration>() == 8usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, span) == 0usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, id) == 8usize);

    assert!(size_of::<TSInstantiationExpression>() == 32usize);
    assert!(align_of::<TSInstantiationExpression>() == 8usize);
    assert!(offset_of!(TSInstantiationExpression, span) == 0usize);
    assert!(offset_of!(TSInstantiationExpression, expression) == 8usize);
    assert!(offset_of!(TSInstantiationExpression, type_parameters) == 24usize);

    assert!(size_of::<ImportOrExportKind>() == 1usize);
    assert!(align_of::<ImportOrExportKind>() == 1usize);

    assert!(size_of::<JSDocNullableType>() == 32usize);
    assert!(align_of::<JSDocNullableType>() == 8usize);
    assert!(offset_of!(JSDocNullableType, span) == 0usize);
    assert!(offset_of!(JSDocNullableType, type_annotation) == 8usize);
    assert!(offset_of!(JSDocNullableType, postfix) == 24usize);

    assert!(size_of::<JSDocNonNullableType>() == 32usize);
    assert!(align_of::<JSDocNonNullableType>() == 8usize);
    assert!(offset_of!(JSDocNonNullableType, span) == 0usize);
    assert!(offset_of!(JSDocNonNullableType, type_annotation) == 8usize);
    assert!(offset_of!(JSDocNonNullableType, postfix) == 24usize);

    assert!(size_of::<JSDocUnknownType>() == 8usize);
    assert!(align_of::<JSDocUnknownType>() == 4usize);
    assert!(offset_of!(JSDocUnknownType, span) == 0usize);

    assert!(size_of::<JSXElement>() == 56usize);
    assert!(align_of::<JSXElement>() == 8usize);
    assert!(offset_of!(JSXElement, span) == 0usize);
    assert!(offset_of!(JSXElement, opening_element) == 8usize);
    assert!(offset_of!(JSXElement, closing_element) == 16usize);
    assert!(offset_of!(JSXElement, children) == 24usize);

    assert!(size_of::<JSXOpeningElement>() == 72usize);
    assert!(align_of::<JSXOpeningElement>() == 8usize);
    assert!(offset_of!(JSXOpeningElement, span) == 0usize);
    assert!(offset_of!(JSXOpeningElement, self_closing) == 8usize);
    assert!(offset_of!(JSXOpeningElement, name) == 16usize);
    assert!(offset_of!(JSXOpeningElement, attributes) == 32usize);
    assert!(offset_of!(JSXOpeningElement, type_parameters) == 64usize);

    assert!(size_of::<JSXClosingElement>() == 24usize);
    assert!(align_of::<JSXClosingElement>() == 8usize);
    assert!(offset_of!(JSXClosingElement, span) == 0usize);
    assert!(offset_of!(JSXClosingElement, name) == 8usize);

    assert!(size_of::<JSXFragment>() == 56usize);
    assert!(align_of::<JSXFragment>() == 8usize);
    assert!(offset_of!(JSXFragment, span) == 0usize);
    assert!(offset_of!(JSXFragment, opening_fragment) == 8usize);
    assert!(offset_of!(JSXFragment, closing_fragment) == 16usize);
    assert!(offset_of!(JSXFragment, children) == 24usize);

    assert!(size_of::<JSXOpeningFragment>() == 8usize);
    assert!(align_of::<JSXOpeningFragment>() == 4usize);
    assert!(offset_of!(JSXOpeningFragment, span) == 0usize);

    assert!(size_of::<JSXClosingFragment>() == 8usize);
    assert!(align_of::<JSXClosingFragment>() == 4usize);
    assert!(offset_of!(JSXClosingFragment, span) == 0usize);

    assert!(size_of::<JSXElementName>() == 16usize);
    assert!(align_of::<JSXElementName>() == 8usize);

    assert!(size_of::<JSXNamespacedName>() == 56usize);
    assert!(align_of::<JSXNamespacedName>() == 8usize);
    assert!(offset_of!(JSXNamespacedName, span) == 0usize);
    assert!(offset_of!(JSXNamespacedName, namespace) == 8usize);
    assert!(offset_of!(JSXNamespacedName, property) == 32usize);

    assert!(size_of::<JSXMemberExpression>() == 48usize);
    assert!(align_of::<JSXMemberExpression>() == 8usize);
    assert!(offset_of!(JSXMemberExpression, span) == 0usize);
    assert!(offset_of!(JSXMemberExpression, object) == 8usize);
    assert!(offset_of!(JSXMemberExpression, property) == 24usize);

    assert!(size_of::<JSXMemberExpressionObject>() == 16usize);
    assert!(align_of::<JSXMemberExpressionObject>() == 8usize);

    assert!(size_of::<JSXExpressionContainer>() == 24usize);
    assert!(align_of::<JSXExpressionContainer>() == 8usize);
    assert!(offset_of!(JSXExpressionContainer, span) == 0usize);
    assert!(offset_of!(JSXExpressionContainer, expression) == 8usize);

    assert!(size_of::<JSXExpression>() == 16usize);
    assert!(align_of::<JSXExpression>() == 8usize);

    assert!(size_of::<JSXEmptyExpression>() == 8usize);
    assert!(align_of::<JSXEmptyExpression>() == 4usize);
    assert!(offset_of!(JSXEmptyExpression, span) == 0usize);

    assert!(size_of::<JSXAttributeItem>() == 16usize);
    assert!(align_of::<JSXAttributeItem>() == 8usize);

    assert!(size_of::<JSXAttribute>() == 40usize);
    assert!(align_of::<JSXAttribute>() == 8usize);
    assert!(offset_of!(JSXAttribute, span) == 0usize);
    assert!(offset_of!(JSXAttribute, name) == 8usize);
    assert!(offset_of!(JSXAttribute, value) == 24usize);

    assert!(size_of::<JSXSpreadAttribute>() == 24usize);
    assert!(align_of::<JSXSpreadAttribute>() == 8usize);
    assert!(offset_of!(JSXSpreadAttribute, span) == 0usize);
    assert!(offset_of!(JSXSpreadAttribute, argument) == 8usize);

    assert!(size_of::<JSXAttributeName>() == 16usize);
    assert!(align_of::<JSXAttributeName>() == 8usize);

    assert!(size_of::<JSXAttributeValue>() == 16usize);
    assert!(align_of::<JSXAttributeValue>() == 8usize);

    assert!(size_of::<JSXIdentifier>() == 24usize);
    assert!(align_of::<JSXIdentifier>() == 8usize);
    assert!(offset_of!(JSXIdentifier, span) == 0usize);
    assert!(offset_of!(JSXIdentifier, name) == 8usize);

    assert!(size_of::<JSXChild>() == 16usize);
    assert!(align_of::<JSXChild>() == 8usize);

    assert!(size_of::<JSXSpreadChild>() == 24usize);
    assert!(align_of::<JSXSpreadChild>() == 8usize);
    assert!(offset_of!(JSXSpreadChild, span) == 0usize);
    assert!(offset_of!(JSXSpreadChild, expression) == 8usize);

    assert!(size_of::<JSXText>() == 24usize);
    assert!(align_of::<JSXText>() == 8usize);
    assert!(offset_of!(JSXText, span) == 0usize);
    assert!(offset_of!(JSXText, value) == 8usize);

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

    assert!(size_of::<Term>() == 16usize);
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

    assert!(size_of::<Quantifier>() == 56usize);
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

    assert!(size_of::<CharacterClassContents>() == 16usize);
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
    assert!(size_of::<BooleanLiteral>() == 12usize);
    assert!(align_of::<BooleanLiteral>() == 4usize);
    assert!(offset_of!(BooleanLiteral, span) == 0usize);
    assert!(offset_of!(BooleanLiteral, value) == 8usize);

    assert!(size_of::<NullLiteral>() == 8usize);
    assert!(align_of::<NullLiteral>() == 4usize);
    assert!(offset_of!(NullLiteral, span) == 0usize);

    assert!(size_of::<NumericLiteral>() == 32usize);
    assert!(align_of::<NumericLiteral>() == 8usize);
    assert!(offset_of!(NumericLiteral, span) == 0usize);
    assert!(offset_of!(NumericLiteral, value) == 8usize);
    assert!(offset_of!(NumericLiteral, raw) == 16usize);
    assert!(offset_of!(NumericLiteral, base) == 24usize);

    assert!(size_of::<BigIntLiteral>() == 20usize);
    assert!(align_of::<BigIntLiteral>() == 4usize);
    assert!(offset_of!(BigIntLiteral, span) == 0usize);
    assert!(offset_of!(BigIntLiteral, raw) == 8usize);
    assert!(offset_of!(BigIntLiteral, base) == 16usize);

    assert!(size_of::<RegExpLiteral>() == 24usize);
    assert!(align_of::<RegExpLiteral>() == 4usize);
    assert!(offset_of!(RegExpLiteral, span) == 0usize);
    assert!(offset_of!(RegExpLiteral, value) == 8usize);
    assert!(offset_of!(RegExpLiteral, regex) == 8usize);

    assert!(size_of::<RegExp>() == 16usize);
    assert!(align_of::<RegExp>() == 4usize);
    assert!(offset_of!(RegExp, pattern) == 0usize);
    assert!(offset_of!(RegExp, flags) == 12usize);

    assert!(size_of::<RegExpPattern>() == 12usize);
    assert!(align_of::<RegExpPattern>() == 4usize);

    assert!(size_of::<EmptyObject>() == 0usize);
    assert!(align_of::<EmptyObject>() == 1usize);

    assert!(size_of::<StringLiteral>() == 16usize);
    assert!(align_of::<StringLiteral>() == 4usize);
    assert!(offset_of!(StringLiteral, span) == 0usize);
    assert!(offset_of!(StringLiteral, value) == 8usize);

    assert!(size_of::<Program>() == 64usize);
    assert!(align_of::<Program>() == 4usize);
    assert!(offset_of!(Program, span) == 0usize);
    assert!(offset_of!(Program, source_type) == 8usize);
    assert!(offset_of!(Program, hashbang) == 12usize);
    assert!(offset_of!(Program, directives) == 28usize);
    assert!(offset_of!(Program, body) == 44usize);
    assert!(offset_of!(Program, scope_id) == 60usize);

    assert!(size_of::<Expression>() == 8usize);
    assert!(align_of::<Expression>() == 4usize);

    assert!(size_of::<IdentifierName>() == 16usize);
    assert!(align_of::<IdentifierName>() == 4usize);
    assert!(offset_of!(IdentifierName, span) == 0usize);
    assert!(offset_of!(IdentifierName, name) == 8usize);

    assert!(size_of::<IdentifierReference>() == 20usize);
    assert!(align_of::<IdentifierReference>() == 4usize);
    assert!(offset_of!(IdentifierReference, span) == 0usize);
    assert!(offset_of!(IdentifierReference, name) == 8usize);
    assert!(offset_of!(IdentifierReference, reference_id) == 16usize);

    assert!(size_of::<BindingIdentifier>() == 20usize);
    assert!(align_of::<BindingIdentifier>() == 4usize);
    assert!(offset_of!(BindingIdentifier, span) == 0usize);
    assert!(offset_of!(BindingIdentifier, name) == 8usize);
    assert!(offset_of!(BindingIdentifier, symbol_id) == 16usize);

    assert!(size_of::<LabelIdentifier>() == 16usize);
    assert!(align_of::<LabelIdentifier>() == 4usize);
    assert!(offset_of!(LabelIdentifier, span) == 0usize);
    assert!(offset_of!(LabelIdentifier, name) == 8usize);

    assert!(size_of::<ThisExpression>() == 8usize);
    assert!(align_of::<ThisExpression>() == 4usize);
    assert!(offset_of!(ThisExpression, span) == 0usize);

    assert!(size_of::<ArrayExpression>() == 36usize);
    assert!(align_of::<ArrayExpression>() == 4usize);
    assert!(offset_of!(ArrayExpression, span) == 0usize);
    assert!(offset_of!(ArrayExpression, elements) == 8usize);
    assert!(offset_of!(ArrayExpression, trailing_comma) == 24usize);

    assert!(size_of::<ArrayExpressionElement>() == 12usize);
    assert!(align_of::<ArrayExpressionElement>() == 4usize);

    assert!(size_of::<Elision>() == 8usize);
    assert!(align_of::<Elision>() == 4usize);
    assert!(offset_of!(Elision, span) == 0usize);

    assert!(size_of::<ObjectExpression>() == 36usize);
    assert!(align_of::<ObjectExpression>() == 4usize);
    assert!(offset_of!(ObjectExpression, span) == 0usize);
    assert!(offset_of!(ObjectExpression, properties) == 8usize);
    assert!(offset_of!(ObjectExpression, trailing_comma) == 24usize);

    assert!(size_of::<ObjectPropertyKind>() == 8usize);
    assert!(align_of::<ObjectPropertyKind>() == 4usize);

    assert!(size_of::<ObjectProperty>() == 40usize);
    assert!(align_of::<ObjectProperty>() == 4usize);
    assert!(offset_of!(ObjectProperty, span) == 0usize);
    assert!(offset_of!(ObjectProperty, kind) == 8usize);
    assert!(offset_of!(ObjectProperty, key) == 12usize);
    assert!(offset_of!(ObjectProperty, value) == 20usize);
    assert!(offset_of!(ObjectProperty, init) == 28usize);
    assert!(offset_of!(ObjectProperty, method) == 36usize);
    assert!(offset_of!(ObjectProperty, shorthand) == 37usize);
    assert!(offset_of!(ObjectProperty, computed) == 38usize);

    assert!(size_of::<PropertyKey>() == 8usize);
    assert!(align_of::<PropertyKey>() == 4usize);

    assert!(size_of::<PropertyKind>() == 1usize);
    assert!(align_of::<PropertyKind>() == 1usize);

    assert!(size_of::<TemplateLiteral>() == 40usize);
    assert!(align_of::<TemplateLiteral>() == 4usize);
    assert!(offset_of!(TemplateLiteral, span) == 0usize);
    assert!(offset_of!(TemplateLiteral, quasis) == 8usize);
    assert!(offset_of!(TemplateLiteral, expressions) == 24usize);

    assert!(size_of::<TaggedTemplateExpression>() == 60usize);
    assert!(align_of::<TaggedTemplateExpression>() == 4usize);
    assert!(offset_of!(TaggedTemplateExpression, span) == 0usize);
    assert!(offset_of!(TaggedTemplateExpression, tag) == 8usize);
    assert!(offset_of!(TaggedTemplateExpression, quasi) == 16usize);
    assert!(offset_of!(TaggedTemplateExpression, type_parameters) == 56usize);

    assert!(size_of::<TemplateElement>() == 28usize);
    assert!(align_of::<TemplateElement>() == 4usize);
    assert!(offset_of!(TemplateElement, span) == 0usize);
    assert!(offset_of!(TemplateElement, tail) == 8usize);
    assert!(offset_of!(TemplateElement, value) == 12usize);

    assert!(size_of::<TemplateElementValue>() == 16usize);
    assert!(align_of::<TemplateElementValue>() == 4usize);
    assert!(offset_of!(TemplateElementValue, raw) == 0usize);
    assert!(offset_of!(TemplateElementValue, cooked) == 8usize);

    assert!(size_of::<MemberExpression>() == 8usize);
    assert!(align_of::<MemberExpression>() == 4usize);

    assert!(size_of::<ComputedMemberExpression>() == 28usize);
    assert!(align_of::<ComputedMemberExpression>() == 4usize);
    assert!(offset_of!(ComputedMemberExpression, span) == 0usize);
    assert!(offset_of!(ComputedMemberExpression, object) == 8usize);
    assert!(offset_of!(ComputedMemberExpression, expression) == 16usize);
    assert!(offset_of!(ComputedMemberExpression, optional) == 24usize);

    assert!(size_of::<StaticMemberExpression>() == 36usize);
    assert!(align_of::<StaticMemberExpression>() == 4usize);
    assert!(offset_of!(StaticMemberExpression, span) == 0usize);
    assert!(offset_of!(StaticMemberExpression, object) == 8usize);
    assert!(offset_of!(StaticMemberExpression, property) == 16usize);
    assert!(offset_of!(StaticMemberExpression, optional) == 32usize);

    assert!(size_of::<PrivateFieldExpression>() == 36usize);
    assert!(align_of::<PrivateFieldExpression>() == 4usize);
    assert!(offset_of!(PrivateFieldExpression, span) == 0usize);
    assert!(offset_of!(PrivateFieldExpression, object) == 8usize);
    assert!(offset_of!(PrivateFieldExpression, field) == 16usize);
    assert!(offset_of!(PrivateFieldExpression, optional) == 32usize);

    assert!(size_of::<CallExpression>() == 40usize);
    assert!(align_of::<CallExpression>() == 4usize);
    assert!(offset_of!(CallExpression, span) == 0usize);
    assert!(offset_of!(CallExpression, callee) == 8usize);
    assert!(offset_of!(CallExpression, type_parameters) == 16usize);
    assert!(offset_of!(CallExpression, arguments) == 20usize);
    assert!(offset_of!(CallExpression, optional) == 36usize);

    assert!(size_of::<NewExpression>() == 36usize);
    assert!(align_of::<NewExpression>() == 4usize);
    assert!(offset_of!(NewExpression, span) == 0usize);
    assert!(offset_of!(NewExpression, callee) == 8usize);
    assert!(offset_of!(NewExpression, arguments) == 16usize);
    assert!(offset_of!(NewExpression, type_parameters) == 32usize);

    assert!(size_of::<MetaProperty>() == 40usize);
    assert!(align_of::<MetaProperty>() == 4usize);
    assert!(offset_of!(MetaProperty, span) == 0usize);
    assert!(offset_of!(MetaProperty, meta) == 8usize);
    assert!(offset_of!(MetaProperty, property) == 24usize);

    assert!(size_of::<SpreadElement>() == 16usize);
    assert!(align_of::<SpreadElement>() == 4usize);
    assert!(offset_of!(SpreadElement, span) == 0usize);
    assert!(offset_of!(SpreadElement, argument) == 8usize);

    assert!(size_of::<Argument>() == 8usize);
    assert!(align_of::<Argument>() == 4usize);

    assert!(size_of::<UpdateExpression>() == 20usize);
    assert!(align_of::<UpdateExpression>() == 4usize);
    assert!(offset_of!(UpdateExpression, span) == 0usize);
    assert!(offset_of!(UpdateExpression, operator) == 8usize);
    assert!(offset_of!(UpdateExpression, prefix) == 9usize);
    assert!(offset_of!(UpdateExpression, argument) == 12usize);

    assert!(size_of::<UnaryExpression>() == 20usize);
    assert!(align_of::<UnaryExpression>() == 4usize);
    assert!(offset_of!(UnaryExpression, span) == 0usize);
    assert!(offset_of!(UnaryExpression, operator) == 8usize);
    assert!(offset_of!(UnaryExpression, argument) == 12usize);

    assert!(size_of::<BinaryExpression>() == 28usize);
    assert!(align_of::<BinaryExpression>() == 4usize);
    assert!(offset_of!(BinaryExpression, span) == 0usize);
    assert!(offset_of!(BinaryExpression, left) == 8usize);
    assert!(offset_of!(BinaryExpression, operator) == 16usize);
    assert!(offset_of!(BinaryExpression, right) == 20usize);

    assert!(size_of::<PrivateInExpression>() == 36usize);
    assert!(align_of::<PrivateInExpression>() == 4usize);
    assert!(offset_of!(PrivateInExpression, span) == 0usize);
    assert!(offset_of!(PrivateInExpression, left) == 8usize);
    assert!(offset_of!(PrivateInExpression, operator) == 24usize);
    assert!(offset_of!(PrivateInExpression, right) == 28usize);

    assert!(size_of::<LogicalExpression>() == 28usize);
    assert!(align_of::<LogicalExpression>() == 4usize);
    assert!(offset_of!(LogicalExpression, span) == 0usize);
    assert!(offset_of!(LogicalExpression, left) == 8usize);
    assert!(offset_of!(LogicalExpression, operator) == 16usize);
    assert!(offset_of!(LogicalExpression, right) == 20usize);

    assert!(size_of::<ConditionalExpression>() == 32usize);
    assert!(align_of::<ConditionalExpression>() == 4usize);
    assert!(offset_of!(ConditionalExpression, span) == 0usize);
    assert!(offset_of!(ConditionalExpression, test) == 8usize);
    assert!(offset_of!(ConditionalExpression, consequent) == 16usize);
    assert!(offset_of!(ConditionalExpression, alternate) == 24usize);

    assert!(size_of::<AssignmentExpression>() == 28usize);
    assert!(align_of::<AssignmentExpression>() == 4usize);
    assert!(offset_of!(AssignmentExpression, span) == 0usize);
    assert!(offset_of!(AssignmentExpression, operator) == 8usize);
    assert!(offset_of!(AssignmentExpression, left) == 12usize);
    assert!(offset_of!(AssignmentExpression, right) == 20usize);

    assert!(size_of::<AssignmentTarget>() == 8usize);
    assert!(align_of::<AssignmentTarget>() == 4usize);

    assert!(size_of::<SimpleAssignmentTarget>() == 8usize);
    assert!(align_of::<SimpleAssignmentTarget>() == 4usize);

    assert!(size_of::<AssignmentTargetPattern>() == 8usize);
    assert!(align_of::<AssignmentTargetPattern>() == 4usize);

    assert!(size_of::<ArrayAssignmentTarget>() == 52usize);
    assert!(align_of::<ArrayAssignmentTarget>() == 4usize);
    assert!(offset_of!(ArrayAssignmentTarget, span) == 0usize);
    assert!(offset_of!(ArrayAssignmentTarget, elements) == 8usize);
    assert!(offset_of!(ArrayAssignmentTarget, rest) == 24usize);
    assert!(offset_of!(ArrayAssignmentTarget, trailing_comma) == 40usize);

    assert!(size_of::<ObjectAssignmentTarget>() == 40usize);
    assert!(align_of::<ObjectAssignmentTarget>() == 4usize);
    assert!(offset_of!(ObjectAssignmentTarget, span) == 0usize);
    assert!(offset_of!(ObjectAssignmentTarget, properties) == 8usize);
    assert!(offset_of!(ObjectAssignmentTarget, rest) == 24usize);

    assert!(size_of::<AssignmentTargetRest>() == 16usize);
    assert!(align_of::<AssignmentTargetRest>() == 4usize);
    assert!(offset_of!(AssignmentTargetRest, span) == 0usize);
    assert!(offset_of!(AssignmentTargetRest, target) == 8usize);

    assert!(size_of::<AssignmentTargetMaybeDefault>() == 8usize);
    assert!(align_of::<AssignmentTargetMaybeDefault>() == 4usize);

    assert!(size_of::<AssignmentTargetWithDefault>() == 24usize);
    assert!(align_of::<AssignmentTargetWithDefault>() == 4usize);
    assert!(offset_of!(AssignmentTargetWithDefault, span) == 0usize);
    assert!(offset_of!(AssignmentTargetWithDefault, binding) == 8usize);
    assert!(offset_of!(AssignmentTargetWithDefault, init) == 16usize);

    assert!(size_of::<AssignmentTargetProperty>() == 8usize);
    assert!(align_of::<AssignmentTargetProperty>() == 4usize);

    assert!(size_of::<AssignmentTargetPropertyIdentifier>() == 36usize);
    assert!(align_of::<AssignmentTargetPropertyIdentifier>() == 4usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, span) == 0usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, binding) == 8usize);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, init) == 28usize);

    assert!(size_of::<AssignmentTargetPropertyProperty>() == 24usize);
    assert!(align_of::<AssignmentTargetPropertyProperty>() == 4usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, span) == 0usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, name) == 8usize);
    assert!(offset_of!(AssignmentTargetPropertyProperty, binding) == 16usize);

    assert!(size_of::<SequenceExpression>() == 24usize);
    assert!(align_of::<SequenceExpression>() == 4usize);
    assert!(offset_of!(SequenceExpression, span) == 0usize);
    assert!(offset_of!(SequenceExpression, expressions) == 8usize);

    assert!(size_of::<Super>() == 8usize);
    assert!(align_of::<Super>() == 4usize);
    assert!(offset_of!(Super, span) == 0usize);

    assert!(size_of::<AwaitExpression>() == 16usize);
    assert!(align_of::<AwaitExpression>() == 4usize);
    assert!(offset_of!(AwaitExpression, span) == 0usize);
    assert!(offset_of!(AwaitExpression, argument) == 8usize);

    assert!(size_of::<ChainExpression>() == 16usize);
    assert!(align_of::<ChainExpression>() == 4usize);
    assert!(offset_of!(ChainExpression, span) == 0usize);
    assert!(offset_of!(ChainExpression, expression) == 8usize);

    assert!(size_of::<ChainElement>() == 8usize);
    assert!(align_of::<ChainElement>() == 4usize);

    assert!(size_of::<ParenthesizedExpression>() == 16usize);
    assert!(align_of::<ParenthesizedExpression>() == 4usize);
    assert!(offset_of!(ParenthesizedExpression, span) == 0usize);
    assert!(offset_of!(ParenthesizedExpression, expression) == 8usize);

    assert!(size_of::<Statement>() == 8usize);
    assert!(align_of::<Statement>() == 4usize);

    assert!(size_of::<Directive>() == 32usize);
    assert!(align_of::<Directive>() == 4usize);
    assert!(offset_of!(Directive, span) == 0usize);
    assert!(offset_of!(Directive, expression) == 8usize);
    assert!(offset_of!(Directive, directive) == 24usize);

    assert!(size_of::<Hashbang>() == 16usize);
    assert!(align_of::<Hashbang>() == 4usize);
    assert!(offset_of!(Hashbang, span) == 0usize);
    assert!(offset_of!(Hashbang, value) == 8usize);

    assert!(size_of::<BlockStatement>() == 28usize);
    assert!(align_of::<BlockStatement>() == 4usize);
    assert!(offset_of!(BlockStatement, span) == 0usize);
    assert!(offset_of!(BlockStatement, body) == 8usize);
    assert!(offset_of!(BlockStatement, scope_id) == 24usize);

    assert!(size_of::<Declaration>() == 8usize);
    assert!(align_of::<Declaration>() == 4usize);

    assert!(size_of::<VariableDeclaration>() == 32usize);
    assert!(align_of::<VariableDeclaration>() == 4usize);
    assert!(offset_of!(VariableDeclaration, span) == 0usize);
    assert!(offset_of!(VariableDeclaration, kind) == 8usize);
    assert!(offset_of!(VariableDeclaration, declarations) == 12usize);
    assert!(offset_of!(VariableDeclaration, declare) == 28usize);

    assert!(size_of::<VariableDeclarationKind>() == 1usize);
    assert!(align_of::<VariableDeclarationKind>() == 1usize);

    assert!(size_of::<VariableDeclarator>() == 40usize);
    assert!(align_of::<VariableDeclarator>() == 4usize);
    assert!(offset_of!(VariableDeclarator, span) == 0usize);
    assert!(offset_of!(VariableDeclarator, kind) == 8usize);
    assert!(offset_of!(VariableDeclarator, id) == 12usize);
    assert!(offset_of!(VariableDeclarator, init) == 28usize);
    assert!(offset_of!(VariableDeclarator, definite) == 36usize);

    assert!(size_of::<EmptyStatement>() == 8usize);
    assert!(align_of::<EmptyStatement>() == 4usize);
    assert!(offset_of!(EmptyStatement, span) == 0usize);

    assert!(size_of::<ExpressionStatement>() == 16usize);
    assert!(align_of::<ExpressionStatement>() == 4usize);
    assert!(offset_of!(ExpressionStatement, span) == 0usize);
    assert!(offset_of!(ExpressionStatement, expression) == 8usize);

    assert!(size_of::<IfStatement>() == 32usize);
    assert!(align_of::<IfStatement>() == 4usize);
    assert!(offset_of!(IfStatement, span) == 0usize);
    assert!(offset_of!(IfStatement, test) == 8usize);
    assert!(offset_of!(IfStatement, consequent) == 16usize);
    assert!(offset_of!(IfStatement, alternate) == 24usize);

    assert!(size_of::<DoWhileStatement>() == 24usize);
    assert!(align_of::<DoWhileStatement>() == 4usize);
    assert!(offset_of!(DoWhileStatement, span) == 0usize);
    assert!(offset_of!(DoWhileStatement, body) == 8usize);
    assert!(offset_of!(DoWhileStatement, test) == 16usize);

    assert!(size_of::<WhileStatement>() == 24usize);
    assert!(align_of::<WhileStatement>() == 4usize);
    assert!(offset_of!(WhileStatement, span) == 0usize);
    assert!(offset_of!(WhileStatement, test) == 8usize);
    assert!(offset_of!(WhileStatement, body) == 16usize);

    assert!(size_of::<ForStatement>() == 44usize);
    assert!(align_of::<ForStatement>() == 4usize);
    assert!(offset_of!(ForStatement, span) == 0usize);
    assert!(offset_of!(ForStatement, init) == 8usize);
    assert!(offset_of!(ForStatement, test) == 16usize);
    assert!(offset_of!(ForStatement, update) == 24usize);
    assert!(offset_of!(ForStatement, body) == 32usize);
    assert!(offset_of!(ForStatement, scope_id) == 40usize);

    assert!(size_of::<ForStatementInit>() == 8usize);
    assert!(align_of::<ForStatementInit>() == 4usize);

    assert!(size_of::<ForInStatement>() == 36usize);
    assert!(align_of::<ForInStatement>() == 4usize);
    assert!(offset_of!(ForInStatement, span) == 0usize);
    assert!(offset_of!(ForInStatement, left) == 8usize);
    assert!(offset_of!(ForInStatement, right) == 16usize);
    assert!(offset_of!(ForInStatement, body) == 24usize);
    assert!(offset_of!(ForInStatement, scope_id) == 32usize);

    assert!(size_of::<ForStatementLeft>() == 8usize);
    assert!(align_of::<ForStatementLeft>() == 4usize);

    assert!(size_of::<ForOfStatement>() == 40usize);
    assert!(align_of::<ForOfStatement>() == 4usize);
    assert!(offset_of!(ForOfStatement, span) == 0usize);
    assert!(offset_of!(ForOfStatement, r#await) == 8usize);
    assert!(offset_of!(ForOfStatement, left) == 12usize);
    assert!(offset_of!(ForOfStatement, right) == 20usize);
    assert!(offset_of!(ForOfStatement, body) == 28usize);
    assert!(offset_of!(ForOfStatement, scope_id) == 36usize);

    assert!(size_of::<ContinueStatement>() == 24usize);
    assert!(align_of::<ContinueStatement>() == 4usize);
    assert!(offset_of!(ContinueStatement, span) == 0usize);
    assert!(offset_of!(ContinueStatement, label) == 8usize);

    assert!(size_of::<BreakStatement>() == 24usize);
    assert!(align_of::<BreakStatement>() == 4usize);
    assert!(offset_of!(BreakStatement, span) == 0usize);
    assert!(offset_of!(BreakStatement, label) == 8usize);

    assert!(size_of::<ReturnStatement>() == 16usize);
    assert!(align_of::<ReturnStatement>() == 4usize);
    assert!(offset_of!(ReturnStatement, span) == 0usize);
    assert!(offset_of!(ReturnStatement, argument) == 8usize);

    assert!(size_of::<WithStatement>() == 24usize);
    assert!(align_of::<WithStatement>() == 4usize);
    assert!(offset_of!(WithStatement, span) == 0usize);
    assert!(offset_of!(WithStatement, object) == 8usize);
    assert!(offset_of!(WithStatement, body) == 16usize);

    assert!(size_of::<SwitchStatement>() == 36usize);
    assert!(align_of::<SwitchStatement>() == 4usize);
    assert!(offset_of!(SwitchStatement, span) == 0usize);
    assert!(offset_of!(SwitchStatement, discriminant) == 8usize);
    assert!(offset_of!(SwitchStatement, cases) == 16usize);
    assert!(offset_of!(SwitchStatement, scope_id) == 32usize);

    assert!(size_of::<SwitchCase>() == 32usize);
    assert!(align_of::<SwitchCase>() == 4usize);
    assert!(offset_of!(SwitchCase, span) == 0usize);
    assert!(offset_of!(SwitchCase, test) == 8usize);
    assert!(offset_of!(SwitchCase, consequent) == 16usize);

    assert!(size_of::<LabeledStatement>() == 32usize);
    assert!(align_of::<LabeledStatement>() == 4usize);
    assert!(offset_of!(LabeledStatement, span) == 0usize);
    assert!(offset_of!(LabeledStatement, label) == 8usize);
    assert!(offset_of!(LabeledStatement, body) == 24usize);

    assert!(size_of::<ThrowStatement>() == 16usize);
    assert!(align_of::<ThrowStatement>() == 4usize);
    assert!(offset_of!(ThrowStatement, span) == 0usize);
    assert!(offset_of!(ThrowStatement, argument) == 8usize);

    assert!(size_of::<TryStatement>() == 20usize);
    assert!(align_of::<TryStatement>() == 4usize);
    assert!(offset_of!(TryStatement, span) == 0usize);
    assert!(offset_of!(TryStatement, block) == 8usize);
    assert!(offset_of!(TryStatement, handler) == 12usize);
    assert!(offset_of!(TryStatement, finalizer) == 16usize);

    assert!(size_of::<CatchClause>() == 40usize);
    assert!(align_of::<CatchClause>() == 4usize);
    assert!(offset_of!(CatchClause, span) == 0usize);
    assert!(offset_of!(CatchClause, param) == 8usize);
    assert!(offset_of!(CatchClause, body) == 32usize);
    assert!(offset_of!(CatchClause, scope_id) == 36usize);

    assert!(size_of::<CatchParameter>() == 24usize);
    assert!(align_of::<CatchParameter>() == 4usize);
    assert!(offset_of!(CatchParameter, span) == 0usize);
    assert!(offset_of!(CatchParameter, pattern) == 8usize);

    assert!(size_of::<DebuggerStatement>() == 8usize);
    assert!(align_of::<DebuggerStatement>() == 4usize);
    assert!(offset_of!(DebuggerStatement, span) == 0usize);

    assert!(size_of::<BindingPattern>() == 16usize);
    assert!(align_of::<BindingPattern>() == 4usize);
    assert!(offset_of!(BindingPattern, kind) == 0usize);
    assert!(offset_of!(BindingPattern, type_annotation) == 8usize);
    assert!(offset_of!(BindingPattern, optional) == 12usize);

    assert!(size_of::<BindingPatternKind>() == 8usize);
    assert!(align_of::<BindingPatternKind>() == 4usize);

    assert!(size_of::<AssignmentPattern>() == 32usize);
    assert!(align_of::<AssignmentPattern>() == 4usize);
    assert!(offset_of!(AssignmentPattern, span) == 0usize);
    assert!(offset_of!(AssignmentPattern, left) == 8usize);
    assert!(offset_of!(AssignmentPattern, right) == 24usize);

    assert!(size_of::<ObjectPattern>() == 28usize);
    assert!(align_of::<ObjectPattern>() == 4usize);
    assert!(offset_of!(ObjectPattern, span) == 0usize);
    assert!(offset_of!(ObjectPattern, properties) == 8usize);
    assert!(offset_of!(ObjectPattern, rest) == 24usize);

    assert!(size_of::<BindingProperty>() == 36usize);
    assert!(align_of::<BindingProperty>() == 4usize);
    assert!(offset_of!(BindingProperty, span) == 0usize);
    assert!(offset_of!(BindingProperty, key) == 8usize);
    assert!(offset_of!(BindingProperty, value) == 16usize);
    assert!(offset_of!(BindingProperty, shorthand) == 32usize);
    assert!(offset_of!(BindingProperty, computed) == 33usize);

    assert!(size_of::<ArrayPattern>() == 28usize);
    assert!(align_of::<ArrayPattern>() == 4usize);
    assert!(offset_of!(ArrayPattern, span) == 0usize);
    assert!(offset_of!(ArrayPattern, elements) == 8usize);
    assert!(offset_of!(ArrayPattern, rest) == 24usize);

    assert!(size_of::<BindingRestElement>() == 24usize);
    assert!(align_of::<BindingRestElement>() == 4usize);
    assert!(offset_of!(BindingRestElement, span) == 0usize);
    assert!(offset_of!(BindingRestElement, argument) == 8usize);

    assert!(size_of::<Function>() == 60usize);
    assert!(align_of::<Function>() == 4usize);
    assert!(offset_of!(Function, r#type) == 0usize);
    assert!(offset_of!(Function, span) == 4usize);
    assert!(offset_of!(Function, id) == 12usize);
    assert!(offset_of!(Function, generator) == 32usize);
    assert!(offset_of!(Function, r#async) == 33usize);
    assert!(offset_of!(Function, declare) == 34usize);
    assert!(offset_of!(Function, type_parameters) == 36usize);
    assert!(offset_of!(Function, this_param) == 40usize);
    assert!(offset_of!(Function, params) == 44usize);
    assert!(offset_of!(Function, return_type) == 48usize);
    assert!(offset_of!(Function, body) == 52usize);
    assert!(offset_of!(Function, scope_id) == 56usize);

    assert!(size_of::<FunctionType>() == 1usize);
    assert!(align_of::<FunctionType>() == 1usize);

    assert!(size_of::<FormalParameters>() == 32usize);
    assert!(align_of::<FormalParameters>() == 4usize);
    assert!(offset_of!(FormalParameters, span) == 0usize);
    assert!(offset_of!(FormalParameters, kind) == 8usize);
    assert!(offset_of!(FormalParameters, items) == 12usize);
    assert!(offset_of!(FormalParameters, rest) == 28usize);

    assert!(size_of::<FormalParameter>() == 44usize);
    assert!(align_of::<FormalParameter>() == 4usize);
    assert!(offset_of!(FormalParameter, span) == 0usize);
    assert!(offset_of!(FormalParameter, decorators) == 8usize);
    assert!(offset_of!(FormalParameter, pattern) == 24usize);
    assert!(offset_of!(FormalParameter, accessibility) == 40usize);
    assert!(offset_of!(FormalParameter, readonly) == 41usize);
    assert!(offset_of!(FormalParameter, r#override) == 42usize);

    assert!(size_of::<FormalParameterKind>() == 1usize);
    assert!(align_of::<FormalParameterKind>() == 1usize);

    assert!(size_of::<FunctionBody>() == 40usize);
    assert!(align_of::<FunctionBody>() == 4usize);
    assert!(offset_of!(FunctionBody, span) == 0usize);
    assert!(offset_of!(FunctionBody, directives) == 8usize);
    assert!(offset_of!(FunctionBody, statements) == 24usize);

    assert!(size_of::<ArrowFunctionExpression>() == 32usize);
    assert!(align_of::<ArrowFunctionExpression>() == 4usize);
    assert!(offset_of!(ArrowFunctionExpression, span) == 0usize);
    assert!(offset_of!(ArrowFunctionExpression, expression) == 8usize);
    assert!(offset_of!(ArrowFunctionExpression, r#async) == 9usize);
    assert!(offset_of!(ArrowFunctionExpression, type_parameters) == 12usize);
    assert!(offset_of!(ArrowFunctionExpression, params) == 16usize);
    assert!(offset_of!(ArrowFunctionExpression, return_type) == 20usize);
    assert!(offset_of!(ArrowFunctionExpression, body) == 24usize);
    assert!(offset_of!(ArrowFunctionExpression, scope_id) == 28usize);

    assert!(size_of::<YieldExpression>() == 20usize);
    assert!(align_of::<YieldExpression>() == 4usize);
    assert!(offset_of!(YieldExpression, span) == 0usize);
    assert!(offset_of!(YieldExpression, delegate) == 8usize);
    assert!(offset_of!(YieldExpression, argument) == 12usize);

    assert!(size_of::<Class>() == 92usize);
    assert!(align_of::<Class>() == 4usize);
    assert!(offset_of!(Class, r#type) == 0usize);
    assert!(offset_of!(Class, span) == 4usize);
    assert!(offset_of!(Class, decorators) == 12usize);
    assert!(offset_of!(Class, id) == 28usize);
    assert!(offset_of!(Class, type_parameters) == 48usize);
    assert!(offset_of!(Class, super_class) == 52usize);
    assert!(offset_of!(Class, super_type_parameters) == 60usize);
    assert!(offset_of!(Class, implements) == 64usize);
    assert!(offset_of!(Class, body) == 80usize);
    assert!(offset_of!(Class, r#abstract) == 84usize);
    assert!(offset_of!(Class, declare) == 85usize);
    assert!(offset_of!(Class, scope_id) == 88usize);

    assert!(size_of::<ClassType>() == 1usize);
    assert!(align_of::<ClassType>() == 1usize);

    assert!(size_of::<ClassBody>() == 24usize);
    assert!(align_of::<ClassBody>() == 4usize);
    assert!(offset_of!(ClassBody, span) == 0usize);
    assert!(offset_of!(ClassBody, body) == 8usize);

    assert!(size_of::<ClassElement>() == 8usize);
    assert!(align_of::<ClassElement>() == 4usize);

    assert!(size_of::<MethodDefinition>() == 48usize);
    assert!(align_of::<MethodDefinition>() == 4usize);
    assert!(offset_of!(MethodDefinition, r#type) == 0usize);
    assert!(offset_of!(MethodDefinition, span) == 4usize);
    assert!(offset_of!(MethodDefinition, decorators) == 12usize);
    assert!(offset_of!(MethodDefinition, key) == 28usize);
    assert!(offset_of!(MethodDefinition, value) == 36usize);
    assert!(offset_of!(MethodDefinition, kind) == 40usize);
    assert!(offset_of!(MethodDefinition, computed) == 41usize);
    assert!(offset_of!(MethodDefinition, r#static) == 42usize);
    assert!(offset_of!(MethodDefinition, r#override) == 43usize);
    assert!(offset_of!(MethodDefinition, optional) == 44usize);
    assert!(offset_of!(MethodDefinition, accessibility) == 45usize);

    assert!(size_of::<MethodDefinitionType>() == 1usize);
    assert!(align_of::<MethodDefinitionType>() == 1usize);

    assert!(size_of::<PropertyDefinition>() == 60usize);
    assert!(align_of::<PropertyDefinition>() == 4usize);
    assert!(offset_of!(PropertyDefinition, r#type) == 0usize);
    assert!(offset_of!(PropertyDefinition, span) == 4usize);
    assert!(offset_of!(PropertyDefinition, decorators) == 12usize);
    assert!(offset_of!(PropertyDefinition, key) == 28usize);
    assert!(offset_of!(PropertyDefinition, value) == 36usize);
    assert!(offset_of!(PropertyDefinition, computed) == 44usize);
    assert!(offset_of!(PropertyDefinition, r#static) == 45usize);
    assert!(offset_of!(PropertyDefinition, declare) == 46usize);
    assert!(offset_of!(PropertyDefinition, r#override) == 47usize);
    assert!(offset_of!(PropertyDefinition, optional) == 48usize);
    assert!(offset_of!(PropertyDefinition, definite) == 49usize);
    assert!(offset_of!(PropertyDefinition, readonly) == 50usize);
    assert!(offset_of!(PropertyDefinition, type_annotation) == 52usize);
    assert!(offset_of!(PropertyDefinition, accessibility) == 56usize);

    assert!(size_of::<PropertyDefinitionType>() == 1usize);
    assert!(align_of::<PropertyDefinitionType>() == 1usize);

    assert!(size_of::<MethodDefinitionKind>() == 1usize);
    assert!(align_of::<MethodDefinitionKind>() == 1usize);

    assert!(size_of::<PrivateIdentifier>() == 16usize);
    assert!(align_of::<PrivateIdentifier>() == 4usize);
    assert!(offset_of!(PrivateIdentifier, span) == 0usize);
    assert!(offset_of!(PrivateIdentifier, name) == 8usize);

    assert!(size_of::<StaticBlock>() == 28usize);
    assert!(align_of::<StaticBlock>() == 4usize);
    assert!(offset_of!(StaticBlock, span) == 0usize);
    assert!(offset_of!(StaticBlock, body) == 8usize);
    assert!(offset_of!(StaticBlock, scope_id) == 24usize);

    assert!(size_of::<ModuleDeclaration>() == 8usize);
    assert!(align_of::<ModuleDeclaration>() == 4usize);

    assert!(size_of::<AccessorPropertyType>() == 1usize);
    assert!(align_of::<AccessorPropertyType>() == 1usize);

    assert!(size_of::<AccessorProperty>() == 56usize);
    assert!(align_of::<AccessorProperty>() == 4usize);
    assert!(offset_of!(AccessorProperty, r#type) == 0usize);
    assert!(offset_of!(AccessorProperty, span) == 4usize);
    assert!(offset_of!(AccessorProperty, decorators) == 12usize);
    assert!(offset_of!(AccessorProperty, key) == 28usize);
    assert!(offset_of!(AccessorProperty, value) == 36usize);
    assert!(offset_of!(AccessorProperty, computed) == 44usize);
    assert!(offset_of!(AccessorProperty, r#static) == 45usize);
    assert!(offset_of!(AccessorProperty, definite) == 46usize);
    assert!(offset_of!(AccessorProperty, type_annotation) == 48usize);
    assert!(offset_of!(AccessorProperty, accessibility) == 52usize);

    assert!(size_of::<ImportExpression>() == 32usize);
    assert!(align_of::<ImportExpression>() == 4usize);
    assert!(offset_of!(ImportExpression, span) == 0usize);
    assert!(offset_of!(ImportExpression, source) == 8usize);
    assert!(offset_of!(ImportExpression, arguments) == 16usize);

    assert!(size_of::<ImportDeclaration>() == 48usize);
    assert!(align_of::<ImportDeclaration>() == 4usize);
    assert!(offset_of!(ImportDeclaration, span) == 0usize);
    assert!(offset_of!(ImportDeclaration, specifiers) == 8usize);
    assert!(offset_of!(ImportDeclaration, source) == 24usize);
    assert!(offset_of!(ImportDeclaration, with_clause) == 40usize);
    assert!(offset_of!(ImportDeclaration, import_kind) == 44usize);

    assert!(size_of::<ImportDeclarationSpecifier>() == 8usize);
    assert!(align_of::<ImportDeclarationSpecifier>() == 4usize);

    assert!(size_of::<ImportSpecifier>() == 56usize);
    assert!(align_of::<ImportSpecifier>() == 4usize);
    assert!(offset_of!(ImportSpecifier, span) == 0usize);
    assert!(offset_of!(ImportSpecifier, imported) == 8usize);
    assert!(offset_of!(ImportSpecifier, local) == 32usize);
    assert!(offset_of!(ImportSpecifier, import_kind) == 52usize);

    assert!(size_of::<ImportDefaultSpecifier>() == 28usize);
    assert!(align_of::<ImportDefaultSpecifier>() == 4usize);
    assert!(offset_of!(ImportDefaultSpecifier, span) == 0usize);
    assert!(offset_of!(ImportDefaultSpecifier, local) == 8usize);

    assert!(size_of::<ImportNamespaceSpecifier>() == 28usize);
    assert!(align_of::<ImportNamespaceSpecifier>() == 4usize);
    assert!(offset_of!(ImportNamespaceSpecifier, span) == 0usize);
    assert!(offset_of!(ImportNamespaceSpecifier, local) == 8usize);

    assert!(size_of::<WithClause>() == 40usize);
    assert!(align_of::<WithClause>() == 4usize);
    assert!(offset_of!(WithClause, span) == 0usize);
    assert!(offset_of!(WithClause, attributes_keyword) == 8usize);
    assert!(offset_of!(WithClause, with_entries) == 24usize);

    assert!(size_of::<ImportAttribute>() == 44usize);
    assert!(align_of::<ImportAttribute>() == 4usize);
    assert!(offset_of!(ImportAttribute, span) == 0usize);
    assert!(offset_of!(ImportAttribute, key) == 8usize);
    assert!(offset_of!(ImportAttribute, value) == 28usize);

    assert!(size_of::<ImportAttributeKey>() == 20usize);
    assert!(align_of::<ImportAttributeKey>() == 4usize);

    assert!(size_of::<ExportNamedDeclaration>() == 56usize);
    assert!(align_of::<ExportNamedDeclaration>() == 4usize);
    assert!(offset_of!(ExportNamedDeclaration, span) == 0usize);
    assert!(offset_of!(ExportNamedDeclaration, declaration) == 8usize);
    assert!(offset_of!(ExportNamedDeclaration, specifiers) == 16usize);
    assert!(offset_of!(ExportNamedDeclaration, source) == 32usize);
    assert!(offset_of!(ExportNamedDeclaration, export_kind) == 48usize);
    assert!(offset_of!(ExportNamedDeclaration, with_clause) == 52usize);

    assert!(size_of::<ExportDefaultDeclaration>() == 40usize);
    assert!(align_of::<ExportDefaultDeclaration>() == 4usize);
    assert!(offset_of!(ExportDefaultDeclaration, span) == 0usize);
    assert!(offset_of!(ExportDefaultDeclaration, declaration) == 8usize);
    assert!(offset_of!(ExportDefaultDeclaration, exported) == 16usize);

    assert!(size_of::<ExportAllDeclaration>() == 56usize);
    assert!(align_of::<ExportAllDeclaration>() == 4usize);
    assert!(offset_of!(ExportAllDeclaration, span) == 0usize);
    assert!(offset_of!(ExportAllDeclaration, exported) == 8usize);
    assert!(offset_of!(ExportAllDeclaration, source) == 32usize);
    assert!(offset_of!(ExportAllDeclaration, with_clause) == 48usize);
    assert!(offset_of!(ExportAllDeclaration, export_kind) == 52usize);

    assert!(size_of::<ExportSpecifier>() == 60usize);
    assert!(align_of::<ExportSpecifier>() == 4usize);
    assert!(offset_of!(ExportSpecifier, span) == 0usize);
    assert!(offset_of!(ExportSpecifier, local) == 8usize);
    assert!(offset_of!(ExportSpecifier, exported) == 32usize);
    assert!(offset_of!(ExportSpecifier, export_kind) == 56usize);

    assert!(size_of::<ExportDefaultDeclarationKind>() == 8usize);
    assert!(align_of::<ExportDefaultDeclarationKind>() == 4usize);

    assert!(size_of::<ModuleExportName>() == 24usize);
    assert!(align_of::<ModuleExportName>() == 4usize);

    assert!(size_of::<TSThisParameter>() == 20usize);
    assert!(align_of::<TSThisParameter>() == 4usize);
    assert!(offset_of!(TSThisParameter, span) == 0usize);
    assert!(offset_of!(TSThisParameter, this_span) == 8usize);
    assert!(offset_of!(TSThisParameter, type_annotation) == 16usize);

    assert!(size_of::<TSEnumDeclaration>() == 52usize);
    assert!(align_of::<TSEnumDeclaration>() == 4usize);
    assert!(offset_of!(TSEnumDeclaration, span) == 0usize);
    assert!(offset_of!(TSEnumDeclaration, id) == 8usize);
    assert!(offset_of!(TSEnumDeclaration, members) == 28usize);
    assert!(offset_of!(TSEnumDeclaration, r#const) == 44usize);
    assert!(offset_of!(TSEnumDeclaration, declare) == 45usize);
    assert!(offset_of!(TSEnumDeclaration, scope_id) == 48usize);

    assert!(size_of::<TSEnumMember>() == 24usize);
    assert!(align_of::<TSEnumMember>() == 4usize);
    assert!(offset_of!(TSEnumMember, span) == 0usize);
    assert!(offset_of!(TSEnumMember, id) == 8usize);
    assert!(offset_of!(TSEnumMember, initializer) == 16usize);

    assert!(size_of::<TSEnumMemberName>() == 8usize);
    assert!(align_of::<TSEnumMemberName>() == 4usize);

    assert!(size_of::<TSTypeAnnotation>() == 16usize);
    assert!(align_of::<TSTypeAnnotation>() == 4usize);
    assert!(offset_of!(TSTypeAnnotation, span) == 0usize);
    assert!(offset_of!(TSTypeAnnotation, type_annotation) == 8usize);

    assert!(size_of::<TSLiteralType>() == 16usize);
    assert!(align_of::<TSLiteralType>() == 4usize);
    assert!(offset_of!(TSLiteralType, span) == 0usize);
    assert!(offset_of!(TSLiteralType, literal) == 8usize);

    assert!(size_of::<TSLiteral>() == 8usize);
    assert!(align_of::<TSLiteral>() == 4usize);

    assert!(size_of::<TSType>() == 8usize);
    assert!(align_of::<TSType>() == 4usize);

    assert!(size_of::<TSConditionalType>() == 44usize);
    assert!(align_of::<TSConditionalType>() == 4usize);
    assert!(offset_of!(TSConditionalType, span) == 0usize);
    assert!(offset_of!(TSConditionalType, check_type) == 8usize);
    assert!(offset_of!(TSConditionalType, extends_type) == 16usize);
    assert!(offset_of!(TSConditionalType, true_type) == 24usize);
    assert!(offset_of!(TSConditionalType, false_type) == 32usize);
    assert!(offset_of!(TSConditionalType, scope_id) == 40usize);

    assert!(size_of::<TSUnionType>() == 24usize);
    assert!(align_of::<TSUnionType>() == 4usize);
    assert!(offset_of!(TSUnionType, span) == 0usize);
    assert!(offset_of!(TSUnionType, types) == 8usize);

    assert!(size_of::<TSIntersectionType>() == 24usize);
    assert!(align_of::<TSIntersectionType>() == 4usize);
    assert!(offset_of!(TSIntersectionType, span) == 0usize);
    assert!(offset_of!(TSIntersectionType, types) == 8usize);

    assert!(size_of::<TSParenthesizedType>() == 16usize);
    assert!(align_of::<TSParenthesizedType>() == 4usize);
    assert!(offset_of!(TSParenthesizedType, span) == 0usize);
    assert!(offset_of!(TSParenthesizedType, type_annotation) == 8usize);

    assert!(size_of::<TSTypeOperator>() == 20usize);
    assert!(align_of::<TSTypeOperator>() == 4usize);
    assert!(offset_of!(TSTypeOperator, span) == 0usize);
    assert!(offset_of!(TSTypeOperator, operator) == 8usize);
    assert!(offset_of!(TSTypeOperator, type_annotation) == 12usize);

    assert!(size_of::<TSTypeOperatorOperator>() == 1usize);
    assert!(align_of::<TSTypeOperatorOperator>() == 1usize);

    assert!(size_of::<TSArrayType>() == 16usize);
    assert!(align_of::<TSArrayType>() == 4usize);
    assert!(offset_of!(TSArrayType, span) == 0usize);
    assert!(offset_of!(TSArrayType, element_type) == 8usize);

    assert!(size_of::<TSIndexedAccessType>() == 24usize);
    assert!(align_of::<TSIndexedAccessType>() == 4usize);
    assert!(offset_of!(TSIndexedAccessType, span) == 0usize);
    assert!(offset_of!(TSIndexedAccessType, object_type) == 8usize);
    assert!(offset_of!(TSIndexedAccessType, index_type) == 16usize);

    assert!(size_of::<TSTupleType>() == 24usize);
    assert!(align_of::<TSTupleType>() == 4usize);
    assert!(offset_of!(TSTupleType, span) == 0usize);
    assert!(offset_of!(TSTupleType, element_types) == 8usize);

    assert!(size_of::<TSNamedTupleMember>() == 36usize);
    assert!(align_of::<TSNamedTupleMember>() == 4usize);
    assert!(offset_of!(TSNamedTupleMember, span) == 0usize);
    assert!(offset_of!(TSNamedTupleMember, element_type) == 8usize);
    assert!(offset_of!(TSNamedTupleMember, label) == 16usize);
    assert!(offset_of!(TSNamedTupleMember, optional) == 32usize);

    assert!(size_of::<TSOptionalType>() == 16usize);
    assert!(align_of::<TSOptionalType>() == 4usize);
    assert!(offset_of!(TSOptionalType, span) == 0usize);
    assert!(offset_of!(TSOptionalType, type_annotation) == 8usize);

    assert!(size_of::<TSRestType>() == 16usize);
    assert!(align_of::<TSRestType>() == 4usize);
    assert!(offset_of!(TSRestType, span) == 0usize);
    assert!(offset_of!(TSRestType, type_annotation) == 8usize);

    assert!(size_of::<TSTupleElement>() == 8usize);
    assert!(align_of::<TSTupleElement>() == 4usize);

    assert!(size_of::<TSAnyKeyword>() == 8usize);
    assert!(align_of::<TSAnyKeyword>() == 4usize);
    assert!(offset_of!(TSAnyKeyword, span) == 0usize);

    assert!(size_of::<TSStringKeyword>() == 8usize);
    assert!(align_of::<TSStringKeyword>() == 4usize);
    assert!(offset_of!(TSStringKeyword, span) == 0usize);

    assert!(size_of::<TSBooleanKeyword>() == 8usize);
    assert!(align_of::<TSBooleanKeyword>() == 4usize);
    assert!(offset_of!(TSBooleanKeyword, span) == 0usize);

    assert!(size_of::<TSNumberKeyword>() == 8usize);
    assert!(align_of::<TSNumberKeyword>() == 4usize);
    assert!(offset_of!(TSNumberKeyword, span) == 0usize);

    assert!(size_of::<TSNeverKeyword>() == 8usize);
    assert!(align_of::<TSNeverKeyword>() == 4usize);
    assert!(offset_of!(TSNeverKeyword, span) == 0usize);

    assert!(size_of::<TSIntrinsicKeyword>() == 8usize);
    assert!(align_of::<TSIntrinsicKeyword>() == 4usize);
    assert!(offset_of!(TSIntrinsicKeyword, span) == 0usize);

    assert!(size_of::<TSUnknownKeyword>() == 8usize);
    assert!(align_of::<TSUnknownKeyword>() == 4usize);
    assert!(offset_of!(TSUnknownKeyword, span) == 0usize);

    assert!(size_of::<TSNullKeyword>() == 8usize);
    assert!(align_of::<TSNullKeyword>() == 4usize);
    assert!(offset_of!(TSNullKeyword, span) == 0usize);

    assert!(size_of::<TSUndefinedKeyword>() == 8usize);
    assert!(align_of::<TSUndefinedKeyword>() == 4usize);
    assert!(offset_of!(TSUndefinedKeyword, span) == 0usize);

    assert!(size_of::<TSVoidKeyword>() == 8usize);
    assert!(align_of::<TSVoidKeyword>() == 4usize);
    assert!(offset_of!(TSVoidKeyword, span) == 0usize);

    assert!(size_of::<TSSymbolKeyword>() == 8usize);
    assert!(align_of::<TSSymbolKeyword>() == 4usize);
    assert!(offset_of!(TSSymbolKeyword, span) == 0usize);

    assert!(size_of::<TSThisType>() == 8usize);
    assert!(align_of::<TSThisType>() == 4usize);
    assert!(offset_of!(TSThisType, span) == 0usize);

    assert!(size_of::<TSObjectKeyword>() == 8usize);
    assert!(align_of::<TSObjectKeyword>() == 4usize);
    assert!(offset_of!(TSObjectKeyword, span) == 0usize);

    assert!(size_of::<TSBigIntKeyword>() == 8usize);
    assert!(align_of::<TSBigIntKeyword>() == 4usize);
    assert!(offset_of!(TSBigIntKeyword, span) == 0usize);

    assert!(size_of::<TSTypeReference>() == 20usize);
    assert!(align_of::<TSTypeReference>() == 4usize);
    assert!(offset_of!(TSTypeReference, span) == 0usize);
    assert!(offset_of!(TSTypeReference, type_name) == 8usize);
    assert!(offset_of!(TSTypeReference, type_parameters) == 16usize);

    assert!(size_of::<TSTypeName>() == 8usize);
    assert!(align_of::<TSTypeName>() == 4usize);

    assert!(size_of::<TSQualifiedName>() == 32usize);
    assert!(align_of::<TSQualifiedName>() == 4usize);
    assert!(offset_of!(TSQualifiedName, span) == 0usize);
    assert!(offset_of!(TSQualifiedName, left) == 8usize);
    assert!(offset_of!(TSQualifiedName, right) == 16usize);

    assert!(size_of::<TSTypeParameterInstantiation>() == 24usize);
    assert!(align_of::<TSTypeParameterInstantiation>() == 4usize);
    assert!(offset_of!(TSTypeParameterInstantiation, span) == 0usize);
    assert!(offset_of!(TSTypeParameterInstantiation, params) == 8usize);

    assert!(size_of::<TSTypeParameter>() == 48usize);
    assert!(align_of::<TSTypeParameter>() == 4usize);
    assert!(offset_of!(TSTypeParameter, span) == 0usize);
    assert!(offset_of!(TSTypeParameter, name) == 8usize);
    assert!(offset_of!(TSTypeParameter, constraint) == 28usize);
    assert!(offset_of!(TSTypeParameter, default) == 36usize);
    assert!(offset_of!(TSTypeParameter, r#in) == 44usize);
    assert!(offset_of!(TSTypeParameter, out) == 45usize);
    assert!(offset_of!(TSTypeParameter, r#const) == 46usize);

    assert!(size_of::<TSTypeParameterDeclaration>() == 24usize);
    assert!(align_of::<TSTypeParameterDeclaration>() == 4usize);
    assert!(offset_of!(TSTypeParameterDeclaration, span) == 0usize);
    assert!(offset_of!(TSTypeParameterDeclaration, params) == 8usize);

    assert!(size_of::<TSTypeAliasDeclaration>() == 48usize);
    assert!(align_of::<TSTypeAliasDeclaration>() == 4usize);
    assert!(offset_of!(TSTypeAliasDeclaration, span) == 0usize);
    assert!(offset_of!(TSTypeAliasDeclaration, id) == 8usize);
    assert!(offset_of!(TSTypeAliasDeclaration, type_parameters) == 28usize);
    assert!(offset_of!(TSTypeAliasDeclaration, type_annotation) == 32usize);
    assert!(offset_of!(TSTypeAliasDeclaration, declare) == 40usize);
    assert!(offset_of!(TSTypeAliasDeclaration, scope_id) == 44usize);

    assert!(size_of::<TSAccessibility>() == 1usize);
    assert!(align_of::<TSAccessibility>() == 1usize);

    assert!(size_of::<TSClassImplements>() == 20usize);
    assert!(align_of::<TSClassImplements>() == 4usize);
    assert!(offset_of!(TSClassImplements, span) == 0usize);
    assert!(offset_of!(TSClassImplements, expression) == 8usize);
    assert!(offset_of!(TSClassImplements, type_parameters) == 16usize);

    assert!(size_of::<TSInterfaceDeclaration>() == 60usize);
    assert!(align_of::<TSInterfaceDeclaration>() == 4usize);
    assert!(offset_of!(TSInterfaceDeclaration, span) == 0usize);
    assert!(offset_of!(TSInterfaceDeclaration, id) == 8usize);
    assert!(offset_of!(TSInterfaceDeclaration, extends) == 28usize);
    assert!(offset_of!(TSInterfaceDeclaration, type_parameters) == 44usize);
    assert!(offset_of!(TSInterfaceDeclaration, body) == 48usize);
    assert!(offset_of!(TSInterfaceDeclaration, declare) == 52usize);
    assert!(offset_of!(TSInterfaceDeclaration, scope_id) == 56usize);

    assert!(size_of::<TSInterfaceBody>() == 24usize);
    assert!(align_of::<TSInterfaceBody>() == 4usize);
    assert!(offset_of!(TSInterfaceBody, span) == 0usize);
    assert!(offset_of!(TSInterfaceBody, body) == 8usize);

    assert!(size_of::<TSPropertySignature>() == 24usize);
    assert!(align_of::<TSPropertySignature>() == 4usize);
    assert!(offset_of!(TSPropertySignature, span) == 0usize);
    assert!(offset_of!(TSPropertySignature, computed) == 8usize);
    assert!(offset_of!(TSPropertySignature, optional) == 9usize);
    assert!(offset_of!(TSPropertySignature, readonly) == 10usize);
    assert!(offset_of!(TSPropertySignature, key) == 12usize);
    assert!(offset_of!(TSPropertySignature, type_annotation) == 20usize);

    assert!(size_of::<TSSignature>() == 8usize);
    assert!(align_of::<TSSignature>() == 4usize);

    assert!(size_of::<TSIndexSignature>() == 32usize);
    assert!(align_of::<TSIndexSignature>() == 4usize);
    assert!(offset_of!(TSIndexSignature, span) == 0usize);
    assert!(offset_of!(TSIndexSignature, parameters) == 8usize);
    assert!(offset_of!(TSIndexSignature, type_annotation) == 24usize);
    assert!(offset_of!(TSIndexSignature, readonly) == 28usize);

    assert!(size_of::<TSCallSignatureDeclaration>() == 44usize);
    assert!(align_of::<TSCallSignatureDeclaration>() == 4usize);
    assert!(offset_of!(TSCallSignatureDeclaration, span) == 0usize);
    assert!(offset_of!(TSCallSignatureDeclaration, this_param) == 8usize);
    assert!(offset_of!(TSCallSignatureDeclaration, params) == 32usize);
    assert!(offset_of!(TSCallSignatureDeclaration, return_type) == 36usize);
    assert!(offset_of!(TSCallSignatureDeclaration, type_parameters) == 40usize);

    assert!(size_of::<TSMethodSignatureKind>() == 1usize);
    assert!(align_of::<TSMethodSignatureKind>() == 1usize);

    assert!(size_of::<TSMethodSignature>() == 40usize);
    assert!(align_of::<TSMethodSignature>() == 4usize);
    assert!(offset_of!(TSMethodSignature, span) == 0usize);
    assert!(offset_of!(TSMethodSignature, key) == 8usize);
    assert!(offset_of!(TSMethodSignature, computed) == 16usize);
    assert!(offset_of!(TSMethodSignature, optional) == 17usize);
    assert!(offset_of!(TSMethodSignature, kind) == 18usize);
    assert!(offset_of!(TSMethodSignature, this_param) == 20usize);
    assert!(offset_of!(TSMethodSignature, params) == 24usize);
    assert!(offset_of!(TSMethodSignature, return_type) == 28usize);
    assert!(offset_of!(TSMethodSignature, type_parameters) == 32usize);
    assert!(offset_of!(TSMethodSignature, scope_id) == 36usize);

    assert!(size_of::<TSConstructSignatureDeclaration>() == 24usize);
    assert!(align_of::<TSConstructSignatureDeclaration>() == 4usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, span) == 0usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, params) == 8usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, return_type) == 12usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, type_parameters) == 16usize);
    assert!(offset_of!(TSConstructSignatureDeclaration, scope_id) == 20usize);

    assert!(size_of::<TSIndexSignatureName>() == 20usize);
    assert!(align_of::<TSIndexSignatureName>() == 4usize);
    assert!(offset_of!(TSIndexSignatureName, span) == 0usize);
    assert!(offset_of!(TSIndexSignatureName, name) == 8usize);
    assert!(offset_of!(TSIndexSignatureName, type_annotation) == 16usize);

    assert!(size_of::<TSInterfaceHeritage>() == 20usize);
    assert!(align_of::<TSInterfaceHeritage>() == 4usize);
    assert!(offset_of!(TSInterfaceHeritage, span) == 0usize);
    assert!(offset_of!(TSInterfaceHeritage, expression) == 8usize);
    assert!(offset_of!(TSInterfaceHeritage, type_parameters) == 16usize);

    assert!(size_of::<TSTypePredicate>() == 28usize);
    assert!(align_of::<TSTypePredicate>() == 4usize);
    assert!(offset_of!(TSTypePredicate, span) == 0usize);
    assert!(offset_of!(TSTypePredicate, parameter_name) == 8usize);
    assert!(offset_of!(TSTypePredicate, asserts) == 20usize);
    assert!(offset_of!(TSTypePredicate, type_annotation) == 24usize);

    assert!(size_of::<TSTypePredicateName>() == 12usize);
    assert!(align_of::<TSTypePredicateName>() == 4usize);

    assert!(size_of::<TSModuleDeclaration>() == 44usize);
    assert!(align_of::<TSModuleDeclaration>() == 4usize);
    assert!(offset_of!(TSModuleDeclaration, span) == 0usize);
    assert!(offset_of!(TSModuleDeclaration, id) == 8usize);
    assert!(offset_of!(TSModuleDeclaration, body) == 28usize);
    assert!(offset_of!(TSModuleDeclaration, kind) == 36usize);
    assert!(offset_of!(TSModuleDeclaration, declare) == 37usize);
    assert!(offset_of!(TSModuleDeclaration, scope_id) == 40usize);

    assert!(size_of::<TSModuleDeclarationKind>() == 1usize);
    assert!(align_of::<TSModuleDeclarationKind>() == 1usize);

    assert!(size_of::<TSModuleDeclarationName>() == 20usize);
    assert!(align_of::<TSModuleDeclarationName>() == 4usize);

    assert!(size_of::<TSModuleDeclarationBody>() == 8usize);
    assert!(align_of::<TSModuleDeclarationBody>() == 4usize);

    assert!(size_of::<TSModuleBlock>() == 40usize);
    assert!(align_of::<TSModuleBlock>() == 4usize);
    assert!(offset_of!(TSModuleBlock, span) == 0usize);
    assert!(offset_of!(TSModuleBlock, directives) == 8usize);
    assert!(offset_of!(TSModuleBlock, body) == 24usize);

    assert!(size_of::<TSTypeLiteral>() == 24usize);
    assert!(align_of::<TSTypeLiteral>() == 4usize);
    assert!(offset_of!(TSTypeLiteral, span) == 0usize);
    assert!(offset_of!(TSTypeLiteral, members) == 8usize);

    assert!(size_of::<TSInferType>() == 12usize);
    assert!(align_of::<TSInferType>() == 4usize);
    assert!(offset_of!(TSInferType, span) == 0usize);
    assert!(offset_of!(TSInferType, type_parameter) == 8usize);

    assert!(size_of::<TSTypeQuery>() == 20usize);
    assert!(align_of::<TSTypeQuery>() == 4usize);
    assert!(offset_of!(TSTypeQuery, span) == 0usize);
    assert!(offset_of!(TSTypeQuery, expr_name) == 8usize);
    assert!(offset_of!(TSTypeQuery, type_parameters) == 16usize);

    assert!(size_of::<TSTypeQueryExprName>() == 8usize);
    assert!(align_of::<TSTypeQueryExprName>() == 4usize);

    assert!(size_of::<TSImportType>() == 36usize);
    assert!(align_of::<TSImportType>() == 4usize);
    assert!(offset_of!(TSImportType, span) == 0usize);
    assert!(offset_of!(TSImportType, is_type_of) == 8usize);
    assert!(offset_of!(TSImportType, parameter) == 12usize);
    assert!(offset_of!(TSImportType, qualifier) == 20usize);
    assert!(offset_of!(TSImportType, attributes) == 28usize);
    assert!(offset_of!(TSImportType, type_parameters) == 32usize);

    assert!(size_of::<TSImportAttributes>() == 40usize);
    assert!(align_of::<TSImportAttributes>() == 4usize);
    assert!(offset_of!(TSImportAttributes, span) == 0usize);
    assert!(offset_of!(TSImportAttributes, attributes_keyword) == 8usize);
    assert!(offset_of!(TSImportAttributes, elements) == 24usize);

    assert!(size_of::<TSImportAttribute>() == 36usize);
    assert!(align_of::<TSImportAttribute>() == 4usize);
    assert!(offset_of!(TSImportAttribute, span) == 0usize);
    assert!(offset_of!(TSImportAttribute, name) == 8usize);
    assert!(offset_of!(TSImportAttribute, value) == 28usize);

    assert!(size_of::<TSImportAttributeName>() == 20usize);
    assert!(align_of::<TSImportAttributeName>() == 4usize);

    assert!(size_of::<TSFunctionType>() == 24usize);
    assert!(align_of::<TSFunctionType>() == 4usize);
    assert!(offset_of!(TSFunctionType, span) == 0usize);
    assert!(offset_of!(TSFunctionType, this_param) == 8usize);
    assert!(offset_of!(TSFunctionType, params) == 12usize);
    assert!(offset_of!(TSFunctionType, return_type) == 16usize);
    assert!(offset_of!(TSFunctionType, type_parameters) == 20usize);

    assert!(size_of::<TSConstructorType>() == 24usize);
    assert!(align_of::<TSConstructorType>() == 4usize);
    assert!(offset_of!(TSConstructorType, span) == 0usize);
    assert!(offset_of!(TSConstructorType, r#abstract) == 8usize);
    assert!(offset_of!(TSConstructorType, params) == 12usize);
    assert!(offset_of!(TSConstructorType, return_type) == 16usize);
    assert!(offset_of!(TSConstructorType, type_parameters) == 20usize);

    assert!(size_of::<TSMappedType>() == 36usize);
    assert!(align_of::<TSMappedType>() == 4usize);
    assert!(offset_of!(TSMappedType, span) == 0usize);
    assert!(offset_of!(TSMappedType, type_parameter) == 8usize);
    assert!(offset_of!(TSMappedType, name_type) == 12usize);
    assert!(offset_of!(TSMappedType, type_annotation) == 20usize);
    assert!(offset_of!(TSMappedType, optional) == 28usize);
    assert!(offset_of!(TSMappedType, readonly) == 29usize);
    assert!(offset_of!(TSMappedType, scope_id) == 32usize);

    assert!(size_of::<TSMappedTypeModifierOperator>() == 1usize);
    assert!(align_of::<TSMappedTypeModifierOperator>() == 1usize);

    assert!(size_of::<TSTemplateLiteralType>() == 40usize);
    assert!(align_of::<TSTemplateLiteralType>() == 4usize);
    assert!(offset_of!(TSTemplateLiteralType, span) == 0usize);
    assert!(offset_of!(TSTemplateLiteralType, quasis) == 8usize);
    assert!(offset_of!(TSTemplateLiteralType, types) == 24usize);

    assert!(size_of::<TSAsExpression>() == 24usize);
    assert!(align_of::<TSAsExpression>() == 4usize);
    assert!(offset_of!(TSAsExpression, span) == 0usize);
    assert!(offset_of!(TSAsExpression, expression) == 8usize);
    assert!(offset_of!(TSAsExpression, type_annotation) == 16usize);

    assert!(size_of::<TSSatisfiesExpression>() == 24usize);
    assert!(align_of::<TSSatisfiesExpression>() == 4usize);
    assert!(offset_of!(TSSatisfiesExpression, span) == 0usize);
    assert!(offset_of!(TSSatisfiesExpression, expression) == 8usize);
    assert!(offset_of!(TSSatisfiesExpression, type_annotation) == 16usize);

    assert!(size_of::<TSTypeAssertion>() == 24usize);
    assert!(align_of::<TSTypeAssertion>() == 4usize);
    assert!(offset_of!(TSTypeAssertion, span) == 0usize);
    assert!(offset_of!(TSTypeAssertion, expression) == 8usize);
    assert!(offset_of!(TSTypeAssertion, type_annotation) == 16usize);

    assert!(size_of::<TSImportEqualsDeclaration>() == 40usize);
    assert!(align_of::<TSImportEqualsDeclaration>() == 4usize);
    assert!(offset_of!(TSImportEqualsDeclaration, span) == 0usize);
    assert!(offset_of!(TSImportEqualsDeclaration, id) == 8usize);
    assert!(offset_of!(TSImportEqualsDeclaration, module_reference) == 28usize);
    assert!(offset_of!(TSImportEqualsDeclaration, import_kind) == 36usize);

    assert!(size_of::<TSModuleReference>() == 8usize);
    assert!(align_of::<TSModuleReference>() == 4usize);

    assert!(size_of::<TSExternalModuleReference>() == 24usize);
    assert!(align_of::<TSExternalModuleReference>() == 4usize);
    assert!(offset_of!(TSExternalModuleReference, span) == 0usize);
    assert!(offset_of!(TSExternalModuleReference, expression) == 8usize);

    assert!(size_of::<TSNonNullExpression>() == 16usize);
    assert!(align_of::<TSNonNullExpression>() == 4usize);
    assert!(offset_of!(TSNonNullExpression, span) == 0usize);
    assert!(offset_of!(TSNonNullExpression, expression) == 8usize);

    assert!(size_of::<Decorator>() == 16usize);
    assert!(align_of::<Decorator>() == 4usize);
    assert!(offset_of!(Decorator, span) == 0usize);
    assert!(offset_of!(Decorator, expression) == 8usize);

    assert!(size_of::<TSExportAssignment>() == 16usize);
    assert!(align_of::<TSExportAssignment>() == 4usize);
    assert!(offset_of!(TSExportAssignment, span) == 0usize);
    assert!(offset_of!(TSExportAssignment, expression) == 8usize);

    assert!(size_of::<TSNamespaceExportDeclaration>() == 24usize);
    assert!(align_of::<TSNamespaceExportDeclaration>() == 4usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, span) == 0usize);
    assert!(offset_of!(TSNamespaceExportDeclaration, id) == 8usize);

    assert!(size_of::<TSInstantiationExpression>() == 20usize);
    assert!(align_of::<TSInstantiationExpression>() == 4usize);
    assert!(offset_of!(TSInstantiationExpression, span) == 0usize);
    assert!(offset_of!(TSInstantiationExpression, expression) == 8usize);
    assert!(offset_of!(TSInstantiationExpression, type_parameters) == 16usize);

    assert!(size_of::<ImportOrExportKind>() == 1usize);
    assert!(align_of::<ImportOrExportKind>() == 1usize);

    assert!(size_of::<JSDocNullableType>() == 20usize);
    assert!(align_of::<JSDocNullableType>() == 4usize);
    assert!(offset_of!(JSDocNullableType, span) == 0usize);
    assert!(offset_of!(JSDocNullableType, type_annotation) == 8usize);
    assert!(offset_of!(JSDocNullableType, postfix) == 16usize);

    assert!(size_of::<JSDocNonNullableType>() == 20usize);
    assert!(align_of::<JSDocNonNullableType>() == 4usize);
    assert!(offset_of!(JSDocNonNullableType, span) == 0usize);
    assert!(offset_of!(JSDocNonNullableType, type_annotation) == 8usize);
    assert!(offset_of!(JSDocNonNullableType, postfix) == 16usize);

    assert!(size_of::<JSDocUnknownType>() == 8usize);
    assert!(align_of::<JSDocUnknownType>() == 4usize);
    assert!(offset_of!(JSDocUnknownType, span) == 0usize);

    assert!(size_of::<JSXElement>() == 32usize);
    assert!(align_of::<JSXElement>() == 4usize);
    assert!(offset_of!(JSXElement, span) == 0usize);
    assert!(offset_of!(JSXElement, opening_element) == 8usize);
    assert!(offset_of!(JSXElement, closing_element) == 12usize);
    assert!(offset_of!(JSXElement, children) == 16usize);

    assert!(size_of::<JSXOpeningElement>() == 40usize);
    assert!(align_of::<JSXOpeningElement>() == 4usize);
    assert!(offset_of!(JSXOpeningElement, span) == 0usize);
    assert!(offset_of!(JSXOpeningElement, self_closing) == 8usize);
    assert!(offset_of!(JSXOpeningElement, name) == 12usize);
    assert!(offset_of!(JSXOpeningElement, attributes) == 20usize);
    assert!(offset_of!(JSXOpeningElement, type_parameters) == 36usize);

    assert!(size_of::<JSXClosingElement>() == 16usize);
    assert!(align_of::<JSXClosingElement>() == 4usize);
    assert!(offset_of!(JSXClosingElement, span) == 0usize);
    assert!(offset_of!(JSXClosingElement, name) == 8usize);

    assert!(size_of::<JSXFragment>() == 40usize);
    assert!(align_of::<JSXFragment>() == 4usize);
    assert!(offset_of!(JSXFragment, span) == 0usize);
    assert!(offset_of!(JSXFragment, opening_fragment) == 8usize);
    assert!(offset_of!(JSXFragment, closing_fragment) == 16usize);
    assert!(offset_of!(JSXFragment, children) == 24usize);

    assert!(size_of::<JSXOpeningFragment>() == 8usize);
    assert!(align_of::<JSXOpeningFragment>() == 4usize);
    assert!(offset_of!(JSXOpeningFragment, span) == 0usize);

    assert!(size_of::<JSXClosingFragment>() == 8usize);
    assert!(align_of::<JSXClosingFragment>() == 4usize);
    assert!(offset_of!(JSXClosingFragment, span) == 0usize);

    assert!(size_of::<JSXElementName>() == 8usize);
    assert!(align_of::<JSXElementName>() == 4usize);

    assert!(size_of::<JSXNamespacedName>() == 40usize);
    assert!(align_of::<JSXNamespacedName>() == 4usize);
    assert!(offset_of!(JSXNamespacedName, span) == 0usize);
    assert!(offset_of!(JSXNamespacedName, namespace) == 8usize);
    assert!(offset_of!(JSXNamespacedName, property) == 24usize);

    assert!(size_of::<JSXMemberExpression>() == 32usize);
    assert!(align_of::<JSXMemberExpression>() == 4usize);
    assert!(offset_of!(JSXMemberExpression, span) == 0usize);
    assert!(offset_of!(JSXMemberExpression, object) == 8usize);
    assert!(offset_of!(JSXMemberExpression, property) == 16usize);

    assert!(size_of::<JSXMemberExpressionObject>() == 8usize);
    assert!(align_of::<JSXMemberExpressionObject>() == 4usize);

    assert!(size_of::<JSXExpressionContainer>() == 20usize);
    assert!(align_of::<JSXExpressionContainer>() == 4usize);
    assert!(offset_of!(JSXExpressionContainer, span) == 0usize);
    assert!(offset_of!(JSXExpressionContainer, expression) == 8usize);

    assert!(size_of::<JSXExpression>() == 12usize);
    assert!(align_of::<JSXExpression>() == 4usize);

    assert!(size_of::<JSXEmptyExpression>() == 8usize);
    assert!(align_of::<JSXEmptyExpression>() == 4usize);
    assert!(offset_of!(JSXEmptyExpression, span) == 0usize);

    assert!(size_of::<JSXAttributeItem>() == 8usize);
    assert!(align_of::<JSXAttributeItem>() == 4usize);

    assert!(size_of::<JSXAttribute>() == 24usize);
    assert!(align_of::<JSXAttribute>() == 4usize);
    assert!(offset_of!(JSXAttribute, span) == 0usize);
    assert!(offset_of!(JSXAttribute, name) == 8usize);
    assert!(offset_of!(JSXAttribute, value) == 16usize);

    assert!(size_of::<JSXSpreadAttribute>() == 16usize);
    assert!(align_of::<JSXSpreadAttribute>() == 4usize);
    assert!(offset_of!(JSXSpreadAttribute, span) == 0usize);
    assert!(offset_of!(JSXSpreadAttribute, argument) == 8usize);

    assert!(size_of::<JSXAttributeName>() == 8usize);
    assert!(align_of::<JSXAttributeName>() == 4usize);

    assert!(size_of::<JSXAttributeValue>() == 8usize);
    assert!(align_of::<JSXAttributeValue>() == 4usize);

    assert!(size_of::<JSXIdentifier>() == 16usize);
    assert!(align_of::<JSXIdentifier>() == 4usize);
    assert!(offset_of!(JSXIdentifier, span) == 0usize);
    assert!(offset_of!(JSXIdentifier, name) == 8usize);

    assert!(size_of::<JSXChild>() == 8usize);
    assert!(align_of::<JSXChild>() == 4usize);

    assert!(size_of::<JSXSpreadChild>() == 16usize);
    assert!(align_of::<JSXSpreadChild>() == 4usize);
    assert!(offset_of!(JSXSpreadChild, span) == 0usize);
    assert!(offset_of!(JSXSpreadChild, expression) == 8usize);

    assert!(size_of::<JSXText>() == 16usize);
    assert!(align_of::<JSXText>() == 4usize);
    assert!(offset_of!(JSXText, span) == 0usize);
    assert!(offset_of!(JSXText, value) == 8usize);

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

    assert!(size_of::<Term>() == 12usize);
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

    assert!(size_of::<Quantifier>() == 48usize);
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

    assert!(size_of::<CharacterClassContents>() == 8usize);
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
