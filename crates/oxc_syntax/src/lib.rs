//! Common code for JavaScript Syntax
#![warn(missing_docs)]

use std::num::NonZeroU32;

use oxc_ast_macros::ast;

pub mod class;
pub mod es_target;
pub mod identifier;
pub mod keyword;
pub mod module_record;
pub mod node;
pub mod number;
pub mod operator;
pub mod precedence;
pub mod reference;
pub mod scope;
pub mod symbol;
pub mod xml_entities;
mod generated {
    mod derive_clone_in;
    mod derive_content_eq;
    #[cfg(feature = "serialize")]
    mod derive_estree;
}

/// Dummy type to communicate the content of `nonmax::NonMaxU32` to `oxc_ast_tools`.
#[ast(foreign = NonMaxU32)]
#[expect(dead_code)]
struct NonMaxU32Alias(NonZeroU32);
