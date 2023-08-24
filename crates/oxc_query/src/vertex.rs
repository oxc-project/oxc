use std::rc::Rc;

use enum_as_inner::EnumAsInner;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{GetSpan, Span};
use trustfall::provider::{Typename, VertexIterator};
use url::Url;

use crate::{
    util::{
        calculate_hash, jsx_attribute_to_constant_string,
        try_get_constant_string_field_value_from_template_lit,
    },
    Adapter,
};

#[non_exhaustive]
#[derive(Debug, Clone, EnumAsInner)]
pub enum Vertex<'a> {
    ASTNode(AstNode<'a>),
    AssignmentType(&'a BindingPatternKind<'a>),
    Class(Rc<ClassVertex<'a>>),
    ClassMethod(Rc<ClassMethodVertex<'a>>),
    ClassProperty(Rc<ClassPropertyVertex<'a>>),
    DefaultImport(&'a ImportDefaultSpecifier),
    Expression(&'a Expression<'a>),
    File,
    Import(Rc<ImportVertex<'a>>),
    Interface(Rc<InterfaceVertex<'a>>),
    InterfaceExtend(Rc<InterfaceExtendVertex<'a>>),
    JSXAttribute(&'a JSXAttribute<'a>),
    JSXElement(Rc<JSXElementVertex<'a>>),
    JSXExpressionContainer(&'a JSXExpressionContainer<'a>),
    JSXFragment(&'a JSXFragment<'a>),
    JSXOpeningElement(Rc<JSXOpeningElementVertex<'a>>),
    JSXSpreadAttribute(&'a JSXSpreadAttribute<'a>),
    JSXSpreadChild(&'a JSXSpreadChild<'a>),
    JSXText(&'a JSXText),
    ObjectLiteral(Rc<ObjectLiteralVertex<'a>>),
    NumberLiteral(Rc<NumberLiteralVertex<'a>>),
    Name(Rc<NameVertex<'a>>),
    PathPart(usize),
    SearchParameter(Rc<SearchParameterVertex>),
    Span(Span),
    SpecificImport(&'a ImportSpecifier),
    TypeAnnotation(Rc<TypeAnnotationVertex<'a>>),
    Type(&'a TSType<'a>),
    Url(Rc<Url>),
    VariableDeclaration(Rc<VariableDeclarationVertex<'a>>),
    Return(Rc<ReturnVertex<'a>>),
    IfStatementAST(Rc<IfStatementVertex<'a>>),
    SpreadIntoObject(Rc<SpreadIntoObjectVertex<'a>>),
    ObjectEntry(Rc<ObjectEntryVertex<'a>>),
    DotProperty(Rc<DotPropertyVertex<'a>>),
    Reassignment(Rc<ReassignmentVertex<'a>>),
    FnCall(Rc<FnCallVertex<'a>>),
    FnDeclaration(Rc<FnDeclarationVertex<'a>>),
    ArrowFunction(Rc<ArrowFunctionVertex<'a>>),
    Argument(Rc<ArgumentVertex<'a>>),
    FunctionBody(Rc<FunctionBodyVertex<'a>>),
    Statement(&'a Statement<'a>),
    Parameter(Rc<ParameterVertex<'a>>),
    LogicalExpression(Rc<LogicalExpressionVertex<'a>>),
    UnaryExpression(Rc<UnaryExpressionVertex<'a>>),
    ExpressionStatement(Rc<ExpressionStatementVertex<'a>>),
    WhileStatement(Rc<WhileStatementVertex<'a>>),
    BlockStatement(Rc<BlockStatementVertex<'a>>),
    VarRef(Rc<VarRefVertex<'a>>),
    DoWhileStatement(Rc<DoWhileStatementVertex<'a>>),
    ForStatement(Rc<ForStatementVertex<'a>>),
    TernaryExpression(Rc<TernaryExpressionVertex<'a>>),
    New(Rc<NewVertex<'a>>),
    Throw(Rc<ThrowVertex<'a>>),
    Array(Rc<ArrayVertex<'a>>),
    ArrayElement(Rc<ArrayElementVertex<'a>>),
    StringLiteral(Rc<StringLiteralVertex<'a>>),
    TemplateLiteral(Rc<TemplateLiteralVertex<'a>>),
    RegExpLiteral(Rc<RegExpLiteralVertex<'a>>),
    ParenthesizedExpression(Rc<ParenthesizedExpressionVertex<'a>>),
}

impl<'a> Vertex<'a> {
    pub fn span(&self) -> Span {
        match &self {
            Self::AssignmentType(data) => data.span(),
            Self::ASTNode(data) => data.kind().span(),
            Self::Class(data) => data.class.span,
            Self::ClassMethod(data) => data.method.span,
            Self::ClassProperty(data) => data.property.span,
            Self::DefaultImport(data) => data.span,
            Self::Expression(data) => data.span(),
            Self::Import(data) => data.import.span,
            Self::Interface(data) => data.interface.span,
            Self::InterfaceExtend(data) => match **data {
                InterfaceExtendVertex::Identifier(ident) => ident.span,
                InterfaceExtendVertex::MemberExpression(membexpr) => (*membexpr).span(),
            },
            Self::JSXAttribute(data) => data.span,
            Self::JSXElement(data) => data.element.span,
            Self::JSXExpressionContainer(data) => data.span,
            Self::JSXFragment(data) => data.span,
            Self::JSXOpeningElement(data) => data.opening_element.span,
            Self::DotProperty(data) => data.static_member_expr.span,
            Self::JSXSpreadAttribute(data) => data.span,
            Self::JSXSpreadChild(data) => data.span,
            Self::JSXText(data) => data.span,
            Self::ObjectLiteral(data) => data.object_expression.span,
            Self::SpreadIntoObject(data) => data.property.span,
            Self::ObjectEntry(data) => data.property.span,
            Self::SpecificImport(data) => data.span,
            Self::TypeAnnotation(data) => data.type_annotation.span,
            Self::Type(data) => data.span(),
            Self::VariableDeclaration(data) => data.variable_declaration.span,
            Self::Return(data) => data.return_statement.span,
            Self::IfStatementAST(data) => data.return_statement.span,
            Self::NumberLiteral(data) => data.number_literal.span,
            Self::Reassignment(data) => data.assignment_expression.span,
            Self::Name(data) => data.name.span,
            Self::FnCall(data) => data.call_expression.span,
            Self::Argument(data) => data.argument.span(),
            Self::FnDeclaration(data) => data.function.span,
            Self::ArrowFunction(data) => data.arrow_expression.span,
            Self::FunctionBody(data) => data.function_body.span,
            Self::Statement(data) => data.span(),
            Self::Parameter(data) => data.parameter.span,
            Self::LogicalExpression(data) => data.logical_expression.span,
            Self::UnaryExpression(data) => data.unary_expression.span,
            Self::ExpressionStatement(data) => data.expression_statement.span,
            Self::WhileStatement(data) => data.while_statement.span,
            Self::DoWhileStatement(data) => data.do_while_statement.span,
            Self::BlockStatement(data) => data.block_statement.span,
            Self::VarRef(data) => data.identifier_reference.span,
            Self::ForStatement(data) => data.for_statement.span,
            Self::TernaryExpression(data) => data.conditional_expression.span,
            Self::New(data) => data.new_expression.span,
            Self::Throw(data) => data.throw_statement.span,
            Self::Array(data) => data.array_expression.span,
            Self::ArrayElement(data) => data.array_expression_element.span(),
            Self::StringLiteral(data) => data.string.span,
            Self::TemplateLiteral(data) => data.template.span,
            Self::RegExpLiteral(data) => data.regexp.span,
            Self::ParenthesizedExpression(data) => data.parenthesized_expression.span,
            Self::File
            | Self::Url(_)
            | Self::PathPart(_)
            | Self::SearchParameter(_)
            | Self::Span(_) => {
                unreachable!("Tried to get the span from a {self:#?}")
            }
        }
    }

    pub fn ast_node_id(&self) -> Option<AstNodeId> {
        match &self {
            Vertex::ASTNode(data) => Some(data.id()),
            Vertex::Class(data) => data.ast_node.map(|x| x.id()),
            Vertex::Import(data) => data.ast_node.map(|x| x.id()),
            Vertex::Interface(data) => data.ast_node.map(|x| x.id()),
            Vertex::JSXElement(data) => data.ast_node.map(|x| x.id()),
            Vertex::TypeAnnotation(data) => data.ast_node.map(|x| x.id()),
            Vertex::VariableDeclaration(data) => data.ast_node.map(|x| x.id()),
            Vertex::ObjectLiteral(data) => data.ast_node.map(|x| x.id()),
            Vertex::Return(data) => data.ast_node.map(|x| x.id()),
            Vertex::IfStatementAST(data) => data.ast_node.map(|x| x.id()),
            Vertex::JSXOpeningElement(data) => data.ast_node.map(|x| x.id()),
            Vertex::NumberLiteral(data) => data.ast_node.map(|x| x.id()),
            Vertex::Name(data) => data.ast_node.map(|x| x.id()),
            Vertex::SpreadIntoObject(data) => data.ast_node.map(|x| x.id()),
            Vertex::ObjectEntry(data) => data.ast_node.map(|x| x.id()),
            Vertex::DotProperty(data) => data.ast_node.map(|x| x.id()),
            Vertex::Reassignment(data) => data.ast_node.map(|x| x.id()),
            Vertex::FnCall(data) => data.ast_node.map(|x| x.id()),
            Vertex::FnDeclaration(data) => data.ast_node.map(|x| x.id()),
            Vertex::ArrowFunction(data) => data.ast_node.map(|x| x.id()),
            Vertex::FunctionBody(data) => data.ast_node.map(|x| x.id()),
            Vertex::Parameter(data) => data.ast_node.map(|x| x.id()),
            Vertex::Argument(data) => data.ast_node.map(|x| x.id()),
            Vertex::LogicalExpression(data) => data.ast_node.map(|x| x.id()),
            Vertex::UnaryExpression(data) => data.ast_node.map(|x| x.id()),
            Vertex::ExpressionStatement(data) => data.ast_node.map(|x| x.id()),
            Vertex::WhileStatement(data) => data.ast_node.map(|x| x.id()),
            Vertex::BlockStatement(data) => data.ast_node.map(|x| x.id()),
            Vertex::VarRef(data) => data.ast_node.map(|x| x.id()),
            Vertex::DoWhileStatement(data) => data.ast_node.map(|x| x.id()),
            Vertex::ForStatement(data) => data.ast_node.map(|x| x.id()),
            Vertex::TernaryExpression(data) => data.ast_node.map(|x| x.id()),
            Vertex::New(data) => data.ast_node.map(|x| x.id()),
            Vertex::Throw(data) => data.ast_node.map(|x| x.id()),
            Vertex::Array(data) => data.ast_node.map(|x| x.id()),
            Vertex::ArrayElement(data) => data.ast_node.map(|x| x.id()),
            Vertex::StringLiteral(data) => data.ast_node.map(|x| x.id()),
            Vertex::TemplateLiteral(data) => data.ast_node.map(|x| x.id()),
            Vertex::RegExpLiteral(data) => data.ast_node.map(|x| x.id()),
            Vertex::ParenthesizedExpression(data) => data.ast_node.map(|x| x.id()),
            Vertex::DefaultImport(_)
            | Vertex::Statement(_)
            | Vertex::AssignmentType(_)
            | Vertex::ClassMethod(_)
            | Vertex::Expression(_)
            | Vertex::File
            | Vertex::InterfaceExtend(_)
            | Vertex::JSXAttribute(_)
            | Vertex::JSXExpressionContainer(_)
            | Vertex::JSXFragment(_)
            | Vertex::JSXText(_)
            | Vertex::JSXSpreadChild(_)
            | Vertex::JSXSpreadAttribute(_)
            | Vertex::PathPart(_)
            | Vertex::Url(_)
            | Vertex::Type(_)
            | Vertex::SpecificImport(_)
            | Vertex::Span(_)
            | Vertex::SearchParameter(_)
            | Vertex::ClassProperty(_) => None,
        }
    }

    pub fn make_url(attr: &'a JSXAttribute<'a>) -> Option<Self> {
        jsx_attribute_to_constant_string(attr)
            .as_deref()
            .and_then(|v| Url::parse(v).ok())
            .map(Rc::new)
            .map(Vertex::Url)
    }

    pub fn as_constant_string(&self) -> Option<String> {
        match &self {
            Vertex::StringLiteral(slit) => Some(slit.string.value.to_string()),
            Vertex::TemplateLiteral(tlit) => {
                try_get_constant_string_field_value_from_template_lit(tlit.template)
            }
            _ => None,
        }
    }

    // todo: remove `Option` when the match covers all the cases
    pub fn try_from_member_expression(member_expression: &'a MemberExpression<'a>) -> Option<Self> {
        match &member_expression {
            MemberExpression::StaticMemberExpression(static_member_expr) => {
                Some(Vertex::DotProperty(
                    DotPropertyVertex { ast_node: None, static_member_expr }.into(),
                ))
            }
            _ => None,
        }
    }

    pub fn try_from_identifier_reference(identifier_reference: &'a IdentifierReference) -> Self {
        Vertex::VarRef(VarRefVertex { ast_node: None, identifier_reference }.into())
    }

    pub fn function_is_async(&self) -> bool {
        match &self {
            Vertex::ArrowFunction(data) => data.arrow_expression.r#async,
            Vertex::FnDeclaration(data) => data.function.r#async,
            _ => unreachable!(
                "'function_is_async' function should only ever be called with an ArrowFunction or FnDeclaration"
            ),
        }
    }

    pub fn function_is_generator(&self) -> bool {
        match &self {
            Vertex::ArrowFunction(data) => data.arrow_expression.generator,
            Vertex::FnDeclaration(data) => data.function.generator,
            _ => unreachable!(
                "'function_is_generator' function should only ever be called with an ArrowFunction or FnDeclaration"
            ),
        }
    }

    pub fn function_parameter(&self) -> VertexIterator<'a, Vertex<'a>> {
        let parameter = match &self {
            Vertex::ArrowFunction(data) => &data.arrow_expression.params.items,
            Vertex::FnDeclaration(data) => &data.function.params.items,
            _ => unreachable!(
                "'function_parameter' function should only ever be called with an ArrowFunction or FnDeclaration"
            ),
        };
        Box::new(parameter.iter().map(|parameter| {
            Vertex::Parameter(ParameterVertex { ast_node: None, parameter }.into())
        }))
    }

    pub fn is_expr(&self) -> bool {
        match &self {
            Vertex::ObjectLiteral(..)
            | Vertex::JSXElement(..)
            | Vertex::NumberLiteral(..)
            | Vertex::Expression(..)
            | Vertex::Reassignment(..)
            | Vertex::FnCall(..)
            | Vertex::FnDeclaration(..)
            | Vertex::ArrowFunction(..)
            | Vertex::LogicalExpression(..)
            | Vertex::UnaryExpression(..)
            | Vertex::VarRef(..)
            | Vertex::TernaryExpression(..)
            | Vertex::New(..)
            | Vertex::StringLiteral(..)
            | Vertex::TemplateLiteral(..)
            | Vertex::RegExpLiteral(..)
            | Vertex::ParenthesizedExpression(..)
            | Vertex::Array(..) => true,
            Vertex::ASTNode(..)
            | Vertex::AssignmentType(..)
            | Vertex::Class(..)
            | Vertex::ClassMethod(..)
            | Vertex::ClassProperty(..)
            | Vertex::DefaultImport(..)
            | Vertex::File
            | Vertex::Import(..)
            | Vertex::Interface(..)
            | Vertex::InterfaceExtend(..)
            | Vertex::JSXAttribute(..)
            | Vertex::JSXExpressionContainer(..)
            | Vertex::JSXFragment(..)
            | Vertex::JSXOpeningElement(..)
            | Vertex::JSXSpreadAttribute(..)
            | Vertex::JSXSpreadChild(..)
            | Vertex::JSXText(..)
            | Vertex::Name(..)
            | Vertex::PathPart(..)
            | Vertex::SearchParameter(..)
            | Vertex::Span(..)
            | Vertex::SpecificImport(..)
            | Vertex::TypeAnnotation(..)
            | Vertex::Type(..)
            | Vertex::Url(..)
            | Vertex::VariableDeclaration(..)
            | Vertex::Return(..)
            | Vertex::IfStatementAST(..)
            | Vertex::SpreadIntoObject(..)
            | Vertex::ObjectEntry(..)
            | Vertex::DotProperty(..)
            | Vertex::Argument(..)
            | Vertex::FunctionBody(..)
            | Vertex::Statement(..)
            | Vertex::Parameter(..)
            | Vertex::ExpressionStatement(..)
            | Vertex::WhileStatement(..)
            | Vertex::BlockStatement(..)
            | Vertex::DoWhileStatement(..)
            | Vertex::ForStatement(..)
            | Vertex::Throw(..)
            | Vertex::ArrayElement(..) => false,
        }
    }
}

impl Typename for Vertex<'_> {
    fn typename(&self) -> &'static str {
        match self {
            Vertex::ASTNode(_) => "ASTNode",
            Vertex::AssignmentType(_) => "AssignmentType",
            Vertex::Class(data) => data.typename(),
            Vertex::ClassMethod(_) => "ClassMethod",
            Vertex::ClassProperty(_) => "ClassProperty",
            Vertex::DefaultImport(_) => "DefaultImport",
            Vertex::Expression(_) => "Expression",
            Vertex::File => "File",
            Vertex::Import(data) => data.typename(),
            Vertex::Interface(data) => data.typename(),
            Vertex::NumberLiteral(data) => data.typename(),
            Vertex::DotProperty(data) => data.typename(),
            Vertex::InterfaceExtend(data) => match **data {
                InterfaceExtendVertex::Identifier(_) => "SimpleExtend",
                InterfaceExtendVertex::MemberExpression(_) => "MemberExtend",
            },
            Vertex::JSXAttribute(_) => "JSXAttribute",
            Vertex::JSXElement(data) => data.typename(),
            Vertex::JSXExpressionContainer(_) => "JSXExpressionContainer",
            Vertex::JSXFragment(_) => "JSXFragment",
            Vertex::JSXOpeningElement(data) => data.typename(),
            Vertex::JSXSpreadAttribute(_) => "JSXSpreadAttribute",
            Vertex::JSXSpreadChild(_) => "JSXSpreadChild",
            Vertex::JSXText(_) => "JSXText",
            Vertex::ObjectLiteral(data) => data.typename(),
            Vertex::PathPart(_) => "PathPart",
            Vertex::SearchParameter(_) => "SearchParameter",
            Vertex::Span(_) => "Span",
            Vertex::SpecificImport(_) => "SpecificImport",
            Vertex::TypeAnnotation(data) => data.typename(),
            Vertex::Type(_) => "Type",
            Vertex::Url(_) => "URL",
            Vertex::VariableDeclaration(datat) => datat.typename(),
            Vertex::Name(data) => data.typename(),
            Vertex::Return(data) => data.typename(),
            Vertex::IfStatementAST(_) => "IfStatementAST",
            Vertex::SpreadIntoObject(data) => data.typename(),
            Vertex::ObjectEntry(data) => data.typename(),
            Vertex::FnCall(data) => data.typename(),
            Vertex::Reassignment(data) => data.typename(),
            Vertex::Argument(data) => data.typename(),
            Vertex::FnDeclaration(data) => data.typename(),
            Vertex::ArrowFunction(data) => data.typename(),
            Vertex::FunctionBody(data) => data.typename(),
            Vertex::Parameter(data) => data.typename(),
            Vertex::LogicalExpression(data) => data.typename(),
            Vertex::UnaryExpression(datat) => datat.typename(),
            Vertex::Statement(_) => "Statement",
            Vertex::ExpressionStatement(data) => data.typename(),
            Vertex::WhileStatement(data) => data.typename(),
            Vertex::DoWhileStatement(data) => data.typename(),
            Vertex::BlockStatement(data) => data.typename(),
            Vertex::VarRef(data) => data.typename(),
            Vertex::ForStatement(data) => data.typename(),
            Vertex::TernaryExpression(data) => data.typename(),
            Vertex::New(data) => data.typename(),
            Vertex::Throw(data) => data.typename(),
            Vertex::Array(data) => data.typename(),
            Vertex::ArrayElement(data) => data.typename(),
            Vertex::StringLiteral(data) => data.typename(),
            Vertex::TemplateLiteral(data) => data.typename(),
            Vertex::RegExpLiteral(data) => data.typename(),
            Vertex::ParenthesizedExpression(data) => data.typename(),
        }
    }
}

#[allow(clippy::too_many_lines)]
impl<'a> From<AstNode<'a>> for Vertex<'a> {
    fn from(ast_node: AstNode<'a>) -> Self {
        match ast_node.kind() {
            AstKind::ReturnStatement(return_statement) => {
                Self::Return(ReturnVertex { ast_node: Some(ast_node), return_statement }.into())
            }
            AstKind::IfStatement(if_statement) => Self::IfStatementAST(
                IfStatementVertex { ast_node: Some(ast_node), return_statement: if_statement }
                    .into(),
            ),
            AstKind::JSXElement(element) => {
                Self::JSXElement(JSXElementVertex { ast_node: Some(ast_node), element }.into())
            }
            AstKind::TSInterfaceDeclaration(interface) => {
                Self::Interface(InterfaceVertex { ast_node: Some(ast_node), interface }.into())
            }
            AstKind::TSTypeAnnotation(type_annotation) => Self::TypeAnnotation(
                TypeAnnotationVertex { ast_node: Some(ast_node), type_annotation }.into(),
            ),
            AstKind::VariableDeclarator(variable_declaration) => Self::VariableDeclaration(
                VariableDeclarationVertex { ast_node: Some(ast_node), variable_declaration }.into(),
            ),
            AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(import)) => {
                Self::Import(ImportVertex { ast_node: Some(ast_node), import }.into())
            }
            AstKind::Class(class) => {
                Self::Class(ClassVertex { ast_node: Some(ast_node), class }.into())
            }
            AstKind::ObjectExpression(objexpr) => Self::ObjectLiteral(
                ObjectLiteralVertex { ast_node: Some(ast_node), object_expression: objexpr }.into(),
            ),
            AstKind::JSXOpeningElement(opening_element) => Self::JSXOpeningElement(
                JSXOpeningElementVertex { ast_node: Some(ast_node), opening_element }.into(),
            ),
            AstKind::NumberLiteral(number_literal) => Self::NumberLiteral(
                NumberLiteralVertex { ast_node: Some(ast_node), number_literal }.into(),
            ),
            AstKind::IdentifierName(identifier_name) => {
                Self::Name(NameVertex { ast_node: Some(ast_node), name: identifier_name }.into())
            }
            AstKind::ObjectProperty(property) => {
                Self::ObjectEntry(ObjectEntryVertex { ast_node: Some(ast_node), property }.into())
            }
            AstKind::SpreadElement(property) => Self::SpreadIntoObject(
                SpreadIntoObjectVertex { ast_node: Some(ast_node), property }.into(),
            ),
            AstKind::MemberExpression(member_expr)
                if matches!(member_expr, MemberExpression::StaticMemberExpression(_)) =>
            {
                match member_expr {
                    MemberExpression::StaticMemberExpression(member_expr) => Self::DotProperty(
                        DotPropertyVertex {
                            ast_node: Some(ast_node),
                            static_member_expr: member_expr,
                        }
                        .into(),
                    ),
                    _ => unreachable!("we should only ever have StaticMemberExpression"),
                }
            }
            AstKind::AssignmentExpression(assignment_expression) => Vertex::Reassignment(
                ReassignmentVertex { ast_node: Some(ast_node), assignment_expression }.into(),
            ),
            AstKind::CallExpression(call_expression) => {
                Vertex::FnCall(FnCallVertex { ast_node: Some(ast_node), call_expression }.into())
            }
            AstKind::Function(function) => Vertex::FnDeclaration(
                FnDeclarationVertex { ast_node: Some(ast_node), function }.into(),
            ),
            AstKind::ArrowExpression(arrow_expression) => Vertex::ArrowFunction(
                ArrowFunctionVertex { ast_node: Some(ast_node), arrow_expression }.into(),
            ),
            AstKind::FunctionBody(function_body) => Vertex::FunctionBody(
                FunctionBodyVertex { ast_node: Some(ast_node), function_body }.into(),
            ),
            AstKind::FormalParameter(parameter) => {
                Vertex::Parameter(ParameterVertex { ast_node: Some(ast_node), parameter }.into())
            }
            AstKind::Argument(argument) => Vertex::Argument(
                ArgumentVertex { ast_node: Some(ast_node), argument, index: None }.into(),
            ),
            AstKind::LogicalExpression(logical_expression) => Vertex::LogicalExpression(
                LogicalExpressionVertex { ast_node: Some(ast_node), logical_expression }.into(),
            ),
            AstKind::UnaryExpression(unary_expression) => Vertex::UnaryExpression(
                UnaryExpressionVertex { ast_node: Some(ast_node), unary_expression }.into(),
            ),
            AstKind::ExpressionStatement(expression_statement) => Vertex::ExpressionStatement(
                ExpressionStatementVertex { ast_node: Some(ast_node), expression_statement }.into(),
            ),
            AstKind::WhileStatement(expression_statement) => Vertex::WhileStatement(
                WhileStatementVertex {
                    ast_node: Some(ast_node),
                    while_statement: expression_statement,
                }
                .into(),
            ),
            AstKind::BlockStatement(block_statement) => Vertex::BlockStatement(
                BlockStatementVertex { ast_node: Some(ast_node), block_statement }.into(),
            ),
            AstKind::IdentifierReference(identifier_reference) => Vertex::VarRef(
                VarRefVertex { ast_node: Some(ast_node), identifier_reference }.into(),
            ),
            AstKind::DoWhileStatement(do_while_statement) => Vertex::DoWhileStatement(
                DoWhileStatementVertex { ast_node: Some(ast_node), do_while_statement }.into(),
            ),
            AstKind::ForStatement(for_statement) => Vertex::ForStatement(
                ForStatementVertex { ast_node: Some(ast_node), for_statement }.into(),
            ),
            AstKind::ConditionalExpression(conditional_expression) => Vertex::TernaryExpression(
                TernaryExpressionVertex { ast_node: Some(ast_node), conditional_expression }.into(),
            ),
            AstKind::NewExpression(new_expression) => {
                Vertex::New(NewVertex { ast_node: Some(ast_node), new_expression }.into())
            }
            AstKind::ThrowStatement(throw_statement) => {
                Vertex::Throw(ThrowVertex { ast_node: Some(ast_node), throw_statement }.into())
            }
            AstKind::ArrayExpressionElement(array_expression_element) => Vertex::ArrayElement(
                ArrayElementVertex { ast_node: Some(ast_node), array_expression_element }.into(),
            ),
            AstKind::StringLiteral(string_literal) => Vertex::StringLiteral(
                StringLiteralVertex { ast_node: Some(ast_node), string: string_literal }.into(),
            ),
            AstKind::TemplateLiteral(template_literal) => Vertex::TemplateLiteral(
                TemplateLiteralVertex { ast_node: Some(ast_node), template: template_literal }
                    .into(),
            ),
            AstKind::RegExpLiteral(regexp_literal) => Vertex::RegExpLiteral(
                RegExpLiteralVertex { ast_node: Some(ast_node), regexp: regexp_literal }.into(),
            ),
            AstKind::ArrayExpression(array_expression) => {
                Vertex::Array(ArrayVertex { ast_node: Some(ast_node), array_expression }.into())
            }
            AstKind::ParenthesizedExpression(parenthesized_expression) => {
                Vertex::ParenthesizedExpression(
                    ParenthesizedExpressionVertex {
                        ast_node: Some(ast_node),
                        parenthesized_expression,
                    }
                    .into(),
                )
            }
            _ => Vertex::ASTNode(ast_node),
        }
    }
}

impl<'a> From<&'a Statement<'a>> for Vertex<'a> {
    fn from(stmt: &'a Statement<'a>) -> Self {
        match &stmt {
            Statement::ReturnStatement(return_statement) => {
                Vertex::Return(ReturnVertex { ast_node: None, return_statement }.into())
            }
            Statement::ExpressionStatement(expression_statement) => Vertex::ExpressionStatement(
                ExpressionStatementVertex { ast_node: None, expression_statement }.into(),
            ),
            Statement::WhileStatement(while_statement) => Vertex::WhileStatement(
                WhileStatementVertex { ast_node: None, while_statement }.into(),
            ),
            Statement::DoWhileStatement(do_while_statement) => Vertex::DoWhileStatement(
                DoWhileStatementVertex { ast_node: None, do_while_statement }.into(),
            ),
            Statement::BlockStatement(block_statement) => Vertex::BlockStatement(
                BlockStatementVertex { ast_node: None, block_statement }.into(),
            ),
            Statement::ForStatement(for_statement) => {
                Vertex::ForStatement(ForStatementVertex { ast_node: None, for_statement }.into())
            }
            Statement::ThrowStatement(throw_statement) => {
                Vertex::Throw(ThrowVertex { ast_node: None, throw_statement }.into())
            }
            _ => Vertex::Statement(stmt),
        }
    }
}

impl<'a> From<&'a Expression<'a>> for Vertex<'a> {
    fn from(expr: &'a Expression<'a>) -> Self {
        match &expr {
            Expression::ObjectExpression(object_expression) => Vertex::ObjectLiteral(
                ObjectLiteralVertex { ast_node: None, object_expression }.into(),
            ),
            Expression::JSXElement(element) => {
                Vertex::JSXElement(JSXElementVertex { ast_node: None, element }.into())
            }
            Expression::NumberLiteral(number_literal) => {
                Vertex::NumberLiteral(NumberLiteralVertex { ast_node: None, number_literal }.into())
            }
            Expression::MemberExpression(me) => {
                let vertex = Vertex::try_from_member_expression(me);

                vertex.unwrap_or(Vertex::Expression(expr))
            }
            Expression::AssignmentExpression(assignment_expression) => Vertex::Reassignment(
                ReassignmentVertex { ast_node: None, assignment_expression }.into(),
            ),
            Expression::CallExpression(call_expression) => {
                Vertex::FnCall(FnCallVertex { ast_node: None, call_expression }.into())
            }
            Expression::FunctionExpression(fn_expression) => Vertex::FnDeclaration(
                FnDeclarationVertex { ast_node: None, function: fn_expression }.into(),
            ),
            Expression::ArrowExpression(arrow_expression) => Vertex::ArrowFunction(
                ArrowFunctionVertex { ast_node: None, arrow_expression }.into(),
            ),
            Expression::LogicalExpression(logical_expression) => Vertex::LogicalExpression(
                LogicalExpressionVertex { ast_node: None, logical_expression }.into(),
            ),
            Expression::UnaryExpression(unary_expression) => Vertex::UnaryExpression(
                UnaryExpressionVertex { ast_node: None, unary_expression }.into(),
            ),
            Expression::Identifier(identifier_reference) => {
                Vertex::VarRef(VarRefVertex { ast_node: None, identifier_reference }.into())
            }
            Expression::ConditionalExpression(conditional_expression) => Vertex::TernaryExpression(
                TernaryExpressionVertex { ast_node: None, conditional_expression }.into(),
            ),
            Expression::NewExpression(new_expression) => {
                Vertex::New(NewVertex { ast_node: None, new_expression }.into())
            }
            Expression::ArrayExpression(array_expression) => {
                Vertex::Array(ArrayVertex { ast_node: None, array_expression }.into())
            }
            Expression::StringLiteral(string_literal) => Vertex::StringLiteral(
                StringLiteralVertex { ast_node: None, string: string_literal }.into(),
            ),
            Expression::TemplateLiteral(template_literal) => Vertex::TemplateLiteral(
                TemplateLiteralVertex { ast_node: None, template: template_literal }.into(),
            ),
            Expression::RegExpLiteral(regexp_literal) => Vertex::RegExpLiteral(
                RegExpLiteralVertex { ast_node: None, regexp: regexp_literal }.into(),
            ),
            Expression::ParenthesizedExpression(parenthesized_expression) => {
                Vertex::ParenthesizedExpression(
                    ParenthesizedExpressionVertex { ast_node: None, parenthesized_expression }
                        .into(),
                )
            }
            _ => Vertex::Expression(expr),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub class: &'a Class<'a>,
}

impl<'a> Typename for ClassVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ClassAST"
        } else {
            "Class"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassMethodVertex<'a> {
    pub method: &'a MethodDefinition<'a>,
    pub is_abstract: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassPropertyVertex<'a> {
    pub property: &'a PropertyDefinition<'a>,
    pub is_abstract: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ImportVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub import: &'a ImportDeclaration<'a>,
}

impl<'a> Typename for ImportVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ImportAST"
        } else {
            "Import"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct InterfaceVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub interface: &'a TSInterfaceDeclaration<'a>,
}

impl<'a> Typename for InterfaceVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "InterfaceAST"
        } else {
            "Interface"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum InterfaceExtendVertex<'a> {
    Identifier(&'a IdentifierReference),
    MemberExpression(&'a MemberExpression<'a>),
}

impl<'a> From<&'a Expression<'a>> for InterfaceExtendVertex<'a> {
    fn from(expr: &'a Expression<'a>) -> Self {
        match &expr {
            Expression::Identifier(ident) => InterfaceExtendVertex::Identifier(ident),
            Expression::MemberExpression(membexpr) => {
                InterfaceExtendVertex::MemberExpression(membexpr)
            }
            _ => unreachable!(
                "Only ever possible to have an interface extend an identifier or memberexpr. see TS:2499"
            ),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct JSXElementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub element: &'a JSXElement<'a>,
}

impl<'a> Typename for JSXElementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "JSXElementAST"
        } else {
            "JSXElement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct IfStatementVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub return_statement: &'a IfStatement<'a>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ReturnVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub return_statement: &'a ReturnStatement<'a>,
}

impl<'a> Typename for ReturnVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ReturnAST"
        } else {
            "ReturnStatement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TypeAnnotationVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub type_annotation: &'a TSTypeAnnotation<'a>,
}

impl<'a> Typename for TypeAnnotationVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "TypeAnnotationAST"
        } else {
            "TypeAnnotation"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SearchParameterVertex {
    pub key: String,
    pub value: String,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct VariableDeclarationVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub variable_declaration: &'a VariableDeclarator<'a>,
}

impl<'a> Typename for VariableDeclarationVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "VariableDeclarationAST"
        } else {
            "VariableDeclaration"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ObjectLiteralVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub object_expression: &'a ObjectExpression<'a>,
}

impl<'a> Typename for ObjectLiteralVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ObjectLiteralAST"
        } else {
            "ObjectLiteral"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct JSXOpeningElementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub opening_element: &'a JSXOpeningElement<'a>,
}

impl<'a> Typename for JSXOpeningElementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "JSXOpeningElementAST"
        } else {
            "JSXOpeningElement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct NumberLiteralVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub number_literal: &'a NumberLiteral<'a>,
}

impl<'a> Typename for NumberLiteralVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "NumberLiteralAST"
        } else {
            "NumberLiteral"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct NameVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub name: &'a IdentifierName,
}

impl<'a> Typename for NameVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "NameAST"
        } else {
            "Name"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ObjectEntryVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub property: &'a ObjectProperty<'a>,
}

impl<'a> Typename for ObjectEntryVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ObjectEntryAST"
        } else {
            "ObjectEntry"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SpreadIntoObjectVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub property: &'a SpreadElement<'a>,
}

impl<'a> Typename for SpreadIntoObjectVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "SpreadIntoObjectAST"
        } else {
            "SpreadIntoObject"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct DotPropertyVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub static_member_expr: &'a StaticMemberExpression<'a>,
}

impl<'a> Typename for DotPropertyVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "DotPropertyAST"
        } else {
            "DotProperty"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ReassignmentVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub assignment_expression: &'a AssignmentExpression<'a>,
}

impl<'a> Typename for ReassignmentVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ReassignmentAST"
        } else {
            "Reassignment"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FnCallVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub call_expression: &'a CallExpression<'a>,
}

impl<'a> Typename for FnCallVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "FnCallAST"
        } else {
            "FnCall"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FnDeclarationVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub function: &'a Function<'a>,
}

impl<'a> Typename for FnDeclarationVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "FnDeclarationAST"
        } else {
            "FnDeclaration"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ArrowFunctionVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub arrow_expression: &'a ArrowExpression<'a>,
}

impl<'a> Typename for ArrowFunctionVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ArrowFunctionAST"
        } else {
            "ArrowFunction"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FunctionBodyVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub function_body: &'a FunctionBody<'a>,
}

impl<'a> Typename for FunctionBodyVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "FunctionBodyAST"
        } else {
            "FunctionBody"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ParameterVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub parameter: &'a FormalParameter<'a>,
}

impl<'a> Typename for ParameterVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ParameterAST"
        } else {
            "Parameter"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ArgumentVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub argument: &'a Argument<'a>,
    pub index: Option<usize>,
}

impl<'a> ArgumentVertex<'a> {
    pub fn index<'c, 'b: 'c>(&self, adapter: &'c Adapter<'b>) -> usize {
        self.index.unwrap_or_else(|| self.ast_node.map_or_else(|| unreachable!("Should always either have an index or an ast_node, any other situation is an invariant"), |ast_node| {
            let AstKind::CallExpression(call_expr) = adapter
                .semantic
                .nodes()
                .parent_node(ast_node.id())
                .expect("argument should have parent")
                .kind() else {unreachable!("Argument's parent should always be a CallExpression")};
            call_expr
                .arguments
                .iter()
                .position(|x| calculate_hash(x) == calculate_hash(self.argument))
                .unwrap()
        }))
    }
}

impl<'a> Typename for ArgumentVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ArgumentAST"
        } else {
            "Argument"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct LogicalExpressionVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub logical_expression: &'a LogicalExpression<'a>,
}

impl<'a> Typename for LogicalExpressionVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "LogicalExpressionAST"
        } else {
            "LogicalExpression"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct UnaryExpressionVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub unary_expression: &'a UnaryExpression<'a>,
}

impl<'a> Typename for UnaryExpressionVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "UnaryExpressionAST"
        } else {
            "UnaryExpression"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ExpressionStatementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub expression_statement: &'a ExpressionStatement<'a>,
}

impl<'a> Typename for ExpressionStatementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ExpressionStatementAST"
        } else {
            "ExpressionStatement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct WhileStatementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub while_statement: &'a WhileStatement<'a>,
}

impl<'a> Typename for WhileStatementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "WhileStatementAST"
        } else {
            "WhileStatement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct BlockStatementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub block_statement: &'a BlockStatement<'a>,
}

impl<'a> Typename for BlockStatementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "BlockStatementAST"
        } else {
            "BlockStatement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct VarRefVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub identifier_reference: &'a IdentifierReference,
}

impl<'a> Typename for VarRefVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "VarRefAST"
        } else {
            "VarRef"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct DoWhileStatementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub do_while_statement: &'a DoWhileStatement<'a>,
}

impl<'a> Typename for DoWhileStatementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "DoWhileStatementAST"
        } else {
            "DoWhileStatement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ForStatementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub for_statement: &'a ForStatement<'a>,
}

impl<'a> Typename for ForStatementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ForStatementAST"
        } else {
            "ForStatement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TernaryExpressionVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub conditional_expression: &'a ConditionalExpression<'a>,
}

impl<'a> Typename for TernaryExpressionVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "TernaryExpressionAST"
        } else {
            "TernaryExpression"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct NewVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub new_expression: &'a NewExpression<'a>,
}

impl<'a> Typename for NewVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "NewAST"
        } else {
            "New"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ThrowVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub throw_statement: &'a ThrowStatement<'a>,
}

impl<'a> Typename for ThrowVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ThrowAST"
        } else {
            "Throw"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ArrayVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub array_expression: &'a ArrayExpression<'a>,
}

impl<'a> Typename for ArrayVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ArrayAST"
        } else {
            "Array"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ArrayElementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub array_expression_element: &'a ArrayExpressionElement<'a>,
}

impl<'a> Typename for ArrayElementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ArrayElementAST"
        } else {
            "ArrayElement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct StringLiteralVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub string: &'a StringLiteral,
}

impl<'a> Typename for StringLiteralVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "StringLiteralAST"
        } else {
            "StringLiteral"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TemplateLiteralVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub template: &'a TemplateLiteral<'a>,
}

impl<'a> Typename for TemplateLiteralVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "TemplateLiteralAST"
        } else {
            "TemplateLiteral"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct RegExpLiteralVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub regexp: &'a RegExpLiteral,
}

impl<'a> Typename for RegExpLiteralVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "RegExpLiteralAST"
        } else {
            "RegExpLiteral"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ParenthesizedExpressionVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub parenthesized_expression: &'a ParenthesizedExpression<'a>,
}

impl<'a> Typename for ParenthesizedExpressionVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ParenthesizedExpressionAST"
        } else {
            "ParenthesizedExpression"
        }
    }
}
