#![expect(missing_docs)] // FIXME

use oxc_allocator::{Address, GetAddress};
use oxc_span::{Atom, GetSpan};

use super::{AstKind, ast::*};

impl<'a> AstKind<'a> {
    #[rustfmt::skip]
    pub fn is_statement(self) -> bool {
        self.is_iteration_statement()
            || matches!(self, Self::BlockStatement(_) | Self::BreakStatement(_) | Self::ContinueStatement(_)
                    | Self::DebuggerStatement(_) | Self::EmptyStatement(_) | Self::ExpressionStatement(_)
                    | Self::LabeledStatement(_) | Self::ReturnStatement(_) | Self::SwitchStatement(_)
                    | Self::ThrowStatement(_) | Self::TryStatement(_) | Self::WithStatement(_)
                    | Self::IfStatement(_) | Self::VariableDeclaration(_) | Self::ExportDefaultDeclaration(_))
    }

    #[rustfmt::skip]
    pub fn is_declaration(self) -> bool {
        matches!(self, Self::Function(func) if func.is_declaration())
        || matches!(self, Self::Class(class) if class.is_declaration())
        || matches!(self, Self::TSEnumDeclaration(_) | Self::TSModuleDeclaration(_)
            | Self::VariableDeclaration(_) | Self::TSInterfaceDeclaration(_)
            | Self::TSTypeAliasDeclaration(_) | Self::TSImportEqualsDeclaration(_) | Self::PropertyDefinition(_)
        ) || self.is_module_declaration()
    }

    pub fn is_module_declaration(self) -> bool {
        self.as_module_declaration_kind().is_some()
    }

    pub fn as_module_declaration_kind(&self) -> Option<ModuleDeclarationKind<'a>> {
        match self {
            Self::ImportDeclaration(decl) => Some(ModuleDeclarationKind::Import(decl)),
            Self::ExportAllDeclaration(decl) => Some(ModuleDeclarationKind::ExportAll(decl)),
            Self::ExportNamedDeclaration(decl) => Some(ModuleDeclarationKind::ExportNamed(decl)),
            Self::ExportDefaultDeclaration(decl) => {
                Some(ModuleDeclarationKind::ExportDefault(decl))
            }
            Self::TSExportAssignment(decl) => Some(ModuleDeclarationKind::TSExportAssignment(decl)),
            Self::TSNamespaceExportDeclaration(decl) => {
                Some(ModuleDeclarationKind::TSNamespaceExport(decl))
            }
            _ => None,
        }
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
            Self::BindingIdentifier(ident) => Some(ident.name),
            Self::IdentifierReference(ident) => Some(ident.name),
            Self::LabelIdentifier(ident) => Some(ident.name),
            Self::IdentifierName(ident) => Some(ident.name),
            _ => None,
        }
    }

    pub fn is_specific_id_reference(&self, name: &str) -> bool {
        match self {
            Self::IdentifierReference(ident) => ident.name == name,
            _ => false,
        }
    }

    /// Returns whether this expression is a member expression, such as `obj.prop`, `obj["prop"]`, or `obj.#prop`.
    pub fn is_member_expression_kind(&self) -> bool {
        self.as_member_expression_kind().is_some()
    }

    /// If this is some kind of member expression, returns it as a
    /// [`MemberExpressionKind`]. Otherwise, returns `None`.
    pub fn as_member_expression_kind(&self) -> Option<MemberExpressionKind<'a>> {
        match self {
            Self::ComputedMemberExpression(member_expr) => {
                Some(MemberExpressionKind::Computed(member_expr))
            }
            Self::StaticMemberExpression(member_expr) => {
                Some(MemberExpressionKind::Static(member_expr))
            }
            Self::PrivateFieldExpression(member_expr) => {
                Some(MemberExpressionKind::PrivateField(member_expr))
            }
            _ => None,
        }
    }

    pub fn is_property_key(&self) -> bool {
        self.as_property_key_kind().is_some()
    }

    pub fn as_property_key_kind(&self) -> Option<PropertyKeyKind<'a>> {
        match self {
            Self::IdentifierName(ident) => Some(PropertyKeyKind::Static(ident)),
            Self::PrivateIdentifier(ident) => Some(PropertyKeyKind::Private(ident)),
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
            Expression::ComputedMemberExpression(e) => Self::ComputedMemberExpression(e),
            Expression::ConditionalExpression(e) => Self::ConditionalExpression(e),
            Expression::FunctionExpression(e) => Self::Function(e),
            Expression::ImportExpression(e) => Self::ImportExpression(e),
            Expression::LogicalExpression(e) => Self::LogicalExpression(e),
            Expression::NewExpression(e) => Self::NewExpression(e),
            Expression::ObjectExpression(e) => Self::ObjectExpression(e),
            Expression::ParenthesizedExpression(e) => Self::ParenthesizedExpression(e),
            Expression::PrivateFieldExpression(e) => Self::PrivateFieldExpression(e),
            Expression::StaticMemberExpression(e) => Self::StaticMemberExpression(e),
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
            Expression::V8IntrinsicExpression(e) => Self::V8IntrinsicExpression(e),
        }
    }

    /// Given an [`AstKind`] which is an [`IdentifierReference`], and [`AstKind`] representing its parent node,
    /// returns `true` if the identifier is assigned to.
    ///
    /// Note: `parent` must be the parent node of the identifier, and `self` must be an `IdentifierReference`.
    /// This method does not check those things, and may return incorrect results if either is not true.
    //
    // This method's implementation is identical to `MemberExpressionKind::is_assigned_to_in_parent`
    // except for addition of `AssignmentTargetPropertyIdentifier`.
    pub fn ident_reference_is_assigned_to_in_parent(&self, parent: &AstKind<'a>) -> bool {
        debug_assert!(matches!(self, AstKind::IdentifierReference(_)));

        #[expect(clippy::match_same_arms)]
        match parent {
            // `ident++`, `--ident`
            // `UpdateExpression` has only 1 field containing child node - `argument`.
            AstKind::UpdateExpression(_) => true,
            // `[ident] = arr`
            // `ArrayAssignmentTarget` has only 1 field containing child nodes - `elements`.
            AstKind::ArrayAssignmentTarget(_) => true,
            // `[...ident] = arr`, `({ ...ident } = obj)`
            // `AssignmentTargetRest` has only 1 field containing child node - `target`.
            AstKind::AssignmentTargetRest(_) => true,
            // `ident = value`
            // Only match if ident is on left
            // - not on right e.g. `assignee = ident`.
            AstKind::AssignmentExpression(assign_expr) => {
                assign_expr.left.address() == self.address()
            }
            // `[ident = value] = arr`, `({ prop: ident = value } = obj)`
            // Only match if ident is the assignee
            // - not the default value e.g. `[assignee = ident] = arr`.
            AstKind::AssignmentTargetWithDefault(assign_target) => {
                assign_target.binding.address() == self.address()
            }
            // `({ ident } = obj)`
            // Only match if ident is the assignee
            // - not the default value e.g. `({ assignee = ident } = obj)`.
            AstKind::AssignmentTargetPropertyIdentifier(assign_target) => {
                let binding = &assign_target.binding;
                Address::from_ptr(binding) == self.address()
            }
            // `({ prop: ident } = obj)`
            // Only match if ident is the assignee
            // - not computed prop key e.g. `({ [ident]: assignee } = obj)`.
            AstKind::AssignmentTargetPropertyProperty(assign_target) => {
                assign_target.binding.address() == self.address()
            }
            // `for (ident in obj)`
            // Only match if ident is on left
            // - not object being iterated e.g. `for (assignee in ident)`
            AstKind::ForInStatement(for_stmt) => for_stmt.left.address() == self.address(),
            // `for (ident of obj)`
            // Only match if ident is on left
            // - not array being iterated e.g. `for (assignee of ident)`
            AstKind::ForOfStatement(for_stmt) => for_stmt.left.address() == self.address(),
            _ => false,
        }
    }
}

impl AstKind<'_> {
    /// Get the AST kind name with minimal details. Particularly useful for
    /// when debugging an iteration over an AST.
    ///
    /// Note that this method does not exist in release builds. Do not include
    /// usage of this method within your code.
    pub fn debug_name(&self) -> std::borrow::Cow<str> {
        use std::borrow::Cow;

        const COMPUTED: Cow<'static, str> = Cow::Borrowed("<computed>");
        const ANONYMOUS: Cow<'static, str> = Cow::Borrowed("<anonymous>");
        const DESTRUCTURE: Cow<'static, str> = Cow::Borrowed("<destructure>");

        #[inline]
        fn or_anonymous<'a>(id: Option<&BindingIdentifier<'a>>) -> Cow<'a, str> {
            id.map_or_else(|| ANONYMOUS.as_ref(), |id| id.name.as_str()).into()
        }

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
            Self::IfStatement(_) => "IfStatement".into(),
            Self::LabeledStatement(l) => format!("LabeledStatement({})", l.label.name).into(),
            Self::ReturnStatement(_) => "ReturnStatement".into(),
            Self::SwitchStatement(_) => "SwitchStatement".into(),
            Self::ThrowStatement(_) => "ThrowStatement".into(),
            Self::TryStatement(_) => "TryStatement".into(),
            Self::WhileStatement(_) => "WhileStatement".into(),
            Self::WithStatement(_) => "WithStatement".into(),

            Self::SwitchCase(_) => "SwitchCase".into(),
            Self::CatchClause(_) => "CatchClause".into(),

            Self::VariableDeclaration(_) => "VariableDeclaration".into(),
            Self::VariableDeclarator(v) => format!(
                "VariableDeclarator({})",
                v.id.get_identifier_name().unwrap_or(Atom::from(DESTRUCTURE.as_ref()))
            )
            .into(),

            Self::IdentifierName(x) => format!("IdentifierName({})", x.name).into(),
            Self::IdentifierReference(x) => format!("IdentifierReference({})", x.name).into(),
            Self::BindingIdentifier(x) => format!("BindingIdentifier({})", x.name).into(),
            Self::LabelIdentifier(x) => format!("LabelIdentifier({})", x.name).into(),
            Self::PrivateIdentifier(x) => format!("PrivateIdentifier({})", x.name).into(),

            Self::NumericLiteral(n) => format!("NumericLiteral({})", n.value).into(),
            Self::StringLiteral(s) => format!("StringLiteral({})", s.value).into(),
            Self::BooleanLiteral(b) => format!("BooleanLiteral({})", b.value).into(),
            Self::NullLiteral(_) => "NullLiteral".into(),
            Self::BigIntLiteral(b) => format!("BigIntLiteral({})", b.value).into(),
            Self::RegExpLiteral(r) => format!("RegExpLiteral({})", r.regex).into(),
            Self::TemplateLiteral(t) => format!(
                "TemplateLiteral({})",
                t.single_quasi().map_or_else(|| "None".into(), |q| format!("Some({q})"))
            )
            .into(),
            Self::TemplateElement(_) => "TemplateElement".into(),

            Self::MetaProperty(_) => "MetaProperty".into(),
            Self::Super(_) => "Super".into(),

            Self::AccessorProperty(_) => "AccessorProperty".into(),

            Self::BindingProperty(_) => "BindingProperty".into(),

            Self::ArrayExpression(_) => "ArrayExpression".into(),
            Self::ArrowFunctionExpression(_) => "ArrowFunctionExpression".into(),
            Self::AssignmentExpression(_) => "AssignmentExpression".into(),
            Self::AwaitExpression(_) => "AwaitExpression".into(),
            Self::BinaryExpression(b) => {
                format!("BinaryExpression({})", b.operator.as_str()).into()
            }
            Self::CallExpression(c) => {
                format!("CallExpression({})", c.callee_name().unwrap_or(&COMPUTED)).into()
            }
            Self::ChainExpression(_) => "ChainExpression".into(),
            Self::ComputedMemberExpression(_) => "ComputedMemberExpression".into(),
            Self::ConditionalExpression(_) => "ConditionalExpression".into(),
            Self::LogicalExpression(_) => "LogicalExpression".into(),
            Self::NewExpression(n) => {
                let callee = match &n.callee {
                    Expression::Identifier(id) => Some(id.name.as_str()),
                    match_member_expression!(Expression) => {
                        n.callee.to_member_expression().static_property_name()
                    }
                    _ => None,
                };
                format!("NewExpression({})", callee.unwrap_or(&COMPUTED)).into()
            }
            Self::ObjectExpression(_) => "ObjectExpression".into(),
            Self::ParenthesizedExpression(_) => "ParenthesizedExpression".into(),
            Self::PrivateFieldExpression(_) => "PrivateFieldExpression".into(),
            Self::StaticMemberExpression(_) => "StaticMemberExpression".into(),
            Self::SequenceExpression(_) => "SequenceExpression".into(),
            Self::TaggedTemplateExpression(_) => "TaggedTemplateExpression".into(),
            Self::ThisExpression(_) => "ThisExpression".into(),
            Self::UnaryExpression(expr) => format!("UnaryExpression({:?})", expr.operator).into(),
            Self::UpdateExpression(_) => "UpdateExpression".into(),
            Self::YieldExpression(_) => "YieldExpression".into(),
            Self::ImportExpression(_) => "ImportExpression".into(),
            Self::PrivateInExpression(_) => "PrivateInExpression".into(),

            Self::ObjectProperty(p) => {
                format!("ObjectProperty({})", p.key.name().unwrap_or(COMPUTED)).into()
            }
            Self::ArrayAssignmentTarget(_) => "ArrayAssignmentTarget".into(),
            Self::ObjectAssignmentTarget(_) => "ObjectAssignmentTarget".into(),
            Self::AssignmentTargetWithDefault(_) => "AssignmentTargetWithDefault".into(),
            Self::SpreadElement(_) => "SpreadElement".into(),
            Self::Elision(_) => "Elision".into(),
            Self::BindingRestElement(_) => "BindingRestElement".into(),

            Self::Function(x) => format!("Function({})", or_anonymous(x.id.as_ref())).into(),
            Self::FunctionBody(_) => "FunctionBody".into(),
            Self::FormalParameters(_) => "FormalParameters".into(),
            Self::FormalParameter(p) => format!(
                "FormalParameter({})",
                p.pattern.get_identifier_name().unwrap_or(Atom::from(DESTRUCTURE.as_ref()))
            )
            .into(),
            Self::CatchParameter(_) => "CatchParameter".into(),

            Self::Class(c) => format!("Class({})", or_anonymous(c.id.as_ref())).into(),
            Self::TSClassImplements(_) => "TSClassImplements".into(),
            Self::ClassBody(_) => "ClassBody".into(),
            Self::StaticBlock(_) => "StaticBlock".into(),
            Self::PropertyDefinition(_) => "PropertyDefinition".into(),
            Self::MethodDefinition(_) => "MethodDefinition".into(),

            Self::ArrayPattern(_) => "ArrayPattern".into(),
            Self::ObjectPattern(_) => "ObjectPattern".into(),
            Self::AssignmentPattern(_) => "AssignmentPattern".into(),

            Self::Decorator(_) => "Decorator".into(),

            Self::ImportDeclaration(_) => "ImportDeclaration".into(),
            Self::ImportSpecifier(i) => format!("ImportSpecifier({})", i.local.name).into(),
            Self::ExportSpecifier(e) => format!("ExportSpecifier({})", e.local.name()).into(),
            Self::ImportDefaultSpecifier(_) => "ImportDefaultSpecifier".into(),
            Self::ImportNamespaceSpecifier(_) => "ImportNamespaceSpecifier".into(),
            Self::ImportAttribute(_) => "ImportAttribute".into(),
            Self::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration".into(),
            Self::ExportNamedDeclaration(_) => "ExportNamedDeclaration".into(),
            Self::ExportAllDeclaration(_) => "ExportAllDeclaration".into(),
            Self::WithClause(_) => "WithClause".into(),
            Self::JSXOpeningElement(_) => "JSXOpeningElement".into(),
            Self::JSXClosingElement(_) => "JSXClosingElement".into(),
            Self::JSXElement(_) => "JSXElement".into(),
            Self::JSXFragment(_) => "JSXFragment".into(),
            Self::JSXOpeningFragment(_) => "JSXOpeningFragment".into(),
            Self::JSXClosingFragment(_) => "JSXClosingFragment".into(),
            Self::JSXEmptyExpression(_) => "JSXEmptyExpression".into(),
            Self::JSXSpreadChild(_) => "JSXSpreadChild".into(),
            Self::JSXAttribute(_) => "JSXAttribute".into(),
            Self::JSXSpreadAttribute(_) => "JSXSpreadAttribute".into(),
            Self::JSXText(_) => "JSXText".into(),
            Self::JSXExpressionContainer(_) => "JSXExpressionContainer".into(),
            Self::JSXIdentifier(id) => format!("JSXIdentifier({id})").into(),
            Self::JSXMemberExpression(_) => "JSXMemberExpression".into(),
            Self::JSXNamespacedName(_) => "JSXNamespacedName".into(),

            Self::TSModuleBlock(_) => "TSModuleBlock".into(),

            Self::TSTupleType(_) => "TSTupleType".into(),
            Self::TSAnyKeyword(_) => "TSAnyKeyword".into(),
            Self::TSIntersectionType(_) => "TSIntersectionType".into(),
            Self::TSLiteralType(_) => "TSLiteralType".into(),
            Self::TSMethodSignature(_) => "TSMethodSignature".into(),
            Self::TSNullKeyword(_) => "TSNullKeyword".into(),
            Self::TSTypeLiteral(_) => "TSTypeLiteral".into(),
            Self::TSTypeReference(t) => format!("TSTypeReference({})", t.type_name).into(),
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
            Self::TSArrayType(_) => "TSArrayType".into(),
            Self::TSOptionalType(_) => "TSOptionalType".into(),
            Self::TSTypeOperator(_) => "TSTypeOperator".into(),
            Self::TSFunctionType(_) => "TSFunctionType".into(),

            Self::TSIndexedAccessType(_) => "TSIndexedAccessType".into(),

            Self::TSRestType(_) => "TSRestType".into(),

            Self::TSAsExpression(_) => "TSAsExpression".into(),
            Self::TSSatisfiesExpression(_) => "TSSatisfiesExpression".into(),
            Self::TSNonNullExpression(_) => "TSNonNullExpression".into(),
            Self::TSInstantiationExpression(_) => "TSInstantiationExpression".into(),

            Self::TSEnumDeclaration(decl) => format!("TSEnumDeclaration({})", &decl.id.name).into(),
            Self::TSEnumBody(_) => "TSEnumBody".into(),
            Self::TSEnumMember(_) => "TSEnumMember".into(),

            Self::TSNamespaceExportDeclaration(_) => "TSNamespaceExportDeclaration".into(),
            Self::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration".into(),
            Self::TSCallSignatureDeclaration(_) => "TSCallSignatureDeclaration".into(),
            Self::TSExternalModuleReference(_) => "TSExternalModuleReference".into(),
            Self::TSQualifiedName(n) => format!("TSQualifiedName({n})").into(),
            Self::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration".into(),
            Self::TSInterfaceHeritage(_) => "TSInterfaceHeritage".into(),
            Self::TSModuleDeclaration(m) => format!("TSModuleDeclaration({})", m.id).into(),
            Self::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration".into(),
            Self::TSTypeAnnotation(_) => "TSTypeAnnotation".into(),
            Self::TSTypeQuery(_) => "TSTypeQuery".into(),
            Self::TSTypeAssertion(_) => "TSTypeAssertion".into(),
            Self::TSThisParameter(_) => "TSThisParameter".into(),
            Self::TSTypeParameter(t) => format!("TSTypeParameter({})", t.name).into(),
            Self::TSTypeParameterDeclaration(_) => "TSTypeParameterDeclaration".into(),
            Self::TSTypeParameterInstantiation(_) => "TSTypeParameterInstantiation".into(),
            Self::TSTypePredicate(_) => "TSTypePredicate".into(),
            Self::TSImportType(_) => "TSImportType".into(),
            Self::TSNamedTupleMember(_) => "TSNamedTupleMember".into(),

            Self::TSPropertySignature(_) => "TSPropertySignature".into(),
            Self::TSIndexSignatureName(_) => "TSIndexSignatureName".into(),
            Self::TSConditionalType(_) => "TSConditionalType".into(),
            Self::TSMappedType(_) => "TSMappedType".into(),
            Self::TSConstructSignatureDeclaration(_) => "TSConstructSignatureDeclaration".into(),
            Self::TSExportAssignment(_) => "TSExportAssignment".into(),
            Self::TSConstructorType(_) => "TSConstructorType".into(),
            Self::TSInterfaceBody(_) => "TSInterfaceBody".into(),
            Self::TSIndexSignature(_) => "TSIndexSignature".into(),
            Self::V8IntrinsicExpression(_) => "V8IntrinsicExpression".into(),

            Self::JSDocNullableType(_) => "JSDocNullableType".into(),
            Self::JSDocNonNullableType(_) => "JSDocNonNullableType".into(),
            Self::JSDocUnknownType(_) => "JSDocUnknownType".into(),
            Self::AssignmentTargetRest(_) => "AssignmentTargetRest".into(),
            Self::AssignmentTargetPropertyIdentifier(_) => {
                "AssignmentTargetPropertyIdentifier".into()
            }
            Self::AssignmentTargetPropertyProperty(_) => "AssignmentTargetPropertyProperty".into(),
        }
    }
}

/// This is a subset of [`AstKind`] that represents member expressions.
///
/// Having a separate enum for this allows us to implement helpful methods that are specific to member expressions,
/// such as getting the property name or the object of the member expression.
#[derive(Debug, Clone, Copy)]
pub enum MemberExpressionKind<'a> {
    /// A static member expression, such as `obj.prop`.
    Static(&'a StaticMemberExpression<'a>),
    /// A computed member expression, such as `obj["prop"]`.
    Computed(&'a ComputedMemberExpression<'a>),
    /// A private field expression, such as `obj.#field`.
    PrivateField(&'a PrivateFieldExpression<'a>),
}

impl<'a> MemberExpressionKind<'a> {
    /// Returns the property name of the member expression, otherwise `None`.
    ///
    /// Example: returns the `prop` in `obj.prop` or `obj["prop"]`.
    pub fn static_property_name(&self) -> Option<Atom<'a>> {
        match self {
            Self::Computed(member_expr) => member_expr.static_property_name(),
            Self::Static(member_expr) => Some(member_expr.property.name),
            Self::PrivateField(_) => None,
        }
    }

    /// Returns the static property name of this member expression, if it has one, along with the source code [`Span`],
    /// or `None` otherwise.
    ///
    /// If you don't need the [`Span`], use [`MemberExpressionKind::static_property_name`] instead.
    pub fn static_property_info(&self) -> Option<(Span, &'a str)> {
        match self {
            Self::Computed(expr) => match &expr.expression {
                Expression::StringLiteral(lit) => Some((lit.span, lit.value.as_str())),
                Expression::TemplateLiteral(lit) => {
                    if lit.quasis.len() == 1 {
                        lit.quasis[0].value.cooked.map(|cooked| (lit.span, cooked.as_str()))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Self::Static(expr) => Some((expr.property.span, expr.property.name.as_str())),
            Self::PrivateField(_) => None,
        }
    }

    /// Returns the object of the member expression, otherwise `None`.
    ///
    /// Example: returns the `obj` in `obj.prop` or `obj["prop"]`.
    pub fn object(&self) -> &Expression<'a> {
        match self {
            Self::Computed(member_expr) => &member_expr.object,
            Self::Static(member_expr) => &member_expr.object,
            Self::PrivateField(member_expr) => &member_expr.object,
        }
    }

    /// Returns whether the member expression is optional, i.e. if it uses the
    /// optional chaining operator (`?.`).
    ///
    /// Example:
    /// - Returns `true` for `obj?.prop` or `obj?.["prop"]`.
    /// - Returns `false` for `obj.prop` or `obj["prop"]`.
    pub fn optional(&self) -> bool {
        match self {
            Self::Computed(member_expr) => member_expr.optional,
            Self::Static(member_expr) => member_expr.optional,
            Self::PrivateField(member_expr) => member_expr.optional,
        }
    }

    /// Given a [`MemberExpressionKind`] and [`AstKind`] representing its parent node,
    /// returns `true` if the member expression is assigned to.
    ///
    /// Note: `parent` must be the parent node of the member expression.
    /// This method does not check that, and may return incorrect results if it's not.
    pub fn is_assigned_to_in_parent(&self, parent: &AstKind<'a>) -> bool {
        #[expect(clippy::match_same_arms)]
        match parent {
            // `x.y++`, `--x.y`
            // `UpdateExpression` has only 1 field containing child node - `argument`.
            AstKind::UpdateExpression(_) => true,
            // `[x.y] = arr`
            // `ArrayAssignmentTarget` has only 1 field containing child nodes - `elements`.
            AstKind::ArrayAssignmentTarget(_) => true,
            // `[...x.y] = arr`, `({ ...x.y } = obj)`
            // `AssignmentTargetRest` has only 1 field containing child node - `target`.
            AstKind::AssignmentTargetRest(_) => true,
            // `x.y = value`
            // Only match if member expr is on left
            // - not on right e.g. `assignee = x.y`.
            AstKind::AssignmentExpression(assign_expr) => {
                assign_expr.left.address() == self.address()
            }
            // `[x.y = value] = arr`, `({ prop: x.y = value } = obj)`
            // Only match if member expr is the assignee
            // - not the default value e.g. `[assignee = x.y] = arr`.
            AstKind::AssignmentTargetWithDefault(assign_target) => {
                assign_target.binding.address() == self.address()
            }
            // `({ prop: x.y } = obj)`
            // Only match if member expr is the assignee
            // - not computed prop key e.g. `({ [x.y]: assignee } = obj)`.
            AstKind::AssignmentTargetPropertyProperty(assign_target) => {
                assign_target.binding.address() == self.address()
            }
            // `for (x.y in obj)`
            // Only match if member expr is on left
            // - not object being iterated e.g. `for (assignee in x.y)`
            AstKind::ForInStatement(for_stmt) => for_stmt.left.address() == self.address(),
            // `for (x.y of obj)`
            // Only match if member expr is on left
            // - not array being iterated e.g. `for (assignee of x.y)`
            AstKind::ForOfStatement(for_stmt) => for_stmt.left.address() == self.address(),
            _ => false,
        }
    }
}

impl GetSpan for MemberExpressionKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Computed(member_expr) => member_expr.span,
            Self::Static(member_expr) => member_expr.span,
            Self::PrivateField(member_expr) => member_expr.span,
        }
    }
}

impl GetAddress for MemberExpressionKind<'_> {
    #[inline] // This should boil down to a single instruction
    fn address(&self) -> Address {
        match *self {
            Self::Computed(member_expr) => Address::from_ptr(member_expr),
            Self::Static(member_expr) => Address::from_ptr(member_expr),
            Self::PrivateField(member_expr) => Address::from_ptr(member_expr),
        }
    }
}

pub enum ModuleDeclarationKind<'a> {
    Import(&'a ImportDeclaration<'a>),
    ExportAll(&'a ExportAllDeclaration<'a>),
    ExportNamed(&'a ExportNamedDeclaration<'a>),
    ExportDefault(&'a ExportDefaultDeclaration<'a>),
    TSExportAssignment(&'a TSExportAssignment<'a>),
    TSNamespaceExport(&'a TSNamespaceExportDeclaration<'a>),
}

impl ModuleDeclarationKind<'_> {
    /// Returns whether this module declaration is an `export` declaration.
    pub fn is_export(&self) -> bool {
        matches!(
            self,
            Self::ExportAll(_)
                | Self::ExportNamed(_)
                | Self::ExportDefault(_)
                | Self::TSExportAssignment(_)
                | Self::TSNamespaceExport(_)
        )
    }
}

impl GetSpan for ModuleDeclarationKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Import(decl) => decl.span,
            Self::ExportAll(decl) => decl.span,
            Self::ExportNamed(decl) => decl.span,
            Self::ExportDefault(decl) => decl.span,
            Self::TSExportAssignment(decl) => decl.span,
            Self::TSNamespaceExport(decl) => decl.span,
        }
    }
}

pub enum PropertyKeyKind<'a> {
    /// A static identifier property key, like `a` in `{ a: 1 }`.
    Static(&'a IdentifierName<'a>),
    /// A private identifier property key, like `#a` in `class C { #a = 1 }`.
    Private(&'a PrivateIdentifier<'a>),
}

impl GetSpan for PropertyKeyKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Static(ident) => ident.span,
            Self::Private(ident) => ident.span,
        }
    }
}
