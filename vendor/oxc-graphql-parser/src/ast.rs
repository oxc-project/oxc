use crate::Error;
use crate::LimitTracker;
use std::fmt;
use std::slice::Iter;

pub use oxc_allocator::{Box as AstBox, Vec as AstVec};

/// A half-open byte range into the source text.
///
/// Offsets are `u32`: source texts are limited to 4 GiB (asserted by
/// [`crate::Parser::new`]), which halves the size of every AST node that
/// carries a span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub struct Ast<'a, T> {
    source: &'a str,
    root: T,
    errors: Vec<Error>,
    comments: Vec<Span>,
    recursion_limit: LimitTracker,
    token_limit: LimitTracker,
}

impl<'a, T> Ast<'a, T> {
    pub(crate) fn new(
        source: &'a str,
        root: T,
        errors: Vec<Error>,
        comments: Vec<Span>,
        recursion_limit: LimitTracker,
        token_limit: LimitTracker,
    ) -> Self {
        Self { source, root, errors, comments, recursion_limit, token_limit }
    }

    pub fn root(&self) -> &T {
        &self.root
    }

    pub fn into_root(self) -> T {
        self.root
    }

    pub fn source(&self) -> &str {
        self.source
    }

    pub fn errors(&self) -> Iter<'_, Error> {
        self.errors.iter()
    }

    /// Comment token spans in document order.
    ///
    /// GraphQL comments are always line comments: each span covers `#` through
    /// the end of the line (excluding the line terminator).
    ///
    /// NOTE: Only comments consumed while parsing are recorded.
    /// [`Parser::parse`] reads to the end of input, so it collects every comment in the source.
    /// Partial roots ([`Parser::parse_selection_set`], [`Parser::parse_type`])
    /// stop at the end of the root, so comments past it are not included.
    ///
    /// [`Parser::parse`]: crate::Parser::parse
    /// [`Parser::parse_selection_set`]: crate::Parser::parse_selection_set
    /// [`Parser::parse_type`]: crate::Parser::parse_type
    pub fn comments(&self) -> &[Span] {
        &self.comments
    }

    pub fn recursion_limit(&self) -> LimitTracker {
        self.recursion_limit
    }

    pub fn token_limit(&self) -> LimitTracker {
        self.token_limit
    }
}

impl<'a> Ast<'a, Document<'a>> {
    pub fn document(&self) -> &Document<'a> {
        self.root()
    }
}

impl<'a> Ast<'a, SelectionSet<'a>> {
    pub fn field_set(&self) -> &SelectionSet<'a> {
        self.root()
    }
}

impl<'a> Ast<'a, Type<'a>> {
    pub fn ty(&self) -> &Type<'a> {
        self.root()
    }
}

#[derive(Debug)]
pub struct Document<'a> {
    pub definitions: AstVec<'a, Definition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub enum Definition<'a> {
    Operation(AstBox<'a, OperationDefinition<'a>>),
    Fragment(AstBox<'a, FragmentDefinition<'a>>),
    Directive(AstBox<'a, DirectiveDefinition<'a>>),
    DirectiveExtension(AstBox<'a, DirectiveExtension<'a>>),
    Schema(AstBox<'a, SchemaDefinition<'a>>),
    SchemaExtension(AstBox<'a, SchemaExtension<'a>>),
    ScalarType(AstBox<'a, ScalarTypeDefinition<'a>>),
    ScalarTypeExtension(AstBox<'a, ScalarTypeExtension<'a>>),
    ObjectType(AstBox<'a, ObjectTypeDefinition<'a>>),
    ObjectTypeExtension(AstBox<'a, ObjectTypeExtension<'a>>),
    InterfaceType(AstBox<'a, InterfaceTypeDefinition<'a>>),
    InterfaceTypeExtension(AstBox<'a, InterfaceTypeExtension<'a>>),
    UnionType(AstBox<'a, UnionTypeDefinition<'a>>),
    UnionTypeExtension(AstBox<'a, UnionTypeExtension<'a>>),
    EnumType(AstBox<'a, EnumTypeDefinition<'a>>),
    EnumTypeExtension(AstBox<'a, EnumTypeExtension<'a>>),
    InputObjectType(AstBox<'a, InputObjectTypeDefinition<'a>>),
    InputObjectTypeExtension(AstBox<'a, InputObjectTypeExtension<'a>>),
}

impl<'a> Definition<'a> {
    pub fn name(&self) -> Option<&Name<'a>> {
        match self {
            Self::Operation(definition) => definition.name.as_ref(),
            Self::Fragment(definition) => Some(&definition.name),
            Self::Directive(definition) => Some(&definition.name),
            Self::DirectiveExtension(definition) => Some(&definition.name),
            Self::Schema(_) | Self::SchemaExtension(_) => None,
            Self::ScalarType(definition) => Some(&definition.name),
            Self::ScalarTypeExtension(definition) => Some(&definition.name),
            Self::ObjectType(definition) => Some(&definition.name),
            Self::ObjectTypeExtension(definition) => Some(&definition.name),
            Self::InterfaceType(definition) => Some(&definition.name),
            Self::InterfaceTypeExtension(definition) => Some(&definition.name),
            Self::UnionType(definition) => Some(&definition.name),
            Self::UnionTypeExtension(definition) => Some(&definition.name),
            Self::EnumType(definition) => Some(&definition.name),
            Self::EnumTypeExtension(definition) => Some(&definition.name),
            Self::InputObjectType(definition) => Some(&definition.name),
            Self::InputObjectTypeExtension(definition) => Some(&definition.name),
        }
    }

    /// The source span of the definition, whichever variant it is.
    ///
    /// When adding a new variant, remember to extend this match as well.
    pub fn span(&self) -> Span {
        match self {
            Self::Operation(definition) => definition.span,
            Self::Fragment(definition) => definition.span,
            Self::Directive(definition) => definition.span,
            Self::DirectiveExtension(definition) => definition.span,
            Self::Schema(definition) => definition.span,
            Self::SchemaExtension(definition) => definition.span,
            Self::ScalarType(definition) => definition.span,
            Self::ScalarTypeExtension(definition) => definition.span,
            Self::ObjectType(definition) => definition.span,
            Self::ObjectTypeExtension(definition) => definition.span,
            Self::InterfaceType(definition) => definition.span,
            Self::InterfaceTypeExtension(definition) => definition.span,
            Self::UnionType(definition) => definition.span,
            Self::UnionTypeExtension(definition) => definition.span,
            Self::EnumType(definition) => definition.span,
            Self::EnumTypeExtension(definition) => definition.span,
            Self::InputObjectType(definition) => definition.span,
            Self::InputObjectTypeExtension(definition) => definition.span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Name<'a> {
    pub value: &'a str,
    pub span: Span,
}

impl Name<'_> {
    pub fn as_str(&self) -> &str {
        self.value
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringValue<'a> {
    pub raw: &'a str,
    pub value: &'a str,
    pub block: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug)]
pub struct OperationDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub operation_type: OperationType,
    pub name: Option<Name<'a>>,
    pub variable_definitions: AstVec<'a, VariableDefinition<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub selection_set: Option<AstBox<'a, SelectionSet<'a>>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct FragmentDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub variable_definitions: AstVec<'a, VariableDefinition<'a>>,
    pub type_condition: NamedType<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub selection_set: Option<AstBox<'a, SelectionSet<'a>>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct SelectionSet<'a> {
    pub selections: AstVec<'a, Selection<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub enum Selection<'a> {
    Field(AstBox<'a, Field<'a>>),
    FragmentSpread(AstBox<'a, FragmentSpread<'a>>),
    InlineFragment(AstBox<'a, InlineFragment<'a>>),
}

impl Selection<'_> {
    /// The source span of the selection, whichever variant it is.
    ///
    /// When adding a new variant, remember to extend this match as well.
    pub fn span(&self) -> Span {
        match self {
            Self::Field(selection) => selection.span,
            Self::FragmentSpread(selection) => selection.span,
            Self::InlineFragment(selection) => selection.span,
        }
    }
}

#[derive(Debug)]
pub struct Field<'a> {
    pub alias: Option<Name<'a>>,
    pub name: Name<'a>,
    pub arguments: AstVec<'a, Argument<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub selection_set: Option<AstBox<'a, SelectionSet<'a>>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct FragmentSpread<'a> {
    pub name: Name<'a>,
    pub arguments: AstVec<'a, Argument<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct InlineFragment<'a> {
    pub type_condition: Option<NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub selection_set: Option<AstBox<'a, SelectionSet<'a>>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct VariableDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub variable: Variable<'a>,
    pub ty: Option<Type<'a>>,
    pub default_value: Option<Value<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variable<'a> {
    pub name: Name<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Argument<'a> {
    pub name: Name<'a>,
    pub value: Option<Value<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Directive<'a> {
    pub name: Name<'a>,
    pub arguments: AstVec<'a, Argument<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub enum Value<'a> {
    Variable(AstBox<'a, Variable<'a>>),
    Int(AstBox<'a, IntValue<'a>>),
    Float(AstBox<'a, FloatValue<'a>>),
    String(AstBox<'a, StringValue<'a>>),
    Boolean(AstBox<'a, BooleanValue>),
    Null(AstBox<'a, NullValue>),
    Enum(AstBox<'a, EnumValue<'a>>),
    List(AstBox<'a, ListValue<'a>>),
    Object(AstBox<'a, ObjectValue<'a>>),
    Missing(Span),
}

impl Value<'_> {
    pub fn span(&self) -> Span {
        match self {
            Self::Variable(value) => value.span,
            Self::Int(value) => value.span,
            Self::Float(value) => value.span,
            Self::String(value) => value.span,
            Self::Boolean(value) => value.span,
            Self::Null(value) => value.span,
            Self::Enum(value) => value.name.span,
            Self::List(value) => value.span,
            Self::Object(value) => value.span,
            Self::Missing(span) => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntValue<'a> {
    pub raw: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FloatValue<'a> {
    pub raw: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BooleanValue {
    pub value: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NullValue {
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumValue<'a> {
    pub name: Name<'a>,
}

#[derive(Debug)]
pub struct ListValue<'a> {
    pub values: AstVec<'a, Value<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ObjectValue<'a> {
    pub fields: AstVec<'a, ObjectField<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ObjectField<'a> {
    pub name: Name<'a>,
    pub value: Option<Value<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub enum Type<'a> {
    Named(AstBox<'a, NamedType<'a>>),
    List(AstBox<'a, ListType<'a>>),
    NonNull(AstBox<'a, NonNullType<'a>>),
    Missing(Span),
}

impl Type<'_> {
    pub fn span(&self) -> Span {
        match self {
            Self::Named(value) => value.name.span,
            Self::List(value) => value.span,
            Self::NonNull(value) => value.span,
            Self::Missing(span) => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NamedType<'a> {
    pub name: Name<'a>,
}

#[derive(Debug)]
pub struct ListType<'a> {
    pub ty: Type<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub struct NonNullType<'a> {
    pub ty: Type<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub struct SchemaDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub root_operations: AstVec<'a, RootOperationTypeDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct SchemaExtension<'a> {
    pub directives: AstVec<'a, Directive<'a>>,
    pub root_operations: AstVec<'a, RootOperationTypeDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RootOperationTypeDefinition<'a> {
    pub operation_type: OperationType,
    pub named_type: NamedType<'a>,
    pub span: Span,
}

#[derive(Debug)]
pub struct DirectiveExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct DirectiveDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub arguments: AstVec<'a, InputValueDefinition<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub repeatable: bool,
    pub locations: AstVec<'a, DirectiveLocation<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectiveLocation<'a> {
    pub name: &'a str,
    pub span: Span,
}

#[derive(Debug)]
pub struct ScalarTypeDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ScalarTypeExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ObjectTypeDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub interfaces: AstVec<'a, NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, FieldDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ObjectTypeExtension<'a> {
    pub name: Name<'a>,
    pub interfaces: AstVec<'a, NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, FieldDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct InterfaceTypeDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub interfaces: AstVec<'a, NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, FieldDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct InterfaceTypeExtension<'a> {
    pub name: Name<'a>,
    pub interfaces: AstVec<'a, NamedType<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, FieldDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct UnionTypeDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub members: AstVec<'a, NamedType<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct UnionTypeExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub members: AstVec<'a, NamedType<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct EnumTypeDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub values: AstVec<'a, EnumValueDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct EnumTypeExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub values: AstVec<'a, EnumValueDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct EnumValueDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub value: EnumValue<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct InputObjectTypeDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, InputValueDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct InputObjectTypeExtension<'a> {
    pub name: Name<'a>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub fields: AstVec<'a, InputValueDefinition<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct FieldDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub arguments: AstVec<'a, InputValueDefinition<'a>>,
    pub ty: Option<Type<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct InputValueDefinition<'a> {
    pub description: Option<AstBox<'a, StringValue<'a>>>,
    pub name: Name<'a>,
    pub ty: Option<Type<'a>>,
    pub default_value: Option<Value<'a>>,
    pub directives: AstVec<'a, Directive<'a>>,
    pub span: Span,
}

/// Compile-time size gates for the memory-sensitive AST nodes, in the style
/// of `oxc_ast`'s `assert_layouts.rs`.
///
/// AST nodes are bulk data: parsers allocate thousands of them per document,
/// so a size regression here is a memory and cache regression everywhere.
/// If an intentional layout change trips these, update the constants.
///
/// PROTOTYPE: This crate is vendored, and these asserts are disabled, because the
/// pointer-compression prototype shrinks `oxc_allocator::Box` from 8 bytes to 4,
/// which changes the size of every type below.
#[cfg(any())] // was: #[cfg(target_pointer_width = "64")]
const _: () = {
    use std::mem::size_of;

    assert!(size_of::<Span>() == 8);
    assert!(size_of::<Name>() == 24);
    assert!(size_of::<StringValue>() == 48);

    assert!(size_of::<Definition>() == 16);
    assert!(size_of::<OperationDefinition>() == 104);
    assert!(size_of::<FragmentDefinition>() == 120);

    assert!(size_of::<SelectionSet>() == 32);
    assert!(size_of::<Selection>() == 16);
    assert!(size_of::<Field>() == 112);
    assert!(size_of::<InlineFragment>() == 64);

    assert!(size_of::<Value>() == 16);
    assert!(size_of::<Argument>() == 48);
    assert!(size_of::<ObjectField>() == 48);
    assert!(size_of::<Directive>() == 56);
    assert!(size_of::<Type>() == 16);

    assert!(size_of::<FieldDefinition>() == 104);
    assert!(size_of::<InputValueDefinition>() == 96);
    assert!(size_of::<VariableDefinition>() == 104);
    assert!(size_of::<EnumValueDefinition>() == 64);
};
