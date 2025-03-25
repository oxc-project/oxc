use super::TextSize;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SyntaxElementKey {
    // node_data: NonNull<()>,
    offset: TextSize,
}
