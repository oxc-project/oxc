// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::ast::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 1 bytes
    assert!(size_of::<Program>() == 160);
    assert!(align_of::<Program>() == 8);
    assert!(offset_of!(Program, span) == 0);
    assert!(offset_of!(Program, source_type) == 156);
    assert!(offset_of!(Program, source_text) == 24);
    assert!(offset_of!(Program, comments) == 40);
    assert!(offset_of!(Program, hashbang) == 64);
    assert!(offset_of!(Program, directives) == 104);
    assert!(offset_of!(Program, body) == 128);
    assert!(offset_of!(Program, scope_id) == 152);

    assert!(size_of::<Expression>() == 16);
    assert!(align_of::<Expression>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<IdentifierName>() == 40);
    assert!(align_of::<IdentifierName>() == 8);
    assert!(offset_of!(IdentifierName, span) == 0);
    assert!(offset_of!(IdentifierName, name) == 24);

    // Padding: 4 bytes
    assert!(size_of::<IdentifierReference>() == 48);
    assert!(align_of::<IdentifierReference>() == 8);
    assert!(offset_of!(IdentifierReference, span) == 0);
    assert!(offset_of!(IdentifierReference, name) == 24);
    assert!(offset_of!(IdentifierReference, reference_id) == 40);

    // Padding: 4 bytes
    assert!(size_of::<BindingIdentifier>() == 48);
    assert!(align_of::<BindingIdentifier>() == 8);
    assert!(offset_of!(BindingIdentifier, span) == 0);
    assert!(offset_of!(BindingIdentifier, name) == 24);
    assert!(offset_of!(BindingIdentifier, symbol_id) == 40);

    // Padding: 0 bytes
    assert!(size_of::<LabelIdentifier>() == 40);
    assert!(align_of::<LabelIdentifier>() == 8);
    assert!(offset_of!(LabelIdentifier, span) == 0);
    assert!(offset_of!(LabelIdentifier, name) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ThisExpression>() == 24);
    assert!(align_of::<ThisExpression>() == 8);
    assert!(offset_of!(ThisExpression, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ArrayExpression>() == 48);
    assert!(align_of::<ArrayExpression>() == 8);
    assert!(offset_of!(ArrayExpression, span) == 0);
    assert!(offset_of!(ArrayExpression, elements) == 24);

    assert!(size_of::<ArrayExpressionElement>() == 32);
    assert!(align_of::<ArrayExpressionElement>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<Elision>() == 24);
    assert!(align_of::<Elision>() == 8);
    assert!(offset_of!(Elision, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ObjectExpression>() == 48);
    assert!(align_of::<ObjectExpression>() == 8);
    assert!(offset_of!(ObjectExpression, span) == 0);
    assert!(offset_of!(ObjectExpression, properties) == 24);

    assert!(size_of::<ObjectPropertyKind>() == 16);
    assert!(align_of::<ObjectPropertyKind>() == 8);

    // Padding: 4 bytes
    assert!(size_of::<ObjectProperty>() == 64);
    assert!(align_of::<ObjectProperty>() == 8);
    assert!(offset_of!(ObjectProperty, span) == 0);
    assert!(offset_of!(ObjectProperty, kind) == 56);
    assert!(offset_of!(ObjectProperty, key) == 24);
    assert!(offset_of!(ObjectProperty, value) == 40);
    assert!(offset_of!(ObjectProperty, method) == 57);
    assert!(offset_of!(ObjectProperty, shorthand) == 58);
    assert!(offset_of!(ObjectProperty, computed) == 59);

    assert!(size_of::<PropertyKey>() == 16);
    assert!(align_of::<PropertyKey>() == 8);

    assert!(size_of::<PropertyKind>() == 1);
    assert!(align_of::<PropertyKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TemplateLiteral>() == 72);
    assert!(align_of::<TemplateLiteral>() == 8);
    assert!(offset_of!(TemplateLiteral, span) == 0);
    assert!(offset_of!(TemplateLiteral, quasis) == 24);
    assert!(offset_of!(TemplateLiteral, expressions) == 48);

    // Padding: 0 bytes
    assert!(size_of::<TaggedTemplateExpression>() == 120);
    assert!(align_of::<TaggedTemplateExpression>() == 8);
    assert!(offset_of!(TaggedTemplateExpression, span) == 0);
    assert!(offset_of!(TaggedTemplateExpression, tag) == 24);
    assert!(offset_of!(TaggedTemplateExpression, type_arguments) == 40);
    assert!(offset_of!(TaggedTemplateExpression, quasi) == 48);

    // Padding: 6 bytes
    assert!(size_of::<TemplateElement>() == 64);
    assert!(align_of::<TemplateElement>() == 8);
    assert!(offset_of!(TemplateElement, span) == 0);
    assert!(offset_of!(TemplateElement, value) == 24);
    assert!(offset_of!(TemplateElement, tail) == 56);
    assert!(offset_of!(TemplateElement, lone_surrogates) == 57);

    // Padding: 0 bytes
    assert!(size_of::<TemplateElementValue>() == 32);
    assert!(align_of::<TemplateElementValue>() == 8);
    assert!(offset_of!(TemplateElementValue, raw) == 0);
    assert!(offset_of!(TemplateElementValue, cooked) == 16);

    assert!(size_of::<MemberExpression>() == 16);
    assert!(align_of::<MemberExpression>() == 8);

    // Padding: 7 bytes
    assert!(size_of::<ComputedMemberExpression>() == 64);
    assert!(align_of::<ComputedMemberExpression>() == 8);
    assert!(offset_of!(ComputedMemberExpression, span) == 0);
    assert!(offset_of!(ComputedMemberExpression, object) == 24);
    assert!(offset_of!(ComputedMemberExpression, expression) == 40);
    assert!(offset_of!(ComputedMemberExpression, optional) == 56);

    // Padding: 7 bytes
    assert!(size_of::<StaticMemberExpression>() == 88);
    assert!(align_of::<StaticMemberExpression>() == 8);
    assert!(offset_of!(StaticMemberExpression, span) == 0);
    assert!(offset_of!(StaticMemberExpression, object) == 24);
    assert!(offset_of!(StaticMemberExpression, property) == 40);
    assert!(offset_of!(StaticMemberExpression, optional) == 80);

    // Padding: 7 bytes
    assert!(size_of::<PrivateFieldExpression>() == 88);
    assert!(align_of::<PrivateFieldExpression>() == 8);
    assert!(offset_of!(PrivateFieldExpression, span) == 0);
    assert!(offset_of!(PrivateFieldExpression, object) == 24);
    assert!(offset_of!(PrivateFieldExpression, field) == 40);
    assert!(offset_of!(PrivateFieldExpression, optional) == 80);

    // Padding: 6 bytes
    assert!(size_of::<CallExpression>() == 80);
    assert!(align_of::<CallExpression>() == 8);
    assert!(offset_of!(CallExpression, span) == 0);
    assert!(offset_of!(CallExpression, callee) == 24);
    assert!(offset_of!(CallExpression, type_arguments) == 40);
    assert!(offset_of!(CallExpression, arguments) == 48);
    assert!(offset_of!(CallExpression, optional) == 72);
    assert!(offset_of!(CallExpression, pure) == 73);

    // Padding: 7 bytes
    assert!(size_of::<NewExpression>() == 80);
    assert!(align_of::<NewExpression>() == 8);
    assert!(offset_of!(NewExpression, span) == 0);
    assert!(offset_of!(NewExpression, callee) == 24);
    assert!(offset_of!(NewExpression, type_arguments) == 40);
    assert!(offset_of!(NewExpression, arguments) == 48);
    assert!(offset_of!(NewExpression, pure) == 72);

    // Padding: 0 bytes
    assert!(size_of::<MetaProperty>() == 104);
    assert!(align_of::<MetaProperty>() == 8);
    assert!(offset_of!(MetaProperty, span) == 0);
    assert!(offset_of!(MetaProperty, meta) == 24);
    assert!(offset_of!(MetaProperty, property) == 64);

    // Padding: 0 bytes
    assert!(size_of::<SpreadElement>() == 40);
    assert!(align_of::<SpreadElement>() == 8);
    assert!(offset_of!(SpreadElement, span) == 0);
    assert!(offset_of!(SpreadElement, argument) == 24);

    assert!(size_of::<Argument>() == 16);
    assert!(align_of::<Argument>() == 8);

    // Padding: 6 bytes
    assert!(size_of::<UpdateExpression>() == 48);
    assert!(align_of::<UpdateExpression>() == 8);
    assert!(offset_of!(UpdateExpression, span) == 0);
    assert!(offset_of!(UpdateExpression, operator) == 40);
    assert!(offset_of!(UpdateExpression, prefix) == 41);
    assert!(offset_of!(UpdateExpression, argument) == 24);

    // Padding: 7 bytes
    assert!(size_of::<UnaryExpression>() == 48);
    assert!(align_of::<UnaryExpression>() == 8);
    assert!(offset_of!(UnaryExpression, span) == 0);
    assert!(offset_of!(UnaryExpression, operator) == 40);
    assert!(offset_of!(UnaryExpression, argument) == 24);

    // Padding: 7 bytes
    assert!(size_of::<BinaryExpression>() == 64);
    assert!(align_of::<BinaryExpression>() == 8);
    assert!(offset_of!(BinaryExpression, span) == 0);
    assert!(offset_of!(BinaryExpression, left) == 24);
    assert!(offset_of!(BinaryExpression, operator) == 56);
    assert!(offset_of!(BinaryExpression, right) == 40);

    // Padding: 0 bytes
    assert!(size_of::<PrivateInExpression>() == 80);
    assert!(align_of::<PrivateInExpression>() == 8);
    assert!(offset_of!(PrivateInExpression, span) == 0);
    assert!(offset_of!(PrivateInExpression, left) == 24);
    assert!(offset_of!(PrivateInExpression, right) == 64);

    // Padding: 7 bytes
    assert!(size_of::<LogicalExpression>() == 64);
    assert!(align_of::<LogicalExpression>() == 8);
    assert!(offset_of!(LogicalExpression, span) == 0);
    assert!(offset_of!(LogicalExpression, left) == 24);
    assert!(offset_of!(LogicalExpression, operator) == 56);
    assert!(offset_of!(LogicalExpression, right) == 40);

    // Padding: 0 bytes
    assert!(size_of::<ConditionalExpression>() == 72);
    assert!(align_of::<ConditionalExpression>() == 8);
    assert!(offset_of!(ConditionalExpression, span) == 0);
    assert!(offset_of!(ConditionalExpression, test) == 24);
    assert!(offset_of!(ConditionalExpression, consequent) == 40);
    assert!(offset_of!(ConditionalExpression, alternate) == 56);

    // Padding: 7 bytes
    assert!(size_of::<AssignmentExpression>() == 64);
    assert!(align_of::<AssignmentExpression>() == 8);
    assert!(offset_of!(AssignmentExpression, span) == 0);
    assert!(offset_of!(AssignmentExpression, operator) == 56);
    assert!(offset_of!(AssignmentExpression, left) == 24);
    assert!(offset_of!(AssignmentExpression, right) == 40);

    assert!(size_of::<AssignmentTarget>() == 16);
    assert!(align_of::<AssignmentTarget>() == 8);

    assert!(size_of::<SimpleAssignmentTarget>() == 16);
    assert!(align_of::<SimpleAssignmentTarget>() == 8);

    assert!(size_of::<AssignmentTargetPattern>() == 16);
    assert!(align_of::<AssignmentTargetPattern>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<ArrayAssignmentTarget>() == 88);
    assert!(align_of::<ArrayAssignmentTarget>() == 8);
    assert!(offset_of!(ArrayAssignmentTarget, span) == 0);
    assert!(offset_of!(ArrayAssignmentTarget, elements) == 24);
    assert!(offset_of!(ArrayAssignmentTarget, rest) == 48);

    // Padding: 0 bytes
    assert!(size_of::<ObjectAssignmentTarget>() == 88);
    assert!(align_of::<ObjectAssignmentTarget>() == 8);
    assert!(offset_of!(ObjectAssignmentTarget, span) == 0);
    assert!(offset_of!(ObjectAssignmentTarget, properties) == 24);
    assert!(offset_of!(ObjectAssignmentTarget, rest) == 48);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetRest>() == 40);
    assert!(align_of::<AssignmentTargetRest>() == 8);
    assert!(offset_of!(AssignmentTargetRest, span) == 0);
    assert!(offset_of!(AssignmentTargetRest, target) == 24);

    assert!(size_of::<AssignmentTargetMaybeDefault>() == 16);
    assert!(align_of::<AssignmentTargetMaybeDefault>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetWithDefault>() == 56);
    assert!(align_of::<AssignmentTargetWithDefault>() == 8);
    assert!(offset_of!(AssignmentTargetWithDefault, span) == 0);
    assert!(offset_of!(AssignmentTargetWithDefault, binding) == 24);
    assert!(offset_of!(AssignmentTargetWithDefault, init) == 40);

    assert!(size_of::<AssignmentTargetProperty>() == 16);
    assert!(align_of::<AssignmentTargetProperty>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetPropertyIdentifier>() == 88);
    assert!(align_of::<AssignmentTargetPropertyIdentifier>() == 8);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, span) == 0);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, binding) == 24);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, init) == 72);

    // Padding: 7 bytes
    assert!(size_of::<AssignmentTargetPropertyProperty>() == 64);
    assert!(align_of::<AssignmentTargetPropertyProperty>() == 8);
    assert!(offset_of!(AssignmentTargetPropertyProperty, span) == 0);
    assert!(offset_of!(AssignmentTargetPropertyProperty, name) == 24);
    assert!(offset_of!(AssignmentTargetPropertyProperty, binding) == 40);
    assert!(offset_of!(AssignmentTargetPropertyProperty, computed) == 56);

    // Padding: 0 bytes
    assert!(size_of::<SequenceExpression>() == 48);
    assert!(align_of::<SequenceExpression>() == 8);
    assert!(offset_of!(SequenceExpression, span) == 0);
    assert!(offset_of!(SequenceExpression, expressions) == 24);

    // Padding: 0 bytes
    assert!(size_of::<Super>() == 24);
    assert!(align_of::<Super>() == 8);
    assert!(offset_of!(Super, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<AwaitExpression>() == 40);
    assert!(align_of::<AwaitExpression>() == 8);
    assert!(offset_of!(AwaitExpression, span) == 0);
    assert!(offset_of!(AwaitExpression, argument) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ChainExpression>() == 40);
    assert!(align_of::<ChainExpression>() == 8);
    assert!(offset_of!(ChainExpression, span) == 0);
    assert!(offset_of!(ChainExpression, expression) == 24);

    assert!(size_of::<ChainElement>() == 16);
    assert!(align_of::<ChainElement>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<ParenthesizedExpression>() == 40);
    assert!(align_of::<ParenthesizedExpression>() == 8);
    assert!(offset_of!(ParenthesizedExpression, span) == 0);
    assert!(offset_of!(ParenthesizedExpression, expression) == 24);

    assert!(size_of::<Statement>() == 16);
    assert!(align_of::<Statement>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<Directive>() == 104);
    assert!(align_of::<Directive>() == 8);
    assert!(offset_of!(Directive, span) == 0);
    assert!(offset_of!(Directive, expression) == 24);
    assert!(offset_of!(Directive, directive) == 88);

    // Padding: 0 bytes
    assert!(size_of::<Hashbang>() == 40);
    assert!(align_of::<Hashbang>() == 8);
    assert!(offset_of!(Hashbang, span) == 0);
    assert!(offset_of!(Hashbang, value) == 24);

    // Padding: 4 bytes
    assert!(size_of::<BlockStatement>() == 56);
    assert!(align_of::<BlockStatement>() == 8);
    assert!(offset_of!(BlockStatement, span) == 0);
    assert!(offset_of!(BlockStatement, body) == 24);
    assert!(offset_of!(BlockStatement, scope_id) == 48);

    assert!(size_of::<Declaration>() == 16);
    assert!(align_of::<Declaration>() == 8);

    // Padding: 6 bytes
    assert!(size_of::<VariableDeclaration>() == 56);
    assert!(align_of::<VariableDeclaration>() == 8);
    assert!(offset_of!(VariableDeclaration, span) == 0);
    assert!(offset_of!(VariableDeclaration, kind) == 48);
    assert!(offset_of!(VariableDeclaration, declarations) == 24);
    assert!(offset_of!(VariableDeclaration, declare) == 49);

    assert!(size_of::<VariableDeclarationKind>() == 1);
    assert!(align_of::<VariableDeclarationKind>() == 1);

    // Padding: 6 bytes
    assert!(size_of::<VariableDeclarator>() == 80);
    assert!(align_of::<VariableDeclarator>() == 8);
    assert!(offset_of!(VariableDeclarator, span) == 0);
    assert!(offset_of!(VariableDeclarator, kind) == 72);
    assert!(offset_of!(VariableDeclarator, id) == 24);
    assert!(offset_of!(VariableDeclarator, init) == 56);
    assert!(offset_of!(VariableDeclarator, definite) == 73);

    // Padding: 0 bytes
    assert!(size_of::<EmptyStatement>() == 24);
    assert!(align_of::<EmptyStatement>() == 8);
    assert!(offset_of!(EmptyStatement, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ExpressionStatement>() == 40);
    assert!(align_of::<ExpressionStatement>() == 8);
    assert!(offset_of!(ExpressionStatement, span) == 0);
    assert!(offset_of!(ExpressionStatement, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<IfStatement>() == 72);
    assert!(align_of::<IfStatement>() == 8);
    assert!(offset_of!(IfStatement, span) == 0);
    assert!(offset_of!(IfStatement, test) == 24);
    assert!(offset_of!(IfStatement, consequent) == 40);
    assert!(offset_of!(IfStatement, alternate) == 56);

    // Padding: 0 bytes
    assert!(size_of::<DoWhileStatement>() == 56);
    assert!(align_of::<DoWhileStatement>() == 8);
    assert!(offset_of!(DoWhileStatement, span) == 0);
    assert!(offset_of!(DoWhileStatement, body) == 24);
    assert!(offset_of!(DoWhileStatement, test) == 40);

    // Padding: 0 bytes
    assert!(size_of::<WhileStatement>() == 56);
    assert!(align_of::<WhileStatement>() == 8);
    assert!(offset_of!(WhileStatement, span) == 0);
    assert!(offset_of!(WhileStatement, test) == 24);
    assert!(offset_of!(WhileStatement, body) == 40);

    // Padding: 4 bytes
    assert!(size_of::<ForStatement>() == 96);
    assert!(align_of::<ForStatement>() == 8);
    assert!(offset_of!(ForStatement, span) == 0);
    assert!(offset_of!(ForStatement, init) == 24);
    assert!(offset_of!(ForStatement, test) == 40);
    assert!(offset_of!(ForStatement, update) == 56);
    assert!(offset_of!(ForStatement, body) == 72);
    assert!(offset_of!(ForStatement, scope_id) == 88);

    assert!(size_of::<ForStatementInit>() == 16);
    assert!(align_of::<ForStatementInit>() == 8);

    // Padding: 4 bytes
    assert!(size_of::<ForInStatement>() == 80);
    assert!(align_of::<ForInStatement>() == 8);
    assert!(offset_of!(ForInStatement, span) == 0);
    assert!(offset_of!(ForInStatement, left) == 24);
    assert!(offset_of!(ForInStatement, right) == 40);
    assert!(offset_of!(ForInStatement, body) == 56);
    assert!(offset_of!(ForInStatement, scope_id) == 72);

    assert!(size_of::<ForStatementLeft>() == 16);
    assert!(align_of::<ForStatementLeft>() == 8);

    // Padding: 3 bytes
    assert!(size_of::<ForOfStatement>() == 80);
    assert!(align_of::<ForOfStatement>() == 8);
    assert!(offset_of!(ForOfStatement, span) == 0);
    assert!(offset_of!(ForOfStatement, r#await) == 76);
    assert!(offset_of!(ForOfStatement, left) == 24);
    assert!(offset_of!(ForOfStatement, right) == 40);
    assert!(offset_of!(ForOfStatement, body) == 56);
    assert!(offset_of!(ForOfStatement, scope_id) == 72);

    // Padding: 0 bytes
    assert!(size_of::<ContinueStatement>() == 64);
    assert!(align_of::<ContinueStatement>() == 8);
    assert!(offset_of!(ContinueStatement, span) == 0);
    assert!(offset_of!(ContinueStatement, label) == 24);

    // Padding: 0 bytes
    assert!(size_of::<BreakStatement>() == 64);
    assert!(align_of::<BreakStatement>() == 8);
    assert!(offset_of!(BreakStatement, span) == 0);
    assert!(offset_of!(BreakStatement, label) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ReturnStatement>() == 40);
    assert!(align_of::<ReturnStatement>() == 8);
    assert!(offset_of!(ReturnStatement, span) == 0);
    assert!(offset_of!(ReturnStatement, argument) == 24);

    // Padding: 0 bytes
    assert!(size_of::<WithStatement>() == 56);
    assert!(align_of::<WithStatement>() == 8);
    assert!(offset_of!(WithStatement, span) == 0);
    assert!(offset_of!(WithStatement, object) == 24);
    assert!(offset_of!(WithStatement, body) == 40);

    // Padding: 4 bytes
    assert!(size_of::<SwitchStatement>() == 72);
    assert!(align_of::<SwitchStatement>() == 8);
    assert!(offset_of!(SwitchStatement, span) == 0);
    assert!(offset_of!(SwitchStatement, discriminant) == 24);
    assert!(offset_of!(SwitchStatement, cases) == 40);
    assert!(offset_of!(SwitchStatement, scope_id) == 64);

    // Padding: 0 bytes
    assert!(size_of::<SwitchCase>() == 64);
    assert!(align_of::<SwitchCase>() == 8);
    assert!(offset_of!(SwitchCase, span) == 0);
    assert!(offset_of!(SwitchCase, test) == 24);
    assert!(offset_of!(SwitchCase, consequent) == 40);

    // Padding: 0 bytes
    assert!(size_of::<LabeledStatement>() == 80);
    assert!(align_of::<LabeledStatement>() == 8);
    assert!(offset_of!(LabeledStatement, span) == 0);
    assert!(offset_of!(LabeledStatement, label) == 24);
    assert!(offset_of!(LabeledStatement, body) == 64);

    // Padding: 0 bytes
    assert!(size_of::<ThrowStatement>() == 40);
    assert!(align_of::<ThrowStatement>() == 8);
    assert!(offset_of!(ThrowStatement, span) == 0);
    assert!(offset_of!(ThrowStatement, argument) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TryStatement>() == 48);
    assert!(align_of::<TryStatement>() == 8);
    assert!(offset_of!(TryStatement, span) == 0);
    assert!(offset_of!(TryStatement, block) == 24);
    assert!(offset_of!(TryStatement, handler) == 32);
    assert!(offset_of!(TryStatement, finalizer) == 40);

    // Padding: 4 bytes
    assert!(size_of::<CatchClause>() == 96);
    assert!(align_of::<CatchClause>() == 8);
    assert!(offset_of!(CatchClause, span) == 0);
    assert!(offset_of!(CatchClause, param) == 24);
    assert!(offset_of!(CatchClause, body) == 80);
    assert!(offset_of!(CatchClause, scope_id) == 88);

    // Padding: 0 bytes
    assert!(size_of::<CatchParameter>() == 56);
    assert!(align_of::<CatchParameter>() == 8);
    assert!(offset_of!(CatchParameter, span) == 0);
    assert!(offset_of!(CatchParameter, pattern) == 24);

    // Padding: 0 bytes
    assert!(size_of::<DebuggerStatement>() == 24);
    assert!(align_of::<DebuggerStatement>() == 8);
    assert!(offset_of!(DebuggerStatement, span) == 0);

    // Padding: 7 bytes
    assert!(size_of::<BindingPattern>() == 32);
    assert!(align_of::<BindingPattern>() == 8);
    assert!(offset_of!(BindingPattern, kind) == 0);
    assert!(offset_of!(BindingPattern, type_annotation) == 16);
    assert!(offset_of!(BindingPattern, optional) == 24);

    assert!(size_of::<BindingPatternKind>() == 16);
    assert!(align_of::<BindingPatternKind>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentPattern>() == 72);
    assert!(align_of::<AssignmentPattern>() == 8);
    assert!(offset_of!(AssignmentPattern, span) == 0);
    assert!(offset_of!(AssignmentPattern, left) == 24);
    assert!(offset_of!(AssignmentPattern, right) == 56);

    // Padding: 0 bytes
    assert!(size_of::<ObjectPattern>() == 56);
    assert!(align_of::<ObjectPattern>() == 8);
    assert!(offset_of!(ObjectPattern, span) == 0);
    assert!(offset_of!(ObjectPattern, properties) == 24);
    assert!(offset_of!(ObjectPattern, rest) == 48);

    // Padding: 6 bytes
    assert!(size_of::<BindingProperty>() == 80);
    assert!(align_of::<BindingProperty>() == 8);
    assert!(offset_of!(BindingProperty, span) == 0);
    assert!(offset_of!(BindingProperty, key) == 24);
    assert!(offset_of!(BindingProperty, value) == 40);
    assert!(offset_of!(BindingProperty, shorthand) == 72);
    assert!(offset_of!(BindingProperty, computed) == 73);

    // Padding: 0 bytes
    assert!(size_of::<ArrayPattern>() == 56);
    assert!(align_of::<ArrayPattern>() == 8);
    assert!(offset_of!(ArrayPattern, span) == 0);
    assert!(offset_of!(ArrayPattern, elements) == 24);
    assert!(offset_of!(ArrayPattern, rest) == 48);

    // Padding: 0 bytes
    assert!(size_of::<BindingRestElement>() == 56);
    assert!(align_of::<BindingRestElement>() == 8);
    assert!(offset_of!(BindingRestElement, span) == 0);
    assert!(offset_of!(BindingRestElement, argument) == 24);

    // Padding: 7 bytes
    assert!(size_of::<Function>() == 128);
    assert!(align_of::<Function>() == 8);
    assert!(offset_of!(Function, span) == 0);
    assert!(offset_of!(Function, r#type) == 116);
    assert!(offset_of!(Function, id) == 24);
    assert!(offset_of!(Function, generator) == 117);
    assert!(offset_of!(Function, r#async) == 118);
    assert!(offset_of!(Function, declare) == 119);
    assert!(offset_of!(Function, type_parameters) == 72);
    assert!(offset_of!(Function, this_param) == 80);
    assert!(offset_of!(Function, params) == 88);
    assert!(offset_of!(Function, return_type) == 96);
    assert!(offset_of!(Function, body) == 104);
    assert!(offset_of!(Function, scope_id) == 112);
    assert!(offset_of!(Function, pure) == 120);

    assert!(size_of::<FunctionType>() == 1);
    assert!(align_of::<FunctionType>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<FormalParameters>() == 64);
    assert!(align_of::<FormalParameters>() == 8);
    assert!(offset_of!(FormalParameters, span) == 0);
    assert!(offset_of!(FormalParameters, kind) == 56);
    assert!(offset_of!(FormalParameters, items) == 24);
    assert!(offset_of!(FormalParameters, rest) == 48);

    // Padding: 5 bytes
    assert!(size_of::<FormalParameter>() == 88);
    assert!(align_of::<FormalParameter>() == 8);
    assert!(offset_of!(FormalParameter, span) == 0);
    assert!(offset_of!(FormalParameter, decorators) == 24);
    assert!(offset_of!(FormalParameter, pattern) == 48);
    assert!(offset_of!(FormalParameter, accessibility) == 80);
    assert!(offset_of!(FormalParameter, readonly) == 81);
    assert!(offset_of!(FormalParameter, r#override) == 82);

    assert!(size_of::<FormalParameterKind>() == 1);
    assert!(align_of::<FormalParameterKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<FunctionBody>() == 72);
    assert!(align_of::<FunctionBody>() == 8);
    assert!(offset_of!(FunctionBody, span) == 0);
    assert!(offset_of!(FunctionBody, directives) == 24);
    assert!(offset_of!(FunctionBody, statements) == 48);

    // Padding: 1 bytes
    assert!(size_of::<ArrowFunctionExpression>() == 64);
    assert!(align_of::<ArrowFunctionExpression>() == 8);
    assert!(offset_of!(ArrowFunctionExpression, span) == 0);
    assert!(offset_of!(ArrowFunctionExpression, expression) == 60);
    assert!(offset_of!(ArrowFunctionExpression, r#async) == 61);
    assert!(offset_of!(ArrowFunctionExpression, type_parameters) == 24);
    assert!(offset_of!(ArrowFunctionExpression, params) == 32);
    assert!(offset_of!(ArrowFunctionExpression, return_type) == 40);
    assert!(offset_of!(ArrowFunctionExpression, body) == 48);
    assert!(offset_of!(ArrowFunctionExpression, scope_id) == 56);
    assert!(offset_of!(ArrowFunctionExpression, pure) == 62);

    // Padding: 7 bytes
    assert!(size_of::<YieldExpression>() == 48);
    assert!(align_of::<YieldExpression>() == 8);
    assert!(offset_of!(YieldExpression, span) == 0);
    assert!(offset_of!(YieldExpression, delegate) == 40);
    assert!(offset_of!(YieldExpression, argument) == 24);

    // Padding: 1 bytes
    assert!(size_of::<Class>() == 168);
    assert!(align_of::<Class>() == 8);
    assert!(offset_of!(Class, span) == 0);
    assert!(offset_of!(Class, r#type) == 164);
    assert!(offset_of!(Class, decorators) == 24);
    assert!(offset_of!(Class, id) == 48);
    assert!(offset_of!(Class, type_parameters) == 96);
    assert!(offset_of!(Class, super_class) == 104);
    assert!(offset_of!(Class, super_type_arguments) == 120);
    assert!(offset_of!(Class, implements) == 128);
    assert!(offset_of!(Class, body) == 152);
    assert!(offset_of!(Class, r#abstract) == 165);
    assert!(offset_of!(Class, declare) == 166);
    assert!(offset_of!(Class, scope_id) == 160);

    assert!(size_of::<ClassType>() == 1);
    assert!(align_of::<ClassType>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ClassBody>() == 48);
    assert!(align_of::<ClassBody>() == 8);
    assert!(offset_of!(ClassBody, span) == 0);
    assert!(offset_of!(ClassBody, body) == 24);

    assert!(size_of::<ClassElement>() == 16);
    assert!(align_of::<ClassElement>() == 8);

    // Padding: 1 bytes
    assert!(size_of::<MethodDefinition>() == 80);
    assert!(align_of::<MethodDefinition>() == 8);
    assert!(offset_of!(MethodDefinition, span) == 0);
    assert!(offset_of!(MethodDefinition, r#type) == 72);
    assert!(offset_of!(MethodDefinition, decorators) == 24);
    assert!(offset_of!(MethodDefinition, key) == 48);
    assert!(offset_of!(MethodDefinition, value) == 64);
    assert!(offset_of!(MethodDefinition, kind) == 73);
    assert!(offset_of!(MethodDefinition, computed) == 74);
    assert!(offset_of!(MethodDefinition, r#static) == 75);
    assert!(offset_of!(MethodDefinition, r#override) == 76);
    assert!(offset_of!(MethodDefinition, optional) == 77);
    assert!(offset_of!(MethodDefinition, accessibility) == 78);

    assert!(size_of::<MethodDefinitionType>() == 1);
    assert!(align_of::<MethodDefinitionType>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<PropertyDefinition>() == 104);
    assert!(align_of::<PropertyDefinition>() == 8);
    assert!(offset_of!(PropertyDefinition, span) == 0);
    assert!(offset_of!(PropertyDefinition, r#type) == 88);
    assert!(offset_of!(PropertyDefinition, decorators) == 24);
    assert!(offset_of!(PropertyDefinition, key) == 48);
    assert!(offset_of!(PropertyDefinition, type_annotation) == 64);
    assert!(offset_of!(PropertyDefinition, value) == 72);
    assert!(offset_of!(PropertyDefinition, computed) == 89);
    assert!(offset_of!(PropertyDefinition, r#static) == 90);
    assert!(offset_of!(PropertyDefinition, declare) == 91);
    assert!(offset_of!(PropertyDefinition, r#override) == 92);
    assert!(offset_of!(PropertyDefinition, optional) == 93);
    assert!(offset_of!(PropertyDefinition, definite) == 94);
    assert!(offset_of!(PropertyDefinition, readonly) == 95);
    assert!(offset_of!(PropertyDefinition, accessibility) == 96);

    assert!(size_of::<PropertyDefinitionType>() == 1);
    assert!(align_of::<PropertyDefinitionType>() == 1);

    assert!(size_of::<MethodDefinitionKind>() == 1);
    assert!(align_of::<MethodDefinitionKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<PrivateIdentifier>() == 40);
    assert!(align_of::<PrivateIdentifier>() == 8);
    assert!(offset_of!(PrivateIdentifier, span) == 0);
    assert!(offset_of!(PrivateIdentifier, name) == 24);

    // Padding: 4 bytes
    assert!(size_of::<StaticBlock>() == 56);
    assert!(align_of::<StaticBlock>() == 8);
    assert!(offset_of!(StaticBlock, span) == 0);
    assert!(offset_of!(StaticBlock, body) == 24);
    assert!(offset_of!(StaticBlock, scope_id) == 48);

    assert!(size_of::<ModuleDeclaration>() == 16);
    assert!(align_of::<ModuleDeclaration>() == 8);

    assert!(size_of::<AccessorPropertyType>() == 1);
    assert!(align_of::<AccessorPropertyType>() == 1);

    // Padding: 2 bytes
    assert!(size_of::<AccessorProperty>() == 96);
    assert!(align_of::<AccessorProperty>() == 8);
    assert!(offset_of!(AccessorProperty, span) == 0);
    assert!(offset_of!(AccessorProperty, r#type) == 88);
    assert!(offset_of!(AccessorProperty, decorators) == 24);
    assert!(offset_of!(AccessorProperty, key) == 48);
    assert!(offset_of!(AccessorProperty, type_annotation) == 64);
    assert!(offset_of!(AccessorProperty, value) == 72);
    assert!(offset_of!(AccessorProperty, computed) == 89);
    assert!(offset_of!(AccessorProperty, r#static) == 90);
    assert!(offset_of!(AccessorProperty, r#override) == 91);
    assert!(offset_of!(AccessorProperty, definite) == 92);
    assert!(offset_of!(AccessorProperty, accessibility) == 93);

    // Padding: 7 bytes
    assert!(size_of::<ImportExpression>() == 64);
    assert!(align_of::<ImportExpression>() == 8);
    assert!(offset_of!(ImportExpression, span) == 0);
    assert!(offset_of!(ImportExpression, source) == 24);
    assert!(offset_of!(ImportExpression, options) == 40);
    assert!(offset_of!(ImportExpression, phase) == 56);

    // Padding: 6 bytes
    assert!(size_of::<ImportDeclaration>() == 128);
    assert!(align_of::<ImportDeclaration>() == 8);
    assert!(offset_of!(ImportDeclaration, span) == 0);
    assert!(offset_of!(ImportDeclaration, specifiers) == 24);
    assert!(offset_of!(ImportDeclaration, source) == 48);
    assert!(offset_of!(ImportDeclaration, phase) == 120);
    assert!(offset_of!(ImportDeclaration, with_clause) == 112);
    assert!(offset_of!(ImportDeclaration, import_kind) == 121);

    assert!(size_of::<ImportPhase>() == 1);
    assert!(align_of::<ImportPhase>() == 1);

    assert!(size_of::<ImportDeclarationSpecifier>() == 16);
    assert!(align_of::<ImportDeclarationSpecifier>() == 8);

    // Padding: 7 bytes
    assert!(size_of::<ImportSpecifier>() == 152);
    assert!(align_of::<ImportSpecifier>() == 8);
    assert!(offset_of!(ImportSpecifier, span) == 0);
    assert!(offset_of!(ImportSpecifier, imported) == 24);
    assert!(offset_of!(ImportSpecifier, local) == 96);
    assert!(offset_of!(ImportSpecifier, import_kind) == 144);

    // Padding: 0 bytes
    assert!(size_of::<ImportDefaultSpecifier>() == 72);
    assert!(align_of::<ImportDefaultSpecifier>() == 8);
    assert!(offset_of!(ImportDefaultSpecifier, span) == 0);
    assert!(offset_of!(ImportDefaultSpecifier, local) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ImportNamespaceSpecifier>() == 72);
    assert!(align_of::<ImportNamespaceSpecifier>() == 8);
    assert!(offset_of!(ImportNamespaceSpecifier, span) == 0);
    assert!(offset_of!(ImportNamespaceSpecifier, local) == 24);

    // Padding: 0 bytes
    assert!(size_of::<WithClause>() == 88);
    assert!(align_of::<WithClause>() == 8);
    assert!(offset_of!(WithClause, span) == 0);
    assert!(offset_of!(WithClause, attributes_keyword) == 24);
    assert!(offset_of!(WithClause, with_entries) == 64);

    // Padding: 0 bytes
    assert!(size_of::<ImportAttribute>() == 160);
    assert!(align_of::<ImportAttribute>() == 8);
    assert!(offset_of!(ImportAttribute, span) == 0);
    assert!(offset_of!(ImportAttribute, key) == 24);
    assert!(offset_of!(ImportAttribute, value) == 96);

    assert!(size_of::<ImportAttributeKey>() == 72);
    assert!(align_of::<ImportAttributeKey>() == 8);

    // Padding: 7 bytes
    assert!(size_of::<ExportNamedDeclaration>() == 144);
    assert!(align_of::<ExportNamedDeclaration>() == 8);
    assert!(offset_of!(ExportNamedDeclaration, span) == 0);
    assert!(offset_of!(ExportNamedDeclaration, declaration) == 24);
    assert!(offset_of!(ExportNamedDeclaration, specifiers) == 40);
    assert!(offset_of!(ExportNamedDeclaration, source) == 64);
    assert!(offset_of!(ExportNamedDeclaration, export_kind) == 136);
    assert!(offset_of!(ExportNamedDeclaration, with_clause) == 128);

    // Padding: 0 bytes
    assert!(size_of::<ExportDefaultDeclaration>() == 112);
    assert!(align_of::<ExportDefaultDeclaration>() == 8);
    assert!(offset_of!(ExportDefaultDeclaration, span) == 0);
    assert!(offset_of!(ExportDefaultDeclaration, exported) == 24);
    assert!(offset_of!(ExportDefaultDeclaration, declaration) == 96);

    // Padding: 7 bytes
    assert!(size_of::<ExportAllDeclaration>() == 176);
    assert!(align_of::<ExportAllDeclaration>() == 8);
    assert!(offset_of!(ExportAllDeclaration, span) == 0);
    assert!(offset_of!(ExportAllDeclaration, exported) == 24);
    assert!(offset_of!(ExportAllDeclaration, source) == 96);
    assert!(offset_of!(ExportAllDeclaration, with_clause) == 160);
    assert!(offset_of!(ExportAllDeclaration, export_kind) == 168);

    // Padding: 7 bytes
    assert!(size_of::<ExportSpecifier>() == 176);
    assert!(align_of::<ExportSpecifier>() == 8);
    assert!(offset_of!(ExportSpecifier, span) == 0);
    assert!(offset_of!(ExportSpecifier, local) == 24);
    assert!(offset_of!(ExportSpecifier, exported) == 96);
    assert!(offset_of!(ExportSpecifier, export_kind) == 168);

    assert!(size_of::<ExportDefaultDeclarationKind>() == 16);
    assert!(align_of::<ExportDefaultDeclarationKind>() == 8);

    assert!(size_of::<ModuleExportName>() == 72);
    assert!(align_of::<ModuleExportName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<V8IntrinsicExpression>() == 88);
    assert!(align_of::<V8IntrinsicExpression>() == 8);
    assert!(offset_of!(V8IntrinsicExpression, span) == 0);
    assert!(offset_of!(V8IntrinsicExpression, name) == 24);
    assert!(offset_of!(V8IntrinsicExpression, arguments) == 64);

    // Padding: 7 bytes
    assert!(size_of::<BooleanLiteral>() == 32);
    assert!(align_of::<BooleanLiteral>() == 8);
    assert!(offset_of!(BooleanLiteral, span) == 0);
    assert!(offset_of!(BooleanLiteral, value) == 24);

    // Padding: 0 bytes
    assert!(size_of::<NullLiteral>() == 24);
    assert!(align_of::<NullLiteral>() == 8);
    assert!(offset_of!(NullLiteral, span) == 0);

    // Padding: 7 bytes
    assert!(size_of::<NumericLiteral>() == 56);
    assert!(align_of::<NumericLiteral>() == 8);
    assert!(offset_of!(NumericLiteral, span) == 0);
    assert!(offset_of!(NumericLiteral, value) == 24);
    assert!(offset_of!(NumericLiteral, raw) == 32);
    assert!(offset_of!(NumericLiteral, base) == 48);

    // Padding: 7 bytes
    assert!(size_of::<StringLiteral>() == 64);
    assert!(align_of::<StringLiteral>() == 8);
    assert!(offset_of!(StringLiteral, span) == 0);
    assert!(offset_of!(StringLiteral, value) == 24);
    assert!(offset_of!(StringLiteral, raw) == 40);
    assert!(offset_of!(StringLiteral, lone_surrogates) == 56);

    // Padding: 7 bytes
    assert!(size_of::<BigIntLiteral>() == 64);
    assert!(align_of::<BigIntLiteral>() == 8);
    assert!(offset_of!(BigIntLiteral, span) == 0);
    assert!(offset_of!(BigIntLiteral, value) == 24);
    assert!(offset_of!(BigIntLiteral, raw) == 40);
    assert!(offset_of!(BigIntLiteral, base) == 56);

    // Padding: 0 bytes
    assert!(size_of::<RegExpLiteral>() == 72);
    assert!(align_of::<RegExpLiteral>() == 8);
    assert!(offset_of!(RegExpLiteral, span) == 0);
    assert!(offset_of!(RegExpLiteral, regex) == 24);
    assert!(offset_of!(RegExpLiteral, raw) == 56);

    // Padding: 7 bytes
    assert!(size_of::<RegExp>() == 32);
    assert!(align_of::<RegExp>() == 8);
    assert!(offset_of!(RegExp, pattern) == 0);
    assert!(offset_of!(RegExp, flags) == 24);

    // Padding: 0 bytes
    assert!(size_of::<RegExpPattern>() == 24);
    assert!(align_of::<RegExpPattern>() == 8);
    assert!(offset_of!(RegExpPattern, text) == 0);
    assert!(offset_of!(RegExpPattern, pattern) == 16);

    // Padding: 0 bytes
    assert!(size_of::<RegExpFlags>() == 1);
    assert!(align_of::<RegExpFlags>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<JSXElement>() == 64);
    assert!(align_of::<JSXElement>() == 8);
    assert!(offset_of!(JSXElement, span) == 0);
    assert!(offset_of!(JSXElement, opening_element) == 24);
    assert!(offset_of!(JSXElement, children) == 32);
    assert!(offset_of!(JSXElement, closing_element) == 56);

    // Padding: 0 bytes
    assert!(size_of::<JSXOpeningElement>() == 72);
    assert!(align_of::<JSXOpeningElement>() == 8);
    assert!(offset_of!(JSXOpeningElement, span) == 0);
    assert!(offset_of!(JSXOpeningElement, name) == 24);
    assert!(offset_of!(JSXOpeningElement, type_arguments) == 40);
    assert!(offset_of!(JSXOpeningElement, attributes) == 48);

    // Padding: 0 bytes
    assert!(size_of::<JSXClosingElement>() == 40);
    assert!(align_of::<JSXClosingElement>() == 8);
    assert!(offset_of!(JSXClosingElement, span) == 0);
    assert!(offset_of!(JSXClosingElement, name) == 24);

    // Padding: 0 bytes
    assert!(size_of::<JSXFragment>() == 96);
    assert!(align_of::<JSXFragment>() == 8);
    assert!(offset_of!(JSXFragment, span) == 0);
    assert!(offset_of!(JSXFragment, opening_fragment) == 24);
    assert!(offset_of!(JSXFragment, children) == 48);
    assert!(offset_of!(JSXFragment, closing_fragment) == 72);

    // Padding: 0 bytes
    assert!(size_of::<JSXOpeningFragment>() == 24);
    assert!(align_of::<JSXOpeningFragment>() == 8);
    assert!(offset_of!(JSXOpeningFragment, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<JSXClosingFragment>() == 24);
    assert!(align_of::<JSXClosingFragment>() == 8);
    assert!(offset_of!(JSXClosingFragment, span) == 0);

    assert!(size_of::<JSXElementName>() == 16);
    assert!(align_of::<JSXElementName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXNamespacedName>() == 104);
    assert!(align_of::<JSXNamespacedName>() == 8);
    assert!(offset_of!(JSXNamespacedName, span) == 0);
    assert!(offset_of!(JSXNamespacedName, namespace) == 24);
    assert!(offset_of!(JSXNamespacedName, name) == 64);

    // Padding: 0 bytes
    assert!(size_of::<JSXMemberExpression>() == 80);
    assert!(align_of::<JSXMemberExpression>() == 8);
    assert!(offset_of!(JSXMemberExpression, span) == 0);
    assert!(offset_of!(JSXMemberExpression, object) == 24);
    assert!(offset_of!(JSXMemberExpression, property) == 40);

    assert!(size_of::<JSXMemberExpressionObject>() == 16);
    assert!(align_of::<JSXMemberExpressionObject>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXExpressionContainer>() == 56);
    assert!(align_of::<JSXExpressionContainer>() == 8);
    assert!(offset_of!(JSXExpressionContainer, span) == 0);
    assert!(offset_of!(JSXExpressionContainer, expression) == 24);

    assert!(size_of::<JSXExpression>() == 32);
    assert!(align_of::<JSXExpression>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXEmptyExpression>() == 24);
    assert!(align_of::<JSXEmptyExpression>() == 8);
    assert!(offset_of!(JSXEmptyExpression, span) == 0);

    assert!(size_of::<JSXAttributeItem>() == 16);
    assert!(align_of::<JSXAttributeItem>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXAttribute>() == 56);
    assert!(align_of::<JSXAttribute>() == 8);
    assert!(offset_of!(JSXAttribute, span) == 0);
    assert!(offset_of!(JSXAttribute, name) == 24);
    assert!(offset_of!(JSXAttribute, value) == 40);

    // Padding: 0 bytes
    assert!(size_of::<JSXSpreadAttribute>() == 40);
    assert!(align_of::<JSXSpreadAttribute>() == 8);
    assert!(offset_of!(JSXSpreadAttribute, span) == 0);
    assert!(offset_of!(JSXSpreadAttribute, argument) == 24);

    assert!(size_of::<JSXAttributeName>() == 16);
    assert!(align_of::<JSXAttributeName>() == 8);

    assert!(size_of::<JSXAttributeValue>() == 16);
    assert!(align_of::<JSXAttributeValue>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXIdentifier>() == 40);
    assert!(align_of::<JSXIdentifier>() == 8);
    assert!(offset_of!(JSXIdentifier, span) == 0);
    assert!(offset_of!(JSXIdentifier, name) == 24);

    assert!(size_of::<JSXChild>() == 16);
    assert!(align_of::<JSXChild>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXSpreadChild>() == 40);
    assert!(align_of::<JSXSpreadChild>() == 8);
    assert!(offset_of!(JSXSpreadChild, span) == 0);
    assert!(offset_of!(JSXSpreadChild, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<JSXText>() == 56);
    assert!(align_of::<JSXText>() == 8);
    assert!(offset_of!(JSXText, span) == 0);
    assert!(offset_of!(JSXText, value) == 24);
    assert!(offset_of!(JSXText, raw) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSThisParameter>() == 56);
    assert!(align_of::<TSThisParameter>() == 8);
    assert!(offset_of!(TSThisParameter, span) == 0);
    assert!(offset_of!(TSThisParameter, this_span) == 24);
    assert!(offset_of!(TSThisParameter, type_annotation) == 48);

    // Padding: 2 bytes
    assert!(size_of::<TSEnumDeclaration>() == 128);
    assert!(align_of::<TSEnumDeclaration>() == 8);
    assert!(offset_of!(TSEnumDeclaration, span) == 0);
    assert!(offset_of!(TSEnumDeclaration, id) == 24);
    assert!(offset_of!(TSEnumDeclaration, body) == 72);
    assert!(offset_of!(TSEnumDeclaration, r#const) == 124);
    assert!(offset_of!(TSEnumDeclaration, declare) == 125);
    assert!(offset_of!(TSEnumDeclaration, scope_id) == 120);

    // Padding: 0 bytes
    assert!(size_of::<TSEnumBody>() == 48);
    assert!(align_of::<TSEnumBody>() == 8);
    assert!(offset_of!(TSEnumBody, span) == 0);
    assert!(offset_of!(TSEnumBody, members) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSEnumMember>() == 56);
    assert!(align_of::<TSEnumMember>() == 8);
    assert!(offset_of!(TSEnumMember, span) == 0);
    assert!(offset_of!(TSEnumMember, id) == 24);
    assert!(offset_of!(TSEnumMember, initializer) == 40);

    assert!(size_of::<TSEnumMemberName>() == 16);
    assert!(align_of::<TSEnumMemberName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeAnnotation>() == 40);
    assert!(align_of::<TSTypeAnnotation>() == 8);
    assert!(offset_of!(TSTypeAnnotation, span) == 0);
    assert!(offset_of!(TSTypeAnnotation, type_annotation) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSLiteralType>() == 40);
    assert!(align_of::<TSLiteralType>() == 8);
    assert!(offset_of!(TSLiteralType, span) == 0);
    assert!(offset_of!(TSLiteralType, literal) == 24);

    assert!(size_of::<TSLiteral>() == 16);
    assert!(align_of::<TSLiteral>() == 8);

    assert!(size_of::<TSType>() == 16);
    assert!(align_of::<TSType>() == 8);

    // Padding: 4 bytes
    assert!(size_of::<TSConditionalType>() == 96);
    assert!(align_of::<TSConditionalType>() == 8);
    assert!(offset_of!(TSConditionalType, span) == 0);
    assert!(offset_of!(TSConditionalType, check_type) == 24);
    assert!(offset_of!(TSConditionalType, extends_type) == 40);
    assert!(offset_of!(TSConditionalType, true_type) == 56);
    assert!(offset_of!(TSConditionalType, false_type) == 72);
    assert!(offset_of!(TSConditionalType, scope_id) == 88);

    // Padding: 0 bytes
    assert!(size_of::<TSUnionType>() == 48);
    assert!(align_of::<TSUnionType>() == 8);
    assert!(offset_of!(TSUnionType, span) == 0);
    assert!(offset_of!(TSUnionType, types) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSIntersectionType>() == 48);
    assert!(align_of::<TSIntersectionType>() == 8);
    assert!(offset_of!(TSIntersectionType, span) == 0);
    assert!(offset_of!(TSIntersectionType, types) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSParenthesizedType>() == 40);
    assert!(align_of::<TSParenthesizedType>() == 8);
    assert!(offset_of!(TSParenthesizedType, span) == 0);
    assert!(offset_of!(TSParenthesizedType, type_annotation) == 24);

    // Padding: 7 bytes
    assert!(size_of::<TSTypeOperator>() == 48);
    assert!(align_of::<TSTypeOperator>() == 8);
    assert!(offset_of!(TSTypeOperator, span) == 0);
    assert!(offset_of!(TSTypeOperator, operator) == 40);
    assert!(offset_of!(TSTypeOperator, type_annotation) == 24);

    assert!(size_of::<TSTypeOperatorOperator>() == 1);
    assert!(align_of::<TSTypeOperatorOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSArrayType>() == 40);
    assert!(align_of::<TSArrayType>() == 8);
    assert!(offset_of!(TSArrayType, span) == 0);
    assert!(offset_of!(TSArrayType, element_type) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSIndexedAccessType>() == 56);
    assert!(align_of::<TSIndexedAccessType>() == 8);
    assert!(offset_of!(TSIndexedAccessType, span) == 0);
    assert!(offset_of!(TSIndexedAccessType, object_type) == 24);
    assert!(offset_of!(TSIndexedAccessType, index_type) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSTupleType>() == 48);
    assert!(align_of::<TSTupleType>() == 8);
    assert!(offset_of!(TSTupleType, span) == 0);
    assert!(offset_of!(TSTupleType, element_types) == 24);

    // Padding: 7 bytes
    assert!(size_of::<TSNamedTupleMember>() == 88);
    assert!(align_of::<TSNamedTupleMember>() == 8);
    assert!(offset_of!(TSNamedTupleMember, span) == 0);
    assert!(offset_of!(TSNamedTupleMember, label) == 24);
    assert!(offset_of!(TSNamedTupleMember, element_type) == 64);
    assert!(offset_of!(TSNamedTupleMember, optional) == 80);

    // Padding: 0 bytes
    assert!(size_of::<TSOptionalType>() == 40);
    assert!(align_of::<TSOptionalType>() == 8);
    assert!(offset_of!(TSOptionalType, span) == 0);
    assert!(offset_of!(TSOptionalType, type_annotation) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSRestType>() == 40);
    assert!(align_of::<TSRestType>() == 8);
    assert!(offset_of!(TSRestType, span) == 0);
    assert!(offset_of!(TSRestType, type_annotation) == 24);

    assert!(size_of::<TSTupleElement>() == 16);
    assert!(align_of::<TSTupleElement>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSAnyKeyword>() == 24);
    assert!(align_of::<TSAnyKeyword>() == 8);
    assert!(offset_of!(TSAnyKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSStringKeyword>() == 24);
    assert!(align_of::<TSStringKeyword>() == 8);
    assert!(offset_of!(TSStringKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSBooleanKeyword>() == 24);
    assert!(align_of::<TSBooleanKeyword>() == 8);
    assert!(offset_of!(TSBooleanKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNumberKeyword>() == 24);
    assert!(align_of::<TSNumberKeyword>() == 8);
    assert!(offset_of!(TSNumberKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNeverKeyword>() == 24);
    assert!(align_of::<TSNeverKeyword>() == 8);
    assert!(offset_of!(TSNeverKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSIntrinsicKeyword>() == 24);
    assert!(align_of::<TSIntrinsicKeyword>() == 8);
    assert!(offset_of!(TSIntrinsicKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSUnknownKeyword>() == 24);
    assert!(align_of::<TSUnknownKeyword>() == 8);
    assert!(offset_of!(TSUnknownKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNullKeyword>() == 24);
    assert!(align_of::<TSNullKeyword>() == 8);
    assert!(offset_of!(TSNullKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSUndefinedKeyword>() == 24);
    assert!(align_of::<TSUndefinedKeyword>() == 8);
    assert!(offset_of!(TSUndefinedKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSVoidKeyword>() == 24);
    assert!(align_of::<TSVoidKeyword>() == 8);
    assert!(offset_of!(TSVoidKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSSymbolKeyword>() == 24);
    assert!(align_of::<TSSymbolKeyword>() == 8);
    assert!(offset_of!(TSSymbolKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSThisType>() == 24);
    assert!(align_of::<TSThisType>() == 8);
    assert!(offset_of!(TSThisType, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSObjectKeyword>() == 24);
    assert!(align_of::<TSObjectKeyword>() == 8);
    assert!(offset_of!(TSObjectKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSBigIntKeyword>() == 24);
    assert!(align_of::<TSBigIntKeyword>() == 8);
    assert!(offset_of!(TSBigIntKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeReference>() == 48);
    assert!(align_of::<TSTypeReference>() == 8);
    assert!(offset_of!(TSTypeReference, span) == 0);
    assert!(offset_of!(TSTypeReference, type_name) == 24);
    assert!(offset_of!(TSTypeReference, type_arguments) == 40);

    assert!(size_of::<TSTypeName>() == 16);
    assert!(align_of::<TSTypeName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSQualifiedName>() == 80);
    assert!(align_of::<TSQualifiedName>() == 8);
    assert!(offset_of!(TSQualifiedName, span) == 0);
    assert!(offset_of!(TSQualifiedName, left) == 24);
    assert!(offset_of!(TSQualifiedName, right) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeParameterInstantiation>() == 48);
    assert!(align_of::<TSTypeParameterInstantiation>() == 8);
    assert!(offset_of!(TSTypeParameterInstantiation, span) == 0);
    assert!(offset_of!(TSTypeParameterInstantiation, params) == 24);

    // Padding: 5 bytes
    assert!(size_of::<TSTypeParameter>() == 112);
    assert!(align_of::<TSTypeParameter>() == 8);
    assert!(offset_of!(TSTypeParameter, span) == 0);
    assert!(offset_of!(TSTypeParameter, name) == 24);
    assert!(offset_of!(TSTypeParameter, constraint) == 72);
    assert!(offset_of!(TSTypeParameter, default) == 88);
    assert!(offset_of!(TSTypeParameter, r#in) == 104);
    assert!(offset_of!(TSTypeParameter, out) == 105);
    assert!(offset_of!(TSTypeParameter, r#const) == 106);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeParameterDeclaration>() == 48);
    assert!(align_of::<TSTypeParameterDeclaration>() == 8);
    assert!(offset_of!(TSTypeParameterDeclaration, span) == 0);
    assert!(offset_of!(TSTypeParameterDeclaration, params) == 24);

    // Padding: 3 bytes
    assert!(size_of::<TSTypeAliasDeclaration>() == 104);
    assert!(align_of::<TSTypeAliasDeclaration>() == 8);
    assert!(offset_of!(TSTypeAliasDeclaration, span) == 0);
    assert!(offset_of!(TSTypeAliasDeclaration, id) == 24);
    assert!(offset_of!(TSTypeAliasDeclaration, type_parameters) == 72);
    assert!(offset_of!(TSTypeAliasDeclaration, type_annotation) == 80);
    assert!(offset_of!(TSTypeAliasDeclaration, declare) == 100);
    assert!(offset_of!(TSTypeAliasDeclaration, scope_id) == 96);

    assert!(size_of::<TSAccessibility>() == 1);
    assert!(align_of::<TSAccessibility>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSClassImplements>() == 48);
    assert!(align_of::<TSClassImplements>() == 8);
    assert!(offset_of!(TSClassImplements, span) == 0);
    assert!(offset_of!(TSClassImplements, expression) == 24);
    assert!(offset_of!(TSClassImplements, type_arguments) == 40);

    // Padding: 3 bytes
    assert!(size_of::<TSInterfaceDeclaration>() == 120);
    assert!(align_of::<TSInterfaceDeclaration>() == 8);
    assert!(offset_of!(TSInterfaceDeclaration, span) == 0);
    assert!(offset_of!(TSInterfaceDeclaration, id) == 24);
    assert!(offset_of!(TSInterfaceDeclaration, type_parameters) == 72);
    assert!(offset_of!(TSInterfaceDeclaration, extends) == 80);
    assert!(offset_of!(TSInterfaceDeclaration, body) == 104);
    assert!(offset_of!(TSInterfaceDeclaration, declare) == 116);
    assert!(offset_of!(TSInterfaceDeclaration, scope_id) == 112);

    // Padding: 0 bytes
    assert!(size_of::<TSInterfaceBody>() == 48);
    assert!(align_of::<TSInterfaceBody>() == 8);
    assert!(offset_of!(TSInterfaceBody, span) == 0);
    assert!(offset_of!(TSInterfaceBody, body) == 24);

    // Padding: 5 bytes
    assert!(size_of::<TSPropertySignature>() == 56);
    assert!(align_of::<TSPropertySignature>() == 8);
    assert!(offset_of!(TSPropertySignature, span) == 0);
    assert!(offset_of!(TSPropertySignature, computed) == 48);
    assert!(offset_of!(TSPropertySignature, optional) == 49);
    assert!(offset_of!(TSPropertySignature, readonly) == 50);
    assert!(offset_of!(TSPropertySignature, key) == 24);
    assert!(offset_of!(TSPropertySignature, type_annotation) == 40);

    assert!(size_of::<TSSignature>() == 16);
    assert!(align_of::<TSSignature>() == 8);

    // Padding: 6 bytes
    assert!(size_of::<TSIndexSignature>() == 64);
    assert!(align_of::<TSIndexSignature>() == 8);
    assert!(offset_of!(TSIndexSignature, span) == 0);
    assert!(offset_of!(TSIndexSignature, parameters) == 24);
    assert!(offset_of!(TSIndexSignature, type_annotation) == 48);
    assert!(offset_of!(TSIndexSignature, readonly) == 56);
    assert!(offset_of!(TSIndexSignature, r#static) == 57);

    // Padding: 0 bytes
    assert!(size_of::<TSCallSignatureDeclaration>() == 56);
    assert!(align_of::<TSCallSignatureDeclaration>() == 8);
    assert!(offset_of!(TSCallSignatureDeclaration, span) == 0);
    assert!(offset_of!(TSCallSignatureDeclaration, type_parameters) == 24);
    assert!(offset_of!(TSCallSignatureDeclaration, this_param) == 32);
    assert!(offset_of!(TSCallSignatureDeclaration, params) == 40);
    assert!(offset_of!(TSCallSignatureDeclaration, return_type) == 48);

    assert!(size_of::<TSMethodSignatureKind>() == 1);
    assert!(align_of::<TSMethodSignatureKind>() == 1);

    // Padding: 1 bytes
    assert!(size_of::<TSMethodSignature>() == 80);
    assert!(align_of::<TSMethodSignature>() == 8);
    assert!(offset_of!(TSMethodSignature, span) == 0);
    assert!(offset_of!(TSMethodSignature, key) == 24);
    assert!(offset_of!(TSMethodSignature, computed) == 76);
    assert!(offset_of!(TSMethodSignature, optional) == 77);
    assert!(offset_of!(TSMethodSignature, kind) == 78);
    assert!(offset_of!(TSMethodSignature, type_parameters) == 40);
    assert!(offset_of!(TSMethodSignature, this_param) == 48);
    assert!(offset_of!(TSMethodSignature, params) == 56);
    assert!(offset_of!(TSMethodSignature, return_type) == 64);
    assert!(offset_of!(TSMethodSignature, scope_id) == 72);

    // Padding: 4 bytes
    assert!(size_of::<TSConstructSignatureDeclaration>() == 56);
    assert!(align_of::<TSConstructSignatureDeclaration>() == 8);
    assert!(offset_of!(TSConstructSignatureDeclaration, span) == 0);
    assert!(offset_of!(TSConstructSignatureDeclaration, type_parameters) == 24);
    assert!(offset_of!(TSConstructSignatureDeclaration, params) == 32);
    assert!(offset_of!(TSConstructSignatureDeclaration, return_type) == 40);
    assert!(offset_of!(TSConstructSignatureDeclaration, scope_id) == 48);

    // Padding: 0 bytes
    assert!(size_of::<TSIndexSignatureName>() == 48);
    assert!(align_of::<TSIndexSignatureName>() == 8);
    assert!(offset_of!(TSIndexSignatureName, span) == 0);
    assert!(offset_of!(TSIndexSignatureName, name) == 24);
    assert!(offset_of!(TSIndexSignatureName, type_annotation) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSInterfaceHeritage>() == 48);
    assert!(align_of::<TSInterfaceHeritage>() == 8);
    assert!(offset_of!(TSInterfaceHeritage, span) == 0);
    assert!(offset_of!(TSInterfaceHeritage, expression) == 24);
    assert!(offset_of!(TSInterfaceHeritage, type_arguments) == 40);

    // Padding: 7 bytes
    assert!(size_of::<TSTypePredicate>() == 72);
    assert!(align_of::<TSTypePredicate>() == 8);
    assert!(offset_of!(TSTypePredicate, span) == 0);
    assert!(offset_of!(TSTypePredicate, parameter_name) == 24);
    assert!(offset_of!(TSTypePredicate, asserts) == 64);
    assert!(offset_of!(TSTypePredicate, type_annotation) == 56);

    assert!(size_of::<TSTypePredicateName>() == 32);
    assert!(align_of::<TSTypePredicateName>() == 8);

    // Padding: 2 bytes
    assert!(size_of::<TSModuleDeclaration>() == 120);
    assert!(align_of::<TSModuleDeclaration>() == 8);
    assert!(offset_of!(TSModuleDeclaration, span) == 0);
    assert!(offset_of!(TSModuleDeclaration, id) == 24);
    assert!(offset_of!(TSModuleDeclaration, body) == 96);
    assert!(offset_of!(TSModuleDeclaration, kind) == 116);
    assert!(offset_of!(TSModuleDeclaration, declare) == 117);
    assert!(offset_of!(TSModuleDeclaration, scope_id) == 112);

    assert!(size_of::<TSModuleDeclarationKind>() == 1);
    assert!(align_of::<TSModuleDeclarationKind>() == 1);

    assert!(size_of::<TSModuleDeclarationName>() == 72);
    assert!(align_of::<TSModuleDeclarationName>() == 8);

    assert!(size_of::<TSModuleDeclarationBody>() == 16);
    assert!(align_of::<TSModuleDeclarationBody>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSModuleBlock>() == 72);
    assert!(align_of::<TSModuleBlock>() == 8);
    assert!(offset_of!(TSModuleBlock, span) == 0);
    assert!(offset_of!(TSModuleBlock, directives) == 24);
    assert!(offset_of!(TSModuleBlock, body) == 48);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeLiteral>() == 48);
    assert!(align_of::<TSTypeLiteral>() == 8);
    assert!(offset_of!(TSTypeLiteral, span) == 0);
    assert!(offset_of!(TSTypeLiteral, members) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSInferType>() == 32);
    assert!(align_of::<TSInferType>() == 8);
    assert!(offset_of!(TSInferType, span) == 0);
    assert!(offset_of!(TSInferType, type_parameter) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeQuery>() == 48);
    assert!(align_of::<TSTypeQuery>() == 8);
    assert!(offset_of!(TSTypeQuery, span) == 0);
    assert!(offset_of!(TSTypeQuery, expr_name) == 24);
    assert!(offset_of!(TSTypeQuery, type_arguments) == 40);

    assert!(size_of::<TSTypeQueryExprName>() == 16);
    assert!(align_of::<TSTypeQueryExprName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSImportType>() == 72);
    assert!(align_of::<TSImportType>() == 8);
    assert!(offset_of!(TSImportType, span) == 0);
    assert!(offset_of!(TSImportType, argument) == 24);
    assert!(offset_of!(TSImportType, options) == 40);
    assert!(offset_of!(TSImportType, qualifier) == 48);
    assert!(offset_of!(TSImportType, type_arguments) == 64);

    // Padding: 4 bytes
    assert!(size_of::<TSFunctionType>() == 64);
    assert!(align_of::<TSFunctionType>() == 8);
    assert!(offset_of!(TSFunctionType, span) == 0);
    assert!(offset_of!(TSFunctionType, type_parameters) == 24);
    assert!(offset_of!(TSFunctionType, this_param) == 32);
    assert!(offset_of!(TSFunctionType, params) == 40);
    assert!(offset_of!(TSFunctionType, return_type) == 48);
    assert!(offset_of!(TSFunctionType, scope_id) == 56);

    // Padding: 7 bytes
    assert!(size_of::<TSConstructorType>() == 56);
    assert!(align_of::<TSConstructorType>() == 8);
    assert!(offset_of!(TSConstructorType, span) == 0);
    assert!(offset_of!(TSConstructorType, r#abstract) == 48);
    assert!(offset_of!(TSConstructorType, type_parameters) == 24);
    assert!(offset_of!(TSConstructorType, params) == 32);
    assert!(offset_of!(TSConstructorType, return_type) == 40);

    // Padding: 2 bytes
    assert!(size_of::<TSMappedType>() == 72);
    assert!(align_of::<TSMappedType>() == 8);
    assert!(offset_of!(TSMappedType, span) == 0);
    assert!(offset_of!(TSMappedType, type_parameter) == 24);
    assert!(offset_of!(TSMappedType, name_type) == 32);
    assert!(offset_of!(TSMappedType, type_annotation) == 48);
    assert!(offset_of!(TSMappedType, optional) == 68);
    assert!(offset_of!(TSMappedType, readonly) == 69);
    assert!(offset_of!(TSMappedType, scope_id) == 64);

    assert!(size_of::<TSMappedTypeModifierOperator>() == 1);
    assert!(align_of::<TSMappedTypeModifierOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSTemplateLiteralType>() == 72);
    assert!(align_of::<TSTemplateLiteralType>() == 8);
    assert!(offset_of!(TSTemplateLiteralType, span) == 0);
    assert!(offset_of!(TSTemplateLiteralType, quasis) == 24);
    assert!(offset_of!(TSTemplateLiteralType, types) == 48);

    // Padding: 0 bytes
    assert!(size_of::<TSAsExpression>() == 56);
    assert!(align_of::<TSAsExpression>() == 8);
    assert!(offset_of!(TSAsExpression, span) == 0);
    assert!(offset_of!(TSAsExpression, expression) == 24);
    assert!(offset_of!(TSAsExpression, type_annotation) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSSatisfiesExpression>() == 56);
    assert!(align_of::<TSSatisfiesExpression>() == 8);
    assert!(offset_of!(TSSatisfiesExpression, span) == 0);
    assert!(offset_of!(TSSatisfiesExpression, expression) == 24);
    assert!(offset_of!(TSSatisfiesExpression, type_annotation) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeAssertion>() == 56);
    assert!(align_of::<TSTypeAssertion>() == 8);
    assert!(offset_of!(TSTypeAssertion, span) == 0);
    assert!(offset_of!(TSTypeAssertion, type_annotation) == 24);
    assert!(offset_of!(TSTypeAssertion, expression) == 40);

    // Padding: 7 bytes
    assert!(size_of::<TSImportEqualsDeclaration>() == 96);
    assert!(align_of::<TSImportEqualsDeclaration>() == 8);
    assert!(offset_of!(TSImportEqualsDeclaration, span) == 0);
    assert!(offset_of!(TSImportEqualsDeclaration, id) == 24);
    assert!(offset_of!(TSImportEqualsDeclaration, module_reference) == 72);
    assert!(offset_of!(TSImportEqualsDeclaration, import_kind) == 88);

    assert!(size_of::<TSModuleReference>() == 16);
    assert!(align_of::<TSModuleReference>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSExternalModuleReference>() == 88);
    assert!(align_of::<TSExternalModuleReference>() == 8);
    assert!(offset_of!(TSExternalModuleReference, span) == 0);
    assert!(offset_of!(TSExternalModuleReference, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSNonNullExpression>() == 40);
    assert!(align_of::<TSNonNullExpression>() == 8);
    assert!(offset_of!(TSNonNullExpression, span) == 0);
    assert!(offset_of!(TSNonNullExpression, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<Decorator>() == 40);
    assert!(align_of::<Decorator>() == 8);
    assert!(offset_of!(Decorator, span) == 0);
    assert!(offset_of!(Decorator, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSExportAssignment>() == 40);
    assert!(align_of::<TSExportAssignment>() == 8);
    assert!(offset_of!(TSExportAssignment, span) == 0);
    assert!(offset_of!(TSExportAssignment, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSNamespaceExportDeclaration>() == 64);
    assert!(align_of::<TSNamespaceExportDeclaration>() == 8);
    assert!(offset_of!(TSNamespaceExportDeclaration, span) == 0);
    assert!(offset_of!(TSNamespaceExportDeclaration, id) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSInstantiationExpression>() == 48);
    assert!(align_of::<TSInstantiationExpression>() == 8);
    assert!(offset_of!(TSInstantiationExpression, span) == 0);
    assert!(offset_of!(TSInstantiationExpression, expression) == 24);
    assert!(offset_of!(TSInstantiationExpression, type_arguments) == 40);

    assert!(size_of::<ImportOrExportKind>() == 1);
    assert!(align_of::<ImportOrExportKind>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<JSDocNullableType>() == 48);
    assert!(align_of::<JSDocNullableType>() == 8);
    assert!(offset_of!(JSDocNullableType, span) == 0);
    assert!(offset_of!(JSDocNullableType, type_annotation) == 24);
    assert!(offset_of!(JSDocNullableType, postfix) == 40);

    // Padding: 7 bytes
    assert!(size_of::<JSDocNonNullableType>() == 48);
    assert!(align_of::<JSDocNonNullableType>() == 8);
    assert!(offset_of!(JSDocNonNullableType, span) == 0);
    assert!(offset_of!(JSDocNonNullableType, type_annotation) == 24);
    assert!(offset_of!(JSDocNonNullableType, postfix) == 40);

    // Padding: 0 bytes
    assert!(size_of::<JSDocUnknownType>() == 24);
    assert!(align_of::<JSDocUnknownType>() == 8);
    assert!(offset_of!(JSDocUnknownType, span) == 0);

    assert!(size_of::<CommentKind>() == 1);
    assert!(align_of::<CommentKind>() == 1);

    assert!(size_of::<CommentPosition>() == 1);
    assert!(align_of::<CommentPosition>() == 1);

    assert!(size_of::<CommentContent>() == 1);
    assert!(align_of::<CommentContent>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<CommentNewlines>() == 1);
    assert!(align_of::<CommentNewlines>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<Comment>() == 32);
    assert!(align_of::<Comment>() == 8);
    assert!(offset_of!(Comment, span) == 0);
    assert!(offset_of!(Comment, attached_to) == 24);
    assert!(offset_of!(Comment, kind) == 28);
    assert!(offset_of!(Comment, position) == 29);
    assert!(offset_of!(Comment, newlines) == 30);
    assert!(offset_of!(Comment, content) == 31);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    // Padding: 1 bytes
    assert!(size_of::<Program>() == 120);
    assert!(align_of::<Program>() == 4);
    assert!(offset_of!(Program, span) == 0);
    assert!(offset_of!(Program, source_type) == 116);
    assert!(offset_of!(Program, source_text) == 24);
    assert!(offset_of!(Program, comments) == 32);
    assert!(offset_of!(Program, hashbang) == 48);
    assert!(offset_of!(Program, directives) == 80);
    assert!(offset_of!(Program, body) == 96);
    assert!(offset_of!(Program, scope_id) == 112);

    assert!(size_of::<Expression>() == 8);
    assert!(align_of::<Expression>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<IdentifierName>() == 32);
    assert!(align_of::<IdentifierName>() == 4);
    assert!(offset_of!(IdentifierName, span) == 0);
    assert!(offset_of!(IdentifierName, name) == 24);

    // Padding: 0 bytes
    assert!(size_of::<IdentifierReference>() == 36);
    assert!(align_of::<IdentifierReference>() == 4);
    assert!(offset_of!(IdentifierReference, span) == 0);
    assert!(offset_of!(IdentifierReference, name) == 24);
    assert!(offset_of!(IdentifierReference, reference_id) == 32);

    // Padding: 0 bytes
    assert!(size_of::<BindingIdentifier>() == 36);
    assert!(align_of::<BindingIdentifier>() == 4);
    assert!(offset_of!(BindingIdentifier, span) == 0);
    assert!(offset_of!(BindingIdentifier, name) == 24);
    assert!(offset_of!(BindingIdentifier, symbol_id) == 32);

    // Padding: 0 bytes
    assert!(size_of::<LabelIdentifier>() == 32);
    assert!(align_of::<LabelIdentifier>() == 4);
    assert!(offset_of!(LabelIdentifier, span) == 0);
    assert!(offset_of!(LabelIdentifier, name) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ThisExpression>() == 24);
    assert!(align_of::<ThisExpression>() == 4);
    assert!(offset_of!(ThisExpression, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ArrayExpression>() == 40);
    assert!(align_of::<ArrayExpression>() == 4);
    assert!(offset_of!(ArrayExpression, span) == 0);
    assert!(offset_of!(ArrayExpression, elements) == 24);

    assert!(size_of::<ArrayExpressionElement>() == 28);
    assert!(align_of::<ArrayExpressionElement>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<Elision>() == 24);
    assert!(align_of::<Elision>() == 4);
    assert!(offset_of!(Elision, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ObjectExpression>() == 40);
    assert!(align_of::<ObjectExpression>() == 4);
    assert!(offset_of!(ObjectExpression, span) == 0);
    assert!(offset_of!(ObjectExpression, properties) == 24);

    assert!(size_of::<ObjectPropertyKind>() == 8);
    assert!(align_of::<ObjectPropertyKind>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ObjectProperty>() == 44);
    assert!(align_of::<ObjectProperty>() == 4);
    assert!(offset_of!(ObjectProperty, span) == 0);
    assert!(offset_of!(ObjectProperty, kind) == 40);
    assert!(offset_of!(ObjectProperty, key) == 24);
    assert!(offset_of!(ObjectProperty, value) == 32);
    assert!(offset_of!(ObjectProperty, method) == 41);
    assert!(offset_of!(ObjectProperty, shorthand) == 42);
    assert!(offset_of!(ObjectProperty, computed) == 43);

    assert!(size_of::<PropertyKey>() == 8);
    assert!(align_of::<PropertyKey>() == 4);

    assert!(size_of::<PropertyKind>() == 1);
    assert!(align_of::<PropertyKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TemplateLiteral>() == 56);
    assert!(align_of::<TemplateLiteral>() == 4);
    assert!(offset_of!(TemplateLiteral, span) == 0);
    assert!(offset_of!(TemplateLiteral, quasis) == 24);
    assert!(offset_of!(TemplateLiteral, expressions) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TaggedTemplateExpression>() == 92);
    assert!(align_of::<TaggedTemplateExpression>() == 4);
    assert!(offset_of!(TaggedTemplateExpression, span) == 0);
    assert!(offset_of!(TaggedTemplateExpression, tag) == 24);
    assert!(offset_of!(TaggedTemplateExpression, type_arguments) == 32);
    assert!(offset_of!(TaggedTemplateExpression, quasi) == 36);

    // Padding: 2 bytes
    assert!(size_of::<TemplateElement>() == 44);
    assert!(align_of::<TemplateElement>() == 4);
    assert!(offset_of!(TemplateElement, span) == 0);
    assert!(offset_of!(TemplateElement, value) == 24);
    assert!(offset_of!(TemplateElement, tail) == 40);
    assert!(offset_of!(TemplateElement, lone_surrogates) == 41);

    // Padding: 0 bytes
    assert!(size_of::<TemplateElementValue>() == 16);
    assert!(align_of::<TemplateElementValue>() == 4);
    assert!(offset_of!(TemplateElementValue, raw) == 0);
    assert!(offset_of!(TemplateElementValue, cooked) == 8);

    assert!(size_of::<MemberExpression>() == 8);
    assert!(align_of::<MemberExpression>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ComputedMemberExpression>() == 44);
    assert!(align_of::<ComputedMemberExpression>() == 4);
    assert!(offset_of!(ComputedMemberExpression, span) == 0);
    assert!(offset_of!(ComputedMemberExpression, object) == 24);
    assert!(offset_of!(ComputedMemberExpression, expression) == 32);
    assert!(offset_of!(ComputedMemberExpression, optional) == 40);

    // Padding: 3 bytes
    assert!(size_of::<StaticMemberExpression>() == 68);
    assert!(align_of::<StaticMemberExpression>() == 4);
    assert!(offset_of!(StaticMemberExpression, span) == 0);
    assert!(offset_of!(StaticMemberExpression, object) == 24);
    assert!(offset_of!(StaticMemberExpression, property) == 32);
    assert!(offset_of!(StaticMemberExpression, optional) == 64);

    // Padding: 3 bytes
    assert!(size_of::<PrivateFieldExpression>() == 68);
    assert!(align_of::<PrivateFieldExpression>() == 4);
    assert!(offset_of!(PrivateFieldExpression, span) == 0);
    assert!(offset_of!(PrivateFieldExpression, object) == 24);
    assert!(offset_of!(PrivateFieldExpression, field) == 32);
    assert!(offset_of!(PrivateFieldExpression, optional) == 64);

    // Padding: 2 bytes
    assert!(size_of::<CallExpression>() == 56);
    assert!(align_of::<CallExpression>() == 4);
    assert!(offset_of!(CallExpression, span) == 0);
    assert!(offset_of!(CallExpression, callee) == 24);
    assert!(offset_of!(CallExpression, type_arguments) == 32);
    assert!(offset_of!(CallExpression, arguments) == 36);
    assert!(offset_of!(CallExpression, optional) == 52);
    assert!(offset_of!(CallExpression, pure) == 53);

    // Padding: 3 bytes
    assert!(size_of::<NewExpression>() == 56);
    assert!(align_of::<NewExpression>() == 4);
    assert!(offset_of!(NewExpression, span) == 0);
    assert!(offset_of!(NewExpression, callee) == 24);
    assert!(offset_of!(NewExpression, type_arguments) == 32);
    assert!(offset_of!(NewExpression, arguments) == 36);
    assert!(offset_of!(NewExpression, pure) == 52);

    // Padding: 0 bytes
    assert!(size_of::<MetaProperty>() == 88);
    assert!(align_of::<MetaProperty>() == 4);
    assert!(offset_of!(MetaProperty, span) == 0);
    assert!(offset_of!(MetaProperty, meta) == 24);
    assert!(offset_of!(MetaProperty, property) == 56);

    // Padding: 0 bytes
    assert!(size_of::<SpreadElement>() == 32);
    assert!(align_of::<SpreadElement>() == 4);
    assert!(offset_of!(SpreadElement, span) == 0);
    assert!(offset_of!(SpreadElement, argument) == 24);

    assert!(size_of::<Argument>() == 8);
    assert!(align_of::<Argument>() == 4);

    // Padding: 2 bytes
    assert!(size_of::<UpdateExpression>() == 36);
    assert!(align_of::<UpdateExpression>() == 4);
    assert!(offset_of!(UpdateExpression, span) == 0);
    assert!(offset_of!(UpdateExpression, operator) == 32);
    assert!(offset_of!(UpdateExpression, prefix) == 33);
    assert!(offset_of!(UpdateExpression, argument) == 24);

    // Padding: 3 bytes
    assert!(size_of::<UnaryExpression>() == 36);
    assert!(align_of::<UnaryExpression>() == 4);
    assert!(offset_of!(UnaryExpression, span) == 0);
    assert!(offset_of!(UnaryExpression, operator) == 32);
    assert!(offset_of!(UnaryExpression, argument) == 24);

    // Padding: 3 bytes
    assert!(size_of::<BinaryExpression>() == 44);
    assert!(align_of::<BinaryExpression>() == 4);
    assert!(offset_of!(BinaryExpression, span) == 0);
    assert!(offset_of!(BinaryExpression, left) == 24);
    assert!(offset_of!(BinaryExpression, operator) == 40);
    assert!(offset_of!(BinaryExpression, right) == 32);

    // Padding: 0 bytes
    assert!(size_of::<PrivateInExpression>() == 64);
    assert!(align_of::<PrivateInExpression>() == 4);
    assert!(offset_of!(PrivateInExpression, span) == 0);
    assert!(offset_of!(PrivateInExpression, left) == 24);
    assert!(offset_of!(PrivateInExpression, right) == 56);

    // Padding: 3 bytes
    assert!(size_of::<LogicalExpression>() == 44);
    assert!(align_of::<LogicalExpression>() == 4);
    assert!(offset_of!(LogicalExpression, span) == 0);
    assert!(offset_of!(LogicalExpression, left) == 24);
    assert!(offset_of!(LogicalExpression, operator) == 40);
    assert!(offset_of!(LogicalExpression, right) == 32);

    // Padding: 0 bytes
    assert!(size_of::<ConditionalExpression>() == 48);
    assert!(align_of::<ConditionalExpression>() == 4);
    assert!(offset_of!(ConditionalExpression, span) == 0);
    assert!(offset_of!(ConditionalExpression, test) == 24);
    assert!(offset_of!(ConditionalExpression, consequent) == 32);
    assert!(offset_of!(ConditionalExpression, alternate) == 40);

    // Padding: 3 bytes
    assert!(size_of::<AssignmentExpression>() == 44);
    assert!(align_of::<AssignmentExpression>() == 4);
    assert!(offset_of!(AssignmentExpression, span) == 0);
    assert!(offset_of!(AssignmentExpression, operator) == 40);
    assert!(offset_of!(AssignmentExpression, left) == 24);
    assert!(offset_of!(AssignmentExpression, right) == 32);

    assert!(size_of::<AssignmentTarget>() == 8);
    assert!(align_of::<AssignmentTarget>() == 4);

    assert!(size_of::<SimpleAssignmentTarget>() == 8);
    assert!(align_of::<SimpleAssignmentTarget>() == 4);

    assert!(size_of::<AssignmentTargetPattern>() == 8);
    assert!(align_of::<AssignmentTargetPattern>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ArrayAssignmentTarget>() == 72);
    assert!(align_of::<ArrayAssignmentTarget>() == 4);
    assert!(offset_of!(ArrayAssignmentTarget, span) == 0);
    assert!(offset_of!(ArrayAssignmentTarget, elements) == 24);
    assert!(offset_of!(ArrayAssignmentTarget, rest) == 40);

    // Padding: 0 bytes
    assert!(size_of::<ObjectAssignmentTarget>() == 72);
    assert!(align_of::<ObjectAssignmentTarget>() == 4);
    assert!(offset_of!(ObjectAssignmentTarget, span) == 0);
    assert!(offset_of!(ObjectAssignmentTarget, properties) == 24);
    assert!(offset_of!(ObjectAssignmentTarget, rest) == 40);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetRest>() == 32);
    assert!(align_of::<AssignmentTargetRest>() == 4);
    assert!(offset_of!(AssignmentTargetRest, span) == 0);
    assert!(offset_of!(AssignmentTargetRest, target) == 24);

    assert!(size_of::<AssignmentTargetMaybeDefault>() == 8);
    assert!(align_of::<AssignmentTargetMaybeDefault>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetWithDefault>() == 40);
    assert!(align_of::<AssignmentTargetWithDefault>() == 4);
    assert!(offset_of!(AssignmentTargetWithDefault, span) == 0);
    assert!(offset_of!(AssignmentTargetWithDefault, binding) == 24);
    assert!(offset_of!(AssignmentTargetWithDefault, init) == 32);

    assert!(size_of::<AssignmentTargetProperty>() == 8);
    assert!(align_of::<AssignmentTargetProperty>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetPropertyIdentifier>() == 68);
    assert!(align_of::<AssignmentTargetPropertyIdentifier>() == 4);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, span) == 0);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, binding) == 24);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, init) == 60);

    // Padding: 3 bytes
    assert!(size_of::<AssignmentTargetPropertyProperty>() == 44);
    assert!(align_of::<AssignmentTargetPropertyProperty>() == 4);
    assert!(offset_of!(AssignmentTargetPropertyProperty, span) == 0);
    assert!(offset_of!(AssignmentTargetPropertyProperty, name) == 24);
    assert!(offset_of!(AssignmentTargetPropertyProperty, binding) == 32);
    assert!(offset_of!(AssignmentTargetPropertyProperty, computed) == 40);

    // Padding: 0 bytes
    assert!(size_of::<SequenceExpression>() == 40);
    assert!(align_of::<SequenceExpression>() == 4);
    assert!(offset_of!(SequenceExpression, span) == 0);
    assert!(offset_of!(SequenceExpression, expressions) == 24);

    // Padding: 0 bytes
    assert!(size_of::<Super>() == 24);
    assert!(align_of::<Super>() == 4);
    assert!(offset_of!(Super, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<AwaitExpression>() == 32);
    assert!(align_of::<AwaitExpression>() == 4);
    assert!(offset_of!(AwaitExpression, span) == 0);
    assert!(offset_of!(AwaitExpression, argument) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ChainExpression>() == 32);
    assert!(align_of::<ChainExpression>() == 4);
    assert!(offset_of!(ChainExpression, span) == 0);
    assert!(offset_of!(ChainExpression, expression) == 24);

    assert!(size_of::<ChainElement>() == 8);
    assert!(align_of::<ChainElement>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ParenthesizedExpression>() == 32);
    assert!(align_of::<ParenthesizedExpression>() == 4);
    assert!(offset_of!(ParenthesizedExpression, span) == 0);
    assert!(offset_of!(ParenthesizedExpression, expression) == 24);

    assert!(size_of::<Statement>() == 8);
    assert!(align_of::<Statement>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<Directive>() == 76);
    assert!(align_of::<Directive>() == 4);
    assert!(offset_of!(Directive, span) == 0);
    assert!(offset_of!(Directive, expression) == 24);
    assert!(offset_of!(Directive, directive) == 68);

    // Padding: 0 bytes
    assert!(size_of::<Hashbang>() == 32);
    assert!(align_of::<Hashbang>() == 4);
    assert!(offset_of!(Hashbang, span) == 0);
    assert!(offset_of!(Hashbang, value) == 24);

    // Padding: 0 bytes
    assert!(size_of::<BlockStatement>() == 44);
    assert!(align_of::<BlockStatement>() == 4);
    assert!(offset_of!(BlockStatement, span) == 0);
    assert!(offset_of!(BlockStatement, body) == 24);
    assert!(offset_of!(BlockStatement, scope_id) == 40);

    assert!(size_of::<Declaration>() == 8);
    assert!(align_of::<Declaration>() == 4);

    // Padding: 2 bytes
    assert!(size_of::<VariableDeclaration>() == 44);
    assert!(align_of::<VariableDeclaration>() == 4);
    assert!(offset_of!(VariableDeclaration, span) == 0);
    assert!(offset_of!(VariableDeclaration, kind) == 40);
    assert!(offset_of!(VariableDeclaration, declarations) == 24);
    assert!(offset_of!(VariableDeclaration, declare) == 41);

    assert!(size_of::<VariableDeclarationKind>() == 1);
    assert!(align_of::<VariableDeclarationKind>() == 1);

    // Padding: 2 bytes
    assert!(size_of::<VariableDeclarator>() == 52);
    assert!(align_of::<VariableDeclarator>() == 4);
    assert!(offset_of!(VariableDeclarator, span) == 0);
    assert!(offset_of!(VariableDeclarator, kind) == 48);
    assert!(offset_of!(VariableDeclarator, id) == 24);
    assert!(offset_of!(VariableDeclarator, init) == 40);
    assert!(offset_of!(VariableDeclarator, definite) == 49);

    // Padding: 0 bytes
    assert!(size_of::<EmptyStatement>() == 24);
    assert!(align_of::<EmptyStatement>() == 4);
    assert!(offset_of!(EmptyStatement, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ExpressionStatement>() == 32);
    assert!(align_of::<ExpressionStatement>() == 4);
    assert!(offset_of!(ExpressionStatement, span) == 0);
    assert!(offset_of!(ExpressionStatement, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<IfStatement>() == 48);
    assert!(align_of::<IfStatement>() == 4);
    assert!(offset_of!(IfStatement, span) == 0);
    assert!(offset_of!(IfStatement, test) == 24);
    assert!(offset_of!(IfStatement, consequent) == 32);
    assert!(offset_of!(IfStatement, alternate) == 40);

    // Padding: 0 bytes
    assert!(size_of::<DoWhileStatement>() == 40);
    assert!(align_of::<DoWhileStatement>() == 4);
    assert!(offset_of!(DoWhileStatement, span) == 0);
    assert!(offset_of!(DoWhileStatement, body) == 24);
    assert!(offset_of!(DoWhileStatement, test) == 32);

    // Padding: 0 bytes
    assert!(size_of::<WhileStatement>() == 40);
    assert!(align_of::<WhileStatement>() == 4);
    assert!(offset_of!(WhileStatement, span) == 0);
    assert!(offset_of!(WhileStatement, test) == 24);
    assert!(offset_of!(WhileStatement, body) == 32);

    // Padding: 0 bytes
    assert!(size_of::<ForStatement>() == 60);
    assert!(align_of::<ForStatement>() == 4);
    assert!(offset_of!(ForStatement, span) == 0);
    assert!(offset_of!(ForStatement, init) == 24);
    assert!(offset_of!(ForStatement, test) == 32);
    assert!(offset_of!(ForStatement, update) == 40);
    assert!(offset_of!(ForStatement, body) == 48);
    assert!(offset_of!(ForStatement, scope_id) == 56);

    assert!(size_of::<ForStatementInit>() == 8);
    assert!(align_of::<ForStatementInit>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ForInStatement>() == 52);
    assert!(align_of::<ForInStatement>() == 4);
    assert!(offset_of!(ForInStatement, span) == 0);
    assert!(offset_of!(ForInStatement, left) == 24);
    assert!(offset_of!(ForInStatement, right) == 32);
    assert!(offset_of!(ForInStatement, body) == 40);
    assert!(offset_of!(ForInStatement, scope_id) == 48);

    assert!(size_of::<ForStatementLeft>() == 8);
    assert!(align_of::<ForStatementLeft>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ForOfStatement>() == 56);
    assert!(align_of::<ForOfStatement>() == 4);
    assert!(offset_of!(ForOfStatement, span) == 0);
    assert!(offset_of!(ForOfStatement, r#await) == 52);
    assert!(offset_of!(ForOfStatement, left) == 24);
    assert!(offset_of!(ForOfStatement, right) == 32);
    assert!(offset_of!(ForOfStatement, body) == 40);
    assert!(offset_of!(ForOfStatement, scope_id) == 48);

    // Padding: 0 bytes
    assert!(size_of::<ContinueStatement>() == 56);
    assert!(align_of::<ContinueStatement>() == 4);
    assert!(offset_of!(ContinueStatement, span) == 0);
    assert!(offset_of!(ContinueStatement, label) == 24);

    // Padding: 0 bytes
    assert!(size_of::<BreakStatement>() == 56);
    assert!(align_of::<BreakStatement>() == 4);
    assert!(offset_of!(BreakStatement, span) == 0);
    assert!(offset_of!(BreakStatement, label) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ReturnStatement>() == 32);
    assert!(align_of::<ReturnStatement>() == 4);
    assert!(offset_of!(ReturnStatement, span) == 0);
    assert!(offset_of!(ReturnStatement, argument) == 24);

    // Padding: 0 bytes
    assert!(size_of::<WithStatement>() == 40);
    assert!(align_of::<WithStatement>() == 4);
    assert!(offset_of!(WithStatement, span) == 0);
    assert!(offset_of!(WithStatement, object) == 24);
    assert!(offset_of!(WithStatement, body) == 32);

    // Padding: 0 bytes
    assert!(size_of::<SwitchStatement>() == 52);
    assert!(align_of::<SwitchStatement>() == 4);
    assert!(offset_of!(SwitchStatement, span) == 0);
    assert!(offset_of!(SwitchStatement, discriminant) == 24);
    assert!(offset_of!(SwitchStatement, cases) == 32);
    assert!(offset_of!(SwitchStatement, scope_id) == 48);

    // Padding: 0 bytes
    assert!(size_of::<SwitchCase>() == 48);
    assert!(align_of::<SwitchCase>() == 4);
    assert!(offset_of!(SwitchCase, span) == 0);
    assert!(offset_of!(SwitchCase, test) == 24);
    assert!(offset_of!(SwitchCase, consequent) == 32);

    // Padding: 0 bytes
    assert!(size_of::<LabeledStatement>() == 64);
    assert!(align_of::<LabeledStatement>() == 4);
    assert!(offset_of!(LabeledStatement, span) == 0);
    assert!(offset_of!(LabeledStatement, label) == 24);
    assert!(offset_of!(LabeledStatement, body) == 56);

    // Padding: 0 bytes
    assert!(size_of::<ThrowStatement>() == 32);
    assert!(align_of::<ThrowStatement>() == 4);
    assert!(offset_of!(ThrowStatement, span) == 0);
    assert!(offset_of!(ThrowStatement, argument) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TryStatement>() == 36);
    assert!(align_of::<TryStatement>() == 4);
    assert!(offset_of!(TryStatement, span) == 0);
    assert!(offset_of!(TryStatement, block) == 24);
    assert!(offset_of!(TryStatement, handler) == 28);
    assert!(offset_of!(TryStatement, finalizer) == 32);

    // Padding: 0 bytes
    assert!(size_of::<CatchClause>() == 72);
    assert!(align_of::<CatchClause>() == 4);
    assert!(offset_of!(CatchClause, span) == 0);
    assert!(offset_of!(CatchClause, param) == 24);
    assert!(offset_of!(CatchClause, body) == 64);
    assert!(offset_of!(CatchClause, scope_id) == 68);

    // Padding: 0 bytes
    assert!(size_of::<CatchParameter>() == 40);
    assert!(align_of::<CatchParameter>() == 4);
    assert!(offset_of!(CatchParameter, span) == 0);
    assert!(offset_of!(CatchParameter, pattern) == 24);

    // Padding: 0 bytes
    assert!(size_of::<DebuggerStatement>() == 24);
    assert!(align_of::<DebuggerStatement>() == 4);
    assert!(offset_of!(DebuggerStatement, span) == 0);

    // Padding: 3 bytes
    assert!(size_of::<BindingPattern>() == 16);
    assert!(align_of::<BindingPattern>() == 4);
    assert!(offset_of!(BindingPattern, kind) == 0);
    assert!(offset_of!(BindingPattern, type_annotation) == 8);
    assert!(offset_of!(BindingPattern, optional) == 12);

    assert!(size_of::<BindingPatternKind>() == 8);
    assert!(align_of::<BindingPatternKind>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentPattern>() == 48);
    assert!(align_of::<AssignmentPattern>() == 4);
    assert!(offset_of!(AssignmentPattern, span) == 0);
    assert!(offset_of!(AssignmentPattern, left) == 24);
    assert!(offset_of!(AssignmentPattern, right) == 40);

    // Padding: 0 bytes
    assert!(size_of::<ObjectPattern>() == 44);
    assert!(align_of::<ObjectPattern>() == 4);
    assert!(offset_of!(ObjectPattern, span) == 0);
    assert!(offset_of!(ObjectPattern, properties) == 24);
    assert!(offset_of!(ObjectPattern, rest) == 40);

    // Padding: 2 bytes
    assert!(size_of::<BindingProperty>() == 52);
    assert!(align_of::<BindingProperty>() == 4);
    assert!(offset_of!(BindingProperty, span) == 0);
    assert!(offset_of!(BindingProperty, key) == 24);
    assert!(offset_of!(BindingProperty, value) == 32);
    assert!(offset_of!(BindingProperty, shorthand) == 48);
    assert!(offset_of!(BindingProperty, computed) == 49);

    // Padding: 0 bytes
    assert!(size_of::<ArrayPattern>() == 44);
    assert!(align_of::<ArrayPattern>() == 4);
    assert!(offset_of!(ArrayPattern, span) == 0);
    assert!(offset_of!(ArrayPattern, elements) == 24);
    assert!(offset_of!(ArrayPattern, rest) == 40);

    // Padding: 0 bytes
    assert!(size_of::<BindingRestElement>() == 40);
    assert!(align_of::<BindingRestElement>() == 4);
    assert!(offset_of!(BindingRestElement, span) == 0);
    assert!(offset_of!(BindingRestElement, argument) == 24);

    // Padding: 3 bytes
    assert!(size_of::<Function>() == 92);
    assert!(align_of::<Function>() == 4);
    assert!(offset_of!(Function, span) == 0);
    assert!(offset_of!(Function, r#type) == 84);
    assert!(offset_of!(Function, id) == 24);
    assert!(offset_of!(Function, generator) == 85);
    assert!(offset_of!(Function, r#async) == 86);
    assert!(offset_of!(Function, declare) == 87);
    assert!(offset_of!(Function, type_parameters) == 60);
    assert!(offset_of!(Function, this_param) == 64);
    assert!(offset_of!(Function, params) == 68);
    assert!(offset_of!(Function, return_type) == 72);
    assert!(offset_of!(Function, body) == 76);
    assert!(offset_of!(Function, scope_id) == 80);
    assert!(offset_of!(Function, pure) == 88);

    assert!(size_of::<FunctionType>() == 1);
    assert!(align_of::<FunctionType>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<FormalParameters>() == 48);
    assert!(align_of::<FormalParameters>() == 4);
    assert!(offset_of!(FormalParameters, span) == 0);
    assert!(offset_of!(FormalParameters, kind) == 44);
    assert!(offset_of!(FormalParameters, items) == 24);
    assert!(offset_of!(FormalParameters, rest) == 40);

    // Padding: 1 bytes
    assert!(size_of::<FormalParameter>() == 60);
    assert!(align_of::<FormalParameter>() == 4);
    assert!(offset_of!(FormalParameter, span) == 0);
    assert!(offset_of!(FormalParameter, decorators) == 24);
    assert!(offset_of!(FormalParameter, pattern) == 40);
    assert!(offset_of!(FormalParameter, accessibility) == 56);
    assert!(offset_of!(FormalParameter, readonly) == 57);
    assert!(offset_of!(FormalParameter, r#override) == 58);

    assert!(size_of::<FormalParameterKind>() == 1);
    assert!(align_of::<FormalParameterKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<FunctionBody>() == 56);
    assert!(align_of::<FunctionBody>() == 4);
    assert!(offset_of!(FunctionBody, span) == 0);
    assert!(offset_of!(FunctionBody, directives) == 24);
    assert!(offset_of!(FunctionBody, statements) == 40);

    // Padding: 1 bytes
    assert!(size_of::<ArrowFunctionExpression>() == 48);
    assert!(align_of::<ArrowFunctionExpression>() == 4);
    assert!(offset_of!(ArrowFunctionExpression, span) == 0);
    assert!(offset_of!(ArrowFunctionExpression, expression) == 44);
    assert!(offset_of!(ArrowFunctionExpression, r#async) == 45);
    assert!(offset_of!(ArrowFunctionExpression, type_parameters) == 24);
    assert!(offset_of!(ArrowFunctionExpression, params) == 28);
    assert!(offset_of!(ArrowFunctionExpression, return_type) == 32);
    assert!(offset_of!(ArrowFunctionExpression, body) == 36);
    assert!(offset_of!(ArrowFunctionExpression, scope_id) == 40);
    assert!(offset_of!(ArrowFunctionExpression, pure) == 46);

    // Padding: 3 bytes
    assert!(size_of::<YieldExpression>() == 36);
    assert!(align_of::<YieldExpression>() == 4);
    assert!(offset_of!(YieldExpression, span) == 0);
    assert!(offset_of!(YieldExpression, delegate) == 32);
    assert!(offset_of!(YieldExpression, argument) == 24);

    // Padding: 1 bytes
    assert!(size_of::<Class>() == 120);
    assert!(align_of::<Class>() == 4);
    assert!(offset_of!(Class, span) == 0);
    assert!(offset_of!(Class, r#type) == 116);
    assert!(offset_of!(Class, decorators) == 24);
    assert!(offset_of!(Class, id) == 40);
    assert!(offset_of!(Class, type_parameters) == 76);
    assert!(offset_of!(Class, super_class) == 80);
    assert!(offset_of!(Class, super_type_arguments) == 88);
    assert!(offset_of!(Class, implements) == 92);
    assert!(offset_of!(Class, body) == 108);
    assert!(offset_of!(Class, r#abstract) == 117);
    assert!(offset_of!(Class, declare) == 118);
    assert!(offset_of!(Class, scope_id) == 112);

    assert!(size_of::<ClassType>() == 1);
    assert!(align_of::<ClassType>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ClassBody>() == 40);
    assert!(align_of::<ClassBody>() == 4);
    assert!(offset_of!(ClassBody, span) == 0);
    assert!(offset_of!(ClassBody, body) == 24);

    assert!(size_of::<ClassElement>() == 8);
    assert!(align_of::<ClassElement>() == 4);

    // Padding: 1 bytes
    assert!(size_of::<MethodDefinition>() == 60);
    assert!(align_of::<MethodDefinition>() == 4);
    assert!(offset_of!(MethodDefinition, span) == 0);
    assert!(offset_of!(MethodDefinition, r#type) == 52);
    assert!(offset_of!(MethodDefinition, decorators) == 24);
    assert!(offset_of!(MethodDefinition, key) == 40);
    assert!(offset_of!(MethodDefinition, value) == 48);
    assert!(offset_of!(MethodDefinition, kind) == 53);
    assert!(offset_of!(MethodDefinition, computed) == 54);
    assert!(offset_of!(MethodDefinition, r#static) == 55);
    assert!(offset_of!(MethodDefinition, r#override) == 56);
    assert!(offset_of!(MethodDefinition, optional) == 57);
    assert!(offset_of!(MethodDefinition, accessibility) == 58);

    assert!(size_of::<MethodDefinitionType>() == 1);
    assert!(align_of::<MethodDefinitionType>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<PropertyDefinition>() == 72);
    assert!(align_of::<PropertyDefinition>() == 4);
    assert!(offset_of!(PropertyDefinition, span) == 0);
    assert!(offset_of!(PropertyDefinition, r#type) == 60);
    assert!(offset_of!(PropertyDefinition, decorators) == 24);
    assert!(offset_of!(PropertyDefinition, key) == 40);
    assert!(offset_of!(PropertyDefinition, type_annotation) == 48);
    assert!(offset_of!(PropertyDefinition, value) == 52);
    assert!(offset_of!(PropertyDefinition, computed) == 61);
    assert!(offset_of!(PropertyDefinition, r#static) == 62);
    assert!(offset_of!(PropertyDefinition, declare) == 63);
    assert!(offset_of!(PropertyDefinition, r#override) == 64);
    assert!(offset_of!(PropertyDefinition, optional) == 65);
    assert!(offset_of!(PropertyDefinition, definite) == 66);
    assert!(offset_of!(PropertyDefinition, readonly) == 67);
    assert!(offset_of!(PropertyDefinition, accessibility) == 68);

    assert!(size_of::<PropertyDefinitionType>() == 1);
    assert!(align_of::<PropertyDefinitionType>() == 1);

    assert!(size_of::<MethodDefinitionKind>() == 1);
    assert!(align_of::<MethodDefinitionKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<PrivateIdentifier>() == 32);
    assert!(align_of::<PrivateIdentifier>() == 4);
    assert!(offset_of!(PrivateIdentifier, span) == 0);
    assert!(offset_of!(PrivateIdentifier, name) == 24);

    // Padding: 0 bytes
    assert!(size_of::<StaticBlock>() == 44);
    assert!(align_of::<StaticBlock>() == 4);
    assert!(offset_of!(StaticBlock, span) == 0);
    assert!(offset_of!(StaticBlock, body) == 24);
    assert!(offset_of!(StaticBlock, scope_id) == 40);

    assert!(size_of::<ModuleDeclaration>() == 8);
    assert!(align_of::<ModuleDeclaration>() == 4);

    assert!(size_of::<AccessorPropertyType>() == 1);
    assert!(align_of::<AccessorPropertyType>() == 1);

    // Padding: 2 bytes
    assert!(size_of::<AccessorProperty>() == 68);
    assert!(align_of::<AccessorProperty>() == 4);
    assert!(offset_of!(AccessorProperty, span) == 0);
    assert!(offset_of!(AccessorProperty, r#type) == 60);
    assert!(offset_of!(AccessorProperty, decorators) == 24);
    assert!(offset_of!(AccessorProperty, key) == 40);
    assert!(offset_of!(AccessorProperty, type_annotation) == 48);
    assert!(offset_of!(AccessorProperty, value) == 52);
    assert!(offset_of!(AccessorProperty, computed) == 61);
    assert!(offset_of!(AccessorProperty, r#static) == 62);
    assert!(offset_of!(AccessorProperty, r#override) == 63);
    assert!(offset_of!(AccessorProperty, definite) == 64);
    assert!(offset_of!(AccessorProperty, accessibility) == 65);

    // Padding: 3 bytes
    assert!(size_of::<ImportExpression>() == 44);
    assert!(align_of::<ImportExpression>() == 4);
    assert!(offset_of!(ImportExpression, span) == 0);
    assert!(offset_of!(ImportExpression, source) == 24);
    assert!(offset_of!(ImportExpression, options) == 32);
    assert!(offset_of!(ImportExpression, phase) == 40);

    // Padding: 2 bytes
    assert!(size_of::<ImportDeclaration>() == 92);
    assert!(align_of::<ImportDeclaration>() == 4);
    assert!(offset_of!(ImportDeclaration, span) == 0);
    assert!(offset_of!(ImportDeclaration, specifiers) == 24);
    assert!(offset_of!(ImportDeclaration, source) == 40);
    assert!(offset_of!(ImportDeclaration, phase) == 88);
    assert!(offset_of!(ImportDeclaration, with_clause) == 84);
    assert!(offset_of!(ImportDeclaration, import_kind) == 89);

    assert!(size_of::<ImportPhase>() == 1);
    assert!(align_of::<ImportPhase>() == 1);

    assert!(size_of::<ImportDeclarationSpecifier>() == 8);
    assert!(align_of::<ImportDeclarationSpecifier>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ImportSpecifier>() == 112);
    assert!(align_of::<ImportSpecifier>() == 4);
    assert!(offset_of!(ImportSpecifier, span) == 0);
    assert!(offset_of!(ImportSpecifier, imported) == 24);
    assert!(offset_of!(ImportSpecifier, local) == 72);
    assert!(offset_of!(ImportSpecifier, import_kind) == 108);

    // Padding: 0 bytes
    assert!(size_of::<ImportDefaultSpecifier>() == 60);
    assert!(align_of::<ImportDefaultSpecifier>() == 4);
    assert!(offset_of!(ImportDefaultSpecifier, span) == 0);
    assert!(offset_of!(ImportDefaultSpecifier, local) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ImportNamespaceSpecifier>() == 60);
    assert!(align_of::<ImportNamespaceSpecifier>() == 4);
    assert!(offset_of!(ImportNamespaceSpecifier, span) == 0);
    assert!(offset_of!(ImportNamespaceSpecifier, local) == 24);

    // Padding: 0 bytes
    assert!(size_of::<WithClause>() == 72);
    assert!(align_of::<WithClause>() == 4);
    assert!(offset_of!(WithClause, span) == 0);
    assert!(offset_of!(WithClause, attributes_keyword) == 24);
    assert!(offset_of!(WithClause, with_entries) == 56);

    // Padding: 0 bytes
    assert!(size_of::<ImportAttribute>() == 116);
    assert!(align_of::<ImportAttribute>() == 4);
    assert!(offset_of!(ImportAttribute, span) == 0);
    assert!(offset_of!(ImportAttribute, key) == 24);
    assert!(offset_of!(ImportAttribute, value) == 72);

    assert!(size_of::<ImportAttributeKey>() == 48);
    assert!(align_of::<ImportAttributeKey>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ExportNamedDeclaration>() == 100);
    assert!(align_of::<ExportNamedDeclaration>() == 4);
    assert!(offset_of!(ExportNamedDeclaration, span) == 0);
    assert!(offset_of!(ExportNamedDeclaration, declaration) == 24);
    assert!(offset_of!(ExportNamedDeclaration, specifiers) == 32);
    assert!(offset_of!(ExportNamedDeclaration, source) == 48);
    assert!(offset_of!(ExportNamedDeclaration, export_kind) == 96);
    assert!(offset_of!(ExportNamedDeclaration, with_clause) == 92);

    // Padding: 0 bytes
    assert!(size_of::<ExportDefaultDeclaration>() == 80);
    assert!(align_of::<ExportDefaultDeclaration>() == 4);
    assert!(offset_of!(ExportDefaultDeclaration, span) == 0);
    assert!(offset_of!(ExportDefaultDeclaration, exported) == 24);
    assert!(offset_of!(ExportDefaultDeclaration, declaration) == 72);

    // Padding: 3 bytes
    assert!(size_of::<ExportAllDeclaration>() == 124);
    assert!(align_of::<ExportAllDeclaration>() == 4);
    assert!(offset_of!(ExportAllDeclaration, span) == 0);
    assert!(offset_of!(ExportAllDeclaration, exported) == 24);
    assert!(offset_of!(ExportAllDeclaration, source) == 72);
    assert!(offset_of!(ExportAllDeclaration, with_clause) == 116);
    assert!(offset_of!(ExportAllDeclaration, export_kind) == 120);

    // Padding: 3 bytes
    assert!(size_of::<ExportSpecifier>() == 124);
    assert!(align_of::<ExportSpecifier>() == 4);
    assert!(offset_of!(ExportSpecifier, span) == 0);
    assert!(offset_of!(ExportSpecifier, local) == 24);
    assert!(offset_of!(ExportSpecifier, exported) == 72);
    assert!(offset_of!(ExportSpecifier, export_kind) == 120);

    assert!(size_of::<ExportDefaultDeclarationKind>() == 8);
    assert!(align_of::<ExportDefaultDeclarationKind>() == 4);

    assert!(size_of::<ModuleExportName>() == 48);
    assert!(align_of::<ModuleExportName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<V8IntrinsicExpression>() == 72);
    assert!(align_of::<V8IntrinsicExpression>() == 4);
    assert!(offset_of!(V8IntrinsicExpression, span) == 0);
    assert!(offset_of!(V8IntrinsicExpression, name) == 24);
    assert!(offset_of!(V8IntrinsicExpression, arguments) == 56);

    // Padding: 3 bytes
    assert!(size_of::<BooleanLiteral>() == 28);
    assert!(align_of::<BooleanLiteral>() == 4);
    assert!(offset_of!(BooleanLiteral, span) == 0);
    assert!(offset_of!(BooleanLiteral, value) == 24);

    // Padding: 0 bytes
    assert!(size_of::<NullLiteral>() == 24);
    assert!(align_of::<NullLiteral>() == 4);
    assert!(offset_of!(NullLiteral, span) == 0);

    // Padding: 7 bytes
    assert!(size_of::<NumericLiteral>() == 48);
    assert!(align_of::<NumericLiteral>() == 8);
    assert!(offset_of!(NumericLiteral, span) == 0);
    assert!(offset_of!(NumericLiteral, value) == 24);
    assert!(offset_of!(NumericLiteral, raw) == 32);
    assert!(offset_of!(NumericLiteral, base) == 40);

    // Padding: 3 bytes
    assert!(size_of::<StringLiteral>() == 44);
    assert!(align_of::<StringLiteral>() == 4);
    assert!(offset_of!(StringLiteral, span) == 0);
    assert!(offset_of!(StringLiteral, value) == 24);
    assert!(offset_of!(StringLiteral, raw) == 32);
    assert!(offset_of!(StringLiteral, lone_surrogates) == 40);

    // Padding: 3 bytes
    assert!(size_of::<BigIntLiteral>() == 44);
    assert!(align_of::<BigIntLiteral>() == 4);
    assert!(offset_of!(BigIntLiteral, span) == 0);
    assert!(offset_of!(BigIntLiteral, value) == 24);
    assert!(offset_of!(BigIntLiteral, raw) == 32);
    assert!(offset_of!(BigIntLiteral, base) == 40);

    // Padding: 0 bytes
    assert!(size_of::<RegExpLiteral>() == 48);
    assert!(align_of::<RegExpLiteral>() == 4);
    assert!(offset_of!(RegExpLiteral, span) == 0);
    assert!(offset_of!(RegExpLiteral, regex) == 24);
    assert!(offset_of!(RegExpLiteral, raw) == 40);

    // Padding: 3 bytes
    assert!(size_of::<RegExp>() == 16);
    assert!(align_of::<RegExp>() == 4);
    assert!(offset_of!(RegExp, pattern) == 0);
    assert!(offset_of!(RegExp, flags) == 12);

    // Padding: 0 bytes
    assert!(size_of::<RegExpPattern>() == 12);
    assert!(align_of::<RegExpPattern>() == 4);
    assert!(offset_of!(RegExpPattern, text) == 0);
    assert!(offset_of!(RegExpPattern, pattern) == 8);

    // Padding: 0 bytes
    assert!(size_of::<RegExpFlags>() == 1);
    assert!(align_of::<RegExpFlags>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<JSXElement>() == 48);
    assert!(align_of::<JSXElement>() == 4);
    assert!(offset_of!(JSXElement, span) == 0);
    assert!(offset_of!(JSXElement, opening_element) == 24);
    assert!(offset_of!(JSXElement, children) == 28);
    assert!(offset_of!(JSXElement, closing_element) == 44);

    // Padding: 0 bytes
    assert!(size_of::<JSXOpeningElement>() == 52);
    assert!(align_of::<JSXOpeningElement>() == 4);
    assert!(offset_of!(JSXOpeningElement, span) == 0);
    assert!(offset_of!(JSXOpeningElement, name) == 24);
    assert!(offset_of!(JSXOpeningElement, type_arguments) == 32);
    assert!(offset_of!(JSXOpeningElement, attributes) == 36);

    // Padding: 0 bytes
    assert!(size_of::<JSXClosingElement>() == 32);
    assert!(align_of::<JSXClosingElement>() == 4);
    assert!(offset_of!(JSXClosingElement, span) == 0);
    assert!(offset_of!(JSXClosingElement, name) == 24);

    // Padding: 0 bytes
    assert!(size_of::<JSXFragment>() == 88);
    assert!(align_of::<JSXFragment>() == 4);
    assert!(offset_of!(JSXFragment, span) == 0);
    assert!(offset_of!(JSXFragment, opening_fragment) == 24);
    assert!(offset_of!(JSXFragment, children) == 48);
    assert!(offset_of!(JSXFragment, closing_fragment) == 64);

    // Padding: 0 bytes
    assert!(size_of::<JSXOpeningFragment>() == 24);
    assert!(align_of::<JSXOpeningFragment>() == 4);
    assert!(offset_of!(JSXOpeningFragment, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<JSXClosingFragment>() == 24);
    assert!(align_of::<JSXClosingFragment>() == 4);
    assert!(offset_of!(JSXClosingFragment, span) == 0);

    assert!(size_of::<JSXElementName>() == 8);
    assert!(align_of::<JSXElementName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXNamespacedName>() == 88);
    assert!(align_of::<JSXNamespacedName>() == 4);
    assert!(offset_of!(JSXNamespacedName, span) == 0);
    assert!(offset_of!(JSXNamespacedName, namespace) == 24);
    assert!(offset_of!(JSXNamespacedName, name) == 56);

    // Padding: 0 bytes
    assert!(size_of::<JSXMemberExpression>() == 64);
    assert!(align_of::<JSXMemberExpression>() == 4);
    assert!(offset_of!(JSXMemberExpression, span) == 0);
    assert!(offset_of!(JSXMemberExpression, object) == 24);
    assert!(offset_of!(JSXMemberExpression, property) == 32);

    assert!(size_of::<JSXMemberExpressionObject>() == 8);
    assert!(align_of::<JSXMemberExpressionObject>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXExpressionContainer>() == 52);
    assert!(align_of::<JSXExpressionContainer>() == 4);
    assert!(offset_of!(JSXExpressionContainer, span) == 0);
    assert!(offset_of!(JSXExpressionContainer, expression) == 24);

    assert!(size_of::<JSXExpression>() == 28);
    assert!(align_of::<JSXExpression>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXEmptyExpression>() == 24);
    assert!(align_of::<JSXEmptyExpression>() == 4);
    assert!(offset_of!(JSXEmptyExpression, span) == 0);

    assert!(size_of::<JSXAttributeItem>() == 8);
    assert!(align_of::<JSXAttributeItem>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXAttribute>() == 40);
    assert!(align_of::<JSXAttribute>() == 4);
    assert!(offset_of!(JSXAttribute, span) == 0);
    assert!(offset_of!(JSXAttribute, name) == 24);
    assert!(offset_of!(JSXAttribute, value) == 32);

    // Padding: 0 bytes
    assert!(size_of::<JSXSpreadAttribute>() == 32);
    assert!(align_of::<JSXSpreadAttribute>() == 4);
    assert!(offset_of!(JSXSpreadAttribute, span) == 0);
    assert!(offset_of!(JSXSpreadAttribute, argument) == 24);

    assert!(size_of::<JSXAttributeName>() == 8);
    assert!(align_of::<JSXAttributeName>() == 4);

    assert!(size_of::<JSXAttributeValue>() == 8);
    assert!(align_of::<JSXAttributeValue>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXIdentifier>() == 32);
    assert!(align_of::<JSXIdentifier>() == 4);
    assert!(offset_of!(JSXIdentifier, span) == 0);
    assert!(offset_of!(JSXIdentifier, name) == 24);

    assert!(size_of::<JSXChild>() == 8);
    assert!(align_of::<JSXChild>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXSpreadChild>() == 32);
    assert!(align_of::<JSXSpreadChild>() == 4);
    assert!(offset_of!(JSXSpreadChild, span) == 0);
    assert!(offset_of!(JSXSpreadChild, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<JSXText>() == 40);
    assert!(align_of::<JSXText>() == 4);
    assert!(offset_of!(JSXText, span) == 0);
    assert!(offset_of!(JSXText, value) == 24);
    assert!(offset_of!(JSXText, raw) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSThisParameter>() == 52);
    assert!(align_of::<TSThisParameter>() == 4);
    assert!(offset_of!(TSThisParameter, span) == 0);
    assert!(offset_of!(TSThisParameter, this_span) == 24);
    assert!(offset_of!(TSThisParameter, type_annotation) == 48);

    // Padding: 2 bytes
    assert!(size_of::<TSEnumDeclaration>() == 108);
    assert!(align_of::<TSEnumDeclaration>() == 4);
    assert!(offset_of!(TSEnumDeclaration, span) == 0);
    assert!(offset_of!(TSEnumDeclaration, id) == 24);
    assert!(offset_of!(TSEnumDeclaration, body) == 60);
    assert!(offset_of!(TSEnumDeclaration, r#const) == 104);
    assert!(offset_of!(TSEnumDeclaration, declare) == 105);
    assert!(offset_of!(TSEnumDeclaration, scope_id) == 100);

    // Padding: 0 bytes
    assert!(size_of::<TSEnumBody>() == 40);
    assert!(align_of::<TSEnumBody>() == 4);
    assert!(offset_of!(TSEnumBody, span) == 0);
    assert!(offset_of!(TSEnumBody, members) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSEnumMember>() == 40);
    assert!(align_of::<TSEnumMember>() == 4);
    assert!(offset_of!(TSEnumMember, span) == 0);
    assert!(offset_of!(TSEnumMember, id) == 24);
    assert!(offset_of!(TSEnumMember, initializer) == 32);

    assert!(size_of::<TSEnumMemberName>() == 8);
    assert!(align_of::<TSEnumMemberName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeAnnotation>() == 32);
    assert!(align_of::<TSTypeAnnotation>() == 4);
    assert!(offset_of!(TSTypeAnnotation, span) == 0);
    assert!(offset_of!(TSTypeAnnotation, type_annotation) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSLiteralType>() == 32);
    assert!(align_of::<TSLiteralType>() == 4);
    assert!(offset_of!(TSLiteralType, span) == 0);
    assert!(offset_of!(TSLiteralType, literal) == 24);

    assert!(size_of::<TSLiteral>() == 8);
    assert!(align_of::<TSLiteral>() == 4);

    assert!(size_of::<TSType>() == 8);
    assert!(align_of::<TSType>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSConditionalType>() == 60);
    assert!(align_of::<TSConditionalType>() == 4);
    assert!(offset_of!(TSConditionalType, span) == 0);
    assert!(offset_of!(TSConditionalType, check_type) == 24);
    assert!(offset_of!(TSConditionalType, extends_type) == 32);
    assert!(offset_of!(TSConditionalType, true_type) == 40);
    assert!(offset_of!(TSConditionalType, false_type) == 48);
    assert!(offset_of!(TSConditionalType, scope_id) == 56);

    // Padding: 0 bytes
    assert!(size_of::<TSUnionType>() == 40);
    assert!(align_of::<TSUnionType>() == 4);
    assert!(offset_of!(TSUnionType, span) == 0);
    assert!(offset_of!(TSUnionType, types) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSIntersectionType>() == 40);
    assert!(align_of::<TSIntersectionType>() == 4);
    assert!(offset_of!(TSIntersectionType, span) == 0);
    assert!(offset_of!(TSIntersectionType, types) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSParenthesizedType>() == 32);
    assert!(align_of::<TSParenthesizedType>() == 4);
    assert!(offset_of!(TSParenthesizedType, span) == 0);
    assert!(offset_of!(TSParenthesizedType, type_annotation) == 24);

    // Padding: 3 bytes
    assert!(size_of::<TSTypeOperator>() == 36);
    assert!(align_of::<TSTypeOperator>() == 4);
    assert!(offset_of!(TSTypeOperator, span) == 0);
    assert!(offset_of!(TSTypeOperator, operator) == 32);
    assert!(offset_of!(TSTypeOperator, type_annotation) == 24);

    assert!(size_of::<TSTypeOperatorOperator>() == 1);
    assert!(align_of::<TSTypeOperatorOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSArrayType>() == 32);
    assert!(align_of::<TSArrayType>() == 4);
    assert!(offset_of!(TSArrayType, span) == 0);
    assert!(offset_of!(TSArrayType, element_type) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSIndexedAccessType>() == 40);
    assert!(align_of::<TSIndexedAccessType>() == 4);
    assert!(offset_of!(TSIndexedAccessType, span) == 0);
    assert!(offset_of!(TSIndexedAccessType, object_type) == 24);
    assert!(offset_of!(TSIndexedAccessType, index_type) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSTupleType>() == 40);
    assert!(align_of::<TSTupleType>() == 4);
    assert!(offset_of!(TSTupleType, span) == 0);
    assert!(offset_of!(TSTupleType, element_types) == 24);

    // Padding: 3 bytes
    assert!(size_of::<TSNamedTupleMember>() == 68);
    assert!(align_of::<TSNamedTupleMember>() == 4);
    assert!(offset_of!(TSNamedTupleMember, span) == 0);
    assert!(offset_of!(TSNamedTupleMember, label) == 24);
    assert!(offset_of!(TSNamedTupleMember, element_type) == 56);
    assert!(offset_of!(TSNamedTupleMember, optional) == 64);

    // Padding: 0 bytes
    assert!(size_of::<TSOptionalType>() == 32);
    assert!(align_of::<TSOptionalType>() == 4);
    assert!(offset_of!(TSOptionalType, span) == 0);
    assert!(offset_of!(TSOptionalType, type_annotation) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSRestType>() == 32);
    assert!(align_of::<TSRestType>() == 4);
    assert!(offset_of!(TSRestType, span) == 0);
    assert!(offset_of!(TSRestType, type_annotation) == 24);

    assert!(size_of::<TSTupleElement>() == 8);
    assert!(align_of::<TSTupleElement>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSAnyKeyword>() == 24);
    assert!(align_of::<TSAnyKeyword>() == 4);
    assert!(offset_of!(TSAnyKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSStringKeyword>() == 24);
    assert!(align_of::<TSStringKeyword>() == 4);
    assert!(offset_of!(TSStringKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSBooleanKeyword>() == 24);
    assert!(align_of::<TSBooleanKeyword>() == 4);
    assert!(offset_of!(TSBooleanKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNumberKeyword>() == 24);
    assert!(align_of::<TSNumberKeyword>() == 4);
    assert!(offset_of!(TSNumberKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNeverKeyword>() == 24);
    assert!(align_of::<TSNeverKeyword>() == 4);
    assert!(offset_of!(TSNeverKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSIntrinsicKeyword>() == 24);
    assert!(align_of::<TSIntrinsicKeyword>() == 4);
    assert!(offset_of!(TSIntrinsicKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSUnknownKeyword>() == 24);
    assert!(align_of::<TSUnknownKeyword>() == 4);
    assert!(offset_of!(TSUnknownKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNullKeyword>() == 24);
    assert!(align_of::<TSNullKeyword>() == 4);
    assert!(offset_of!(TSNullKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSUndefinedKeyword>() == 24);
    assert!(align_of::<TSUndefinedKeyword>() == 4);
    assert!(offset_of!(TSUndefinedKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSVoidKeyword>() == 24);
    assert!(align_of::<TSVoidKeyword>() == 4);
    assert!(offset_of!(TSVoidKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSSymbolKeyword>() == 24);
    assert!(align_of::<TSSymbolKeyword>() == 4);
    assert!(offset_of!(TSSymbolKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSThisType>() == 24);
    assert!(align_of::<TSThisType>() == 4);
    assert!(offset_of!(TSThisType, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSObjectKeyword>() == 24);
    assert!(align_of::<TSObjectKeyword>() == 4);
    assert!(offset_of!(TSObjectKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSBigIntKeyword>() == 24);
    assert!(align_of::<TSBigIntKeyword>() == 4);
    assert!(offset_of!(TSBigIntKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeReference>() == 36);
    assert!(align_of::<TSTypeReference>() == 4);
    assert!(offset_of!(TSTypeReference, span) == 0);
    assert!(offset_of!(TSTypeReference, type_name) == 24);
    assert!(offset_of!(TSTypeReference, type_arguments) == 32);

    assert!(size_of::<TSTypeName>() == 8);
    assert!(align_of::<TSTypeName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSQualifiedName>() == 64);
    assert!(align_of::<TSQualifiedName>() == 4);
    assert!(offset_of!(TSQualifiedName, span) == 0);
    assert!(offset_of!(TSQualifiedName, left) == 24);
    assert!(offset_of!(TSQualifiedName, right) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeParameterInstantiation>() == 40);
    assert!(align_of::<TSTypeParameterInstantiation>() == 4);
    assert!(offset_of!(TSTypeParameterInstantiation, span) == 0);
    assert!(offset_of!(TSTypeParameterInstantiation, params) == 24);

    // Padding: 1 bytes
    assert!(size_of::<TSTypeParameter>() == 80);
    assert!(align_of::<TSTypeParameter>() == 4);
    assert!(offset_of!(TSTypeParameter, span) == 0);
    assert!(offset_of!(TSTypeParameter, name) == 24);
    assert!(offset_of!(TSTypeParameter, constraint) == 60);
    assert!(offset_of!(TSTypeParameter, default) == 68);
    assert!(offset_of!(TSTypeParameter, r#in) == 76);
    assert!(offset_of!(TSTypeParameter, out) == 77);
    assert!(offset_of!(TSTypeParameter, r#const) == 78);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeParameterDeclaration>() == 40);
    assert!(align_of::<TSTypeParameterDeclaration>() == 4);
    assert!(offset_of!(TSTypeParameterDeclaration, span) == 0);
    assert!(offset_of!(TSTypeParameterDeclaration, params) == 24);

    // Padding: 3 bytes
    assert!(size_of::<TSTypeAliasDeclaration>() == 80);
    assert!(align_of::<TSTypeAliasDeclaration>() == 4);
    assert!(offset_of!(TSTypeAliasDeclaration, span) == 0);
    assert!(offset_of!(TSTypeAliasDeclaration, id) == 24);
    assert!(offset_of!(TSTypeAliasDeclaration, type_parameters) == 60);
    assert!(offset_of!(TSTypeAliasDeclaration, type_annotation) == 64);
    assert!(offset_of!(TSTypeAliasDeclaration, declare) == 76);
    assert!(offset_of!(TSTypeAliasDeclaration, scope_id) == 72);

    assert!(size_of::<TSAccessibility>() == 1);
    assert!(align_of::<TSAccessibility>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSClassImplements>() == 36);
    assert!(align_of::<TSClassImplements>() == 4);
    assert!(offset_of!(TSClassImplements, span) == 0);
    assert!(offset_of!(TSClassImplements, expression) == 24);
    assert!(offset_of!(TSClassImplements, type_arguments) == 32);

    // Padding: 3 bytes
    assert!(size_of::<TSInterfaceDeclaration>() == 92);
    assert!(align_of::<TSInterfaceDeclaration>() == 4);
    assert!(offset_of!(TSInterfaceDeclaration, span) == 0);
    assert!(offset_of!(TSInterfaceDeclaration, id) == 24);
    assert!(offset_of!(TSInterfaceDeclaration, type_parameters) == 60);
    assert!(offset_of!(TSInterfaceDeclaration, extends) == 64);
    assert!(offset_of!(TSInterfaceDeclaration, body) == 80);
    assert!(offset_of!(TSInterfaceDeclaration, declare) == 88);
    assert!(offset_of!(TSInterfaceDeclaration, scope_id) == 84);

    // Padding: 0 bytes
    assert!(size_of::<TSInterfaceBody>() == 40);
    assert!(align_of::<TSInterfaceBody>() == 4);
    assert!(offset_of!(TSInterfaceBody, span) == 0);
    assert!(offset_of!(TSInterfaceBody, body) == 24);

    // Padding: 1 bytes
    assert!(size_of::<TSPropertySignature>() == 40);
    assert!(align_of::<TSPropertySignature>() == 4);
    assert!(offset_of!(TSPropertySignature, span) == 0);
    assert!(offset_of!(TSPropertySignature, computed) == 36);
    assert!(offset_of!(TSPropertySignature, optional) == 37);
    assert!(offset_of!(TSPropertySignature, readonly) == 38);
    assert!(offset_of!(TSPropertySignature, key) == 24);
    assert!(offset_of!(TSPropertySignature, type_annotation) == 32);

    assert!(size_of::<TSSignature>() == 8);
    assert!(align_of::<TSSignature>() == 4);

    // Padding: 2 bytes
    assert!(size_of::<TSIndexSignature>() == 48);
    assert!(align_of::<TSIndexSignature>() == 4);
    assert!(offset_of!(TSIndexSignature, span) == 0);
    assert!(offset_of!(TSIndexSignature, parameters) == 24);
    assert!(offset_of!(TSIndexSignature, type_annotation) == 40);
    assert!(offset_of!(TSIndexSignature, readonly) == 44);
    assert!(offset_of!(TSIndexSignature, r#static) == 45);

    // Padding: 0 bytes
    assert!(size_of::<TSCallSignatureDeclaration>() == 40);
    assert!(align_of::<TSCallSignatureDeclaration>() == 4);
    assert!(offset_of!(TSCallSignatureDeclaration, span) == 0);
    assert!(offset_of!(TSCallSignatureDeclaration, type_parameters) == 24);
    assert!(offset_of!(TSCallSignatureDeclaration, this_param) == 28);
    assert!(offset_of!(TSCallSignatureDeclaration, params) == 32);
    assert!(offset_of!(TSCallSignatureDeclaration, return_type) == 36);

    assert!(size_of::<TSMethodSignatureKind>() == 1);
    assert!(align_of::<TSMethodSignatureKind>() == 1);

    // Padding: 1 bytes
    assert!(size_of::<TSMethodSignature>() == 56);
    assert!(align_of::<TSMethodSignature>() == 4);
    assert!(offset_of!(TSMethodSignature, span) == 0);
    assert!(offset_of!(TSMethodSignature, key) == 24);
    assert!(offset_of!(TSMethodSignature, computed) == 52);
    assert!(offset_of!(TSMethodSignature, optional) == 53);
    assert!(offset_of!(TSMethodSignature, kind) == 54);
    assert!(offset_of!(TSMethodSignature, type_parameters) == 32);
    assert!(offset_of!(TSMethodSignature, this_param) == 36);
    assert!(offset_of!(TSMethodSignature, params) == 40);
    assert!(offset_of!(TSMethodSignature, return_type) == 44);
    assert!(offset_of!(TSMethodSignature, scope_id) == 48);

    // Padding: 0 bytes
    assert!(size_of::<TSConstructSignatureDeclaration>() == 40);
    assert!(align_of::<TSConstructSignatureDeclaration>() == 4);
    assert!(offset_of!(TSConstructSignatureDeclaration, span) == 0);
    assert!(offset_of!(TSConstructSignatureDeclaration, type_parameters) == 24);
    assert!(offset_of!(TSConstructSignatureDeclaration, params) == 28);
    assert!(offset_of!(TSConstructSignatureDeclaration, return_type) == 32);
    assert!(offset_of!(TSConstructSignatureDeclaration, scope_id) == 36);

    // Padding: 0 bytes
    assert!(size_of::<TSIndexSignatureName>() == 36);
    assert!(align_of::<TSIndexSignatureName>() == 4);
    assert!(offset_of!(TSIndexSignatureName, span) == 0);
    assert!(offset_of!(TSIndexSignatureName, name) == 24);
    assert!(offset_of!(TSIndexSignatureName, type_annotation) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSInterfaceHeritage>() == 36);
    assert!(align_of::<TSInterfaceHeritage>() == 4);
    assert!(offset_of!(TSInterfaceHeritage, span) == 0);
    assert!(offset_of!(TSInterfaceHeritage, expression) == 24);
    assert!(offset_of!(TSInterfaceHeritage, type_arguments) == 32);

    // Padding: 3 bytes
    assert!(size_of::<TSTypePredicate>() == 60);
    assert!(align_of::<TSTypePredicate>() == 4);
    assert!(offset_of!(TSTypePredicate, span) == 0);
    assert!(offset_of!(TSTypePredicate, parameter_name) == 24);
    assert!(offset_of!(TSTypePredicate, asserts) == 56);
    assert!(offset_of!(TSTypePredicate, type_annotation) == 52);

    assert!(size_of::<TSTypePredicateName>() == 28);
    assert!(align_of::<TSTypePredicateName>() == 4);

    // Padding: 2 bytes
    assert!(size_of::<TSModuleDeclaration>() == 88);
    assert!(align_of::<TSModuleDeclaration>() == 4);
    assert!(offset_of!(TSModuleDeclaration, span) == 0);
    assert!(offset_of!(TSModuleDeclaration, id) == 24);
    assert!(offset_of!(TSModuleDeclaration, body) == 72);
    assert!(offset_of!(TSModuleDeclaration, kind) == 84);
    assert!(offset_of!(TSModuleDeclaration, declare) == 85);
    assert!(offset_of!(TSModuleDeclaration, scope_id) == 80);

    assert!(size_of::<TSModuleDeclarationKind>() == 1);
    assert!(align_of::<TSModuleDeclarationKind>() == 1);

    assert!(size_of::<TSModuleDeclarationName>() == 48);
    assert!(align_of::<TSModuleDeclarationName>() == 4);

    assert!(size_of::<TSModuleDeclarationBody>() == 8);
    assert!(align_of::<TSModuleDeclarationBody>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSModuleBlock>() == 56);
    assert!(align_of::<TSModuleBlock>() == 4);
    assert!(offset_of!(TSModuleBlock, span) == 0);
    assert!(offset_of!(TSModuleBlock, directives) == 24);
    assert!(offset_of!(TSModuleBlock, body) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeLiteral>() == 40);
    assert!(align_of::<TSTypeLiteral>() == 4);
    assert!(offset_of!(TSTypeLiteral, span) == 0);
    assert!(offset_of!(TSTypeLiteral, members) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSInferType>() == 28);
    assert!(align_of::<TSInferType>() == 4);
    assert!(offset_of!(TSInferType, span) == 0);
    assert!(offset_of!(TSInferType, type_parameter) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeQuery>() == 36);
    assert!(align_of::<TSTypeQuery>() == 4);
    assert!(offset_of!(TSTypeQuery, span) == 0);
    assert!(offset_of!(TSTypeQuery, expr_name) == 24);
    assert!(offset_of!(TSTypeQuery, type_arguments) == 32);

    assert!(size_of::<TSTypeQueryExprName>() == 8);
    assert!(align_of::<TSTypeQueryExprName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSImportType>() == 48);
    assert!(align_of::<TSImportType>() == 4);
    assert!(offset_of!(TSImportType, span) == 0);
    assert!(offset_of!(TSImportType, argument) == 24);
    assert!(offset_of!(TSImportType, options) == 32);
    assert!(offset_of!(TSImportType, qualifier) == 36);
    assert!(offset_of!(TSImportType, type_arguments) == 44);

    // Padding: 0 bytes
    assert!(size_of::<TSFunctionType>() == 44);
    assert!(align_of::<TSFunctionType>() == 4);
    assert!(offset_of!(TSFunctionType, span) == 0);
    assert!(offset_of!(TSFunctionType, type_parameters) == 24);
    assert!(offset_of!(TSFunctionType, this_param) == 28);
    assert!(offset_of!(TSFunctionType, params) == 32);
    assert!(offset_of!(TSFunctionType, return_type) == 36);
    assert!(offset_of!(TSFunctionType, scope_id) == 40);

    // Padding: 3 bytes
    assert!(size_of::<TSConstructorType>() == 40);
    assert!(align_of::<TSConstructorType>() == 4);
    assert!(offset_of!(TSConstructorType, span) == 0);
    assert!(offset_of!(TSConstructorType, r#abstract) == 36);
    assert!(offset_of!(TSConstructorType, type_parameters) == 24);
    assert!(offset_of!(TSConstructorType, params) == 28);
    assert!(offset_of!(TSConstructorType, return_type) == 32);

    // Padding: 2 bytes
    assert!(size_of::<TSMappedType>() == 52);
    assert!(align_of::<TSMappedType>() == 4);
    assert!(offset_of!(TSMappedType, span) == 0);
    assert!(offset_of!(TSMappedType, type_parameter) == 24);
    assert!(offset_of!(TSMappedType, name_type) == 28);
    assert!(offset_of!(TSMappedType, type_annotation) == 36);
    assert!(offset_of!(TSMappedType, optional) == 48);
    assert!(offset_of!(TSMappedType, readonly) == 49);
    assert!(offset_of!(TSMappedType, scope_id) == 44);

    assert!(size_of::<TSMappedTypeModifierOperator>() == 1);
    assert!(align_of::<TSMappedTypeModifierOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSTemplateLiteralType>() == 56);
    assert!(align_of::<TSTemplateLiteralType>() == 4);
    assert!(offset_of!(TSTemplateLiteralType, span) == 0);
    assert!(offset_of!(TSTemplateLiteralType, quasis) == 24);
    assert!(offset_of!(TSTemplateLiteralType, types) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSAsExpression>() == 40);
    assert!(align_of::<TSAsExpression>() == 4);
    assert!(offset_of!(TSAsExpression, span) == 0);
    assert!(offset_of!(TSAsExpression, expression) == 24);
    assert!(offset_of!(TSAsExpression, type_annotation) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSSatisfiesExpression>() == 40);
    assert!(align_of::<TSSatisfiesExpression>() == 4);
    assert!(offset_of!(TSSatisfiesExpression, span) == 0);
    assert!(offset_of!(TSSatisfiesExpression, expression) == 24);
    assert!(offset_of!(TSSatisfiesExpression, type_annotation) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeAssertion>() == 40);
    assert!(align_of::<TSTypeAssertion>() == 4);
    assert!(offset_of!(TSTypeAssertion, span) == 0);
    assert!(offset_of!(TSTypeAssertion, type_annotation) == 24);
    assert!(offset_of!(TSTypeAssertion, expression) == 32);

    // Padding: 3 bytes
    assert!(size_of::<TSImportEqualsDeclaration>() == 72);
    assert!(align_of::<TSImportEqualsDeclaration>() == 4);
    assert!(offset_of!(TSImportEqualsDeclaration, span) == 0);
    assert!(offset_of!(TSImportEqualsDeclaration, id) == 24);
    assert!(offset_of!(TSImportEqualsDeclaration, module_reference) == 60);
    assert!(offset_of!(TSImportEqualsDeclaration, import_kind) == 68);

    assert!(size_of::<TSModuleReference>() == 8);
    assert!(align_of::<TSModuleReference>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSExternalModuleReference>() == 68);
    assert!(align_of::<TSExternalModuleReference>() == 4);
    assert!(offset_of!(TSExternalModuleReference, span) == 0);
    assert!(offset_of!(TSExternalModuleReference, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSNonNullExpression>() == 32);
    assert!(align_of::<TSNonNullExpression>() == 4);
    assert!(offset_of!(TSNonNullExpression, span) == 0);
    assert!(offset_of!(TSNonNullExpression, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<Decorator>() == 32);
    assert!(align_of::<Decorator>() == 4);
    assert!(offset_of!(Decorator, span) == 0);
    assert!(offset_of!(Decorator, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSExportAssignment>() == 32);
    assert!(align_of::<TSExportAssignment>() == 4);
    assert!(offset_of!(TSExportAssignment, span) == 0);
    assert!(offset_of!(TSExportAssignment, expression) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSNamespaceExportDeclaration>() == 56);
    assert!(align_of::<TSNamespaceExportDeclaration>() == 4);
    assert!(offset_of!(TSNamespaceExportDeclaration, span) == 0);
    assert!(offset_of!(TSNamespaceExportDeclaration, id) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSInstantiationExpression>() == 36);
    assert!(align_of::<TSInstantiationExpression>() == 4);
    assert!(offset_of!(TSInstantiationExpression, span) == 0);
    assert!(offset_of!(TSInstantiationExpression, expression) == 24);
    assert!(offset_of!(TSInstantiationExpression, type_arguments) == 32);

    assert!(size_of::<ImportOrExportKind>() == 1);
    assert!(align_of::<ImportOrExportKind>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<JSDocNullableType>() == 36);
    assert!(align_of::<JSDocNullableType>() == 4);
    assert!(offset_of!(JSDocNullableType, span) == 0);
    assert!(offset_of!(JSDocNullableType, type_annotation) == 24);
    assert!(offset_of!(JSDocNullableType, postfix) == 32);

    // Padding: 3 bytes
    assert!(size_of::<JSDocNonNullableType>() == 36);
    assert!(align_of::<JSDocNonNullableType>() == 4);
    assert!(offset_of!(JSDocNonNullableType, span) == 0);
    assert!(offset_of!(JSDocNonNullableType, type_annotation) == 24);
    assert!(offset_of!(JSDocNonNullableType, postfix) == 32);

    // Padding: 0 bytes
    assert!(size_of::<JSDocUnknownType>() == 24);
    assert!(align_of::<JSDocUnknownType>() == 4);
    assert!(offset_of!(JSDocUnknownType, span) == 0);

    assert!(size_of::<CommentKind>() == 1);
    assert!(align_of::<CommentKind>() == 1);

    assert!(size_of::<CommentPosition>() == 1);
    assert!(align_of::<CommentPosition>() == 1);

    assert!(size_of::<CommentContent>() == 1);
    assert!(align_of::<CommentContent>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<CommentNewlines>() == 1);
    assert!(align_of::<CommentNewlines>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<Comment>() == 32);
    assert!(align_of::<Comment>() == 4);
    assert!(offset_of!(Comment, span) == 0);
    assert!(offset_of!(Comment, attached_to) == 24);
    assert!(offset_of!(Comment, kind) == 28);
    assert!(offset_of!(Comment, position) == 29);
    assert!(offset_of!(Comment, newlines) == 30);
    assert!(offset_of!(Comment, content) == 31);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
