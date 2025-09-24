use std::fmt::{self, Display};

use bitflags::bitflags;

use crate::{
    DERIVES, GENERATORS, Result,
    codegen::{DeriveId, GeneratorId},
    schema::{Def, EnumDef, MetaType, StructDef},
};

/// Processor of an attribute - either a derive or a generator.
#[derive(Clone, Copy, Debug)]
pub enum AttrProcessor {
    Derive(DeriveId),
    Generator(GeneratorId),
}

impl AttrProcessor {
    /// Get name of this [`AttrProcessor`].
    pub fn name(self) -> &'static str {
        match self {
            Self::Derive(id) => DERIVES[id].trait_name(),
            Self::Generator(id) => GENERATORS[id].name(),
        }
    }
}

bitflags! {
    /// Positions in which an attribute is legal.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AttrPositions: u8 {
        /// Attribute on a struct which derives the trait
        const Struct = 1 << 0;
        /// Attribute on a struct which doesn't derive the trait
        const StructNotDerived = 1 << 1;
        /// Attribute on an enum which derives the trait
        const Enum = 1 << 2;
        /// Attribute on an enum which doesn't derive the trait
        const EnumNotDerived = 1 << 3;
        /// Attribute on a struct field
        const StructField = 1 << 4;
        /// Attribute on an enum variant
        const EnumVariant = 1 << 5;
        /// Attribute on a meta type
        const Meta = 1 << 6;
        /// Part of `#[ast]` attr e.g. `visit` in `#[ast(visit)]`
        const AstAttr = 1 << 7;

        /// Attribute on a struct which may or may not derive the trait
        const StructMaybeDerived = Self::Struct.bits() | Self::StructNotDerived.bits();
        /// Attribute on an enum which may or may not derive the trait
        const EnumMaybeDerived = Self::Enum.bits() | Self::EnumNotDerived.bits();
    }
}

/// Macro to combine multiple `AttrPositions` as a const.
///
/// `attr_positions!(Struct | Enum)` is equivalent to `AttrPositions::Struct | AttrPositions::Enum`,
/// except it evaluates in const context.
///
/// Useful for `Derive::attrs` and `Generator::attrs` methods, where a const is required.
macro_rules! attr_positions {
    ($($positions:ident)|+) => {
        const {
            use $crate::parse::attr::AttrPositions;
            AttrPositions::empty() $(.union(AttrPositions::$positions))+
        }
    }
}
pub(crate) use attr_positions;

/// Attribute location.
pub enum AttrLocation<'d> {
    /// Attribute on a struct
    Struct(&'d mut StructDef),
    /// Attribute on an enum
    Enum(&'d mut EnumDef),
    /// Attribute on a struct field.
    /// Comprises [`StructDef`] and field index.
    StructField(&'d mut StructDef, usize),
    /// Attribute on an enum variant.
    /// Comprises [`EnumDef`]` and variant index.
    EnumVariant(&'d mut EnumDef, usize),
    /// Attribute on a meta type
    Meta(&'d mut MetaType),
    /// Part of `#[ast]` attr on a struct
    StructAstAttr(&'d mut StructDef),
    /// Part of `#[ast]` attr on an enum
    EnumAstAttr(&'d mut EnumDef),
}

impl AttrLocation<'_> {
    /// Convert `&mut AttrLocation` to `AttrLocation`.
    pub fn unpack(&mut self) -> AttrLocation<'_> {
        match self {
            AttrLocation::Struct(struct_def) => AttrLocation::Struct(struct_def),
            AttrLocation::Enum(enum_def) => AttrLocation::Enum(enum_def),
            AttrLocation::StructField(struct_def, field_index) => {
                AttrLocation::StructField(struct_def, *field_index)
            }
            AttrLocation::EnumVariant(enum_def, variant_index) => {
                AttrLocation::EnumVariant(enum_def, *variant_index)
            }
            AttrLocation::Meta(meta) => AttrLocation::Meta(meta),
            AttrLocation::StructAstAttr(struct_def) => AttrLocation::StructAstAttr(struct_def),
            AttrLocation::EnumAstAttr(enum_def) => AttrLocation::EnumAstAttr(enum_def),
        }
    }
}

impl Display for AttrLocation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AttrLocation::Struct(struct_def) | AttrLocation::StructAstAttr(struct_def) => {
                f.write_str(struct_def.name())
            }
            AttrLocation::Enum(enum_def) | AttrLocation::EnumAstAttr(enum_def) => {
                f.write_str(enum_def.name())
            }
            AttrLocation::StructField(struct_def, field_index) => {
                write!(f, "{}::{}", struct_def.name(), struct_def.fields[*field_index].name())
            }
            AttrLocation::EnumVariant(enum_def, variant_index) => {
                write!(f, "{}::{}", enum_def.name(), enum_def.variants[*variant_index].name())
            }
            AttrLocation::Meta(meta) => f.write_str(meta.name()),
        }
    }
}

/// Part of an attribute.
///
/// e.g.:
///
/// * `#[span]` translates to a single `AttrPart::None`.
/// * `#[estree(skip)]` translates to a single `AttrPart::Tag("skip")`.
/// * `#[estree(skip, rename = "Foo", ts_type = "Foo | null")]` translates to 3 `AttrPart`s:
///   * `AttrPart::Tag("skip")`
///   * `AttrPart::String("rename", "Foo")`
///   * `AttrPart::String("ts_type", "Foo | null")`
#[derive(Debug)]
pub enum AttrPart<'p> {
    /// No parts in attribute.
    /// e.g. `#[ts]`.
    None,
    /// Named part.
    /// e.g. `#[estree(skip)]`.
    Tag(&'p str),
    /// String part.
    /// e.g. `#[estree(rename = "Foo")]` or `#[estree(via = crate::serialize::OptionVecDefault)]`.
    String(&'p str, String),
    /// List part.
    /// This `Vec` is never empty.
    /// e.g. `#[visit(args(flags = ScopeFlags::Function))]`.
    List(&'p str, Vec<AttrPartListElement>),
}

/// An element of an [`AttrPart::List`].
#[derive(Debug)]
pub enum AttrPartListElement {
    /// Named part.
    /// e.g. `qux` in `#[foo(bar(qux))]`.
    Tag(String),
    /// String part.
    /// e.g. `flags = ScopeFlags::Function` in `#[visit(args(flags = ScopeFlags::Function))]`.
    String(String, String),
    /// List part.
    /// e.g. `qux(doof, bing = donk)` in `#[foo(bar(qux(doof, bing = donk)))]`.
    List(String, Vec<AttrPartListElement>),
}

impl AttrPartListElement {
    /// Unwrap this [`AttrPartListElement`] if it is an [`AttrPartListElement::Tag`].
    pub fn try_into_tag(self) -> Result<String> {
        if let Self::Tag(name) = self { Ok(name) } else { Err(()) }
    }

    /// Unwrap this [`AttrPartListElement`] if it is an [`AttrPartListElement::String`].
    pub fn try_into_string(self) -> Result<(String, String)> {
        if let Self::String(name, value) = self { Ok((name, value)) } else { Err(()) }
    }

    /// Unwrap this [`AttrPartListElement`] if it is an [`AttrPartListElement::List`].
    #[expect(dead_code)]
    pub fn try_into_list(self) -> Result<(String, Vec<AttrPartListElement>)> {
        if let Self::List(name, elements) = self { Ok((name, elements)) } else { Err(()) }
    }
}
