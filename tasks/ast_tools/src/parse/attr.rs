use std::fmt::{self, Display};

use bitflags::bitflags;
use syn::MetaList;

use crate::{
    codegen::{DeriveId, GeneratorId},
    schema::{Def, EnumDef, StructDef},
    DERIVES, GENERATORS,
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
        /// Attribute on a struct
        const Struct = 1 << 0;
        /// Attribute on an enum
        const Enum = 1 << 1;
        /// Attribute on a struct field
        const StructField = 1 << 2;
        /// Attribute on an enum variant
        const EnumVariant = 1 << 3;
        /// Part of `#[ast]` attr e.g. `visit` in `#[ast(visit)]`
        const AstAttr = 1 << 4;
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
    /// Part of `#[ast]` attr on a struct
    StructAstAttr(&'d mut StructDef),
    /// Part of `#[ast]` attr on an enum
    EnumAstAttr(&'d mut EnumDef),
}

impl AttrLocation<'_> {
    /// Convert `&mut AttrLocation` to `AttrLocation`.
    pub fn unpack(&mut self) -> AttrLocation {
        match self {
            AttrLocation::Struct(struct_def) => AttrLocation::Struct(struct_def),
            AttrLocation::Enum(enum_def) => AttrLocation::Enum(enum_def),
            AttrLocation::StructField(struct_def, field_index) => {
                AttrLocation::StructField(struct_def, *field_index)
            }
            AttrLocation::EnumVariant(enum_def, variant_index) => {
                AttrLocation::EnumVariant(enum_def, *variant_index)
            }
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
    /// e.g. `#[visit(args(flags = ScopeFlags::Function))]`.
    List(&'p str, &'p MetaList),
}
