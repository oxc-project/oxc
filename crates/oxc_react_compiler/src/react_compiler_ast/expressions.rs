use serde::Serialize;

use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::common::RawNode;
use crate::react_compiler_ast::jsx::JSXElement;
use crate::react_compiler_ast::jsx::JSXFragment;
use crate::react_compiler_ast::literals::*;
use crate::react_compiler_ast::operators::*;
use crate::react_compiler_ast::patterns::AssignmentPattern;
use crate::react_compiler_ast::patterns::PatternLike;
use crate::react_compiler_ast::statements::BlockStatement;

#[derive(Debug, Clone, Serialize)]
pub struct Identifier {
    #[serde(flatten)]
    pub base: BaseNode,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeAnnotation")]
    pub type_annotation: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Expression {
    Identifier(Identifier),
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    BooleanLiteral(BooleanLiteral),
    NullLiteral(NullLiteral),
    BigIntLiteral(BigIntLiteral),
    RegExpLiteral(RegExpLiteral),
    CallExpression(CallExpression),
    MemberExpression(MemberExpression),
    OptionalCallExpression(OptionalCallExpression),
    OptionalMemberExpression(OptionalMemberExpression),
    BinaryExpression(BinaryExpression),
    LogicalExpression(LogicalExpression),
    UnaryExpression(UnaryExpression),
    UpdateExpression(UpdateExpression),
    ConditionalExpression(ConditionalExpression),
    AssignmentExpression(AssignmentExpression),
    SequenceExpression(SequenceExpression),
    ArrowFunctionExpression(ArrowFunctionExpression),
    FunctionExpression(FunctionExpression),
    ObjectExpression(ObjectExpression),
    ArrayExpression(ArrayExpression),
    NewExpression(NewExpression),
    TemplateLiteral(TemplateLiteral),
    TaggedTemplateExpression(TaggedTemplateExpression),
    AwaitExpression(AwaitExpression),
    YieldExpression(YieldExpression),
    SpreadElement(SpreadElement),
    MetaProperty(MetaProperty),
    ClassExpression(ClassExpression),
    PrivateName(PrivateName),
    Super(Super),
    Import(Import),
    ThisExpression(ThisExpression),
    ParenthesizedExpression(ParenthesizedExpression),
    // JSX expressions
    JSXElement(Box<JSXElement>),
    JSXFragment(JSXFragment),
    // Pattern (can appear in expression position in error recovery)
    AssignmentPattern(AssignmentPattern),
    // TypeScript expressions
    TSAsExpression(TSAsExpression),
    TSSatisfiesExpression(TSSatisfiesExpression),
    TSNonNullExpression(TSNonNullExpression),
    TSTypeAssertion(TSTypeAssertion),
    TSInstantiationExpression(TSInstantiationExpression),
    // Flow expressions
    TypeCastExpression(TypeCastExpression),
}

#[derive(Debug, Clone, Serialize)]
pub struct CallExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeArguments")]
    pub type_arguments: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub object: Box<Expression>,
    pub property: Box<Expression>,
    pub computed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptionalCallExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub optional: bool,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeArguments")]
    pub type_arguments: Option<RawNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptionalMemberExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub object: Box<Expression>,
    pub property: Box<Expression>,
    pub computed: bool,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BinaryExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub operator: BinaryOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogicalExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub operator: LogicalOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnaryExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub operator: UnaryOperator,
    pub prefix: bool,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub operator: UpdateOperator,
    pub argument: Box<Expression>,
    pub prefix: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConditionalExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub test: Box<Expression>,
    pub consequent: Box<Expression>,
    pub alternate: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub operator: AssignmentOperator,
    pub left: Box<PatternLike>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SequenceExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArrowFunctionExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub params: Vec<PatternLike>,
    pub body: Box<ArrowFunctionBody>,
    #[serde(default)]
    pub id: Option<Identifier>,
    #[serde(default)]
    pub generator: bool,
    #[serde(default, rename = "async")]
    pub is_async: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expression: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "returnType")]
    pub return_type: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "predicate")]
    pub predicate: Option<RawNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ArrowFunctionBody {
    BlockStatement(BlockStatement),
    #[serde(untagged)]
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub params: Vec<PatternLike>,
    pub body: BlockStatement,
    #[serde(default)]
    pub id: Option<Identifier>,
    #[serde(default)]
    pub generator: bool,
    #[serde(default, rename = "async")]
    pub is_async: bool,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "returnType")]
    pub return_type: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "predicate")]
    pub predicate: Option<RawNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub properties: Vec<ObjectExpressionProperty>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ObjectExpressionProperty {
    ObjectProperty(ObjectProperty),
    ObjectMethod(ObjectMethod),
    SpreadElement(SpreadElement),
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectProperty {
    #[serde(flatten)]
    pub base: BaseNode,
    pub key: Box<Expression>,
    pub value: Box<Expression>,
    pub computed: bool,
    pub shorthand: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectMethod {
    #[serde(flatten)]
    pub base: BaseNode,
    pub method: bool,
    pub kind: ObjectMethodKind,
    pub key: Box<Expression>,
    pub params: Vec<PatternLike>,
    pub body: BlockStatement,
    pub computed: bool,
    #[serde(default)]
    pub id: Option<Identifier>,
    #[serde(default)]
    pub generator: bool,
    #[serde(default, rename = "async")]
    pub is_async: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "returnType")]
    pub return_type: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "predicate")]
    pub predicate: Option<RawNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectMethodKind {
    Method,
    Get,
    Set,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArrayExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub elements: Vec<Option<Expression>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeArguments")]
    pub type_arguments: Option<RawNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateLiteral {
    #[serde(flatten)]
    pub base: BaseNode,
    pub quasis: Vec<TemplateElement>,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaggedTemplateExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub tag: Box<Expression>,
    pub quasi: TemplateLiteral,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AwaitExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YieldExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub argument: Option<Box<Expression>>,
    pub delegate: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpreadElement {
    #[serde(flatten)]
    pub base: BaseNode,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetaProperty {
    #[serde(flatten)]
    pub base: BaseNode,
    pub meta: Identifier,
    pub property: Identifier,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    #[serde(default)]
    pub id: Option<Identifier>,
    #[serde(rename = "superClass")]
    pub super_class: Option<Box<Expression>>,
    pub body: ClassBody,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decorators: Option<Vec<RawNode>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "implements")]
    pub implements: Option<Vec<RawNode>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "superTypeParameters")]
    pub super_type_parameters: Option<RawNode>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "typeParameters")]
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassBody {
    #[serde(flatten)]
    pub base: BaseNode,
    pub body: Vec<RawNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrivateName {
    #[serde(flatten)]
    pub base: BaseNode,
    pub id: Identifier,
}

#[derive(Debug, Clone, Serialize)]
pub struct Super {
    #[serde(flatten)]
    pub base: BaseNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct Import {
    #[serde(flatten)]
    pub base: BaseNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThisExpression {
    #[serde(flatten)]
    pub base: BaseNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParenthesizedExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expression: Box<Expression>,
}

// TypeScript expression nodes (pass-through with RawNode for type args)
#[derive(Debug, Clone, Serialize)]
pub struct TSAsExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expression: Box<Expression>,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: RawNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct TSSatisfiesExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expression: Box<Expression>,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: RawNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct TSNonNullExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TSTypeAssertion {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expression: Box<Expression>,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: RawNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct TSInstantiationExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expression: Box<Expression>,
    #[serde(rename = "typeParameters")]
    pub type_parameters: RawNode,
}

// Flow expression nodes
#[derive(Debug, Clone, Serialize)]
pub struct TypeCastExpression {
    #[serde(flatten)]
    pub base: BaseNode,
    pub expression: Box<Expression>,
    #[serde(rename = "typeAnnotation")]
    pub type_annotation: RawNode,
}
