pub mod ast;
mod cell;
mod orphan;

#[allow(unused_imports)] // just for now
pub use orphan::Orphan;

pub use cell::{SharedBox, SharedVec};

/// This trait is only for checking that we didn't forgot to add `ast_node` attribute to any
/// of essential types, Would get removed after cleaning things up.
pub trait TraversableTest {
    fn does_support_traversable() {
        // Yes, Yes it does my friend!
    }
}
