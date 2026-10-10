/// Details for `CloneIn` derive on a struct or enum.
#[derive(Default, Debug)]
pub struct CloneInType {
    /// `true` if should be replaced with default value when cloning
    pub is_default: bool,
    /// `true` if type is a semantic ID (`NodeId`, `ScopeId`, `SymbolId`, `ReferenceId`).
    /// Fields of these types are cloned using `SemanticId` trait.
    pub is_semantic_id: bool,
}

/// Details for `CloneIn` derive on a struct field.
#[derive(Default, Debug)]
pub struct CloneInStructField {
    /// `true` if field should be filled with default value when cloning
    pub is_default: bool,
}
