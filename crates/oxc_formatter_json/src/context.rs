use oxc_allocator::Allocator;
use oxc_formatter_core::FormatContext;

use crate::JsonFormatOptions;

#[derive(Clone, Copy)]
pub struct JsonFormatContext<'ast> {
    options: JsonFormatOptions,
    allocator: &'ast Allocator,
}

impl<'ast> JsonFormatContext<'ast> {
    pub fn new(allocator: &'ast Allocator, options: JsonFormatOptions) -> Self {
        Self { options, allocator }
    }

    pub fn options(self) -> JsonFormatOptions {
        self.options
    }
}

impl<'ast> FormatContext<'ast> for JsonFormatContext<'ast> {
    type Options = JsonFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn allocator(&self) -> &'ast Allocator {
        self.allocator
    }
}
