use crate::{CoreFormatOptions, FormatContext};

/// Simple format context useful for testing.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct SimpleFormatContext<'src> {
    options: CoreFormatOptions,
    source_code: &'src str,
    tailwind_classes: Vec<String>,
}

impl<'src> SimpleFormatContext<'src> {
    #[must_use]
    pub fn with_source_code(mut self, code: &'src str) -> Self {
        self.source_code = code;
        self
    }

    /// Set the collected sorted Tailwind CSS classes used when rendering
    /// `FormatElement::TailwindClass` entries.
    pub fn set_tailwind_classes(&mut self, classes: Vec<String>) {
        self.tailwind_classes = classes;
    }
}

impl FormatContext for SimpleFormatContext<'_> {
    type Options = CoreFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        self.source_code
    }

    fn get_tailwind_class(&self, idx: usize) -> Option<&str> {
        self.tailwind_classes.get(idx).map(String::as_str)
    }
}
