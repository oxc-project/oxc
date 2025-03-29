/// Details for `AstBuilder` generator on a struct or enum.
#[derive(Default, Debug)]
pub struct AstBuilderType {
    /// `true` if no builder methods should be created for this struct/enum
    pub skip: bool,
    /// `true` if should be replaced with default value in AST builder methods
    pub is_default: bool,
}

/// Details for `AstBuilder` generator on a struct field.
#[derive(Default, Debug)]
pub struct AstBuilderStructField {
    /// `true` if should be replaced with default value in AST builder methods
    pub is_default: bool,
}
