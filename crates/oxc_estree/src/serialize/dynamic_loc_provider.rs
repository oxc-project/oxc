use super::structs::LocProvider;

/// A dynamic location provider that wraps any type implementing the offset-to-location conversion.
/// This allows us to use translation tables without circular dependencies.
pub struct DynamicLocProvider<F>
where
    F: Fn(u32) -> Option<(u32, u32)>,
{
    converter: F,
}

impl<F> DynamicLocProvider<F>
where
    F: Fn(u32) -> Option<(u32, u32)>,
{
    pub fn new(converter: F) -> Self {
        Self { converter }
    }
}

impl<F> LocProvider for DynamicLocProvider<F>
where
    F: Fn(u32) -> Option<(u32, u32)>,
{
    fn offset_to_line_column(&self, utf8_offset: u32) -> Option<(u32, u32)> {
        (self.converter)(utf8_offset)
    }
}
