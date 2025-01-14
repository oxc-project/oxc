//! Common code for JavaScript Syntax
#![warn(missing_docs)]
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
