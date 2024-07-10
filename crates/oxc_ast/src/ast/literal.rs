//! Literals

// NB: `#[span]`, `#[scope(...)]`, `#[visit(...)]`, `#[visit_as(...)]` and `#[visit_args(...)]` do
// not do anything to the code, They are purely markers for codegen used in
// `tasts/ast_codegen` and `crates/oxc_traverse/scripts`. See docs in that crate.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::hash::Hash;

use bitflags::bitflags;
use oxc_ast_macros::visited_node;
use oxc_span::{Atom, Span};
use oxc_syntax::number::{BigintBase, NumberBase};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

#[visited_node]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct BooleanLiteral {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub value: bool,
}

#[visited_node]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct NullLiteral {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct NumericLiteral<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub value: f64,
    pub raw: &'a str,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub base: NumberBase,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct BigIntLiteral<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub raw: Atom<'a>,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub base: BigintBase,
}

#[visited_node]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct RegExpLiteral<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    // valid regex is printed as {}
    // invalid regex is printed as null, which we can't implement yet
    pub value: EmptyObject,
    pub regex: RegExp<'a>,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct RegExp<'a> {
    pub pattern: Atom<'a>,
    pub flags: RegExpFlags,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct EmptyObject;

#[visited_node]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct StringLiteral<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub value: Atom<'a>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RegExpFlags: u8 {
        const G = 1 << 0;
        const I = 1 << 1;
        const M = 1 << 2;
        const S = 1 << 3;
        const U = 1 << 4;
        const Y = 1 << 5;
        const D = 1 << 6;
        /// v flag from `https://github.com/tc39/proposal-regexp-set-notation`
        const V = 1 << 7;
    }
}

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type RegExpFlags = {
    G: 1,
    I: 2,
    M: 4,
    S: 8,
    U: 16,
    Y: 32,
    D: 64,
    V: 128
};
"#;
