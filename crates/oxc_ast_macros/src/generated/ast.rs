// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/ast_macro.rs`

#![allow(clippy::useless_conversion)]

#[allow(unused_imports)]
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub fn gen(name: &str, input: TokenStream) -> TokenStream {
    match name {
        "AssignmentOperator" => gen_assignment_operator(input),
        "BooleanLiteral" => gen_boolean_literal(input),
        "JSXElement" => gen_jsx_element(input),
        "NumberBase" => gen_number_base(input),
        "Program" => gen_program(input),
        "RegularExpression" => gen_regular_expression(input),
        "TSThisParameter" => gen_ts_this_parameter(input),
        "AccessorProperty"
        | "Alternative"
        | "ArrayAssignmentTarget"
        | "ArrayExpression"
        | "ArrayPattern"
        | "ArrowFunctionExpression"
        | "AssignmentExpression"
        | "AssignmentPattern"
        | "AssignmentTargetPropertyIdentifier"
        | "AssignmentTargetPropertyProperty"
        | "AssignmentTargetRest"
        | "AssignmentTargetWithDefault"
        | "AwaitExpression"
        | "BigIntLiteral"
        | "BinaryExpression"
        | "BindingIdentifier"
        | "BindingPattern"
        | "BindingProperty"
        | "BindingRestElement"
        | "BlockStatement"
        | "BoundaryAssertion"
        | "BreakStatement"
        | "CallExpression"
        | "CapturingGroup"
        | "CatchClause"
        | "CatchParameter"
        | "ChainExpression"
        | "Character"
        | "CharacterClass"
        | "CharacterClassEscape"
        | "CharacterClassRange"
        | "Class"
        | "ClassBody"
        | "ClassString"
        | "ClassStringDisjunction"
        | "ComputedMemberExpression"
        | "ConditionalExpression"
        | "ContinueStatement"
        | "DebuggerStatement"
        | "Decorator"
        | "Directive"
        | "Disjunction"
        | "DoWhileStatement"
        | "Dot"
        | "Elision"
        | "EmptyObject"
        | "EmptyStatement"
        | "ExportAllDeclaration"
        | "ExportDefaultDeclaration"
        | "ExportNamedDeclaration"
        | "ExportSpecifier"
        | "ExpressionStatement"
        | "Flags"
        | "ForInStatement"
        | "ForOfStatement"
        | "ForStatement"
        | "FormalParameter"
        | "FormalParameters"
        | "Function"
        | "FunctionBody"
        | "Hashbang"
        | "IdentifierName"
        | "IdentifierReference"
        | "IfStatement"
        | "IgnoreGroup"
        | "ImportAttribute"
        | "ImportDeclaration"
        | "ImportDefaultSpecifier"
        | "ImportExpression"
        | "ImportNamespaceSpecifier"
        | "ImportSpecifier"
        | "IndexedReference"
        | "JSDocNonNullableType"
        | "JSDocNullableType"
        | "JSDocUnknownType"
        | "JSXAttribute"
        | "JSXClosingElement"
        | "JSXClosingFragment"
        | "JSXEmptyExpression"
        | "JSXExpressionContainer"
        | "JSXFragment"
        | "JSXIdentifier"
        | "JSXMemberExpression"
        | "JSXNamespacedName"
        | "JSXOpeningElement"
        | "JSXOpeningFragment"
        | "JSXSpreadAttribute"
        | "JSXSpreadChild"
        | "JSXText"
        | "LabelIdentifier"
        | "LabeledStatement"
        | "LogicalExpression"
        | "LookAroundAssertion"
        | "MetaProperty"
        | "MethodDefinition"
        | "ModifierFlags"
        | "NamedReference"
        | "NewExpression"
        | "NullLiteral"
        | "NumericLiteral"
        | "ObjectAssignmentTarget"
        | "ObjectExpression"
        | "ObjectPattern"
        | "ObjectProperty"
        | "ParenthesizedExpression"
        | "Pattern"
        | "PrivateFieldExpression"
        | "PrivateIdentifier"
        | "PrivateInExpression"
        | "PropertyDefinition"
        | "Quantifier"
        | "RegExp"
        | "RegExpLiteral"
        | "ReturnStatement"
        | "SequenceExpression"
        | "SourceType"
        | "Span"
        | "SpreadElement"
        | "StaticBlock"
        | "StaticMemberExpression"
        | "StringLiteral"
        | "Super"
        | "SwitchCase"
        | "SwitchStatement"
        | "TSAnyKeyword"
        | "TSArrayType"
        | "TSAsExpression"
        | "TSBigIntKeyword"
        | "TSBooleanKeyword"
        | "TSCallSignatureDeclaration"
        | "TSClassImplements"
        | "TSConditionalType"
        | "TSConstructSignatureDeclaration"
        | "TSConstructorType"
        | "TSEnumDeclaration"
        | "TSEnumMember"
        | "TSExportAssignment"
        | "TSExternalModuleReference"
        | "TSFunctionType"
        | "TSImportAttribute"
        | "TSImportAttributes"
        | "TSImportEqualsDeclaration"
        | "TSImportType"
        | "TSIndexSignature"
        | "TSIndexSignatureName"
        | "TSIndexedAccessType"
        | "TSInferType"
        | "TSInstantiationExpression"
        | "TSInterfaceBody"
        | "TSInterfaceDeclaration"
        | "TSInterfaceHeritage"
        | "TSIntersectionType"
        | "TSIntrinsicKeyword"
        | "TSLiteralType"
        | "TSMappedType"
        | "TSMethodSignature"
        | "TSModuleBlock"
        | "TSModuleDeclaration"
        | "TSNamedTupleMember"
        | "TSNamespaceExportDeclaration"
        | "TSNeverKeyword"
        | "TSNonNullExpression"
        | "TSNullKeyword"
        | "TSNumberKeyword"
        | "TSObjectKeyword"
        | "TSOptionalType"
        | "TSParenthesizedType"
        | "TSPropertySignature"
        | "TSQualifiedName"
        | "TSRestType"
        | "TSSatisfiesExpression"
        | "TSStringKeyword"
        | "TSSymbolKeyword"
        | "TSTemplateLiteralType"
        | "TSThisType"
        | "TSTupleType"
        | "TSTypeAliasDeclaration"
        | "TSTypeAnnotation"
        | "TSTypeAssertion"
        | "TSTypeLiteral"
        | "TSTypeOperator"
        | "TSTypeParameter"
        | "TSTypeParameterDeclaration"
        | "TSTypeParameterInstantiation"
        | "TSTypePredicate"
        | "TSTypeQuery"
        | "TSTypeReference"
        | "TSUndefinedKeyword"
        | "TSUnionType"
        | "TSUnknownKeyword"
        | "TSVoidKeyword"
        | "TaggedTemplateExpression"
        | "TemplateElement"
        | "TemplateElementValue"
        | "TemplateLiteral"
        | "ThisExpression"
        | "ThrowStatement"
        | "TryStatement"
        | "UnaryExpression"
        | "UnicodePropertyEscape"
        | "UpdateExpression"
        | "VariableDeclaration"
        | "VariableDeclarator"
        | "WhileStatement"
        | "WithClause"
        | "WithStatement"
        | "YieldExpression" => repr_c(input),
        "Argument"
        | "ArrayExpressionElement"
        | "AssignmentTargetMaybeDefault"
        | "AssignmentTargetPattern"
        | "AssignmentTargetProperty"
        | "BindingPatternKind"
        | "ChainElement"
        | "CharacterClassContents"
        | "ClassElement"
        | "Declaration"
        | "ExportDefaultDeclarationKind"
        | "Expression"
        | "ForStatementInit"
        | "ForStatementLeft"
        | "ImportAttributeKey"
        | "ImportDeclarationSpecifier"
        | "JSXAttributeItem"
        | "JSXAttributeName"
        | "JSXAttributeValue"
        | "JSXChild"
        | "JSXElementName"
        | "JSXExpression"
        | "JSXMemberExpressionObject"
        | "MemberExpression"
        | "ModuleDeclaration"
        | "ModuleExportName"
        | "ObjectPropertyKind"
        | "PropertyKey"
        | "RegExpPattern"
        | "SimpleAssignmentTarget"
        | "Statement"
        | "TSEnumMemberName"
        | "TSImportAttributeName"
        | "TSLiteral"
        | "TSModuleDeclarationBody"
        | "TSModuleDeclarationName"
        | "TSModuleReference"
        | "TSSignature"
        | "TSTupleElement"
        | "TSType"
        | "TSTypeName"
        | "TSTypePredicateName"
        | "TSTypeQueryExprName"
        | "Term" => repr_c_u8(input),
        "AccessorPropertyType"
        | "AssignmentTarget"
        | "BigintBase"
        | "BinaryOperator"
        | "BoundaryAssertionKind"
        | "CharacterClassContentsKind"
        | "CharacterClassEscapeKind"
        | "CharacterKind"
        | "ClassType"
        | "FormalParameterKind"
        | "FunctionType"
        | "ImportOrExportKind"
        | "Language"
        | "LanguageVariant"
        | "LogicalOperator"
        | "LookAroundAssertionKind"
        | "MethodDefinitionKind"
        | "MethodDefinitionType"
        | "ModuleKind"
        | "PropertyDefinitionType"
        | "PropertyKind"
        | "TSAccessibility"
        | "TSMappedTypeModifierOperator"
        | "TSMethodSignatureKind"
        | "TSModuleDeclarationKind"
        | "TSTypeOperatorOperator"
        | "UnaryOperator"
        | "UpdateOperator"
        | "VariableDeclarationKind" => repr_u8(input),
        _ => unreachable!(),
    }
}

fn gen_boolean_literal(input: TokenStream) -> TokenStream {
    let mut stream = repr_c(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_program(input: TokenStream) -> TokenStream {
    let mut stream = repr_c(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_ts_this_parameter(input: TokenStream) -> TokenStream {
    let mut stream = repr_c(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_jsx_element(input: TokenStream) -> TokenStream {
    let mut stream = repr_c(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream.extend(assert_get_span());
    stream.extend(assert_get_span_mut());
    stream
}

fn gen_number_base(input: TokenStream) -> TokenStream {
    let mut stream = repr_u8(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_assignment_operator(input: TokenStream) -> TokenStream {
    let mut stream = repr_u8(input);
    stream.extend(assert_clone_in());
    stream.extend(assert_content_eq());
    stream.extend(assert_content_hash());
    stream
}

fn gen_regular_expression(input: TokenStream) -> TokenStream {
    let mut stream = repr_c(input);
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

fn repr_c(input: TokenStream) -> TokenStream {
    repr(TokenStream::from(TokenTree::Ident(Ident::new("C", Span::call_site()))), input)
}

fn repr_u8(input: TokenStream) -> TokenStream {
    repr(TokenStream::from(TokenTree::Ident(Ident::new("u8", Span::call_site()))), input)
}

fn repr_c_u8(input: TokenStream) -> TokenStream {
    repr(
        [
            TokenTree::Ident(Ident::new("C", Span::call_site())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("u8", Span::call_site())),
        ]
        .into_iter()
        .collect(),
        input,
    )
}

fn repr(rep: TokenStream, input: TokenStream) -> TokenStream {
    let mut stream = derive_ast();
    stream.extend(repr_raw(rep));
    stream.extend(input);
    stream
}

fn repr_raw(rep: TokenStream) -> TokenStream {
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
