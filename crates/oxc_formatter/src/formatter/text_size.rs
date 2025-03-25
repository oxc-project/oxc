use std::num::TryFromIntError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextSize {
    pub(crate) raw: u32,
}

impl From<u32> for TextSize {
    #[inline]
    fn from(raw: u32) -> Self {
        TextSize { raw }
    }
}

impl From<TextSize> for u32 {
    #[inline]
    fn from(value: TextSize) -> Self {
        value.raw
    }
}

impl TryFrom<usize> for TextSize {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(value: usize) -> Result<Self, TryFromIntError> {
        Ok(u32::try_from(value)?.into())
    }
}

impl From<TextSize> for usize {
    #[inline]
    fn from(value: TextSize) -> Self {
        value.raw as usize
    }
}
