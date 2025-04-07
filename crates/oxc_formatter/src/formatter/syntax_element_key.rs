use super::TextSize;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SyntaxElementKey {
    offset: TextSize,
}
