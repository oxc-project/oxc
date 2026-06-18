use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::common::RawNode;
use crate::react_compiler_ast::expressions::Expression;
use crate::react_compiler_ast::literals::StringLiteral;

#[derive(Debug, Clone)]
pub struct JSXElement {
    pub base: BaseNode,
    pub opening_element: JSXOpeningElement,
    pub closing_element: Option<JSXClosingElement>,
    pub children: Vec<JSXChild>,
    pub self_closing: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct JSXFragment {
    pub base: BaseNode,
    pub opening_fragment: JSXOpeningFragment,
    pub closing_fragment: JSXClosingFragment,
    pub children: Vec<JSXChild>,
}

#[derive(Debug, Clone)]
pub struct JSXOpeningElement {
    pub base: BaseNode,
    pub name: JSXElementName,
    pub attributes: Vec<JSXAttributeItem>,
    pub self_closing: bool,
    pub type_parameters: Option<RawNode>,
}

#[derive(Debug, Clone)]
pub struct JSXClosingElement {
    pub base: BaseNode,
    pub name: JSXElementName,
}

#[derive(Debug, Clone)]
pub struct JSXOpeningFragment {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub struct JSXClosingFragment {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub enum JSXElementName {
    JSXIdentifier(JSXIdentifier),
    JSXMemberExpression(JSXMemberExpression),
    JSXNamespacedName(JSXNamespacedName),
}

#[derive(Debug, Clone)]
pub enum JSXChild {
    JSXElement(Box<JSXElement>),
    JSXFragment(JSXFragment),
    JSXExpressionContainer(JSXExpressionContainer),
    JSXSpreadChild(JSXSpreadChild),
    JSXText(JSXText),
}

#[derive(Debug, Clone)]
pub enum JSXAttributeItem {
    JSXAttribute(JSXAttribute),
    JSXSpreadAttribute(JSXSpreadAttribute),
}

#[derive(Debug, Clone)]
pub struct JSXAttribute {
    pub base: BaseNode,
    pub name: JSXAttributeName,
    pub value: Option<JSXAttributeValue>,
}

#[derive(Debug, Clone)]
pub enum JSXAttributeName {
    JSXIdentifier(JSXIdentifier),
    JSXNamespacedName(JSXNamespacedName),
}

#[derive(Debug, Clone)]
pub enum JSXAttributeValue {
    StringLiteral(StringLiteral),
    JSXExpressionContainer(JSXExpressionContainer),
    JSXElement(Box<JSXElement>),
    JSXFragment(JSXFragment),
}

#[derive(Debug, Clone)]
pub struct JSXSpreadAttribute {
    pub base: BaseNode,
    pub argument: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct JSXExpressionContainer {
    pub base: BaseNode,
    pub expression: JSXExpressionContainerExpr,
}

#[derive(Debug, Clone)]
pub enum JSXExpressionContainerExpr {
    JSXEmptyExpression(JSXEmptyExpression),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct JSXSpreadChild {
    pub base: BaseNode,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct JSXText {
    pub base: BaseNode,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct JSXEmptyExpression {
    pub base: BaseNode,
}

#[derive(Debug, Clone)]
pub struct JSXIdentifier {
    pub base: BaseNode,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct JSXMemberExpression {
    pub base: BaseNode,
    pub object: Box<JSXMemberExprObject>,
    pub property: JSXIdentifier,
}

#[derive(Debug, Clone)]
pub enum JSXMemberExprObject {
    JSXIdentifier(JSXIdentifier),
    JSXMemberExpression(Box<JSXMemberExpression>),
}

#[derive(Debug, Clone)]
pub struct JSXNamespacedName {
    pub base: BaseNode,
    pub namespace: JSXIdentifier,
    pub name: JSXIdentifier,
}
