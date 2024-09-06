//! Common code for JavaScript Syntax

pub mod class;
pub mod identifier;
pub mod keyword;
pub mod module_graph_visitor;
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
    mod derive_content_hash;
}
