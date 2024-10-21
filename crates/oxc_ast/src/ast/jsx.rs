//! [JSX](https://facebook.github.io/jsx)
#![warn(missing_docs)]

// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

use oxc_allocator::{Box, CloneIn, Vec};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{cmp::ContentEq, hash::ContentHash, Atom, GetSpan, GetSpanMut, Span};

use super::{inherit_variants, js::*, literal::*, ts::*};

// 1.2 JSX Elements

/// JSX Element
///
/// Note that fragments (`<></>`) are represented as [`JSXFragment`], unless they are written as
/// members of React (e.g. `<React.Fragment></React.Fragment>`).
/// ## Examples
///
/// ```tsx
/// <Foo>        // <- opening_element
///   some text  // <- children
/// </Foo>       // <- closing_element
/// ```
///
/// ```tsx
/// <Foo />     // <- opening_element, no closing_element
/// ```
///
/// See: [JSX Syntax](https://facebook.github.io/jsx/)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXElement<'a> {
    #[estree(flatten)]
    /// Node location in source code
    pub span: Span,
    /// Opening tag of the element.
    pub opening_element: Box<'a, JSXOpeningElement<'a>>,
    /// Closing tag of the element. Will be [`None`] for self-closing tags.
    pub closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
    /// Children of the element. This can be text, other elements, or expressions.
    pub children: Vec<'a, JSXChild<'a>>,
}

/// JSX Opening Element
///
/// Opening tag in a [`JSXElement`].
///
/// ## Examples
/// ```tsx
/// // element with opening and closing tags (self_closing = false)
/// //   ___ name
///     <Foo bar baz={4}>
/// //       ^^^^^^^^^^^ attributes
///
/// // element with self-closing tag (self_closing = true)
/// <Component<T> />
/// //         ^ type_parameters
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXOpeningElement<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// Is this tag self-closing?
    ///
    /// ## Examples
    /// ```tsx
    /// <Foo />  // <- self_closing = true
    /// <Foo>    // <- self_closing = false
    /// ```
    pub self_closing: bool,
    /// The possibly-namespaced tag name, e.g. `Foo` in `<Foo />`.
    pub name: JSXElementName<'a>,
    /// List of JSX attributes. In React-like applications, these become props.
    pub attributes: Vec<'a, JSXAttributeItem<'a>>,
    /// Type parameters for generic JSX elements.
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// JSX Closing Element
///
/// Closing tag in a [`JSXElement`]. Self-closing tags do not have closing elements.
///
/// ## Example
///
/// ```tsx
/// <Foo>Hello, World!</Foo>
/// //                  ^^^ name
/// <Bar /> // <- no closing element
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXClosingElement<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The tag name, e.g. `Foo` in `</Foo>`.
    pub name: JSXElementName<'a>,
}

/// JSX Fragment
///
/// A fragment written with the special `<></>` syntax. When written as a `<Fragment>` component,
/// fragments will be represented as [`JSXElement`]s.
///
/// Note that fragments cannot have attributes or type parameters.
///
/// See: [`React.Fragment`](https://react.dev/reference/react/Fragment)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXFragment<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// `<>`
    pub opening_fragment: JSXOpeningFragment,
    /// `</>`
    pub closing_fragment: JSXClosingFragment,
    /// Elements inside the fragment.
    pub children: Vec<'a, JSXChild<'a>>,
}

/// JSX Opening Fragment (`<>`)
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXOpeningFragment {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
}

/// JSX Closing Fragment (`</>`)
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXClosingFragment {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
}

/// JSX Element Name
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash)]
#[estree(untagged)]
pub enum JSXElementName<'a> {
    /// `<div />`
    Identifier(Box<'a, JSXIdentifier<'a>>) = 0,
    /// `<Apple />`
    IdentifierReference(Box<'a, IdentifierReference<'a>>) = 1,
    /// `<Apple:Orange />`
    NamespacedName(Box<'a, JSXNamespacedName<'a>>) = 2,
    /// `<Apple.Orange />`
    MemberExpression(Box<'a, JSXMemberExpression<'a>>) = 3,
    /// `<this />`
    ThisExpression(Box<'a, ThisExpression>) = 4,
}

/// JSX Namespaced Name
///
/// ## Example
///
/// ```tsx
/// <Apple:Orange />
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXNamespacedName<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    pub namespace: JSXIdentifier<'a>,
    /// Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    pub property: JSXIdentifier<'a>,
}

/// JSX Member Expression
///
/// Used in [`JSXElementName`]. Multiple member expressions may be chained together. In this case,
/// [`object`] will be a [`member expression`].
///
/// ## Example
///
/// ```tsx
/// // <object.property />
/// <Apple.Orange />
/// <Foo.Bar.Baz.Bang />
/// ```
///
/// [`object`]: JSXMemberExpression::object
/// [`member expression`]: JSXMemberExpressionObject::MemberExpression
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXMemberExpression<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The object being accessed. This is everything before the last `.`.
    pub object: JSXMemberExpressionObject<'a>,
    /// The property being accessed. This is everything after the last `.`.
    pub property: JSXIdentifier<'a>,
}

/// JSX Member Expression Object
///
/// Part of a [`JSXMemberExpression`]. This is the object being accessed in
/// namespace-like JSX tag names.
///
/// ## Example
/// ```tsx
/// const x = <Apple.Orange />
/// //         ^^^^^ IdentifierReference
///
/// const y = <Apple.Orange.Banana />
/// //         ^^^^^^^^^^^^ MemberExpression
///
/// const z = <this.Orange />
/// //         ^^^^ ThisExpression
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash)]
#[estree(untagged)]
pub enum JSXMemberExpressionObject<'a> {
    /// `<Apple.Orange />`
    IdentifierReference(Box<'a, IdentifierReference<'a>>) = 0,
    /// `<Apple.Orange.Banana />`
    MemberExpression(Box<'a, JSXMemberExpression<'a>>) = 1,
    /// `<this.Orange />`
    ThisExpression(Box<'a, ThisExpression>) = 2,
}

/// JSX Expression Container
///
/// Expression containers wrap [`JSXExpression`]s in JSX attributes and children using `{}`.
///
/// ## Example
///
/// ```tsx
/// // boolean-like and string-like expressions are not wrapped in containers.
/// // Here, only `container` is a JSXExpressionContainer.
/// <Foo bar baz="bang" container={4}/>
///   {4}  // <- wrapped in container
/// </Foo>
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXExpressionContainer<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The expression inside the container.
    pub expression: JSXExpression<'a>,
}

inherit_variants! {
/// JSX Expression
///
/// Gets wrapped by a [`JSXExpressionContainer`]. Inherits variants from [`Expression`]. See [`ast`
/// module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum JSXExpression<'a> {
    /// An empty expression
    ///
    /// ## Example
    /// ```tsx
    /// <Foo bar={} />
    /// //       ^^
    /// ```
    EmptyExpression(JSXEmptyExpression) = 64,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// An empty JSX expression (`{}`)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXEmptyExpression {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
}

// 1.3 JSX Attributes

/// JSX Attributes
///
/// ## Example
///
/// ```tsx
/// <Component foo="bar" baz={4} {...rest} />
/// //         ^^^^^^^^^ ^^^^^^^ ^^^^^^^^^
/// //             Attribute     SpreadAttribute
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum JSXAttributeItem<'a> {
    /// A `key="value"` attribute
    Attribute(Box<'a, JSXAttribute<'a>>) = 0,
    /// a `{...spread}` attribute
    SpreadAttribute(Box<'a, JSXSpreadAttribute<'a>>) = 1,
}

/// JSX Attribute
///
/// An attribute in a JSX opening tag. May or may not have a value. Part of
/// [`JSXAttributeItem`].
///
/// ## Example
///
/// ```tsx
/// // `has-no-value` is a JSXAttribute with no value.
/// <Component has-no-value foo="foo" />
/// //                 name ^^^ ^^^^ value
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXAttribute<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The name of the attribute. This is a prop in React-like applications.
    pub name: JSXAttributeName<'a>,
    /// The value of the attribute. This can be a string literal, an expression,
    /// or an element. Will be [`None`] for boolean-like attributes (e.g.
    /// `<button disabled />`).
    pub value: Option<JSXAttributeValue<'a>>,
}

/// JSX Spread Attribute
///
/// ## Example
/// ```tsx
/// <Component {...props} />
/// //          ^^^^^^^^ argument
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXSpreadAttribute<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The expression being spread.
    pub argument: Expression<'a>,
}

/// JSX Attribute Name
///
/// Part of a [`JSXAttribute`].
///
/// "Normal" attributes will be a [`JSXIdentifier`], while namespaced attributes
/// will be a [`JSXNamespacedName`].
///
/// ## Example
///
/// ```tsx
/// const Foo = <Component foo="bar" />;
/// //                     ^^^ Identifier
/// const Bar = <Component foo:bar="baz" />;
/// //                     ^^^^^^^ NamespacedName
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum JSXAttributeName<'a> {
    /// An attribute name without a namespace prefix, e.g. `foo` in `foo="bar"`.
    Identifier(Box<'a, JSXIdentifier<'a>>) = 0,
    /// An attribute name with a namespace prefix, e.g. `foo:bar` in `foo:bar="baz"`.
    NamespacedName(Box<'a, JSXNamespacedName<'a>>) = 1,
}

/// JSX Attribute Value
///
/// Part of a [`JSXAttribute`].
///
/// You're most likely interested in [`StringLiteral`] and
/// [`JSXExpressionContainer`].
///
/// ## Example
///
/// ```tsx
/// //                        v ExpressionContainer storing a NumericLiteral
/// <Component foo="bar" baz={4} />
/// //              ^^^ StringLiteral
///
/// // not a very common case, but it is valid syntax. Could also be a fragment.
/// <Component foo=<Element /> />
/// //             ^^^^^^^^^^^ Element
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum JSXAttributeValue<'a> {
    /// `<Component foo="bar" />`
    StringLiteral(Box<'a, StringLiteral<'a>>) = 0,
    /// `<Component foo={someExpr} />`
    ExpressionContainer(Box<'a, JSXExpressionContainer<'a>>) = 1,
    /// `<Component foo=<Element /> />`
    Element(Box<'a, JSXElement<'a>>) = 2,
    /// `<Component foo=<></> />`
    Fragment(Box<'a, JSXFragment<'a>>) = 3,
}

/// JSX Identifier
///
/// Similar to [`IdentifierName`], but used in JSX elements.
///
/// [`IdentifierName`]: super::IdentifierName
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXIdentifier<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The name of the identifier.
    pub name: Atom<'a>,
}

// 1.4 JSX Children

/// JSX Child
///
/// Part of a [`JSXElement`].
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum JSXChild<'a> {
    /// `<Foo>Some Text</Foo>`
    Text(Box<'a, JSXText<'a>>) = 0,
    /// `<Foo><Child /></Foo>`
    Element(Box<'a, JSXElement<'a>>) = 1,
    /// `<Foo><></></Foo>`
    Fragment(Box<'a, JSXFragment<'a>>) = 2,
    /// `<Foo>{expression}</Foo>`
    ExpressionContainer(Box<'a, JSXExpressionContainer<'a>>) = 3,
    /// `<Foo>{...spread}</Foo>`
    Spread(Box<'a, JSXSpreadChild<'a>>) = 4,
}

/// JSX Spread Child.
///
/// Variant of [`JSXChild`] that represents an object spread (`{...expression}`).
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXSpreadChild<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The expression being spread.
    pub expression: Expression<'a>,
}

/// Text inside a JSX element.
///
/// Not to be confused with a [`StringLiteral`].
///
/// ## Example
///
/// ```tsx
/// <Foo>Some text</Foo>     // `Some Text` is a JSXText,
/// <Foo>"Some string"</Foo> // but `"Some string"` is a StringLiteral.
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSXText<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The text content.
    pub value: Atom<'a>,
}
