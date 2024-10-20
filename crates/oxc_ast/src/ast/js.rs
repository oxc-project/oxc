// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

use std::cell::Cell;

use oxc_allocator::{Box, CloneIn, Vec};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{cmp::ContentEq, hash::ContentHash, Atom, GetSpan, GetSpanMut, SourceType, Span};
use oxc_syntax::{
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
    reference::ReferenceId,
    scope::ScopeId,
    symbol::SymbolId,
};

use super::{macros::inherit_variants, *};

/// Represents the root of a JavaScript abstract syntax tree (AST), containing metadata about the source, directives, top-level statements, and scope information.
#[ast(visit)]
#[scope(
    flags(ScopeFlags::Top),
    strict_if(self.source_type.is_strict() || self.directives.iter().any(Directive::is_use_strict)),
)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct Program<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub source_type: SourceType,
    #[estree(skip)]
    pub source_text: &'a str,
    /// Sorted comments
    #[estree(skip)]
    pub comments: Vec<'a, Comment>,
    pub hashbang: Option<Hashbang<'a>>,
    pub directives: Vec<'a, Directive<'a>>,
    pub body: Vec<'a, Statement<'a>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

inherit_variants! {
/// Represents a type for AST nodes corresponding to JavaScript's expressions.
///
/// Inherits variants from [`MemberExpression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum Expression<'a> {
    /// See [`BooleanLiteral`] for AST node details.
    BooleanLiteral(Box<'a, BooleanLiteral>) = 0,
    /// See [`NullLiteral`] for AST node details.
    NullLiteral(Box<'a, NullLiteral>) = 1,
    /// See [`NumericLiteral`] for AST node details.
    NumericLiteral(Box<'a, NumericLiteral<'a>>) = 2,
    /// See [`BigIntLiteral`] for AST node details.
    BigIntLiteral(Box<'a, BigIntLiteral<'a>>) = 3,
    /// See [`RegExpLiteral`] for AST node details.
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>) = 4,
    /// See [`StringLiteral`] for AST node details.
    StringLiteral(Box<'a, StringLiteral<'a>>) = 5,
    /// See [`TemplateLiteral`] for AST node details.
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>) = 6,

    /// See [`IdentifierReference`] for AST node details.
    Identifier(Box<'a, IdentifierReference<'a>>) = 7,

    /// See [`MetaProperty`] for AST node details.
    MetaProperty(Box<'a, MetaProperty<'a>>) = 8,
    /// See [`Super`] for AST node details.
    Super(Box<'a, Super>) = 9,

    /// See [`ArrayExpression`] for AST node details.
    ArrayExpression(Box<'a, ArrayExpression<'a>>) = 10,
    /// See [`ArrowFunctionExpression`] for AST node details.
    ArrowFunctionExpression(Box<'a, ArrowFunctionExpression<'a>>) = 11,
    /// See [`AssignmentExpression`] for AST node details.
    AssignmentExpression(Box<'a, AssignmentExpression<'a>>) = 12,
    /// See [`AwaitExpression`] for AST node details.
    AwaitExpression(Box<'a, AwaitExpression<'a>>) = 13,
    /// See [`BinaryExpression`] for AST node details.
    BinaryExpression(Box<'a, BinaryExpression<'a>>) = 14,
    /// See [`CallExpression`] for AST node details.
    CallExpression(Box<'a, CallExpression<'a>>) = 15,
    /// See [`ChainExpression`] for AST node details.
    ChainExpression(Box<'a, ChainExpression<'a>>) = 16,
    /// See [`Class`] for AST node details.
    ClassExpression(Box<'a, Class<'a>>) = 17,
    /// See [`ConditionalExpression`] for AST node details.
    ConditionalExpression(Box<'a, ConditionalExpression<'a>>) = 18,
    /// See [`Function`] for AST node details.
    #[visit(args(flags = ScopeFlags::Function))]
    FunctionExpression(Box<'a, Function<'a>>) = 19,
    /// See [`ImportExpression`] for AST node details.
    ImportExpression(Box<'a, ImportExpression<'a>>) = 20,
    /// See [`LogicalExpression`] for AST node details.
    LogicalExpression(Box<'a, LogicalExpression<'a>>) = 21,
    /// See [`NewExpression`] for AST node details.
    NewExpression(Box<'a, NewExpression<'a>>) = 22,
    /// See [`ObjectExpression`] for AST node details.
    ObjectExpression(Box<'a, ObjectExpression<'a>>) = 23,
    /// See [`ParenthesizedExpression`] for AST node details.
    ParenthesizedExpression(Box<'a, ParenthesizedExpression<'a>>) = 24,
    /// See [`SequenceExpression`] for AST node details.
    SequenceExpression(Box<'a, SequenceExpression<'a>>) = 25,
    /// See [`TaggedTemplateExpression`] for AST node details.
    TaggedTemplateExpression(Box<'a, TaggedTemplateExpression<'a>>) = 26,
    /// See [`ThisExpression`] for AST node details.
    ThisExpression(Box<'a, ThisExpression>) = 27,
    /// See [`UnaryExpression`] for AST node details.
    UnaryExpression(Box<'a, UnaryExpression<'a>>) = 28,
    /// See [`UpdateExpression`] for AST node details.
    UpdateExpression(Box<'a, UpdateExpression<'a>>) = 29,
    /// See [`YieldExpression`] for AST node details.
    YieldExpression(Box<'a, YieldExpression<'a>>) = 30,
    /// See [`PrivateInExpression`] for AST node details.
    PrivateInExpression(Box<'a, PrivateInExpression<'a>>) = 31,

    /// See [`JSXElement`] for AST node details.
    JSXElement(Box<'a, JSXElement<'a>>) = 32,
    /// See [`JSXFragment`] for AST node details.
    JSXFragment(Box<'a, JSXFragment<'a>>) = 33,

    /// See [`TSAsExpression`] for AST node details.
    TSAsExpression(Box<'a, TSAsExpression<'a>>) = 34,
    /// See [`TSSatisfiesExpression`] for AST node details.
    TSSatisfiesExpression(Box<'a, TSSatisfiesExpression<'a>>) = 35,
    /// See [`TSTypeAssertion`] for AST node details.
    TSTypeAssertion(Box<'a, TSTypeAssertion<'a>>) = 36,
    /// See [`TSNonNullExpression`] for AST node details.
    TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>) = 37,
    /// See [`TSInstantiationExpression`] for AST node details.
    TSInstantiationExpression(Box<'a, TSInstantiationExpression<'a>>) = 38,

    // `MemberExpression` variants added here by `inherit_variants!` macro
    @inherit MemberExpression
}
}

/// Macro for matching `Expression`'s variants.
/// Includes `MemberExpression`'s variants.
#[macro_export]
macro_rules! match_expression {
    ($ty:ident) => {
        $ty::BooleanLiteral(_)
            | $ty::NullLiteral(_)
            | $ty::NumericLiteral(_)
            | $ty::BigIntLiteral(_)
            | $ty::RegExpLiteral(_)
            | $ty::StringLiteral(_)
            | $ty::TemplateLiteral(_)
            | $ty::Identifier(_)
            | $ty::MetaProperty(_)
            | $ty::Super(_)
            | $ty::ArrayExpression(_)
            | $ty::ArrowFunctionExpression(_)
            | $ty::AssignmentExpression(_)
            | $ty::AwaitExpression(_)
            | $ty::BinaryExpression(_)
            | $ty::CallExpression(_)
            | $ty::ChainExpression(_)
            | $ty::ClassExpression(_)
            | $ty::ConditionalExpression(_)
            | $ty::FunctionExpression(_)
            | $ty::ImportExpression(_)
            | $ty::LogicalExpression(_)
            | $ty::NewExpression(_)
            | $ty::ObjectExpression(_)
            | $ty::ParenthesizedExpression(_)
            | $ty::SequenceExpression(_)
            | $ty::TaggedTemplateExpression(_)
            | $ty::ThisExpression(_)
            | $ty::UnaryExpression(_)
            | $ty::UpdateExpression(_)
            | $ty::YieldExpression(_)
            | $ty::PrivateInExpression(_)
            | $ty::JSXElement(_)
            | $ty::JSXFragment(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSInstantiationExpression(_)
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
    };
}
pub use match_expression;

/// `foo` in `let foo = 1;`
///
/// Fundamental syntactic structure used for naming variables, functions, and properties. It must start with a Unicode letter (including $ and _) and can be followed by Unicode letters, digits, $, or _.
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(type = "Identifier")]
pub struct IdentifierName<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
}

/// `x` inside `func` in `const x = 0; function func() { console.log(x); }`
///
/// Represents an identifier reference, which is a reference to a variable, function, class, or object.
///
/// See: [13.1 Identifiers](https://tc39.es/ecma262/#sec-identifiers)
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(type = "Identifier")]
pub struct IdentifierReference<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// The name of the identifier being referenced.
    pub name: Atom<'a>,
    /// Reference ID
    ///
    /// Identifies what identifier this refers to, and how it is used. This is
    /// set in the bind step of semantic analysis, and will always be [`None`]
    /// immediately after parsing.
    #[estree(skip)]
    #[clone_in(default)]
    pub reference_id: Cell<Option<ReferenceId>>,
}

/// `x` in `const x = 0;`
///
/// Represents a binding identifier, which is an identifier that is used to declare a variable, function, class, or object.
///
/// See: [13.1 Identifiers](https://tc39.es/ecma262/#sec-identifiers)
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(type = "Identifier")]
pub struct BindingIdentifier<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// The identifier name being bound.
    pub name: Atom<'a>,
    /// Unique identifier for this binding.
    ///
    /// This gets initialized during [`semantic analysis`] in the bind step. If
    /// you choose to skip semantic analysis, this will always be [`None`].
    ///
    /// [`semantic analysis`]: <https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.SemanticBuilder.html>
    #[estree(skip)]
    #[clone_in(default)]
    pub symbol_id: Cell<Option<SymbolId>>,
}

/// `loop` in `loop: while (true) { break loop; }`
///
/// Represents a label identifier, which is an identifier that is used to label a statement.
///
/// See: [13.1 Identifiers](https://tc39.es/ecma262/#sec-identifiers)
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(type = "Identifier")]
pub struct LabelIdentifier<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
}

/// `this` in `return this.prop;`
///
/// Represents a `this` expression, which is a reference to the current object.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ThisExpression {
    #[estree(flatten)]
    pub span: Span,
}

/// `[1, 2, ...[3, 4], null]` in `const array = [1, 2, ...[3, 4], null];`
///
/// Represents an array literal, which can include elements, spread elements, or null values.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ArrayExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    #[tsify(type = "Array<SpreadElement | Expression | null>")]
    pub elements: Vec<'a, ArrayExpressionElement<'a>>,
    /// Array trailing comma
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Trailing_commas#arrays>
    #[estree(skip)]
    pub trailing_comma: Option<Span>,
}

inherit_variants! {
/// Represents a element in an array literal.
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged, custom_ts_def)]
pub enum ArrayExpressionElement<'a> {
    /// `...[3, 4]` in `const array = [1, 2, ...[3, 4], null];`
    SpreadElement(Box<'a, SpreadElement<'a>>) = 64,
    /// `<empty>` in `const array = [1, , 2];`
    ///
    /// Array hole for sparse arrays
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Trailing_commas#arrays>
    Elision(Elision) = 65,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// empty slot in `const array = [1, , 2];`
///
/// Array Expression Elision Element
/// Serialized as `null` in JSON AST. See `serialize.rs`.
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash)]
pub struct Elision {
    pub span: Span,
}

/// `{ a: 1 }` in `const obj = { a: 1 };`
///
/// Represents an object literal, which can include properties, spread properties, or computed properties and trailing comma.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ObjectExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// Properties declared in the object
    pub properties: Vec<'a, ObjectPropertyKind<'a>>,
    #[estree(skip)]
    pub trailing_comma: Option<Span>,
}

/// Represents a property in an object literal.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ObjectPropertyKind<'a> {
    /// `a: 1` in `const obj = { a: 1 };`
    ObjectProperty(Box<'a, ObjectProperty<'a>>) = 0,
    /// `...{ a: 1 }` in `const obj = { ...{ a: 1 } };`
    SpreadProperty(Box<'a, SpreadElement<'a>>) = 1,
}

/// `a: 1` in `const obj = { a: 1 };`
///
/// Represents a property in an object literal.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ObjectProperty<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub kind: PropertyKind,
    pub key: PropertyKey<'a>,
    pub value: Expression<'a>,
    pub init: Option<Expression<'a>>, // for `CoverInitializedName`
    pub method: bool,
    pub shorthand: bool,
    pub computed: bool,
}

inherit_variants! {
/// Property Key
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum PropertyKey<'a> {
    /// `a` in `const obj = { a: 1 }; obj.a;`
    StaticIdentifier(Box<'a, IdentifierName<'a>>) = 64,
    /// `#a` in `class C { #a = 1; }; const c = new C(); c.#a;`
    PrivateIdentifier(Box<'a, PrivateIdentifier<'a>>) = 65,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// Represents the kind of property in an object literal or class.
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(rename_all = "camelCase")]
pub enum PropertyKind {
    /// `a: 1` in `const obj = { a: 1 };`
    Init = 0,
    /// `get a() { return 1; }` in `const obj = { get a() { return 1; } };`
    Get = 1,
    /// `set a(value) { this._a = value; }` in `const obj = { set a(value) { this._a = value; } };`
    Set = 2,
}

/// `` `Hello, ${name}` `` in `` const foo = `Hello, ${name}` ``
///
/// Represents a template literal, which can include quasi elements and expression elements.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TemplateLiteral<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement<'a>>,
    pub expressions: Vec<'a, Expression<'a>>,
}

/// `` foo`Hello, ${name}` `` in `` const foo = foo`Hello, ${name}` ``
///
/// Represents a tagged template expression, which can include a tag and a quasi.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TaggedTemplateExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub tag: Expression<'a>,
    pub quasi: TemplateLiteral<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// `Hello, ` in `` `Hello, ${name}` ``
///
/// Represents a quasi element in a template literal.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TemplateElement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub tail: bool,
    pub value: TemplateElementValue<'a>,
}

/// See [template-strings-cooked-vs-raw](https://exploringjs.com/js/book/ch_template-literals.html#template-strings-cooked-vs-raw)
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(no_type)]
pub struct TemplateElementValue<'a> {
    /// A raw interpretation where backslashes do not have special meaning.
    /// For example, \t produces two characters – a backslash and a t.
    /// This interpretation of the template strings is stored in property .raw of the first argument (an Array).
    pub raw: Atom<'a>,
    /// A cooked interpretation where backslashes have special meaning.
    /// For example, \t produces a tab character.
    /// This interpretation of the template strings is stored as an Array in the first argument.
    /// cooked = None when template literal has invalid escape sequence
    pub cooked: Option<Atom<'a>>,
}

/// Represents a member access expression, which can include computed member access, static member access, or private field access.
///
/// <https://tc39.es/ecma262/#prod-MemberExpression>
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum MemberExpression<'a> {
    /// `ar[0]` in `const ar = [1, 2]; ar[0];`
    ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>) = 48,
    /// `console.log` in `console.log('Hello, World!');`
    StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>) = 49,
    /// `c.#a` in `class C { #a = 1; }; const c = new C(); c.#a;`
    PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>) = 50,
}

/// Macro for matching `MemberExpression`'s variants.
#[macro_export]
macro_rules! match_member_expression {
    ($ty:ident) => {
        $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
    };
}
pub use match_member_expression;

/// `ar[0]` in `const ar = [1, 2]; ar[0];`
///
/// Represents a computed member access expression, which can include an object and an expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ComputedMemberExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub object: Expression<'a>,
    pub expression: Expression<'a>,
    pub optional: bool, // for optional chaining
}

/// `console.log` in `console.log('Hello, World!');`
///
/// Represents a static member access expression, which can include an object and a property.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct StaticMemberExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub object: Expression<'a>,
    pub property: IdentifierName<'a>,
    pub optional: bool, // for optional chaining
}

/// `c.#a` in `class C { #a = 1; }; const c = new C(); c.#a;`
///
/// Represents a private field access expression, which can include an object and a private identifier.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct PrivateFieldExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub object: Expression<'a>,
    pub field: PrivateIdentifier<'a>,
    pub optional: bool, // for optional chaining
}

/// `foo()` in `function foo() { return 1; }; foo();`
///
/// Represents a call expression, which can include a callee and arguments.
///
/// ## Examples
/// ```ts
/// //        ___ callee
/// const x = foo(1, 2)
///
/// //            ^^^^ arguments
/// const y = foo.bar?.(1, 2)
/// //               ^ optional
///
/// const z = foo<number, string>(1, 2)
/// //            ^^^^^^^^^^^^^^ type_parameters
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct CallExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub callee: Expression<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    pub arguments: Vec<'a, Argument<'a>>,
    pub optional: bool, // for optional chaining
}

/// `new C()` in `class C {}; new C();`
///
/// Represents a new expression, which can include a callee and arguments.
///
/// ## Example
/// ```ts
/// //           callee         arguments
/// //              ↓↓↓         ↓↓↓↓
/// const foo = new Foo<number>(1, 2)
/// //                 ↑↑↑↑↑↑↑↑
/// //                 type_parameters
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct NewExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub callee: Expression<'a>,
    pub arguments: Vec<'a, Argument<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// `import.meta` in `console.log(import.meta);`
///
/// Represents a meta property. The following syntaxes are supported. `import.meta`, `new.target`.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct MetaProperty<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub meta: IdentifierName<'a>,
    pub property: IdentifierName<'a>,
}

/// `...[1, 2]` in `const arr = [...[1, 2]];`
///
/// Represents a spread element, which can include an argument.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct SpreadElement<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// The expression being spread.
    pub argument: Expression<'a>,
}

inherit_variants! {
/// Argument
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum Argument<'a> {
    /// `...[1, 2]` in `const arr = [...[1, 2]];`
    SpreadElement(Box<'a, SpreadElement<'a>>) = 64,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// `++i` in `let i = 0; ++i;`
///
/// Represents an update expression, which can include an operator and an argument. The following syntaxes are supported. `++a`, `a++`, `--a`, `a--`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct UpdateExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub operator: UpdateOperator,
    pub prefix: bool,
    pub argument: SimpleAssignmentTarget<'a>,
}

/// `typeof` in `typeof a === "string"`
///
/// Represents a unary expression, which can include an operator and an argument. The following syntaxes are supported. `+a`, `-a`, `~a`, `!a`, `delete a`, `void a`, `typeof a`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct UnaryExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub operator: UnaryOperator,
    pub argument: Expression<'a>,
}

/// `1 + 1` in `const two = 1 + 1;`
///
/// Represents a binary expression, which can include a left expression, an operator, and a right expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct BinaryExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub left: Expression<'a>,
    pub operator: BinaryOperator,
    pub right: Expression<'a>,
}

/// `#brand in obj` in `class Foo { #brand; static isFoo(obj) { return #brand in obj; } }`
///
/// Represents a private in expression, which can include a private identifier, an operator, and a expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct PrivateInExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub left: PrivateIdentifier<'a>,
    pub operator: BinaryOperator, // BinaryOperator::In
    pub right: Expression<'a>,
}

/// `||` in `const foo = bar || 2;`
///
/// Represents a logical expression, which can include a left expression, an operator, and a right expression. The following syntaxes are supported. `||`, `&&` and `??`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct LogicalExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub left: Expression<'a>,
    pub operator: LogicalOperator,
    pub right: Expression<'a>,
}

/// `bar ? 1 : 2` in `const foo = bar ? 1 : 2;`
///
/// Represents a conditional expression, which can include a test, a consequent, and an alternate.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ConditionalExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub test: Expression<'a>,
    pub consequent: Expression<'a>,
    pub alternate: Expression<'a>,
}

/// `foo = 1` in `let foo; foo = 1;`
///
/// Represents an assignment expression, which can include an operator, a target, and a expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct AssignmentExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub operator: AssignmentOperator,
    pub left: AssignmentTarget<'a>,
    pub right: Expression<'a>,
}

inherit_variants! {
/// Destructuring Assignment
///
/// Inherits variants from [`SimpleAssignmentTarget`] and [`AssignmentTargetPattern`].
/// See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum AssignmentTarget<'a> {
    // `SimpleAssignmentTarget` variants added here by `inherit_variants!` macro
    @inherit SimpleAssignmentTarget
    // `AssignmentTargetPattern` variants added here by `inherit_variants!` macro
    @inherit AssignmentTargetPattern
}
}

inherit_variants! {
/// Simple Assignment Target
///
/// Inherits variants from [`MemberExpression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum SimpleAssignmentTarget<'a> {
    AssignmentTargetIdentifier(Box<'a, IdentifierReference<'a>>) = 0,
    TSAsExpression(Box<'a, TSAsExpression<'a>>) = 1,
    TSSatisfiesExpression(Box<'a, TSSatisfiesExpression<'a>>) = 2,
    TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>) = 3,
    TSTypeAssertion(Box<'a, TSTypeAssertion<'a>>) = 4,
    TSInstantiationExpression(Box<'a, TSInstantiationExpression<'a>>) = 5,
    // `MemberExpression` variants added here by `inherit_variants!` macro
    @inherit MemberExpression
}
}

/// Macro for matching `AssignmentTarget`'s variants.
/// Includes `SimpleAssignmentTarget`'s and `AssignmentTargetPattern`'s variants.
#[macro_export]
macro_rules! match_assignment_target {
    ($ty:ident) => {
        $ty::AssignmentTargetIdentifier(_)
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::TSInstantiationExpression(_)
            | $ty::ArrayAssignmentTarget(_)
            | $ty::ObjectAssignmentTarget(_)
    };
}
pub use match_assignment_target;

/// Macro for matching `SimpleAssignmentTarget`'s variants.
/// Includes `MemberExpression`'s variants
#[macro_export]
macro_rules! match_simple_assignment_target {
    ($ty:ident) => {
        $ty::AssignmentTargetIdentifier(_)
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::TSInstantiationExpression(_)
    };
}
pub use match_simple_assignment_target;

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum AssignmentTargetPattern<'a> {
    ArrayAssignmentTarget(Box<'a, ArrayAssignmentTarget<'a>>) = 8,
    ObjectAssignmentTarget(Box<'a, ObjectAssignmentTarget<'a>>) = 9,
}

/// Macro for matching `AssignmentTargetPattern`'s variants.
#[macro_export]
macro_rules! match_assignment_target_pattern {
    ($ty:ident) => {
        $ty::ArrayAssignmentTarget(_) | $ty::ObjectAssignmentTarget(_)
    };
}
pub use match_assignment_target_pattern;

/// `[a, b]` in `[a, b] = arr;`
///
/// Represents an array assignment target, which can include elements and a rest element.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(custom_serialize)]
pub struct ArrayAssignmentTarget<'a> {
    #[estree(flatten)]
    pub span: Span,
    #[tsify(type = "Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>")]
    pub elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
    #[estree(skip)]
    pub rest: Option<AssignmentTargetRest<'a>>,
    #[estree(skip)]
    pub trailing_comma: Option<Span>,
}

/// `{ foo }` in `({ foo } = obj);`
///
/// Represents an object assignment target, which can include properties and a rest element.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(custom_serialize)]
pub struct ObjectAssignmentTarget<'a> {
    #[estree(flatten)]
    pub span: Span,
    #[tsify(type = "Array<AssignmentTargetProperty | AssignmentTargetRest>")]
    pub properties: Vec<'a, AssignmentTargetProperty<'a>>,
    #[estree(skip)]
    pub rest: Option<AssignmentTargetRest<'a>>,
}

/// `rest` in `[foo, ...rest] = arr;`
///
/// Represents a rest element in an array assignment target, which can include a target.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(type = "RestElement")]
pub struct AssignmentTargetRest<'a> {
    #[estree(flatten)]
    pub span: Span,
    #[estree(rename = "argument")]
    pub target: AssignmentTarget<'a>,
}

inherit_variants! {
/// Assignment Target Maybe Default
///
/// Inherits variants from [`AssignmentTarget`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum AssignmentTargetMaybeDefault<'a> {
    AssignmentTargetWithDefault(Box<'a, AssignmentTargetWithDefault<'a>>) = 16,
    // `AssignmentTarget` variants added here by `inherit_variants!` macro
    @inherit AssignmentTarget
}
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct AssignmentTargetWithDefault<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub binding: AssignmentTarget<'a>,
    pub init: Expression<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum AssignmentTargetProperty<'a> {
    AssignmentTargetPropertyIdentifier(Box<'a, AssignmentTargetPropertyIdentifier<'a>>) = 0,
    AssignmentTargetPropertyProperty(Box<'a, AssignmentTargetPropertyProperty<'a>>) = 1,
}

/// `foo` in `({ foo } = obj);`
///
/// Represents an assignment target property identifier, which can include a binding and an init expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct AssignmentTargetPropertyIdentifier<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub binding: IdentifierReference<'a>,
    pub init: Option<Expression<'a>>,
}

/// `foo: bar` in `({ foo: bar } = obj);`
///
/// Represents an assignment target property property, which can include a name and a binding.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct AssignmentTargetPropertyProperty<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub name: PropertyKey<'a>,
    pub binding: AssignmentTargetMaybeDefault<'a>,
}

/// `a++, b++` in `let a = 1, b = 2; let result = (a++, b++);`
///
/// Represents a sequence expression, which can include expressions.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct SequenceExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub expressions: Vec<'a, Expression<'a>>,
}

/// `super` in `class C extends B { constructor() { super(); } }`
///
/// Represents a super expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct Super {
    #[estree(flatten)]
    pub span: Span,
}

/// `await` in `await foo();`
///
/// Represents an await expression, which can include an argument.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct AwaitExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// `foo?.bar` in `foo?.bar;`
///
/// Represents a chain expression, which can include an expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ChainExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub expression: ChainElement<'a>,
}

inherit_variants! {
/// Chain Element
///
/// Inherits variants from [`MemberExpression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ChainElement<'a> {
    CallExpression(Box<'a, CallExpression<'a>>) = 0,
    // `MemberExpression` variants added here by `inherit_variants!` macro
    @inherit MemberExpression
}
}

/// `(a + b)` in `const res = (a + b) / c;`
///
/// Represents a parenthesized expression, which can include an expression.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ParenthesizedExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

inherit_variants! {
/// Statement
///
/// Inherits variants from [`Declaration`] and [`ModuleDeclaration`].
/// See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum Statement<'a> {
    // Statements
    BlockStatement(Box<'a, BlockStatement<'a>>) = 0,
    BreakStatement(Box<'a, BreakStatement<'a>>) = 1,
    ContinueStatement(Box<'a, ContinueStatement<'a>>) = 2,
    DebuggerStatement(Box<'a, DebuggerStatement>) = 3,
    DoWhileStatement(Box<'a, DoWhileStatement<'a>>) = 4,
    EmptyStatement(Box<'a, EmptyStatement>) = 5,
    ExpressionStatement(Box<'a, ExpressionStatement<'a>>) = 6,
    ForInStatement(Box<'a, ForInStatement<'a>>) = 7,
    ForOfStatement(Box<'a, ForOfStatement<'a>>) = 8,
    ForStatement(Box<'a, ForStatement<'a>>) = 9,
    IfStatement(Box<'a, IfStatement<'a>>) = 10,
    LabeledStatement(Box<'a, LabeledStatement<'a>>) = 11,
    ReturnStatement(Box<'a, ReturnStatement<'a>>) = 12,
    SwitchStatement(Box<'a, SwitchStatement<'a>>) = 13,
    ThrowStatement(Box<'a, ThrowStatement<'a>>) = 14,
    TryStatement(Box<'a, TryStatement<'a>>) = 15,
    WhileStatement(Box<'a, WhileStatement<'a>>) = 16,
    WithStatement(Box<'a, WithStatement<'a>>) = 17,
    // `Declaration` variants added here by `inherit_variants!` macro
    @inherit Declaration
    // `ModuleDeclaration` variants added here by `inherit_variants!` macro
    @inherit ModuleDeclaration
}
}

/// `"use strict";` in `"use strict";`
///
/// Represents a directive statement, which can include a string literal.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct Directive<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// Directive with any escapes unescaped
    pub expression: StringLiteral<'a>,
    /// Raw content of directive as it appears in source, any escapes left as is
    pub directive: Atom<'a>,
}

/// `#! /usr/bin/env node` in `#! /usr/bin/env node`
///
/// Represents a hashbang directive, which can include a value.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct Hashbang<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub value: Atom<'a>,
}

/// `{ let foo = 1; }` in `if(true) { let foo = 1; }`
///
/// Represents a block statement, which can include a body.
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct BlockStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Declarations and the Variable Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum Declaration<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 32,
    #[visit(args(flags = ScopeFlags::Function))]
    FunctionDeclaration(Box<'a, Function<'a>>) = 33,
    ClassDeclaration(Box<'a, Class<'a>>) = 34,

    TSTypeAliasDeclaration(Box<'a, TSTypeAliasDeclaration<'a>>) = 35,
    TSInterfaceDeclaration(Box<'a, TSInterfaceDeclaration<'a>>) = 36,
    TSEnumDeclaration(Box<'a, TSEnumDeclaration<'a>>) = 37,
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>) = 38,
    TSImportEqualsDeclaration(Box<'a, TSImportEqualsDeclaration<'a>>) = 39,
}

/// Macro for matching `Declaration`'s variants.
#[macro_export]
macro_rules! match_declaration {
    ($ty:ident) => {
        $ty::VariableDeclaration(_)
            | $ty::FunctionDeclaration(_)
            | $ty::ClassDeclaration(_)
            | $ty::TSTypeAliasDeclaration(_)
            | $ty::TSInterfaceDeclaration(_)
            | $ty::TSEnumDeclaration(_)
            | $ty::TSModuleDeclaration(_)
            | $ty::TSImportEqualsDeclaration(_)
    };
}
pub use match_declaration;

/// `let a;` in `let a; a = 1;`
///
/// Represents a variable declaration, which can include a kind, declarations, and modifiers.
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct VariableDeclaration<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub kind: VariableDeclarationKind,
    pub declarations: Vec<'a, VariableDeclarator<'a>>,
    pub declare: bool,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(rename_all = "camelCase")]
pub enum VariableDeclarationKind {
    Var = 0,
    Const = 1,
    Let = 2,
    Using = 3,
    #[estree(rename = "await using")]
    AwaitUsing = 4,
}

/// A single variable declaration in a list of [variable declarations](VariableDeclaration).
///
/// ## Examples
/// ```ts
/// // declarators may or may not have initializers
/// let foo, b = 1;
/// //  ^^^ id   ^ init
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct VariableDeclarator<'a> {
    #[estree(flatten)]
    pub span: Span,
    #[estree(skip)]
    pub kind: VariableDeclarationKind,
    pub id: BindingPattern<'a>,
    pub init: Option<Expression<'a>>,
    pub definite: bool,
}

/// Empty Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct EmptyStatement {
    #[estree(flatten)]
    pub span: Span,
}

/// Expression Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ExpressionStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// If Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct IfStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub test: Expression<'a>,
    pub consequent: Statement<'a>,
    pub alternate: Option<Statement<'a>>,
}

/// Do-While Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct DoWhileStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub body: Statement<'a>,
    pub test: Expression<'a>,
}

/// While Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct WhileStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub test: Expression<'a>,
    pub body: Statement<'a>,
}

/// For Statement
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ForStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub init: Option<ForStatementInit<'a>>,
    pub test: Option<Expression<'a>>,
    pub update: Option<Expression<'a>>,
    pub body: Statement<'a>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

inherit_variants! {
/// For Statement Init
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ForStatementInit<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 64,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// For-In Statement
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ForInStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub body: Statement<'a>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

inherit_variants! {
/// For Statement Left
///
/// Inherits variants from [`AssignmentTarget`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ForStatementLeft<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 16,
    // `AssignmentTarget` variants added here by `inherit_variants!` macro
    @inherit AssignmentTarget
}
}
/// For-Of Statement
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ForOfStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub r#await: bool,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub body: Statement<'a>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Continue Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ContinueStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub label: Option<LabelIdentifier<'a>>,
}

/// Break Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct BreakStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub label: Option<LabelIdentifier<'a>>,
}

/// Return Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ReturnStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub argument: Option<Expression<'a>>,
}

/// With Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct WithStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub object: Expression<'a>,
    pub body: Statement<'a>,
}

/// Switch Statement
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct SwitchStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub discriminant: Expression<'a>,
    #[scope(enter_before)]
    pub cases: Vec<'a, SwitchCase<'a>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct SwitchCase<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub test: Option<Expression<'a>>,
    pub consequent: Vec<'a, Statement<'a>>,
}

/// Labelled Statement
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct LabeledStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub label: LabelIdentifier<'a>,
    pub body: Statement<'a>,
}

/// Throw Statement
///
/// # Example
/// ```ts
/// throw new Error('something went wrong!');
/// //    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ argument
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ThrowStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// The expression being thrown, e.g. `err` in `throw err;`
    pub argument: Expression<'a>,
}

/// Try Statement
///
/// # Example
/// ```ts
/// var x;
/// let didRun = false;
///
/// try {                 // block
///     x = 1;
/// } catch (e) {         // handler
///     console.error(e);
/// } finally {           // finalizer
///     didRun = true;
/// }
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TryStatement<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// Statements in the `try` block
    pub block: Box<'a, BlockStatement<'a>>,
    /// The `catch` clause, including the parameter and the block statement
    pub handler: Option<Box<'a, CatchClause<'a>>>,
    /// The `finally` clause
    #[visit(as(FinallyClause))]
    pub finalizer: Option<Box<'a, BlockStatement<'a>>>,
}

/// Catch Clause in a [`try/catch` statement](TryStatement).
///
/// This node creates a new scope inside its `body`.
///
/// # Example
/// ```ts
/// try {
///   throw new Error('foo');
/// } catch (e) {             // `param` is `e`
///   console.error(e);       // `body`
/// }
/// ```
#[ast(visit)]
#[scope(flags(ScopeFlags::CatchClause))]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct CatchClause<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// The caught error parameter, e.g. `e` in `catch (e) {}`
    pub param: Option<CatchParameter<'a>>,
    /// The statements run when an error is caught
    pub body: Box<'a, BlockStatement<'a>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// A caught error parameter in a [catch clause](CatchClause).
///
/// # Examples
///
/// ```ts
/// try {} catch (err) {}
/// //            ^^^ pattern
/// ```
///
/// ```ts
/// try {} catch ({ err }) {}
/// //            ^^^^^^^  pattern
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct CatchParameter<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// The bound error
    pub pattern: BindingPattern<'a>,
}

/// Debugger Statement
///
/// # Example
/// ```ts
/// let x = 1;
/// debugger; // <--
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct DebuggerStatement {
    #[estree(flatten)]
    pub span: Span,
}

/// Destructuring Binding Patterns
/// * <https://tc39.es/ecma262/#prod-BindingPattern>
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(no_type)]
pub struct BindingPattern<'a> {
    // serde(flatten) the attributes because estree has no `BindingPattern`
    #[estree(flatten)]
    #[tsify(type = "(BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern)")]
    #[span]
    pub kind: BindingPatternKind<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub optional: bool,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum BindingPatternKind<'a> {
    /// `const a = 1`
    BindingIdentifier(Box<'a, BindingIdentifier<'a>>) = 0,
    /// `const {a} = 1`
    ObjectPattern(Box<'a, ObjectPattern<'a>>) = 1,
    /// `const [a] = 1`
    ArrayPattern(Box<'a, ArrayPattern<'a>>) = 2,
    /// A defaulted binding pattern, i.e.:
    /// `const {a = 1} = 1`
    /// the assignment pattern is `a = 1`
    /// it has an inner left that has a BindingIdentifier
    AssignmentPattern(Box<'a, AssignmentPattern<'a>>) = 3,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct AssignmentPattern<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub left: BindingPattern<'a>,
    pub right: Expression<'a>,
}

// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(custom_serialize)]
pub struct ObjectPattern<'a> {
    #[estree(flatten)]
    pub span: Span,
    #[tsify(type = "Array<BindingProperty | BindingRestElement>")]
    pub properties: Vec<'a, BindingProperty<'a>>,
    #[estree(skip)]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct BindingProperty<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: BindingPattern<'a>,
    pub shorthand: bool,
    pub computed: bool,
}

// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(custom_serialize)]
pub struct ArrayPattern<'a> {
    #[estree(flatten)]
    pub span: Span,
    #[tsify(type = "Array<BindingPattern | BindingRestElement | null>")]
    pub elements: Vec<'a, Option<BindingPattern<'a>>>,
    #[estree(skip)]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

/// A `...rest` binding in an [array](ArrayPattern) or [object](ObjectPattern) destructure.
///
/// ## Examples
/// ```ts
/// const [a, ...rest] = [1, 2, 3];
/// //           ^^^^  argument
/// const { x, y, ...others} = foo.bar();
/// //               ^^^^^^  argument
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(type = "RestElement")]
pub struct BindingRestElement<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub argument: BindingPattern<'a>,
}

/// Function Statement or Expression
///
/// Includes generator functions and function-valued class properties.
/// Arrow functions are represented by [`ArrowFunctionExpression`].
///
/// # Examples
/// ```ts
/// //    id ___             ____ return_type
/// function foo(a: number): void {
/// //           ^^^^^^^^^ params
///     console.log(a);
/// }
/// ```
///
/// ```ts
/// // `async` and `generator` are true
/// async function* foo() {
///     yield 1;
/// }
/// ```
///
/// ```js
/// // function.id is None
/// // use function.r#type to check if a node is a function expression.
/// const foo = function() { }
/// ```
///
/// ```ts
/// // Function overloads will not have a body
/// function add(a: number, b: number): number; // <-- No body
/// function add(a: string, b: string): string; // <-- No body
/// function add(a: any, b: any): any {         // <-- Body is between `{}`, inclusive.
///    return a + b;
/// }
/// ```
#[ast(visit)]
#[scope(
    // `flags` passed in to visitor via parameter defined by `#[visit(args(flags = ...))]` on parents
    flags(flags),
    strict_if(self.is_strict()),
)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct Function<'a> {
    pub r#type: FunctionType,
    #[estree(flatten)]
    pub span: Span,
    /// The function identifier. [`None`] for anonymous function expressions.
    pub id: Option<BindingIdentifier<'a>>,
    /// Is this a generator function?
    ///
    /// ```ts
    /// function* foo() { } // <- generator: true
    /// function bar() { }  // <- generator: false
    /// ```
    pub generator: bool,
    pub r#async: bool,
    pub declare: bool,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    ///
    /// The JavaScript specification states that you cannot have a parameter called `this`,
    /// and so TypeScript uses that syntax space to let you declare the type for `this` in the function body.
    ///
    /// ```ts
    /// interface DB {
    ///     filterUsers(filter: (this: User) => boolean): User[];
    ///     //                   ^^^^
    /// }
    ///
    /// const db = getDB();
    /// const admins = db.filterUsers(function (this: User) {
    ///     return this.admin;
    /// });
    /// ```
    pub this_param: Option<Box<'a, TSThisParameter<'a>>>,
    /// Function parameters.
    ///
    /// Does not include `this` parameters used by some TypeScript functions.
    pub params: Box<'a, FormalParameters<'a>>,
    /// The TypeScript return type annotation.
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    /// The function body.
    ///
    /// [`None`] for function declarations, e.g.
    /// ```ts
    /// // TypeScript function declarations have no body
    /// declare function foo(a: number): number;
    ///
    /// function bar(a: number): number; // <- overloads have no body
    /// function bar(a: number): number {
    ///     return a;
    /// }
    /// ```
    pub body: Option<Box<'a, FunctionBody<'a>>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum FunctionType {
    FunctionDeclaration = 0,
    FunctionExpression = 1,
    TSDeclareFunction = 2,
    /// <https://github.com/typescript-eslint/typescript-eslint/pull/1289>
    TSEmptyBodyFunctionExpression = 3,
}

/// <https://tc39.es/ecma262/#prod-FormalParameters>
// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(custom_serialize)]
pub struct FormalParameters<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub kind: FormalParameterKind,
    #[tsify(type = "Array<FormalParameter | FormalParameterRest>")]
    pub items: Vec<'a, FormalParameter<'a>>,
    #[estree(skip)]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct FormalParameter<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub decorators: Vec<'a, Decorator<'a>>,
    pub pattern: BindingPattern<'a>,
    pub accessibility: Option<TSAccessibility>,
    pub readonly: bool,
    pub r#override: bool,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum FormalParameterKind {
    /// <https://tc39.es/ecma262/#prod-FormalParameters>
    FormalParameter = 0,
    /// <https://tc39.es/ecma262/#prod-UniqueFormalParameters>
    UniqueFormalParameters = 1,
    /// <https://tc39.es/ecma262/#prod-ArrowFormalParameters>
    ArrowFormalParameters = 2,
    /// Part of TypeScript type signatures
    Signature = 3,
}

/// <https://tc39.es/ecma262/#prod-FunctionBody>
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct FunctionBody<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub directives: Vec<'a, Directive<'a>>,
    pub statements: Vec<'a, Statement<'a>>,
}

/// Arrow Function Definitions
#[ast(visit)]
#[scope(
    flags(ScopeFlags::Function | ScopeFlags::Arrow),
    strict_if(self.body.has_use_strict_directive()),
)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ArrowFunctionExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    pub expression: bool,
    pub r#async: bool,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    /// See `expression` for whether this arrow expression returns an expression.
    pub body: Box<'a, FunctionBody<'a>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Generator Function Definitions
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct YieldExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub delegate: bool,
    pub argument: Option<Expression<'a>>,
}

/// Class Definitions
#[ast(visit)]
#[scope(flags(ScopeFlags::StrictMode))]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct Class<'a> {
    pub r#type: ClassType,
    #[estree(flatten)]
    pub span: Span,
    /// Decorators applied to the class.
    ///
    /// Decorators are currently a stage 3 proposal. Oxc handles both TC39 and
    /// legacy TypeScript decorators.
    ///
    /// ## Example
    /// ```ts
    /// @Bar() // <-- Decorator
    /// class Foo {}
    /// ```
    pub decorators: Vec<'a, Decorator<'a>>,
    /// Class identifier, AKA the name
    pub id: Option<BindingIdentifier<'a>>,
    #[scope(enter_before)]
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// Super class. When present, this will usually be an [`IdentifierReference`].
    ///
    /// ## Example
    /// ```ts
    /// class Foo extends Bar {}
    /// //                ^^^
    /// ```
    #[visit(as(ClassHeritage))]
    pub super_class: Option<Expression<'a>>,
    /// Type parameters passed to super class.
    ///
    /// ## Example
    /// ```ts
    /// class Foo<T> extends Bar<T> {}
    /// //                       ^
    /// ```
    pub super_type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    /// Interface implementation clause for TypeScript classes.
    ///
    /// ## Example
    /// ```ts
    /// interface Bar {}
    /// class Foo implements Bar {}
    /// //                   ^^^
    /// ```
    pub implements: Option<Vec<'a, TSClassImplements<'a>>>,
    pub body: Box<'a, ClassBody<'a>>,
    /// Whether the class is abstract
    ///
    /// ## Example
    /// ```ts
    /// class Foo {}          // true
    /// abstract class Bar {} // false
    /// ```
    pub r#abstract: bool,
    /// Whether the class was `declare`ed
    ///
    /// ## Example
    /// ```ts
    /// declare class Foo {}
    /// ```
    pub declare: bool,
    /// Id of the scope created by the [`Class`], including type parameters and
    /// statements within the [`ClassBody`].
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum ClassType {
    /// Class declaration statement
    /// ```ts
    /// class Foo { }
    /// ```
    ClassDeclaration = 0,
    /// Class expression
    ///
    /// ```ts
    /// const Foo = class {}
    /// ```
    ClassExpression = 1,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ClassBody<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub body: Vec<'a, ClassElement<'a>>,
}

/// Class Body Element
///
/// ## Example
/// ```ts
/// class Foo {
///   [prop: string]: string // ClassElement::TSIndexSignature
///
///   public x: number // ClassElement::PropertyDefinition
///
///   accessor z() { return 5 } // ClassElement::AccessorProperty
///
///   // These are all ClassElement::MethodDefinitions
///   get y() { return 5 }
///   set y(value) { }
///   static foo() {}
///   bar() {}
/// }
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ClassElement<'a> {
    StaticBlock(Box<'a, StaticBlock<'a>>) = 0,
    /// Class Methods
    ///
    /// Includes static and non-static methods, constructors, getters, and setters.
    MethodDefinition(Box<'a, MethodDefinition<'a>>) = 1,
    PropertyDefinition(Box<'a, PropertyDefinition<'a>>) = 2,
    AccessorProperty(Box<'a, AccessorProperty<'a>>) = 3,
    /// Index Signature
    ///
    /// ## Example
    /// ```ts
    /// class Foo {
    ///   [keys: string]: string
    /// }
    /// ```
    TSIndexSignature(Box<'a, TSIndexSignature<'a>>) = 4,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct MethodDefinition<'a> {
    /// Method definition type
    ///
    /// This will always be true when an `abstract` modifier is used on the method.
    pub r#type: MethodDefinitionType,
    #[estree(flatten)]
    pub span: Span,
    pub decorators: Vec<'a, Decorator<'a>>,
    pub key: PropertyKey<'a>,
    #[visit(args(flags = match self.kind {
        MethodDefinitionKind::Get => ScopeFlags::Function | ScopeFlags::GetAccessor,
        MethodDefinitionKind::Set => ScopeFlags::Function | ScopeFlags::SetAccessor,
        MethodDefinitionKind::Constructor => ScopeFlags::Function | ScopeFlags::Constructor,
        MethodDefinitionKind::Method => ScopeFlags::Function,
    }))]
    pub value: Box<'a, Function<'a>>, // FunctionExpression
    pub kind: MethodDefinitionKind,
    pub computed: bool,
    pub r#static: bool,
    pub r#override: bool,
    pub optional: bool,
    pub accessibility: Option<TSAccessibility>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum MethodDefinitionType {
    MethodDefinition = 0,
    TSAbstractMethodDefinition = 1,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct PropertyDefinition<'a> {
    pub r#type: PropertyDefinitionType,
    #[estree(flatten)]
    pub span: Span,
    /// Decorators applied to the property.
    ///
    /// See [`Decorator`] for more information.
    pub decorators: Vec<'a, Decorator<'a>>,
    /// The expression used to declare the property.
    pub key: PropertyKey<'a>,
    /// Initialized value in the declaration.
    ///
    /// ## Example
    /// ```
    /// class Foo {
    ///   x = 5     // Some(NumericLiteral)
    ///   y: string // None
    ///
    ///   constructor() {
    ///     this.y = "hello"
    ///   }
    /// }
    /// ```
    pub value: Option<Expression<'a>>,
    /// Property was declared with a computed key
    ///
    /// ## Example
    /// ```ts
    /// class Foo {
    ///   ["a"]: string // true
    ///   b: number     // false
    /// }
    /// ```
    pub computed: bool,
    /// Property was declared with a `static` modifier
    pub r#static: bool,
    /// Property is declared with a `declare` modifier.
    ///
    /// ## Example
    /// ```ts
    /// class Foo {
    ///   x: number         // false
    ///   declare y: string // true
    /// }
    ///
    /// declare class Bar {
    ///   x: number         // false
    /// }
    /// ```
    pub declare: bool,
    pub r#override: bool,
    /// `true` when created with an optional modifier (`?`)
    pub optional: bool,
    pub definite: bool,
    /// `true` when declared with a `readonly` modifier
    pub readonly: bool,
    /// Type annotation on the property.
    ///
    /// Will only ever be [`Some`] for TypeScript files.
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    /// Accessibility modifier.
    ///
    /// Only ever [`Some`] for TypeScript files.
    ///
    /// ## Example
    ///
    /// ```ts
    /// class Foo {
    ///   public w: number     // Some(TSAccessibility::Public)
    ///   private x: string    // Some(TSAccessibility::Private)
    ///   protected y: boolean // Some(TSAccessibility::Protected)
    ///   readonly z           // None
    /// }
    /// ```
    pub accessibility: Option<TSAccessibility>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum PropertyDefinitionType {
    PropertyDefinition = 0,
    TSAbstractPropertyDefinition = 1,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(rename_all = "camelCase")]
pub enum MethodDefinitionKind {
    /// Class constructor
    Constructor = 0,
    /// Static or instance method
    Method = 1,
    /// Getter method
    Get = 2,
    /// Setter method
    Set = 3,
}

/// An identifier for a private class member.
///
/// See: [MDN - Private class fields](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_class_fields)
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct PrivateIdentifier<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
}

/// Class Static Block
///
/// See: [MDN - Static initialization blocks](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Static_initialization_blocks)
///
/// ## Example
///
/// ```ts
/// class Foo {
///     static {
///         this.someStaticProperty = 5;
///     }
/// }
/// ```
#[ast(visit)]
#[scope(flags(ScopeFlags::ClassStaticBlock))]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct StaticBlock<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// ES6 Module Declaration
///
/// An ESM import or export statement.
///
/// ## Example
///
/// ```ts
/// // ImportDeclaration
/// import { foo } from 'foo';
/// import bar from 'bar';
/// import * as baz from 'baz';
///
/// // Not a ModuleDeclaration
/// export const a = 5;
///
/// const b = 6;
///
/// export { b };             // ExportNamedDeclaration
/// export default b;         // ExportDefaultDeclaration
/// export * as c from './c'; // ExportAllDeclaration
/// export = b;               // TSExportAssignment
/// export as namespace d;    // TSNamespaceExportDeclaration
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ModuleDeclaration<'a> {
    /// `import hello from './world.js';`
    /// `import * as t from './world.js';`
    ImportDeclaration(Box<'a, ImportDeclaration<'a>>) = 64,
    /// `export * as numbers from '../numbers.js'`
    ExportAllDeclaration(Box<'a, ExportAllDeclaration<'a>>) = 65,
    /// `export default 5;`
    ExportDefaultDeclaration(Box<'a, ExportDefaultDeclaration<'a>>) = 66,
    /// `export {five} from './numbers.js';`
    /// `export {six, seven};`
    ExportNamedDeclaration(Box<'a, ExportNamedDeclaration<'a>>) = 67,

    /// `export = 5;`
    TSExportAssignment(Box<'a, TSExportAssignment<'a>>) = 68,
    /// `export as namespace React;`
    TSNamespaceExportDeclaration(Box<'a, TSNamespaceExportDeclaration<'a>>) = 69,
}

/// Macro for matching `ModuleDeclaration`'s variants.
#[macro_export]
macro_rules! match_module_declaration {
    ($ty:ident) => {
        $ty::ImportDeclaration(_)
            | $ty::ExportAllDeclaration(_)
            | $ty::ExportDefaultDeclaration(_)
            | $ty::ExportNamedDeclaration(_)
            | $ty::TSExportAssignment(_)
            | $ty::TSNamespaceExportDeclaration(_)
    };
}
pub use match_module_declaration;

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum AccessorPropertyType {
    AccessorProperty = 0,
    TSAbstractAccessorProperty = 1,
}

/// Class Accessor Property
///
/// ## Example
/// ```ts
/// class Foo {
///   accessor y: string
/// }
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct AccessorProperty<'a> {
    pub r#type: AccessorPropertyType,
    #[estree(flatten)]
    pub span: Span,
    /// Decorators applied to the accessor property.
    ///
    /// See [`Decorator`] for more information.
    pub decorators: Vec<'a, Decorator<'a>>,
    /// The expression used to declare the property.
    pub key: PropertyKey<'a>,
    /// Initialized value in the declaration, if present.
    pub value: Option<Expression<'a>>,
    /// Property was declared with a computed key
    pub computed: bool,
    /// Property was declared with a `static` modifier
    pub r#static: bool,
    /// Property has a `!` after its key.
    pub definite: bool,
    /// Type annotation on the property.
    ///
    /// Will only ever be [`Some`] for TypeScript files.
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    /// Accessibility modifier.
    ///
    /// Only ever [`Some`] for TypeScript files.
    ///
    /// ## Example
    ///
    /// ```ts
    /// class Foo {
    ///   public accessor w: number     // Some(TSAccessibility::Public)
    ///   private accessor x: string    // Some(TSAccessibility::Private)
    ///   protected accessor y: boolean // Some(TSAccessibility::Protected)
    ///   accessor z           // None
    /// }
    /// ```
    pub accessibility: Option<TSAccessibility>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ImportExpression<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub source: Expression<'a>,
    pub arguments: Vec<'a, Expression<'a>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ImportDeclaration<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    pub specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
    pub source: StringLiteral<'a>,
    /// Some(vec![]) for empty assertion
    pub with_clause: Option<Box<'a, WithClause<'a>>>,
    /// `import type { foo } from 'bar'`
    pub import_kind: ImportOrExportKind,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ImportDeclarationSpecifier<'a> {
    /// import {imported} from "source"
    /// import {imported as local} from "source"
    ImportSpecifier(Box<'a, ImportSpecifier<'a>>) = 0,
    /// import local from "source"
    ImportDefaultSpecifier(Box<'a, ImportDefaultSpecifier<'a>>) = 1,
    /// import * as local from "source"
    ImportNamespaceSpecifier(Box<'a, ImportNamespaceSpecifier<'a>>) = 2,
}

// import {imported} from "source"
// import {imported as local} from "source"
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ImportSpecifier<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub imported: ModuleExportName<'a>,
    /// The name of the imported symbol.
    ///
    /// ## Example
    /// ```ts
    /// // local and imported name are the same
    /// import { Foo } from 'foo';
    /// //       ^^^
    /// // imports can be renamed, changing the local name
    /// import { Foo as Bar } from 'foo';
    /// //              ^^^
    /// ```
    pub local: BindingIdentifier<'a>,
    pub import_kind: ImportOrExportKind,
}

/// Default Import Specifier
///
/// ## Example
/// ```ts
/// import local from "source";
/// ```
///
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ImportDefaultSpecifier<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// The name of the imported symbol.
    pub local: BindingIdentifier<'a>,
}

/// Namespace import specifier
///
/// ## Example
/// ```ts
/// import * as local from "source";
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ImportNamespaceSpecifier<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub local: BindingIdentifier<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct WithClause<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub attributes_keyword: IdentifierName<'a>, // `with` or `assert`
    pub with_entries: Vec<'a, ImportAttribute<'a>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ImportAttribute<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub key: ImportAttributeKey<'a>,
    pub value: StringLiteral<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ImportAttributeKey<'a> {
    Identifier(IdentifierName<'a>) = 0,
    StringLiteral(StringLiteral<'a>) = 1,
}

/// Named Export Declaration
///
/// ## Example
///
/// ```ts
/// //       ________ specifiers
/// export { Foo, Bar };
/// export type { Baz } from 'baz';
/// //     ^^^^              ^^^^^
/// // export_kind           source
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ExportNamedDeclaration<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub declaration: Option<Declaration<'a>>,
    pub specifiers: Vec<'a, ExportSpecifier<'a>>,
    pub source: Option<StringLiteral<'a>>,
    /// `export type { foo }`
    pub export_kind: ImportOrExportKind,
    /// Some(vec![]) for empty assertion
    pub with_clause: Option<Box<'a, WithClause<'a>>>,
}

/// Export Default Declaration
///
/// ## Example
///
/// ```ts
/// export default HoistableDeclaration
/// export default ClassDeclaration
/// export default AssignmentExpression
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ExportDefaultDeclaration<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub declaration: ExportDefaultDeclarationKind<'a>,
    pub exported: ModuleExportName<'a>, // the `default` Keyword
}

/// Export All Declaration
///
/// ## Example
///
/// ```ts
/// //          _______ exported
/// export * as numbers from '../numbers.js';
/// //                       ^^^^^^^^^^^^^^^ source
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ExportAllDeclaration<'a> {
    #[estree(flatten)]
    pub span: Span,
    /// If this declaration is re-named
    pub exported: Option<ModuleExportName<'a>>,
    pub source: StringLiteral<'a>,
    /// Will be `Some(vec![])` for empty assertion
    pub with_clause: Option<Box<'a, WithClause<'a>>>, // Some(vec![]) for empty assertion
    pub export_kind: ImportOrExportKind, // `export type *`
}

/// Export Specifier
///
/// Each [`ExportSpecifier`] is one of the named exports in an [`ExportNamedDeclaration`].
///
/// ## Example
///
/// ```ts
/// //       ____ export_kind
/// import { type Foo as Bar } from './foo';
/// //   exported ^^^    ^^^ local
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct ExportSpecifier<'a> {
    #[estree(flatten)]
    pub span: Span,
    pub local: ModuleExportName<'a>,
    pub exported: ModuleExportName<'a>,
    pub export_kind: ImportOrExportKind, // `export type *`
}

inherit_variants! {
/// Export Default Declaration Kind
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ExportDefaultDeclarationKind<'a> {
    #[visit(args(flags = ScopeFlags::Function))]
    FunctionDeclaration(Box<'a, Function<'a>>) = 64,
    ClassDeclaration(Box<'a, Class<'a>>) = 65,

    TSInterfaceDeclaration(Box<'a, TSInterfaceDeclaration<'a>>) = 66,

    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// Module Export Name
///
/// Supports:
///   * `import {"\0 any unicode" as foo} from ""`
///   * `export {foo as "\0 any unicode"}`
/// * es2022: <https://github.com/estree/estree/blob/master/es2022.md#modules>
/// * <https://github.com/tc39/ecma262/pull/2154>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
pub enum ModuleExportName<'a> {
    IdentifierName(IdentifierName<'a>) = 0,
    /// For `local` in `ExportSpecifier`: `foo` in `export { foo }`
    IdentifierReference(IdentifierReference<'a>) = 1,
    StringLiteral(StringLiteral<'a>) = 2,
}
