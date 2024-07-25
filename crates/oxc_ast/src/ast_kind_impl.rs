use oxc_span::Atom;
use oxc_syntax::scope::ScopeId;

use super::{ast::*, AstKind};

impl<'a> AstKind<'a> {
    #[rustfmt::skip]
    pub fn is_statement(self) -> bool {
        self.is_iteration_statement()
            || matches!(self, Self::BlockStatement(_) | Self::BreakStatement(_) | Self::ContinueStatement(_)
                    | Self::DebuggerStatement(_) | Self::EmptyStatement(_) | Self::ExpressionStatement(_)
                    | Self::LabeledStatement(_) | Self::ReturnStatement(_) | Self::SwitchStatement(_)
                    | Self::ThrowStatement(_) | Self::TryStatement(_) | Self::WithStatement(_)
                    | Self::IfStatement(_) | Self::VariableDeclaration(_))
    }

    #[rustfmt::skip]
    pub fn is_declaration(self) -> bool {
        matches!(self, Self::Function(func) if func.is_declaration())
        || matches!(self, Self::Class(class) if class.is_declaration())
        || matches!(self, Self::ModuleDeclaration(_) | Self::TSEnumDeclaration(_) | Self::TSModuleDeclaration(_)
            | Self::VariableDeclaration(_) | Self::TSInterfaceDeclaration(_)
            | Self::TSTypeAliasDeclaration(_) | Self::TSImportEqualsDeclaration(_) | Self::PropertyDefinition(_)
        )
    }

    #[rustfmt::skip]
    pub fn is_iteration_statement(self) -> bool {
        matches!(self, Self::DoWhileStatement(_) | Self::WhileStatement(_) | Self::ForInStatement(_)
                | Self::ForOfStatement(_) | Self::ForStatement(_))
    }

    #[rustfmt::skip]
    pub fn is_identifier(self) -> bool {
        matches!(self, Self::BindingIdentifier(_)
                | Self::IdentifierReference(_)
                | Self::LabelIdentifier(_))
    }

    #[rustfmt::skip]
    pub fn is_type(self) -> bool {
        matches!(self, Self::TSAnyKeyword(_) | Self::TSBigIntKeyword(_) | Self::TSBooleanKeyword(_) | Self::TSIntrinsicKeyword(_)
                | Self::TSNeverKeyword(_) | Self::TSNullKeyword(_) | Self::TSNumberKeyword(_) | Self::TSObjectKeyword(_)
                | Self::TSStringKeyword(_) | Self::TSSymbolKeyword(_) | Self::TSUndefinedKeyword(_) | Self::TSUnknownKeyword(_)
                | Self::TSVoidKeyword(_) | Self::TSIndexedAccessType(_) | Self::TSInferType(_) | Self::TSIntersectionType(_)
                | Self::TSLiteralType(_) | Self::TSMethodSignature(_) | Self::TSTemplateLiteralType(_) | Self::TSThisType(_)
                | Self::TSTypeLiteral(_) | Self::TSTypeReference(_) | Self::TSUnionType(_))
    }

    pub fn is_literal(self) -> bool {
        matches!(
            self,
            Self::NumericLiteral(_)
                | Self::StringLiteral(_)
                | Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::TemplateLiteral(_)
        )
    }

    pub fn is_function_like(self) -> bool {
        matches!(self, Self::Function(_) | Self::ArrowFunctionExpression(_))
    }

    pub fn identifier_name(self) -> Option<Atom<'a>> {
        match self {
            Self::BindingIdentifier(ident) => Some(ident.name.clone()),
            Self::IdentifierReference(ident) => Some(ident.name.clone()),
            Self::LabelIdentifier(ident) => Some(ident.name.clone()),
            Self::IdentifierName(ident) => Some(ident.name.clone()),
            _ => None,
        }
    }

    pub fn is_jsx(self) -> bool {
        matches!(
            self,
            Self::JSXElement(_)
                | Self::JSXOpeningElement(_)
                | Self::JSXElementName(_)
                | Self::JSXFragment(_)
                | Self::JSXAttributeItem(_)
                | Self::JSXText(_)
                | Self::JSXExpressionContainer(_)
        )
    }

    pub fn is_specific_id_reference(&self, name: &str) -> bool {
        match self {
            Self::IdentifierReference(ident) => ident.name == name,
            _ => false,
        }
    }

    /// If this node is a container, get the [`ScopeId`] it creates.
    ///
    /// Will always be none if semantic analysis has not been run.
    pub fn get_container_scope_id(self) -> Option<ScopeId> {
        match self {
            Self::Program(p) => p.scope_id.get(),
            Self::BlockStatement(b) => b.scope_id.get(),
            Self::ForStatement(f) => f.scope_id.get(),
            Self::ForInStatement(f) => f.scope_id.get(),
            Self::ForOfStatement(f) => f.scope_id.get(),
            Self::SwitchStatement(switch) => switch.scope_id.get(),
            Self::CatchClause(catch) => catch.scope_id.get(),
            Self::Function(f) => f.scope_id.get(),
            Self::ArrowFunctionExpression(f) => f.scope_id.get(),
            Self::Class(class) => class.scope_id.get(),
            Self::StaticBlock(b) => b.scope_id.get(),
            Self::TSEnumDeclaration(e) => e.scope_id.get(),
            Self::TSConditionalType(e) => e.scope_id.get(),
            Self::TSTypeAliasDeclaration(e) => e.scope_id.get(),
            Self::TSInterfaceDeclaration(e) => e.scope_id.get(),
            Self::TSMethodSignature(e) => e.scope_id.get(),
            Self::TSConstructSignatureDeclaration(e) => e.scope_id.get(),
            Self::TSModuleDeclaration(e) => e.scope_id.get(),
            Self::TSMappedType(e) => e.scope_id.get(),
            _ => None,
        }
    }

    pub fn from_expression(e: &'a Expression<'a>) -> Self {
        match e {
            Expression::BooleanLiteral(e) => Self::BooleanLiteral(e),
            Expression::NullLiteral(e) => Self::NullLiteral(e),
            Expression::NumericLiteral(e) => Self::NumericLiteral(e),
            Expression::BigIntLiteral(e) => Self::BigIntLiteral(e),
            Expression::RegExpLiteral(e) => Self::RegExpLiteral(e),
            Expression::StringLiteral(e) => Self::StringLiteral(e),
            Expression::TemplateLiteral(e) => Self::TemplateLiteral(e),
            Expression::Identifier(e) => Self::IdentifierReference(e),
            Expression::MetaProperty(e) => Self::MetaProperty(e),
            Expression::Super(e) => Self::Super(e),
            Expression::ArrayExpression(e) => Self::ArrayExpression(e),
            Expression::ArrowFunctionExpression(e) => Self::ArrowFunctionExpression(e),
            Expression::AssignmentExpression(e) => Self::AssignmentExpression(e),
            Expression::AwaitExpression(e) => Self::AwaitExpression(e),
            Expression::BinaryExpression(e) => Self::BinaryExpression(e),
            Expression::CallExpression(e) => Self::CallExpression(e),
            Expression::ChainExpression(e) => Self::ChainExpression(e),
            Expression::ClassExpression(e) => Self::Class(e),
            Expression::ConditionalExpression(e) => Self::ConditionalExpression(e),
            Expression::FunctionExpression(e) => Self::Function(e),
            Expression::ImportExpression(e) => Self::ImportExpression(e),
            Expression::LogicalExpression(e) => Self::LogicalExpression(e),
            match_member_expression!(Expression) => {
                Self::MemberExpression(e.to_member_expression())
            }
            Expression::NewExpression(e) => Self::NewExpression(e),
            Expression::ObjectExpression(e) => Self::ObjectExpression(e),
            Expression::ParenthesizedExpression(e) => Self::ParenthesizedExpression(e),
            Expression::SequenceExpression(e) => Self::SequenceExpression(e),
            Expression::TaggedTemplateExpression(e) => Self::TaggedTemplateExpression(e),
            Expression::ThisExpression(e) => Self::ThisExpression(e),
            Expression::UnaryExpression(e) => Self::UnaryExpression(e),
            Expression::UpdateExpression(e) => Self::UpdateExpression(e),
            Expression::YieldExpression(e) => Self::YieldExpression(e),
            Expression::PrivateInExpression(e) => Self::PrivateInExpression(e),
            Expression::JSXElement(e) => Self::JSXElement(e),
            Expression::JSXFragment(e) => Self::JSXFragment(e),
            Expression::TSAsExpression(e) => Self::TSAsExpression(e),
            Expression::TSSatisfiesExpression(e) => Self::TSSatisfiesExpression(e),
            Expression::TSTypeAssertion(e) => Self::TSTypeAssertion(e),
            Expression::TSNonNullExpression(e) => Self::TSNonNullExpression(e),
            Expression::TSInstantiationExpression(e) => Self::TSInstantiationExpression(e),
        }
    }
}

impl<'a> AstKind<'a> {
    #[allow(clippy::match_same_arms)]
    /// Get the AST kind name with minimal details. Particularly useful for
    /// when debugging an iteration over an AST.
    ///
    /// Note that this method does not exist in release builds. Do not include
    /// usage of this method within your code.
    pub fn debug_name(&self) -> std::borrow::Cow<str> {
        match self {
            Self::Program(_) => "Program".into(),
            Self::Directive(d) => d.directive.as_ref().into(),
            Self::Hashbang(_) => "Hashbang".into(),

            Self::BlockStatement(_) => "BlockStatement".into(),
            Self::BreakStatement(_) => "BreakStatement".into(),
            Self::ContinueStatement(_) => "ContinueStatement".into(),
            Self::DebuggerStatement(_) => "DebuggerStatement".into(),
            Self::DoWhileStatement(_) => "DoWhileStatement".into(),
            Self::EmptyStatement(_) => "EmptyStatement".into(),
            Self::ExpressionStatement(_) => "ExpressionStatement".into(),
            Self::ForInStatement(_) => "ForInStatement".into(),
            Self::ForOfStatement(_) => "ForOfStatement".into(),
            Self::ForStatement(_) => "ForStatement".into(),
            Self::ForStatementInit(_) => "ForStatementInit".into(),
            Self::IfStatement(_) => "IfStatement".into(),
            Self::LabeledStatement(_) => "LabeledStatement".into(),
            Self::ReturnStatement(_) => "ReturnStatement".into(),
            Self::SwitchStatement(_) => "SwitchStatement".into(),
            Self::ThrowStatement(_) => "ThrowStatement".into(),
            Self::TryStatement(_) => "TryStatement".into(),
            Self::WhileStatement(_) => "WhileStatement".into(),
            Self::WithStatement(_) => "WithStatement".into(),

            Self::SwitchCase(_) => "SwitchCase".into(),
            Self::CatchClause(_) => "CatchClause".into(),
            Self::FinallyClause(_) => "FinallyClause".into(),

            Self::VariableDeclaration(_) => "VariableDeclaration".into(),
            Self::VariableDeclarator(_) => "VariableDeclarator".into(),

            Self::UsingDeclaration(_) => "UsingDeclaration".into(),

            Self::IdentifierName(x) => format!("IdentifierName({})", x.name).into(),
            Self::IdentifierReference(x) => format!("IdentifierReference({})", x.name).into(),
            Self::BindingIdentifier(x) => format!("BindingIdentifier({})", x.name).into(),
            Self::LabelIdentifier(x) => format!("LabelIdentifier({})", x.name).into(),
            Self::PrivateIdentifier(x) => format!("PrivateIdentifier({})", x.name).into(),

            Self::NumericLiteral(n) => format!("NumericLiteral({})", n.value).into(),
            Self::StringLiteral(s) => format!("StringLiteral({})", s.value).into(),
            Self::BooleanLiteral(b) => format!("BooleanLiteral({})", b.value).into(),
            Self::NullLiteral(_) => "NullLiteral".into(),
            Self::BigIntLiteral(b) => format!("BigIntLiteral({})", b.raw).into(),
            Self::RegExpLiteral(r) => format!("RegExpLiteral({})", r.regex).into(),
            Self::TemplateLiteral(t) => format!(
                "TemplateLiteral({})",
                t.quasi().map_or_else(|| "None".into(), |q| format!("Some({q})"))
            )
            .into(),

            Self::MetaProperty(_) => "MetaProperty".into(),
            Self::Super(_) => "Super".into(),

            Self::ArrayExpression(_) => "ArrayExpression".into(),
            Self::ArrowFunctionExpression(_) => "ArrowFunctionExpression".into(),
            Self::AssignmentExpression(_) => "AssignmentExpression".into(),
            Self::AwaitExpression(_) => "AwaitExpression".into(),
            Self::BinaryExpression(b) => format!("BinaryExpression{}", b.operator.as_str()).into(),
            Self::CallExpression(_) => "CallExpression".into(),
            Self::ChainExpression(_) => "ChainExpression".into(),
            Self::ConditionalExpression(_) => "ConditionalExpression".into(),
            Self::LogicalExpression(_) => "LogicalExpression".into(),
            Self::MemberExpression(_) => "MemberExpression".into(),
            Self::NewExpression(_) => "NewExpression".into(),
            Self::ObjectExpression(_) => "ObjectExpression".into(),
            Self::ParenthesizedExpression(_) => "ParenthesizedExpression".into(),
            Self::SequenceExpression(_) => "SequenceExpression".into(),
            Self::TaggedTemplateExpression(_) => "TaggedTemplateExpression".into(),
            Self::ThisExpression(_) => "ThisExpression".into(),
            Self::UnaryExpression(expr) => format!("UnaryExpression({:?})", expr.operator).into(),
            Self::UpdateExpression(_) => "UpdateExpression".into(),
            Self::YieldExpression(_) => "YieldExpression".into(),
            Self::ImportExpression(_) => "ImportExpression".into(),
            Self::PrivateInExpression(_) => "PrivateInExpression".into(),

            Self::ObjectProperty(_) => "ObjectProperty".into(),
            Self::PropertyKey(_) => "PropertyKey".into(),
            Self::Argument(_) => "Argument".into(),
            Self::ArrayExpressionElement(_) => "ArrayExpressionElement".into(),
            Self::AssignmentTarget(_) => "AssignmentTarget".into(),
            Self::SimpleAssignmentTarget(_) => "SimpleAssignmentTarget".into(),
            Self::AssignmentTargetWithDefault(_) => "AssignmentTargetWithDefault".into(),
            Self::SpreadElement(_) => "SpreadElement".into(),
            Self::Elision(_) => "Elision".into(),
            Self::ExpressionArrayElement(_) => "ExpressionArrayElement".into(),
            Self::BindingRestElement(_) => "BindingRestElement".into(),

            Self::Function(x) => format!(
                "Function({})",
                x.id.as_ref().map_or_else(|| "<anonymous>", |id| id.name.as_str())
            )
            .into(),
            Self::FunctionBody(_) => "FunctionBody".into(),
            Self::FormalParameters(_) => "FormalParameters".into(),
            Self::FormalParameter(_) => "FormalParameter".into(),
            Self::CatchParameter(_) => "CatchParameter".into(),

            Self::Class(c) => format!(
                "Class({})",
                c.id.as_ref().map_or_else(|| "<anonymous>", |id| id.name.as_str())
            )
            .into(),
            Self::TSClassImplements(_) => "TSClassImplements".into(),
            Self::ClassBody(_) => "ClassBody".into(),
            Self::ClassHeritage(_) => "ClassHeritage".into(),
            Self::StaticBlock(_) => "StaticBlock".into(),
            Self::PropertyDefinition(_) => "PropertyDefinition".into(),
            Self::MethodDefinition(_) => "MethodDefinition".into(),

            Self::ArrayPattern(_) => "ArrayPattern".into(),
            Self::ObjectPattern(_) => "ObjectPattern".into(),
            Self::AssignmentPattern(_) => "AssignmentPattern".into(),

            Self::Decorator(_) => "Decorator".into(),

            Self::ModuleDeclaration(_) => "ModuleDeclaration".into(),
            Self::ImportDeclaration(_) => "ImportDeclaration".into(),
            Self::ImportSpecifier(_) => "ImportSpecifier".into(),
            Self::ExportSpecifier(_) => "ExportSpecifier".into(),
            Self::ImportDefaultSpecifier(_) => "ImportDefaultSpecifier".into(),
            Self::ImportNamespaceSpecifier(_) => "ImportNamespaceSpecifier".into(),
            Self::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration".into(),
            Self::ExportNamedDeclaration(_) => "ExportNamedDeclaration".into(),
            Self::ExportAllDeclaration(_) => "ExportAllDeclaration".into(),
            Self::JSXOpeningElement(_) => "JSXOpeningElement".into(),
            Self::JSXClosingElement(_) => "JSXClosingElement".into(),
            Self::JSXElementName(_) => "JSXElementName".into(),
            Self::JSXElement(_) => "JSXElement".into(),
            Self::JSXFragment(_) => "JSXFragment".into(),
            Self::JSXAttributeItem(_) => "JSXAttributeItem".into(),
            Self::JSXSpreadAttribute(_) => "JSXSpreadAttribute".into(),
            Self::JSXText(_) => "JSXText".into(),
            Self::JSXExpressionContainer(_) => "JSXExpressionContainer".into(),
            Self::JSXIdentifier(_) => "JSXIdentifier".into(),
            Self::JSXMemberExpression(_) => "JSXMemberExpression".into(),
            Self::JSXMemberExpressionObject(_) => "JSXMemberExpressionObject".into(),
            Self::JSXNamespacedName(_) => "JSXNamespacedName".into(),

            Self::TSModuleBlock(_) => "TSModuleBlock".into(),

            Self::TSAnyKeyword(_) => "TSAnyKeyword".into(),
            Self::TSIntersectionType(_) => "TSIntersectionType".into(),
            Self::TSLiteralType(_) => "TSLiteralType".into(),
            Self::TSMethodSignature(_) => "TSMethodSignature".into(),
            Self::TSNullKeyword(_) => "TSNullKeyword".into(),
            Self::TSTypeLiteral(_) => "TSTypeLiteral".into(),
            Self::TSTypeReference(_) => "TSTypeReference".into(),
            Self::TSUnionType(_) => "TSUnionType".into(),
            Self::TSParenthesizedType(_) => "TSParenthesizedType".into(),
            Self::TSVoidKeyword(_) => "TSVoidKeyword".into(),
            Self::TSBigIntKeyword(_) => "TSBigIntKeyword".into(),
            Self::TSBooleanKeyword(_) => "TSBooleanKeyword".into(),
            Self::TSIntrinsicKeyword(_) => "TSIntrinsicKeyword".into(),
            Self::TSNeverKeyword(_) => "TSNeverKeyword".into(),
            Self::TSNumberKeyword(_) => "TSNumberKeyword".into(),
            Self::TSObjectKeyword(_) => "TSObjectKeyword".into(),
            Self::TSStringKeyword(_) => "TSStringKeyword".into(),
            Self::TSSymbolKeyword(_) => "TSSymbolKeyword".into(),
            Self::TSThisType(_) => "TSThisType".into(),
            Self::TSUndefinedKeyword(_) => "TSUndefinedKeyword".into(),
            Self::TSUnknownKeyword(_) => "TSUnknownKeyword".into(),
            Self::TSInferType(_) => "TSInferType".into(),
            Self::TSTemplateLiteralType(_) => "TSTemplateLiteralType".into(),

            Self::TSIndexedAccessType(_) => "TSIndexedAccessType".into(),

            Self::TSAsExpression(_) => "TSAsExpression".into(),
            Self::TSSatisfiesExpression(_) => "TSSatisfiesExpression".into(),
            Self::TSNonNullExpression(_) => "TSNonNullExpression".into(),
            Self::TSInstantiationExpression(_) => "TSInstantiationExpression".into(),

            Self::TSEnumDeclaration(decl) => format!("TSEnumDeclaration({})", &decl.id.name).into(),

            Self::TSEnumMember(_) => "TSEnumMember".into(),

            Self::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration".into(),
            Self::TSTypeName(_) => "TSTypeName".into(),
            Self::TSExternalModuleReference(_) => "TSExternalModuleReference".into(),
            Self::TSQualifiedName(_) => "TSQualifiedName".into(),
            Self::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration".into(),
            Self::TSInterfaceHeritage(_) => "TSInterfaceHeritage".into(),
            Self::TSModuleDeclaration(_) => "TSModuleDeclaration".into(),
            Self::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration".into(),
            Self::TSTypeAnnotation(_) => "TSTypeAnnotation".into(),
            Self::TSTypeQuery(_) => "TSTypeQuery".into(),
            Self::TSTypeAssertion(_) => "TSTypeAssertion".into(),
            Self::TSThisParameter(_) => "TSThisParameter".into(),
            Self::TSTypeParameter(_) => "TSTypeParameter".into(),
            Self::TSTypeParameterDeclaration(_) => "TSTypeParameterDeclaration".into(),
            Self::TSTypeParameterInstantiation(_) => "TSTypeParameterInstantiation".into(),
            Self::TSImportType(_) => "TSImportType".into(),
            Self::TSNamedTupleMember(_) => "TSNamedTupleMember".into(),

            Self::TSPropertySignature(_) => "TSPropertySignature".into(),
            Self::TSConditionalType(_) => "TSConditionalType".into(),
            Self::TSMappedType(_) => "TSMappedType".into(),
            Self::TSConstructSignatureDeclaration(_) => "TSConstructSignatureDeclaration".into(),
            Self::TSModuleReference(_) => "TSModuleReference".into(),
        }
    }
}
