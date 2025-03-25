use super::TextSize;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TextRange {
    // Invariant: start <= end
    start: TextSize,
    end: TextSize,
}
