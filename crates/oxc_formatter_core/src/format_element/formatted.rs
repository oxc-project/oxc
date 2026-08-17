use crate::{Document, FormatContext, FormatOptions, PrintResult, Printed};

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

    /// Consumes `self` and returns the finalized document:
    /// group expansion is propagated across the whole IR, including any embedded IR merged into it.
    ///
    /// Finalization must run once on the final document, never on partial embedded IR;
    /// [`Self::print`] / [`Self::print_with_indent`] finalize themselves,
    /// so this is only for consumers that bypass them (e.g. IR to Prettier Doc conversion).
    pub fn into_final_document(self) -> Document<'a> {
        self.document.propagate_expand();
        self.document
    }
}

impl<C: FormatContext> Formatted<'_, C> {
    /// Prints the formatted document to a string.
    ///
    /// Finalizes the document first (see [`Self::into_final_document`]).
    ///
    /// # Errors
    /// Returns `PrintError` if the document contains invalid structure.
    pub fn print(self) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();
        let source_size_hint = self.context.source_code().len();
        self.document.print(source_size_hint, print_options)
    }

    /// Prints the formatted document to a string, starting at the given indentation level.
    ///
    /// Finalizes the document first (see [`Self::into_final_document`]).
    ///
    /// # Errors
    /// Returns `PrintError` if the document contains invalid structure.
    pub fn print_with_indent(self, indent: u16) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();
        let source_size_hint = self.context.source_code().len();
        self.document.print_with_indent(source_size_hint, print_options, indent)
    }
}
