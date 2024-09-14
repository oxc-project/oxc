// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/ast_macro.rs`

#![allow(clippy::useless_conversion)]

#[allow(unused_imports)]
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub fn gen(name: &str, input: TokenStream) -> TokenStream {
    match name {
        "BooleanLiteral" => gen_boolean_literal(input),
        "NullLiteral" => gen_null_literal(input),
        "NumericLiteral" => gen_numeric_literal(input),
        "BigIntLiteral" => gen_big_int_literal(input),
        "RegExpLiteral" => gen_reg_exp_literal(input),
        "RegExp" => gen_reg_exp(input),
        "RegExpPattern" => gen_reg_exp_pattern(input),
        "EmptyObject" => gen_empty_object(input),
        "StringLiteral" => gen_string_literal(input),
        "Program" => gen_program(input),
        "Expression" => gen_expression(input),
        "IdentifierName" => gen_identifier_name(input),
        "IdentifierReference" => gen_identifier_reference(input),
        "BindingIdentifier" => gen_binding_identifier(input),
        "LabelIdentifier" => gen_label_identifier(input),
        "ThisExpression" => gen_this_expression(input),
        "ArrayExpression" => gen_array_expression(input),
        "ArrayExpressionElement" => gen_array_expression_element(input),
        "Elision" => gen_elision(input),
        "ObjectExpression" => gen_object_expression(input),
        "ObjectPropertyKind" => gen_object_property_kind(input),
        "ObjectProperty" => gen_object_property(input),
        "PropertyKey" => gen_property_key(input),
        "PropertyKind" => gen_property_kind(input),
        "TemplateLiteral" => gen_template_literal(input),
        "TaggedTemplateExpression" => gen_tagged_template_expression(input),
        "TemplateElement" => gen_template_element(input),
        "TemplateElementValue" => gen_template_element_value(input),
        "MemberExpression" => gen_member_expression(input),
        "ComputedMemberExpression" => gen_computed_member_expression(input),
        "StaticMemberExpression" => gen_static_member_expression(input),
        "PrivateFieldExpression" => gen_private_field_expression(input),
        "CallExpression" => gen_call_expression(input),
        "NewExpression" => gen_new_expression(input),
        "MetaProperty" => gen_meta_property(input),
        "SpreadElement" => gen_spread_element(input),
        "Argument" => gen_argument(input),
        "UpdateExpression" => gen_update_expression(input),
        "UnaryExpression" => gen_unary_expression(input),
        "BinaryExpression" => gen_binary_expression(input),
        "PrivateInExpression" => gen_private_in_expression(input),
        "LogicalExpression" => gen_logical_expression(input),
        "ConditionalExpression" => gen_conditional_expression(input),
        "AssignmentExpression" => gen_assignment_expression(input),
        "AssignmentTarget" => gen_assignment_target(input),
        "SimpleAssignmentTarget" => gen_simple_assignment_target(input),
        "AssignmentTargetPattern" => gen_assignment_target_pattern(input),
        "ArrayAssignmentTarget" => gen_array_assignment_target(input),
        "ObjectAssignmentTarget" => gen_object_assignment_target(input),
        "AssignmentTargetRest" => gen_assignment_target_rest(input),
        "AssignmentTargetMaybeDefault" => gen_assignment_target_maybe_default(input),
        "AssignmentTargetWithDefault" => gen_assignment_target_with_default(input),
        "AssignmentTargetProperty" => gen_assignment_target_property(input),
        "AssignmentTargetPropertyIdentifier" => gen_assignment_target_property_identifier(input),
        "AssignmentTargetPropertyProperty" => gen_assignment_target_property_property(input),
        "SequenceExpression" => gen_sequence_expression(input),
        "Super" => gen_super(input),
        "AwaitExpression" => gen_await_expression(input),
        "ChainExpression" => gen_chain_expression(input),
        "ChainElement" => gen_chain_element(input),
        "ParenthesizedExpression" => gen_parenthesized_expression(input),
        "Statement" => gen_statement(input),
        "Directive" => gen_directive(input),
        "Hashbang" => gen_hashbang(input),
        "BlockStatement" => gen_block_statement(input),
        "Declaration" => gen_declaration(input),
        "VariableDeclaration" => gen_variable_declaration(input),
        "VariableDeclarationKind" => gen_variable_declaration_kind(input),
        "VariableDeclarator" => gen_variable_declarator(input),
        "EmptyStatement" => gen_empty_statement(input),
        "ExpressionStatement" => gen_expression_statement(input),
        "IfStatement" => gen_if_statement(input),
        "DoWhileStatement" => gen_do_while_statement(input),
        "WhileStatement" => gen_while_statement(input),
        "ForStatement" => gen_for_statement(input),
        "ForStatementInit" => gen_for_statement_init(input),
        "ForInStatement" => gen_for_in_statement(input),
        "ForStatementLeft" => gen_for_statement_left(input),
        "ForOfStatement" => gen_for_of_statement(input),
        "ContinueStatement" => gen_continue_statement(input),
        "BreakStatement" => gen_break_statement(input),
        "ReturnStatement" => gen_return_statement(input),
        "WithStatement" => gen_with_statement(input),
        "SwitchStatement" => gen_switch_statement(input),
        "SwitchCase" => gen_switch_case(input),
        "LabeledStatement" => gen_labeled_statement(input),
        "ThrowStatement" => gen_throw_statement(input),
        "TryStatement" => gen_try_statement(input),
        "CatchClause" => gen_catch_clause(input),
        "CatchParameter" => gen_catch_parameter(input),
        "DebuggerStatement" => gen_debugger_statement(input),
        "BindingPattern" => gen_binding_pattern(input),
        "BindingPatternKind" => gen_binding_pattern_kind(input),
        "AssignmentPattern" => gen_assignment_pattern(input),
        "ObjectPattern" => gen_object_pattern(input),
        "BindingProperty" => gen_binding_property(input),
        "ArrayPattern" => gen_array_pattern(input),
        "BindingRestElement" => gen_binding_rest_element(input),
        "Function" => gen_function(input),
        "FunctionType" => gen_function_type(input),
        "FormalParameters" => gen_formal_parameters(input),
        "FormalParameter" => gen_formal_parameter(input),
        "FormalParameterKind" => gen_formal_parameter_kind(input),
        "FunctionBody" => gen_function_body(input),
        "ArrowFunctionExpression" => gen_arrow_function_expression(input),
        "YieldExpression" => gen_yield_expression(input),
        "Class" => gen_class(input),
        "ClassType" => gen_class_type(input),
        "ClassBody" => gen_class_body(input),
        "ClassElement" => gen_class_element(input),
        "MethodDefinition" => gen_method_definition(input),
        "MethodDefinitionType" => gen_method_definition_type(input),
        "PropertyDefinition" => gen_property_definition(input),
        "PropertyDefinitionType" => gen_property_definition_type(input),
        "MethodDefinitionKind" => gen_method_definition_kind(input),
        "PrivateIdentifier" => gen_private_identifier(input),
        "StaticBlock" => gen_static_block(input),
        "ModuleDeclaration" => gen_module_declaration(input),
        "AccessorPropertyType" => gen_accessor_property_type(input),
        "AccessorProperty" => gen_accessor_property(input),
        "ImportExpression" => gen_import_expression(input),
        "ImportDeclaration" => gen_import_declaration(input),
        "ImportDeclarationSpecifier" => gen_import_declaration_specifier(input),
        "ImportSpecifier" => gen_import_specifier(input),
        "ImportDefaultSpecifier" => gen_import_default_specifier(input),
        "ImportNamespaceSpecifier" => gen_import_namespace_specifier(input),
        "WithClause" => gen_with_clause(input),
        "ImportAttribute" => gen_import_attribute(input),
        "ImportAttributeKey" => gen_import_attribute_key(input),
        "ExportNamedDeclaration" => gen_export_named_declaration(input),
        "ExportDefaultDeclaration" => gen_export_default_declaration(input),
        "ExportAllDeclaration" => gen_export_all_declaration(input),
        "ExportSpecifier" => gen_export_specifier(input),
        "ExportDefaultDeclarationKind" => gen_export_default_declaration_kind(input),
        "ModuleExportName" => gen_module_export_name(input),
        "TSThisParameter" => gen_ts_this_parameter(input),
        "TSEnumDeclaration" => gen_ts_enum_declaration(input),
        "TSEnumMember" => gen_ts_enum_member(input),
        "TSEnumMemberName" => gen_ts_enum_member_name(input),
        "TSTypeAnnotation" => gen_ts_type_annotation(input),
        "TSLiteralType" => gen_ts_literal_type(input),
        "TSLiteral" => gen_ts_literal(input),
        "TSType" => gen_ts_type(input),
        "TSConditionalType" => gen_ts_conditional_type(input),
        "TSUnionType" => gen_ts_union_type(input),
        "TSIntersectionType" => gen_ts_intersection_type(input),
        "TSParenthesizedType" => gen_ts_parenthesized_type(input),
        "TSTypeOperator" => gen_ts_type_operator(input),
        "TSTypeOperatorOperator" => gen_ts_type_operator_operator(input),
        "TSArrayType" => gen_ts_array_type(input),
        "TSIndexedAccessType" => gen_ts_indexed_access_type(input),
        "TSTupleType" => gen_ts_tuple_type(input),
        "TSNamedTupleMember" => gen_ts_named_tuple_member(input),
        "TSOptionalType" => gen_ts_optional_type(input),
        "TSRestType" => gen_ts_rest_type(input),
        "TSTupleElement" => gen_ts_tuple_element(input),
        "TSAnyKeyword" => gen_ts_any_keyword(input),
        "TSStringKeyword" => gen_ts_string_keyword(input),
        "TSBooleanKeyword" => gen_ts_boolean_keyword(input),
        "TSNumberKeyword" => gen_ts_number_keyword(input),
        "TSNeverKeyword" => gen_ts_never_keyword(input),
        "TSIntrinsicKeyword" => gen_ts_intrinsic_keyword(input),
        "TSUnknownKeyword" => gen_ts_unknown_keyword(input),
        "TSNullKeyword" => gen_ts_null_keyword(input),
        "TSUndefinedKeyword" => gen_ts_undefined_keyword(input),
        "TSVoidKeyword" => gen_ts_void_keyword(input),
        "TSSymbolKeyword" => gen_ts_symbol_keyword(input),
        "TSThisType" => gen_ts_this_type(input),
        "TSObjectKeyword" => gen_ts_object_keyword(input),
        "TSBigIntKeyword" => gen_ts_big_int_keyword(input),
        "TSTypeReference" => gen_ts_type_reference(input),
        "TSTypeName" => gen_ts_type_name(input),
        "TSQualifiedName" => gen_ts_qualified_name(input),
        "TSTypeParameterInstantiation" => gen_ts_type_parameter_instantiation(input),
        "TSTypeParameter" => gen_ts_type_parameter(input),
        "TSTypeParameterDeclaration" => gen_ts_type_parameter_declaration(input),
        "TSTypeAliasDeclaration" => gen_ts_type_alias_declaration(input),
        "TSAccessibility" => gen_ts_accessibility(input),
        "TSClassImplements" => gen_ts_class_implements(input),
        "TSInterfaceDeclaration" => gen_ts_interface_declaration(input),
        "TSInterfaceBody" => gen_ts_interface_body(input),
        "TSPropertySignature" => gen_ts_property_signature(input),
        "TSSignature" => gen_ts_signature(input),
        "TSIndexSignature" => gen_ts_index_signature(input),
        "TSCallSignatureDeclaration" => gen_ts_call_signature_declaration(input),
        "TSMethodSignatureKind" => gen_ts_method_signature_kind(input),
        "TSMethodSignature" => gen_ts_method_signature(input),
        "TSConstructSignatureDeclaration" => gen_ts_construct_signature_declaration(input),
        "TSIndexSignatureName" => gen_ts_index_signature_name(input),
        "TSInterfaceHeritage" => gen_ts_interface_heritage(input),
        "TSTypePredicate" => gen_ts_type_predicate(input),
        "TSTypePredicateName" => gen_ts_type_predicate_name(input),
        "TSModuleDeclaration" => gen_ts_module_declaration(input),
        "TSModuleDeclarationKind" => gen_ts_module_declaration_kind(input),
        "TSModuleDeclarationName" => gen_ts_module_declaration_name(input),
        "TSModuleDeclarationBody" => gen_ts_module_declaration_body(input),
        "TSModuleBlock" => gen_ts_module_block(input),
        "TSTypeLiteral" => gen_ts_type_literal(input),
        "TSInferType" => gen_ts_infer_type(input),
        "TSTypeQuery" => gen_ts_type_query(input),
        "TSTypeQueryExprName" => gen_ts_type_query_expr_name(input),
        "TSImportType" => gen_ts_import_type(input),
        "TSImportAttributes" => gen_ts_import_attributes(input),
        "TSImportAttribute" => gen_ts_import_attribute(input),
        "TSImportAttributeName" => gen_ts_import_attribute_name(input),
        "TSFunctionType" => gen_ts_function_type(input),
        "TSConstructorType" => gen_ts_constructor_type(input),
        "TSMappedType" => gen_ts_mapped_type(input),
        "TSMappedTypeModifierOperator" => gen_ts_mapped_type_modifier_operator(input),
        "TSTemplateLiteralType" => gen_ts_template_literal_type(input),
        "TSAsExpression" => gen_ts_as_expression(input),
        "TSSatisfiesExpression" => gen_ts_satisfies_expression(input),
        "TSTypeAssertion" => gen_ts_type_assertion(input),
        "TSImportEqualsDeclaration" => gen_ts_import_equals_declaration(input),
        "TSModuleReference" => gen_ts_module_reference(input),
        "TSExternalModuleReference" => gen_ts_external_module_reference(input),
        "TSNonNullExpression" => gen_ts_non_null_expression(input),
        "Decorator" => gen_decorator(input),
        "TSExportAssignment" => gen_ts_export_assignment(input),
        "TSNamespaceExportDeclaration" => gen_ts_namespace_export_declaration(input),
        "TSInstantiationExpression" => gen_ts_instantiation_expression(input),
        "ImportOrExportKind" => gen_import_or_export_kind(input),
        "JSDocNullableType" => gen_js_doc_nullable_type(input),
        "JSDocNonNullableType" => gen_js_doc_non_nullable_type(input),
        "JSDocUnknownType" => gen_js_doc_unknown_type(input),
        "JSXElement" => gen_jsx_element(input),
        "JSXOpeningElement" => gen_jsx_opening_element(input),
        "JSXClosingElement" => gen_jsx_closing_element(input),
        "JSXFragment" => gen_jsx_fragment(input),
        "JSXOpeningFragment" => gen_jsx_opening_fragment(input),
        "JSXClosingFragment" => gen_jsx_closing_fragment(input),
        "JSXElementName" => gen_jsx_element_name(input),
        "JSXNamespacedName" => gen_jsx_namespaced_name(input),
        "JSXMemberExpression" => gen_jsx_member_expression(input),
        "JSXMemberExpressionObject" => gen_jsx_member_expression_object(input),
        "JSXExpressionContainer" => gen_jsx_expression_container(input),
        "JSXExpression" => gen_jsx_expression(input),
        "JSXEmptyExpression" => gen_jsx_empty_expression(input),
        "JSXAttributeItem" => gen_jsx_attribute_item(input),
        "JSXAttribute" => gen_jsx_attribute(input),
        "JSXSpreadAttribute" => gen_jsx_spread_attribute(input),
        "JSXAttributeName" => gen_jsx_attribute_name(input),
        "JSXAttributeValue" => gen_jsx_attribute_value(input),
        "JSXIdentifier" => gen_jsx_identifier(input),
        "JSXChild" => gen_jsx_child(input),
        "JSXSpreadChild" => gen_jsx_spread_child(input),
        "JSXText" => gen_jsx_text(input),
        "NumberBase" => gen_number_base(input),
        "BigintBase" => gen_bigint_base(input),
        "AssignmentOperator" => gen_assignment_operator(input),
        "BinaryOperator" => gen_binary_operator(input),
        "LogicalOperator" => gen_logical_operator(input),
        "UnaryOperator" => gen_unary_operator(input),
        "UpdateOperator" => gen_update_operator(input),
        "Span" => gen_span(input),
        "SourceType" => gen_source_type(input),
        "Language" => gen_language(input),
        "ModuleKind" => gen_module_kind(input),
        "LanguageVariant" => gen_language_variant(input),
        "RegularExpression" => gen_regular_expression(input),
        "Flags" => gen_flags(input),
        "Pattern" => gen_pattern(input),
        "Disjunction" => gen_disjunction(input),
        "Alternative" => gen_alternative(input),
        "Term" => gen_term(input),
        "BoundaryAssertion" => gen_boundary_assertion(input),
        "BoundaryAssertionKind" => gen_boundary_assertion_kind(input),
        "LookAroundAssertion" => gen_look_around_assertion(input),
        "LookAroundAssertionKind" => gen_look_around_assertion_kind(input),
        "Quantifier" => gen_quantifier(input),
        "Character" => gen_character(input),
        "CharacterKind" => gen_character_kind(input),
        "CharacterClassEscape" => gen_character_class_escape(input),
        "CharacterClassEscapeKind" => gen_character_class_escape_kind(input),
        "UnicodePropertyEscape" => gen_unicode_property_escape(input),
        "Dot" => gen_dot(input),
        "CharacterClass" => gen_character_class(input),
        "CharacterClassContentsKind" => gen_character_class_contents_kind(input),
        "CharacterClassContents" => gen_character_class_contents(input),
        "CharacterClassRange" => gen_character_class_range(input),
        "ClassStringDisjunction" => gen_class_string_disjunction(input),
        "ClassString" => gen_class_string(input),
        "CapturingGroup" => gen_capturing_group(input),
        "IgnoreGroup" => gen_ignore_group(input),
        "ModifierFlags" => gen_modifier_flags(input),
        "IndexedReference" => gen_indexed_reference(input),
        "NamedReference" => gen_named_reference(input),
        _ => unreachable!(),
    }
}

fn gen_boolean_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_null_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_numeric_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_big_int_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_reg_exp_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_reg_exp(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_reg_exp_pattern(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_empty_object(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_string_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_program(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_identifier_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_identifier_reference(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_binding_identifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_label_identifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_this_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_array_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_array_expression_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_elision(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_object_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_object_property_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_object_property(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_property_key(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_property_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_template_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_tagged_template_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_template_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_template_element_value(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_member_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_computed_member_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_static_member_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_private_field_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_call_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_new_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_meta_property(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_spread_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_argument(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_update_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_unary_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_binary_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_private_in_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_logical_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_conditional_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_target(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_simple_assignment_target(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_target_pattern(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_array_assignment_target(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_object_assignment_target(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_target_rest(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_target_maybe_default(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_target_with_default(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_target_property(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_target_property_identifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_target_property_property(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_sequence_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_super(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_await_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_chain_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_chain_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_parenthesized_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_directive(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_hashbang(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_block_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_variable_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_variable_declaration_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_variable_declarator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_empty_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_expression_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_if_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_do_while_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_while_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_for_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_for_statement_init(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_for_in_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_for_statement_left(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_for_of_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_continue_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_break_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_return_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_with_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_switch_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_switch_case(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_labeled_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_throw_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_try_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_catch_clause(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_catch_parameter(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_debugger_statement(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_binding_pattern(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_binding_pattern_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_assignment_pattern(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_object_pattern(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_binding_property(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_array_pattern(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_binding_rest_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_function(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_function_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_formal_parameters(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_formal_parameter(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_formal_parameter_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_function_body(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_arrow_function_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_yield_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_class(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_class_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_class_body(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_class_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_method_definition(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_method_definition_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_property_definition(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_property_definition_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_method_definition_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_private_identifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_static_block(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_module_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_accessor_property_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_accessor_property(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_declaration_specifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_specifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_default_specifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_namespace_specifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_with_clause(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_attribute(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_attribute_key(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_export_named_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_export_default_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_export_all_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_export_specifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_export_default_declaration_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_module_export_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_this_parameter(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_enum_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_enum_member(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_enum_member_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_annotation(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_literal_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_conditional_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_union_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_intersection_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_parenthesized_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_operator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_operator_operator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_ts_array_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_indexed_access_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_tuple_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_named_tuple_member(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_optional_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_rest_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_tuple_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_any_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_string_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_boolean_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_number_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_never_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_intrinsic_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_unknown_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_null_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_undefined_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_void_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_symbol_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_this_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_object_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_big_int_keyword(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_reference(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_qualified_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_parameter_instantiation(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_parameter(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_parameter_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_alias_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_accessibility(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_ts_class_implements(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_interface_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_interface_body(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_property_signature(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_signature(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_index_signature(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_call_signature_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_method_signature_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_ts_method_signature(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_construct_signature_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_index_signature_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_interface_heritage(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_predicate(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_predicate_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_module_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_module_declaration_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_ts_module_declaration_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_module_declaration_body(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_module_block(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_literal(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_infer_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_query(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_query_expr_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_import_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_import_attributes(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_import_attribute(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_import_attribute_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_function_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_constructor_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_mapped_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_mapped_type_modifier_operator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_ts_template_literal_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_as_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_satisfies_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_type_assertion(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_import_equals_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_module_reference(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_external_module_reference(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_non_null_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_decorator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_export_assignment(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_namespace_export_declaration(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_instantiation_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_import_or_export_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_js_doc_nullable_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_js_doc_non_nullable_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_js_doc_unknown_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_opening_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_closing_element(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_fragment(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_opening_fragment(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_closing_fragment(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_element_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_namespaced_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_member_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_member_expression_object(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_expression_container(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_empty_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_attribute_item(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_attribute(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_spread_attribute(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_attribute_name(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_attribute_value(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_identifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_child(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_spread_child(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_text(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_number_base(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_bigint_base(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_assignment_operator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_binary_operator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_logical_operator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_unary_operator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_update_operator(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_span(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream
}

fn gen_source_type(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream
}

fn gen_language(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream
}

fn gen_module_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream
}

fn gen_language_variant(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream
}

fn gen_regular_expression(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_flags(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_pattern(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_disjunction(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_alternative(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_term(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_boundary_assertion(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_boundary_assertion_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_look_around_assertion(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_look_around_assertion_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_quantifier(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_character(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_character_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_character_class_escape(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_character_class_escape_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_unicode_property_escape(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_dot(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_character_class(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_character_class_contents_kind(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_character_class_contents(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c_u8());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_character_class_range(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_class_string_disjunction(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_class_string(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_capturing_group(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_ignore_group(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_modifier_flags(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_indexed_reference(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_named_reference(input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_c());
    stream.extend(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn derive_ast() -> TokenStream {
    [
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(
            Delimiter::Bracket,
            [
                TokenTree::Ident(Ident::new("derive", Span::call_site())),
                TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    [
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("oxc_ast_macros", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("Ast", Span::call_site())),
                    ]
                    .into_iter()
                    .collect(),
                )),
            ]
            .into_iter()
            .collect(),
        )),
    ]
    .into_iter()
    .collect()
}

fn repr_c() -> TokenStream {
    repr(TokenStream::from(TokenTree::Ident(Ident::new("C", Span::call_site()))))
}

fn repr_u8() -> TokenStream {
    repr(TokenStream::from(TokenTree::Ident(Ident::new("u8", Span::call_site()))))
}

fn repr_c_u8() -> TokenStream {
    repr(
        [
            TokenTree::Ident(Ident::new("C", Span::call_site())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("u8", Span::call_site())),
        ]
        .into_iter()
        .collect(),
    )
}

fn repr(rep: TokenStream) -> TokenStream {
    [
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(
            Delimiter::Bracket,
            [
                TokenTree::Ident(Ident::new("repr", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, rep.into_iter().collect())),
            ]
            .into_iter()
            .collect(),
        )),
    ]
    .into_iter()
    .collect()
}

fn assert_clone_in() -> TokenStream {
    assert(
        [
            TokenTree::Ident(Ident::new("CloneIn", Span::call_site())),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
            TokenTree::Ident(Ident::new("static", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        ]
        .into_iter()
        .collect(),
        [
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("oxc_allocator", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("CloneIn", Span::call_site())),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
            TokenTree::Ident(Ident::new("static", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        ]
        .into_iter()
        .collect(),
    )
}

fn assert_get_span() -> TokenStream {
    assert(
        TokenStream::from(TokenTree::Ident(Ident::new("GetSpan", Span::call_site()))),
        [
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("oxc_span", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("GetSpan", Span::call_site())),
        ]
        .into_iter()
        .collect(),
    )
}

fn assert_get_span_mut() -> TokenStream {
    assert(
        TokenStream::from(TokenTree::Ident(Ident::new("GetSpanMut", Span::call_site()))),
        [
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("oxc_span", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("GetSpanMut", Span::call_site())),
        ]
        .into_iter()
        .collect(),
    )
}

fn assert_content_eq() -> TokenStream {
    assert(
        TokenStream::from(TokenTree::Ident(Ident::new("ContentEq", Span::call_site()))),
        [
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("oxc_span", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("cmp", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("ContentEq", Span::call_site())),
        ]
        .into_iter()
        .collect(),
    )
}

fn assert_content_hash() -> TokenStream {
    assert(
        TokenStream::from(TokenTree::Ident(Ident::new("ContentHash", Span::call_site()))),
        [
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("oxc_span", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("hash", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("ContentHash", Span::call_site())),
        ]
        .into_iter()
        .collect(),
    )
}

fn assert(name: TokenStream, path: TokenStream) -> TokenStream {
    [
        TokenTree::Ident(Ident::new("const", Span::call_site())),
        TokenTree::Ident(Ident::new("_", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Group(Group::new(
            Delimiter::Brace,
            [
                TokenTree::Ident(Ident::new("trait", Span::call_site())),
                TokenTree::Ident(Ident::new("AssertionTrait", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            ]
            .into_iter()
            .chain(path.into_iter())
            .chain(
                [
                    TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())),
                    TokenTree::Ident(Ident::new("impl", Span::call_site())),
                    TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("T", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                ]
                .into_iter(),
            )
            .chain(name.into_iter())
            .chain(
                [
                    TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("AssertionTrait", Span::call_site())),
                    TokenTree::Ident(Ident::new("for", Span::call_site())),
                    TokenTree::Ident(Ident::new("T", Span::call_site())),
                    TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())),
                ]
                .into_iter(),
            )
            .collect(),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]
    .into_iter()
    .collect()
}
