//! Tests for the AST builders in `oxc_ast::builder::builders`.
//!
//! Not exhaustive over AST types - one test per distinct piece of the API, on whichever
//! node exercises it most directly.

use std::cell::Cell;

use oxc_allocator::{Allocator, ArenaBox, ArenaVec, GetAllocator};
use oxc_ast::{
    ast::{
        Expression, FormalParameterKind, FunctionType, IdentifierReference, ImportAttribute,
        ImportAttributeKey, NumberBase, Statement, StringLiteral, UnaryExpression, UnaryOperator,
    },
    builder::{AstBuild, AstBuilder, GetAstBuilder, builders::traits::*},
};
use oxc_span::{SPAN, Span};
use oxc_str::{Ident, Str};
use oxc_syntax::{node::NodeId, reference::ReferenceId, scope::ScopeId};

// ---------------------------------------------------------------------------------------
// The basics
// ---------------------------------------------------------------------------------------

/// `build` -> setters -> `finish` yields a [`ArenaBox`] of the node.
#[test]
fn build_finish_returns_boxed_node() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let ident: ArenaBox<IdentifierReference> =
        IdentifierReference::build(&builder).span(Span::new(3, 6)).name("foo").defaults().finish();

    assert_eq!(ident.span, Span::new(3, 6));
    assert_eq!(ident.name.as_str(), "foo");
}

/// `span_start` and `span_end` write the two halves separately.
#[test]
fn span_halves_set_separately() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let ident = IdentifierReference::build(&builder)
        .span_start(3)
        .name("foo")
        .span_end(6)
        .defaults()
        .finish();

    assert_eq!(ident.span, Span::new(3, 6));
}

/// `span` writes both halves at once, for a node whose full extent is known in one go.
#[test]
fn span_set_as_one() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let ident =
        IdentifierReference::build(&builder).span(Span::new(3, 6)).name("foo").defaults().finish();

    assert_eq!(ident.span, Span::new(3, 6));
}

/// Fields can be set in any order.
#[test]
fn fields_set_out_of_order() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let ident = IdentifierReference::build(&builder)
        .defaults()
        .name("foo")
        .span_end(6)
        .span_start(3)
        .finish();

    assert_eq!(ident.span, Span::new(3, 6));
    assert_eq!(ident.name.as_str(), "foo");
}

/// An `Ident` field's setter takes `impl Into<Ident>`, so a `&str` works directly -
/// and so does an `Ident` itself, via the blanket `impl From<T> for T`.
#[test]
fn ident_setter_takes_impl_into() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    // A `&'static str`
    let a = IdentifierReference::build(&builder).span(SPAN).name("foo").defaults().finish();
    // A `&'a str` allocated in the arena
    let str = allocator.alloc_str("bar");
    let b = IdentifierReference::build(&builder).span(SPAN).name(str).defaults().finish();
    // An `Ident` built up front
    let ident = Ident::from_str_in("qux", &builder);
    let c = IdentifierReference::build(&builder).span(SPAN).name(ident).defaults().finish();

    assert_eq!(a.name.as_str(), "foo");
    assert_eq!(b.name.as_str(), "bar");
    assert_eq!(c.name.as_str(), "qux");
}

/// Likewise a `Str` field's setter takes `impl Into<Str>`.
#[test]
fn str_setter_takes_impl_into() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    // A `&str`
    let a = StringLiteral::build(&builder).span(SPAN).value("foo").raw(None).defaults().finish();
    // `Str`s built up front
    let value = Str::from_str_in("bar", &builder);
    let raw = Str::from_str_in("'bar'", &builder);
    let b =
        StringLiteral::build(&builder).span(SPAN).value(value).raw(Some(raw)).defaults().finish();

    assert_eq!(a.value.as_str(), "foo");
    assert_eq!(b.value.as_str(), "bar");
    assert_eq!(b.raw.unwrap().as_str(), "'bar'");
}

// ---------------------------------------------------------------------------------------
// `defaults`
// ---------------------------------------------------------------------------------------

/// `defaults` fills in the fields which have a default value (the semantic IDs).
#[test]
fn defaults_sets_unset_default_fields() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let ident = IdentifierReference::build(&builder).span(SPAN).name("foo").defaults().finish();

    assert_eq!(ident.reference_id.get(), None);
}

/// A default field which has been set already is left alone by `defaults`.
#[test]
fn defaults_skips_fields_already_set() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let reference_id = ReferenceId::new(42);
    let ident = IdentifierReference::build(&builder)
        .span(SPAN)
        .name("foo")
        .reference_id(reference_id)
        .defaults()
        .finish();

    assert_eq!(ident.reference_id.get(), Some(reference_id));
}

/// `defaults` can be called before the field it would set.
/// The state, not the call order, decides - so the explicit value still wins.
#[test]
fn defaults_before_explicit_set() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let reference_id = ReferenceId::new(7);
    let ident = IdentifierReference::build(&builder)
        .span(SPAN)
        .name("foo")
        .defaults()
        .reference_id(reference_id)
        .finish();

    assert_eq!(ident.reference_id.get(), Some(reference_id));
}

// ---------------------------------------------------------------------------------------
// Building fields in place
// ---------------------------------------------------------------------------------------

/// `<field>_with` hands the callee a [`Slot`] for the field. `into_<variant>` narrows it
/// to one for an enum variant's payload, `into_contents` to one for a [`Box`]'s contents,
/// and `Slot::build` starts a builder writing straight into it.
#[test]
#[expect(clippy::float_cmp, reason = "value is written and read back unchanged")]
fn field_with_enum_narrowing_box_narrowing_and_slot_build() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let unary = UnaryExpression::build(&builder)
        .span(SPAN)
        .operator(UnaryOperator::UnaryNegation)
        .argument_with(|slot| {
            // `Expression::NumericLiteral` is a boxed variant, so it takes 2 narrowings
            slot.into_numeric_literal()
                .into_contents(&builder)
                .build(&builder)
                .span(SPAN)
                .value(1.0)
                .raw(None)
                .base(NumberBase::Decimal)
                .finish()
        })
        .finish();

    let Expression::NumericLiteral(num) = &unary.argument else { panic!("wrong variant") };
    assert_eq!(num.value, 1.0);
    assert_eq!(num.base, NumberBase::Decimal);
}

/// `build_<variant>` is sugar for the 3 steps above -
/// narrow to the variant, narrow through the [`Box`], and start a builder.
#[test]
#[expect(clippy::float_cmp, reason = "value is written and read back unchanged")]
fn build_variant_boxed_shortcut() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let unary = UnaryExpression::build(&builder)
        .span(SPAN)
        .operator(UnaryOperator::UnaryNegation)
        .argument_with(|slot| {
            slot.build_numeric_literal(&builder)
                .span(SPAN)
                .value(1.0)
                .raw(None)
                .base(NumberBase::Decimal)
                .finish()
        })
        .finish();

    let Expression::NumericLiteral(num) = &unary.argument else { panic!("wrong variant") };
    assert_eq!(num.value, 1.0);
    assert_eq!(num.base, NumberBase::Decimal);
}

/// `<field>_with` for a variant whose payload is *not* boxed skips the `Box` narrowing (`into_contents`).
#[test]
fn field_with_enum_narrowing_and_slot_build() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let attr = ImportAttribute::build(&builder)
        .span(SPAN)
        .key_with(|slot| slot.into_identifier().build(&builder).span(SPAN).name("type").finish())
        .value_with(|slot| {
            slot.build(&builder).span(SPAN).value("json").raw(None).defaults().finish()
        })
        .finish();

    let ImportAttributeKey::Identifier(key) = &attr.key else { panic!("wrong variant") };
    assert_eq!(key.name.as_str(), "type");
}

/// `build_<variant>` for a variant whose payload is *not* boxed still works.
/// It skips the `Box` narrowing (`into_contents`) internally.
#[test]
fn build_variant_unboxed_shortcut() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    // `ImportAttributeKey::Identifier` holds an `IdentifierName` inline, not a `Box`
    let attr = ImportAttribute::build(&builder)
        .span(SPAN)
        .key_with(|slot| slot.build_identifier(&builder).span(SPAN).name("type").finish())
        .value_with(|slot| {
            slot.build(&builder).span(SPAN).value("json").raw(None).defaults().finish()
        })
        .finish();

    let ImportAttributeKey::Identifier(key) = &attr.key else { panic!("wrong variant") };
    assert_eq!(key.name.as_str(), "type");
    assert_eq!(attr.value.value.as_str(), "json");
}

/// A field holding an enum can also be set by value, with the whole enum.
#[test]
#[expect(clippy::float_cmp, reason = "value is written and read back unchanged")]
fn enum_field_set_by_value() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let argument = Expression::new_numeric_literal(SPAN, 2.0, None, NumberBase::Decimal, &builder);
    let unary = UnaryExpression::build(&builder)
        .span(SPAN)
        .operator(UnaryOperator::UnaryNegation)
        .argument(argument)
        .finish();

    let Expression::NumericLiteral(num) = &unary.argument else { panic!("wrong variant") };
    assert_eq!(num.value, 2.0);
}

/// `into_some` narrows a `Slot<Option<T>>` to a `Slot<T>`, leaving the field holding `Some`.
#[test]
fn slot_into_some() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let func = function!(
        &builder,
        .id_with(|slot| slot.into_some().build(&builder).span(SPAN).name("f").defaults().finish())
        .body(None)
    );

    assert_eq!(func.id.as_ref().unwrap().name.as_str(), "f");
}

/// Writing `None` fills the `Option` slot itself, without narrowing.
#[test]
fn slot_filled_with_none() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let func = function!(&builder, .id_with(|slot| slot.fill(None)).body(None));

    assert!(func.id.is_none());
}

/// `into_contents` reserves arena memory for the `T`, writes the [`Box`] into the field,
/// and hands back a [`Slot`] for the `T`. Reached here through an `Option`, so 2 narrowings,
/// with one token at the end standing for both.
#[test]
fn slot_into_contents_through_option() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let func = function!(
        &builder,
        .id(None)
        .body_with(|slot| {
            slot.into_some()
                .into_contents(&builder)
                .build(&builder)
                .span(SPAN)
                .directives(ArenaVec::new_in(&builder))
                .statements(ArenaVec::new_in(&builder))
                .finish()
        })
    );

    assert!(func.body.as_ref().unwrap().statements.is_empty());
}

/// A `Vec` field is set by value, from a [`ArenaVec`] built in the arena.
#[test]
fn vec_field() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let mut statements = ArenaVec::new_in(&builder);
    statements.push(Statement::new_empty_statement(SPAN, &builder));

    let func = function!(
        &builder,
        .id(None)
        .body_with(|slot| {
            slot.into_some()
                .into_contents(&builder)
                .build(&builder)
                .span(SPAN)
                .directives(ArenaVec::new_in(&builder))
                .statements(statements)
                .finish()
        })
    );

    assert_eq!(func.body.as_ref().unwrap().statements.len(), 1);
}

// ---------------------------------------------------------------------------------------
// `with`
// ---------------------------------------------------------------------------------------

/// `with` retargets a builder onto a [`Slot`], so a caller can set some fields, hand the
/// builder off for the rest, and get the finished node back. Fields already set stay set.
#[test]
fn with_retargets_builder() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let ident = IdentifierReference::build(&builder)
        .span_start(3)
        // Handed off with only `span_start` set
        .with(|inner| inner.name("foo").span_end(6).defaults().finish());

    assert_eq!(ident.span, Span::new(3, 6));
    assert_eq!(ident.name.as_str(), "foo");
}

// ---------------------------------------------------------------------------------------
// `uninit`
// ---------------------------------------------------------------------------------------

/// `uninit` reserves the node's memory without initializing it. `fill` writes a whole value.
#[test]
fn uninit_then_fill() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let uninit = IdentifierReference::uninit(&builder);
    let value = IdentifierReference::new(Span::new(3, 6), "foo", &builder);
    let ident = uninit.fill(value);

    assert_eq!(ident.span, Span::new(3, 6));
    assert_eq!(ident.name.as_str(), "foo");
}

/// `fill_with` hands a [`Slot`] for the reserved memory to a builder.
#[test]
fn uninit_then_fill_with() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let ident = IdentifierReference::uninit(&builder).fill_with(|slot| {
        slot.build(&builder).span(Span::new(3, 6)).name("foo").defaults().finish()
    });

    assert_eq!(ident.span, Span::new(3, 6));
    assert_eq!(ident.name.as_str(), "foo");
}

// ---------------------------------------------------------------------------------------
// `new` / `boxed`, which build a node from all its fields at once
// ---------------------------------------------------------------------------------------

/// `boxed` is `new` plus an arena allocation.
#[test]
fn new_and_boxed() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let owned = IdentifierReference::new(Span::new(3, 6), "foo", &builder);
    let boxed = IdentifierReference::boxed(Span::new(3, 6), "foo", &builder);

    assert_eq!(owned.name.as_str(), "foo");
    assert_eq!(boxed.name.as_str(), "foo");
    // Default fields are filled in for you
    assert_eq!(owned.reference_id.get(), None);
}

/// A node with default fields also gets `new_with_<fields>` / `boxed_with_<fields>`,
/// which take those fields rather than defaulting them.
#[test]
fn new_with_default_fields() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let reference_id = ReferenceId::new(42);
    let owned = IdentifierReference::new_with_reference_id(SPAN, "foo", reference_id, &builder);
    let boxed = IdentifierReference::boxed_with_reference_id(SPAN, "foo", reference_id, &builder);

    assert_eq!(owned.reference_id.get(), Some(reference_id));
    assert_eq!(boxed.reference_id.get(), Some(reference_id));
}

// ---------------------------------------------------------------------------------------
// `node_id`, and building through a `GetAstBuilder`
// ---------------------------------------------------------------------------------------

/// An [`AstBuild`]er which hands out increasing [`NodeId`]s.
struct CountingBuilder<'a> {
    allocator: &'a Allocator,
    next: Cell<usize>,
}

impl<'a> AstBuild<'a> for CountingBuilder<'a> {
    fn node_id(&self) -> NodeId {
        let id = self.next.get();
        self.next.set(id + 1);
        NodeId::new(id)
    }
}

impl<'a> GetAstBuilder<'a> for CountingBuilder<'a> {
    type Builder = Self;
    fn builder(&self) -> &Self {
        self
    }
}

impl<'a> GetAllocator<'a> for CountingBuilder<'a> {
    fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
}

/// A type which *holds* a builder rather than being one - like a parser or traversal context.
struct Ctx<'a> {
    builder: CountingBuilder<'a>,
}

impl<'a> GetAstBuilder<'a> for Ctx<'a> {
    type Builder = CountingBuilder<'a>;
    fn builder(&self) -> &CountingBuilder<'a> {
        &self.builder
    }
}

/// `build` writes `node_id` itself - it is the one field with no setter. Nested nodes each get
/// their own, and building through a [`GetAstBuilder`] which is not itself the builder works.
#[test]
fn node_id_is_written_by_build() {
    let allocator = Allocator::new();
    let ctx = Ctx { builder: CountingBuilder { allocator: &allocator, next: Cell::new(0) } };

    let unary = UnaryExpression::build(&ctx)
        .span(SPAN)
        .operator(UnaryOperator::UnaryNegation)
        .argument_with(|slot| {
            slot.build_numeric_literal(&ctx)
                .span(SPAN)
                .value(1.0)
                .raw(None)
                .base(NumberBase::Decimal)
                .finish()
        })
        .finish();

    // The outer node's builder was created first, so it took id 0, and the inner one id 1
    assert_eq!(unary.node_id.get(), NodeId::new(0));
    let Expression::NumericLiteral(num) = &unary.argument else { panic!("wrong variant") };
    assert_eq!(num.node_id.get(), NodeId::new(1));
}

/// The default [`AstBuilder`] assigns [`NodeId::DUMMY`].
#[test]
fn ast_builder_assigns_dummy_node_id() {
    let allocator = Allocator::new();
    let builder = AstBuilder::new(&allocator);

    let ident = IdentifierReference::build(&builder).span(SPAN).name("foo").defaults().finish();

    assert_eq!(ident.node_id.get(), NodeId::DUMMY);
}

// ---------------------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------------------

/// Build a `Function`, with the setters passed in filling `id` and `body`.
///
/// `Function` has 15 field states - the most of any AST type - so this also covers a builder
/// which uses every slot of `FieldsState`.
///
/// A macro rather than a function because a function would have to name the builder's `State`
/// at each point in the chain, and code generic over `State` is not supported.
macro_rules! function {
    ($builder:expr, $($setters:tt)*) => {{
        let builder = $builder;
        oxc_ast::ast::Function::build(builder)
            .span(SPAN)
            .r#type(FunctionType::FunctionDeclaration)
            .generator(false)
            .r#async(false)
            .declare(false)
            .type_parameters(None)
            .this_param(None)
            .params_with(|slot| {
                slot.into_contents(builder)
                    .build(builder)
                    .span(SPAN)
                    .kind(FormalParameterKind::FormalParameter)
                    .items(ArenaVec::new_in(builder))
                    .rest(None)
                    .finish()
            })
            .return_type(None)
            .scope_id(ScopeId::new(0))
            .pure(false)
            .pife(false)
            $($setters)*
            .defaults()
            .finish()
    }};
}
use function;
