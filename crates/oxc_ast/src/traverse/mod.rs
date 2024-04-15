/// This trait is only for checking that we didn't forgot to add `ast_node` attribute to any
/// essential types, Would get removed after cleaning things up.
pub trait TraversableTest {
    fn does_support_traversable() {
        // Yes, Yes it does my friend!
    }
}
