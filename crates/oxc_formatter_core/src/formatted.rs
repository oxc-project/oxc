use crate::{Document, FormatContext, FormatOptions, PrintResult, Printed, Printer};

#[derive(Debug)]
pub struct Formatted<'a, C> {
    document: Document<'a>,
    context: C,
}

impl<'a, C> Formatted<'a, C> {
    pub fn new(document: Document<'a>, context: C) -> Self {
        Self { document, context }
    }

    /// Returns the context used during formatting.
    pub fn context(&self) -> &C {
        &self.context
    }

    /// Returns the formatted document.
    pub fn document(&self) -> &Document<'a> {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut Document<'a> {
        &mut self.document
    }

    /// Consumes `self` and returns the formatted document.
    pub fn into_document(self) -> Document<'a> {
        self.document
    }
}

impl<C: FormatContext> Formatted<'_, C> {
    /// Prints the formatted document to a string.
    ///
    /// # Errors
    /// Returns `PrintError` if the document contains invalid structure.
    pub fn print(self) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();
        let (elements, sorted_tailwind_classes) =
            self.document.into_elements_and_tailwind_classes();
        let printed = Printer::new(print_options, &sorted_tailwind_classes).print(elements)?;
        Ok(printed)
    }

    /// Prints the formatted document to a string, starting at the given indentation level.
    ///
    /// # Errors
    /// Returns `PrintError` if the document contains invalid structure.
    pub fn print_with_indent(self, indent: u16) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();
        let (elements, sorted_tailwind_classes) =
            self.document.into_elements_and_tailwind_classes();
        let printed = Printer::new(print_options, &sorted_tailwind_classes)
            .print_with_indent(elements, indent)?;
        Ok(printed)
    }
}
