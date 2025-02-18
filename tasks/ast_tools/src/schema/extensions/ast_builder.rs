/// Details for `AstBuilder` generator on a struct or enum.
#[derive(Default, Debug)]
pub struct AstBuilderType {
    /// `true` if should be replaced with default value in AST builder methods
    pub is_default: bool,
}
