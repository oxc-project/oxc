use super::TextSize;

pub trait TextLen: Copy /*+ Sealed*/ {
    /// The textual length of this primitive.
    fn text_len(self) -> TextSize;
}

impl TextLen for &'_ str {
    #[inline]
    fn text_len(self) -> TextSize {
        TextSize::try_from(self.len()).unwrap()
    }
}

impl TextLen for &'_ String {
    #[inline]
    fn text_len(self) -> TextSize {
        self.as_str().text_len()
    }
}
