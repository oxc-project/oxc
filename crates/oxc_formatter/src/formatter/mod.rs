//! Infrastructure for code formatting
//!
//! This module defines [FormatElement], an IR to format code documents and provides a mean to print
//! such a document to a string. Objects that know how to format themselves implement the [Format] trait.
//!
//! ## Formatting Traits
//!
//! * [Format]: Implemented by objects that can be formatted.
//!
//! ## Formatting Macros
//!
//! This crate defines two macros to construct the IR. These are inspired by Rust's `fmt` macros
//! * [`format!`]: Formats a formattable object
//! * [`format_args!`]: Concatenates a sequence of Format objects.
//! * [`write!`]: Writes a sequence of formattable objects into an output buffer.

// FIXME
#![allow(rustdoc::broken_intra_doc_links)]

mod arguments;
pub mod buffer;
mod builders;
pub mod comments;
mod context;
pub mod diagnostics;
pub mod format_element;
mod format_extensions;
pub mod formatter;
pub mod group_id;
pub mod macros;
pub mod parent_stack;
pub mod prelude;
#[cfg(debug_assertions)]
pub mod printed_tokens;
pub mod printer;
pub mod separated;
mod state;
mod syntax_element_key;
mod syntax_node;
mod syntax_token;
mod syntax_trivia_piece_comments;
mod text_len;
mod text_range;
mod text_size;
pub mod token;
mod token_text;
pub mod trivia;
mod verbatim;

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

pub use buffer::{Buffer, BufferExtensions, VecBuffer};
pub use format_element::FormatElement;
pub use group_id::GroupId;
use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::Program};
use oxc_ast_visit::Visit;
use rustc_hash::FxHashMap;

pub use self::comments::{Comments, SourceComment};
use self::printer::Printer;
pub use self::{
    arguments::{Argument, Arguments},
    context::FormatContext,
    diagnostics::{ActualStart, FormatError, InvalidDocumentError, PrintError},
    formatter::Formatter,
    state::{FormatState, FormatStateSnapshot},
    syntax_node::SyntaxNode,
    syntax_token::SyntaxToken,
    syntax_trivia_piece_comments::SyntaxTriviaPieceComments,
    text_len::TextLen,
    text_range::TextRange,
    text_size::TextSize,
    token_text::TokenText,
};
use self::{format_element::document::Document, group_id::UniqueGroupIdBuilder, prelude::TagKind};

#[derive(Debug, Clone)]
pub struct Formatted<'a> {
    document: Document,
    context: FormatContext<'a>,
}

impl<'a> Formatted<'a> {
    pub fn new(document: Document, context: FormatContext<'a>) -> Self {
        Self { document, context }
    }

    /// Returns the context used during formatting.
    pub fn context(&self) -> &FormatContext<'a> {
        &self.context
    }

    /// Returns the formatted document.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Consumes `self` and returns the formatted document.
    pub fn into_document(self) -> Document {
        self.document
    }
}

impl Formatted<'_> {
    pub fn print(&self) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();

        let printed = Printer::new(print_options).print(&self.document)?;

        // let printed = match self.context.source_map() {
        // Some(source_map) => source_map.map_printed(printed),
        // None => printed,
        // };

        Ok(printed)
    }

    pub fn print_with_indent(&self, indent: u16) -> PrintResult<Printed> {
        todo!()
        // let print_options = self.context.options().as_print_options();
        // let printed = Printer::new(print_options).print_with_indent(&self.document, indent)?;

        // let printed = match self.context.source_map() {
        // Some(source_map) => source_map.map_printed(printed),
        // None => printed,
        // };

        // Ok(printed)
    }
}
pub type PrintResult<T> = Result<T, PrintError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Printed {
    code: String,
    range: Option<TextRange>,
    verbatim_ranges: Vec<TextRange>,
}

impl Printed {
    pub fn new(code: String, range: Option<TextRange>, verbatim_source: Vec<TextRange>) -> Self {
        Self { code, range, verbatim_ranges: verbatim_source }
    }

    /// Construct an empty formatter result
    pub fn new_empty() -> Self {
        Self { code: String::new(), range: None, verbatim_ranges: Vec::new() }
    }

    /// Range of the input source file covered by this formatted code,
    /// or None if the entire file is covered in this instance
    pub fn range(&self) -> Option<TextRange> {
        self.range
    }

    /// Access the resulting code, borrowing the result
    pub fn as_code(&self) -> &str {
        &self.code
    }

    /// Access the resulting code, consuming the result
    pub fn into_code(self) -> String {
        self.code
    }

    /// The text in the formatted code that has been formatted as verbatim.
    pub fn verbatim(&self) -> impl Iterator<Item = (TextRange, &str)> {
        panic!();
        std::iter::empty()
        // self.verbatim_ranges.iter().map(|range| (*range, &self.code[*range]))
    }

    /// Ranges of the formatted code that have been formatted as verbatim.
    pub fn verbatim_ranges(&self) -> &[TextRange] {
        &self.verbatim_ranges
    }

    /// Takes the ranges of nodes that have been formatted as verbatim, replacing them with an empty list.
    pub fn take_verbatim_ranges(&mut self) -> Vec<TextRange> {
        std::mem::take(&mut self.verbatim_ranges)
    }
}

// Public return type of the formatter
pub type FormatResult<F> = Result<F, FormatError>;

/// Formatting trait for types that can create a formatted representation. The `biome_formatter` equivalent
/// to [std::fmt::Display].
///
/// ## Example
/// Implementing `Format` for a custom struct
///
/// ```
/// use biome_formatter::{format, write, IndentStyle, LineWidth};
/// use biome_formatter::prelude::*;
/// use biome_rowan::TextSize;
///
/// struct Paragraph(String);
///
/// impl Format<SimpleFormatContext> for Paragraph {
///     fn fmt(&self, f: &mut Formatter<SimpleFormatContext>) -> FormatResult<()> {
///         write!(f, [
///             hard_line_break(),
///             dynamic_text(&self.0, TextSize::from(0)),
///             hard_line_break(),
///         ])
///     }
/// }
///
/// # fn main() -> FormatResult<()> {
/// let paragraph = Paragraph(String::from("test"));
/// let formatted = format!(SimpleFormatContext::default(), [paragraph])?;
///
/// assert_eq!("test\n", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
pub trait Format<'ast> {
    /// Formats the object using the given formatter.
    /// # Errors
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()>;
}

impl<'ast, T> Format<'ast> for &T
where
    T: ?Sized + Format<'ast>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        Format::fmt(&**self, f)
    }
}

impl<'ast, T> Format<'ast> for &mut T
where
    T: ?Sized + Format<'ast>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        Format::fmt(&**self, f)
    }
}

impl<'ast, T> Format<'ast> for Option<T>
where
    T: Format<'ast>,
{
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        match self {
            Some(value) => value.fmt(f),
            None => Ok(()),
        }
    }
}

impl Format<'_> for () {
    #[inline]
    fn fmt(&self, _: &mut Formatter) -> FormatResult<()> {
        // Intentionally left empty
        Ok(())
    }
}

impl Format<'_> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        crate::write!(f, builders::text(self))
    }
}

/// Default implementation for formatting a token
pub struct FormatToken<C> {
    context: PhantomData<C>,
}

impl<C> Default for FormatToken<C> {
    fn default() -> Self {
        Self { context: PhantomData }
    }
}

/// The `write` function takes a target buffer and an `Arguments` struct that can be precompiled with the `format_args!` macro.
///
/// The arguments will be formatted in-order into the output buffer provided.
///
/// # Examples
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{VecBuffer, format_args, FormatState, write, Formatted};
///
/// # fn main() -> FormatResult<()> {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// write!(&mut buffer, [format_args!(text("Hello World"))])?;
///
/// let formatted = Formatted::new(Document::from(buffer.into_vec()), SimpleFormatContext::default());
///
/// assert_eq!("Hello World", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
/// Please note that using [`write!`] might be preferable. Example:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{VecBuffer, format_args, FormatState, write, Formatted};
///
/// # fn main() -> FormatResult<()> {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// write!(&mut buffer, [text("Hello World")])?;
///
/// let formatted = Formatted::new(Document::from(buffer.into_vec()), SimpleFormatContext::default());
///
/// assert_eq!("Hello World", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
#[inline(always)]
pub fn write<'ast>(output: &mut dyn Buffer<'ast>, args: Arguments<'_, 'ast>) -> FormatResult<()> {
    Formatter::new(output).write_fmt(args)
}

/// The `format` function takes an [`Arguments`] struct and returns the resulting formatting IR.
///
/// The [`Arguments`] instance can be created with the [`format_args!`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format, format_args};
///
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(SimpleFormatContext::default(), [&format_args!(text("test"))])?;
/// assert_eq!("test", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
/// Please note that using [`format!`] might be preferable. Example:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format};
///
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(SimpleFormatContext::default(), [text("test")])?;
/// assert_eq!("test", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
pub fn format<'ast>(
    program: &'ast Program<'ast>,
    context: FormatContext<'ast>,
    arguments: Arguments<'_, 'ast>,
) -> FormatResult<Formatted<'ast>> {
    let (parents, kinds) = build_parents_and_kinds(program);
    let mut state = FormatState::new(program, parents, kinds, context);
    let mut buffer = VecBuffer::with_capacity(arguments.items().len(), &mut state);

    buffer.write_fmt(arguments)?;

    let mut document = Document::from(buffer.into_vec());
    document.propagate_expand();

    Ok(Formatted::new(document, state.into_context()))
}

fn build_parents_and_kinds<'a>(
    program: &'a Program<'a>,
) -> (FxHashMap<Address, Address>, FxHashMap<Address, AstKind<'a>>) {
    struct Starter<'a> {
        program: &'a Program<'a>,
        parents: FxHashMap<Address, Address>,
        kinds: FxHashMap<Address, AstKind<'a>>,
        current_address: Address,
    };
    impl<'a> Visit<'a> for Starter<'a> {
        fn enter_node(&mut self, kind: AstKind<'a>) {
            let address = match kind {
                AstKind::Program(a) => Address::from_ptr(a),
                AstKind::IdentifierName(a) => Address::from_ptr(a),
                AstKind::IdentifierReference(a) => Address::from_ptr(a),
                AstKind::BindingIdentifier(a) => Address::from_ptr(a),
                AstKind::LabelIdentifier(a) => Address::from_ptr(a),
                AstKind::ThisExpression(a) => Address::from_ptr(a),
                AstKind::ArrayExpression(a) => Address::from_ptr(a),
                AstKind::ArrayExpressionElement(a) => Address::from_ptr(a),
                AstKind::Elision(a) => Address::from_ptr(a),
                AstKind::ObjectExpression(a) => Address::from_ptr(a),
                AstKind::ObjectProperty(a) => Address::from_ptr(a),
                AstKind::PropertyKey(a) => Address::from_ptr(a),
                AstKind::TemplateLiteral(a) => Address::from_ptr(a),
                AstKind::TaggedTemplateExpression(a) => Address::from_ptr(a),
                AstKind::MemberExpression(a) => Address::from_ptr(a),
                AstKind::CallExpression(a) => Address::from_ptr(a),
                AstKind::NewExpression(a) => Address::from_ptr(a),
                AstKind::MetaProperty(a) => Address::from_ptr(a),
                AstKind::SpreadElement(a) => Address::from_ptr(a),
                AstKind::Argument(a) => Address::from_ptr(a),
                AstKind::UpdateExpression(a) => Address::from_ptr(a),
                AstKind::UnaryExpression(a) => Address::from_ptr(a),
                AstKind::BinaryExpression(a) => Address::from_ptr(a),
                AstKind::PrivateInExpression(a) => Address::from_ptr(a),
                AstKind::LogicalExpression(a) => Address::from_ptr(a),
                AstKind::ConditionalExpression(a) => Address::from_ptr(a),
                AstKind::AssignmentExpression(a) => Address::from_ptr(a),
                AstKind::AssignmentTarget(a) => Address::from_ptr(a),
                AstKind::SimpleAssignmentTarget(a) => Address::from_ptr(a),
                AstKind::AssignmentTargetPattern(a) => Address::from_ptr(a),
                AstKind::ArrayAssignmentTarget(a) => Address::from_ptr(a),
                AstKind::ObjectAssignmentTarget(a) => Address::from_ptr(a),
                AstKind::AssignmentTargetWithDefault(a) => Address::from_ptr(a),
                AstKind::SequenceExpression(a) => Address::from_ptr(a),
                AstKind::Super(a) => Address::from_ptr(a),
                AstKind::AwaitExpression(a) => Address::from_ptr(a),
                AstKind::ChainExpression(a) => Address::from_ptr(a),
                AstKind::ParenthesizedExpression(a) => {
                    // Don't need to call `Expression::without_parentheses` if we never have parentheses
                    return;
                }
                AstKind::Directive(a) => Address::from_ptr(a),
                AstKind::Hashbang(a) => Address::from_ptr(a),
                AstKind::BlockStatement(a) => Address::from_ptr(a),
                AstKind::VariableDeclaration(a) => Address::from_ptr(a),
                AstKind::VariableDeclarator(a) => Address::from_ptr(a),
                AstKind::EmptyStatement(a) => Address::from_ptr(a),
                AstKind::ExpressionStatement(a) => Address::from_ptr(a),
                AstKind::IfStatement(a) => Address::from_ptr(a),
                AstKind::DoWhileStatement(a) => Address::from_ptr(a),
                AstKind::WhileStatement(a) => Address::from_ptr(a),
                AstKind::ForStatement(a) => Address::from_ptr(a),
                AstKind::ForStatementInit(a) => Address::from_ptr(a),
                AstKind::ForInStatement(a) => Address::from_ptr(a),
                AstKind::ForOfStatement(a) => Address::from_ptr(a),
                AstKind::ContinueStatement(a) => Address::from_ptr(a),
                AstKind::BreakStatement(a) => Address::from_ptr(a),
                AstKind::ReturnStatement(a) => Address::from_ptr(a),
                AstKind::WithStatement(a) => Address::from_ptr(a),
                AstKind::SwitchStatement(a) => Address::from_ptr(a),
                AstKind::SwitchCase(a) => Address::from_ptr(a),
                AstKind::LabeledStatement(a) => Address::from_ptr(a),
                AstKind::ThrowStatement(a) => Address::from_ptr(a),
                AstKind::TryStatement(a) => Address::from_ptr(a),
                AstKind::CatchClause(a) => Address::from_ptr(a),
                AstKind::CatchParameter(a) => Address::from_ptr(a),
                AstKind::DebuggerStatement(a) => Address::from_ptr(a),
                AstKind::AssignmentPattern(a) => Address::from_ptr(a),
                AstKind::ObjectPattern(a) => Address::from_ptr(a),
                AstKind::ArrayPattern(a) => Address::from_ptr(a),
                AstKind::BindingRestElement(a) => Address::from_ptr(a),
                AstKind::Function(a) => Address::from_ptr(a),
                AstKind::FormalParameters(a) => Address::from_ptr(a),
                AstKind::FormalParameter(a) => Address::from_ptr(a),
                AstKind::FunctionBody(a) => Address::from_ptr(a),
                AstKind::ArrowFunctionExpression(a) => Address::from_ptr(a),
                AstKind::YieldExpression(a) => Address::from_ptr(a),
                AstKind::Class(a) => Address::from_ptr(a),
                AstKind::ClassBody(a) => Address::from_ptr(a),
                AstKind::MethodDefinition(a) => Address::from_ptr(a),
                AstKind::PropertyDefinition(a) => Address::from_ptr(a),
                AstKind::PrivateIdentifier(a) => Address::from_ptr(a),
                AstKind::StaticBlock(a) => Address::from_ptr(a),
                AstKind::ModuleDeclaration(a) => Address::from_ptr(a),
                AstKind::ImportExpression(a) => Address::from_ptr(a),
                AstKind::ImportDeclaration(a) => Address::from_ptr(a),
                AstKind::ImportSpecifier(a) => Address::from_ptr(a),
                AstKind::ImportDefaultSpecifier(a) => Address::from_ptr(a),
                AstKind::ImportNamespaceSpecifier(a) => Address::from_ptr(a),
                AstKind::ExportNamedDeclaration(a) => Address::from_ptr(a),
                AstKind::ExportDefaultDeclaration(a) => Address::from_ptr(a),
                AstKind::ExportAllDeclaration(a) => Address::from_ptr(a),
                AstKind::ExportSpecifier(a) => Address::from_ptr(a),
                AstKind::V8IntrinsicExpression(a) => Address::from_ptr(a),
                AstKind::BooleanLiteral(a) => Address::from_ptr(a),
                AstKind::NullLiteral(a) => Address::from_ptr(a),
                AstKind::NumericLiteral(a) => Address::from_ptr(a),
                AstKind::StringLiteral(a) => Address::from_ptr(a),
                AstKind::BigIntLiteral(a) => Address::from_ptr(a),
                AstKind::RegExpLiteral(a) => Address::from_ptr(a),
                AstKind::JSXElement(a) => Address::from_ptr(a),
                AstKind::JSXOpeningElement(a) => Address::from_ptr(a),
                AstKind::JSXClosingElement(a) => Address::from_ptr(a),
                AstKind::JSXFragment(a) => Address::from_ptr(a),
                AstKind::JSXElementName(a) => Address::from_ptr(a),
                AstKind::JSXNamespacedName(a) => Address::from_ptr(a),
                AstKind::JSXMemberExpression(a) => Address::from_ptr(a),
                AstKind::JSXMemberExpressionObject(a) => Address::from_ptr(a),
                AstKind::JSXExpressionContainer(a) => Address::from_ptr(a),
                AstKind::JSXAttributeItem(a) => Address::from_ptr(a),
                AstKind::JSXSpreadAttribute(a) => Address::from_ptr(a),
                AstKind::JSXIdentifier(a) => Address::from_ptr(a),
                AstKind::JSXText(a) => Address::from_ptr(a),
                AstKind::TSThisParameter(a) => Address::from_ptr(a),
                AstKind::TSEnumDeclaration(a) => Address::from_ptr(a),
                AstKind::TSEnumBody(a) => Address::from_ptr(a),
                AstKind::TSEnumMember(a) => Address::from_ptr(a),
                AstKind::TSTypeAnnotation(a) => Address::from_ptr(a),
                AstKind::TSLiteralType(a) => Address::from_ptr(a),
                AstKind::TSConditionalType(a) => Address::from_ptr(a),
                AstKind::TSUnionType(a) => Address::from_ptr(a),
                AstKind::TSIntersectionType(a) => Address::from_ptr(a),
                AstKind::TSParenthesizedType(a) => Address::from_ptr(a),
                AstKind::TSIndexedAccessType(a) => Address::from_ptr(a),
                AstKind::TSNamedTupleMember(a) => Address::from_ptr(a),
                AstKind::TSAnyKeyword(a) => Address::from_ptr(a),
                AstKind::TSStringKeyword(a) => Address::from_ptr(a),
                AstKind::TSBooleanKeyword(a) => Address::from_ptr(a),
                AstKind::TSNumberKeyword(a) => Address::from_ptr(a),
                AstKind::TSNeverKeyword(a) => Address::from_ptr(a),
                AstKind::TSIntrinsicKeyword(a) => Address::from_ptr(a),
                AstKind::TSUnknownKeyword(a) => Address::from_ptr(a),
                AstKind::TSNullKeyword(a) => Address::from_ptr(a),
                AstKind::TSUndefinedKeyword(a) => Address::from_ptr(a),
                AstKind::TSVoidKeyword(a) => Address::from_ptr(a),
                AstKind::TSSymbolKeyword(a) => Address::from_ptr(a),
                AstKind::TSThisType(a) => Address::from_ptr(a),
                AstKind::TSObjectKeyword(a) => Address::from_ptr(a),
                AstKind::TSBigIntKeyword(a) => Address::from_ptr(a),
                AstKind::TSTypeReference(a) => Address::from_ptr(a),
                AstKind::TSTypeName(a) => Address::from_ptr(a),
                AstKind::TSQualifiedName(a) => Address::from_ptr(a),
                AstKind::TSTypeParameterInstantiation(a) => Address::from_ptr(a),
                AstKind::TSTypeParameter(a) => Address::from_ptr(a),
                AstKind::TSTypeParameterDeclaration(a) => Address::from_ptr(a),
                AstKind::TSTypeAliasDeclaration(a) => Address::from_ptr(a),
                AstKind::TSClassImplements(a) => Address::from_ptr(a),
                AstKind::TSInterfaceDeclaration(a) => Address::from_ptr(a),
                AstKind::TSPropertySignature(a) => Address::from_ptr(a),
                AstKind::TSMethodSignature(a) => Address::from_ptr(a),
                AstKind::TSConstructSignatureDeclaration(a) => Address::from_ptr(a),
                AstKind::TSInterfaceHeritage(a) => Address::from_ptr(a),
                AstKind::TSModuleDeclaration(a) => Address::from_ptr(a),
                AstKind::TSModuleBlock(a) => Address::from_ptr(a),
                AstKind::TSTypeLiteral(a) => Address::from_ptr(a),
                AstKind::TSInferType(a) => Address::from_ptr(a),
                AstKind::TSTypeQuery(a) => Address::from_ptr(a),
                AstKind::TSImportType(a) => Address::from_ptr(a),
                AstKind::TSMappedType(a) => Address::from_ptr(a),
                AstKind::TSTemplateLiteralType(a) => Address::from_ptr(a),
                AstKind::TSAsExpression(a) => Address::from_ptr(a),
                AstKind::TSSatisfiesExpression(a) => Address::from_ptr(a),
                AstKind::TSTypeAssertion(a) => Address::from_ptr(a),
                AstKind::TSImportEqualsDeclaration(a) => Address::from_ptr(a),
                AstKind::TSModuleReference(a) => Address::from_ptr(a),
                AstKind::TSExternalModuleReference(a) => Address::from_ptr(a),
                AstKind::TSNonNullExpression(a) => Address::from_ptr(a),
                AstKind::Decorator(a) => Address::from_ptr(a),
                AstKind::TSExportAssignment(a) => Address::from_ptr(a),
                AstKind::TSInstantiationExpression(a) => Address::from_ptr(a),
            };

            if !self.parents.contains_key(&address) {
                self.parents.insert(address, self.current_address);
            }
            if !self.kinds.contains_key(&address) {
                self.kinds.insert(address, kind);
            }

            self.current_address = address;
        }

        fn leave_node(&mut self, kind: AstKind<'a>) {
            let address = match kind {
                AstKind::Program(a) => Address::from_ptr(a),
                AstKind::IdentifierName(a) => Address::from_ptr(a),
                AstKind::IdentifierReference(a) => Address::from_ptr(a),
                AstKind::BindingIdentifier(a) => Address::from_ptr(a),
                AstKind::LabelIdentifier(a) => Address::from_ptr(a),
                AstKind::ThisExpression(a) => Address::from_ptr(a),
                AstKind::ArrayExpression(a) => Address::from_ptr(a),
                AstKind::ArrayExpressionElement(a) => Address::from_ptr(a),
                AstKind::Elision(a) => Address::from_ptr(a),
                AstKind::ObjectExpression(a) => Address::from_ptr(a),
                AstKind::ObjectProperty(a) => Address::from_ptr(a),
                AstKind::PropertyKey(a) => Address::from_ptr(a),
                AstKind::TemplateLiteral(a) => Address::from_ptr(a),
                AstKind::TaggedTemplateExpression(a) => Address::from_ptr(a),
                AstKind::MemberExpression(a) => Address::from_ptr(a),
                AstKind::CallExpression(a) => Address::from_ptr(a),
                AstKind::NewExpression(a) => Address::from_ptr(a),
                AstKind::MetaProperty(a) => Address::from_ptr(a),
                AstKind::SpreadElement(a) => Address::from_ptr(a),
                AstKind::Argument(a) => Address::from_ptr(a),
                AstKind::UpdateExpression(a) => Address::from_ptr(a),
                AstKind::UnaryExpression(a) => Address::from_ptr(a),
                AstKind::BinaryExpression(a) => Address::from_ptr(a),
                AstKind::PrivateInExpression(a) => Address::from_ptr(a),
                AstKind::LogicalExpression(a) => Address::from_ptr(a),
                AstKind::ConditionalExpression(a) => Address::from_ptr(a),
                AstKind::AssignmentExpression(a) => Address::from_ptr(a),
                AstKind::AssignmentTarget(a) => Address::from_ptr(a),
                AstKind::SimpleAssignmentTarget(a) => Address::from_ptr(a),
                AstKind::AssignmentTargetPattern(a) => Address::from_ptr(a),
                AstKind::ArrayAssignmentTarget(a) => Address::from_ptr(a),
                AstKind::ObjectAssignmentTarget(a) => Address::from_ptr(a),
                AstKind::AssignmentTargetWithDefault(a) => Address::from_ptr(a),
                AstKind::SequenceExpression(a) => Address::from_ptr(a),
                AstKind::Super(a) => Address::from_ptr(a),
                AstKind::AwaitExpression(a) => Address::from_ptr(a),
                AstKind::ChainExpression(a) => Address::from_ptr(a),
                AstKind::ParenthesizedExpression(a) => {
                    //
                    return;
                }
                AstKind::Directive(a) => Address::from_ptr(a),
                AstKind::Hashbang(a) => Address::from_ptr(a),
                AstKind::BlockStatement(a) => Address::from_ptr(a),
                AstKind::VariableDeclaration(a) => Address::from_ptr(a),
                AstKind::VariableDeclarator(a) => Address::from_ptr(a),
                AstKind::EmptyStatement(a) => Address::from_ptr(a),
                AstKind::ExpressionStatement(a) => Address::from_ptr(a),
                AstKind::IfStatement(a) => Address::from_ptr(a),
                AstKind::DoWhileStatement(a) => Address::from_ptr(a),
                AstKind::WhileStatement(a) => Address::from_ptr(a),
                AstKind::ForStatement(a) => Address::from_ptr(a),
                AstKind::ForStatementInit(a) => Address::from_ptr(a),
                AstKind::ForInStatement(a) => Address::from_ptr(a),
                AstKind::ForOfStatement(a) => Address::from_ptr(a),
                AstKind::ContinueStatement(a) => Address::from_ptr(a),
                AstKind::BreakStatement(a) => Address::from_ptr(a),
                AstKind::ReturnStatement(a) => Address::from_ptr(a),
                AstKind::WithStatement(a) => Address::from_ptr(a),
                AstKind::SwitchStatement(a) => Address::from_ptr(a),
                AstKind::SwitchCase(a) => Address::from_ptr(a),
                AstKind::LabeledStatement(a) => Address::from_ptr(a),
                AstKind::ThrowStatement(a) => Address::from_ptr(a),
                AstKind::TryStatement(a) => Address::from_ptr(a),
                AstKind::CatchClause(a) => Address::from_ptr(a),
                AstKind::CatchParameter(a) => Address::from_ptr(a),
                AstKind::DebuggerStatement(a) => Address::from_ptr(a),
                AstKind::AssignmentPattern(a) => Address::from_ptr(a),
                AstKind::ObjectPattern(a) => Address::from_ptr(a),
                AstKind::ArrayPattern(a) => Address::from_ptr(a),
                AstKind::BindingRestElement(a) => Address::from_ptr(a),
                AstKind::Function(a) => Address::from_ptr(a),
                AstKind::FormalParameters(a) => Address::from_ptr(a),
                AstKind::FormalParameter(a) => Address::from_ptr(a),
                AstKind::FunctionBody(a) => Address::from_ptr(a),
                AstKind::ArrowFunctionExpression(a) => Address::from_ptr(a),
                AstKind::YieldExpression(a) => Address::from_ptr(a),
                AstKind::Class(a) => Address::from_ptr(a),
                AstKind::ClassBody(a) => Address::from_ptr(a),
                AstKind::MethodDefinition(a) => Address::from_ptr(a),
                AstKind::PropertyDefinition(a) => Address::from_ptr(a),
                AstKind::PrivateIdentifier(a) => Address::from_ptr(a),
                AstKind::StaticBlock(a) => Address::from_ptr(a),
                AstKind::ModuleDeclaration(a) => Address::from_ptr(a),
                AstKind::ImportExpression(a) => Address::from_ptr(a),
                AstKind::ImportDeclaration(a) => Address::from_ptr(a),
                AstKind::ImportSpecifier(a) => Address::from_ptr(a),
                AstKind::ImportDefaultSpecifier(a) => Address::from_ptr(a),
                AstKind::ImportNamespaceSpecifier(a) => Address::from_ptr(a),
                AstKind::ExportNamedDeclaration(a) => Address::from_ptr(a),
                AstKind::ExportDefaultDeclaration(a) => Address::from_ptr(a),
                AstKind::ExportAllDeclaration(a) => Address::from_ptr(a),
                AstKind::ExportSpecifier(a) => Address::from_ptr(a),
                AstKind::V8IntrinsicExpression(a) => Address::from_ptr(a),
                AstKind::BooleanLiteral(a) => Address::from_ptr(a),
                AstKind::NullLiteral(a) => Address::from_ptr(a),
                AstKind::NumericLiteral(a) => Address::from_ptr(a),
                AstKind::StringLiteral(a) => Address::from_ptr(a),
                AstKind::BigIntLiteral(a) => Address::from_ptr(a),
                AstKind::RegExpLiteral(a) => Address::from_ptr(a),
                AstKind::JSXElement(a) => Address::from_ptr(a),
                AstKind::JSXOpeningElement(a) => Address::from_ptr(a),
                AstKind::JSXClosingElement(a) => Address::from_ptr(a),
                AstKind::JSXFragment(a) => Address::from_ptr(a),
                AstKind::JSXElementName(a) => Address::from_ptr(a),
                AstKind::JSXNamespacedName(a) => Address::from_ptr(a),
                AstKind::JSXMemberExpression(a) => Address::from_ptr(a),
                AstKind::JSXMemberExpressionObject(a) => Address::from_ptr(a),
                AstKind::JSXExpressionContainer(a) => Address::from_ptr(a),
                AstKind::JSXAttributeItem(a) => Address::from_ptr(a),
                AstKind::JSXSpreadAttribute(a) => Address::from_ptr(a),
                AstKind::JSXIdentifier(a) => Address::from_ptr(a),
                AstKind::JSXText(a) => Address::from_ptr(a),
                AstKind::TSThisParameter(a) => Address::from_ptr(a),
                AstKind::TSEnumDeclaration(a) => Address::from_ptr(a),
                AstKind::TSEnumBody(a) => Address::from_ptr(a),
                AstKind::TSEnumMember(a) => Address::from_ptr(a),
                AstKind::TSTypeAnnotation(a) => Address::from_ptr(a),
                AstKind::TSLiteralType(a) => Address::from_ptr(a),
                AstKind::TSConditionalType(a) => Address::from_ptr(a),
                AstKind::TSUnionType(a) => Address::from_ptr(a),
                AstKind::TSIntersectionType(a) => Address::from_ptr(a),
                AstKind::TSParenthesizedType(a) => Address::from_ptr(a),
                AstKind::TSIndexedAccessType(a) => Address::from_ptr(a),
                AstKind::TSNamedTupleMember(a) => Address::from_ptr(a),
                AstKind::TSAnyKeyword(a) => Address::from_ptr(a),
                AstKind::TSStringKeyword(a) => Address::from_ptr(a),
                AstKind::TSBooleanKeyword(a) => Address::from_ptr(a),
                AstKind::TSNumberKeyword(a) => Address::from_ptr(a),
                AstKind::TSNeverKeyword(a) => Address::from_ptr(a),
                AstKind::TSIntrinsicKeyword(a) => Address::from_ptr(a),
                AstKind::TSUnknownKeyword(a) => Address::from_ptr(a),
                AstKind::TSNullKeyword(a) => Address::from_ptr(a),
                AstKind::TSUndefinedKeyword(a) => Address::from_ptr(a),
                AstKind::TSVoidKeyword(a) => Address::from_ptr(a),
                AstKind::TSSymbolKeyword(a) => Address::from_ptr(a),
                AstKind::TSThisType(a) => Address::from_ptr(a),
                AstKind::TSObjectKeyword(a) => Address::from_ptr(a),
                AstKind::TSBigIntKeyword(a) => Address::from_ptr(a),
                AstKind::TSTypeReference(a) => Address::from_ptr(a),
                AstKind::TSTypeName(a) => Address::from_ptr(a),
                AstKind::TSQualifiedName(a) => Address::from_ptr(a),
                AstKind::TSTypeParameterInstantiation(a) => Address::from_ptr(a),
                AstKind::TSTypeParameter(a) => Address::from_ptr(a),
                AstKind::TSTypeParameterDeclaration(a) => Address::from_ptr(a),
                AstKind::TSTypeAliasDeclaration(a) => Address::from_ptr(a),
                AstKind::TSClassImplements(a) => Address::from_ptr(a),
                AstKind::TSInterfaceDeclaration(a) => Address::from_ptr(a),
                AstKind::TSPropertySignature(a) => Address::from_ptr(a),
                AstKind::TSMethodSignature(a) => Address::from_ptr(a),
                AstKind::TSConstructSignatureDeclaration(a) => Address::from_ptr(a),
                AstKind::TSInterfaceHeritage(a) => Address::from_ptr(a),
                AstKind::TSModuleDeclaration(a) => Address::from_ptr(a),
                AstKind::TSModuleBlock(a) => Address::from_ptr(a),
                AstKind::TSTypeLiteral(a) => Address::from_ptr(a),
                AstKind::TSInferType(a) => Address::from_ptr(a),
                AstKind::TSTypeQuery(a) => Address::from_ptr(a),
                AstKind::TSImportType(a) => Address::from_ptr(a),
                AstKind::TSMappedType(a) => Address::from_ptr(a),
                AstKind::TSTemplateLiteralType(a) => Address::from_ptr(a),
                AstKind::TSAsExpression(a) => Address::from_ptr(a),
                AstKind::TSSatisfiesExpression(a) => Address::from_ptr(a),
                AstKind::TSTypeAssertion(a) => Address::from_ptr(a),
                AstKind::TSImportEqualsDeclaration(a) => Address::from_ptr(a),
                AstKind::TSModuleReference(a) => Address::from_ptr(a),
                AstKind::TSExternalModuleReference(a) => Address::from_ptr(a),
                AstKind::TSNonNullExpression(a) => Address::from_ptr(a),
                AstKind::Decorator(a) => Address::from_ptr(a),
                AstKind::TSExportAssignment(a) => Address::from_ptr(a),
                AstKind::TSInstantiationExpression(a) => Address::from_ptr(a),
            };

            self.current_address = self.parents.get(&address).cloned().unwrap_or(Address::DUMMY);
        }
    }

    impl<'a> Starter<'a> {
        fn new(program: &'a Program<'a>) -> Self {
            Self {
                program,
                parents: FxHashMap::default(),
                kinds: FxHashMap::default(),
                current_address: Address::DUMMY,
            }
        }
    }

    let mut starter = Starter::new(program);
    starter.visit_program(program);

    (starter.parents, starter.kinds)
}
