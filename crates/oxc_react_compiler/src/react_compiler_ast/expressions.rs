use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::common::RawNode;
use crate::react_compiler_ast::jsx::JSXElement;
use crate::react_compiler_ast::jsx::JSXFragment;
use crate::react_compiler_ast::literals::*;
use crate::react_compiler_ast::operators::*;
use crate::react_compiler_ast::patterns::AssignmentPattern;
use crate::react_compiler_ast::patterns::PatternLike;
use crate::react_compiler_ast::statements::BlockStatement;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub base: BaseNode,
    pub name: String,
    pub type_annotation: Option<RawNode>,
    pub optional: Option<bool>,
    pub decorators: Option<Vec<RawNode>>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub base: BaseNode,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub type_parameters: Option<RawNode>,
    pub type_arguments: Option<RawNode>,
    pub optional: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct MemberExpression {
    pub base: BaseNode,
    pub object: Box<Expression>,
    pub property: Box<Expression>,
    pub computed: bool,
}

#[derive(Debug, Clone)]
pub struct OptionalCallExpression {
    pub base: BaseNode,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub optional: bool,
    pub type_parameters: Option<RawNode>,
    pub type_arguments: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct OptionalMemberExpression {
    pub base: BaseNode,
    pub object: Box<Expression>,
    pub property: Box<Expression>,
    pub computed: bool,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub base: BaseNode,
    pub operator: BinaryOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct LogicalExpression {
    pub base: BaseNode,
    pub operator: LogicalOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub base: BaseNode,
    pub operator: UnaryOperator,
    pub prefix: bool,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct UpdateExpression {
    pub base: BaseNode,
    pub operator: UpdateOperator,
    pub argument: Box<Expression>,
    pub prefix: bool,
}

#[derive(Debug, Clone)]
pub struct ConditionalExpression {
    pub base: BaseNode,
    pub test: Box<Expression>,
    pub consequent: Box<Expression>,
    pub alternate: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpression {
    pub base: BaseNode,
    pub operator: AssignmentOperator,
    pub left: Box<PatternLike>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct SequenceExpression {
    pub base: BaseNode,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct ArrowFunctionExpression {
    pub base: BaseNode,
    pub params: Vec<PatternLike>,
    pub body: Box<ArrowFunctionBody>,
    pub id: Option<Identifier>,
    pub generator: bool,
    pub is_async: bool,
    pub expression: Option<bool>,
    pub return_type: Option<RawNode>,
    pub type_parameters: Option<RawNode>,
    pub predicate: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub enum ArrowFunctionBody {
    BlockStatement(BlockStatement),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct FunctionExpression {
    pub base: BaseNode,
    pub params: Vec<PatternLike>,
    pub body: BlockStatement,
    pub id: Option<Identifier>,
    pub generator: bool,
    pub is_async: bool,
    pub return_type: Option<RawNode>,
    pub type_parameters: Option<RawNode>,
    pub predicate: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct ObjectExpression {
    pub base: BaseNode,
    pub properties: Vec<ObjectExpressionProperty>,
}

#[derive(Debug, Clone)]
pub enum ObjectExpressionProperty {
    ObjectProperty(ObjectProperty),
    ObjectMethod(ObjectMethod),
    SpreadElement(SpreadElement),
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub base: BaseNode,
    pub key: Box<Expression>,
    pub value: Box<Expression>,
    pub computed: bool,
    pub shorthand: bool,
    pub decorators: Option<Vec<RawNode>>,
    pub method: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ObjectMethod {
    pub base: BaseNode,
    pub method: bool,
    pub kind: ObjectMethodKind,
    pub key: Box<Expression>,
    pub params: Vec<PatternLike>,
    pub body: BlockStatement,
    pub computed: bool,
    pub id: Option<Identifier>,
    pub generator: bool,
    pub is_async: bool,
    pub decorators: Option<Vec<RawNode>>,
    pub return_type: Option<RawNode>,
    pub type_parameters: Option<RawNode>,
    pub predicate: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub enum ObjectMethodKind {
    Method,
    Get,
    Set,
}

#[derive(Debug, Clone)]
pub struct ArrayExpression {
    pub base: BaseNode,
    pub elements: Vec<Option<Expression>>,
}

#[derive(Debug, Clone)]
pub struct NewExpression {
    pub base: BaseNode,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub type_parameters: Option<RawNode>,
    pub type_arguments: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct TemplateLiteral {
    pub base: BaseNode,
    pub quasis: Vec<TemplateElement>,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct TaggedTemplateExpression {
    pub base: BaseNode,
    pub tag: Box<Expression>,
    pub quasi: TemplateLiteral,
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct AwaitExpression {
    pub base: BaseNode,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct YieldExpression {
    pub base: BaseNode,
    pub argument: Option<Box<Expression>>,
    pub delegate: bool,
}

#[derive(Debug, Clone)]
pub struct SpreadElement {
    pub base: BaseNode,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct MetaProperty {
    pub base: BaseNode,
    pub meta: Identifier,
    pub property: Identifier,
}

#[derive(Debug, Clone)]
pub struct ClassExpression {
    pub base: BaseNode,
    pub id: Option<Identifier>,
    pub super_class: Option<Box<Expression>>,
    pub body: ClassBody,
    pub decorators: Option<Vec<RawNode>>,
    pub implements: Option<Vec<RawNode>>,
    pub super_type_parameters: Option<RawNode>,
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct ClassBody {
    pub base: BaseNode,
    pub body: Vec<RawNode>,
}

#[derive(Debug, Clone)]
pub struct PrivateName {
    pub base: BaseNode,
    pub id: Identifier,
}

#[derive(Debug, Clone)]
pub struct Super {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub struct ThisExpression {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub struct ParenthesizedExpression {
    pub base: BaseNode,
    pub expression: Box<Expression>,
}

// TypeScript expression nodes (pass-through with RawNode for type args)
#[derive(Debug, Clone)]
pub struct TSAsExpression {
    pub base: BaseNode,
    pub expression: Box<Expression>,
    pub type_annotation: RawNode,
}

#[derive(Debug, Clone)]
pub struct TSSatisfiesExpression {
    pub base: BaseNode,
    pub expression: Box<Expression>,
    pub type_annotation: RawNode,
}

#[derive(Debug, Clone)]
pub struct TSNonNullExpression {
    pub base: BaseNode,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct TSTypeAssertion {
    pub base: BaseNode,
    pub expression: Box<Expression>,
    pub type_annotation: RawNode,
}

#[derive(Debug, Clone)]
pub struct TSInstantiationExpression {
    pub base: BaseNode,
    pub expression: Box<Expression>,
    pub type_parameters: RawNode,
}

// Flow expression nodes
#[derive(Debug, Clone)]
pub struct TypeCastExpression {
    pub base: BaseNode,
    pub expression: Box<Expression>,
    pub type_annotation: RawNode,
}
