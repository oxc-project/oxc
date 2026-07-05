/// Coarse classification of a type annotation, mirroring the cases HIR type
/// lowering distinguishes (array / primitive / everything else).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RawTypeCategory {
    Array,
    Primitive,
    #[default]
    Other,
}
