//! ArkUI AST node definitions
//!
//! This module contains AST node definitions for HarmonyOS ArkUI syntax including:
//! - Struct declarations (`struct ComponentName { ... }`)
//! - Annotation declarations (`annotation MyAnnotation { ... }`)
//! - ArkUI component expressions (`Column() { ... }`)
//!
//! ArkUI is a declarative UI framework for HarmonyOS applications.

use oxc_allocator::{Box, CloneIn, Dummy, GetAddress, TakeIn, UnstableAddress, Vec};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{ContentEq, GetSpan, GetSpanMut, Span};
use oxc_syntax::{node::NodeId, scope::ScopeId};
use std::cell::Cell;

use super::{ets::*, js::*, ts::*};

/// Struct Declaration Statement
///
/// Represents an ArkUI struct declaration, which is similar to a class but uses the `struct` keyword.
///
/// ## Example
/// ```arkui
/// @ComponentV2
/// struct MyComponent {
///   @Local message: string = 'Hello World';
///   build() {
///     Column() {
///       Text(`Parent message: ${this.message}`)
///     }
///   }
/// }
/// ```
#[ast(visit)]
#[scope(flags = ScopeFlags::StrictMode)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct StructStatement<'a> {
    pub node_id: Cell<NodeId>,
    /// Span
    pub span: Span,
    /// Decorators applied to the struct.
    ///
    /// Common decorators include `@ComponentV2`, `@Entry`, etc.
    ///
    /// ## Example
    /// ```arkui
    /// @ComponentV2  // <- decorator
    /// @Entry        // <- decorator
    /// struct MyComponent {}
    /// ```
    pub decorators: Vec<'a, Decorator<'a>>,
    /// Struct identifier, AKA the name
    pub id: BindingIdentifier<'a>,
    /// Type parameters (for generic structs, if supported)
    #[ts]
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// Super class declared by an `extends` clause.
    pub super_class: Option<Expression<'a>>,
    /// Type arguments passed to the super class.
    #[ts]
    pub super_type_arguments: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    /// Interfaces declared by an `implements` clause.
    #[ts]
    pub implements: Vec<'a, TSClassImplements<'a>>,
    /// Struct body containing properties and methods
    pub body: Box<'a, StructBody<'a>>,
    /// Whether this struct is marked with `abstract`.
    #[ts]
    pub r#abstract: bool,
    /// Whether this struct is marked with `declare`.
    #[ts]
    pub declare: bool,
    /// Static ETS `final` modifier.
    #[builder(default, skip)]
    #[estree(omit_if_default)]
    #[ts]
    pub r#final: bool,
    /// Static ETS `native` modifier. Preserved for diagnostics/round-tripping.
    #[builder(default, skip)]
    #[estree(omit_if_default)]
    #[ts]
    pub native: bool,
    /// Static ETS `static` modifier on nested structs.
    #[builder(default, skip)]
    #[estree(omit_if_default)]
    #[ts]
    pub r#static: bool,
    /// Id of the scope created by the [`StructStatement`], including type parameters and
    /// statements within the [`StructBody`].
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Struct Body
///
/// Contains the elements (properties and methods) within a struct declaration.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct StructBody<'a> {
    pub node_id: Cell<NodeId>,
    /// Span
    pub span: Span,
    /// Elements within the struct body
    pub body: Vec<'a, StructElement<'a>>,
}

/// Struct Body Element
///
/// Represents an element within a struct body, which can be:
/// - Property definitions (with decorators like `@Param`, `@Local`, `@Once`)
/// - Method definitions (like `build()`)
///
/// ## Example
/// ```arkui
/// struct MyComponent {
///   @Param @Once onceParam: string = '';  // StructElement::PropertyDefinition
///   @Local message: string = 'Hello';      // StructElement::PropertyDefinition
///   build() {                              // StructElement::MethodDefinition
///     Column() {}
///   }
/// }
/// ```
#[ast(visit)]
#[builder(skip)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, GetAddress, ContentEq, ESTree)]
pub enum StructElement<'a> {
    /// Property definitions with decorators
    ///
    /// Properties can have decorators like `@Param`, `@Local`, `@Once`, etc.
    PropertyDefinition(Box<'a, PropertyDefinition<'a>>) = 0,
    /// Method definitions (like `build()`)
    MethodDefinition(Box<'a, MethodDefinition<'a>>) = 1,
    /// Static initialization block.
    StaticBlock(Box<'a, StaticBlock<'a>>) = 2,
    /// TypeScript index signature.
    TSIndexSignature(Box<'a, TSIndexSignature<'a>>) = 3,
    /// Auto-accessor property.
    AccessorProperty(Box<'a, AccessorProperty<'a>>) = 4,
    /// Static ETS managed overload declaration.
    ETSOverloadDeclaration(Box<'a, ETSOverloadDeclaration<'a>>) = 5,
}

/// ArkUI Component Expression
///
/// Represents an ArkUI component call expression with children, similar to JSX but using
/// function call syntax.
///
/// ## Example
/// ```arkui
/// Column() {
///   Text(`onceParam: ${this.onceParam}`)
///   Button('change message')
///     .onClick(() => {
///       this.message = 'Hello Tomorrow';
///     })
/// }
/// ```
///
/// This is similar to JSX but uses function call syntax:
/// - JSX: `<Column><Text>Hello</Text></Column>`
/// - ArkUI: `Column() { Text('Hello') }`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct ArkUIComponentExpression<'a> {
    pub node_id: Cell<NodeId>,
    /// Span
    pub span: Span,
    /// The component name/callee (e.g., `Column`, `Text`, `Button`)
    pub callee: Expression<'a>,
    /// Type arguments for generic components (if supported)
    #[ts]
    pub type_arguments: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    /// Arguments passed to the component constructor
    ///
    /// ## Example
    /// ```arkui
    /// Button('change message')  // <- arguments contains StringLiteral('change message')
    /// ```
    pub arguments: Vec<'a, Argument<'a>>,
    /// Children of the component (the content inside `{ ... }`)
    ///
    /// Children can be:
    /// - Other ArkUI component expressions
    /// - Text expressions
    /// - Template literals
    /// - Regular expressions
    pub children: Vec<'a, ArkUIChild<'a>>,
    /// Whether the component call had an explicit child block. This distinguishes
    /// `Column()` from `Column() {}` when `children` is empty.
    #[estree(skip)]
    pub has_children: bool,
    /// Chain expressions (like `.onClick(...)`)
    ///
    /// ArkUI supports method chaining on components:
    /// ```arkui
    /// Button('click me')
    ///   .onClick(() => { ... })  // <- chain_expression (CallExpression with MemberExpression callee)
    /// ```
    ///
    /// Each chain expression is a `CallExpression` where the callee is a `MemberExpression`.
    /// The chain is built by nesting these expressions.
    pub chain_expressions: Vec<'a, CallExpression<'a>>,
}

/// ArkUI Child
///
/// Represents a child element within an ArkUI component expression.
///
/// ## Example
/// ```arkui
/// Column() {
///   Text('Hello')                    // ArkUIChild::Component
///   Button('Click')                  // ArkUIChild::Component
///   `Template: ${value}`             // ArkUIChild::Expression
///   if (condition) {                  // ArkUIChild::Statement
///     Text('Conditional')
///   }
/// }
/// ```
#[ast(visit)]
#[builder(skip)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, GetAddress, ContentEq, ESTree)]
pub enum ArkUIChild<'a> {
    /// Another ArkUI component expression (nested component)
    Component(Box<'a, ArkUIComponentExpression<'a>>) = 0,
    /// A regular expression (for text, template literals, etc.)
    Expression(Box<'a, Expression<'a>>) = 1,
    /// A statement (for control flow like if, for, etc.)
    Statement(Box<'a, Statement<'a>>) = 2,
}

/// Annotation Declaration Statement
///
/// Represents an ArkTS annotation declaration, which is used to define custom annotations.
///
/// ## Example
/// ```arkts
/// @interface MyAnnotation {
///   value: string;
///   count: number = 10;
/// }
/// ```
#[ast(visit)]
#[scope(flags = ScopeFlags::StrictMode)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct AnnotationDeclaration<'a> {
    pub node_id: Cell<NodeId>,
    /// Span
    pub span: Span,
    /// Decorators applied to the annotation (not used for @interface syntax).
    pub decorators: Vec<'a, Decorator<'a>>,
    /// Annotation identifier, AKA the name
    pub id: BindingIdentifier<'a>,
    /// Annotation body containing properties
    pub body: Box<'a, AnnotationBody<'a>>,
    /// Whether this annotation is marked with `declare`.
    #[ts]
    pub declare: bool,
    /// Id of the scope created by the [`AnnotationDeclaration`], including
    /// statements within the [`AnnotationBody`].
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Annotation Body
///
/// Contains the elements (properties) within an annotation declaration.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
pub struct AnnotationBody<'a> {
    pub node_id: Cell<NodeId>,
    /// Span
    pub span: Span,
    /// Elements within the annotation body
    pub body: Vec<'a, AnnotationElement<'a>>,
}

/// Annotation Body Element
///
/// Represents an element within an annotation body, which can be:
/// - Property definitions
///
/// ## Example
/// ```arkts
/// annotation MyAnnotation {
///   value: string;      // AnnotationElement::PropertyDefinition
///   count?: number;     // AnnotationElement::PropertyDefinition
/// }
/// ```
#[ast(visit)]
#[builder(skip)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, GetAddress, ContentEq, ESTree)]
pub enum AnnotationElement<'a> {
    /// Property definitions
    PropertyDefinition(Box<'a, PropertyDefinition<'a>>) = 0,
}
