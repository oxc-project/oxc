use crate::{FormatContext, format_element::FormatElement, printer::Printer};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Document {
    elements: Vec<FormatElement>,
}

impl Document {
    pub fn new(elements: Vec<FormatElement>) -> Self {
        Self { elements }
    }

    pub fn elements(&self) -> &[FormatElement] {
        &self.elements
    }

    pub fn into_elements(self) -> Vec<FormatElement> {
        self.elements
    }
}

#[derive(Debug)]
pub struct Formatted<'a, Ctx>
where
    Ctx: FormatContext,
{
    document: Document,
    context: &'a Ctx,
}

impl<'a, Ctx> Formatted<'a, Ctx>
where
    Ctx: FormatContext,
{
    pub fn new(document: Document, context: &'a Ctx) -> Self {
        Self { document, context }
    }

    pub fn context(&self) -> &'a Ctx {
        self.context
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn into_document(self) -> Document {
        self.document
    }

    pub fn print(self) -> String {
        let printer = Printer::new(self.context.printer_options());
        printer.print(self.document.elements())
    }
}
