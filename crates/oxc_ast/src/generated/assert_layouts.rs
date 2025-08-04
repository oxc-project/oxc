// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::ast::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 1 bytes
    assert!(size_of::<Program>() == 128);
    assert!(align_of::<Program>() == 8);
    assert!(offset_of!(Program, span) == 0);
    assert!(offset_of!(Program, source_type) == 124);
    assert!(offset_of!(Program, source_text) == 8);
    assert!(offset_of!(Program, comments) == 24);
    assert!(offset_of!(Program, hashbang) == 48);
    assert!(offset_of!(Program, directives) == 72);
    assert!(offset_of!(Program, body) == 96);
    assert!(offset_of!(Program, scope_id) == 120);

    assert!(size_of::<Expression>() == 16);
    assert!(align_of::<Expression>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<IdentifierName>() == 24);
    assert!(align_of::<IdentifierName>() == 8);
    assert!(offset_of!(IdentifierName, span) == 0);
    assert!(offset_of!(IdentifierName, name) == 8);

    // Padding: 4 bytes
    assert!(size_of::<IdentifierReference>() == 32);
    assert!(align_of::<IdentifierReference>() == 8);
    assert!(offset_of!(IdentifierReference, span) == 0);
    assert!(offset_of!(IdentifierReference, name) == 8);
    assert!(offset_of!(IdentifierReference, reference_id) == 24);

    // Padding: 4 bytes
    assert!(size_of::<BindingIdentifier>() == 32);
    assert!(align_of::<BindingIdentifier>() == 8);
    assert!(offset_of!(BindingIdentifier, span) == 0);
    assert!(offset_of!(BindingIdentifier, name) == 8);
    assert!(offset_of!(BindingIdentifier, symbol_id) == 24);

    // Padding: 0 bytes
    assert!(size_of::<LabelIdentifier>() == 24);
    assert!(align_of::<LabelIdentifier>() == 8);
    assert!(offset_of!(LabelIdentifier, span) == 0);
    assert!(offset_of!(LabelIdentifier, name) == 8);

    // Padding: 0 bytes
    assert!(size_of::<ThisExpression>() == 8);
    assert!(align_of::<ThisExpression>() == 8);
    assert!(offset_of!(ThisExpression, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ArrayExpression>() == 32);
    assert!(align_of::<ArrayExpression>() == 8);
    assert!(offset_of!(ArrayExpression, span) == 0);
    assert!(offset_of!(ArrayExpression, elements) == 8);

    assert!(size_of::<ArrayExpressionElement>() == 16);
    assert!(align_of::<ArrayExpressionElement>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<Elision>() == 8);
    assert!(align_of::<Elision>() == 8);
    assert!(offset_of!(Elision, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ObjectExpression>() == 32);
    assert!(align_of::<ObjectExpression>() == 8);
    assert!(offset_of!(ObjectExpression, span) == 0);
    assert!(offset_of!(ObjectExpression, properties) == 8);

    assert!(size_of::<ObjectPropertyKind>() == 16);
    assert!(align_of::<ObjectPropertyKind>() == 8);

    // Padding: 4 bytes
    assert!(size_of::<ObjectProperty>() == 48);
    assert!(align_of::<ObjectProperty>() == 8);
    assert!(offset_of!(ObjectProperty, span) == 0);
    assert!(offset_of!(ObjectProperty, kind) == 40);
    assert!(offset_of!(ObjectProperty, key) == 8);
    assert!(offset_of!(ObjectProperty, value) == 24);
    assert!(offset_of!(ObjectProperty, method) == 41);
    assert!(offset_of!(ObjectProperty, shorthand) == 42);
    assert!(offset_of!(ObjectProperty, computed) == 43);

    assert!(size_of::<PropertyKey>() == 16);
    assert!(align_of::<PropertyKey>() == 8);

    assert!(size_of::<PropertyKind>() == 1);
    assert!(align_of::<PropertyKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TemplateLiteral>() == 56);
    assert!(align_of::<TemplateLiteral>() == 8);
    assert!(offset_of!(TemplateLiteral, span) == 0);
    assert!(offset_of!(TemplateLiteral, quasis) == 8);
    assert!(offset_of!(TemplateLiteral, expressions) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TaggedTemplateExpression>() == 88);
    assert!(align_of::<TaggedTemplateExpression>() == 8);
    assert!(offset_of!(TaggedTemplateExpression, span) == 0);
    assert!(offset_of!(TaggedTemplateExpression, tag) == 8);
    assert!(offset_of!(TaggedTemplateExpression, type_arguments) == 24);
    assert!(offset_of!(TaggedTemplateExpression, quasi) == 32);

    // Padding: 6 bytes
    assert!(size_of::<TemplateElement>() == 48);
    assert!(align_of::<TemplateElement>() == 8);
    assert!(offset_of!(TemplateElement, span) == 0);
    assert!(offset_of!(TemplateElement, value) == 8);
    assert!(offset_of!(TemplateElement, tail) == 40);
    assert!(offset_of!(TemplateElement, lone_surrogates) == 41);

    // Padding: 0 bytes
    assert!(size_of::<TemplateElementValue>() == 32);
    assert!(align_of::<TemplateElementValue>() == 8);
    assert!(offset_of!(TemplateElementValue, raw) == 0);
    assert!(offset_of!(TemplateElementValue, cooked) == 16);

    assert!(size_of::<MemberExpression>() == 16);
    assert!(align_of::<MemberExpression>() == 8);

    // Padding: 7 bytes
    assert!(size_of::<ComputedMemberExpression>() == 48);
    assert!(align_of::<ComputedMemberExpression>() == 8);
    assert!(offset_of!(ComputedMemberExpression, span) == 0);
    assert!(offset_of!(ComputedMemberExpression, object) == 8);
    assert!(offset_of!(ComputedMemberExpression, expression) == 24);
    assert!(offset_of!(ComputedMemberExpression, optional) == 40);

    // Padding: 7 bytes
    assert!(size_of::<StaticMemberExpression>() == 56);
    assert!(align_of::<StaticMemberExpression>() == 8);
    assert!(offset_of!(StaticMemberExpression, span) == 0);
    assert!(offset_of!(StaticMemberExpression, object) == 8);
    assert!(offset_of!(StaticMemberExpression, property) == 24);
    assert!(offset_of!(StaticMemberExpression, optional) == 48);

    // Padding: 7 bytes
    assert!(size_of::<PrivateFieldExpression>() == 56);
    assert!(align_of::<PrivateFieldExpression>() == 8);
    assert!(offset_of!(PrivateFieldExpression, span) == 0);
    assert!(offset_of!(PrivateFieldExpression, object) == 8);
    assert!(offset_of!(PrivateFieldExpression, field) == 24);
    assert!(offset_of!(PrivateFieldExpression, optional) == 48);

    // Padding: 6 bytes
    assert!(size_of::<CallExpression>() == 64);
    assert!(align_of::<CallExpression>() == 8);
    assert!(offset_of!(CallExpression, span) == 0);
    assert!(offset_of!(CallExpression, callee) == 8);
    assert!(offset_of!(CallExpression, type_arguments) == 24);
    assert!(offset_of!(CallExpression, arguments) == 32);
    assert!(offset_of!(CallExpression, optional) == 56);
    assert!(offset_of!(CallExpression, pure) == 57);

    // Padding: 7 bytes
    assert!(size_of::<NewExpression>() == 64);
    assert!(align_of::<NewExpression>() == 8);
    assert!(offset_of!(NewExpression, span) == 0);
    assert!(offset_of!(NewExpression, callee) == 8);
    assert!(offset_of!(NewExpression, type_arguments) == 24);
    assert!(offset_of!(NewExpression, arguments) == 32);
    assert!(offset_of!(NewExpression, pure) == 56);

    // Padding: 0 bytes
    assert!(size_of::<MetaProperty>() == 56);
    assert!(align_of::<MetaProperty>() == 8);
    assert!(offset_of!(MetaProperty, span) == 0);
    assert!(offset_of!(MetaProperty, meta) == 8);
    assert!(offset_of!(MetaProperty, property) == 32);

    // Padding: 0 bytes
    assert!(size_of::<SpreadElement>() == 24);
    assert!(align_of::<SpreadElement>() == 8);
    assert!(offset_of!(SpreadElement, span) == 0);
    assert!(offset_of!(SpreadElement, argument) == 8);

    assert!(size_of::<Argument>() == 16);
    assert!(align_of::<Argument>() == 8);

    // Padding: 6 bytes
    assert!(size_of::<UpdateExpression>() == 32);
    assert!(align_of::<UpdateExpression>() == 8);
    assert!(offset_of!(UpdateExpression, span) == 0);
    assert!(offset_of!(UpdateExpression, operator) == 24);
    assert!(offset_of!(UpdateExpression, prefix) == 25);
    assert!(offset_of!(UpdateExpression, argument) == 8);

    // Padding: 7 bytes
    assert!(size_of::<UnaryExpression>() == 32);
    assert!(align_of::<UnaryExpression>() == 8);
    assert!(offset_of!(UnaryExpression, span) == 0);
    assert!(offset_of!(UnaryExpression, operator) == 24);
    assert!(offset_of!(UnaryExpression, argument) == 8);

    // Padding: 7 bytes
    assert!(size_of::<BinaryExpression>() == 48);
    assert!(align_of::<BinaryExpression>() == 8);
    assert!(offset_of!(BinaryExpression, span) == 0);
    assert!(offset_of!(BinaryExpression, left) == 8);
    assert!(offset_of!(BinaryExpression, operator) == 40);
    assert!(offset_of!(BinaryExpression, right) == 24);

    // Padding: 0 bytes
    assert!(size_of::<PrivateInExpression>() == 48);
    assert!(align_of::<PrivateInExpression>() == 8);
    assert!(offset_of!(PrivateInExpression, span) == 0);
    assert!(offset_of!(PrivateInExpression, left) == 8);
    assert!(offset_of!(PrivateInExpression, right) == 32);

    // Padding: 7 bytes
    assert!(size_of::<LogicalExpression>() == 48);
    assert!(align_of::<LogicalExpression>() == 8);
    assert!(offset_of!(LogicalExpression, span) == 0);
    assert!(offset_of!(LogicalExpression, left) == 8);
    assert!(offset_of!(LogicalExpression, operator) == 40);
    assert!(offset_of!(LogicalExpression, right) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ConditionalExpression>() == 56);
    assert!(align_of::<ConditionalExpression>() == 8);
    assert!(offset_of!(ConditionalExpression, span) == 0);
    assert!(offset_of!(ConditionalExpression, test) == 8);
    assert!(offset_of!(ConditionalExpression, consequent) == 24);
    assert!(offset_of!(ConditionalExpression, alternate) == 40);

    // Padding: 7 bytes
    assert!(size_of::<AssignmentExpression>() == 48);
    assert!(align_of::<AssignmentExpression>() == 8);
    assert!(offset_of!(AssignmentExpression, span) == 0);
    assert!(offset_of!(AssignmentExpression, operator) == 40);
    assert!(offset_of!(AssignmentExpression, left) == 8);
    assert!(offset_of!(AssignmentExpression, right) == 24);

    assert!(size_of::<AssignmentTarget>() == 16);
    assert!(align_of::<AssignmentTarget>() == 8);

    assert!(size_of::<SimpleAssignmentTarget>() == 16);
    assert!(align_of::<SimpleAssignmentTarget>() == 8);

    assert!(size_of::<AssignmentTargetPattern>() == 16);
    assert!(align_of::<AssignmentTargetPattern>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<ArrayAssignmentTarget>() == 40);
    assert!(align_of::<ArrayAssignmentTarget>() == 8);
    assert!(offset_of!(ArrayAssignmentTarget, span) == 0);
    assert!(offset_of!(ArrayAssignmentTarget, elements) == 8);
    assert!(offset_of!(ArrayAssignmentTarget, rest) == 32);

    // Padding: 0 bytes
    assert!(size_of::<ObjectAssignmentTarget>() == 40);
    assert!(align_of::<ObjectAssignmentTarget>() == 8);
    assert!(offset_of!(ObjectAssignmentTarget, span) == 0);
    assert!(offset_of!(ObjectAssignmentTarget, properties) == 8);
    assert!(offset_of!(ObjectAssignmentTarget, rest) == 32);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetRest>() == 24);
    assert!(align_of::<AssignmentTargetRest>() == 8);
    assert!(offset_of!(AssignmentTargetRest, span) == 0);
    assert!(offset_of!(AssignmentTargetRest, target) == 8);

    assert!(size_of::<AssignmentTargetMaybeDefault>() == 16);
    assert!(align_of::<AssignmentTargetMaybeDefault>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetWithDefault>() == 40);
    assert!(align_of::<AssignmentTargetWithDefault>() == 8);
    assert!(offset_of!(AssignmentTargetWithDefault, span) == 0);
    assert!(offset_of!(AssignmentTargetWithDefault, binding) == 8);
    assert!(offset_of!(AssignmentTargetWithDefault, init) == 24);

    assert!(size_of::<AssignmentTargetProperty>() == 16);
    assert!(align_of::<AssignmentTargetProperty>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetPropertyIdentifier>() == 56);
    assert!(align_of::<AssignmentTargetPropertyIdentifier>() == 8);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, span) == 0);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, binding) == 8);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, init) == 40);

    // Padding: 7 bytes
    assert!(size_of::<AssignmentTargetPropertyProperty>() == 48);
    assert!(align_of::<AssignmentTargetPropertyProperty>() == 8);
    assert!(offset_of!(AssignmentTargetPropertyProperty, span) == 0);
    assert!(offset_of!(AssignmentTargetPropertyProperty, name) == 8);
    assert!(offset_of!(AssignmentTargetPropertyProperty, binding) == 24);
    assert!(offset_of!(AssignmentTargetPropertyProperty, computed) == 40);

    // Padding: 0 bytes
    assert!(size_of::<SequenceExpression>() == 32);
    assert!(align_of::<SequenceExpression>() == 8);
    assert!(offset_of!(SequenceExpression, span) == 0);
    assert!(offset_of!(SequenceExpression, expressions) == 8);

    // Padding: 0 bytes
    assert!(size_of::<Super>() == 8);
    assert!(align_of::<Super>() == 8);
    assert!(offset_of!(Super, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<AwaitExpression>() == 24);
    assert!(align_of::<AwaitExpression>() == 8);
    assert!(offset_of!(AwaitExpression, span) == 0);
    assert!(offset_of!(AwaitExpression, argument) == 8);

    // Padding: 0 bytes
    assert!(size_of::<ChainExpression>() == 24);
    assert!(align_of::<ChainExpression>() == 8);
    assert!(offset_of!(ChainExpression, span) == 0);
    assert!(offset_of!(ChainExpression, expression) == 8);

    assert!(size_of::<ChainElement>() == 16);
    assert!(align_of::<ChainElement>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<ParenthesizedExpression>() == 24);
    assert!(align_of::<ParenthesizedExpression>() == 8);
    assert!(offset_of!(ParenthesizedExpression, span) == 0);
    assert!(offset_of!(ParenthesizedExpression, expression) == 8);

    assert!(size_of::<Statement>() == 16);
    assert!(align_of::<Statement>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<Directive>() == 72);
    assert!(align_of::<Directive>() == 8);
    assert!(offset_of!(Directive, span) == 0);
    assert!(offset_of!(Directive, expression) == 8);
    assert!(offset_of!(Directive, directive) == 56);

    // Padding: 0 bytes
    assert!(size_of::<Hashbang>() == 24);
    assert!(align_of::<Hashbang>() == 8);
    assert!(offset_of!(Hashbang, span) == 0);
    assert!(offset_of!(Hashbang, value) == 8);

    // Padding: 4 bytes
    assert!(size_of::<BlockStatement>() == 40);
    assert!(align_of::<BlockStatement>() == 8);
    assert!(offset_of!(BlockStatement, span) == 0);
    assert!(offset_of!(BlockStatement, body) == 8);
    assert!(offset_of!(BlockStatement, scope_id) == 32);

    assert!(size_of::<Declaration>() == 16);
    assert!(align_of::<Declaration>() == 8);

    // Padding: 6 bytes
    assert!(size_of::<VariableDeclaration>() == 40);
    assert!(align_of::<VariableDeclaration>() == 8);
    assert!(offset_of!(VariableDeclaration, span) == 0);
    assert!(offset_of!(VariableDeclaration, kind) == 32);
    assert!(offset_of!(VariableDeclaration, declarations) == 8);
    assert!(offset_of!(VariableDeclaration, declare) == 33);

    assert!(size_of::<VariableDeclarationKind>() == 1);
    assert!(align_of::<VariableDeclarationKind>() == 1);

    // Padding: 6 bytes
    assert!(size_of::<VariableDeclarator>() == 64);
    assert!(align_of::<VariableDeclarator>() == 8);
    assert!(offset_of!(VariableDeclarator, span) == 0);
    assert!(offset_of!(VariableDeclarator, kind) == 56);
    assert!(offset_of!(VariableDeclarator, id) == 8);
    assert!(offset_of!(VariableDeclarator, init) == 40);
    assert!(offset_of!(VariableDeclarator, definite) == 57);

    // Padding: 0 bytes
    assert!(size_of::<EmptyStatement>() == 8);
    assert!(align_of::<EmptyStatement>() == 8);
    assert!(offset_of!(EmptyStatement, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ExpressionStatement>() == 24);
    assert!(align_of::<ExpressionStatement>() == 8);
    assert!(offset_of!(ExpressionStatement, span) == 0);
    assert!(offset_of!(ExpressionStatement, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<IfStatement>() == 56);
    assert!(align_of::<IfStatement>() == 8);
    assert!(offset_of!(IfStatement, span) == 0);
    assert!(offset_of!(IfStatement, test) == 8);
    assert!(offset_of!(IfStatement, consequent) == 24);
    assert!(offset_of!(IfStatement, alternate) == 40);

    // Padding: 0 bytes
    assert!(size_of::<DoWhileStatement>() == 40);
    assert!(align_of::<DoWhileStatement>() == 8);
    assert!(offset_of!(DoWhileStatement, span) == 0);
    assert!(offset_of!(DoWhileStatement, body) == 8);
    assert!(offset_of!(DoWhileStatement, test) == 24);

    // Padding: 0 bytes
    assert!(size_of::<WhileStatement>() == 40);
    assert!(align_of::<WhileStatement>() == 8);
    assert!(offset_of!(WhileStatement, span) == 0);
    assert!(offset_of!(WhileStatement, test) == 8);
    assert!(offset_of!(WhileStatement, body) == 24);

    // Padding: 4 bytes
    assert!(size_of::<ForStatement>() == 80);
    assert!(align_of::<ForStatement>() == 8);
    assert!(offset_of!(ForStatement, span) == 0);
    assert!(offset_of!(ForStatement, init) == 8);
    assert!(offset_of!(ForStatement, test) == 24);
    assert!(offset_of!(ForStatement, update) == 40);
    assert!(offset_of!(ForStatement, body) == 56);
    assert!(offset_of!(ForStatement, scope_id) == 72);

    assert!(size_of::<ForStatementInit>() == 16);
    assert!(align_of::<ForStatementInit>() == 8);

    // Padding: 4 bytes
    assert!(size_of::<ForInStatement>() == 64);
    assert!(align_of::<ForInStatement>() == 8);
    assert!(offset_of!(ForInStatement, span) == 0);
    assert!(offset_of!(ForInStatement, left) == 8);
    assert!(offset_of!(ForInStatement, right) == 24);
    assert!(offset_of!(ForInStatement, body) == 40);
    assert!(offset_of!(ForInStatement, scope_id) == 56);

    assert!(size_of::<ForStatementLeft>() == 16);
    assert!(align_of::<ForStatementLeft>() == 8);

    // Padding: 3 bytes
    assert!(size_of::<ForOfStatement>() == 64);
    assert!(align_of::<ForOfStatement>() == 8);
    assert!(offset_of!(ForOfStatement, span) == 0);
    assert!(offset_of!(ForOfStatement, r#await) == 60);
    assert!(offset_of!(ForOfStatement, left) == 8);
    assert!(offset_of!(ForOfStatement, right) == 24);
    assert!(offset_of!(ForOfStatement, body) == 40);
    assert!(offset_of!(ForOfStatement, scope_id) == 56);

    // Padding: 0 bytes
    assert!(size_of::<ContinueStatement>() == 32);
    assert!(align_of::<ContinueStatement>() == 8);
    assert!(offset_of!(ContinueStatement, span) == 0);
    assert!(offset_of!(ContinueStatement, label) == 8);

    // Padding: 0 bytes
    assert!(size_of::<BreakStatement>() == 32);
    assert!(align_of::<BreakStatement>() == 8);
    assert!(offset_of!(BreakStatement, span) == 0);
    assert!(offset_of!(BreakStatement, label) == 8);

    // Padding: 0 bytes
    assert!(size_of::<ReturnStatement>() == 24);
    assert!(align_of::<ReturnStatement>() == 8);
    assert!(offset_of!(ReturnStatement, span) == 0);
    assert!(offset_of!(ReturnStatement, argument) == 8);

    // Padding: 0 bytes
    assert!(size_of::<WithStatement>() == 40);
    assert!(align_of::<WithStatement>() == 8);
    assert!(offset_of!(WithStatement, span) == 0);
    assert!(offset_of!(WithStatement, object) == 8);
    assert!(offset_of!(WithStatement, body) == 24);

    // Padding: 4 bytes
    assert!(size_of::<SwitchStatement>() == 56);
    assert!(align_of::<SwitchStatement>() == 8);
    assert!(offset_of!(SwitchStatement, span) == 0);
    assert!(offset_of!(SwitchStatement, discriminant) == 8);
    assert!(offset_of!(SwitchStatement, cases) == 24);
    assert!(offset_of!(SwitchStatement, scope_id) == 48);

    // Padding: 0 bytes
    assert!(size_of::<SwitchCase>() == 48);
    assert!(align_of::<SwitchCase>() == 8);
    assert!(offset_of!(SwitchCase, span) == 0);
    assert!(offset_of!(SwitchCase, test) == 8);
    assert!(offset_of!(SwitchCase, consequent) == 24);

    // Padding: 0 bytes
    assert!(size_of::<LabeledStatement>() == 48);
    assert!(align_of::<LabeledStatement>() == 8);
    assert!(offset_of!(LabeledStatement, span) == 0);
    assert!(offset_of!(LabeledStatement, label) == 8);
    assert!(offset_of!(LabeledStatement, body) == 32);

    // Padding: 0 bytes
    assert!(size_of::<ThrowStatement>() == 24);
    assert!(align_of::<ThrowStatement>() == 8);
    assert!(offset_of!(ThrowStatement, span) == 0);
    assert!(offset_of!(ThrowStatement, argument) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TryStatement>() == 32);
    assert!(align_of::<TryStatement>() == 8);
    assert!(offset_of!(TryStatement, span) == 0);
    assert!(offset_of!(TryStatement, block) == 8);
    assert!(offset_of!(TryStatement, handler) == 16);
    assert!(offset_of!(TryStatement, finalizer) == 24);

    // Padding: 4 bytes
    assert!(size_of::<CatchClause>() == 64);
    assert!(align_of::<CatchClause>() == 8);
    assert!(offset_of!(CatchClause, span) == 0);
    assert!(offset_of!(CatchClause, param) == 8);
    assert!(offset_of!(CatchClause, body) == 48);
    assert!(offset_of!(CatchClause, scope_id) == 56);

    // Padding: 0 bytes
    assert!(size_of::<CatchParameter>() == 40);
    assert!(align_of::<CatchParameter>() == 8);
    assert!(offset_of!(CatchParameter, span) == 0);
    assert!(offset_of!(CatchParameter, pattern) == 8);

    // Padding: 0 bytes
    assert!(size_of::<DebuggerStatement>() == 8);
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
    assert!(size_of::<AssignmentPattern>() == 56);
    assert!(align_of::<AssignmentPattern>() == 8);
    assert!(offset_of!(AssignmentPattern, span) == 0);
    assert!(offset_of!(AssignmentPattern, left) == 8);
    assert!(offset_of!(AssignmentPattern, right) == 40);

    // Padding: 0 bytes
    assert!(size_of::<ObjectPattern>() == 40);
    assert!(align_of::<ObjectPattern>() == 8);
    assert!(offset_of!(ObjectPattern, span) == 0);
    assert!(offset_of!(ObjectPattern, properties) == 8);
    assert!(offset_of!(ObjectPattern, rest) == 32);

    // Padding: 6 bytes
    assert!(size_of::<BindingProperty>() == 64);
    assert!(align_of::<BindingProperty>() == 8);
    assert!(offset_of!(BindingProperty, span) == 0);
    assert!(offset_of!(BindingProperty, key) == 8);
    assert!(offset_of!(BindingProperty, value) == 24);
    assert!(offset_of!(BindingProperty, shorthand) == 56);
    assert!(offset_of!(BindingProperty, computed) == 57);

    // Padding: 0 bytes
    assert!(size_of::<ArrayPattern>() == 40);
    assert!(align_of::<ArrayPattern>() == 8);
    assert!(offset_of!(ArrayPattern, span) == 0);
    assert!(offset_of!(ArrayPattern, elements) == 8);
    assert!(offset_of!(ArrayPattern, rest) == 32);

    // Padding: 0 bytes
    assert!(size_of::<BindingRestElement>() == 40);
    assert!(align_of::<BindingRestElement>() == 8);
    assert!(offset_of!(BindingRestElement, span) == 0);
    assert!(offset_of!(BindingRestElement, argument) == 8);

    // Padding: 6 bytes
    assert!(size_of::<Function>() == 96);
    assert!(align_of::<Function>() == 8);
    assert!(offset_of!(Function, span) == 0);
    assert!(offset_of!(Function, r#type) == 84);
    assert!(offset_of!(Function, id) == 8);
    assert!(offset_of!(Function, generator) == 85);
    assert!(offset_of!(Function, r#async) == 86);
    assert!(offset_of!(Function, declare) == 87);
    assert!(offset_of!(Function, type_parameters) == 40);
    assert!(offset_of!(Function, this_param) == 48);
    assert!(offset_of!(Function, params) == 56);
    assert!(offset_of!(Function, return_type) == 64);
    assert!(offset_of!(Function, body) == 72);
    assert!(offset_of!(Function, scope_id) == 80);
    assert!(offset_of!(Function, pure) == 88);
    assert!(offset_of!(Function, pife) == 89);

    assert!(size_of::<FunctionType>() == 1);
    assert!(align_of::<FunctionType>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<FormalParameters>() == 48);
    assert!(align_of::<FormalParameters>() == 8);
    assert!(offset_of!(FormalParameters, span) == 0);
    assert!(offset_of!(FormalParameters, kind) == 40);
    assert!(offset_of!(FormalParameters, items) == 8);
    assert!(offset_of!(FormalParameters, rest) == 32);

    // Padding: 5 bytes
    assert!(size_of::<FormalParameter>() == 72);
    assert!(align_of::<FormalParameter>() == 8);
    assert!(offset_of!(FormalParameter, span) == 0);
    assert!(offset_of!(FormalParameter, decorators) == 8);
    assert!(offset_of!(FormalParameter, pattern) == 32);
    assert!(offset_of!(FormalParameter, accessibility) == 64);
    assert!(offset_of!(FormalParameter, readonly) == 65);
    assert!(offset_of!(FormalParameter, r#override) == 66);

    assert!(size_of::<FormalParameterKind>() == 1);
    assert!(align_of::<FormalParameterKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<FunctionBody>() == 56);
    assert!(align_of::<FunctionBody>() == 8);
    assert!(offset_of!(FunctionBody, span) == 0);
    assert!(offset_of!(FunctionBody, directives) == 8);
    assert!(offset_of!(FunctionBody, statements) == 32);

    // Padding: 0 bytes
    assert!(size_of::<ArrowFunctionExpression>() == 48);
    assert!(align_of::<ArrowFunctionExpression>() == 8);
    assert!(offset_of!(ArrowFunctionExpression, span) == 0);
    assert!(offset_of!(ArrowFunctionExpression, expression) == 44);
    assert!(offset_of!(ArrowFunctionExpression, r#async) == 45);
    assert!(offset_of!(ArrowFunctionExpression, type_parameters) == 8);
    assert!(offset_of!(ArrowFunctionExpression, params) == 16);
    assert!(offset_of!(ArrowFunctionExpression, return_type) == 24);
    assert!(offset_of!(ArrowFunctionExpression, body) == 32);
    assert!(offset_of!(ArrowFunctionExpression, scope_id) == 40);
    assert!(offset_of!(ArrowFunctionExpression, pure) == 46);
    assert!(offset_of!(ArrowFunctionExpression, pife) == 47);

    // Padding: 7 bytes
    assert!(size_of::<YieldExpression>() == 32);
    assert!(align_of::<YieldExpression>() == 8);
    assert!(offset_of!(YieldExpression, span) == 0);
    assert!(offset_of!(YieldExpression, delegate) == 24);
    assert!(offset_of!(YieldExpression, argument) == 8);

    // Padding: 1 bytes
    assert!(size_of::<Class>() == 136);
    assert!(align_of::<Class>() == 8);
    assert!(offset_of!(Class, span) == 0);
    assert!(offset_of!(Class, r#type) == 132);
    assert!(offset_of!(Class, decorators) == 8);
    assert!(offset_of!(Class, id) == 32);
    assert!(offset_of!(Class, type_parameters) == 64);
    assert!(offset_of!(Class, super_class) == 72);
    assert!(offset_of!(Class, super_type_arguments) == 88);
    assert!(offset_of!(Class, implements) == 96);
    assert!(offset_of!(Class, body) == 120);
    assert!(offset_of!(Class, r#abstract) == 133);
    assert!(offset_of!(Class, declare) == 134);
    assert!(offset_of!(Class, scope_id) == 128);

    assert!(size_of::<ClassType>() == 1);
    assert!(align_of::<ClassType>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ClassBody>() == 32);
    assert!(align_of::<ClassBody>() == 8);
    assert!(offset_of!(ClassBody, span) == 0);
    assert!(offset_of!(ClassBody, body) == 8);

    assert!(size_of::<ClassElement>() == 16);
    assert!(align_of::<ClassElement>() == 8);

    // Padding: 1 bytes
    assert!(size_of::<MethodDefinition>() == 64);
    assert!(align_of::<MethodDefinition>() == 8);
    assert!(offset_of!(MethodDefinition, span) == 0);
    assert!(offset_of!(MethodDefinition, r#type) == 56);
    assert!(offset_of!(MethodDefinition, decorators) == 8);
    assert!(offset_of!(MethodDefinition, key) == 32);
    assert!(offset_of!(MethodDefinition, value) == 48);
    assert!(offset_of!(MethodDefinition, kind) == 57);
    assert!(offset_of!(MethodDefinition, computed) == 58);
    assert!(offset_of!(MethodDefinition, r#static) == 59);
    assert!(offset_of!(MethodDefinition, r#override) == 60);
    assert!(offset_of!(MethodDefinition, optional) == 61);
    assert!(offset_of!(MethodDefinition, accessibility) == 62);

    assert!(size_of::<MethodDefinitionType>() == 1);
    assert!(align_of::<MethodDefinitionType>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<PropertyDefinition>() == 88);
    assert!(align_of::<PropertyDefinition>() == 8);
    assert!(offset_of!(PropertyDefinition, span) == 0);
    assert!(offset_of!(PropertyDefinition, r#type) == 72);
    assert!(offset_of!(PropertyDefinition, decorators) == 8);
    assert!(offset_of!(PropertyDefinition, key) == 32);
    assert!(offset_of!(PropertyDefinition, type_annotation) == 48);
    assert!(offset_of!(PropertyDefinition, value) == 56);
    assert!(offset_of!(PropertyDefinition, computed) == 73);
    assert!(offset_of!(PropertyDefinition, r#static) == 74);
    assert!(offset_of!(PropertyDefinition, declare) == 75);
    assert!(offset_of!(PropertyDefinition, r#override) == 76);
    assert!(offset_of!(PropertyDefinition, optional) == 77);
    assert!(offset_of!(PropertyDefinition, definite) == 78);
    assert!(offset_of!(PropertyDefinition, readonly) == 79);
    assert!(offset_of!(PropertyDefinition, accessibility) == 80);

    assert!(size_of::<PropertyDefinitionType>() == 1);
    assert!(align_of::<PropertyDefinitionType>() == 1);

    assert!(size_of::<MethodDefinitionKind>() == 1);
    assert!(align_of::<MethodDefinitionKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<PrivateIdentifier>() == 24);
    assert!(align_of::<PrivateIdentifier>() == 8);
    assert!(offset_of!(PrivateIdentifier, span) == 0);
    assert!(offset_of!(PrivateIdentifier, name) == 8);

    // Padding: 4 bytes
    assert!(size_of::<StaticBlock>() == 40);
    assert!(align_of::<StaticBlock>() == 8);
    assert!(offset_of!(StaticBlock, span) == 0);
    assert!(offset_of!(StaticBlock, body) == 8);
    assert!(offset_of!(StaticBlock, scope_id) == 32);

    assert!(size_of::<ModuleDeclaration>() == 16);
    assert!(align_of::<ModuleDeclaration>() == 8);

    assert!(size_of::<AccessorPropertyType>() == 1);
    assert!(align_of::<AccessorPropertyType>() == 1);

    // Padding: 2 bytes
    assert!(size_of::<AccessorProperty>() == 80);
    assert!(align_of::<AccessorProperty>() == 8);
    assert!(offset_of!(AccessorProperty, span) == 0);
    assert!(offset_of!(AccessorProperty, r#type) == 72);
    assert!(offset_of!(AccessorProperty, decorators) == 8);
    assert!(offset_of!(AccessorProperty, key) == 32);
    assert!(offset_of!(AccessorProperty, type_annotation) == 48);
    assert!(offset_of!(AccessorProperty, value) == 56);
    assert!(offset_of!(AccessorProperty, computed) == 73);
    assert!(offset_of!(AccessorProperty, r#static) == 74);
    assert!(offset_of!(AccessorProperty, r#override) == 75);
    assert!(offset_of!(AccessorProperty, definite) == 76);
    assert!(offset_of!(AccessorProperty, accessibility) == 77);

    // Padding: 7 bytes
    assert!(size_of::<ImportExpression>() == 48);
    assert!(align_of::<ImportExpression>() == 8);
    assert!(offset_of!(ImportExpression, span) == 0);
    assert!(offset_of!(ImportExpression, source) == 8);
    assert!(offset_of!(ImportExpression, options) == 24);
    assert!(offset_of!(ImportExpression, phase) == 40);

    // Padding: 6 bytes
    assert!(size_of::<ImportDeclaration>() == 96);
    assert!(align_of::<ImportDeclaration>() == 8);
    assert!(offset_of!(ImportDeclaration, span) == 0);
    assert!(offset_of!(ImportDeclaration, specifiers) == 8);
    assert!(offset_of!(ImportDeclaration, source) == 32);
    assert!(offset_of!(ImportDeclaration, phase) == 88);
    assert!(offset_of!(ImportDeclaration, with_clause) == 80);
    assert!(offset_of!(ImportDeclaration, import_kind) == 89);

    assert!(size_of::<ImportPhase>() == 1);
    assert!(align_of::<ImportPhase>() == 1);

    assert!(size_of::<ImportDeclarationSpecifier>() == 16);
    assert!(align_of::<ImportDeclarationSpecifier>() == 8);

    // Padding: 7 bytes
    assert!(size_of::<ImportSpecifier>() == 104);
    assert!(align_of::<ImportSpecifier>() == 8);
    assert!(offset_of!(ImportSpecifier, span) == 0);
    assert!(offset_of!(ImportSpecifier, imported) == 8);
    assert!(offset_of!(ImportSpecifier, local) == 64);
    assert!(offset_of!(ImportSpecifier, import_kind) == 96);

    // Padding: 0 bytes
    assert!(size_of::<ImportDefaultSpecifier>() == 40);
    assert!(align_of::<ImportDefaultSpecifier>() == 8);
    assert!(offset_of!(ImportDefaultSpecifier, span) == 0);
    assert!(offset_of!(ImportDefaultSpecifier, local) == 8);

    // Padding: 0 bytes
    assert!(size_of::<ImportNamespaceSpecifier>() == 40);
    assert!(align_of::<ImportNamespaceSpecifier>() == 8);
    assert!(offset_of!(ImportNamespaceSpecifier, span) == 0);
    assert!(offset_of!(ImportNamespaceSpecifier, local) == 8);

    // Padding: 7 bytes
    assert!(size_of::<WithClause>() == 40);
    assert!(align_of::<WithClause>() == 8);
    assert!(offset_of!(WithClause, span) == 0);
    assert!(offset_of!(WithClause, keyword) == 32);
    assert!(offset_of!(WithClause, with_entries) == 8);

    assert!(size_of::<WithClauseKeyword>() == 1);
    assert!(align_of::<WithClauseKeyword>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ImportAttribute>() == 112);
    assert!(align_of::<ImportAttribute>() == 8);
    assert!(offset_of!(ImportAttribute, span) == 0);
    assert!(offset_of!(ImportAttribute, key) == 8);
    assert!(offset_of!(ImportAttribute, value) == 64);

    assert!(size_of::<ImportAttributeKey>() == 56);
    assert!(align_of::<ImportAttributeKey>() == 8);

    // Padding: 7 bytes
    assert!(size_of::<ExportNamedDeclaration>() == 112);
    assert!(align_of::<ExportNamedDeclaration>() == 8);
    assert!(offset_of!(ExportNamedDeclaration, span) == 0);
    assert!(offset_of!(ExportNamedDeclaration, declaration) == 8);
    assert!(offset_of!(ExportNamedDeclaration, specifiers) == 24);
    assert!(offset_of!(ExportNamedDeclaration, source) == 48);
    assert!(offset_of!(ExportNamedDeclaration, export_kind) == 104);
    assert!(offset_of!(ExportNamedDeclaration, with_clause) == 96);

    // Padding: 0 bytes
    assert!(size_of::<ExportDefaultDeclaration>() == 80);
    assert!(align_of::<ExportDefaultDeclaration>() == 8);
    assert!(offset_of!(ExportDefaultDeclaration, span) == 0);
    assert!(offset_of!(ExportDefaultDeclaration, exported) == 8);
    assert!(offset_of!(ExportDefaultDeclaration, declaration) == 64);

    // Padding: 7 bytes
    assert!(size_of::<ExportAllDeclaration>() == 128);
    assert!(align_of::<ExportAllDeclaration>() == 8);
    assert!(offset_of!(ExportAllDeclaration, span) == 0);
    assert!(offset_of!(ExportAllDeclaration, exported) == 8);
    assert!(offset_of!(ExportAllDeclaration, source) == 64);
    assert!(offset_of!(ExportAllDeclaration, with_clause) == 112);
    assert!(offset_of!(ExportAllDeclaration, export_kind) == 120);

    // Padding: 7 bytes
    assert!(size_of::<ExportSpecifier>() == 128);
    assert!(align_of::<ExportSpecifier>() == 8);
    assert!(offset_of!(ExportSpecifier, span) == 0);
    assert!(offset_of!(ExportSpecifier, local) == 8);
    assert!(offset_of!(ExportSpecifier, exported) == 64);
    assert!(offset_of!(ExportSpecifier, export_kind) == 120);

    assert!(size_of::<ExportDefaultDeclarationKind>() == 16);
    assert!(align_of::<ExportDefaultDeclarationKind>() == 8);

    assert!(size_of::<ModuleExportName>() == 56);
    assert!(align_of::<ModuleExportName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<V8IntrinsicExpression>() == 56);
    assert!(align_of::<V8IntrinsicExpression>() == 8);
    assert!(offset_of!(V8IntrinsicExpression, span) == 0);
    assert!(offset_of!(V8IntrinsicExpression, name) == 8);
    assert!(offset_of!(V8IntrinsicExpression, arguments) == 32);

    // Padding: 7 bytes
    assert!(size_of::<BooleanLiteral>() == 16);
    assert!(align_of::<BooleanLiteral>() == 8);
    assert!(offset_of!(BooleanLiteral, span) == 0);
    assert!(offset_of!(BooleanLiteral, value) == 8);

    // Padding: 0 bytes
    assert!(size_of::<NullLiteral>() == 8);
    assert!(align_of::<NullLiteral>() == 8);
    assert!(offset_of!(NullLiteral, span) == 0);

    // Padding: 7 bytes
    assert!(size_of::<NumericLiteral>() == 40);
    assert!(align_of::<NumericLiteral>() == 8);
    assert!(offset_of!(NumericLiteral, span) == 0);
    assert!(offset_of!(NumericLiteral, value) == 8);
    assert!(offset_of!(NumericLiteral, raw) == 16);
    assert!(offset_of!(NumericLiteral, base) == 32);

    // Padding: 7 bytes
    assert!(size_of::<StringLiteral>() == 48);
    assert!(align_of::<StringLiteral>() == 8);
    assert!(offset_of!(StringLiteral, span) == 0);
    assert!(offset_of!(StringLiteral, value) == 8);
    assert!(offset_of!(StringLiteral, raw) == 24);
    assert!(offset_of!(StringLiteral, lone_surrogates) == 40);

    // Padding: 7 bytes
    assert!(size_of::<BigIntLiteral>() == 48);
    assert!(align_of::<BigIntLiteral>() == 8);
    assert!(offset_of!(BigIntLiteral, span) == 0);
    assert!(offset_of!(BigIntLiteral, value) == 8);
    assert!(offset_of!(BigIntLiteral, raw) == 24);
    assert!(offset_of!(BigIntLiteral, base) == 40);

    // Padding: 0 bytes
    assert!(size_of::<RegExpLiteral>() == 56);
    assert!(align_of::<RegExpLiteral>() == 8);
    assert!(offset_of!(RegExpLiteral, span) == 0);
    assert!(offset_of!(RegExpLiteral, regex) == 8);
    assert!(offset_of!(RegExpLiteral, raw) == 40);

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
    assert!(size_of::<JSXElement>() == 48);
    assert!(align_of::<JSXElement>() == 8);
    assert!(offset_of!(JSXElement, span) == 0);
    assert!(offset_of!(JSXElement, opening_element) == 8);
    assert!(offset_of!(JSXElement, children) == 16);
    assert!(offset_of!(JSXElement, closing_element) == 40);

    // Padding: 0 bytes
    assert!(size_of::<JSXOpeningElement>() == 56);
    assert!(align_of::<JSXOpeningElement>() == 8);
    assert!(offset_of!(JSXOpeningElement, span) == 0);
    assert!(offset_of!(JSXOpeningElement, name) == 8);
    assert!(offset_of!(JSXOpeningElement, type_arguments) == 24);
    assert!(offset_of!(JSXOpeningElement, attributes) == 32);

    // Padding: 0 bytes
    assert!(size_of::<JSXClosingElement>() == 24);
    assert!(align_of::<JSXClosingElement>() == 8);
    assert!(offset_of!(JSXClosingElement, span) == 0);
    assert!(offset_of!(JSXClosingElement, name) == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXFragment>() == 48);
    assert!(align_of::<JSXFragment>() == 8);
    assert!(offset_of!(JSXFragment, span) == 0);
    assert!(offset_of!(JSXFragment, opening_fragment) == 8);
    assert!(offset_of!(JSXFragment, children) == 16);
    assert!(offset_of!(JSXFragment, closing_fragment) == 40);

    // Padding: 0 bytes
    assert!(size_of::<JSXOpeningFragment>() == 8);
    assert!(align_of::<JSXOpeningFragment>() == 8);
    assert!(offset_of!(JSXOpeningFragment, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<JSXClosingFragment>() == 8);
    assert!(align_of::<JSXClosingFragment>() == 8);
    assert!(offset_of!(JSXClosingFragment, span) == 0);

    assert!(size_of::<JSXElementName>() == 16);
    assert!(align_of::<JSXElementName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXNamespacedName>() == 56);
    assert!(align_of::<JSXNamespacedName>() == 8);
    assert!(offset_of!(JSXNamespacedName, span) == 0);
    assert!(offset_of!(JSXNamespacedName, namespace) == 8);
    assert!(offset_of!(JSXNamespacedName, name) == 32);

    // Padding: 0 bytes
    assert!(size_of::<JSXMemberExpression>() == 48);
    assert!(align_of::<JSXMemberExpression>() == 8);
    assert!(offset_of!(JSXMemberExpression, span) == 0);
    assert!(offset_of!(JSXMemberExpression, object) == 8);
    assert!(offset_of!(JSXMemberExpression, property) == 24);

    assert!(size_of::<JSXMemberExpressionObject>() == 16);
    assert!(align_of::<JSXMemberExpressionObject>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXExpressionContainer>() == 24);
    assert!(align_of::<JSXExpressionContainer>() == 8);
    assert!(offset_of!(JSXExpressionContainer, span) == 0);
    assert!(offset_of!(JSXExpressionContainer, expression) == 8);

    assert!(size_of::<JSXExpression>() == 16);
    assert!(align_of::<JSXExpression>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXEmptyExpression>() == 8);
    assert!(align_of::<JSXEmptyExpression>() == 8);
    assert!(offset_of!(JSXEmptyExpression, span) == 0);

    assert!(size_of::<JSXAttributeItem>() == 16);
    assert!(align_of::<JSXAttributeItem>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXAttribute>() == 40);
    assert!(align_of::<JSXAttribute>() == 8);
    assert!(offset_of!(JSXAttribute, span) == 0);
    assert!(offset_of!(JSXAttribute, name) == 8);
    assert!(offset_of!(JSXAttribute, value) == 24);

    // Padding: 0 bytes
    assert!(size_of::<JSXSpreadAttribute>() == 24);
    assert!(align_of::<JSXSpreadAttribute>() == 8);
    assert!(offset_of!(JSXSpreadAttribute, span) == 0);
    assert!(offset_of!(JSXSpreadAttribute, argument) == 8);

    assert!(size_of::<JSXAttributeName>() == 16);
    assert!(align_of::<JSXAttributeName>() == 8);

    assert!(size_of::<JSXAttributeValue>() == 16);
    assert!(align_of::<JSXAttributeValue>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXIdentifier>() == 24);
    assert!(align_of::<JSXIdentifier>() == 8);
    assert!(offset_of!(JSXIdentifier, span) == 0);
    assert!(offset_of!(JSXIdentifier, name) == 8);

    assert!(size_of::<JSXChild>() == 16);
    assert!(align_of::<JSXChild>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXSpreadChild>() == 24);
    assert!(align_of::<JSXSpreadChild>() == 8);
    assert!(offset_of!(JSXSpreadChild, span) == 0);
    assert!(offset_of!(JSXSpreadChild, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXText>() == 40);
    assert!(align_of::<JSXText>() == 8);
    assert!(offset_of!(JSXText, span) == 0);
    assert!(offset_of!(JSXText, value) == 8);
    assert!(offset_of!(JSXText, raw) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSThisParameter>() == 24);
    assert!(align_of::<TSThisParameter>() == 8);
    assert!(offset_of!(TSThisParameter, span) == 0);
    assert!(offset_of!(TSThisParameter, this_span) == 8);
    assert!(offset_of!(TSThisParameter, type_annotation) == 16);

    // Padding: 2 bytes
    assert!(size_of::<TSEnumDeclaration>() == 80);
    assert!(align_of::<TSEnumDeclaration>() == 8);
    assert!(offset_of!(TSEnumDeclaration, span) == 0);
    assert!(offset_of!(TSEnumDeclaration, id) == 8);
    assert!(offset_of!(TSEnumDeclaration, body) == 40);
    assert!(offset_of!(TSEnumDeclaration, r#const) == 76);
    assert!(offset_of!(TSEnumDeclaration, declare) == 77);
    assert!(offset_of!(TSEnumDeclaration, scope_id) == 72);

    // Padding: 0 bytes
    assert!(size_of::<TSEnumBody>() == 32);
    assert!(align_of::<TSEnumBody>() == 8);
    assert!(offset_of!(TSEnumBody, span) == 0);
    assert!(offset_of!(TSEnumBody, members) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSEnumMember>() == 40);
    assert!(align_of::<TSEnumMember>() == 8);
    assert!(offset_of!(TSEnumMember, span) == 0);
    assert!(offset_of!(TSEnumMember, id) == 8);
    assert!(offset_of!(TSEnumMember, initializer) == 24);

    assert!(size_of::<TSEnumMemberName>() == 16);
    assert!(align_of::<TSEnumMemberName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeAnnotation>() == 24);
    assert!(align_of::<TSTypeAnnotation>() == 8);
    assert!(offset_of!(TSTypeAnnotation, span) == 0);
    assert!(offset_of!(TSTypeAnnotation, type_annotation) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSLiteralType>() == 24);
    assert!(align_of::<TSLiteralType>() == 8);
    assert!(offset_of!(TSLiteralType, span) == 0);
    assert!(offset_of!(TSLiteralType, literal) == 8);

    assert!(size_of::<TSLiteral>() == 16);
    assert!(align_of::<TSLiteral>() == 8);

    assert!(size_of::<TSType>() == 16);
    assert!(align_of::<TSType>() == 8);

    // Padding: 4 bytes
    assert!(size_of::<TSConditionalType>() == 80);
    assert!(align_of::<TSConditionalType>() == 8);
    assert!(offset_of!(TSConditionalType, span) == 0);
    assert!(offset_of!(TSConditionalType, check_type) == 8);
    assert!(offset_of!(TSConditionalType, extends_type) == 24);
    assert!(offset_of!(TSConditionalType, true_type) == 40);
    assert!(offset_of!(TSConditionalType, false_type) == 56);
    assert!(offset_of!(TSConditionalType, scope_id) == 72);

    // Padding: 0 bytes
    assert!(size_of::<TSUnionType>() == 32);
    assert!(align_of::<TSUnionType>() == 8);
    assert!(offset_of!(TSUnionType, span) == 0);
    assert!(offset_of!(TSUnionType, types) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSIntersectionType>() == 32);
    assert!(align_of::<TSIntersectionType>() == 8);
    assert!(offset_of!(TSIntersectionType, span) == 0);
    assert!(offset_of!(TSIntersectionType, types) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSParenthesizedType>() == 24);
    assert!(align_of::<TSParenthesizedType>() == 8);
    assert!(offset_of!(TSParenthesizedType, span) == 0);
    assert!(offset_of!(TSParenthesizedType, type_annotation) == 8);

    // Padding: 7 bytes
    assert!(size_of::<TSTypeOperator>() == 32);
    assert!(align_of::<TSTypeOperator>() == 8);
    assert!(offset_of!(TSTypeOperator, span) == 0);
    assert!(offset_of!(TSTypeOperator, operator) == 24);
    assert!(offset_of!(TSTypeOperator, type_annotation) == 8);

    assert!(size_of::<TSTypeOperatorOperator>() == 1);
    assert!(align_of::<TSTypeOperatorOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSArrayType>() == 24);
    assert!(align_of::<TSArrayType>() == 8);
    assert!(offset_of!(TSArrayType, span) == 0);
    assert!(offset_of!(TSArrayType, element_type) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSIndexedAccessType>() == 40);
    assert!(align_of::<TSIndexedAccessType>() == 8);
    assert!(offset_of!(TSIndexedAccessType, span) == 0);
    assert!(offset_of!(TSIndexedAccessType, object_type) == 8);
    assert!(offset_of!(TSIndexedAccessType, index_type) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSTupleType>() == 32);
    assert!(align_of::<TSTupleType>() == 8);
    assert!(offset_of!(TSTupleType, span) == 0);
    assert!(offset_of!(TSTupleType, element_types) == 8);

    // Padding: 7 bytes
    assert!(size_of::<TSNamedTupleMember>() == 56);
    assert!(align_of::<TSNamedTupleMember>() == 8);
    assert!(offset_of!(TSNamedTupleMember, span) == 0);
    assert!(offset_of!(TSNamedTupleMember, label) == 8);
    assert!(offset_of!(TSNamedTupleMember, element_type) == 32);
    assert!(offset_of!(TSNamedTupleMember, optional) == 48);

    // Padding: 0 bytes
    assert!(size_of::<TSOptionalType>() == 24);
    assert!(align_of::<TSOptionalType>() == 8);
    assert!(offset_of!(TSOptionalType, span) == 0);
    assert!(offset_of!(TSOptionalType, type_annotation) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSRestType>() == 24);
    assert!(align_of::<TSRestType>() == 8);
    assert!(offset_of!(TSRestType, span) == 0);
    assert!(offset_of!(TSRestType, type_annotation) == 8);

    assert!(size_of::<TSTupleElement>() == 16);
    assert!(align_of::<TSTupleElement>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSAnyKeyword>() == 8);
    assert!(align_of::<TSAnyKeyword>() == 8);
    assert!(offset_of!(TSAnyKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSStringKeyword>() == 8);
    assert!(align_of::<TSStringKeyword>() == 8);
    assert!(offset_of!(TSStringKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSBooleanKeyword>() == 8);
    assert!(align_of::<TSBooleanKeyword>() == 8);
    assert!(offset_of!(TSBooleanKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNumberKeyword>() == 8);
    assert!(align_of::<TSNumberKeyword>() == 8);
    assert!(offset_of!(TSNumberKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNeverKeyword>() == 8);
    assert!(align_of::<TSNeverKeyword>() == 8);
    assert!(offset_of!(TSNeverKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSIntrinsicKeyword>() == 8);
    assert!(align_of::<TSIntrinsicKeyword>() == 8);
    assert!(offset_of!(TSIntrinsicKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSUnknownKeyword>() == 8);
    assert!(align_of::<TSUnknownKeyword>() == 8);
    assert!(offset_of!(TSUnknownKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNullKeyword>() == 8);
    assert!(align_of::<TSNullKeyword>() == 8);
    assert!(offset_of!(TSNullKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSUndefinedKeyword>() == 8);
    assert!(align_of::<TSUndefinedKeyword>() == 8);
    assert!(offset_of!(TSUndefinedKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSVoidKeyword>() == 8);
    assert!(align_of::<TSVoidKeyword>() == 8);
    assert!(offset_of!(TSVoidKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSSymbolKeyword>() == 8);
    assert!(align_of::<TSSymbolKeyword>() == 8);
    assert!(offset_of!(TSSymbolKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSThisType>() == 8);
    assert!(align_of::<TSThisType>() == 8);
    assert!(offset_of!(TSThisType, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSObjectKeyword>() == 8);
    assert!(align_of::<TSObjectKeyword>() == 8);
    assert!(offset_of!(TSObjectKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSBigIntKeyword>() == 8);
    assert!(align_of::<TSBigIntKeyword>() == 8);
    assert!(offset_of!(TSBigIntKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeReference>() == 32);
    assert!(align_of::<TSTypeReference>() == 8);
    assert!(offset_of!(TSTypeReference, span) == 0);
    assert!(offset_of!(TSTypeReference, type_name) == 8);
    assert!(offset_of!(TSTypeReference, type_arguments) == 24);

    assert!(size_of::<TSTypeName>() == 16);
    assert!(align_of::<TSTypeName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSQualifiedName>() == 48);
    assert!(align_of::<TSQualifiedName>() == 8);
    assert!(offset_of!(TSQualifiedName, span) == 0);
    assert!(offset_of!(TSQualifiedName, left) == 8);
    assert!(offset_of!(TSQualifiedName, right) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeParameterInstantiation>() == 32);
    assert!(align_of::<TSTypeParameterInstantiation>() == 8);
    assert!(offset_of!(TSTypeParameterInstantiation, span) == 0);
    assert!(offset_of!(TSTypeParameterInstantiation, params) == 8);

    // Padding: 5 bytes
    assert!(size_of::<TSTypeParameter>() == 80);
    assert!(align_of::<TSTypeParameter>() == 8);
    assert!(offset_of!(TSTypeParameter, span) == 0);
    assert!(offset_of!(TSTypeParameter, name) == 8);
    assert!(offset_of!(TSTypeParameter, constraint) == 40);
    assert!(offset_of!(TSTypeParameter, default) == 56);
    assert!(offset_of!(TSTypeParameter, r#in) == 72);
    assert!(offset_of!(TSTypeParameter, out) == 73);
    assert!(offset_of!(TSTypeParameter, r#const) == 74);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeParameterDeclaration>() == 32);
    assert!(align_of::<TSTypeParameterDeclaration>() == 8);
    assert!(offset_of!(TSTypeParameterDeclaration, span) == 0);
    assert!(offset_of!(TSTypeParameterDeclaration, params) == 8);

    // Padding: 3 bytes
    assert!(size_of::<TSTypeAliasDeclaration>() == 72);
    assert!(align_of::<TSTypeAliasDeclaration>() == 8);
    assert!(offset_of!(TSTypeAliasDeclaration, span) == 0);
    assert!(offset_of!(TSTypeAliasDeclaration, id) == 8);
    assert!(offset_of!(TSTypeAliasDeclaration, type_parameters) == 40);
    assert!(offset_of!(TSTypeAliasDeclaration, type_annotation) == 48);
    assert!(offset_of!(TSTypeAliasDeclaration, declare) == 68);
    assert!(offset_of!(TSTypeAliasDeclaration, scope_id) == 64);

    assert!(size_of::<TSAccessibility>() == 1);
    assert!(align_of::<TSAccessibility>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSClassImplements>() == 32);
    assert!(align_of::<TSClassImplements>() == 8);
    assert!(offset_of!(TSClassImplements, span) == 0);
    assert!(offset_of!(TSClassImplements, expression) == 8);
    assert!(offset_of!(TSClassImplements, type_arguments) == 24);

    // Padding: 3 bytes
    assert!(size_of::<TSInterfaceDeclaration>() == 88);
    assert!(align_of::<TSInterfaceDeclaration>() == 8);
    assert!(offset_of!(TSInterfaceDeclaration, span) == 0);
    assert!(offset_of!(TSInterfaceDeclaration, id) == 8);
    assert!(offset_of!(TSInterfaceDeclaration, type_parameters) == 40);
    assert!(offset_of!(TSInterfaceDeclaration, extends) == 48);
    assert!(offset_of!(TSInterfaceDeclaration, body) == 72);
    assert!(offset_of!(TSInterfaceDeclaration, declare) == 84);
    assert!(offset_of!(TSInterfaceDeclaration, scope_id) == 80);

    // Padding: 0 bytes
    assert!(size_of::<TSInterfaceBody>() == 32);
    assert!(align_of::<TSInterfaceBody>() == 8);
    assert!(offset_of!(TSInterfaceBody, span) == 0);
    assert!(offset_of!(TSInterfaceBody, body) == 8);

    // Padding: 5 bytes
    assert!(size_of::<TSPropertySignature>() == 40);
    assert!(align_of::<TSPropertySignature>() == 8);
    assert!(offset_of!(TSPropertySignature, span) == 0);
    assert!(offset_of!(TSPropertySignature, computed) == 32);
    assert!(offset_of!(TSPropertySignature, optional) == 33);
    assert!(offset_of!(TSPropertySignature, readonly) == 34);
    assert!(offset_of!(TSPropertySignature, key) == 8);
    assert!(offset_of!(TSPropertySignature, type_annotation) == 24);

    assert!(size_of::<TSSignature>() == 16);
    assert!(align_of::<TSSignature>() == 8);

    // Padding: 6 bytes
    assert!(size_of::<TSIndexSignature>() == 48);
    assert!(align_of::<TSIndexSignature>() == 8);
    assert!(offset_of!(TSIndexSignature, span) == 0);
    assert!(offset_of!(TSIndexSignature, parameters) == 8);
    assert!(offset_of!(TSIndexSignature, type_annotation) == 32);
    assert!(offset_of!(TSIndexSignature, readonly) == 40);
    assert!(offset_of!(TSIndexSignature, r#static) == 41);

    // Padding: 0 bytes
    assert!(size_of::<TSCallSignatureDeclaration>() == 40);
    assert!(align_of::<TSCallSignatureDeclaration>() == 8);
    assert!(offset_of!(TSCallSignatureDeclaration, span) == 0);
    assert!(offset_of!(TSCallSignatureDeclaration, type_parameters) == 8);
    assert!(offset_of!(TSCallSignatureDeclaration, this_param) == 16);
    assert!(offset_of!(TSCallSignatureDeclaration, params) == 24);
    assert!(offset_of!(TSCallSignatureDeclaration, return_type) == 32);

    assert!(size_of::<TSMethodSignatureKind>() == 1);
    assert!(align_of::<TSMethodSignatureKind>() == 1);

    // Padding: 1 bytes
    assert!(size_of::<TSMethodSignature>() == 64);
    assert!(align_of::<TSMethodSignature>() == 8);
    assert!(offset_of!(TSMethodSignature, span) == 0);
    assert!(offset_of!(TSMethodSignature, key) == 8);
    assert!(offset_of!(TSMethodSignature, computed) == 60);
    assert!(offset_of!(TSMethodSignature, optional) == 61);
    assert!(offset_of!(TSMethodSignature, kind) == 62);
    assert!(offset_of!(TSMethodSignature, type_parameters) == 24);
    assert!(offset_of!(TSMethodSignature, this_param) == 32);
    assert!(offset_of!(TSMethodSignature, params) == 40);
    assert!(offset_of!(TSMethodSignature, return_type) == 48);
    assert!(offset_of!(TSMethodSignature, scope_id) == 56);

    // Padding: 4 bytes
    assert!(size_of::<TSConstructSignatureDeclaration>() == 40);
    assert!(align_of::<TSConstructSignatureDeclaration>() == 8);
    assert!(offset_of!(TSConstructSignatureDeclaration, span) == 0);
    assert!(offset_of!(TSConstructSignatureDeclaration, type_parameters) == 8);
    assert!(offset_of!(TSConstructSignatureDeclaration, params) == 16);
    assert!(offset_of!(TSConstructSignatureDeclaration, return_type) == 24);
    assert!(offset_of!(TSConstructSignatureDeclaration, scope_id) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSIndexSignatureName>() == 32);
    assert!(align_of::<TSIndexSignatureName>() == 8);
    assert!(offset_of!(TSIndexSignatureName, span) == 0);
    assert!(offset_of!(TSIndexSignatureName, name) == 8);
    assert!(offset_of!(TSIndexSignatureName, type_annotation) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSInterfaceHeritage>() == 32);
    assert!(align_of::<TSInterfaceHeritage>() == 8);
    assert!(offset_of!(TSInterfaceHeritage, span) == 0);
    assert!(offset_of!(TSInterfaceHeritage, expression) == 8);
    assert!(offset_of!(TSInterfaceHeritage, type_arguments) == 24);

    // Padding: 7 bytes
    assert!(size_of::<TSTypePredicate>() == 40);
    assert!(align_of::<TSTypePredicate>() == 8);
    assert!(offset_of!(TSTypePredicate, span) == 0);
    assert!(offset_of!(TSTypePredicate, parameter_name) == 8);
    assert!(offset_of!(TSTypePredicate, asserts) == 32);
    assert!(offset_of!(TSTypePredicate, type_annotation) == 24);

    assert!(size_of::<TSTypePredicateName>() == 16);
    assert!(align_of::<TSTypePredicateName>() == 8);

    // Padding: 2 bytes
    assert!(size_of::<TSModuleDeclaration>() == 88);
    assert!(align_of::<TSModuleDeclaration>() == 8);
    assert!(offset_of!(TSModuleDeclaration, span) == 0);
    assert!(offset_of!(TSModuleDeclaration, id) == 8);
    assert!(offset_of!(TSModuleDeclaration, body) == 64);
    assert!(offset_of!(TSModuleDeclaration, kind) == 84);
    assert!(offset_of!(TSModuleDeclaration, declare) == 85);
    assert!(offset_of!(TSModuleDeclaration, scope_id) == 80);

    assert!(size_of::<TSModuleDeclarationKind>() == 1);
    assert!(align_of::<TSModuleDeclarationKind>() == 1);

    assert!(size_of::<TSModuleDeclarationName>() == 56);
    assert!(align_of::<TSModuleDeclarationName>() == 8);

    assert!(size_of::<TSModuleDeclarationBody>() == 16);
    assert!(align_of::<TSModuleDeclarationBody>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSModuleBlock>() == 56);
    assert!(align_of::<TSModuleBlock>() == 8);
    assert!(offset_of!(TSModuleBlock, span) == 0);
    assert!(offset_of!(TSModuleBlock, directives) == 8);
    assert!(offset_of!(TSModuleBlock, body) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeLiteral>() == 32);
    assert!(align_of::<TSTypeLiteral>() == 8);
    assert!(offset_of!(TSTypeLiteral, span) == 0);
    assert!(offset_of!(TSTypeLiteral, members) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSInferType>() == 16);
    assert!(align_of::<TSInferType>() == 8);
    assert!(offset_of!(TSInferType, span) == 0);
    assert!(offset_of!(TSInferType, type_parameter) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeQuery>() == 32);
    assert!(align_of::<TSTypeQuery>() == 8);
    assert!(offset_of!(TSTypeQuery, span) == 0);
    assert!(offset_of!(TSTypeQuery, expr_name) == 8);
    assert!(offset_of!(TSTypeQuery, type_arguments) == 24);

    assert!(size_of::<TSTypeQueryExprName>() == 16);
    assert!(align_of::<TSTypeQueryExprName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSImportType>() == 56);
    assert!(align_of::<TSImportType>() == 8);
    assert!(offset_of!(TSImportType, span) == 0);
    assert!(offset_of!(TSImportType, argument) == 8);
    assert!(offset_of!(TSImportType, options) == 24);
    assert!(offset_of!(TSImportType, qualifier) == 32);
    assert!(offset_of!(TSImportType, type_arguments) == 48);

    // Padding: 4 bytes
    assert!(size_of::<TSFunctionType>() == 48);
    assert!(align_of::<TSFunctionType>() == 8);
    assert!(offset_of!(TSFunctionType, span) == 0);
    assert!(offset_of!(TSFunctionType, type_parameters) == 8);
    assert!(offset_of!(TSFunctionType, this_param) == 16);
    assert!(offset_of!(TSFunctionType, params) == 24);
    assert!(offset_of!(TSFunctionType, return_type) == 32);
    assert!(offset_of!(TSFunctionType, scope_id) == 40);

    // Padding: 7 bytes
    assert!(size_of::<TSConstructorType>() == 40);
    assert!(align_of::<TSConstructorType>() == 8);
    assert!(offset_of!(TSConstructorType, span) == 0);
    assert!(offset_of!(TSConstructorType, r#abstract) == 32);
    assert!(offset_of!(TSConstructorType, type_parameters) == 8);
    assert!(offset_of!(TSConstructorType, params) == 16);
    assert!(offset_of!(TSConstructorType, return_type) == 24);

    // Padding: 2 bytes
    assert!(size_of::<TSMappedType>() == 56);
    assert!(align_of::<TSMappedType>() == 8);
    assert!(offset_of!(TSMappedType, span) == 0);
    assert!(offset_of!(TSMappedType, type_parameter) == 8);
    assert!(offset_of!(TSMappedType, name_type) == 16);
    assert!(offset_of!(TSMappedType, type_annotation) == 32);
    assert!(offset_of!(TSMappedType, optional) == 52);
    assert!(offset_of!(TSMappedType, readonly) == 53);
    assert!(offset_of!(TSMappedType, scope_id) == 48);

    assert!(size_of::<TSMappedTypeModifierOperator>() == 1);
    assert!(align_of::<TSMappedTypeModifierOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSTemplateLiteralType>() == 56);
    assert!(align_of::<TSTemplateLiteralType>() == 8);
    assert!(offset_of!(TSTemplateLiteralType, span) == 0);
    assert!(offset_of!(TSTemplateLiteralType, quasis) == 8);
    assert!(offset_of!(TSTemplateLiteralType, types) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSAsExpression>() == 40);
    assert!(align_of::<TSAsExpression>() == 8);
    assert!(offset_of!(TSAsExpression, span) == 0);
    assert!(offset_of!(TSAsExpression, expression) == 8);
    assert!(offset_of!(TSAsExpression, type_annotation) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSSatisfiesExpression>() == 40);
    assert!(align_of::<TSSatisfiesExpression>() == 8);
    assert!(offset_of!(TSSatisfiesExpression, span) == 0);
    assert!(offset_of!(TSSatisfiesExpression, expression) == 8);
    assert!(offset_of!(TSSatisfiesExpression, type_annotation) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeAssertion>() == 40);
    assert!(align_of::<TSTypeAssertion>() == 8);
    assert!(offset_of!(TSTypeAssertion, span) == 0);
    assert!(offset_of!(TSTypeAssertion, type_annotation) == 8);
    assert!(offset_of!(TSTypeAssertion, expression) == 24);

    // Padding: 7 bytes
    assert!(size_of::<TSImportEqualsDeclaration>() == 64);
    assert!(align_of::<TSImportEqualsDeclaration>() == 8);
    assert!(offset_of!(TSImportEqualsDeclaration, span) == 0);
    assert!(offset_of!(TSImportEqualsDeclaration, id) == 8);
    assert!(offset_of!(TSImportEqualsDeclaration, module_reference) == 40);
    assert!(offset_of!(TSImportEqualsDeclaration, import_kind) == 56);

    assert!(size_of::<TSModuleReference>() == 16);
    assert!(align_of::<TSModuleReference>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSExternalModuleReference>() == 56);
    assert!(align_of::<TSExternalModuleReference>() == 8);
    assert!(offset_of!(TSExternalModuleReference, span) == 0);
    assert!(offset_of!(TSExternalModuleReference, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSNonNullExpression>() == 24);
    assert!(align_of::<TSNonNullExpression>() == 8);
    assert!(offset_of!(TSNonNullExpression, span) == 0);
    assert!(offset_of!(TSNonNullExpression, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<Decorator>() == 24);
    assert!(align_of::<Decorator>() == 8);
    assert!(offset_of!(Decorator, span) == 0);
    assert!(offset_of!(Decorator, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSExportAssignment>() == 24);
    assert!(align_of::<TSExportAssignment>() == 8);
    assert!(offset_of!(TSExportAssignment, span) == 0);
    assert!(offset_of!(TSExportAssignment, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSNamespaceExportDeclaration>() == 32);
    assert!(align_of::<TSNamespaceExportDeclaration>() == 8);
    assert!(offset_of!(TSNamespaceExportDeclaration, span) == 0);
    assert!(offset_of!(TSNamespaceExportDeclaration, id) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSInstantiationExpression>() == 32);
    assert!(align_of::<TSInstantiationExpression>() == 8);
    assert!(offset_of!(TSInstantiationExpression, span) == 0);
    assert!(offset_of!(TSInstantiationExpression, expression) == 8);
    assert!(offset_of!(TSInstantiationExpression, type_arguments) == 24);

    assert!(size_of::<ImportOrExportKind>() == 1);
    assert!(align_of::<ImportOrExportKind>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<JSDocNullableType>() == 32);
    assert!(align_of::<JSDocNullableType>() == 8);
    assert!(offset_of!(JSDocNullableType, span) == 0);
    assert!(offset_of!(JSDocNullableType, type_annotation) == 8);
    assert!(offset_of!(JSDocNullableType, postfix) == 24);

    // Padding: 7 bytes
    assert!(size_of::<JSDocNonNullableType>() == 32);
    assert!(align_of::<JSDocNonNullableType>() == 8);
    assert!(offset_of!(JSDocNonNullableType, span) == 0);
    assert!(offset_of!(JSDocNonNullableType, type_annotation) == 8);
    assert!(offset_of!(JSDocNonNullableType, postfix) == 24);

    // Padding: 0 bytes
    assert!(size_of::<JSDocUnknownType>() == 8);
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
    assert!(size_of::<Comment>() == 16);
    assert!(align_of::<Comment>() == 8);
    assert!(offset_of!(Comment, span) == 0);
    assert!(offset_of!(Comment, attached_to) == 8);
    assert!(offset_of!(Comment, kind) == 12);
    assert!(offset_of!(Comment, position) == 13);
    assert!(offset_of!(Comment, newlines) == 14);
    assert!(offset_of!(Comment, content) == 15);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    // Padding: 1 bytes
    assert!(size_of::<Program>() == 88);
    assert!(align_of::<Program>() == 4);
    assert!(offset_of!(Program, span) == 0);
    assert!(offset_of!(Program, source_type) == 84);
    assert!(offset_of!(Program, source_text) == 8);
    assert!(offset_of!(Program, comments) == 16);
    assert!(offset_of!(Program, hashbang) == 32);
    assert!(offset_of!(Program, directives) == 48);
    assert!(offset_of!(Program, body) == 64);
    assert!(offset_of!(Program, scope_id) == 80);

    assert!(size_of::<Expression>() == 8);
    assert!(align_of::<Expression>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<IdentifierName>() == 16);
    assert!(align_of::<IdentifierName>() == 4);
    assert!(offset_of!(IdentifierName, span) == 0);
    assert!(offset_of!(IdentifierName, name) == 8);

    // Padding: 0 bytes
    assert!(size_of::<IdentifierReference>() == 20);
    assert!(align_of::<IdentifierReference>() == 4);
    assert!(offset_of!(IdentifierReference, span) == 0);
    assert!(offset_of!(IdentifierReference, name) == 8);
    assert!(offset_of!(IdentifierReference, reference_id) == 16);

    // Padding: 0 bytes
    assert!(size_of::<BindingIdentifier>() == 20);
    assert!(align_of::<BindingIdentifier>() == 4);
    assert!(offset_of!(BindingIdentifier, span) == 0);
    assert!(offset_of!(BindingIdentifier, name) == 8);
    assert!(offset_of!(BindingIdentifier, symbol_id) == 16);

    // Padding: 0 bytes
    assert!(size_of::<LabelIdentifier>() == 16);
    assert!(align_of::<LabelIdentifier>() == 4);
    assert!(offset_of!(LabelIdentifier, span) == 0);
    assert!(offset_of!(LabelIdentifier, name) == 8);

    // Padding: 0 bytes
    assert!(size_of::<ThisExpression>() == 8);
    assert!(align_of::<ThisExpression>() == 4);
    assert!(offset_of!(ThisExpression, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ArrayExpression>() == 24);
    assert!(align_of::<ArrayExpression>() == 4);
    assert!(offset_of!(ArrayExpression, span) == 0);
    assert!(offset_of!(ArrayExpression, elements) == 8);

    assert!(size_of::<ArrayExpressionElement>() == 12);
    assert!(align_of::<ArrayExpressionElement>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<Elision>() == 8);
    assert!(align_of::<Elision>() == 4);
    assert!(offset_of!(Elision, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ObjectExpression>() == 24);
    assert!(align_of::<ObjectExpression>() == 4);
    assert!(offset_of!(ObjectExpression, span) == 0);
    assert!(offset_of!(ObjectExpression, properties) == 8);

    assert!(size_of::<ObjectPropertyKind>() == 8);
    assert!(align_of::<ObjectPropertyKind>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ObjectProperty>() == 28);
    assert!(align_of::<ObjectProperty>() == 4);
    assert!(offset_of!(ObjectProperty, span) == 0);
    assert!(offset_of!(ObjectProperty, kind) == 24);
    assert!(offset_of!(ObjectProperty, key) == 8);
    assert!(offset_of!(ObjectProperty, value) == 16);
    assert!(offset_of!(ObjectProperty, method) == 25);
    assert!(offset_of!(ObjectProperty, shorthand) == 26);
    assert!(offset_of!(ObjectProperty, computed) == 27);

    assert!(size_of::<PropertyKey>() == 8);
    assert!(align_of::<PropertyKey>() == 4);

    assert!(size_of::<PropertyKind>() == 1);
    assert!(align_of::<PropertyKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TemplateLiteral>() == 40);
    assert!(align_of::<TemplateLiteral>() == 4);
    assert!(offset_of!(TemplateLiteral, span) == 0);
    assert!(offset_of!(TemplateLiteral, quasis) == 8);
    assert!(offset_of!(TemplateLiteral, expressions) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TaggedTemplateExpression>() == 60);
    assert!(align_of::<TaggedTemplateExpression>() == 4);
    assert!(offset_of!(TaggedTemplateExpression, span) == 0);
    assert!(offset_of!(TaggedTemplateExpression, tag) == 8);
    assert!(offset_of!(TaggedTemplateExpression, type_arguments) == 16);
    assert!(offset_of!(TaggedTemplateExpression, quasi) == 20);

    // Padding: 2 bytes
    assert!(size_of::<TemplateElement>() == 28);
    assert!(align_of::<TemplateElement>() == 4);
    assert!(offset_of!(TemplateElement, span) == 0);
    assert!(offset_of!(TemplateElement, value) == 8);
    assert!(offset_of!(TemplateElement, tail) == 24);
    assert!(offset_of!(TemplateElement, lone_surrogates) == 25);

    // Padding: 0 bytes
    assert!(size_of::<TemplateElementValue>() == 16);
    assert!(align_of::<TemplateElementValue>() == 4);
    assert!(offset_of!(TemplateElementValue, raw) == 0);
    assert!(offset_of!(TemplateElementValue, cooked) == 8);

    assert!(size_of::<MemberExpression>() == 8);
    assert!(align_of::<MemberExpression>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ComputedMemberExpression>() == 28);
    assert!(align_of::<ComputedMemberExpression>() == 4);
    assert!(offset_of!(ComputedMemberExpression, span) == 0);
    assert!(offset_of!(ComputedMemberExpression, object) == 8);
    assert!(offset_of!(ComputedMemberExpression, expression) == 16);
    assert!(offset_of!(ComputedMemberExpression, optional) == 24);

    // Padding: 3 bytes
    assert!(size_of::<StaticMemberExpression>() == 36);
    assert!(align_of::<StaticMemberExpression>() == 4);
    assert!(offset_of!(StaticMemberExpression, span) == 0);
    assert!(offset_of!(StaticMemberExpression, object) == 8);
    assert!(offset_of!(StaticMemberExpression, property) == 16);
    assert!(offset_of!(StaticMemberExpression, optional) == 32);

    // Padding: 3 bytes
    assert!(size_of::<PrivateFieldExpression>() == 36);
    assert!(align_of::<PrivateFieldExpression>() == 4);
    assert!(offset_of!(PrivateFieldExpression, span) == 0);
    assert!(offset_of!(PrivateFieldExpression, object) == 8);
    assert!(offset_of!(PrivateFieldExpression, field) == 16);
    assert!(offset_of!(PrivateFieldExpression, optional) == 32);

    // Padding: 2 bytes
    assert!(size_of::<CallExpression>() == 40);
    assert!(align_of::<CallExpression>() == 4);
    assert!(offset_of!(CallExpression, span) == 0);
    assert!(offset_of!(CallExpression, callee) == 8);
    assert!(offset_of!(CallExpression, type_arguments) == 16);
    assert!(offset_of!(CallExpression, arguments) == 20);
    assert!(offset_of!(CallExpression, optional) == 36);
    assert!(offset_of!(CallExpression, pure) == 37);

    // Padding: 3 bytes
    assert!(size_of::<NewExpression>() == 40);
    assert!(align_of::<NewExpression>() == 4);
    assert!(offset_of!(NewExpression, span) == 0);
    assert!(offset_of!(NewExpression, callee) == 8);
    assert!(offset_of!(NewExpression, type_arguments) == 16);
    assert!(offset_of!(NewExpression, arguments) == 20);
    assert!(offset_of!(NewExpression, pure) == 36);

    // Padding: 0 bytes
    assert!(size_of::<MetaProperty>() == 40);
    assert!(align_of::<MetaProperty>() == 4);
    assert!(offset_of!(MetaProperty, span) == 0);
    assert!(offset_of!(MetaProperty, meta) == 8);
    assert!(offset_of!(MetaProperty, property) == 24);

    // Padding: 0 bytes
    assert!(size_of::<SpreadElement>() == 16);
    assert!(align_of::<SpreadElement>() == 4);
    assert!(offset_of!(SpreadElement, span) == 0);
    assert!(offset_of!(SpreadElement, argument) == 8);

    assert!(size_of::<Argument>() == 8);
    assert!(align_of::<Argument>() == 4);

    // Padding: 2 bytes
    assert!(size_of::<UpdateExpression>() == 20);
    assert!(align_of::<UpdateExpression>() == 4);
    assert!(offset_of!(UpdateExpression, span) == 0);
    assert!(offset_of!(UpdateExpression, operator) == 16);
    assert!(offset_of!(UpdateExpression, prefix) == 17);
    assert!(offset_of!(UpdateExpression, argument) == 8);

    // Padding: 3 bytes
    assert!(size_of::<UnaryExpression>() == 20);
    assert!(align_of::<UnaryExpression>() == 4);
    assert!(offset_of!(UnaryExpression, span) == 0);
    assert!(offset_of!(UnaryExpression, operator) == 16);
    assert!(offset_of!(UnaryExpression, argument) == 8);

    // Padding: 3 bytes
    assert!(size_of::<BinaryExpression>() == 28);
    assert!(align_of::<BinaryExpression>() == 4);
    assert!(offset_of!(BinaryExpression, span) == 0);
    assert!(offset_of!(BinaryExpression, left) == 8);
    assert!(offset_of!(BinaryExpression, operator) == 24);
    assert!(offset_of!(BinaryExpression, right) == 16);

    // Padding: 0 bytes
    assert!(size_of::<PrivateInExpression>() == 32);
    assert!(align_of::<PrivateInExpression>() == 4);
    assert!(offset_of!(PrivateInExpression, span) == 0);
    assert!(offset_of!(PrivateInExpression, left) == 8);
    assert!(offset_of!(PrivateInExpression, right) == 24);

    // Padding: 3 bytes
    assert!(size_of::<LogicalExpression>() == 28);
    assert!(align_of::<LogicalExpression>() == 4);
    assert!(offset_of!(LogicalExpression, span) == 0);
    assert!(offset_of!(LogicalExpression, left) == 8);
    assert!(offset_of!(LogicalExpression, operator) == 24);
    assert!(offset_of!(LogicalExpression, right) == 16);

    // Padding: 0 bytes
    assert!(size_of::<ConditionalExpression>() == 32);
    assert!(align_of::<ConditionalExpression>() == 4);
    assert!(offset_of!(ConditionalExpression, span) == 0);
    assert!(offset_of!(ConditionalExpression, test) == 8);
    assert!(offset_of!(ConditionalExpression, consequent) == 16);
    assert!(offset_of!(ConditionalExpression, alternate) == 24);

    // Padding: 3 bytes
    assert!(size_of::<AssignmentExpression>() == 28);
    assert!(align_of::<AssignmentExpression>() == 4);
    assert!(offset_of!(AssignmentExpression, span) == 0);
    assert!(offset_of!(AssignmentExpression, operator) == 24);
    assert!(offset_of!(AssignmentExpression, left) == 8);
    assert!(offset_of!(AssignmentExpression, right) == 16);

    assert!(size_of::<AssignmentTarget>() == 8);
    assert!(align_of::<AssignmentTarget>() == 4);

    assert!(size_of::<SimpleAssignmentTarget>() == 8);
    assert!(align_of::<SimpleAssignmentTarget>() == 4);

    assert!(size_of::<AssignmentTargetPattern>() == 8);
    assert!(align_of::<AssignmentTargetPattern>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ArrayAssignmentTarget>() == 28);
    assert!(align_of::<ArrayAssignmentTarget>() == 4);
    assert!(offset_of!(ArrayAssignmentTarget, span) == 0);
    assert!(offset_of!(ArrayAssignmentTarget, elements) == 8);
    assert!(offset_of!(ArrayAssignmentTarget, rest) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ObjectAssignmentTarget>() == 28);
    assert!(align_of::<ObjectAssignmentTarget>() == 4);
    assert!(offset_of!(ObjectAssignmentTarget, span) == 0);
    assert!(offset_of!(ObjectAssignmentTarget, properties) == 8);
    assert!(offset_of!(ObjectAssignmentTarget, rest) == 24);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetRest>() == 16);
    assert!(align_of::<AssignmentTargetRest>() == 4);
    assert!(offset_of!(AssignmentTargetRest, span) == 0);
    assert!(offset_of!(AssignmentTargetRest, target) == 8);

    assert!(size_of::<AssignmentTargetMaybeDefault>() == 8);
    assert!(align_of::<AssignmentTargetMaybeDefault>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetWithDefault>() == 24);
    assert!(align_of::<AssignmentTargetWithDefault>() == 4);
    assert!(offset_of!(AssignmentTargetWithDefault, span) == 0);
    assert!(offset_of!(AssignmentTargetWithDefault, binding) == 8);
    assert!(offset_of!(AssignmentTargetWithDefault, init) == 16);

    assert!(size_of::<AssignmentTargetProperty>() == 8);
    assert!(align_of::<AssignmentTargetProperty>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<AssignmentTargetPropertyIdentifier>() == 36);
    assert!(align_of::<AssignmentTargetPropertyIdentifier>() == 4);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, span) == 0);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, binding) == 8);
    assert!(offset_of!(AssignmentTargetPropertyIdentifier, init) == 28);

    // Padding: 3 bytes
    assert!(size_of::<AssignmentTargetPropertyProperty>() == 28);
    assert!(align_of::<AssignmentTargetPropertyProperty>() == 4);
    assert!(offset_of!(AssignmentTargetPropertyProperty, span) == 0);
    assert!(offset_of!(AssignmentTargetPropertyProperty, name) == 8);
    assert!(offset_of!(AssignmentTargetPropertyProperty, binding) == 16);
    assert!(offset_of!(AssignmentTargetPropertyProperty, computed) == 24);

    // Padding: 0 bytes
    assert!(size_of::<SequenceExpression>() == 24);
    assert!(align_of::<SequenceExpression>() == 4);
    assert!(offset_of!(SequenceExpression, span) == 0);
    assert!(offset_of!(SequenceExpression, expressions) == 8);

    // Padding: 0 bytes
    assert!(size_of::<Super>() == 8);
    assert!(align_of::<Super>() == 4);
    assert!(offset_of!(Super, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<AwaitExpression>() == 16);
    assert!(align_of::<AwaitExpression>() == 4);
    assert!(offset_of!(AwaitExpression, span) == 0);
    assert!(offset_of!(AwaitExpression, argument) == 8);

    // Padding: 0 bytes
    assert!(size_of::<ChainExpression>() == 16);
    assert!(align_of::<ChainExpression>() == 4);
    assert!(offset_of!(ChainExpression, span) == 0);
    assert!(offset_of!(ChainExpression, expression) == 8);

    assert!(size_of::<ChainElement>() == 8);
    assert!(align_of::<ChainElement>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ParenthesizedExpression>() == 16);
    assert!(align_of::<ParenthesizedExpression>() == 4);
    assert!(offset_of!(ParenthesizedExpression, span) == 0);
    assert!(offset_of!(ParenthesizedExpression, expression) == 8);

    assert!(size_of::<Statement>() == 8);
    assert!(align_of::<Statement>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<Directive>() == 44);
    assert!(align_of::<Directive>() == 4);
    assert!(offset_of!(Directive, span) == 0);
    assert!(offset_of!(Directive, expression) == 8);
    assert!(offset_of!(Directive, directive) == 36);

    // Padding: 0 bytes
    assert!(size_of::<Hashbang>() == 16);
    assert!(align_of::<Hashbang>() == 4);
    assert!(offset_of!(Hashbang, span) == 0);
    assert!(offset_of!(Hashbang, value) == 8);

    // Padding: 0 bytes
    assert!(size_of::<BlockStatement>() == 28);
    assert!(align_of::<BlockStatement>() == 4);
    assert!(offset_of!(BlockStatement, span) == 0);
    assert!(offset_of!(BlockStatement, body) == 8);
    assert!(offset_of!(BlockStatement, scope_id) == 24);

    assert!(size_of::<Declaration>() == 8);
    assert!(align_of::<Declaration>() == 4);

    // Padding: 2 bytes
    assert!(size_of::<VariableDeclaration>() == 28);
    assert!(align_of::<VariableDeclaration>() == 4);
    assert!(offset_of!(VariableDeclaration, span) == 0);
    assert!(offset_of!(VariableDeclaration, kind) == 24);
    assert!(offset_of!(VariableDeclaration, declarations) == 8);
    assert!(offset_of!(VariableDeclaration, declare) == 25);

    assert!(size_of::<VariableDeclarationKind>() == 1);
    assert!(align_of::<VariableDeclarationKind>() == 1);

    // Padding: 2 bytes
    assert!(size_of::<VariableDeclarator>() == 36);
    assert!(align_of::<VariableDeclarator>() == 4);
    assert!(offset_of!(VariableDeclarator, span) == 0);
    assert!(offset_of!(VariableDeclarator, kind) == 32);
    assert!(offset_of!(VariableDeclarator, id) == 8);
    assert!(offset_of!(VariableDeclarator, init) == 24);
    assert!(offset_of!(VariableDeclarator, definite) == 33);

    // Padding: 0 bytes
    assert!(size_of::<EmptyStatement>() == 8);
    assert!(align_of::<EmptyStatement>() == 4);
    assert!(offset_of!(EmptyStatement, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<ExpressionStatement>() == 16);
    assert!(align_of::<ExpressionStatement>() == 4);
    assert!(offset_of!(ExpressionStatement, span) == 0);
    assert!(offset_of!(ExpressionStatement, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<IfStatement>() == 32);
    assert!(align_of::<IfStatement>() == 4);
    assert!(offset_of!(IfStatement, span) == 0);
    assert!(offset_of!(IfStatement, test) == 8);
    assert!(offset_of!(IfStatement, consequent) == 16);
    assert!(offset_of!(IfStatement, alternate) == 24);

    // Padding: 0 bytes
    assert!(size_of::<DoWhileStatement>() == 24);
    assert!(align_of::<DoWhileStatement>() == 4);
    assert!(offset_of!(DoWhileStatement, span) == 0);
    assert!(offset_of!(DoWhileStatement, body) == 8);
    assert!(offset_of!(DoWhileStatement, test) == 16);

    // Padding: 0 bytes
    assert!(size_of::<WhileStatement>() == 24);
    assert!(align_of::<WhileStatement>() == 4);
    assert!(offset_of!(WhileStatement, span) == 0);
    assert!(offset_of!(WhileStatement, test) == 8);
    assert!(offset_of!(WhileStatement, body) == 16);

    // Padding: 0 bytes
    assert!(size_of::<ForStatement>() == 44);
    assert!(align_of::<ForStatement>() == 4);
    assert!(offset_of!(ForStatement, span) == 0);
    assert!(offset_of!(ForStatement, init) == 8);
    assert!(offset_of!(ForStatement, test) == 16);
    assert!(offset_of!(ForStatement, update) == 24);
    assert!(offset_of!(ForStatement, body) == 32);
    assert!(offset_of!(ForStatement, scope_id) == 40);

    assert!(size_of::<ForStatementInit>() == 8);
    assert!(align_of::<ForStatementInit>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ForInStatement>() == 36);
    assert!(align_of::<ForInStatement>() == 4);
    assert!(offset_of!(ForInStatement, span) == 0);
    assert!(offset_of!(ForInStatement, left) == 8);
    assert!(offset_of!(ForInStatement, right) == 16);
    assert!(offset_of!(ForInStatement, body) == 24);
    assert!(offset_of!(ForInStatement, scope_id) == 32);

    assert!(size_of::<ForStatementLeft>() == 8);
    assert!(align_of::<ForStatementLeft>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ForOfStatement>() == 40);
    assert!(align_of::<ForOfStatement>() == 4);
    assert!(offset_of!(ForOfStatement, span) == 0);
    assert!(offset_of!(ForOfStatement, r#await) == 36);
    assert!(offset_of!(ForOfStatement, left) == 8);
    assert!(offset_of!(ForOfStatement, right) == 16);
    assert!(offset_of!(ForOfStatement, body) == 24);
    assert!(offset_of!(ForOfStatement, scope_id) == 32);

    // Padding: 0 bytes
    assert!(size_of::<ContinueStatement>() == 24);
    assert!(align_of::<ContinueStatement>() == 4);
    assert!(offset_of!(ContinueStatement, span) == 0);
    assert!(offset_of!(ContinueStatement, label) == 8);

    // Padding: 0 bytes
    assert!(size_of::<BreakStatement>() == 24);
    assert!(align_of::<BreakStatement>() == 4);
    assert!(offset_of!(BreakStatement, span) == 0);
    assert!(offset_of!(BreakStatement, label) == 8);

    // Padding: 0 bytes
    assert!(size_of::<ReturnStatement>() == 16);
    assert!(align_of::<ReturnStatement>() == 4);
    assert!(offset_of!(ReturnStatement, span) == 0);
    assert!(offset_of!(ReturnStatement, argument) == 8);

    // Padding: 0 bytes
    assert!(size_of::<WithStatement>() == 24);
    assert!(align_of::<WithStatement>() == 4);
    assert!(offset_of!(WithStatement, span) == 0);
    assert!(offset_of!(WithStatement, object) == 8);
    assert!(offset_of!(WithStatement, body) == 16);

    // Padding: 0 bytes
    assert!(size_of::<SwitchStatement>() == 36);
    assert!(align_of::<SwitchStatement>() == 4);
    assert!(offset_of!(SwitchStatement, span) == 0);
    assert!(offset_of!(SwitchStatement, discriminant) == 8);
    assert!(offset_of!(SwitchStatement, cases) == 16);
    assert!(offset_of!(SwitchStatement, scope_id) == 32);

    // Padding: 0 bytes
    assert!(size_of::<SwitchCase>() == 32);
    assert!(align_of::<SwitchCase>() == 4);
    assert!(offset_of!(SwitchCase, span) == 0);
    assert!(offset_of!(SwitchCase, test) == 8);
    assert!(offset_of!(SwitchCase, consequent) == 16);

    // Padding: 0 bytes
    assert!(size_of::<LabeledStatement>() == 32);
    assert!(align_of::<LabeledStatement>() == 4);
    assert!(offset_of!(LabeledStatement, span) == 0);
    assert!(offset_of!(LabeledStatement, label) == 8);
    assert!(offset_of!(LabeledStatement, body) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ThrowStatement>() == 16);
    assert!(align_of::<ThrowStatement>() == 4);
    assert!(offset_of!(ThrowStatement, span) == 0);
    assert!(offset_of!(ThrowStatement, argument) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TryStatement>() == 20);
    assert!(align_of::<TryStatement>() == 4);
    assert!(offset_of!(TryStatement, span) == 0);
    assert!(offset_of!(TryStatement, block) == 8);
    assert!(offset_of!(TryStatement, handler) == 12);
    assert!(offset_of!(TryStatement, finalizer) == 16);

    // Padding: 0 bytes
    assert!(size_of::<CatchClause>() == 40);
    assert!(align_of::<CatchClause>() == 4);
    assert!(offset_of!(CatchClause, span) == 0);
    assert!(offset_of!(CatchClause, param) == 8);
    assert!(offset_of!(CatchClause, body) == 32);
    assert!(offset_of!(CatchClause, scope_id) == 36);

    // Padding: 0 bytes
    assert!(size_of::<CatchParameter>() == 24);
    assert!(align_of::<CatchParameter>() == 4);
    assert!(offset_of!(CatchParameter, span) == 0);
    assert!(offset_of!(CatchParameter, pattern) == 8);

    // Padding: 0 bytes
    assert!(size_of::<DebuggerStatement>() == 8);
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
    assert!(size_of::<AssignmentPattern>() == 32);
    assert!(align_of::<AssignmentPattern>() == 4);
    assert!(offset_of!(AssignmentPattern, span) == 0);
    assert!(offset_of!(AssignmentPattern, left) == 8);
    assert!(offset_of!(AssignmentPattern, right) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ObjectPattern>() == 28);
    assert!(align_of::<ObjectPattern>() == 4);
    assert!(offset_of!(ObjectPattern, span) == 0);
    assert!(offset_of!(ObjectPattern, properties) == 8);
    assert!(offset_of!(ObjectPattern, rest) == 24);

    // Padding: 2 bytes
    assert!(size_of::<BindingProperty>() == 36);
    assert!(align_of::<BindingProperty>() == 4);
    assert!(offset_of!(BindingProperty, span) == 0);
    assert!(offset_of!(BindingProperty, key) == 8);
    assert!(offset_of!(BindingProperty, value) == 16);
    assert!(offset_of!(BindingProperty, shorthand) == 32);
    assert!(offset_of!(BindingProperty, computed) == 33);

    // Padding: 0 bytes
    assert!(size_of::<ArrayPattern>() == 28);
    assert!(align_of::<ArrayPattern>() == 4);
    assert!(offset_of!(ArrayPattern, span) == 0);
    assert!(offset_of!(ArrayPattern, elements) == 8);
    assert!(offset_of!(ArrayPattern, rest) == 24);

    // Padding: 0 bytes
    assert!(size_of::<BindingRestElement>() == 24);
    assert!(align_of::<BindingRestElement>() == 4);
    assert!(offset_of!(BindingRestElement, span) == 0);
    assert!(offset_of!(BindingRestElement, argument) == 8);

    // Padding: 2 bytes
    assert!(size_of::<Function>() == 60);
    assert!(align_of::<Function>() == 4);
    assert!(offset_of!(Function, span) == 0);
    assert!(offset_of!(Function, r#type) == 52);
    assert!(offset_of!(Function, id) == 8);
    assert!(offset_of!(Function, generator) == 53);
    assert!(offset_of!(Function, r#async) == 54);
    assert!(offset_of!(Function, declare) == 55);
    assert!(offset_of!(Function, type_parameters) == 28);
    assert!(offset_of!(Function, this_param) == 32);
    assert!(offset_of!(Function, params) == 36);
    assert!(offset_of!(Function, return_type) == 40);
    assert!(offset_of!(Function, body) == 44);
    assert!(offset_of!(Function, scope_id) == 48);
    assert!(offset_of!(Function, pure) == 56);
    assert!(offset_of!(Function, pife) == 57);

    assert!(size_of::<FunctionType>() == 1);
    assert!(align_of::<FunctionType>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<FormalParameters>() == 32);
    assert!(align_of::<FormalParameters>() == 4);
    assert!(offset_of!(FormalParameters, span) == 0);
    assert!(offset_of!(FormalParameters, kind) == 28);
    assert!(offset_of!(FormalParameters, items) == 8);
    assert!(offset_of!(FormalParameters, rest) == 24);

    // Padding: 1 bytes
    assert!(size_of::<FormalParameter>() == 44);
    assert!(align_of::<FormalParameter>() == 4);
    assert!(offset_of!(FormalParameter, span) == 0);
    assert!(offset_of!(FormalParameter, decorators) == 8);
    assert!(offset_of!(FormalParameter, pattern) == 24);
    assert!(offset_of!(FormalParameter, accessibility) == 40);
    assert!(offset_of!(FormalParameter, readonly) == 41);
    assert!(offset_of!(FormalParameter, r#override) == 42);

    assert!(size_of::<FormalParameterKind>() == 1);
    assert!(align_of::<FormalParameterKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<FunctionBody>() == 40);
    assert!(align_of::<FunctionBody>() == 4);
    assert!(offset_of!(FunctionBody, span) == 0);
    assert!(offset_of!(FunctionBody, directives) == 8);
    assert!(offset_of!(FunctionBody, statements) == 24);

    // Padding: 0 bytes
    assert!(size_of::<ArrowFunctionExpression>() == 32);
    assert!(align_of::<ArrowFunctionExpression>() == 4);
    assert!(offset_of!(ArrowFunctionExpression, span) == 0);
    assert!(offset_of!(ArrowFunctionExpression, expression) == 28);
    assert!(offset_of!(ArrowFunctionExpression, r#async) == 29);
    assert!(offset_of!(ArrowFunctionExpression, type_parameters) == 8);
    assert!(offset_of!(ArrowFunctionExpression, params) == 12);
    assert!(offset_of!(ArrowFunctionExpression, return_type) == 16);
    assert!(offset_of!(ArrowFunctionExpression, body) == 20);
    assert!(offset_of!(ArrowFunctionExpression, scope_id) == 24);
    assert!(offset_of!(ArrowFunctionExpression, pure) == 30);
    assert!(offset_of!(ArrowFunctionExpression, pife) == 31);

    // Padding: 3 bytes
    assert!(size_of::<YieldExpression>() == 20);
    assert!(align_of::<YieldExpression>() == 4);
    assert!(offset_of!(YieldExpression, span) == 0);
    assert!(offset_of!(YieldExpression, delegate) == 16);
    assert!(offset_of!(YieldExpression, argument) == 8);

    // Padding: 1 bytes
    assert!(size_of::<Class>() == 88);
    assert!(align_of::<Class>() == 4);
    assert!(offset_of!(Class, span) == 0);
    assert!(offset_of!(Class, r#type) == 84);
    assert!(offset_of!(Class, decorators) == 8);
    assert!(offset_of!(Class, id) == 24);
    assert!(offset_of!(Class, type_parameters) == 44);
    assert!(offset_of!(Class, super_class) == 48);
    assert!(offset_of!(Class, super_type_arguments) == 56);
    assert!(offset_of!(Class, implements) == 60);
    assert!(offset_of!(Class, body) == 76);
    assert!(offset_of!(Class, r#abstract) == 85);
    assert!(offset_of!(Class, declare) == 86);
    assert!(offset_of!(Class, scope_id) == 80);

    assert!(size_of::<ClassType>() == 1);
    assert!(align_of::<ClassType>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ClassBody>() == 24);
    assert!(align_of::<ClassBody>() == 4);
    assert!(offset_of!(ClassBody, span) == 0);
    assert!(offset_of!(ClassBody, body) == 8);

    assert!(size_of::<ClassElement>() == 8);
    assert!(align_of::<ClassElement>() == 4);

    // Padding: 1 bytes
    assert!(size_of::<MethodDefinition>() == 44);
    assert!(align_of::<MethodDefinition>() == 4);
    assert!(offset_of!(MethodDefinition, span) == 0);
    assert!(offset_of!(MethodDefinition, r#type) == 36);
    assert!(offset_of!(MethodDefinition, decorators) == 8);
    assert!(offset_of!(MethodDefinition, key) == 24);
    assert!(offset_of!(MethodDefinition, value) == 32);
    assert!(offset_of!(MethodDefinition, kind) == 37);
    assert!(offset_of!(MethodDefinition, computed) == 38);
    assert!(offset_of!(MethodDefinition, r#static) == 39);
    assert!(offset_of!(MethodDefinition, r#override) == 40);
    assert!(offset_of!(MethodDefinition, optional) == 41);
    assert!(offset_of!(MethodDefinition, accessibility) == 42);

    assert!(size_of::<MethodDefinitionType>() == 1);
    assert!(align_of::<MethodDefinitionType>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<PropertyDefinition>() == 56);
    assert!(align_of::<PropertyDefinition>() == 4);
    assert!(offset_of!(PropertyDefinition, span) == 0);
    assert!(offset_of!(PropertyDefinition, r#type) == 44);
    assert!(offset_of!(PropertyDefinition, decorators) == 8);
    assert!(offset_of!(PropertyDefinition, key) == 24);
    assert!(offset_of!(PropertyDefinition, type_annotation) == 32);
    assert!(offset_of!(PropertyDefinition, value) == 36);
    assert!(offset_of!(PropertyDefinition, computed) == 45);
    assert!(offset_of!(PropertyDefinition, r#static) == 46);
    assert!(offset_of!(PropertyDefinition, declare) == 47);
    assert!(offset_of!(PropertyDefinition, r#override) == 48);
    assert!(offset_of!(PropertyDefinition, optional) == 49);
    assert!(offset_of!(PropertyDefinition, definite) == 50);
    assert!(offset_of!(PropertyDefinition, readonly) == 51);
    assert!(offset_of!(PropertyDefinition, accessibility) == 52);

    assert!(size_of::<PropertyDefinitionType>() == 1);
    assert!(align_of::<PropertyDefinitionType>() == 1);

    assert!(size_of::<MethodDefinitionKind>() == 1);
    assert!(align_of::<MethodDefinitionKind>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<PrivateIdentifier>() == 16);
    assert!(align_of::<PrivateIdentifier>() == 4);
    assert!(offset_of!(PrivateIdentifier, span) == 0);
    assert!(offset_of!(PrivateIdentifier, name) == 8);

    // Padding: 0 bytes
    assert!(size_of::<StaticBlock>() == 28);
    assert!(align_of::<StaticBlock>() == 4);
    assert!(offset_of!(StaticBlock, span) == 0);
    assert!(offset_of!(StaticBlock, body) == 8);
    assert!(offset_of!(StaticBlock, scope_id) == 24);

    assert!(size_of::<ModuleDeclaration>() == 8);
    assert!(align_of::<ModuleDeclaration>() == 4);

    assert!(size_of::<AccessorPropertyType>() == 1);
    assert!(align_of::<AccessorPropertyType>() == 1);

    // Padding: 2 bytes
    assert!(size_of::<AccessorProperty>() == 52);
    assert!(align_of::<AccessorProperty>() == 4);
    assert!(offset_of!(AccessorProperty, span) == 0);
    assert!(offset_of!(AccessorProperty, r#type) == 44);
    assert!(offset_of!(AccessorProperty, decorators) == 8);
    assert!(offset_of!(AccessorProperty, key) == 24);
    assert!(offset_of!(AccessorProperty, type_annotation) == 32);
    assert!(offset_of!(AccessorProperty, value) == 36);
    assert!(offset_of!(AccessorProperty, computed) == 45);
    assert!(offset_of!(AccessorProperty, r#static) == 46);
    assert!(offset_of!(AccessorProperty, r#override) == 47);
    assert!(offset_of!(AccessorProperty, definite) == 48);
    assert!(offset_of!(AccessorProperty, accessibility) == 49);

    // Padding: 3 bytes
    assert!(size_of::<ImportExpression>() == 28);
    assert!(align_of::<ImportExpression>() == 4);
    assert!(offset_of!(ImportExpression, span) == 0);
    assert!(offset_of!(ImportExpression, source) == 8);
    assert!(offset_of!(ImportExpression, options) == 16);
    assert!(offset_of!(ImportExpression, phase) == 24);

    // Padding: 2 bytes
    assert!(size_of::<ImportDeclaration>() == 60);
    assert!(align_of::<ImportDeclaration>() == 4);
    assert!(offset_of!(ImportDeclaration, span) == 0);
    assert!(offset_of!(ImportDeclaration, specifiers) == 8);
    assert!(offset_of!(ImportDeclaration, source) == 24);
    assert!(offset_of!(ImportDeclaration, phase) == 56);
    assert!(offset_of!(ImportDeclaration, with_clause) == 52);
    assert!(offset_of!(ImportDeclaration, import_kind) == 57);

    assert!(size_of::<ImportPhase>() == 1);
    assert!(align_of::<ImportPhase>() == 1);

    assert!(size_of::<ImportDeclarationSpecifier>() == 8);
    assert!(align_of::<ImportDeclarationSpecifier>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ImportSpecifier>() == 64);
    assert!(align_of::<ImportSpecifier>() == 4);
    assert!(offset_of!(ImportSpecifier, span) == 0);
    assert!(offset_of!(ImportSpecifier, imported) == 8);
    assert!(offset_of!(ImportSpecifier, local) == 40);
    assert!(offset_of!(ImportSpecifier, import_kind) == 60);

    // Padding: 0 bytes
    assert!(size_of::<ImportDefaultSpecifier>() == 28);
    assert!(align_of::<ImportDefaultSpecifier>() == 4);
    assert!(offset_of!(ImportDefaultSpecifier, span) == 0);
    assert!(offset_of!(ImportDefaultSpecifier, local) == 8);

    // Padding: 0 bytes
    assert!(size_of::<ImportNamespaceSpecifier>() == 28);
    assert!(align_of::<ImportNamespaceSpecifier>() == 4);
    assert!(offset_of!(ImportNamespaceSpecifier, span) == 0);
    assert!(offset_of!(ImportNamespaceSpecifier, local) == 8);

    // Padding: 3 bytes
    assert!(size_of::<WithClause>() == 28);
    assert!(align_of::<WithClause>() == 4);
    assert!(offset_of!(WithClause, span) == 0);
    assert!(offset_of!(WithClause, keyword) == 24);
    assert!(offset_of!(WithClause, with_entries) == 8);

    assert!(size_of::<WithClauseKeyword>() == 1);
    assert!(align_of::<WithClauseKeyword>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ImportAttribute>() == 68);
    assert!(align_of::<ImportAttribute>() == 4);
    assert!(offset_of!(ImportAttribute, span) == 0);
    assert!(offset_of!(ImportAttribute, key) == 8);
    assert!(offset_of!(ImportAttribute, value) == 40);

    assert!(size_of::<ImportAttributeKey>() == 32);
    assert!(align_of::<ImportAttributeKey>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ExportNamedDeclaration>() == 68);
    assert!(align_of::<ExportNamedDeclaration>() == 4);
    assert!(offset_of!(ExportNamedDeclaration, span) == 0);
    assert!(offset_of!(ExportNamedDeclaration, declaration) == 8);
    assert!(offset_of!(ExportNamedDeclaration, specifiers) == 16);
    assert!(offset_of!(ExportNamedDeclaration, source) == 32);
    assert!(offset_of!(ExportNamedDeclaration, export_kind) == 64);
    assert!(offset_of!(ExportNamedDeclaration, with_clause) == 60);

    // Padding: 0 bytes
    assert!(size_of::<ExportDefaultDeclaration>() == 48);
    assert!(align_of::<ExportDefaultDeclaration>() == 4);
    assert!(offset_of!(ExportDefaultDeclaration, span) == 0);
    assert!(offset_of!(ExportDefaultDeclaration, exported) == 8);
    assert!(offset_of!(ExportDefaultDeclaration, declaration) == 40);

    // Padding: 3 bytes
    assert!(size_of::<ExportAllDeclaration>() == 76);
    assert!(align_of::<ExportAllDeclaration>() == 4);
    assert!(offset_of!(ExportAllDeclaration, span) == 0);
    assert!(offset_of!(ExportAllDeclaration, exported) == 8);
    assert!(offset_of!(ExportAllDeclaration, source) == 40);
    assert!(offset_of!(ExportAllDeclaration, with_clause) == 68);
    assert!(offset_of!(ExportAllDeclaration, export_kind) == 72);

    // Padding: 3 bytes
    assert!(size_of::<ExportSpecifier>() == 76);
    assert!(align_of::<ExportSpecifier>() == 4);
    assert!(offset_of!(ExportSpecifier, span) == 0);
    assert!(offset_of!(ExportSpecifier, local) == 8);
    assert!(offset_of!(ExportSpecifier, exported) == 40);
    assert!(offset_of!(ExportSpecifier, export_kind) == 72);

    assert!(size_of::<ExportDefaultDeclarationKind>() == 8);
    assert!(align_of::<ExportDefaultDeclarationKind>() == 4);

    assert!(size_of::<ModuleExportName>() == 32);
    assert!(align_of::<ModuleExportName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<V8IntrinsicExpression>() == 40);
    assert!(align_of::<V8IntrinsicExpression>() == 4);
    assert!(offset_of!(V8IntrinsicExpression, span) == 0);
    assert!(offset_of!(V8IntrinsicExpression, name) == 8);
    assert!(offset_of!(V8IntrinsicExpression, arguments) == 24);

    // Padding: 3 bytes
    assert!(size_of::<BooleanLiteral>() == 12);
    assert!(align_of::<BooleanLiteral>() == 4);
    assert!(offset_of!(BooleanLiteral, span) == 0);
    assert!(offset_of!(BooleanLiteral, value) == 8);

    // Padding: 0 bytes
    assert!(size_of::<NullLiteral>() == 8);
    assert!(align_of::<NullLiteral>() == 4);
    assert!(offset_of!(NullLiteral, span) == 0);

    // Padding: 7 bytes
    assert!(size_of::<NumericLiteral>() == 32);
    assert!(align_of::<NumericLiteral>() == 8);
    assert!(offset_of!(NumericLiteral, span) == 0);
    assert!(offset_of!(NumericLiteral, value) == 8);
    assert!(offset_of!(NumericLiteral, raw) == 16);
    assert!(offset_of!(NumericLiteral, base) == 24);

    // Padding: 3 bytes
    assert!(size_of::<StringLiteral>() == 28);
    assert!(align_of::<StringLiteral>() == 4);
    assert!(offset_of!(StringLiteral, span) == 0);
    assert!(offset_of!(StringLiteral, value) == 8);
    assert!(offset_of!(StringLiteral, raw) == 16);
    assert!(offset_of!(StringLiteral, lone_surrogates) == 24);

    // Padding: 3 bytes
    assert!(size_of::<BigIntLiteral>() == 28);
    assert!(align_of::<BigIntLiteral>() == 4);
    assert!(offset_of!(BigIntLiteral, span) == 0);
    assert!(offset_of!(BigIntLiteral, value) == 8);
    assert!(offset_of!(BigIntLiteral, raw) == 16);
    assert!(offset_of!(BigIntLiteral, base) == 24);

    // Padding: 0 bytes
    assert!(size_of::<RegExpLiteral>() == 32);
    assert!(align_of::<RegExpLiteral>() == 4);
    assert!(offset_of!(RegExpLiteral, span) == 0);
    assert!(offset_of!(RegExpLiteral, regex) == 8);
    assert!(offset_of!(RegExpLiteral, raw) == 24);

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
    assert!(size_of::<JSXElement>() == 32);
    assert!(align_of::<JSXElement>() == 4);
    assert!(offset_of!(JSXElement, span) == 0);
    assert!(offset_of!(JSXElement, opening_element) == 8);
    assert!(offset_of!(JSXElement, children) == 12);
    assert!(offset_of!(JSXElement, closing_element) == 28);

    // Padding: 0 bytes
    assert!(size_of::<JSXOpeningElement>() == 36);
    assert!(align_of::<JSXOpeningElement>() == 4);
    assert!(offset_of!(JSXOpeningElement, span) == 0);
    assert!(offset_of!(JSXOpeningElement, name) == 8);
    assert!(offset_of!(JSXOpeningElement, type_arguments) == 16);
    assert!(offset_of!(JSXOpeningElement, attributes) == 20);

    // Padding: 0 bytes
    assert!(size_of::<JSXClosingElement>() == 16);
    assert!(align_of::<JSXClosingElement>() == 4);
    assert!(offset_of!(JSXClosingElement, span) == 0);
    assert!(offset_of!(JSXClosingElement, name) == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXFragment>() == 40);
    assert!(align_of::<JSXFragment>() == 4);
    assert!(offset_of!(JSXFragment, span) == 0);
    assert!(offset_of!(JSXFragment, opening_fragment) == 8);
    assert!(offset_of!(JSXFragment, children) == 16);
    assert!(offset_of!(JSXFragment, closing_fragment) == 32);

    // Padding: 0 bytes
    assert!(size_of::<JSXOpeningFragment>() == 8);
    assert!(align_of::<JSXOpeningFragment>() == 4);
    assert!(offset_of!(JSXOpeningFragment, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<JSXClosingFragment>() == 8);
    assert!(align_of::<JSXClosingFragment>() == 4);
    assert!(offset_of!(JSXClosingFragment, span) == 0);

    assert!(size_of::<JSXElementName>() == 8);
    assert!(align_of::<JSXElementName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXNamespacedName>() == 40);
    assert!(align_of::<JSXNamespacedName>() == 4);
    assert!(offset_of!(JSXNamespacedName, span) == 0);
    assert!(offset_of!(JSXNamespacedName, namespace) == 8);
    assert!(offset_of!(JSXNamespacedName, name) == 24);

    // Padding: 0 bytes
    assert!(size_of::<JSXMemberExpression>() == 32);
    assert!(align_of::<JSXMemberExpression>() == 4);
    assert!(offset_of!(JSXMemberExpression, span) == 0);
    assert!(offset_of!(JSXMemberExpression, object) == 8);
    assert!(offset_of!(JSXMemberExpression, property) == 16);

    assert!(size_of::<JSXMemberExpressionObject>() == 8);
    assert!(align_of::<JSXMemberExpressionObject>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXExpressionContainer>() == 20);
    assert!(align_of::<JSXExpressionContainer>() == 4);
    assert!(offset_of!(JSXExpressionContainer, span) == 0);
    assert!(offset_of!(JSXExpressionContainer, expression) == 8);

    assert!(size_of::<JSXExpression>() == 12);
    assert!(align_of::<JSXExpression>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXEmptyExpression>() == 8);
    assert!(align_of::<JSXEmptyExpression>() == 4);
    assert!(offset_of!(JSXEmptyExpression, span) == 0);

    assert!(size_of::<JSXAttributeItem>() == 8);
    assert!(align_of::<JSXAttributeItem>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXAttribute>() == 24);
    assert!(align_of::<JSXAttribute>() == 4);
    assert!(offset_of!(JSXAttribute, span) == 0);
    assert!(offset_of!(JSXAttribute, name) == 8);
    assert!(offset_of!(JSXAttribute, value) == 16);

    // Padding: 0 bytes
    assert!(size_of::<JSXSpreadAttribute>() == 16);
    assert!(align_of::<JSXSpreadAttribute>() == 4);
    assert!(offset_of!(JSXSpreadAttribute, span) == 0);
    assert!(offset_of!(JSXSpreadAttribute, argument) == 8);

    assert!(size_of::<JSXAttributeName>() == 8);
    assert!(align_of::<JSXAttributeName>() == 4);

    assert!(size_of::<JSXAttributeValue>() == 8);
    assert!(align_of::<JSXAttributeValue>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXIdentifier>() == 16);
    assert!(align_of::<JSXIdentifier>() == 4);
    assert!(offset_of!(JSXIdentifier, span) == 0);
    assert!(offset_of!(JSXIdentifier, name) == 8);

    assert!(size_of::<JSXChild>() == 8);
    assert!(align_of::<JSXChild>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<JSXSpreadChild>() == 16);
    assert!(align_of::<JSXSpreadChild>() == 4);
    assert!(offset_of!(JSXSpreadChild, span) == 0);
    assert!(offset_of!(JSXSpreadChild, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<JSXText>() == 24);
    assert!(align_of::<JSXText>() == 4);
    assert!(offset_of!(JSXText, span) == 0);
    assert!(offset_of!(JSXText, value) == 8);
    assert!(offset_of!(JSXText, raw) == 16);

    // Padding: 0 bytes
    assert!(size_of::<TSThisParameter>() == 20);
    assert!(align_of::<TSThisParameter>() == 4);
    assert!(offset_of!(TSThisParameter, span) == 0);
    assert!(offset_of!(TSThisParameter, this_span) == 8);
    assert!(offset_of!(TSThisParameter, type_annotation) == 16);

    // Padding: 2 bytes
    assert!(size_of::<TSEnumDeclaration>() == 60);
    assert!(align_of::<TSEnumDeclaration>() == 4);
    assert!(offset_of!(TSEnumDeclaration, span) == 0);
    assert!(offset_of!(TSEnumDeclaration, id) == 8);
    assert!(offset_of!(TSEnumDeclaration, body) == 28);
    assert!(offset_of!(TSEnumDeclaration, r#const) == 56);
    assert!(offset_of!(TSEnumDeclaration, declare) == 57);
    assert!(offset_of!(TSEnumDeclaration, scope_id) == 52);

    // Padding: 0 bytes
    assert!(size_of::<TSEnumBody>() == 24);
    assert!(align_of::<TSEnumBody>() == 4);
    assert!(offset_of!(TSEnumBody, span) == 0);
    assert!(offset_of!(TSEnumBody, members) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSEnumMember>() == 24);
    assert!(align_of::<TSEnumMember>() == 4);
    assert!(offset_of!(TSEnumMember, span) == 0);
    assert!(offset_of!(TSEnumMember, id) == 8);
    assert!(offset_of!(TSEnumMember, initializer) == 16);

    assert!(size_of::<TSEnumMemberName>() == 8);
    assert!(align_of::<TSEnumMemberName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeAnnotation>() == 16);
    assert!(align_of::<TSTypeAnnotation>() == 4);
    assert!(offset_of!(TSTypeAnnotation, span) == 0);
    assert!(offset_of!(TSTypeAnnotation, type_annotation) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSLiteralType>() == 16);
    assert!(align_of::<TSLiteralType>() == 4);
    assert!(offset_of!(TSLiteralType, span) == 0);
    assert!(offset_of!(TSLiteralType, literal) == 8);

    assert!(size_of::<TSLiteral>() == 8);
    assert!(align_of::<TSLiteral>() == 4);

    assert!(size_of::<TSType>() == 8);
    assert!(align_of::<TSType>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSConditionalType>() == 44);
    assert!(align_of::<TSConditionalType>() == 4);
    assert!(offset_of!(TSConditionalType, span) == 0);
    assert!(offset_of!(TSConditionalType, check_type) == 8);
    assert!(offset_of!(TSConditionalType, extends_type) == 16);
    assert!(offset_of!(TSConditionalType, true_type) == 24);
    assert!(offset_of!(TSConditionalType, false_type) == 32);
    assert!(offset_of!(TSConditionalType, scope_id) == 40);

    // Padding: 0 bytes
    assert!(size_of::<TSUnionType>() == 24);
    assert!(align_of::<TSUnionType>() == 4);
    assert!(offset_of!(TSUnionType, span) == 0);
    assert!(offset_of!(TSUnionType, types) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSIntersectionType>() == 24);
    assert!(align_of::<TSIntersectionType>() == 4);
    assert!(offset_of!(TSIntersectionType, span) == 0);
    assert!(offset_of!(TSIntersectionType, types) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSParenthesizedType>() == 16);
    assert!(align_of::<TSParenthesizedType>() == 4);
    assert!(offset_of!(TSParenthesizedType, span) == 0);
    assert!(offset_of!(TSParenthesizedType, type_annotation) == 8);

    // Padding: 3 bytes
    assert!(size_of::<TSTypeOperator>() == 20);
    assert!(align_of::<TSTypeOperator>() == 4);
    assert!(offset_of!(TSTypeOperator, span) == 0);
    assert!(offset_of!(TSTypeOperator, operator) == 16);
    assert!(offset_of!(TSTypeOperator, type_annotation) == 8);

    assert!(size_of::<TSTypeOperatorOperator>() == 1);
    assert!(align_of::<TSTypeOperatorOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSArrayType>() == 16);
    assert!(align_of::<TSArrayType>() == 4);
    assert!(offset_of!(TSArrayType, span) == 0);
    assert!(offset_of!(TSArrayType, element_type) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSIndexedAccessType>() == 24);
    assert!(align_of::<TSIndexedAccessType>() == 4);
    assert!(offset_of!(TSIndexedAccessType, span) == 0);
    assert!(offset_of!(TSIndexedAccessType, object_type) == 8);
    assert!(offset_of!(TSIndexedAccessType, index_type) == 16);

    // Padding: 0 bytes
    assert!(size_of::<TSTupleType>() == 24);
    assert!(align_of::<TSTupleType>() == 4);
    assert!(offset_of!(TSTupleType, span) == 0);
    assert!(offset_of!(TSTupleType, element_types) == 8);

    // Padding: 3 bytes
    assert!(size_of::<TSNamedTupleMember>() == 36);
    assert!(align_of::<TSNamedTupleMember>() == 4);
    assert!(offset_of!(TSNamedTupleMember, span) == 0);
    assert!(offset_of!(TSNamedTupleMember, label) == 8);
    assert!(offset_of!(TSNamedTupleMember, element_type) == 24);
    assert!(offset_of!(TSNamedTupleMember, optional) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSOptionalType>() == 16);
    assert!(align_of::<TSOptionalType>() == 4);
    assert!(offset_of!(TSOptionalType, span) == 0);
    assert!(offset_of!(TSOptionalType, type_annotation) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSRestType>() == 16);
    assert!(align_of::<TSRestType>() == 4);
    assert!(offset_of!(TSRestType, span) == 0);
    assert!(offset_of!(TSRestType, type_annotation) == 8);

    assert!(size_of::<TSTupleElement>() == 8);
    assert!(align_of::<TSTupleElement>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSAnyKeyword>() == 8);
    assert!(align_of::<TSAnyKeyword>() == 4);
    assert!(offset_of!(TSAnyKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSStringKeyword>() == 8);
    assert!(align_of::<TSStringKeyword>() == 4);
    assert!(offset_of!(TSStringKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSBooleanKeyword>() == 8);
    assert!(align_of::<TSBooleanKeyword>() == 4);
    assert!(offset_of!(TSBooleanKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNumberKeyword>() == 8);
    assert!(align_of::<TSNumberKeyword>() == 4);
    assert!(offset_of!(TSNumberKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNeverKeyword>() == 8);
    assert!(align_of::<TSNeverKeyword>() == 4);
    assert!(offset_of!(TSNeverKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSIntrinsicKeyword>() == 8);
    assert!(align_of::<TSIntrinsicKeyword>() == 4);
    assert!(offset_of!(TSIntrinsicKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSUnknownKeyword>() == 8);
    assert!(align_of::<TSUnknownKeyword>() == 4);
    assert!(offset_of!(TSUnknownKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSNullKeyword>() == 8);
    assert!(align_of::<TSNullKeyword>() == 4);
    assert!(offset_of!(TSNullKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSUndefinedKeyword>() == 8);
    assert!(align_of::<TSUndefinedKeyword>() == 4);
    assert!(offset_of!(TSUndefinedKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSVoidKeyword>() == 8);
    assert!(align_of::<TSVoidKeyword>() == 4);
    assert!(offset_of!(TSVoidKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSSymbolKeyword>() == 8);
    assert!(align_of::<TSSymbolKeyword>() == 4);
    assert!(offset_of!(TSSymbolKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSThisType>() == 8);
    assert!(align_of::<TSThisType>() == 4);
    assert!(offset_of!(TSThisType, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSObjectKeyword>() == 8);
    assert!(align_of::<TSObjectKeyword>() == 4);
    assert!(offset_of!(TSObjectKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSBigIntKeyword>() == 8);
    assert!(align_of::<TSBigIntKeyword>() == 4);
    assert!(offset_of!(TSBigIntKeyword, span) == 0);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeReference>() == 20);
    assert!(align_of::<TSTypeReference>() == 4);
    assert!(offset_of!(TSTypeReference, span) == 0);
    assert!(offset_of!(TSTypeReference, type_name) == 8);
    assert!(offset_of!(TSTypeReference, type_arguments) == 16);

    assert!(size_of::<TSTypeName>() == 8);
    assert!(align_of::<TSTypeName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSQualifiedName>() == 32);
    assert!(align_of::<TSQualifiedName>() == 4);
    assert!(offset_of!(TSQualifiedName, span) == 0);
    assert!(offset_of!(TSQualifiedName, left) == 8);
    assert!(offset_of!(TSQualifiedName, right) == 16);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeParameterInstantiation>() == 24);
    assert!(align_of::<TSTypeParameterInstantiation>() == 4);
    assert!(offset_of!(TSTypeParameterInstantiation, span) == 0);
    assert!(offset_of!(TSTypeParameterInstantiation, params) == 8);

    // Padding: 1 bytes
    assert!(size_of::<TSTypeParameter>() == 48);
    assert!(align_of::<TSTypeParameter>() == 4);
    assert!(offset_of!(TSTypeParameter, span) == 0);
    assert!(offset_of!(TSTypeParameter, name) == 8);
    assert!(offset_of!(TSTypeParameter, constraint) == 28);
    assert!(offset_of!(TSTypeParameter, default) == 36);
    assert!(offset_of!(TSTypeParameter, r#in) == 44);
    assert!(offset_of!(TSTypeParameter, out) == 45);
    assert!(offset_of!(TSTypeParameter, r#const) == 46);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeParameterDeclaration>() == 24);
    assert!(align_of::<TSTypeParameterDeclaration>() == 4);
    assert!(offset_of!(TSTypeParameterDeclaration, span) == 0);
    assert!(offset_of!(TSTypeParameterDeclaration, params) == 8);

    // Padding: 3 bytes
    assert!(size_of::<TSTypeAliasDeclaration>() == 48);
    assert!(align_of::<TSTypeAliasDeclaration>() == 4);
    assert!(offset_of!(TSTypeAliasDeclaration, span) == 0);
    assert!(offset_of!(TSTypeAliasDeclaration, id) == 8);
    assert!(offset_of!(TSTypeAliasDeclaration, type_parameters) == 28);
    assert!(offset_of!(TSTypeAliasDeclaration, type_annotation) == 32);
    assert!(offset_of!(TSTypeAliasDeclaration, declare) == 44);
    assert!(offset_of!(TSTypeAliasDeclaration, scope_id) == 40);

    assert!(size_of::<TSAccessibility>() == 1);
    assert!(align_of::<TSAccessibility>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSClassImplements>() == 20);
    assert!(align_of::<TSClassImplements>() == 4);
    assert!(offset_of!(TSClassImplements, span) == 0);
    assert!(offset_of!(TSClassImplements, expression) == 8);
    assert!(offset_of!(TSClassImplements, type_arguments) == 16);

    // Padding: 3 bytes
    assert!(size_of::<TSInterfaceDeclaration>() == 60);
    assert!(align_of::<TSInterfaceDeclaration>() == 4);
    assert!(offset_of!(TSInterfaceDeclaration, span) == 0);
    assert!(offset_of!(TSInterfaceDeclaration, id) == 8);
    assert!(offset_of!(TSInterfaceDeclaration, type_parameters) == 28);
    assert!(offset_of!(TSInterfaceDeclaration, extends) == 32);
    assert!(offset_of!(TSInterfaceDeclaration, body) == 48);
    assert!(offset_of!(TSInterfaceDeclaration, declare) == 56);
    assert!(offset_of!(TSInterfaceDeclaration, scope_id) == 52);

    // Padding: 0 bytes
    assert!(size_of::<TSInterfaceBody>() == 24);
    assert!(align_of::<TSInterfaceBody>() == 4);
    assert!(offset_of!(TSInterfaceBody, span) == 0);
    assert!(offset_of!(TSInterfaceBody, body) == 8);

    // Padding: 1 bytes
    assert!(size_of::<TSPropertySignature>() == 24);
    assert!(align_of::<TSPropertySignature>() == 4);
    assert!(offset_of!(TSPropertySignature, span) == 0);
    assert!(offset_of!(TSPropertySignature, computed) == 20);
    assert!(offset_of!(TSPropertySignature, optional) == 21);
    assert!(offset_of!(TSPropertySignature, readonly) == 22);
    assert!(offset_of!(TSPropertySignature, key) == 8);
    assert!(offset_of!(TSPropertySignature, type_annotation) == 16);

    assert!(size_of::<TSSignature>() == 8);
    assert!(align_of::<TSSignature>() == 4);

    // Padding: 2 bytes
    assert!(size_of::<TSIndexSignature>() == 32);
    assert!(align_of::<TSIndexSignature>() == 4);
    assert!(offset_of!(TSIndexSignature, span) == 0);
    assert!(offset_of!(TSIndexSignature, parameters) == 8);
    assert!(offset_of!(TSIndexSignature, type_annotation) == 24);
    assert!(offset_of!(TSIndexSignature, readonly) == 28);
    assert!(offset_of!(TSIndexSignature, r#static) == 29);

    // Padding: 0 bytes
    assert!(size_of::<TSCallSignatureDeclaration>() == 24);
    assert!(align_of::<TSCallSignatureDeclaration>() == 4);
    assert!(offset_of!(TSCallSignatureDeclaration, span) == 0);
    assert!(offset_of!(TSCallSignatureDeclaration, type_parameters) == 8);
    assert!(offset_of!(TSCallSignatureDeclaration, this_param) == 12);
    assert!(offset_of!(TSCallSignatureDeclaration, params) == 16);
    assert!(offset_of!(TSCallSignatureDeclaration, return_type) == 20);

    assert!(size_of::<TSMethodSignatureKind>() == 1);
    assert!(align_of::<TSMethodSignatureKind>() == 1);

    // Padding: 1 bytes
    assert!(size_of::<TSMethodSignature>() == 40);
    assert!(align_of::<TSMethodSignature>() == 4);
    assert!(offset_of!(TSMethodSignature, span) == 0);
    assert!(offset_of!(TSMethodSignature, key) == 8);
    assert!(offset_of!(TSMethodSignature, computed) == 36);
    assert!(offset_of!(TSMethodSignature, optional) == 37);
    assert!(offset_of!(TSMethodSignature, kind) == 38);
    assert!(offset_of!(TSMethodSignature, type_parameters) == 16);
    assert!(offset_of!(TSMethodSignature, this_param) == 20);
    assert!(offset_of!(TSMethodSignature, params) == 24);
    assert!(offset_of!(TSMethodSignature, return_type) == 28);
    assert!(offset_of!(TSMethodSignature, scope_id) == 32);

    // Padding: 0 bytes
    assert!(size_of::<TSConstructSignatureDeclaration>() == 24);
    assert!(align_of::<TSConstructSignatureDeclaration>() == 4);
    assert!(offset_of!(TSConstructSignatureDeclaration, span) == 0);
    assert!(offset_of!(TSConstructSignatureDeclaration, type_parameters) == 8);
    assert!(offset_of!(TSConstructSignatureDeclaration, params) == 12);
    assert!(offset_of!(TSConstructSignatureDeclaration, return_type) == 16);
    assert!(offset_of!(TSConstructSignatureDeclaration, scope_id) == 20);

    // Padding: 0 bytes
    assert!(size_of::<TSIndexSignatureName>() == 20);
    assert!(align_of::<TSIndexSignatureName>() == 4);
    assert!(offset_of!(TSIndexSignatureName, span) == 0);
    assert!(offset_of!(TSIndexSignatureName, name) == 8);
    assert!(offset_of!(TSIndexSignatureName, type_annotation) == 16);

    // Padding: 0 bytes
    assert!(size_of::<TSInterfaceHeritage>() == 20);
    assert!(align_of::<TSInterfaceHeritage>() == 4);
    assert!(offset_of!(TSInterfaceHeritage, span) == 0);
    assert!(offset_of!(TSInterfaceHeritage, expression) == 8);
    assert!(offset_of!(TSInterfaceHeritage, type_arguments) == 16);

    // Padding: 3 bytes
    assert!(size_of::<TSTypePredicate>() == 28);
    assert!(align_of::<TSTypePredicate>() == 4);
    assert!(offset_of!(TSTypePredicate, span) == 0);
    assert!(offset_of!(TSTypePredicate, parameter_name) == 8);
    assert!(offset_of!(TSTypePredicate, asserts) == 24);
    assert!(offset_of!(TSTypePredicate, type_annotation) == 20);

    assert!(size_of::<TSTypePredicateName>() == 12);
    assert!(align_of::<TSTypePredicateName>() == 4);

    // Padding: 2 bytes
    assert!(size_of::<TSModuleDeclaration>() == 56);
    assert!(align_of::<TSModuleDeclaration>() == 4);
    assert!(offset_of!(TSModuleDeclaration, span) == 0);
    assert!(offset_of!(TSModuleDeclaration, id) == 8);
    assert!(offset_of!(TSModuleDeclaration, body) == 40);
    assert!(offset_of!(TSModuleDeclaration, kind) == 52);
    assert!(offset_of!(TSModuleDeclaration, declare) == 53);
    assert!(offset_of!(TSModuleDeclaration, scope_id) == 48);

    assert!(size_of::<TSModuleDeclarationKind>() == 1);
    assert!(align_of::<TSModuleDeclarationKind>() == 1);

    assert!(size_of::<TSModuleDeclarationName>() == 32);
    assert!(align_of::<TSModuleDeclarationName>() == 4);

    assert!(size_of::<TSModuleDeclarationBody>() == 8);
    assert!(align_of::<TSModuleDeclarationBody>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSModuleBlock>() == 40);
    assert!(align_of::<TSModuleBlock>() == 4);
    assert!(offset_of!(TSModuleBlock, span) == 0);
    assert!(offset_of!(TSModuleBlock, directives) == 8);
    assert!(offset_of!(TSModuleBlock, body) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeLiteral>() == 24);
    assert!(align_of::<TSTypeLiteral>() == 4);
    assert!(offset_of!(TSTypeLiteral, span) == 0);
    assert!(offset_of!(TSTypeLiteral, members) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSInferType>() == 12);
    assert!(align_of::<TSInferType>() == 4);
    assert!(offset_of!(TSInferType, span) == 0);
    assert!(offset_of!(TSInferType, type_parameter) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeQuery>() == 20);
    assert!(align_of::<TSTypeQuery>() == 4);
    assert!(offset_of!(TSTypeQuery, span) == 0);
    assert!(offset_of!(TSTypeQuery, expr_name) == 8);
    assert!(offset_of!(TSTypeQuery, type_arguments) == 16);

    assert!(size_of::<TSTypeQueryExprName>() == 8);
    assert!(align_of::<TSTypeQueryExprName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSImportType>() == 32);
    assert!(align_of::<TSImportType>() == 4);
    assert!(offset_of!(TSImportType, span) == 0);
    assert!(offset_of!(TSImportType, argument) == 8);
    assert!(offset_of!(TSImportType, options) == 16);
    assert!(offset_of!(TSImportType, qualifier) == 20);
    assert!(offset_of!(TSImportType, type_arguments) == 28);

    // Padding: 0 bytes
    assert!(size_of::<TSFunctionType>() == 28);
    assert!(align_of::<TSFunctionType>() == 4);
    assert!(offset_of!(TSFunctionType, span) == 0);
    assert!(offset_of!(TSFunctionType, type_parameters) == 8);
    assert!(offset_of!(TSFunctionType, this_param) == 12);
    assert!(offset_of!(TSFunctionType, params) == 16);
    assert!(offset_of!(TSFunctionType, return_type) == 20);
    assert!(offset_of!(TSFunctionType, scope_id) == 24);

    // Padding: 3 bytes
    assert!(size_of::<TSConstructorType>() == 24);
    assert!(align_of::<TSConstructorType>() == 4);
    assert!(offset_of!(TSConstructorType, span) == 0);
    assert!(offset_of!(TSConstructorType, r#abstract) == 20);
    assert!(offset_of!(TSConstructorType, type_parameters) == 8);
    assert!(offset_of!(TSConstructorType, params) == 12);
    assert!(offset_of!(TSConstructorType, return_type) == 16);

    // Padding: 2 bytes
    assert!(size_of::<TSMappedType>() == 36);
    assert!(align_of::<TSMappedType>() == 4);
    assert!(offset_of!(TSMappedType, span) == 0);
    assert!(offset_of!(TSMappedType, type_parameter) == 8);
    assert!(offset_of!(TSMappedType, name_type) == 12);
    assert!(offset_of!(TSMappedType, type_annotation) == 20);
    assert!(offset_of!(TSMappedType, optional) == 32);
    assert!(offset_of!(TSMappedType, readonly) == 33);
    assert!(offset_of!(TSMappedType, scope_id) == 28);

    assert!(size_of::<TSMappedTypeModifierOperator>() == 1);
    assert!(align_of::<TSMappedTypeModifierOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<TSTemplateLiteralType>() == 40);
    assert!(align_of::<TSTemplateLiteralType>() == 4);
    assert!(offset_of!(TSTemplateLiteralType, span) == 0);
    assert!(offset_of!(TSTemplateLiteralType, quasis) == 8);
    assert!(offset_of!(TSTemplateLiteralType, types) == 24);

    // Padding: 0 bytes
    assert!(size_of::<TSAsExpression>() == 24);
    assert!(align_of::<TSAsExpression>() == 4);
    assert!(offset_of!(TSAsExpression, span) == 0);
    assert!(offset_of!(TSAsExpression, expression) == 8);
    assert!(offset_of!(TSAsExpression, type_annotation) == 16);

    // Padding: 0 bytes
    assert!(size_of::<TSSatisfiesExpression>() == 24);
    assert!(align_of::<TSSatisfiesExpression>() == 4);
    assert!(offset_of!(TSSatisfiesExpression, span) == 0);
    assert!(offset_of!(TSSatisfiesExpression, expression) == 8);
    assert!(offset_of!(TSSatisfiesExpression, type_annotation) == 16);

    // Padding: 0 bytes
    assert!(size_of::<TSTypeAssertion>() == 24);
    assert!(align_of::<TSTypeAssertion>() == 4);
    assert!(offset_of!(TSTypeAssertion, span) == 0);
    assert!(offset_of!(TSTypeAssertion, type_annotation) == 8);
    assert!(offset_of!(TSTypeAssertion, expression) == 16);

    // Padding: 3 bytes
    assert!(size_of::<TSImportEqualsDeclaration>() == 40);
    assert!(align_of::<TSImportEqualsDeclaration>() == 4);
    assert!(offset_of!(TSImportEqualsDeclaration, span) == 0);
    assert!(offset_of!(TSImportEqualsDeclaration, id) == 8);
    assert!(offset_of!(TSImportEqualsDeclaration, module_reference) == 28);
    assert!(offset_of!(TSImportEqualsDeclaration, import_kind) == 36);

    assert!(size_of::<TSModuleReference>() == 8);
    assert!(align_of::<TSModuleReference>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<TSExternalModuleReference>() == 36);
    assert!(align_of::<TSExternalModuleReference>() == 4);
    assert!(offset_of!(TSExternalModuleReference, span) == 0);
    assert!(offset_of!(TSExternalModuleReference, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSNonNullExpression>() == 16);
    assert!(align_of::<TSNonNullExpression>() == 4);
    assert!(offset_of!(TSNonNullExpression, span) == 0);
    assert!(offset_of!(TSNonNullExpression, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<Decorator>() == 16);
    assert!(align_of::<Decorator>() == 4);
    assert!(offset_of!(Decorator, span) == 0);
    assert!(offset_of!(Decorator, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSExportAssignment>() == 16);
    assert!(align_of::<TSExportAssignment>() == 4);
    assert!(offset_of!(TSExportAssignment, span) == 0);
    assert!(offset_of!(TSExportAssignment, expression) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSNamespaceExportDeclaration>() == 24);
    assert!(align_of::<TSNamespaceExportDeclaration>() == 4);
    assert!(offset_of!(TSNamespaceExportDeclaration, span) == 0);
    assert!(offset_of!(TSNamespaceExportDeclaration, id) == 8);

    // Padding: 0 bytes
    assert!(size_of::<TSInstantiationExpression>() == 20);
    assert!(align_of::<TSInstantiationExpression>() == 4);
    assert!(offset_of!(TSInstantiationExpression, span) == 0);
    assert!(offset_of!(TSInstantiationExpression, expression) == 8);
    assert!(offset_of!(TSInstantiationExpression, type_arguments) == 16);

    assert!(size_of::<ImportOrExportKind>() == 1);
    assert!(align_of::<ImportOrExportKind>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<JSDocNullableType>() == 20);
    assert!(align_of::<JSDocNullableType>() == 4);
    assert!(offset_of!(JSDocNullableType, span) == 0);
    assert!(offset_of!(JSDocNullableType, type_annotation) == 8);
    assert!(offset_of!(JSDocNullableType, postfix) == 16);

    // Padding: 3 bytes
    assert!(size_of::<JSDocNonNullableType>() == 20);
    assert!(align_of::<JSDocNonNullableType>() == 4);
    assert!(offset_of!(JSDocNonNullableType, span) == 0);
    assert!(offset_of!(JSDocNonNullableType, type_annotation) == 8);
    assert!(offset_of!(JSDocNonNullableType, postfix) == 16);

    // Padding: 0 bytes
    assert!(size_of::<JSDocUnknownType>() == 8);
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
    assert!(size_of::<Comment>() == 16);
    assert!(align_of::<Comment>() == 4);
    assert!(offset_of!(Comment, span) == 0);
    assert!(offset_of!(Comment, attached_to) == 8);
    assert!(offset_of!(Comment, kind) == 12);
    assert!(offset_of!(Comment, position) == 13);
    assert!(offset_of!(Comment, newlines) == 14);
    assert!(offset_of!(Comment, content) == 15);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
