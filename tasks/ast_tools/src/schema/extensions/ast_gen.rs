/// Random AST generation settings shared by structs, enums, fields, and variants.
#[derive(Default, Debug)]
pub struct AstGen {
    /// Handwritten generator function which replaces structural generation.
    pub with: Option<String>,
    /// Relative selection weight for an enum variant.
    pub weight: Option<u32>,
}
